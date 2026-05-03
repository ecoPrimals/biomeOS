// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! BTSP Phase 3 — Client-side negotiate and encrypted framing for outbound calls.
//!
//! After a successful Phase 2 handshake, the client sends `btsp.negotiate` to
//! upgrade the connection from authenticated-plaintext to ChaCha20-Poly1305
//! encrypted framing. Falls back to plaintext if the server returns a null cipher.

use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{debug, info};

use super::btsp_client::{
    BTSP_VERSION, BtspHandshakeError, ChallengeResponse, ClientHello, HandshakeComplete,
    client_keygen, read_json_line, security_provider_socket_path, serialize_line, write_line_to,
};

/// Outcome of a client-side Phase 3 handshake + negotiate.
pub enum ClientPhase3Outcome {
    /// Phase 3 negotiated — connection uses encrypted framing.
    Encrypted {
        /// Directional session keys for encrypted I/O.
        keys: crate::btsp_crypto::SessionKeys,
        /// The connected stream (post-handshake, pre-framing).
        stream: tokio::net::UnixStream,
    },
    /// Phase 3 not available — connection stays on plaintext NDJSON.
    Plaintext {
        /// The connected stream (post-handshake).
        stream: tokio::net::UnixStream,
    },
}

/// Perform Phase 2 handshake + Phase 3 negotiate on an already-connected stream.
///
/// If the remote primal supports `btsp.negotiate` with ChaCha20-Poly1305, the
/// returned `Encrypted` variant carries directional `SessionKeys` for encrypted
/// framing. Otherwise, the `Plaintext` variant carries the raw stream.
pub async fn perform_client_handshake_phase3(
    stream: tokio::net::UnixStream,
) -> Result<ClientPhase3Outcome, BtspHandshakeError> {
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

    let mut reader = BufReader::new(stream);
    write_line_to(&mut reader, &hello_line).await?;

    let server_hello = read_json_line(&mut reader).await?;
    let (response, shared_secret) =
        client_challenge_response_with_key(&bd, &client_secret, &server_hello).await?;

    let cr = ChallengeResponse {
        response,
        preferred_cipher: "chacha20-poly1305".into(),
    };
    let cr_line = serialize_line(&cr)?;
    write_line_to(&mut reader, &cr_line).await?;

    let complete: HandshakeComplete = read_json_line(&mut reader).await?;
    debug!(session_id = %complete.session_id, "BTSP Phase 2 complete, attempting Phase 3 negotiate");

    let Some(handshake_key) = decode_shared_secret_to_key(&shared_secret) else {
        debug!("No handshake key material — staying plaintext");
        return Ok(ClientPhase3Outcome::Plaintext {
            stream: reader.into_inner(),
        });
    };

    let negotiate_outcome =
        client_negotiate(&mut reader, &complete.session_id, &handshake_key).await;

    match negotiate_outcome {
        Ok(keys) => {
            info!(session_id = %complete.session_id, "Phase 3 encrypted channel established");
            Ok(ClientPhase3Outcome::Encrypted {
                keys,
                stream: reader.into_inner(),
            })
        }
        Err(e) => {
            debug!("Phase 3 negotiate failed ({e}), staying plaintext");
            Ok(ClientPhase3Outcome::Plaintext {
                stream: reader.into_inner(),
            })
        }
    }
}

/// Send `btsp.negotiate` and derive session keys if the server agrees to encrypt.
async fn client_negotiate(
    reader: &mut BufReader<tokio::net::UnixStream>,
    session_id: &str,
    handshake_key: &[u8; 32],
) -> Result<crate::btsp_crypto::SessionKeys, BtspHandshakeError> {
    use base64::Engine;

    let mut client_nonce = [0u8; 32];
    rand::Rng::fill(&mut rand::rng(), &mut client_nonce);
    let client_nonce_b64 = base64::engine::general_purpose::STANDARD.encode(client_nonce);

    let negotiate_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "btsp.negotiate",
        "params": {
            "session_id": session_id,
            "preferred_cipher": "chacha20-poly1305",
            "client_nonce": client_nonce_b64,
            "bond_type": "Covalent",
        },
        "id": 1,
    });

    let mut req_line = serde_json::to_string(&negotiate_req)
        .map_err(|e| BtspHandshakeError::Protocol(format!("serialize negotiate: {e}")))?;
    req_line.push('\n');
    write_line_to(reader, &req_line).await?;

    let mut resp_line = String::new();
    let n = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        reader.read_line(&mut resp_line),
    )
    .await
    .map_err(|_| BtspHandshakeError::Timeout)?
    .map_err(BtspHandshakeError::Io)?;

    if n == 0 {
        return Err(BtspHandshakeError::ConnectionClosed);
    }

    let resp: serde_json::Value = serde_json::from_str(resp_line.trim())
        .map_err(|e| BtspHandshakeError::Protocol(format!("parse negotiate response: {e}")))?;

    let result = resp.get("result").ok_or_else(|| {
        let err_msg = resp
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("unknown error");
        BtspHandshakeError::Protocol(format!("negotiate rejected: {err_msg}"))
    })?;

    let cipher = result
        .get("cipher")
        .and_then(|v| v.as_str())
        .unwrap_or("null");

    if !cipher.to_ascii_lowercase().contains("chacha20") {
        return Err(BtspHandshakeError::Protocol(
            "server responded with null cipher".into(),
        ));
    }

    let server_nonce_str = result
        .get("server_nonce")
        .and_then(|v| v.as_str())
        .ok_or_else(|| BtspHandshakeError::Protocol("missing server_nonce".into()))?;

    let server_nonce = base64::engine::general_purpose::STANDARD
        .decode(server_nonce_str)
        .or_else(|_| hex::decode(server_nonce_str))
        .map_err(|e| BtspHandshakeError::Protocol(format!("decode server_nonce: {e}")))?;

    let keys = crate::btsp_crypto::derive_session_keys(handshake_key, &client_nonce, &server_nonce);
    Ok(keys)
}

/// Derive the HMAC response AND return the raw shared secret for Phase 3 key derivation.
async fn client_challenge_response_with_key(
    bd: &crate::AtomicClient,
    client_secret: &str,
    server_hello: &super::btsp_client::ServerHello,
) -> Result<(String, String), BtspHandshakeError> {
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
        .ok_or_else(|| BtspHandshakeError::Protocol("missing shared_secret from derive".into()))?
        .to_owned();
    let hmac_result = bd
        .call(
            "hmac_sha256",
            serde_json::json!({ "key": &shared_secret, "data": server_hello.challenge }),
        )
        .await
        .map_err(|e| BtspHandshakeError::SecurityProviderError(format!("hmac: {e}")))?;
    let hmac = hmac_result["hmac"]
        .as_str()
        .or_else(|| hmac_result["result"].as_str())
        .ok_or_else(|| BtspHandshakeError::Protocol("missing hmac from response".into()))?
        .to_owned();
    Ok((hmac, shared_secret))
}

/// Decode a hex-encoded shared secret to a 32-byte key.
fn decode_shared_secret_to_key(hex_str: &str) -> Option<[u8; 32]> {
    let bytes: Vec<u8> = (0..hex_str.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(hex_str.get(i..i + 2)?, 16).ok())
        .collect();
    <[u8; 32]>::try_from(bytes.as_slice()).ok()
}
