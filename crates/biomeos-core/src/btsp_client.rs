// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! BTSP Client — biomeOS-side handshake for Secure Socket Architecture.
//!
//! When biomeOS connects to a family-scoped primal socket (`{primal}-{fid}.sock`),
//! it MUST perform a BTSP handshake to prove family membership before sending
//! any JSON-RPC requests.
//!
//! This module provides:
//! - Detection of family-scoped sockets (BTSP-required vs development-mode)
//! - BTSP session state tracking
//! - The INSECURE guard (refuse to run with both `FAMILY_ID` and `BIOMEOS_INSECURE`)
//! - Phase 2 server-side handshake enforcement for UDS listeners
//! - Phase 2 client-side handshake initiation for outbound forwarding
//!
//! The actual cryptographic handshake is delegated to the security provider
//! via JSON-RPC (`btsp.session.create`, `btsp.session.verify`). biomeOS is a
//! family member and holds the family seed for key derivation.

use biomeos_types::defaults::DEFAULT_FAMILY_ID;
use biomeos_types::primal_names;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tracing::{debug, info, warn};

/// BTSP protocol version implemented by this module.
pub const BTSP_VERSION: u8 = 1;

/// Security mode for biomeOS socket connections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityMode {
    /// Production: `FAMILY_ID` is set. BTSP handshake required for family-scoped sockets.
    Production {
        /// Whether a security provider socket is reachable for handshake delegation.
        btsp_available: bool,
    },
    /// Development: `BIOMEOS_INSECURE=1` or no `FAMILY_ID`. Raw cleartext JSON-RPC.
    Development,
}

/// Outcome of a BTSP handshake attempt.
#[derive(Debug, Clone)]
pub enum HandshakeOutcome {
    /// Handshake succeeded; session_id is available for optional encryption.
    Authenticated {
        /// Opaque session identifier returned by the security provider.
        session_id: String,
        /// Session key from the security provider's `btsp.session.verify` response.
        /// `None` if the provider didn't return key material (older versions).
        /// Used by Phase 3 HKDF key derivation.
        handshake_key: Option<[u8; 32]>,
    },
    /// No FAMILY_ID set — connection accepted without handshake (dev mode).
    DevMode,
    /// FAMILY_ID is set but security provider is unavailable — behaviour
    /// depends on `BIOMEOS_BTSP_ENFORCE`.
    SecurityProviderUnavailable,
}

// ── BTSP Handshake Wire Types (Phase 2) ────────────────────────────────

/// First message from client → server on a new connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientHello {
    /// Always `"btsp"`.
    pub protocol: String,
    /// Protocol version (currently 1).
    pub version: u8,
    /// Base64-encoded X25519 ephemeral public key.
    pub client_ephemeral_pub: String,
}

/// Server → client: challenge after receiving `ClientHello`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerHello {
    /// Protocol version (must match `BTSP_VERSION`).
    pub version: u8,
    /// Base64-encoded X25519 ephemeral public key (from security provider).
    pub server_ephemeral_pub: String,
    /// Base64-encoded random 32-byte challenge.
    pub challenge: String,
    /// Session ID for security provider delegation.
    pub session_id: String,
}

/// Client → server: HMAC response to the challenge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResponse {
    /// Base64-encoded HMAC-SHA256 of (challenge ‖ client_pub ‖ server_pub).
    pub response: String,
    /// Preferred cipher suite (e.g. `"chacha20_poly1305"`, `"null"`).
    #[serde(default = "default_cipher")]
    pub preferred_cipher: String,
}

fn default_cipher() -> String {
    "null".to_owned()
}

/// Server → client: handshake succeeded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeComplete {
    /// Negotiated cipher suite for this session.
    pub cipher: String,
    /// Session identifier (matches `ServerHello::session_id`).
    pub session_id: String,
}

/// Server → client: handshake failed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeError {
    /// Always `"handshake_failed"`.
    pub error: String,
    /// Diagnostic reason (e.g. `"family_verification"`).
    pub reason: String,
}

// ── Environment helpers ────────────────────────────────────────────────

/// Determine the security mode from environment.
#[must_use]
pub fn security_mode() -> SecurityMode {
    let has_family = has_family_id();

    if has_family {
        let btsp_available = security_provider_socket_path().is_some();
        SecurityMode::Production { btsp_available }
    } else {
        SecurityMode::Development
    }
}

/// Whether `FAMILY_ID` (or `BIOMEOS_FAMILY_ID`) is set to a non-default value.
#[must_use]
pub fn has_family_id() -> bool {
    std::env::var("FAMILY_ID")
        .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
        .map(|v| !v.is_empty() && v != DEFAULT_FAMILY_ID)
        .unwrap_or(false)
}

/// Read the family ID string from environment.
#[must_use]
pub fn family_id() -> Option<String> {
    std::env::var("FAMILY_ID")
        .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
        .ok()
        .filter(|v| !v.is_empty() && v != DEFAULT_FAMILY_ID)
}

/// Whether BTSP enforcement is active. When `true`, connections from
/// clients that do not complete a BTSP handshake are rejected. When
/// `false`, unauthenticated connections log a warning but proceed.
///
/// Default: `true` when `FAMILY_ID` is set, `false` otherwise.
/// Override: `BIOMEOS_BTSP_ENFORCE=0` disables enforcement during rollout.
#[must_use]
pub fn btsp_enforce() -> bool {
    if !has_family_id() {
        return false;
    }
    std::env::var("BIOMEOS_BTSP_ENFORCE")
        .map(|v| v != "0" && v != "false")
        .unwrap_or(true)
}

/// Locate the security provider socket for BTSP delegation.
///
/// The security provider is resolved via `BIOMEOS_SECURITY_PROVIDER` (defaulting
/// to the canonical `beardog` constant). This function does not hardcode which
/// primal provides security — it discovers the socket by provider name.
///
/// Resolution order:
/// 1. `BIOMEOS_SECURITY_SOCKET` environment variable (explicit path)
/// 2. `BEARDOG_SOCKET` / `BIOMEOS_BEARDOG_SOCKET` (legacy compat)
/// 3. Family-scoped socket `{provider}-{fid}.sock` in socket dir
/// 4. Development socket `{provider}.sock` in socket dir
#[must_use]
pub fn security_provider_socket_path() -> Option<std::path::PathBuf> {
    for env_key in [
        "BIOMEOS_SECURITY_SOCKET",
        "BEARDOG_SOCKET",
        "BIOMEOS_BEARDOG_SOCKET",
    ] {
        if let Ok(p) = std::env::var(env_key) {
            let path = std::path::PathBuf::from(&p);
            if path.exists() {
                return Some(path);
            }
        }
    }

    let provider = std::env::var("BIOMEOS_SECURITY_PROVIDER")
        .unwrap_or_else(|_| primal_names::BEARDOG.to_string());

    let socket_dir = socket_dir()?;
    if let Some(fid) = family_id() {
        let family_path = socket_dir.join(format!("{provider}-{fid}.sock"));
        if family_path.exists() {
            return Some(family_path);
        }
    }
    let dev_path = socket_dir.join(format!("{provider}.sock"));
    if dev_path.exists() {
        return Some(dev_path);
    }
    None
}

fn socket_dir() -> Option<std::path::PathBuf> {
    if let Ok(dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
        return Some(std::path::PathBuf::from(dir));
    }
    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
        let dir = std::path::PathBuf::from(runtime).join("biomeos");
        if dir.is_dir() {
            return Some(dir);
        }
    }
    None
}

// ── Phase 2: Server-side handshake (UDS listener) ──────────────────────

/// Perform the server-side BTSP handshake on an accepted connection.
///
/// Reads the first line from the stream. If it is a `ClientHello`, delegates
/// the crypto to the security provider and completes the 4-step handshake. If the first
/// line is a raw JSON-RPC request, the line is returned so the caller can
/// dispatch it without data loss.
///
/// # Returns
///
/// - `Ok(HandshakeOutcome::Authenticated { .. })` — handshake completed.
/// - `Ok(HandshakeOutcome::DevMode)` — no `FAMILY_ID`, skipped.
/// - `Ok(HandshakeOutcome::SecurityProviderUnavailable)` — `FAMILY_ID` set
///   but security provider unreachable. The caller should check [`btsp_enforce`] to
///   decide whether to accept or reject the connection.
/// - `Err(_)` — handshake failed (client not in family, protocol error).
///
/// When the first line is **not** a `ClientHello`, it is returned inside the
/// error variant `BtspHandshakeError::RawJsonRpc` so the caller can replay it.
pub async fn server_handshake<S>(
    reader: &mut BufReader<S>,
) -> Result<HandshakeOutcome, BtspHandshakeError>
where
    S: tokio::io::AsyncRead + AsyncWrite + Unpin,
{
    if !has_family_id() {
        return Ok(HandshakeOutcome::DevMode);
    }

    let mut first_line = String::new();
    let read = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        reader.read_line(&mut first_line),
    )
    .await
    .map_err(|_| BtspHandshakeError::Timeout)?
    .map_err(BtspHandshakeError::Io)?;

    if read == 0 {
        return Err(BtspHandshakeError::ConnectionClosed);
    }

    let hello: ClientHello = match serde_json::from_str::<ClientHello>(first_line.trim()) {
        Ok(h) if h.protocol == "btsp" => h,
        _ => {
            return Err(BtspHandshakeError::RawJsonRpc(first_line));
        }
    };

    debug!(
        version = hello.version,
        "BTSP ClientHello received, delegating to security provider"
    );

    let provider_path =
        security_provider_socket_path().ok_or(BtspHandshakeError::SecurityProviderNotFound)?;

    let session =
        create_session_via_security_provider(&provider_path, &hello.client_ephemeral_pub).await?;

    let server_hello = ServerHello {
        version: BTSP_VERSION,
        server_ephemeral_pub: session.server_ephemeral_pub.clone(),
        challenge: session.challenge.clone(),
        session_id: session.session_id.clone(),
    };

    let mut hello_line = serde_json::to_string(&server_hello)
        .map_err(|e| BtspHandshakeError::Protocol(e.to_string()))?;
    hello_line.push('\n');
    let stream = reader.get_mut();
    stream
        .write_all(hello_line.as_bytes())
        .await
        .map_err(BtspHandshakeError::Io)?;
    stream.flush().await.map_err(BtspHandshakeError::Io)?;

    let mut response_line = String::new();
    let read = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        reader.read_line(&mut response_line),
    )
    .await
    .map_err(|_| BtspHandshakeError::Timeout)?
    .map_err(BtspHandshakeError::Io)?;

    if read == 0 {
        return Err(BtspHandshakeError::ConnectionClosed);
    }

    let challenge_resp: ChallengeResponse = serde_json::from_str(response_line.trim())
        .map_err(|e| BtspHandshakeError::Protocol(format!("invalid ChallengeResponse: {e}")))?;

    let verify_result = verify_session_via_security_provider(
        &provider_path,
        &session.session_id,
        &challenge_resp.response,
        &hello.client_ephemeral_pub,
        &session.server_ephemeral_pub,
        &session.challenge,
    )
    .await?;

    if !verify_result.verified {
        let err = HandshakeError {
            error: "handshake_failed".to_owned(),
            reason: "family_verification".to_owned(),
        };
        let mut err_line = serde_json::to_string(&err).unwrap_or_default();
        err_line.push('\n');
        let stream = reader.get_mut();
        let _ = stream.write_all(err_line.as_bytes()).await;
        let _ = stream.flush().await;
        return Err(BtspHandshakeError::VerificationFailed);
    }

    let complete = HandshakeComplete {
        cipher: challenge_resp.preferred_cipher,
        session_id: session.session_id.clone(),
    };
    let mut complete_line = serde_json::to_string(&complete)
        .map_err(|e| BtspHandshakeError::Protocol(e.to_string()))?;
    complete_line.push('\n');
    let stream = reader.get_mut();
    stream
        .write_all(complete_line.as_bytes())
        .await
        .map_err(BtspHandshakeError::Io)?;
    stream.flush().await.map_err(BtspHandshakeError::Io)?;

    debug!(session_id = %session.session_id, "BTSP handshake complete");

    Ok(HandshakeOutcome::Authenticated {
        session_id: session.session_id,
        handshake_key: verify_result.handshake_key,
    })
}

/// Errors during BTSP handshake.
#[derive(Debug, thiserror::Error)]
pub enum BtspHandshakeError {
    /// First line was a raw JSON-RPC request, not a `ClientHello`. The line
    /// content is preserved so the caller can dispatch it as a normal request.
    #[error("client sent raw JSON-RPC (no BTSP handshake)")]
    RawJsonRpc(String),
    /// Security provider socket not found — cannot delegate crypto.
    #[error("security provider socket not found for BTSP delegation")]
    SecurityProviderNotFound,
    /// Security provider returned an error during session creation or verification.
    #[error("BTSP security provider error: {0}")]
    SecurityProviderError(String),
    /// Client failed family verification.
    #[error("BTSP family verification failed")]
    VerificationFailed,
    /// Wire protocol error (malformed message, serialization failure).
    #[error("BTSP protocol error: {0}")]
    Protocol(String),
    /// Handshake timed out.
    #[error("BTSP handshake timed out")]
    Timeout,
    /// Client disconnected during handshake.
    #[error("client disconnected during BTSP handshake")]
    ConnectionClosed,
    /// I/O error on the connection.
    #[error("BTSP I/O error: {0}")]
    Io(std::io::Error),
}

// ── Security provider RPC delegation helpers ──────────────────────────

struct BtspSession {
    session_id: String,
    server_ephemeral_pub: String,
    challenge: String,
}

async fn create_session_via_security_provider(
    provider_path: &Path,
    client_ephemeral_pub: &str,
) -> Result<BtspSession, BtspHandshakeError> {
    use crate::AtomicClient;

    let client = AtomicClient::unix(provider_path);
    let result = client
        .call(
            "btsp.session.create",
            serde_json::json!({
                "family_seed_ref": "env:FAMILY_SEED",
                "client_ephemeral_pub": client_ephemeral_pub,
            }),
        )
        .await
        .map_err(|e| BtspHandshakeError::SecurityProviderError(e.to_string()))?;

    Ok(BtspSession {
        session_id: result["session_id"].as_str().unwrap_or_default().to_owned(),
        server_ephemeral_pub: result["server_ephemeral_pub"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
        challenge: result["challenge"].as_str().unwrap_or_default().to_owned(),
    })
}

/// Result of session verification including optional key material.
struct VerifyResult {
    verified: bool,
    /// 32-byte session key from the security provider, if provided.
    handshake_key: Option<[u8; 32]>,
}

async fn verify_session_via_security_provider(
    provider_path: &Path,
    session_id: &str,
    client_response: &str,
    client_ephemeral_pub: &str,
    server_ephemeral_pub: &str,
    challenge: &str,
) -> Result<VerifyResult, BtspHandshakeError> {
    use crate::AtomicClient;

    let client = AtomicClient::unix(provider_path);
    let result = client
        .call(
            "btsp.session.verify",
            serde_json::json!({
                "session_id": session_id,
                "client_response": client_response,
                "client_ephemeral_pub": client_ephemeral_pub,
                "server_ephemeral_pub": server_ephemeral_pub,
                "challenge": challenge,
            }),
        )
        .await
        .map_err(|e| BtspHandshakeError::SecurityProviderError(e.to_string()))?;

    let verified = result["verified"].as_bool().unwrap_or(false);

    let handshake_key = result
        .get("session_key")
        .and_then(|v| v.as_str())
        .and_then(|hex| {
            let bytes: Vec<u8> = (0..hex.len())
                .step_by(2)
                .filter_map(|i| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok())
                .collect();
            <[u8; 32]>::try_from(bytes.as_slice()).ok()
        });

    Ok(VerifyResult {
        verified,
        handshake_key,
    })
}

/// Check that `FAMILY_ID` and `BIOMEOS_INSECURE` are not both set.
///
/// biomeOS MUST call this at startup before binding any sockets or connecting
/// to primals.
///
/// # Errors
///
/// Returns a human-readable error message when both are set.
pub fn validate_insecure_guard() -> Result<(), String> {
    let has_family = std::env::var("FAMILY_ID")
        .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
        .map(|v| !v.is_empty() && v != DEFAULT_FAMILY_ID)
        .unwrap_or(false);
    let insecure = std::env::var("BIOMEOS_INSECURE")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false);

    if has_family && insecure {
        return Err("FATAL: FAMILY_ID and BIOMEOS_INSECURE=1 cannot coexist. \
             Production mode (FAMILY_ID set) requires BTSP authentication. \
             Remove BIOMEOS_INSECURE to run in production, or unset FAMILY_ID for development."
            .to_owned());
    }
    Ok(())
}

/// Detect whether a socket path is family-scoped (requires BTSP handshake).
///
/// Family-scoped sockets match the pattern `{primal}-{family_id}.sock`.
/// Non-family sockets are `{primal}.sock` (development mode).
#[must_use]
pub fn is_family_scoped_socket(path: &Path) -> bool {
    let Some(filename) = path.file_name().and_then(|f| f.to_str()) else {
        return false;
    };
    let Some(stem) = filename.strip_suffix(".sock") else {
        return false;
    };
    // Family-scoped: `{canonical_primal_id}-{family_id}.sock` (see `primal_names`).
    stem.contains('-') && stem.split('-').count() >= 2
}

/// Extract the family ID from a family-scoped socket path.
///
/// Returns `None` if the socket is not family-scoped.
#[must_use]
pub fn extract_family_id(path: &Path) -> Option<String> {
    let filename = path.file_name()?.to_str()?;
    let stem = filename.strip_suffix(".sock")?;
    let dash_pos = stem.find('-')?;
    Some(stem[dash_pos + 1..].to_owned())
}

/// Log the security posture at startup.
pub fn log_security_posture() {
    match security_mode() {
        SecurityMode::Production { .. } => {
            let fid = std::env::var("FAMILY_ID")
                .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
                .unwrap_or_else(|_| "unknown".to_owned());
            info!(
                family_id = %fid,
                mode = "production",
                "Secure Socket Architecture: BTSP authentication required for all primal connections"
            );
        }
        SecurityMode::Development => {
            let insecure = std::env::var("BIOMEOS_INSECURE")
                .map(|v| v == "1" || v == "true")
                .unwrap_or(false);
            if insecure {
                warn!("INSECURE MODE — no BTSP authentication. Development only.");
            } else {
                info!(
                    mode = "standalone",
                    "No FAMILY_ID set — running in development/standalone mode"
                );
            }
        }
    }
}

// ── Phase 2: Client-side handshake (outbound forwarding) ───────────────

/// Perform a client-side BTSP handshake on an already-connected Unix
/// stream, delegating all cryptographic operations to the security provider.
///
/// Returns the stream wrapped in a `BufReader` so the caller can
/// immediately send JSON-RPC lines over the authenticated channel.
///
/// # Errors
///
/// Returns `BtspHandshakeError` when the security provider is unreachable, the remote
/// primal rejects the handshake, or a timeout occurs.
pub async fn perform_client_handshake(
    stream: tokio::net::UnixStream,
) -> Result<tokio::io::BufReader<tokio::net::UnixStream>, BtspHandshakeError> {
    let provider_path =
        security_provider_socket_path().ok_or(BtspHandshakeError::SecurityProviderNotFound)?;
    let bd = crate::AtomicClient::unix(&provider_path);

    let (client_pub, client_secret) = client_keygen(&bd).await?;

    let hello = ClientHello {
        protocol: "btsp".into(),
        version: BTSP_VERSION,
        client_ephemeral_pub: client_pub,
    };
    let hello_line = serialize_line(&hello)?;

    let mut reader = tokio::io::BufReader::new(stream);
    write_line_to(&mut reader, &hello_line).await?;

    let server_hello: ServerHello = read_json_line(&mut reader).await?;
    let response = client_challenge_response(&bd, &client_secret, &server_hello).await?;

    let cr = ChallengeResponse {
        response,
        preferred_cipher: "null".into(),
    };
    let cr_line = serialize_line(&cr)?;
    write_line_to(&mut reader, &cr_line).await?;

    let complete: HandshakeComplete = read_json_line(&mut reader).await?;
    debug!(session_id = %complete.session_id, "BTSP client handshake complete");

    Ok(reader)
}

async fn client_keygen(bd: &crate::AtomicClient) -> Result<(String, String), BtspHandshakeError> {
    let kp = bd
        .call("x25519_generate_ephemeral", serde_json::json!({}))
        .await
        .map_err(|e| BtspHandshakeError::SecurityProviderError(format!("keygen: {e}")))?;
    let pub_key = kp["public_key"]
        .as_str()
        .ok_or_else(|| BtspHandshakeError::Protocol("missing public_key".into()))?
        .to_owned();
    let sec_key = kp["secret_key"]
        .as_str()
        .ok_or_else(|| BtspHandshakeError::Protocol("missing secret_key".into()))?
        .to_owned();
    Ok((pub_key, sec_key))
}

async fn client_challenge_response(
    bd: &crate::AtomicClient,
    client_secret: &str,
    server_hello: &ServerHello,
) -> Result<String, BtspHandshakeError> {
    let shared = bd
        .call(
            "crypto.x25519_derive_secret",
            serde_json::json!({
                "secret_key": client_secret,
                "peer_public": server_hello.server_ephemeral_pub,
            }),
        )
        .await
        .map_err(|e| BtspHandshakeError::SecurityProviderError(format!("derive: {e}")))?;
    let shared_secret = shared["shared_secret"]
        .as_str()
        .or_else(|| shared["result"].as_str())
        .ok_or_else(|| BtspHandshakeError::Protocol("missing shared_secret from derive".into()))?;
    let hmac_result = bd
        .call(
            "hmac_sha256",
            serde_json::json!({ "key": shared_secret, "data": server_hello.challenge }),
        )
        .await
        .map_err(|e| BtspHandshakeError::SecurityProviderError(format!("hmac: {e}")))?;
    hmac_result["hmac"]
        .as_str()
        .or_else(|| hmac_result["result"].as_str())
        .map(str::to_owned)
        .ok_or_else(|| BtspHandshakeError::Protocol("missing hmac from response".into()))
}

fn serialize_line(value: &impl serde::Serialize) -> Result<String, BtspHandshakeError> {
    let mut s =
        serde_json::to_string(value).map_err(|e| BtspHandshakeError::Protocol(e.to_string()))?;
    s.push('\n');
    Ok(s)
}

async fn write_line_to(
    reader: &mut tokio::io::BufReader<tokio::net::UnixStream>,
    data: &str,
) -> Result<(), BtspHandshakeError> {
    use tokio::io::AsyncWriteExt;
    reader
        .get_mut()
        .write_all(data.as_bytes())
        .await
        .map_err(BtspHandshakeError::Io)?;
    reader
        .get_mut()
        .flush()
        .await
        .map_err(BtspHandshakeError::Io)
}

async fn read_json_line<T: serde::de::DeserializeOwned>(
    reader: &mut tokio::io::BufReader<tokio::net::UnixStream>,
) -> Result<T, BtspHandshakeError> {
    use tokio::io::AsyncBufReadExt;
    let mut line = String::new();
    let n = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        reader.read_line(&mut line),
    )
    .await
    .map_err(|_| BtspHandshakeError::Timeout)?
    .map_err(BtspHandshakeError::Io)?;
    if n == 0 {
        return Err(BtspHandshakeError::ConnectionClosed);
    }
    if let Ok(err) = serde_json::from_str::<HandshakeError>(line.trim()) {
        return Err(BtspHandshakeError::Protocol(format!(
            "handshake rejected: {}",
            err.reason
        )));
    }
    serde_json::from_str(line.trim())
        .map_err(|e| BtspHandshakeError::Protocol(format!("parse: {e}")))
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
#[path = "btsp_client_tests.rs"]
mod tests;
