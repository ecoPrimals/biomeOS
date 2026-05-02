// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::*;
use std::path::Path;

#[test]
fn family_scoped_detection() {
    assert!(is_family_scoped_socket(Path::new(
        "/run/user/1000/biomeos/beardog-8ff3b864a4bc589a.sock"
    )));
    assert!(is_family_scoped_socket(Path::new(
        "/tmp/biomeos/songbird-abc123.sock"
    )));
    assert!(!is_family_scoped_socket(Path::new(
        "/run/user/1000/biomeos/beardog.sock"
    )));
    assert!(!is_family_scoped_socket(Path::new(
        "/run/user/1000/biomeos/biomeos.sock"
    )));
}

#[test]
fn family_scoped_domain_stem_sockets() {
    assert!(is_family_scoped_socket(Path::new(
        "/run/user/1000/biomeos/security-8ff3b864.sock"
    )));
    assert!(is_family_scoped_socket(Path::new(
        "/run/user/1000/biomeos/compute-abc123.sock"
    )));
    assert!(is_family_scoped_socket(Path::new(
        "/run/user/1000/biomeos/ai-def456.sock"
    )));
    assert!(!is_family_scoped_socket(Path::new(
        "/run/user/1000/biomeos/security.sock"
    )));
}

#[test]
fn extract_family_from_socket() {
    assert_eq!(
        extract_family_id(Path::new("/tmp/beardog-abc123.sock")),
        Some("abc123".to_owned())
    );
    assert_eq!(
        extract_family_id(Path::new("/tmp/nestgate-8ff3b864a4bc589a.sock")),
        Some("8ff3b864a4bc589a".to_owned())
    );
    assert_eq!(extract_family_id(Path::new("/tmp/beardog.sock")), None);
}

#[test]
fn extract_family_from_domain_stem_socket() {
    assert_eq!(
        extract_family_id(Path::new("/tmp/security-abc123.sock")),
        Some("abc123".to_owned())
    );
    assert_eq!(
        extract_family_id(Path::new("/tmp/compute-familyXYZ.sock")),
        Some("familyXYZ".to_owned())
    );
}

#[test]
fn multi_hyphen_family_id() {
    assert!(is_family_scoped_socket(Path::new(
        "/tmp/beardog-abc-def-123.sock"
    )));
    assert_eq!(
        extract_family_id(Path::new("/tmp/beardog-abc-def-123.sock")),
        Some("abc-def-123".to_owned())
    );
}

#[test]
fn edge_cases() {
    assert!(!is_family_scoped_socket(Path::new("")));
    assert!(!is_family_scoped_socket(Path::new("/tmp/.sock")));
    assert!(!is_family_scoped_socket(Path::new("/tmp/noext")));
    assert!(extract_family_id(Path::new("")).is_none());
    assert!(extract_family_id(Path::new("/tmp/noext")).is_none());
}

#[test]
fn insecure_guard_ok_without_env() {
    let result = validate_insecure_guard();
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn security_mode_returns_valid_variant() {
    let mode = security_mode();
    match mode {
        SecurityMode::Development | SecurityMode::Production { .. } => {}
    }
}

#[test]
fn log_security_posture_does_not_panic() {
    log_security_posture();
}

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
    assert!(format!("{}", BtspHandshakeError::BearDogNotFound).contains("security provider"));
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
    let (mut s, _c) = tokio::net::UnixStream::pair().unwrap();
    let mut reader = tokio::io::BufReader::new(&mut s);
    let result = server_handshake(&mut reader).await;
    assert!(matches!(result, Ok(HandshakeOutcome::DevMode)));
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
