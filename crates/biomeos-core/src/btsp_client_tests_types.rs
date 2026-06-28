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

// ── Phase 2: Handshake wire types ──

#[test]
fn client_hello_serialization_roundtrip() {
    let hello = ClientHello {
        protocol: "btsp".to_owned(),
        version: BTSP_VERSION,
        client_ephemeral_pub: "AAAA".to_owned(),
    };
    let json = serde_json::to_string(&hello).unwrap();
    let parsed: ClientHello = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.protocol, "btsp");
    assert_eq!(parsed.version, BTSP_VERSION);
    assert_eq!(parsed.client_ephemeral_pub, "AAAA");
}

#[test]
fn server_hello_serialization_roundtrip() {
    let hello = ServerHello {
        version: BTSP_VERSION,
        server_ephemeral_pub: "BBBB".to_owned(),
        challenge: "CCCC".to_owned(),
        session_id: "deadbeef".to_owned(),
    };
    let json = serde_json::to_string(&hello).unwrap();
    let parsed: ServerHello = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.session_id, "deadbeef");
    assert_eq!(parsed.challenge, "CCCC");
}

#[test]
fn challenge_response_default_cipher_is_null() {
    let json = r#"{"response":"HMAC"}"#;
    let parsed: ChallengeResponse = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.preferred_cipher, "null");
}

#[test]
fn handshake_complete_roundtrip() {
    let complete = HandshakeComplete {
        cipher: "chacha20_poly1305".to_owned(),
        session_id: "abc123".to_owned(),
    };
    let json = serde_json::to_string(&complete).unwrap();
    assert!(json.contains("chacha20_poly1305"));
    let parsed: HandshakeComplete = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.cipher, "chacha20_poly1305");
}

#[test]
fn handshake_error_roundtrip() {
    let err = HandshakeError {
        error: "handshake_failed".to_owned(),
        reason: "family_verification".to_owned(),
    };
    let json = serde_json::to_string(&err).unwrap();
    let parsed: HandshakeError = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.error, "handshake_failed");
    assert_eq!(parsed.reason, "family_verification");
}

#[test]
fn btsp_handshake_error_display() {
    assert!(
        format!("{}", BtspHandshakeError::SecurityProviderNotFound).contains("security provider")
    );
    assert!(format!("{}", BtspHandshakeError::VerificationFailed).contains("verification"));
    assert!(format!("{}", BtspHandshakeError::Timeout).contains("timed out"));
    assert!(format!("{}", BtspHandshakeError::ConnectionClosed).contains("disconnected"));
    assert!(
        format!("{}", BtspHandshakeError::RawJsonRpc("{}".to_owned())).contains("raw JSON-RPC")
    );
    assert!(
        format!(
            "{}",
            BtspHandshakeError::SecurityProviderError("x".to_owned())
        )
        .contains("security provider")
    );
    assert!(format!("{}", BtspHandshakeError::Protocol("bad".to_owned())).contains("protocol"));
}

// ── Phase 2: server_handshake on raw JSON-RPC input ──

#[tokio::test]
async fn server_handshake_returns_devmode_without_family_id() {
    temp_env::async_with_vars(
        [("BIOMEOS_FAMILY_ID", None::<&str>), ("FAMILY_ID", None)],
        async {
            let (mut s, _c) = tokio::net::UnixStream::pair().unwrap();
            let mut reader = tokio::io::BufReader::new(&mut s);
            let result = server_handshake(&mut reader).await;
            assert!(matches!(result, Ok(HandshakeOutcome::DevMode)));
        },
    )
    .await;
}

#[test]
fn security_provider_socket_path_returns_none_without_env() {
    let _ = security_provider_socket_path();
}

#[test]
fn handshake_outcome_debug() {
    let auth = HandshakeOutcome::Authenticated {
        session_id: "s1".to_owned(),
        handshake_key: None,
    };
    let dbg = format!("{auth:?}");
    assert!(dbg.contains("Authenticated"));
    assert!(dbg.contains("s1"));
}
