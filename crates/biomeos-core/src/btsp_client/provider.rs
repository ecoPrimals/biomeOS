// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::path::Path;

use super::types::BtspHandshakeError;

pub(crate) struct BtspSession {
    pub(super) session_id: String,
    pub(super) server_ephemeral_pub: String,
    pub(super) challenge: String,
}

/// Result of session verification including optional key material.
pub(crate) struct VerifyResult {
    pub(super) verified: bool,
    /// 32-byte session key from the security provider, if provided.
    pub(super) handshake_key: Option<[u8; 32]>,
}

pub(crate) async fn create_session_via_security_provider(
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

    let session_id = result["session_id"]
        .as_str()
        .ok_or_else(|| {
            BtspHandshakeError::SecurityProviderError(
                "missing session_id in btsp.session.create response".into(),
            )
        })?
        .to_owned();
    let server_ephemeral_pub = result["server_ephemeral_pub"]
        .as_str()
        .ok_or_else(|| {
            BtspHandshakeError::SecurityProviderError(
                "missing server_ephemeral_pub in btsp.session.create response".into(),
            )
        })?
        .to_owned();
    let challenge = result["challenge"]
        .as_str()
        .ok_or_else(|| {
            BtspHandshakeError::SecurityProviderError(
                "missing challenge in btsp.session.create response".into(),
            )
        })?
        .to_owned();

    Ok(BtspSession {
        session_id,
        server_ephemeral_pub,
        challenge,
    })
}

pub(crate) async fn verify_session_via_security_provider(
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
