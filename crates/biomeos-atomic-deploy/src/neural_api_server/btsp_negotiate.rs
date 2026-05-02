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

/// Directional session keys derived via HKDF-SHA256 after Phase 3 negotiate.
///
/// The server uses `server_to_client` to encrypt outgoing frames and
/// `client_to_server` to decrypt incoming frames.
#[derive(Clone)]
#[allow(
    dead_code,
    reason = "keys consumed by encrypted framing layer (Phase 3 wire evolution)"
)]
pub struct SessionKeys {
    pub client_to_server: [u8; 32],
    pub server_to_client: [u8; 32],
}

impl std::fmt::Debug for SessionKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionKeys")
            .field("client_to_server", &"[REDACTED]")
            .field("server_to_client", &"[REDACTED]")
            .finish()
    }
}

/// Derive directional session keys via HKDF-SHA256.
///
/// ```text
/// salt = client_nonce || server_nonce
/// c2s = HKDF-SHA256(ikm=handshake_key, salt, info="btsp-session-v1-c2s")
/// s2c = HKDF-SHA256(ikm=handshake_key, salt, info="btsp-session-v1-s2c")
/// ```
pub fn derive_session_keys(
    handshake_key: &[u8; 32],
    client_nonce: &[u8],
    server_nonce: &[u8],
) -> SessionKeys {
    use hkdf::Hkdf;
    use sha2::Sha256;

    let mut salt = Vec::with_capacity(client_nonce.len() + server_nonce.len());
    salt.extend_from_slice(client_nonce);
    salt.extend_from_slice(server_nonce);

    let hk = Hkdf::<Sha256>::new(Some(&salt), handshake_key);

    let mut c2s = [0u8; 32];
    // HKDF-SHA256 expand for 32 bytes is infallible (max OKM = 255 * 32 = 8160).
    let _ = hk.expand(b"btsp-session-v1-c2s", &mut c2s);

    let mut s2c = [0u8; 32];
    let _ = hk.expand(b"btsp-session-v1-s2c", &mut s2c);

    SessionKeys {
        client_to_server: c2s,
        server_to_client: s2c,
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

    /// Handshake key from BearDog's `btsp.session.verify` response.
    /// `None` when BearDog didn't return key material (older versions).
    handshake_key: Option<[u8; 32]>,

    /// Derived session keys (populated after successful Phase 3 negotiate
    /// when both `handshake_key` and `client_nonce` are available).
    pub session_keys: Option<SessionKeys>,
}

/// Thread-safe store of active BTSP sessions, keyed by `session_id`.
pub type BtspSessionStore = Arc<RwLock<HashMap<String, BtspSessionState>>>;

/// Create a new empty session store.
pub fn new_session_store() -> BtspSessionStore {
    Arc::new(RwLock::new(HashMap::new()))
}

/// Register a session after successful Phase 2 handshake.
pub async fn register_session(
    store: &BtspSessionStore,
    session_id: impl Into<String>,
    handshake_key: Option<[u8; 32]>,
) {
    let id = session_id.into();
    let mut sessions = store.write().await;
    sessions.insert(
        id.clone(),
        BtspSessionState {
            session_id: id,
            cipher: BtspCipher::Null,
            server_nonce: None,
            handshake_key,
            session_keys: None,
        },
    );
}

/// Handle `btsp.negotiate` JSON-RPC method.
///
/// Validates the session, generates a server nonce, and returns the negotiated
/// cipher. If the requested cipher is not supported, falls back to `"null"`.
/// When both `handshake_key` (from Phase 2) and `client_nonce` (from params)
/// are available, derives directional session keys via HKDF-SHA256.
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

    let client_nonce_hex = params
        .get("client_nonce")
        .and_then(|v| v.as_str())
        .map(String::from);

    let mut cipher = BtspCipher::from_str_loose(preferred);

    let server_nonce = generate_server_nonce();
    let server_nonce_bytes = hex_decode(&server_nonce);

    let mut sessions = store.write().await;
    let session = sessions
        .get_mut(session_id)
        .ok_or_else(|| anyhow::anyhow!("Unknown session_id: {session_id}"))?;

    if cipher == BtspCipher::ChaCha20Poly1305 {
        if let (Some(hk), Some(cn_hex)) = (&session.handshake_key, &client_nonce_hex) {
            let client_nonce_bytes = hex_decode(cn_hex);
            let keys = derive_session_keys(hk, &client_nonce_bytes, &server_nonce_bytes);
            session.session_keys = Some(keys);
        } else {
            cipher = BtspCipher::Null;
        }
    }

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

fn hex_decode(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok())
        .collect()
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
    async fn test_register_and_negotiate_with_key() {
        let store = new_session_store();
        let fake_key = [0xABu8; 32];
        register_session(&store, "test-session-123", Some(fake_key)).await;

        let params = serde_json::json!({
            "session_id": "test-session-123",
            "preferred_cipher": "chacha20-poly1305",
            "client_nonce": "aabbccdd11223344aabbccdd",
            "bond_type": "Covalent"
        });

        let result = handle_negotiate(&store, &params).await.unwrap();
        assert_eq!(result["cipher"], "chacha20-poly1305");
        assert_eq!(result["allowed"], true);
        assert!(result["server_nonce"].as_str().unwrap().len() == 24);

        let sessions = store.read().await;
        let sess = sessions.get("test-session-123").unwrap();
        assert!(sess.session_keys.is_some());
    }

    #[tokio::test]
    async fn test_negotiate_no_handshake_key_falls_back_to_null() {
        let store = new_session_store();
        register_session(&store, "sess-no-key", None).await;

        let params = serde_json::json!({
            "session_id": "sess-no-key",
            "preferred_cipher": "chacha20-poly1305",
            "client_nonce": "aabbccdd11223344aabbccdd"
        });

        let result = handle_negotiate(&store, &params).await.unwrap();
        assert_eq!(result["cipher"], "null");

        let sessions = store.read().await;
        assert!(sessions.get("sess-no-key").unwrap().session_keys.is_none());
    }

    #[tokio::test]
    async fn test_negotiate_no_client_nonce_falls_back_to_null() {
        let store = new_session_store();
        let fake_key = [0xBBu8; 32];
        register_session(&store, "sess-no-nonce", Some(fake_key)).await;

        let params = serde_json::json!({
            "session_id": "sess-no-nonce",
            "preferred_cipher": "chacha20-poly1305"
        });

        let result = handle_negotiate(&store, &params).await.unwrap();
        assert_eq!(result["cipher"], "null");
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
        register_session(&store, "sess-1", None).await;

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

    #[test]
    fn test_derive_session_keys_deterministic() {
        let key = [0xCCu8; 32];
        let cn = [1u8; 12];
        let sn = [2u8; 12];

        let k1 = derive_session_keys(&key, &cn, &sn);
        let k2 = derive_session_keys(&key, &cn, &sn);
        assert_eq!(k1.client_to_server, k2.client_to_server);
        assert_eq!(k1.server_to_client, k2.server_to_client);
    }

    #[test]
    fn test_derive_session_keys_directional() {
        let key = [0xDDu8; 32];
        let cn = [3u8; 12];
        let sn = [4u8; 12];

        let keys = derive_session_keys(&key, &cn, &sn);
        assert_ne!(keys.client_to_server, keys.server_to_client);
    }

    #[test]
    fn test_derive_session_keys_different_nonces_produce_different_keys() {
        let key = [0xEEu8; 32];
        let cn1 = [5u8; 12];
        let cn2 = [6u8; 12];
        let sn = [7u8; 12];

        let k1 = derive_session_keys(&key, &cn1, &sn);
        let k2 = derive_session_keys(&key, &cn2, &sn);
        assert_ne!(k1.client_to_server, k2.client_to_server);
    }

    #[test]
    fn test_session_keys_debug_redacted() {
        let keys = derive_session_keys(&[0u8; 32], &[1u8; 12], &[2u8; 12]);
        let dbg = format!("{keys:?}");
        assert!(dbg.contains("REDACTED"));
        assert!(!dbg.contains(&format!("{:02x}", keys.client_to_server[0])));
    }

    #[test]
    fn test_hex_roundtrip() {
        let original = [0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x23];
        let encoded = hex_encode(&original);
        let decoded = hex_decode(&encoded);
        assert_eq!(&original[..], &decoded[..]);
    }
}
