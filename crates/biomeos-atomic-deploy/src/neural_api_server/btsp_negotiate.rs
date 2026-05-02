// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! BTSP Phase 3 — Cipher negotiation, HKDF key derivation, and encrypted framing.
//!
//! After a successful Phase 2 handshake, the client may send `btsp.negotiate`
//! to upgrade the connection from authenticated-plaintext (NULL cipher) to
//! ChaCha20-Poly1305 encrypted framing.
//!
//! Nonces and keys are base64-encoded on the wire (aligned with the security provider,
//! sweetGrass, and primalSpring). Hex-encoded `client_nonce` is auto-detected
//! for backward compatibility with barraCuda-style clients.
//!
//! If the client does not negotiate (or requests `"null"`), the connection
//! stays on plaintext NDJSON — backward-compatible with Phase 2 clients.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use zeroize::{Zeroize, ZeroizeOnDrop};

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

    /// Parse a cipher name from the client's `preferred_cipher` or `ciphers[]`.
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
/// `client_to_server` to decrypt incoming frames. Keys are zeroized on drop.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
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

/// Encrypt a plaintext frame using ChaCha20-Poly1305.
///
/// Returns the wire frame: `[4B length BE u32][12B nonce][ciphertext + 16B tag]`
#[allow(
    dead_code,
    reason = "consumed when connection loop switches to encrypted framing"
)]
pub fn encrypt_frame(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, FrameError> {
    use chacha20poly1305::aead::Aead;
    use chacha20poly1305::{ChaCha20Poly1305, KeyInit};

    let cipher = ChaCha20Poly1305::new(key.into());

    let mut nonce_bytes = [0u8; 12];
    rand::Rng::fill(&mut rand::rng(), &mut nonce_bytes);
    let nonce = chacha20poly1305::Nonce::from(nonce_bytes);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| FrameError::Encryption)?;

    let frame_len: u32 = (12 + ciphertext.len())
        .try_into()
        .map_err(|_| FrameError::FrameTooLarge)?;

    let mut frame = Vec::with_capacity(4 + 12 + ciphertext.len());
    frame.extend_from_slice(&frame_len.to_be_bytes());
    frame.extend_from_slice(&nonce_bytes);
    frame.extend_from_slice(&ciphertext);
    Ok(frame)
}

/// Decrypt a received frame (nonce || ciphertext+tag) using ChaCha20-Poly1305.
///
/// Input is the payload after the 4-byte length header: `[12B nonce][ciphertext + tag]`.
#[allow(
    dead_code,
    reason = "consumed when connection loop switches to encrypted framing"
)]
pub fn decrypt_frame(key: &[u8; 32], frame_payload: &[u8]) -> Result<Vec<u8>, FrameError> {
    use chacha20poly1305::aead::Aead;
    use chacha20poly1305::{ChaCha20Poly1305, KeyInit};

    if frame_payload.len() < 12 + 16 {
        return Err(FrameError::FrameTooShort);
    }

    let (nonce_bytes, ciphertext) = frame_payload.split_at(12);
    let nonce = chacha20poly1305::Nonce::from_slice(nonce_bytes);
    let cipher = ChaCha20Poly1305::new(key.into());

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| FrameError::Decryption)
}

/// Errors from encrypted frame operations.
#[derive(Debug, thiserror::Error)]
#[allow(dead_code, reason = "consumed by encrypt_frame/decrypt_frame")]
pub enum FrameError {
    #[error("encryption failed")]
    Encryption,
    #[error("decryption failed (invalid key or corrupted frame)")]
    Decryption,
    #[error("frame too large for u32 length header")]
    FrameTooLarge,
    #[error("frame too short (need at least 12B nonce + 16B tag)")]
    FrameTooShort,
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

    /// Server nonce generated during negotiate (base64-encoded).
    pub server_nonce: Option<String>,

    /// Handshake key from the security provider's `btsp.session.verify` response.
    /// `None` when the provider didn't return key material (older versions).
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
/// Validates the session, generates a 32-byte server nonce (base64-encoded),
/// and returns the negotiated cipher. If the requested cipher is not supported,
/// falls back to `"null"`. When both `handshake_key` (from Phase 2) and
/// `client_nonce` (from params) are available, derives directional session
/// keys via HKDF-SHA256.
///
/// Accepts `client_nonce` in either base64 or hex encoding (auto-detected).
pub async fn handle_negotiate(
    store: &BtspSessionStore,
    params: &serde_json::Value,
) -> Result<serde_json::Value, anyhow::Error> {
    let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("btsp.negotiate requires 'session_id' parameter"))?;

    let preferred = resolve_preferred_cipher(params);

    let client_nonce_raw = params
        .get("client_nonce")
        .and_then(|v| v.as_str())
        .map(decode_nonce);

    let mut cipher = BtspCipher::from_str_loose(&preferred);

    let server_nonce_bytes = generate_server_nonce_bytes();
    let server_nonce_b64 = base64_encode(&server_nonce_bytes);

    let mut sessions = store.write().await;
    let session = sessions
        .get_mut(session_id)
        .ok_or_else(|| anyhow::anyhow!("Unknown session_id: {session_id}"))?;

    if cipher == BtspCipher::ChaCha20Poly1305 {
        if let (Some(hk), Some(cn_bytes)) = (&session.handshake_key, &client_nonce_raw) {
            let keys = derive_session_keys(hk, cn_bytes, &server_nonce_bytes);
            session.session_keys = Some(keys);
        } else {
            cipher = BtspCipher::Null;
        }
    }

    session.cipher = cipher;
    session.server_nonce = Some(server_nonce_b64.clone());

    Ok(serde_json::json!({
        "cipher": cipher.as_str(),
        "server_nonce": server_nonce_b64,
        "allowed": true,
    }))
}

/// Resolve the preferred cipher from params.
///
/// Supports both `"preferred_cipher"` (string) and `"ciphers"` (array) fields,
/// matching the wire formats used by different primals.
fn resolve_preferred_cipher(params: &serde_json::Value) -> String {
    if let Some(s) = params.get("preferred_cipher").and_then(|v| v.as_str()) {
        return s.to_string();
    }
    if let Some(arr) = params.get("ciphers").and_then(|v| v.as_array()) {
        for c in arr {
            if let Some(s) = c.as_str() {
                if BtspCipher::from_str_loose(s) == BtspCipher::ChaCha20Poly1305 {
                    return s.to_string();
                }
            }
        }
    }
    "null".to_string()
}

/// Generate a random 32-byte server nonce (raw bytes).
fn generate_server_nonce_bytes() -> [u8; 32] {
    use rand::Rng;
    let mut nonce = [0u8; 32];
    rand::rng().fill(&mut nonce);
    nonce
}

fn base64_encode(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn base64_decode(s: &str) -> Option<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(s).ok()
}

/// Decode a nonce string, auto-detecting base64 vs hex encoding.
///
/// Heuristic: if the string contains `+`, `/`, `=`, or non-hex chars, treat
/// as base64. If it's all hex digits, check length — hex-encoded bytes produce
/// an even-length string of exactly `2 * byte_count`.
fn decode_nonce(s: &str) -> Vec<u8> {
    let is_all_hex = s.len().is_multiple_of(2) && s.chars().all(|c| c.is_ascii_hexdigit());
    if is_all_hex && s.len() >= 24 {
        hex_decode(s)
    } else if let Some(decoded) = base64_decode(s) {
        decoded
    } else {
        hex_decode(s)
    }
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
    fn test_server_nonce_is_32_bytes_base64() {
        let nonce_bytes = generate_server_nonce_bytes();
        assert_eq!(nonce_bytes.len(), 32);
        let encoded = base64_encode(&nonce_bytes);
        assert_eq!(encoded.len(), 44); // 32 bytes → 44 base64 chars
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, nonce_bytes);
    }

    #[tokio::test]
    async fn test_register_and_negotiate_with_key_base64_nonce() {
        let store = new_session_store();
        let fake_key = [0xABu8; 32];
        register_session(&store, "test-session-b64", Some(fake_key)).await;

        let client_nonce = base64_encode(&[0x11u8; 32]);
        let params = serde_json::json!({
            "session_id": "test-session-b64",
            "preferred_cipher": "chacha20-poly1305",
            "client_nonce": client_nonce,
            "bond_type": "Covalent"
        });

        let result = handle_negotiate(&store, &params).await.unwrap();
        assert_eq!(result["cipher"], "chacha20-poly1305");
        assert_eq!(result["allowed"], true);
        let sn = result["server_nonce"].as_str().unwrap();
        let sn_decoded = base64_decode(sn).unwrap();
        assert_eq!(sn_decoded.len(), 32);

        let sessions = store.read().await;
        let sess = sessions.get("test-session-b64").unwrap();
        assert!(sess.session_keys.is_some());
    }

    #[tokio::test]
    async fn test_negotiate_with_hex_client_nonce_compat() {
        let store = new_session_store();
        let fake_key = [0xABu8; 32];
        register_session(&store, "test-session-hex", Some(fake_key)).await;

        let params = serde_json::json!({
            "session_id": "test-session-hex",
            "preferred_cipher": "chacha20-poly1305",
            "client_nonce": "aabbccdd11223344aabbccdd11223344",
            "bond_type": "Covalent"
        });

        let result = handle_negotiate(&store, &params).await.unwrap();
        assert_eq!(result["cipher"], "chacha20-poly1305");

        let sessions = store.read().await;
        assert!(
            sessions
                .get("test-session-hex")
                .unwrap()
                .session_keys
                .is_some()
        );
    }

    #[tokio::test]
    async fn test_negotiate_ciphers_array_param() {
        let store = new_session_store();
        let fake_key = [0xCCu8; 32];
        register_session(&store, "sess-arr", Some(fake_key)).await;

        let client_nonce = base64_encode(&[0x22u8; 32]);
        let params = serde_json::json!({
            "session_id": "sess-arr",
            "ciphers": ["chacha20-poly1305"],
            "client_nonce": client_nonce
        });

        let result = handle_negotiate(&store, &params).await.unwrap();
        assert_eq!(result["cipher"], "chacha20-poly1305");
    }

    #[tokio::test]
    async fn test_negotiate_no_handshake_key_falls_back_to_null() {
        let store = new_session_store();
        register_session(&store, "sess-no-key", None).await;

        let client_nonce = base64_encode(&[0x33u8; 32]);
        let params = serde_json::json!({
            "session_id": "sess-no-key",
            "preferred_cipher": "chacha20-poly1305",
            "client_nonce": client_nonce
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
    fn test_encrypt_decrypt_roundtrip() {
        let key = [0xAAu8; 32];
        let plaintext = b"hello btsp phase 3";
        let frame = encrypt_frame(&key, plaintext).unwrap();

        assert_eq!(
            u32::from_be_bytes([frame[0], frame[1], frame[2], frame[3]]) as usize,
            frame.len() - 4
        );

        let payload = &frame[4..];
        let decrypted = decrypt_frame(&key, payload).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_wrong_key_fails() {
        let key1 = [0xAAu8; 32];
        let key2 = [0xBBu8; 32];
        let frame = encrypt_frame(&key1, b"secret message").unwrap();
        let payload = &frame[4..];
        let result = decrypt_frame(&key2, payload);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_truncated_frame_fails() {
        let result = decrypt_frame(&[0u8; 32], &[0u8; 10]);
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_produces_unique_nonces() {
        let key = [0xCCu8; 32];
        let f1 = encrypt_frame(&key, b"msg1").unwrap();
        let f2 = encrypt_frame(&key, b"msg1").unwrap();
        assert_ne!(&f1[4..16], &f2[4..16]);
    }

    #[test]
    fn test_decode_nonce_base64() {
        let original = vec![0x11u8; 32];
        let b64 = base64_encode(&original);
        let decoded = decode_nonce(&b64);
        assert_eq!(decoded, original);
    }

    #[test]
    fn test_decode_nonce_hex() {
        let hex_str = "aabbccdd11223344aabbccdd11223344";
        let decoded = decode_nonce(hex_str);
        assert_eq!(decoded.len(), 16);
        assert_eq!(decoded[0], 0xaa);
        assert_eq!(decoded[1], 0xbb);
    }

    #[test]
    fn test_resolve_preferred_cipher_string() {
        let params = serde_json::json!({"preferred_cipher": "chacha20-poly1305"});
        assert_eq!(resolve_preferred_cipher(&params), "chacha20-poly1305");
    }

    #[test]
    fn test_resolve_preferred_cipher_array() {
        let params = serde_json::json!({"ciphers": ["chacha20-poly1305"]});
        assert_eq!(resolve_preferred_cipher(&params), "chacha20-poly1305");
    }

    #[test]
    fn test_resolve_preferred_cipher_empty() {
        let params = serde_json::json!({});
        assert_eq!(resolve_preferred_cipher(&params), "null");
    }

    #[test]
    fn test_hex_roundtrip() {
        let original = [0xDE, 0xAD, 0xBE, 0xEF, 0x01, 0x23];
        let mut s = String::with_capacity(original.len() * 2);
        for b in &original {
            use std::fmt::Write;
            let _ = write!(s, "{b:02x}");
        }
        let decoded = hex_decode(&s);
        assert_eq!(&original[..], &decoded[..]);
    }
}
