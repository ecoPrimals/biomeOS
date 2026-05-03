// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! BTSP Phase 3 shared crypto primitives — used by both server-side (neural API)
//! and client-side (outbound calls) encrypted framing.
//!
//! Cipher negotiation, HKDF-SHA256 key derivation, and ChaCha20-Poly1305
//! AEAD frame encrypt/decrypt. Wire format:
//!
//! ```text
//! [4 bytes: payload length (big-endian u32)]
//! [12 bytes: nonce]
//! [ciphertext + 16 bytes: Poly1305 tag]
//! ```

use zeroize::{Zeroize, ZeroizeOnDrop};

/// Supported BTSP cipher suites.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtspCipher {
    /// No encryption — plaintext NDJSON (backward-compatible fallback).
    Null,
    /// ChaCha20-Poly1305 AEAD with 12-byte nonce per frame.
    ChaCha20Poly1305,
}

impl BtspCipher {
    /// Wire name per `BTSP_PROTOCOL_STANDARD.md`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::ChaCha20Poly1305 => "chacha20-poly1305",
        }
    }

    /// Parse a cipher name from the client's `preferred_cipher` or `ciphers[]`.
    #[must_use]
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
/// The client uses `client_to_server` to encrypt outgoing frames and
/// `server_to_client` to decrypt incoming frames. Keys are zeroized on drop.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SessionKeys {
    /// Key for encrypting frames from client to server.
    pub client_to_server: [u8; 32],
    /// Key for encrypting frames from server to client.
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
pub enum FrameError {
    /// AEAD encryption failed.
    #[error("encryption failed")]
    Encryption,
    /// AEAD decryption failed (wrong key or corrupted frame).
    #[error("decryption failed (invalid key or corrupted frame)")]
    Decryption,
    /// Payload exceeds u32 length header capacity.
    #[error("frame too large for u32 length header")]
    FrameTooLarge,
    /// Frame payload too short to contain nonce + tag.
    #[error("frame too short (need at least 12B nonce + 16B tag)")]
    FrameTooShort,
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_encrypt_decrypt() {
        let key = [42u8; 32];
        let plaintext = b"hello encrypted world";
        let frame = encrypt_frame(&key, plaintext).unwrap();

        assert!(frame.len() > 4 + 12 + 16);
        let payload = &frame[4..];
        let decrypted = decrypt_frame(&key, payload).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn wrong_key_rejected() {
        let key = [42u8; 32];
        let wrong_key = [99u8; 32];
        let frame = encrypt_frame(&key, b"secret").unwrap();
        let payload = &frame[4..];
        assert!(decrypt_frame(&wrong_key, payload).is_err());
    }

    #[test]
    fn derive_keys_deterministic() {
        let hk = [1u8; 32];
        let cn = [2u8; 12];
        let sn = [3u8; 12];
        let k1 = derive_session_keys(&hk, &cn, &sn);
        let k2 = derive_session_keys(&hk, &cn, &sn);
        assert_eq!(k1.client_to_server, k2.client_to_server);
        assert_eq!(k1.server_to_client, k2.server_to_client);
        assert_ne!(k1.client_to_server, k1.server_to_client);
    }

    #[test]
    fn cipher_from_str_loose() {
        assert_eq!(
            BtspCipher::from_str_loose("chacha20-poly1305"),
            BtspCipher::ChaCha20Poly1305
        );
        assert_eq!(
            BtspCipher::from_str_loose("ChaCha20Poly1305"),
            BtspCipher::ChaCha20Poly1305
        );
        assert_eq!(BtspCipher::from_str_loose("null"), BtspCipher::Null);
        assert_eq!(BtspCipher::from_str_loose("unknown"), BtspCipher::Null);
    }

    #[test]
    fn frame_too_short() {
        let key = [42u8; 32];
        assert!(matches!(
            decrypt_frame(&key, &[0u8; 10]),
            Err(FrameError::FrameTooShort)
        ));
    }
}
