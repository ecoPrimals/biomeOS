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
    VALID_SESSION_KEY_HEX, decode_session_key_hex, spawn_btsp_session_provider, with_family_env,
};
use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

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
