// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::or_fun_call,
    clippy::future_not_send,
    reason = "test assertions"
)]

use super::super::*;
use super::{
    VALID_SESSION_KEY_HEX, decode_session_key_hex, spawn_btsp_session_provider, spawn_rpc_mock,
};
use serde_json::json;
use std::sync::{Arc, Mutex};

// ── provider.rs ──

#[tokio::test]
async fn create_session_via_provider_sends_expected_rpc_and_parses_response() {
    let dir = tempfile::tempdir().expect("tempdir");
    let captured = Arc::new(Mutex::new(None));
    let captured_clone = Arc::clone(&captured);
    let provider_path = spawn_rpc_mock(dir.path(), move |method, params| {
        if method == "btsp.session.create" {
            *captured_clone.lock().expect("lock") = Some((method.to_owned(), params.clone()));
        }
        json!({
            "session_id": "sess-123",
            "server_ephemeral_pub": "server-pub-b64",
            "challenge": "challenge-b64",
        })
    })
    .await;

    temp_env::async_with_vars(
        [(
            "BIOMEOS_SECURITY_SOCKET",
            Some(provider_path.to_str().unwrap()),
        )],
        async {
            let session = create_session_via_security_provider(&provider_path, "client-pub-b64")
                .await
                .expect("create session");

            assert_eq!(session.session_id, "sess-123");
            assert_eq!(session.server_ephemeral_pub, "server-pub-b64");
            assert_eq!(session.challenge, "challenge-b64");

            let (method, params) = captured
                .lock()
                .expect("lock")
                .clone()
                .expect("captured rpc");
            assert_eq!(method, "btsp.session.create");
            assert_eq!(params["family_seed_ref"], "env:FAMILY_SEED");
            assert_eq!(params["client_ephemeral_pub"], "client-pub-b64");
        },
    )
    .await;
}

#[tokio::test]
async fn create_session_via_provider_missing_session_id() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider_path = spawn_btsp_session_provider(
        dir.path(),
        json!({ "server_ephemeral_pub": "pub", "challenge": "chal" }),
        json!({ "verified": true }),
    )
    .await;

    assert!(matches!(
        create_session_via_security_provider(&provider_path, "pub").await,
        Err(BtspHandshakeError::SecurityProviderError(ref msg)) if msg.contains("session_id")
    ));
}

#[tokio::test]
async fn create_session_via_provider_missing_server_ephemeral_pub() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider_path = spawn_btsp_session_provider(
        dir.path(),
        json!({ "session_id": "s1", "challenge": "chal" }),
        json!({ "verified": true }),
    )
    .await;

    assert!(matches!(
        create_session_via_security_provider(&provider_path, "pub").await,
        Err(BtspHandshakeError::SecurityProviderError(ref msg))
            if msg.contains("server_ephemeral_pub")
    ));
}

#[tokio::test]
async fn create_session_via_provider_missing_challenge() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider_path = spawn_btsp_session_provider(
        dir.path(),
        json!({ "session_id": "s1", "server_ephemeral_pub": "pub" }),
        json!({ "verified": true }),
    )
    .await;

    assert!(matches!(
        create_session_via_security_provider(&provider_path, "pub").await,
        Err(BtspHandshakeError::SecurityProviderError(ref msg)) if msg.contains("challenge")
    ));
}

#[tokio::test]
async fn create_session_via_provider_rpc_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing_path = dir.path().join("does-not-exist.sock");
    assert!(matches!(
        create_session_via_security_provider(&missing_path, "pub").await,
        Err(BtspHandshakeError::SecurityProviderError(_))
    ));
}

#[tokio::test]
async fn verify_session_via_provider_sends_expected_rpc_and_parses_key() {
    let dir = tempfile::tempdir().expect("tempdir");
    let captured = Arc::new(Mutex::new(None));
    let captured_clone = Arc::clone(&captured);
    let provider_path = spawn_rpc_mock(dir.path(), move |method, params| {
        if method == "btsp.session.verify" {
            *captured_clone.lock().expect("lock") = Some(params.clone());
        }
        json!({
            "verified": true,
            "session_key": VALID_SESSION_KEY_HEX,
        })
    })
    .await;

    let result = verify_session_via_security_provider(
        &provider_path,
        "sess-1",
        "hmac-response",
        "client-pub",
        "server-pub",
        "challenge-b64",
    )
    .await
    .expect("verify ok");

    assert!(result.verified);
    assert_eq!(result.handshake_key, Some(decode_session_key_hex()));

    let params = captured
        .lock()
        .expect("lock")
        .clone()
        .expect("verify params");
    assert_eq!(params["session_id"], "sess-1");
    assert_eq!(params["client_response"], "hmac-response");
    assert_eq!(params["client_ephemeral_pub"], "client-pub");
    assert_eq!(params["server_ephemeral_pub"], "server-pub");
    assert_eq!(params["challenge"], "challenge-b64");
}

#[tokio::test]
async fn verify_session_via_provider_rejects_unverified_client() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider_path = spawn_btsp_session_provider(
        dir.path(),
        json!({ "session_id": "s1", "server_ephemeral_pub": "pub", "challenge": "chal" }),
        json!({ "verified": false }),
    )
    .await;

    let result =
        verify_session_via_security_provider(&provider_path, "s1", "resp", "cpub", "spub", "chal")
            .await
            .expect("verify call ok");

    assert!(!result.verified);
    assert!(result.handshake_key.is_none());
}

#[tokio::test]
async fn verify_session_via_provider_ignores_invalid_session_key() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider_path = spawn_btsp_session_provider(
        dir.path(),
        json!({ "session_id": "s1", "server_ephemeral_pub": "pub", "challenge": "chal" }),
        json!({ "verified": true, "session_key": "not-valid-hex-key" }),
    )
    .await;

    let result =
        verify_session_via_security_provider(&provider_path, "s1", "resp", "cpub", "spub", "chal")
            .await
            .expect("verify call ok");

    assert!(result.verified);
    assert!(result.handshake_key.is_none());
}

#[tokio::test]
async fn verify_session_via_provider_defaults_verified_to_false() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider_path = spawn_btsp_session_provider(
        dir.path(),
        json!({ "session_id": "s1", "server_ephemeral_pub": "pub", "challenge": "chal" }),
        json!({}),
    )
    .await;

    let result =
        verify_session_via_security_provider(&provider_path, "s1", "resp", "cpub", "spub", "chal")
            .await
            .expect("verify call ok");
    assert!(!result.verified);
}
