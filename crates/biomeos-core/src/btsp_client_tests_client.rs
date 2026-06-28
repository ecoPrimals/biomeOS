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
    VALID_SESSION_KEY_HEX, run_phase2_btsp_server, spawn_client_crypto_provider, spawn_rpc_mock,
};
use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

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
                run_phase2_btsp_server(server_stream, "phase2-session").await;
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
