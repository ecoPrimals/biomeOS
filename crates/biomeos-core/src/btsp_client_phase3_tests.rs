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
use crate::btsp_client::{BTSP_VERSION, HandshakeComplete, HandshakeError, ServerHello};
use base64::Engine;
use biomeos_types::JsonRpcResponse;
use serde_json::json;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::oneshot;

/// 32-byte shared secret encoded as hex (used for Phase 3 HKDF input).
const VALID_SHARED_SECRET_HEX: &str =
    "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20";

fn handshake_key_from_hex() -> [u8; 32] {
    decode_shared_secret_to_key(VALID_SHARED_SECRET_HEX).expect("valid test hex key")
}

async fn spawn_security_provider_mock(dir: &Path, shared_secret_hex: &str) -> PathBuf {
    let path = dir.join("security-provider.sock");
    let listener = UnixListener::bind(&path).expect("bind security provider mock");
    let shared_secret_hex = shared_secret_hex.to_owned();
    let (ready_tx, ready_rx) = oneshot::channel();

    tokio::spawn(async move {
        ready_tx.send(()).expect("signal security mock ready");
        loop {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            let shared_secret_hex = shared_secret_hex.clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_err() {
                    return;
                }
                let Ok(req) = serde_json::from_str::<serde_json::Value>(line.trim()) else {
                    return;
                };
                let id = req.get("id").cloned().unwrap_or(json!(1));
                let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
                let result = match method {
                    "x25519_generate_ephemeral" => json!({
                        "public_key": "dGVzdC1jbGllbnQtcHVi",
                        "secret_key": "dGVzdC1jbGllbnQtc2VjcmV0",
                    }),
                    "crypto.x25519_derive_secret" => json!({
                        "shared_secret": shared_secret_hex,
                    }),
                    "hmac_sha256" => json!({
                        "hmac": "dGVzdC1obWFjLXJlc3BvbnNl",
                    }),
                    _ => json!({ "error": format!("unexpected method: {method}") }),
                };
                let response = JsonRpcResponse::success(id, result);
                let mut stream = reader.into_inner();
                let body = format!("{}\n", serde_json::to_string(&response).expect("serialize"));
                let _ = stream.write_all(body.as_bytes()).await;
                let _ = stream.flush().await;
            });
        }
    });

    ready_rx
        .await
        .expect("security provider mock failed to start");
    path
}

enum Phase2Behavior {
    Success { session_id: String },
    Reject { reason: String },
    CloseAfterHello,
}

enum NegotiateBehavior {
    Encrypted { server_nonce_hex: String },
    NullCipher,
    JsonRpcError { message: String },
    MalformedJson,
    CloseWithoutResponse,
    InvalidServerNonce,
}

struct BtspServerConfig {
    phase2: Phase2Behavior,
    negotiate: NegotiateBehavior,
}

async fn run_btsp_server(stream: UnixStream, config: BtspServerConfig) {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    if reader.read_line(&mut line).await.is_err() {
        return;
    }

    match config.phase2 {
        Phase2Behavior::CloseAfterHello => return,
        Phase2Behavior::Reject { reason } => {
            let err = HandshakeError {
                error: "handshake_failed".to_owned(),
                reason,
            };
            let body = format!("{}\n", serde_json::to_string(&err).unwrap());
            let _ = reader.get_mut().write_all(body.as_bytes()).await;
            return;
        }
        Phase2Behavior::Success { session_id } => {
            let server_hello = ServerHello {
                version: BTSP_VERSION,
                server_ephemeral_pub: "dGVzdC1zZXJ2ZXItcHVi".to_owned(),
                challenge: "dGVzdC1jaGFsbGVuZ2U=".to_owned(),
                session_id: session_id.clone(),
            };
            let body = format!("{}\n", serde_json::to_string(&server_hello).unwrap());
            let _ = reader.get_mut().write_all(body.as_bytes()).await;
            let _ = reader.get_mut().flush().await;

            line.clear();
            if reader.read_line(&mut line).await.is_err() {
                return;
            }

            let complete = HandshakeComplete {
                cipher: "chacha20-poly1305".to_owned(),
                session_id,
            };
            let body = format!("{}\n", serde_json::to_string(&complete).unwrap());
            let _ = reader.get_mut().write_all(body.as_bytes()).await;
            let _ = reader.get_mut().flush().await;
        }
    }

    line.clear();
    if reader.read_line(&mut line).await.is_err() {
        return;
    }

    match config.negotiate {
        NegotiateBehavior::CloseWithoutResponse => {}
        NegotiateBehavior::MalformedJson => {
            let _ = reader.get_mut().write_all(b"not-json\n").await;
        }
        NegotiateBehavior::JsonRpcError { message } => {
            let resp = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "error": { "code": -32000, "message": message },
            });
            let body = format!("{}\n", serde_json::to_string(&resp).unwrap());
            let _ = reader.get_mut().write_all(body.as_bytes()).await;
        }
        NegotiateBehavior::NullCipher => {
            let resp = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "cipher": "null",
                    "server_nonce": "0102030405060708090a0b0c",
                },
            });
            let body = format!("{}\n", serde_json::to_string(&resp).unwrap());
            let _ = reader.get_mut().write_all(body.as_bytes()).await;
        }
        NegotiateBehavior::InvalidServerNonce => {
            let resp = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "cipher": "chacha20-poly1305",
                    "server_nonce": "!!!not-valid-base64-or-hex!!!",
                },
            });
            let body = format!("{}\n", serde_json::to_string(&resp).unwrap());
            let _ = reader.get_mut().write_all(body.as_bytes()).await;
        }
        NegotiateBehavior::Encrypted { server_nonce_hex } => {
            let resp = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": {
                    "cipher": "chacha20-poly1305",
                    "server_nonce": server_nonce_hex,
                },
            });
            let body = format!("{}\n", serde_json::to_string(&resp).unwrap());
            let _ = reader.get_mut().write_all(body.as_bytes()).await;
        }
    }
}

async fn with_security_provider<F, Fut>(shared_secret_hex: &str, f: F)
where
    F: FnOnce(PathBuf) -> Fut,
    Fut: std::future::Future<Output = ()> + Send,
{
    let dir = tempfile::tempdir().expect("tempdir");
    let iso = dir.path().to_str().unwrap().to_string();
    let provider_path = spawn_security_provider_mock(dir.path(), shared_secret_hex).await;
    let provider_path_str = provider_path.to_str().unwrap().to_string();
    temp_env::async_with_vars(
        [
            ("BIOMEOS_SECURITY_SOCKET", Some(provider_path_str.as_str())),
            ("SECURITY_PROVIDER_SOCKET", None::<&str>),
            ("FAMILY_ID", None::<&str>),
            ("BIOMEOS_FAMILY_ID", None::<&str>),
            ("BIOMEOS_SOCKET_DIR", Some(iso.as_str())),
            ("XDG_RUNTIME_DIR", Some(iso.as_str())),
        ],
        f(provider_path),
    )
    .await;
}

// ── decode_shared_secret_to_key ──

#[test]
fn decode_shared_secret_valid_hex_produces_32_byte_key() {
    let key = decode_shared_secret_to_key(VALID_SHARED_SECRET_HEX).expect("valid hex");
    assert_eq!(key[0], 0x01);
    assert_eq!(key[31], 0x20);
}

#[test]
fn decode_shared_secret_rejects_wrong_length() {
    assert!(decode_shared_secret_to_key("0102").is_none());
    assert!(decode_shared_secret_to_key("").is_none());
}

#[test]
fn decode_shared_secret_rejects_invalid_hex() {
    assert!(decode_shared_secret_to_key("zzzz").is_none());
    let bad = "gg".repeat(32);
    assert!(decode_shared_secret_to_key(&bad).is_none());
}

#[test]
fn decode_shared_secret_rejects_non_hex_ascii() {
    assert!(decode_shared_secret_to_key("not-a-valid-shared-secret-hex!!!").is_none());
}

// ── client_negotiate ──

#[tokio::test]
async fn client_negotiate_derives_session_keys_on_success() {
    let (client, server) = UnixStream::pair().expect("pair");
    let server_nonce_hex = "0a0b0c0d0e0f101112131415161718191a1b1c1d1e1f2021222324252627";
    tokio::spawn(async move {
        let mut reader = BufReader::new(server);
        let mut line = String::new();
        reader.read_line(&mut line).await.expect("read negotiate");
        let req: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(req["method"], "btsp.negotiate");
        assert_eq!(req["params"]["preferred_cipher"], "chacha20-poly1305");
        assert_eq!(req["params"]["session_id"], "sess-123");
        assert!(req["params"]["client_nonce"].as_str().is_some());
        let resp = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "cipher": "chacha20-poly1305",
                "server_nonce": server_nonce_hex,
            },
        });
        let mut stream = reader.into_inner();
        stream
            .write_all(format!("{}\n", serde_json::to_string(&resp).unwrap()).as_bytes())
            .await
            .unwrap();
    });

    let mut reader = BufReader::new(client);
    let keys = client_negotiate(&mut reader, "sess-123", &handshake_key_from_hex())
        .await
        .expect("negotiate ok");
    assert_ne!(keys.client_to_server, [0u8; 32]);
    assert_ne!(keys.server_to_client, [0u8; 32]);
    assert_ne!(keys.client_to_server, keys.server_to_client);
}

#[tokio::test]
async fn client_negotiate_accepts_base64_server_nonce() {
    let (client, server) = UnixStream::pair().expect("pair");
    let server_nonce = vec![9u8; 16];
    let server_nonce_b64 = base64::engine::general_purpose::STANDARD.encode(&server_nonce);
    tokio::spawn(async move {
        let mut reader = BufReader::new(server);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let resp = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "cipher": "ChaCha20-Poly1305",
                "server_nonce": server_nonce_b64,
            },
        });
        let mut stream = reader.into_inner();
        stream
            .write_all(format!("{}\n", serde_json::to_string(&resp).unwrap()).as_bytes())
            .await
            .unwrap();
    });

    let mut reader = BufReader::new(client);
    let keys = client_negotiate(&mut reader, "sid", &handshake_key_from_hex())
        .await
        .expect("base64 server nonce");
    assert_ne!(keys.client_to_server, keys.server_to_client);
}

#[tokio::test]
async fn client_negotiate_rejects_null_cipher() {
    let (client, server) = UnixStream::pair().expect("pair");
    tokio::spawn(async move {
        let mut reader = BufReader::new(server);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let resp = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": { "cipher": "null", "server_nonce": "0102" },
        });
        let mut stream = reader.into_inner();
        stream
            .write_all(format!("{}\n", serde_json::to_string(&resp).unwrap()).as_bytes())
            .await
            .unwrap();
    });

    let mut reader = BufReader::new(client);
    let err = client_negotiate(&mut reader, "sid", &handshake_key_from_hex())
        .await
        .expect_err("null cipher");
    assert!(matches!(err, BtspHandshakeError::Protocol(msg) if msg.contains("null cipher")));
}

#[tokio::test]
async fn client_negotiate_rejects_jsonrpc_error() {
    let (client, server) = UnixStream::pair().expect("pair");
    tokio::spawn(async move {
        let mut reader = BufReader::new(server);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let resp = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": { "message": "negotiate not supported" },
        });
        let mut stream = reader.into_inner();
        stream
            .write_all(format!("{}\n", serde_json::to_string(&resp).unwrap()).as_bytes())
            .await
            .unwrap();
    });

    let mut reader = BufReader::new(client);
    let err = client_negotiate(&mut reader, "sid", &handshake_key_from_hex())
        .await
        .expect_err("jsonrpc error");
    assert!(matches!(err, BtspHandshakeError::Protocol(msg) if msg.contains("negotiate rejected")));
}

#[tokio::test]
async fn client_negotiate_rejects_malformed_json() {
    let (client, server) = UnixStream::pair().expect("pair");
    tokio::spawn(async move {
        let mut reader = BufReader::new(server);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let mut stream = reader.into_inner();
        stream.write_all(b"{broken\n").await.unwrap();
    });

    let mut reader = BufReader::new(client);
    let err = client_negotiate(&mut reader, "sid", &handshake_key_from_hex())
        .await
        .expect_err("malformed json");
    assert!(matches!(err, BtspHandshakeError::Protocol(msg) if msg.contains("parse negotiate")));
}

#[tokio::test]
async fn client_negotiate_connection_closed_returns_error() {
    let (client, server) = UnixStream::pair().expect("pair");
    tokio::spawn(async move {
        let mut reader = BufReader::new(server);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        drop(reader);
    });

    let mut reader = BufReader::new(client);
    let err = client_negotiate(&mut reader, "sid", &handshake_key_from_hex())
        .await
        .expect_err("connection closed");
    assert!(matches!(err, BtspHandshakeError::ConnectionClosed));
}

#[tokio::test]
async fn client_negotiate_missing_server_nonce_is_protocol_error() {
    let (client, server) = UnixStream::pair().expect("pair");
    tokio::spawn(async move {
        let mut reader = BufReader::new(server);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let resp = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": { "cipher": "chacha20-poly1305" },
        });
        let mut stream = reader.into_inner();
        stream
            .write_all(format!("{}\n", serde_json::to_string(&resp).unwrap()).as_bytes())
            .await
            .unwrap();
    });

    let mut reader = BufReader::new(client);
    let err = client_negotiate(&mut reader, "sid", &handshake_key_from_hex())
        .await
        .expect_err("missing server_nonce");
    assert!(
        matches!(err, BtspHandshakeError::Protocol(msg) if msg.contains("missing server_nonce"))
    );
}

#[tokio::test]
async fn client_negotiate_invalid_server_nonce_encoding() {
    let (client, server) = UnixStream::pair().expect("pair");
    tokio::spawn(async move {
        let mut reader = BufReader::new(server);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let resp = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "cipher": "chacha20-poly1305",
                "server_nonce": "!!!",
            },
        });
        let mut stream = reader.into_inner();
        stream
            .write_all(format!("{}\n", serde_json::to_string(&resp).unwrap()).as_bytes())
            .await
            .unwrap();
    });

    let mut reader = BufReader::new(client);
    let err = client_negotiate(&mut reader, "sid", &handshake_key_from_hex())
        .await
        .expect_err("bad nonce");
    assert!(
        matches!(err, BtspHandshakeError::Protocol(msg) if msg.contains("decode server_nonce"))
    );
}

// ── client_challenge_response_with_key ──

#[tokio::test]
async fn client_challenge_response_with_key_success() {
    with_security_provider(VALID_SHARED_SECRET_HEX, |provider_path| async move {
        let bd = crate::AtomicClient::unix(&provider_path);
        let server_hello = ServerHello {
            version: BTSP_VERSION,
            server_ephemeral_pub: "dGVzdC1zZXJ2ZXItcHVi".to_owned(),
            challenge: "dGVzdC1jaGFsbGVuZ2U=".to_owned(),
            session_id: "sess".to_owned(),
        };
        let (hmac, shared) =
            client_challenge_response_with_key(&bd, "dGVzdC1jbGllbnQtc2VjcmV0", &server_hello)
                .await
                .expect("challenge response");
        assert_eq!(shared, VALID_SHARED_SECRET_HEX);
        assert_eq!(hmac, "dGVzdC1obWFjLXJlc3BvbnNl");
    })
    .await;
}

async fn spawn_single_method_security_provider(
    dir: &Path,
    method: &str,
    result: serde_json::Value,
) -> PathBuf {
    let path = dir.join("security-single.sock");
    let listener = UnixListener::bind(&path).expect("bind");
    let method = method.to_owned();
    let (ready_tx, ready_rx) = oneshot::channel();
    tokio::spawn(async move {
        ready_tx.send(()).expect("ready");
        if let Ok((stream, _)) = listener.accept().await {
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.ok();
            let req: serde_json::Value = serde_json::from_str(line.trim()).unwrap_or(json!({}));
            let id = req.get("id").cloned().unwrap_or(json!(1));
            let response = JsonRpcResponse::success(id, result);
            let mut stream = reader.into_inner();
            let body = format!("{}\n", serde_json::to_string(&response).unwrap());
            let _ = stream.write_all(body.as_bytes()).await;
            let _ = method; // keep method param for call-site clarity
        }
    });
    ready_rx.await.expect("provider ready");
    path
}

#[tokio::test]
async fn client_challenge_response_missing_shared_secret() {
    let dir = tempfile::tempdir().expect("tempdir");
    let provider_path = spawn_single_method_security_provider(
        dir.path(),
        "crypto.x25519_derive_secret",
        json!({ "status": "ok" }),
    )
    .await;
    temp_env::async_with_vars(
        [(
            "BIOMEOS_SECURITY_SOCKET",
            Some(provider_path.to_str().unwrap()),
        )],
        async {
            let bd = crate::AtomicClient::unix(&provider_path);
            let server_hello = ServerHello {
                version: 1,
                server_ephemeral_pub: "pub".to_owned(),
                challenge: "chal".to_owned(),
                session_id: "s".to_owned(),
            };
            let err = client_challenge_response_with_key(&bd, "sec", &server_hello)
                .await
                .expect_err("missing shared secret");
            assert!(
                matches!(err, BtspHandshakeError::Protocol(msg) if msg.contains("shared_secret"))
            );
        },
    )
    .await;
}

#[tokio::test]
async fn client_challenge_response_missing_hmac() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("security-hmac-missing.sock");
    let listener = UnixListener::bind(&path).expect("bind");
    let (ready_tx, ready_rx) = oneshot::channel();
    tokio::spawn(async move {
        ready_tx.send(()).expect("ready");
        for call in 0..2 {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.ok();
            let req: serde_json::Value = serde_json::from_str(line.trim()).unwrap_or(json!({}));
            let id = req.get("id").cloned().unwrap_or(json!(1));
            let result = if call == 0 {
                json!({ "shared_secret": VALID_SHARED_SECRET_HEX })
            } else {
                json!({ "status": "ok" })
            };
            let response = JsonRpcResponse::success(id, result);
            let mut stream = reader.into_inner();
            let body = format!("{}\n", serde_json::to_string(&response).unwrap());
            let _ = stream.write_all(body.as_bytes()).await;
        }
    });
    ready_rx.await.expect("provider ready");

    temp_env::async_with_vars(
        [("BIOMEOS_SECURITY_SOCKET", Some(path.to_str().unwrap()))],
        async {
            let bd = crate::AtomicClient::unix(&path);
            let server_hello = ServerHello {
                version: 1,
                server_ephemeral_pub: "pub".to_owned(),
                challenge: "chal".to_owned(),
                session_id: "s".to_owned(),
            };
            let err = client_challenge_response_with_key(&bd, "sec", &server_hello)
                .await
                .expect_err("missing hmac");
            assert!(matches!(err, BtspHandshakeError::Protocol(msg) if msg.contains("hmac")));
        },
    )
    .await;
}

#[tokio::test]
async fn client_challenge_response_accepts_result_field_aliases() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("alias-provider.sock");
    let listener = UnixListener::bind(&path).expect("bind alias provider");
    let (ready_tx, ready_rx) = oneshot::channel();
    tokio::spawn(async move {
        ready_tx.send(()).expect("ready");
        for _ in 0..2 {
            let Ok((stream, _)) = listener.accept().await else {
                break;
            };
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.ok();
            let req: serde_json::Value = serde_json::from_str(line.trim()).unwrap_or(json!({}));
            let id = req.get("id").cloned().unwrap_or(json!(1));
            let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
            let result = match method {
                "crypto.x25519_derive_secret" => json!({ "result": VALID_SHARED_SECRET_HEX }),
                "hmac_sha256" => json!({ "result": "alias-hmac" }),
                _ => json!({}),
            };
            let response = JsonRpcResponse::success(id, result);
            let mut stream = reader.into_inner();
            let body = format!("{}\n", serde_json::to_string(&response).unwrap());
            let _ = stream.write_all(body.as_bytes()).await;
        }
    });
    ready_rx.await.expect("alias provider ready");

    temp_env::async_with_vars(
        [("BIOMEOS_SECURITY_SOCKET", Some(path.to_str().unwrap()))],
        async {
            let bd = crate::AtomicClient::unix(&path);
            let server_hello = ServerHello {
                version: 1,
                server_ephemeral_pub: "pub".to_owned(),
                challenge: "chal".to_owned(),
                session_id: "s".to_owned(),
            };
            let (hmac, shared) = client_challenge_response_with_key(&bd, "sec", &server_hello)
                .await
                .expect("alias fields");
            assert_eq!(shared, VALID_SHARED_SECRET_HEX);
            assert_eq!(hmac, "alias-hmac");
        },
    )
    .await;
}

// ── perform_client_handshake_phase3 (integration) ──

#[tokio::test]
async fn perform_client_handshake_phase3_security_provider_not_found() {
    let dir = tempfile::tempdir().expect("tempdir");
    let iso = dir.path().to_str().unwrap();
    temp_env::async_with_vars(
        [
            ("BIOMEOS_SECURITY_SOCKET", None::<&str>),
            ("SECURITY_PROVIDER_SOCKET", None::<&str>),
            ("FAMILY_ID", None::<&str>),
            ("BIOMEOS_FAMILY_ID", None::<&str>),
            ("BIOMEOS_SOCKET_DIR", Some(iso)),
            ("XDG_RUNTIME_DIR", Some(iso)),
        ],
        async {
            let (client, _server) = UnixStream::pair().expect("pair");
            assert!(matches!(
                perform_client_handshake_phase3(client).await,
                Err(BtspHandshakeError::SecurityProviderNotFound)
            ));
        },
    )
    .await;
}

#[tokio::test]
async fn perform_client_handshake_phase3_establishes_encrypted_session() {
    with_security_provider(VALID_SHARED_SECRET_HEX, |_provider_path| async move {
        let (client, server) = UnixStream::pair().expect("pair");
        let server_nonce_hex = "aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899";
        let server_task = tokio::spawn(async move {
            run_btsp_server(
                server,
                BtspServerConfig {
                    phase2: Phase2Behavior::Success {
                        session_id: "phase3-session".to_owned(),
                    },
                    negotiate: NegotiateBehavior::Encrypted {
                        server_nonce_hex: server_nonce_hex.to_owned(),
                    },
                },
            )
            .await;
        });

        let outcome = perform_client_handshake_phase3(client)
            .await
            .expect("handshake ok");
        match outcome {
            ClientPhase3Outcome::Encrypted { keys, stream: _ } => {
                assert_ne!(keys.client_to_server, [0u8; 32]);
                assert_ne!(keys.server_to_client, [0u8; 32]);
                assert_ne!(keys.client_to_server, keys.server_to_client);
            }
            ClientPhase3Outcome::Plaintext { .. } => panic!("expected encrypted outcome"),
        }
        server_task.await.expect("server");
    })
    .await;
}

#[tokio::test]
async fn perform_client_handshake_phase3_plaintext_when_shared_secret_not_hex_key() {
    with_security_provider(
        "not-valid-32-byte-hex-key-material!!!",
        |provider_path| async move {
            let (client, server) = UnixStream::pair().expect("pair");
            let _server_task = tokio::spawn(async move {
                run_btsp_server(
                    server,
                    BtspServerConfig {
                        phase2: Phase2Behavior::Success {
                            session_id: "plain-session".to_owned(),
                        },
                        negotiate: NegotiateBehavior::CloseWithoutResponse,
                    },
                )
                .await;
            });

            let outcome = perform_client_handshake_phase3(client)
                .await
                .expect("handshake ok");
            assert!(matches!(outcome, ClientPhase3Outcome::Plaintext { .. }));
            let _ = provider_path;
        },
    )
    .await;
}

#[tokio::test]
async fn perform_client_handshake_phase3_plaintext_when_negotiate_returns_null_cipher() {
    with_security_provider(VALID_SHARED_SECRET_HEX, |provider_path| async move {
        let (client, server) = UnixStream::pair().expect("pair");
        let server_task = tokio::spawn(async move {
            run_btsp_server(
                server,
                BtspServerConfig {
                    phase2: Phase2Behavior::Success {
                        session_id: "null-cipher".to_owned(),
                    },
                    negotiate: NegotiateBehavior::NullCipher,
                },
            )
            .await;
        });

        let outcome = perform_client_handshake_phase3(client)
            .await
            .expect("handshake ok");
        assert!(matches!(outcome, ClientPhase3Outcome::Plaintext { .. }));
        server_task.await.expect("server");
        let _ = provider_path;
    })
    .await;
}

#[tokio::test]
async fn perform_client_handshake_phase3_plaintext_when_negotiate_has_invalid_server_nonce() {
    with_security_provider(VALID_SHARED_SECRET_HEX, |provider_path| async move {
        let (client, server) = UnixStream::pair().expect("pair");
        let server_task = tokio::spawn(async move {
            run_btsp_server(
                server,
                BtspServerConfig {
                    phase2: Phase2Behavior::Success {
                        session_id: "bad-server-nonce".to_owned(),
                    },
                    negotiate: NegotiateBehavior::InvalidServerNonce,
                },
            )
            .await;
        });

        let outcome = perform_client_handshake_phase3(client)
            .await
            .expect("handshake ok");
        assert!(matches!(outcome, ClientPhase3Outcome::Plaintext { .. }));
        server_task.await.expect("server");
        let _ = provider_path;
    })
    .await;
}

#[tokio::test]
async fn perform_client_handshake_phase3_plaintext_when_negotiate_rejected() {
    with_security_provider(VALID_SHARED_SECRET_HEX, |provider_path| async move {
        let (client, server) = UnixStream::pair().expect("pair");
        let server_task = tokio::spawn(async move {
            run_btsp_server(
                server,
                BtspServerConfig {
                    phase2: Phase2Behavior::Success {
                        session_id: "reject-negotiate".to_owned(),
                    },
                    negotiate: NegotiateBehavior::JsonRpcError {
                        message: "phase 3 unavailable".to_owned(),
                    },
                },
            )
            .await;
        });

        let outcome = perform_client_handshake_phase3(client)
            .await
            .expect("handshake ok");
        assert!(matches!(outcome, ClientPhase3Outcome::Plaintext { .. }));
        server_task.await.expect("server");
        let _ = provider_path;
    })
    .await;
}

#[tokio::test]
async fn perform_client_handshake_phase3_plaintext_on_malformed_negotiate_response() {
    with_security_provider(VALID_SHARED_SECRET_HEX, |provider_path| async move {
        let (client, server) = UnixStream::pair().expect("pair");
        let server_task = tokio::spawn(async move {
            run_btsp_server(
                server,
                BtspServerConfig {
                    phase2: Phase2Behavior::Success {
                        session_id: "bad-negotiate-json".to_owned(),
                    },
                    negotiate: NegotiateBehavior::MalformedJson,
                },
            )
            .await;
        });

        let outcome = perform_client_handshake_phase3(client)
            .await
            .expect("handshake ok");
        assert!(matches!(outcome, ClientPhase3Outcome::Plaintext { .. }));
        server_task.await.expect("server");
        let _ = provider_path;
    })
    .await;
}

#[tokio::test]
async fn perform_client_handshake_phase3_rejects_phase2_handshake_failure() {
    with_security_provider(VALID_SHARED_SECRET_HEX, |provider_path| async move {
        let (client, server) = UnixStream::pair().expect("pair");
        let server_task = tokio::spawn(async move {
            run_btsp_server(
                server,
                BtspServerConfig {
                    phase2: Phase2Behavior::Reject {
                        reason: "family_verification".to_owned(),
                    },
                    negotiate: NegotiateBehavior::CloseWithoutResponse,
                },
            )
            .await;
        });

        assert!(matches!(
            perform_client_handshake_phase3(client).await,
            Err(BtspHandshakeError::Protocol(msg)) if msg.contains("family_verification")
        ));
        server_task.await.expect("server");
        let _ = provider_path;
    })
    .await;
}

#[tokio::test]
async fn perform_client_handshake_phase3_connection_closed_during_phase2() {
    with_security_provider(VALID_SHARED_SECRET_HEX, |provider_path| async move {
        let (client, server) = UnixStream::pair().expect("pair");
        let server_task = tokio::spawn(async move {
            run_btsp_server(
                server,
                BtspServerConfig {
                    phase2: Phase2Behavior::CloseAfterHello,
                    negotiate: NegotiateBehavior::CloseWithoutResponse,
                },
            )
            .await;
        });

        assert!(matches!(
            perform_client_handshake_phase3(client).await,
            Err(BtspHandshakeError::ConnectionClosed | BtspHandshakeError::Timeout)
        ));
        server_task.await.expect("server");
        let _ = provider_path;
    })
    .await;
}

#[tokio::test]
async fn client_phase3_outcome_variants_are_constructible() {
    let (s1, _s2) = UnixStream::pair().expect("pair");
    let keys = crate::btsp_crypto::derive_session_keys(&[1u8; 32], &[2u8; 8], &[3u8; 8]);
    let encrypted = ClientPhase3Outcome::Encrypted { keys, stream: s1 };
    let (s3, _s4) = UnixStream::pair().expect("pair");
    let plaintext = ClientPhase3Outcome::Plaintext { stream: s3 };
    assert!(matches!(encrypted, ClientPhase3Outcome::Encrypted { .. }));
    assert!(matches!(plaintext, ClientPhase3Outcome::Plaintext { .. }));
}
