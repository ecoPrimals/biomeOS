// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use tokio::io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tracing::debug;

use super::config::{has_family_id, security_provider_socket_path};
use super::provider::{create_session_via_security_provider, verify_session_via_security_provider};
use super::types::{
    BTSP_VERSION, BtspHandshakeError, ChallengeResponse, ClientHello, HandshakeComplete,
    HandshakeError, HandshakeOutcome, ServerHello,
};

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
///
/// [`btsp_enforce`]: super::config::btsp_enforce
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
        biomeos_types::constants::timeouts::BTSP_CALL_TIMEOUT,
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
        biomeos_types::constants::timeouts::BTSP_CALL_TIMEOUT,
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
        let mut err_line = serde_json::to_string(&err).unwrap_or_else(|e| {
            tracing::warn!("Failed to serialize BTSP handshake error: {e}");
            r#"{"error":"handshake_failed","reason":"family_verification"}"#.to_owned()
        });
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
