// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use tokio::io::AsyncWriteExt;
use tracing::debug;

use super::config::security_provider_socket_path;
use super::types::{
    BTSP_VERSION, BtspHandshakeError, ChallengeResponse, ClientHello, HandshakeComplete,
    HandshakeError, ServerHello,
};

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

pub(crate) async fn client_keygen(
    bd: &crate::AtomicClient,
) -> Result<(String, String), BtspHandshakeError> {
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

pub(crate) fn serialize_line(value: &impl serde::Serialize) -> Result<String, BtspHandshakeError> {
    let mut s =
        serde_json::to_string(value).map_err(|e| BtspHandshakeError::Protocol(e.to_string()))?;
    s.push('\n');
    Ok(s)
}

pub(crate) async fn write_line_to(
    reader: &mut tokio::io::BufReader<tokio::net::UnixStream>,
    data: &str,
) -> Result<(), BtspHandshakeError> {
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

pub(crate) async fn read_json_line<T: serde::de::DeserializeOwned>(
    reader: &mut tokio::io::BufReader<tokio::net::UnixStream>,
) -> Result<T, BtspHandshakeError> {
    use tokio::io::AsyncBufReadExt;
    let mut line = String::new();
    let n = tokio::time::timeout(
        biomeos_types::constants::timeouts::BTSP_CALL_TIMEOUT,
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
