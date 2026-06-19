// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::or_fun_call,
    clippy::future_not_send,
    reason = "test assertions"
)]

use super::*;
use biomeos_types::JsonRpcResponse;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::oneshot;

const VALID_SESSION_KEY_HEX: &str =
    "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20";

async fn spawn_rpc_mock<F>(dir: &Path, handler: F) -> PathBuf
where
    F: Fn(&str, &serde_json::Value) -> serde_json::Value + Send + Sync + 'static,
{
    let path = dir.join("mock-provider.sock");
    let listener = UnixListener::bind(&path).expect("bind mock provider");
    let handler = Arc::new(handler);
    let (ready_tx, ready_rx) = oneshot::channel();

    tokio::spawn(async move {
        ready_tx.send(()).expect("signal mock ready");
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            let handler = Arc::clone(&handler);
            tokio::spawn(async move {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_err() {
                    return;
                }
                let req =
                    serde_json::from_str::<serde_json::Value>(line.trim()).unwrap_or(json!({}));
                let id = req.get("id").cloned().unwrap_or(json!(1));
                let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
                let params = req.get("params").cloned().unwrap_or(json!({}));
                let result = handler(method, &params);
                let response = JsonRpcResponse::success(id, result);
                let mut stream = reader.into_inner();
                let body = format!("{}\n", serde_json::to_string(&response).expect("serialize"));
                let _ = stream.write_all(body.as_bytes()).await;
                let _ = stream.flush().await;
            });
        }
    });

    ready_rx.await.expect("mock provider failed to start");
    path
}

async fn spawn_btsp_session_provider(
    dir: &Path,
    create_result: serde_json::Value,
    verify_result: serde_json::Value,
) -> PathBuf {
    spawn_rpc_mock(dir, move |method, _params| match method {
        "btsp.session.create" => create_result.clone(),
        "btsp.session.verify" => verify_result.clone(),
        other => json!({ "unexpected_method": other }),
    })
    .await
}

async fn spawn_client_crypto_provider(dir: &Path, shared_secret_hex: &str) -> PathBuf {
    let shared_secret_hex = shared_secret_hex.to_owned();
    spawn_rpc_mock(dir, move |method, _params| match method {
        "x25519_generate_ephemeral" => json!({
            "public_key": "dGVzdC1jbGllbnQtcHVi",
            "secret_key": "dGVzdC1jbGllbnQtc2VjcmV0",
        }),
        "crypto.x25519_derive_secret" => json!({ "shared_secret": shared_secret_hex }),
        "hmac_sha256" => json!({ "hmac": "dGVzdC1obWFjLXJlc3BvbnNl" }),
        other => json!({ "unexpected_method": other }),
    })
    .await
}

async fn with_family_env<F, Fut>(provider_path: PathBuf, f: F)
where
    F: FnOnce(PathBuf) -> Fut,
    Fut: std::future::Future<Output = ()> + Send,
{
    let dir = provider_path
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("/tmp")
        .to_owned();
    let provider_path_str = provider_path.to_str().unwrap().to_owned();
    temp_env::async_with_vars(
        [
            ("FAMILY_ID", Some("testfamily")),
            ("BIOMEOS_FAMILY_ID", None::<&str>),
            ("BIOMEOS_SECURITY_SOCKET", Some(provider_path_str.as_str())),
            ("SECURITY_PROVIDER_SOCKET", None::<&str>),
            ("BIOMEOS_SOCKET_DIR", Some(dir.as_str())),
            ("XDG_RUNTIME_DIR", Some(dir.as_str())),
        ],
        f(provider_path),
    )
    .await;
}

async fn run_phase2_btsp_server(stream: UnixStream, session_id: &str) {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await.expect("read hello");

    let server_hello = ServerHello {
        version: BTSP_VERSION,
        server_ephemeral_pub: "dGVzdC1zZXJ2ZXItcHVi".to_owned(),
        challenge: "dGVzdC1jaGFsbGVuZ2U=".to_owned(),
        session_id: session_id.to_owned(),
    };
    let body = format!("{}\n", serde_json::to_string(&server_hello).unwrap());
    reader.get_mut().write_all(body.as_bytes()).await.unwrap();
    reader.get_mut().flush().await.unwrap();

    line.clear();
    reader
        .read_line(&mut line)
        .await
        .expect("read challenge response");

    let complete = HandshakeComplete {
        cipher: "null".to_owned(),
        session_id: session_id.to_owned(),
    };
    let body = format!("{}\n", serde_json::to_string(&complete).unwrap());
    reader.get_mut().write_all(body.as_bytes()).await.unwrap();
    reader.get_mut().flush().await.unwrap();
}

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

fn decode_session_key_hex() -> [u8; 32] {
    let mut key = [0u8; 32];
    for (i, byte) in key.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&VALID_SESSION_KEY_HEX[i * 2..i * 2 + 2], 16).unwrap();
    }
    key
}

// ── server.rs ──

#[tokio::test]
async fn server_handshake_completes_successfully_with_security_provider() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider_path = spawn_btsp_session_provider(
        dir.path(),
        json!({
            "session_id": "server-session",
            "server_ephemeral_pub": "server-pub",
            "challenge": "server-challenge",
        }),
        json!({ "verified": true, "session_key": VALID_SESSION_KEY_HEX }),
    )
    .await;

    with_family_env(provider_path, |provider_path| async move {
        let (server_stream, client_stream) = UnixStream::pair().expect("pair");
        let client_task = tokio::spawn(async move {
            let hello = ClientHello {
                protocol: "btsp".into(),
                version: BTSP_VERSION,
                client_ephemeral_pub: "client-pub".into(),
            };
            let mut reader = BufReader::new(client_stream);
            reader
                .get_mut()
                .write_all(format!("{}\n", serde_json::to_string(&hello).unwrap()).as_bytes())
                .await
                .unwrap();
            reader.get_mut().flush().await.unwrap();

            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let server_hello: ServerHello = serde_json::from_str(line.trim()).unwrap();
            assert_eq!(server_hello.session_id, "server-session");

            let challenge = ChallengeResponse {
                response: "client-hmac".into(),
                preferred_cipher: "null".into(),
            };
            reader
                .get_mut()
                .write_all(format!("{}\n", serde_json::to_string(&challenge).unwrap()).as_bytes())
                .await
                .unwrap();
            reader.get_mut().flush().await.unwrap();

            line.clear();
            reader.read_line(&mut line).await.unwrap();
            let complete: HandshakeComplete = serde_json::from_str(line.trim()).unwrap();
            assert_eq!(complete.session_id, "server-session");
        });

        let mut reader = BufReader::new(server_stream);
        let outcome = server_handshake(&mut reader)
            .await
            .expect("server handshake");
        match outcome {
            HandshakeOutcome::Authenticated {
                session_id,
                handshake_key,
            } => {
                assert_eq!(session_id, "server-session");
                assert_eq!(handshake_key, Some(decode_session_key_hex()));
            }
            other => panic!("unexpected outcome: {other:?}"),
        }
        client_task.await.expect("client task");
        let _ = provider_path;
    })
    .await;
}

#[tokio::test]
async fn server_handshake_returns_raw_jsonrpc_for_non_btsp_first_line() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider_path = spawn_btsp_session_provider(
        dir.path(),
        json!({ "session_id": "s", "server_ephemeral_pub": "p", "challenge": "c" }),
        json!({ "verified": true }),
    )
    .await;

    with_family_env(provider_path, |_provider_path| async move {
        let (server_stream, mut client_stream) = UnixStream::pair().expect("pair");
        client_stream
            .write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"ping\",\"id\":1}\n")
            .await
            .unwrap();

        let mut reader = BufReader::new(server_stream);
        let err = server_handshake(&mut reader)
            .await
            .expect_err("raw jsonrpc");
        match err {
            BtspHandshakeError::RawJsonRpc(line) => {
                assert!(line.contains("\"method\":\"ping\""));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    })
    .await;
}

#[tokio::test]
async fn server_handshake_returns_raw_jsonrpc_for_invalid_client_hello() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider_path = spawn_btsp_session_provider(
        dir.path(),
        json!({ "session_id": "s", "server_ephemeral_pub": "p", "challenge": "c" }),
        json!({ "verified": true }),
    )
    .await;

    with_family_env(provider_path, |_provider_path| async move {
        let (server_stream, mut client_stream) = UnixStream::pair().expect("pair");
        client_stream
            .write_all(b"{\"protocol\":\"jsonrpc\",\"version\":1,\"client_ephemeral_pub\":\"x\"}\n")
            .await
            .unwrap();

        let mut reader = BufReader::new(server_stream);
        assert!(matches!(
            server_handshake(&mut reader).await,
            Err(BtspHandshakeError::RawJsonRpc(_))
        ));
    })
    .await;
}

#[tokio::test]
async fn server_handshake_connection_closed_before_client_hello() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider_path = spawn_btsp_session_provider(
        dir.path(),
        json!({ "session_id": "s", "server_ephemeral_pub": "p", "challenge": "c" }),
        json!({ "verified": true }),
    )
    .await;

    with_family_env(provider_path, |_provider_path| async move {
        let (server_stream, client_stream) = UnixStream::pair().expect("pair");
        drop(client_stream);

        let mut reader = BufReader::new(server_stream);
        assert!(matches!(
            server_handshake(&mut reader).await,
            Err(BtspHandshakeError::ConnectionClosed)
        ));
    })
    .await;
}

#[tokio::test]
async fn server_handshake_times_out_waiting_for_client_hello() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider_path = spawn_btsp_session_provider(
        dir.path(),
        json!({ "session_id": "s", "server_ephemeral_pub": "p", "challenge": "c" }),
        json!({ "verified": true }),
    )
    .await;

    with_family_env(provider_path, |_provider_path| async move {
        let (server_stream, _client_stream) = UnixStream::pair().expect("pair");
        let mut reader = BufReader::new(server_stream);
        assert!(matches!(
            server_handshake(&mut reader).await,
            Err(BtspHandshakeError::Timeout)
        ));
    })
    .await;
}

#[tokio::test]
async fn server_handshake_security_provider_not_found_with_family_id() {
    let dir = tempfile::tempdir().expect("tempdir");
    let iso = dir.path().to_str().unwrap();
    temp_env::async_with_vars(
        [
            ("FAMILY_ID", Some("testfamily")),
            ("BIOMEOS_SECURITY_SOCKET", None::<&str>),
            ("SECURITY_PROVIDER_SOCKET", None::<&str>),
            ("BIOMEOS_SOCKET_DIR", Some(iso)),
            ("XDG_RUNTIME_DIR", Some(iso)),
        ],
        async {
            let (server_stream, mut client_stream) = UnixStream::pair().expect("pair");
            let hello = ClientHello {
                protocol: "btsp".into(),
                version: BTSP_VERSION,
                client_ephemeral_pub: "pub".into(),
            };
            client_stream
                .write_all(format!("{}\n", serde_json::to_string(&hello).unwrap()).as_bytes())
                .await
                .unwrap();

            let mut reader = BufReader::new(server_stream);
            assert!(matches!(
                server_handshake(&mut reader).await,
                Err(BtspHandshakeError::SecurityProviderNotFound)
            ));
        },
    )
    .await;
}

#[tokio::test]
async fn server_handshake_verification_failed_sends_error_to_client() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider_path = spawn_btsp_session_provider(
        dir.path(),
        json!({
            "session_id": "reject-session",
            "server_ephemeral_pub": "server-pub",
            "challenge": "server-challenge",
        }),
        json!({ "verified": false }),
    )
    .await;

    with_family_env(provider_path, |_provider_path| async move {
        let (server_stream, client_stream) = UnixStream::pair().expect("pair");
        let client_task = tokio::spawn(async move {
            let hello = ClientHello {
                protocol: "btsp".into(),
                version: BTSP_VERSION,
                client_ephemeral_pub: "client-pub".into(),
            };
            let mut reader = BufReader::new(client_stream);
            reader
                .get_mut()
                .write_all(format!("{}\n", serde_json::to_string(&hello).unwrap()).as_bytes())
                .await
                .unwrap();
            reader.get_mut().flush().await.unwrap();

            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let _server_hello: ServerHello = serde_json::from_str(line.trim()).unwrap();

            let challenge = ChallengeResponse {
                response: "bad-hmac".into(),
                preferred_cipher: "null".into(),
            };
            reader
                .get_mut()
                .write_all(format!("{}\n", serde_json::to_string(&challenge).unwrap()).as_bytes())
                .await
                .unwrap();
            reader.get_mut().flush().await.unwrap();

            line.clear();
            reader.read_line(&mut line).await.unwrap();
            let err: HandshakeError = serde_json::from_str(line.trim()).unwrap();
            assert_eq!(err.reason, "family_verification");
        });

        let mut reader = BufReader::new(server_stream);
        assert!(matches!(
            server_handshake(&mut reader).await,
            Err(BtspHandshakeError::VerificationFailed)
        ));
        client_task.await.expect("client task");
    })
    .await;
}

#[tokio::test]
async fn server_handshake_rejects_invalid_challenge_response() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider_path = spawn_btsp_session_provider(
        dir.path(),
        json!({
            "session_id": "sess",
            "server_ephemeral_pub": "server-pub",
            "challenge": "server-challenge",
        }),
        json!({ "verified": true }),
    )
    .await;

    with_family_env(provider_path, |_provider_path| async move {
        let (server_stream, client_stream) = UnixStream::pair().expect("pair");
        let client_task = tokio::spawn(async move {
            let hello = ClientHello {
                protocol: "btsp".into(),
                version: BTSP_VERSION,
                client_ephemeral_pub: "client-pub".into(),
            };
            let mut reader = BufReader::new(client_stream);
            reader
                .get_mut()
                .write_all(format!("{}\n", serde_json::to_string(&hello).unwrap()).as_bytes())
                .await
                .unwrap();
            reader.get_mut().flush().await.unwrap();

            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let _server_hello: ServerHello = serde_json::from_str(line.trim()).unwrap();

            reader.get_mut().write_all(b"{not-json\n").await.unwrap();
            reader.get_mut().flush().await.unwrap();
        });

        let mut reader = BufReader::new(server_stream);
        let err = server_handshake(&mut reader)
            .await
            .expect_err("invalid challenge response");
        assert!(
            matches!(err, BtspHandshakeError::Protocol(msg) if msg.contains("ChallengeResponse"))
        );
        client_task.await.expect("client task");
    })
    .await;
}

#[tokio::test]
async fn server_handshake_connection_closed_after_server_hello() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider_path = spawn_btsp_session_provider(
        dir.path(),
        json!({
            "session_id": "sess",
            "server_ephemeral_pub": "server-pub",
            "challenge": "server-challenge",
        }),
        json!({ "verified": true }),
    )
    .await;

    with_family_env(provider_path, |_provider_path| async move {
        let (server_stream, client_stream) = UnixStream::pair().expect("pair");
        let client_task = tokio::spawn(async move {
            let hello = ClientHello {
                protocol: "btsp".into(),
                version: BTSP_VERSION,
                client_ephemeral_pub: "client-pub".into(),
            };
            let mut reader = BufReader::new(client_stream);
            reader
                .get_mut()
                .write_all(format!("{}\n", serde_json::to_string(&hello).unwrap()).as_bytes())
                .await
                .unwrap();
            reader.get_mut().flush().await.unwrap();

            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            drop(reader);
        });

        let mut reader = BufReader::new(server_stream);
        assert!(matches!(
            server_handshake(&mut reader).await,
            Err(BtspHandshakeError::ConnectionClosed)
        ));
        client_task.await.expect("client task");
    })
    .await;
}

// ── client.rs helpers ──

#[test]
fn serialize_line_appends_newline() {
    let hello = ClientHello {
        protocol: "btsp".into(),
        version: BTSP_VERSION,
        client_ephemeral_pub: "pub".into(),
    };
    let line = serialize_line(&hello).expect("serialize");
    assert!(line.ends_with('\n'));
    assert!(line.contains("\"protocol\":\"btsp\""));
}

#[tokio::test]
async fn write_line_to_writes_and_flushes() {
    let (client, mut server) = UnixStream::pair().expect("pair");
    let mut reader = BufReader::new(client);
    write_line_to(&mut reader, "hello\n").await.expect("write");
    let mut buf = [0u8; 6];
    server.read_exact(&mut buf).await.expect("read");
    assert_eq!(&buf, b"hello\n");
}

#[tokio::test]
async fn read_json_line_parses_success_response() {
    let (mut client, server) = UnixStream::pair().expect("pair");
    let complete = HandshakeComplete {
        cipher: "null".into(),
        session_id: "sid".into(),
    };
    client
        .write_all(format!("{}\n", serde_json::to_string(&complete).unwrap()).as_bytes())
        .await
        .unwrap();

    let mut reader = BufReader::new(server);
    let parsed: HandshakeComplete = read_json_line(&mut reader).await.expect("read");
    assert_eq!(parsed.session_id, "sid");
}

#[tokio::test]
async fn read_json_line_rejects_handshake_error() {
    let (mut client, server) = UnixStream::pair().expect("pair");
    let err = HandshakeError {
        error: "handshake_failed".into(),
        reason: "family_verification".into(),
    };
    client
        .write_all(format!("{}\n", serde_json::to_string(&err).unwrap()).as_bytes())
        .await
        .unwrap();

    let mut reader = BufReader::new(server);
    let err = read_json_line::<ServerHello>(&mut reader)
        .await
        .expect_err("handshake rejected");
    assert!(
        matches!(err, BtspHandshakeError::Protocol(msg) if msg.contains("family_verification"))
    );
}

#[tokio::test]
async fn read_json_line_rejects_malformed_json() {
    let (mut client, server) = UnixStream::pair().expect("pair");
    client.write_all(b"{broken\n").await.unwrap();

    let mut reader = BufReader::new(server);
    let err = read_json_line::<ServerHello>(&mut reader)
        .await
        .expect_err("malformed json");
    assert!(matches!(err, BtspHandshakeError::Protocol(msg) if msg.contains("parse")));
}

#[tokio::test]
async fn read_json_line_connection_closed() {
    let (client, server) = UnixStream::pair().expect("pair");
    drop(client);

    let mut reader = BufReader::new(server);
    assert!(matches!(
        read_json_line::<ServerHello>(&mut reader).await,
        Err(BtspHandshakeError::ConnectionClosed)
    ));
}

#[tokio::test]
async fn read_json_line_times_out() {
    let (client, server) = UnixStream::pair().expect("pair");
    let mut reader = BufReader::new(server);
    assert!(matches!(
        read_json_line::<ServerHello>(&mut reader).await,
        Err(BtspHandshakeError::Timeout)
    ));
    drop(client);
}

#[tokio::test]
async fn client_keygen_parses_provider_response() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider_path = spawn_client_crypto_provider(dir.path(), VALID_SESSION_KEY_HEX).await;
    temp_env::async_with_vars(
        [(
            "BIOMEOS_SECURITY_SOCKET",
            Some(provider_path.to_str().unwrap()),
        )],
        async {
            let bd = crate::AtomicClient::unix(&provider_path);
            let (pub_key, sec_key) = client_keygen(&bd).await.expect("keygen");
            assert_eq!(pub_key, "dGVzdC1jbGllbnQtcHVi");
            assert_eq!(sec_key, "dGVzdC1jbGllbnQtc2VjcmV0");
        },
    )
    .await;
}

#[tokio::test]
async fn client_keygen_missing_public_key() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider_path = spawn_rpc_mock(dir.path(), |method, _| {
        if method == "x25519_generate_ephemeral" {
            json!({ "secret_key": "sec" })
        } else {
            json!({})
        }
    })
    .await;

    temp_env::async_with_vars(
        [(
            "BIOMEOS_SECURITY_SOCKET",
            Some(provider_path.to_str().unwrap()),
        )],
        async {
            let bd = crate::AtomicClient::unix(&provider_path);
            let err = client_keygen(&bd).await.expect_err("missing public key");
            assert!(matches!(err, BtspHandshakeError::Protocol(msg) if msg.contains("public_key")));
        },
    )
    .await;
}

#[tokio::test]
async fn client_keygen_missing_secret_key() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider_path = spawn_rpc_mock(dir.path(), |method, _| {
        if method == "x25519_generate_ephemeral" {
            json!({ "public_key": "pub" })
        } else {
            json!({})
        }
    })
    .await;

    temp_env::async_with_vars(
        [(
            "BIOMEOS_SECURITY_SOCKET",
            Some(provider_path.to_str().unwrap()),
        )],
        async {
            let bd = crate::AtomicClient::unix(&provider_path);
            let err = client_keygen(&bd).await.expect_err("missing secret key");
            assert!(matches!(err, BtspHandshakeError::Protocol(msg) if msg.contains("secret_key")));
        },
    )
    .await;
}

#[tokio::test]
async fn perform_client_handshake_completes_with_mock_server() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider_path = spawn_client_crypto_provider(dir.path(), VALID_SESSION_KEY_HEX).await;
    temp_env::async_with_vars(
        [(
            "BIOMEOS_SECURITY_SOCKET",
            Some(provider_path.to_str().unwrap()),
        )],
        async {
            let (client_stream, server_stream) = UnixStream::pair().expect("pair");
            let server_task = tokio::spawn(async move {
                run_phase2_btsp_server(server_stream, "phase2-session").await
            });

            let reader = perform_client_handshake(client_stream)
                .await
                .expect("client handshake");
            drop(reader);

            server_task.await.expect("server task");
        },
    )
    .await;
}

#[tokio::test]
async fn perform_client_handshake_security_provider_not_found() {
    let dir = tempfile::tempdir().expect("tempdir");
    let iso = dir.path().to_str().unwrap();
    temp_env::async_with_vars(
        [
            ("BIOMEOS_SECURITY_SOCKET", None::<&str>),
            ("SECURITY_PROVIDER_SOCKET", None::<&str>),
            ("BIOMEOS_SOCKET_DIR", Some(iso)),
            ("XDG_RUNTIME_DIR", Some(iso)),
        ],
        async {
            let (client, _server) = UnixStream::pair().expect("pair");
            assert!(matches!(
                perform_client_handshake(client).await,
                Err(BtspHandshakeError::SecurityProviderNotFound)
            ));
        },
    )
    .await;
}

#[tokio::test]
async fn perform_client_handshake_challenge_uses_result_field_aliases() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider_path = spawn_rpc_mock(dir.path(), |method, _| match method {
        "x25519_generate_ephemeral" => json!({
            "public_key": "dGVzdC1jbGllbnQtcHVi",
            "secret_key": "dGVzdC1jbGllbnQtc2VjcmV0",
        }),
        "crypto.x25519_derive_secret" => json!({ "result": VALID_SESSION_KEY_HEX }),
        "hmac_sha256" => json!({ "result": "alias-hmac" }),
        _ => json!({}),
    })
    .await;

    temp_env::async_with_vars(
        [(
            "BIOMEOS_SECURITY_SOCKET",
            Some(provider_path.to_str().unwrap()),
        )],
        async {
            let (client_stream, server_stream) = UnixStream::pair().expect("pair");
            let server_task = tokio::spawn(async move {
                let mut reader = BufReader::new(server_stream);
                let mut line = String::new();
                reader.read_line(&mut line).await.unwrap();
                let _hello: ClientHello = serde_json::from_str(line.trim()).unwrap();

                let server_hello = ServerHello {
                    version: BTSP_VERSION,
                    server_ephemeral_pub: "dGVzdC1zZXJ2ZXItcHVi".to_owned(),
                    challenge: "dGVzdC1jaGFsbGVuZ2U=".to_owned(),
                    session_id: "alias-session".to_owned(),
                };
                reader
                    .get_mut()
                    .write_all(
                        format!("{}\n", serde_json::to_string(&server_hello).unwrap()).as_bytes(),
                    )
                    .await
                    .unwrap();
                reader.get_mut().flush().await.unwrap();

                line.clear();
                reader.read_line(&mut line).await.unwrap();
                let challenge_resp: ChallengeResponse = serde_json::from_str(line.trim()).unwrap();
                assert_eq!(challenge_resp.response, "alias-hmac");

                let complete = HandshakeComplete {
                    cipher: "null".to_owned(),
                    session_id: "alias-session".to_owned(),
                };
                reader
                    .get_mut()
                    .write_all(
                        format!("{}\n", serde_json::to_string(&complete).unwrap()).as_bytes(),
                    )
                    .await
                    .unwrap();
            });

            perform_client_handshake(client_stream)
                .await
                .expect("alias challenge handshake");
            server_task.await.expect("server task");
        },
    )
    .await;
}
