// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Minimal BTSP client handshake for the primal SDK.
//!
//! Delegates all cryptographic operations to the security provider via JSON-RPC,
//! keeping the SDK zero-crypto. The handshake protocol:
//!
//! 1. Client → Server: `ClientHello { protocol: "btsp", version: 1, client_ephemeral_pub }`
//! 2. Server → Client: `ServerHello { challenge, session_id, server_ephemeral_pub }`
//! 3. Client → Server: `ChallengeResponse { response: HMAC-SHA256(...), preferred_cipher }`
//! 4. Server → Client: `HandshakeComplete { session_id, cipher }`

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, trace, warn};

use super::TransportStream;

const BTSP_VERSION: u8 = 1;
const HANDSHAKE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

#[derive(Serialize)]
struct ClientHello {
    protocol: &'static str,
    version: u8,
    client_ephemeral_pub: String,
}

#[derive(Deserialize)]
struct ServerHello {
    challenge: String,
    #[allow(dead_code)]
    session_id: String,
    server_ephemeral_pub: String,
}

#[derive(Serialize)]
struct ChallengeResponse {
    response: String,
    preferred_cipher: &'static str,
}

#[derive(Deserialize)]
struct HandshakeComplete {
    session_id: String,
    #[allow(dead_code)]
    cipher: String,
}

/// Whether BTSP handshake should be performed for this endpoint.
pub(crate) fn should_btsp(path: &Path) -> bool {
    has_family_id() && is_family_scoped(path)
}

/// Perform BTSP handshake on an already-connected stream, returning a `BufReader`
/// ready for JSON-RPC communication.
#[allow(clippy::future_not_send)]
pub(crate) async fn perform_handshake(
    stream: TransportStream,
) -> Result<BufReader<TransportStream>> {
    let provider_path = security_provider_path()
        .ok_or_else(|| anyhow!("BTSP required but security provider socket not found"))?;

    let (client_pub, client_secret) = keygen(&provider_path).await?;

    let hello = ClientHello {
        protocol: "btsp",
        version: BTSP_VERSION,
        client_ephemeral_pub: client_pub,
    };

    let mut reader = BufReader::new(stream);
    write_json_line(&mut reader, &hello).await?;

    let server_hello: ServerHello = read_json_line(&mut reader).await?;

    let response = challenge_response(&provider_path, &client_secret, &server_hello).await?;

    let cr = ChallengeResponse {
        response,
        preferred_cipher: "null",
    };
    write_json_line(&mut reader, &cr).await?;

    let complete: HandshakeComplete = read_json_line(&mut reader).await?;
    debug!(session_id = %complete.session_id, "SDK BTSP handshake complete");

    Ok(reader)
}

fn has_family_id() -> bool {
    std::env::var(biomeos_types::env_config::vars::FAMILY_ID_LEGACY)
        .or_else(|_| std::env::var(biomeos_types::env_config::vars::FAMILY_ID))
        .is_ok_and(|v| !v.is_empty() && v != biomeos_types::defaults::DEFAULT_FAMILY_ID)
}

fn family_id() -> Option<String> {
    std::env::var(biomeos_types::env_config::vars::FAMILY_ID_LEGACY)
        .or_else(|_| std::env::var(biomeos_types::env_config::vars::FAMILY_ID))
        .ok()
        .filter(|v| !v.is_empty() && v != biomeos_types::defaults::DEFAULT_FAMILY_ID)
}

fn is_family_scoped(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or_default();
    ext.eq_ignore_ascii_case("sock") && name.contains('-')
}

fn security_provider_path() -> Option<PathBuf> {
    for env_key in ["BIOMEOS_SECURITY_SOCKET", "SECURITY_PROVIDER_SOCKET"] {
        if let Ok(p) = std::env::var(env_key) {
            let path = PathBuf::from(&p);
            if path.exists() {
                return Some(path);
            }
        }
    }

    let socket_dir = socket_dir()?;
    let fid = family_id();
    let provider = std::env::var(biomeos_types::env_config::vars::SECURITY_PROVIDER)
        .ok()
        .or_else(|| {
            biomeos_types::capability_taxonomy::CapabilityTaxonomy::resolve_to_primal("security")
                .map(String::from)
        })
        .unwrap_or_else(|| biomeos_types::primal_names::BEARDOG.to_string());

    if let Some(fid) = &fid {
        let family_sock = socket_dir.join(format!("{provider}-{fid}.sock"));
        if family_sock.exists() {
            return Some(family_sock);
        }
    }

    let dev_sock = socket_dir.join(format!("{provider}.sock"));
    if dev_sock.exists() {
        return Some(dev_sock);
    }

    None
}

fn socket_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var(biomeos_types::env_config::vars::SOCKET_DIR) {
        return Some(PathBuf::from(dir));
    }
    if let Ok(runtime) = std::env::var(biomeos_types::env_config::vars::XDG_RUNTIME_DIR) {
        let dir = PathBuf::from(runtime)
            .join(biomeos_types::constants::runtime_paths::MEMBRANE_SUBDIR);
        if dir.is_dir() {
            return Some(dir);
        }
    }
    None
}

async fn keygen(provider_path: &Path) -> Result<(String, String)> {
    let request = biomeos_types::JsonRpcRequest::new(
        "x25519_generate_ephemeral",
        serde_json::json!({}),
    );
    let response = provider_call(provider_path, request).await?;
    let pub_key = response["public_key"]
        .as_str()
        .ok_or_else(|| anyhow!("missing public_key from keygen"))?
        .to_owned();
    let secret_key = response["secret_key"]
        .as_str()
        .ok_or_else(|| anyhow!("missing secret_key from keygen"))?
        .to_owned();
    Ok((pub_key, secret_key))
}

async fn challenge_response(
    provider_path: &Path,
    client_secret: &str,
    server_hello: &ServerHello,
) -> Result<String> {
    let derive_request = biomeos_types::JsonRpcRequest::new(
        "crypto.x25519_derive_secret",
        serde_json::json!({
            "secret_key": client_secret,
            "peer_public": server_hello.server_ephemeral_pub,
        }),
    );
    let derive_resp = provider_call(provider_path, derive_request).await?;
    let shared_secret = derive_resp["shared_secret"]
        .as_str()
        .or_else(|| derive_resp["result"].as_str())
        .ok_or_else(|| anyhow!("missing shared_secret from derive"))?;

    let hmac_request = biomeos_types::JsonRpcRequest::new(
        "hmac_sha256",
        serde_json::json!({
            "key": shared_secret,
            "data": server_hello.challenge,
        }),
    );
    let hmac_resp = provider_call(provider_path, hmac_request).await?;
    hmac_resp["hmac"]
        .as_str()
        .or_else(|| hmac_resp["result"].as_str())
        .map(str::to_owned)
        .ok_or_else(|| anyhow!("missing hmac from response"))
}

/// Send a single JSON-RPC call to the security provider (raw, no BTSP).
///
/// Uses `send_jsonrpc_over_stream` directly to avoid recursion — the security
/// provider socket is not family-scoped, so BTSP is never required for it.
async fn provider_call(
    provider_path: &Path,
    request: biomeos_types::JsonRpcRequest,
) -> Result<serde_json::Value> {
    let endpoint = super::TransportEndpoint::UnixSocket {
        path: provider_path.to_path_buf(),
    };
    let stream = super::connect_transport(&endpoint)
        .await
        .with_context(|| format!("Failed to connect to security provider at {}", provider_path.display()))?;
    let response = super::send_jsonrpc_over_stream(stream, request).await?;
    if let Some(err) = response.error {
        return Err(anyhow!("Security provider error {}: {}", err.code, err.message));
    }
    response
        .result
        .ok_or_else(|| anyhow!("Empty response from security provider"))
}

#[allow(clippy::future_not_send)]
async fn write_json_line(
    reader: &mut BufReader<TransportStream>,
    value: &impl Serialize,
) -> Result<()> {
    let mut line = serde_json::to_string(value).context("serialize handshake message")?;
    line.push('\n');
    reader
        .get_mut()
        .write_all(line.as_bytes())
        .await
        .context("write handshake")?;
    reader
        .get_mut()
        .flush()
        .await
        .context("flush handshake")?;
    Ok(())
}

#[allow(clippy::future_not_send)]
async fn read_json_line<T: serde::de::DeserializeOwned>(
    reader: &mut BufReader<TransportStream>,
) -> Result<T> {
    let mut line = String::new();
    let n = tokio::time::timeout(HANDSHAKE_TIMEOUT, reader.read_line(&mut line))
        .await
        .map_err(|_| anyhow!("BTSP handshake timeout"))?
        .context("read handshake line")?;
    if n == 0 {
        return Err(anyhow!("Connection closed during BTSP handshake"));
    }
    trace!("BTSP recv: {}", line.trim());

    #[derive(Deserialize)]
    struct HandshakeError {
        reason: String,
    }
    if let Ok(err) = serde_json::from_str::<HandshakeError>(line.trim()) {
        if !err.reason.is_empty() {
            return Err(anyhow!("BTSP handshake rejected: {}", err.reason));
        }
    }

    serde_json::from_str(line.trim()).context("parse BTSP handshake message")
}

/// Log a warning when BTSP enforcement is active but handshake is skipped.
pub(crate) fn warn_btsp_skipped(path: &Path) {
    warn!(
        path = %path.display(),
        "BTSP handshake skipped — security provider not found. \
         Connection may be rejected by strict-mode primal."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_family_scoped_recognizes_family_sockets() {
        assert!(is_family_scoped(Path::new(
            "/run/membrane/beardog-abc123.sock"
        )));
        assert!(is_family_scoped(Path::new(
            "/tmp/biomeos/songbird-main.sock"
        )));
    }

    #[test]
    fn is_family_scoped_rejects_non_family() {
        assert!(!is_family_scoped(Path::new("/run/membrane/beardog.sock")));
        assert!(!is_family_scoped(Path::new("/tmp/plain.sock")));
        assert!(!is_family_scoped(Path::new("")));
    }

    #[test]
    fn should_btsp_requires_family_id_and_scoped_socket() {
        temp_env::with_vars(
            [
                (biomeos_types::env_config::vars::FAMILY_ID, Some("test-fam")),
                (biomeos_types::env_config::vars::FAMILY_ID_LEGACY, None),
            ],
            || {
                assert!(should_btsp(Path::new("/run/membrane/beardog-abc.sock")));
                assert!(!should_btsp(Path::new("/run/membrane/beardog.sock")));
            },
        );
    }

    #[test]
    fn should_btsp_false_without_family_id() {
        temp_env::with_vars(
            [
                (biomeos_types::env_config::vars::FAMILY_ID, None::<&str>),
                (biomeos_types::env_config::vars::FAMILY_ID_LEGACY, None),
            ],
            || {
                assert!(!should_btsp(Path::new("/run/membrane/beardog-abc.sock")));
            },
        );
    }

    #[test]
    fn should_btsp_ignores_default_family_id() {
        temp_env::with_vars(
            [(
                biomeos_types::env_config::vars::FAMILY_ID,
                Some(biomeos_types::defaults::DEFAULT_FAMILY_ID),
            )],
            || {
                assert!(!should_btsp(Path::new("/run/membrane/beardog-abc.sock")));
            },
        );
    }
}
