// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use serde::{Deserialize, Serialize};

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
    /// FAMILY_ID and BIOMEOS_INSECURE=1 set simultaneously.
    #[error(
        "FAMILY_ID and BIOMEOS_INSECURE=1 cannot coexist — production mode requires BTSP authentication"
    )]
    InsecureGuard,
}
