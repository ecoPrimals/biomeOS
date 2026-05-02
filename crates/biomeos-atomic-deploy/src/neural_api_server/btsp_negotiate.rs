// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! BTSP Phase 3 — Cipher negotiation and session key derivation.
//!
//! After a successful Phase 2 handshake, the client may send `btsp.negotiate`
//! to upgrade the connection from authenticated-plaintext (NULL cipher) to
//! ChaCha20-Poly1305 encrypted framing.
//!
//! If the client does not negotiate (or requests `"null"`), the connection
//! stays on plaintext NDJSON — backward-compatible with Phase 2 clients.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Supported BTSP cipher suites.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtspCipher {
    Null,
    ChaCha20Poly1305,
}

impl BtspCipher {
    /// Wire name per `BTSP_PROTOCOL_STANDARD.md`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::ChaCha20Poly1305 => "chacha20-poly1305",
        }
    }

    /// Parse a cipher name from the client's `preferred_cipher` field.
    pub fn from_str_loose(s: &str) -> Self {
        let lower = s.to_ascii_lowercase();
        if lower.contains("chacha20") {
            Self::ChaCha20Poly1305
        } else {
            Self::Null
        }
    }
}

/// Per-session state stored after Phase 2 handshake completes.
#[derive(Debug, Clone)]
pub struct BtspSessionState {
    /// Session ID from the handshake (also the HashMap key — retained for
    /// downstream consumers that receive a cloned `BtspSessionState`).
    #[expect(dead_code, reason = "stored for downstream session introspection")]
    pub session_id: String,

    /// Negotiated cipher (starts as Null, upgraded by `btsp.negotiate`).
    pub cipher: BtspCipher,

    /// Server nonce generated during negotiate (hex-encoded).
    pub server_nonce: Option<String>,
}

/// Thread-safe store of active BTSP sessions, keyed by `session_id`.
pub type BtspSessionStore = Arc<RwLock<HashMap<String, BtspSessionState>>>;

/// Create a new empty session store.
pub fn new_session_store() -> BtspSessionStore {
    Arc::new(RwLock::new(HashMap::new()))
}

/// Register a session after successful Phase 2 handshake.
pub async fn register_session(store: &BtspSessionStore, session_id: String) {
    let mut sessions = store.write().await;
    sessions.insert(
        session_id.clone(),
        BtspSessionState {
            session_id,
            cipher: BtspCipher::Null,
            server_nonce: None,
        },
    );
}

/// Handle `btsp.negotiate` JSON-RPC method.
///
/// Validates the session, generates a server nonce, and returns the negotiated
/// cipher. If the requested cipher is not supported, falls back to `"null"`.
pub async fn handle_negotiate(
    store: &BtspSessionStore,
    params: &serde_json::Value,
) -> Result<serde_json::Value, anyhow::Error> {
    let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("btsp.negotiate requires 'session_id' parameter"))?;

    let preferred = params
        .get("preferred_cipher")
        .and_then(|v| v.as_str())
        .unwrap_or("null");

    let cipher = BtspCipher::from_str_loose(preferred);

    let server_nonce = generate_server_nonce();

    let mut sessions = store.write().await;
    let session = sessions
        .get_mut(session_id)
        .ok_or_else(|| anyhow::anyhow!("Unknown session_id: {session_id}"))?;

    session.cipher = cipher;
    session.server_nonce = Some(server_nonce.clone());

    Ok(serde_json::json!({
        "cipher": cipher.as_str(),
        "server_nonce": server_nonce,
        "allowed": true,
    }))
}

/// Generate a random 12-byte nonce, hex-encoded.
fn generate_server_nonce() -> String {
    use rand::Rng;
    let mut nonce = [0u8; 12];
    rand::rng().fill(&mut nonce);
    hex_encode(&nonce)
}

fn hex_encode(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cipher_from_str() {
        assert_eq!(
            BtspCipher::from_str_loose("chacha20-poly1305"),
            BtspCipher::ChaCha20Poly1305
        );
        assert_eq!(
            BtspCipher::from_str_loose("ChaCha20_Poly1305"),
            BtspCipher::ChaCha20Poly1305
        );
        assert_eq!(BtspCipher::from_str_loose("null"), BtspCipher::Null);
        assert_eq!(BtspCipher::from_str_loose("aes-gcm"), BtspCipher::Null);
    }

    #[test]
    fn test_cipher_as_str() {
        assert_eq!(BtspCipher::Null.as_str(), "null");
        assert_eq!(BtspCipher::ChaCha20Poly1305.as_str(), "chacha20-poly1305");
    }

    #[test]
    fn test_server_nonce_format() {
        let nonce = generate_server_nonce();
        assert_eq!(nonce.len(), 24); // 12 bytes × 2 hex chars
        assert!(nonce.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[tokio::test]
    async fn test_register_and_negotiate() {
        let store = new_session_store();

        register_session(&store, "test-session-123".to_string()).await;

        let params = serde_json::json!({
            "session_id": "test-session-123",
            "preferred_cipher": "chacha20-poly1305",
            "bond_type": "Covalent"
        });

        let result = handle_negotiate(&store, &params).await.unwrap();
        assert_eq!(result["cipher"], "chacha20-poly1305");
        assert_eq!(result["allowed"], true);
        assert!(result["server_nonce"].as_str().unwrap().len() == 24);
    }

    #[tokio::test]
    async fn test_negotiate_unknown_session() {
        let store = new_session_store();

        let params = serde_json::json!({
            "session_id": "nonexistent",
            "preferred_cipher": "chacha20-poly1305"
        });

        let result = handle_negotiate(&store, &params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_negotiate_null_fallback() {
        let store = new_session_store();
        register_session(&store, "sess-1".to_string()).await;

        let params = serde_json::json!({
            "session_id": "sess-1",
            "preferred_cipher": "aes-256-gcm"
        });

        let result = handle_negotiate(&store, &params).await.unwrap();
        assert_eq!(result["cipher"], "null");
        assert_eq!(result["allowed"], true);
    }

    #[tokio::test]
    async fn test_negotiate_missing_session_id() {
        let store = new_session_store();

        let params = serde_json::json!({
            "preferred_cipher": "chacha20-poly1305"
        });

        let result = handle_negotiate(&store, &params).await;
        assert!(result.is_err());
    }
}
