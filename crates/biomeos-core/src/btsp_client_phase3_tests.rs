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
use biomeos_types::JsonRpcResponse;
use serde_json::json;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::oneshot;

/// 32-byte shared secret encoded as hex (used for Phase 3 HKDF input).
pub(super) const VALID_SHARED_SECRET_HEX: &str =
    "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20";

pub(super) fn handshake_key_from_hex() -> [u8; 32] {
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

pub(super) enum Phase2Behavior {
    Success { session_id: String },
    Reject { reason: String },
    CloseAfterHello,
}

pub(super) enum NegotiateBehavior {
    Encrypted { server_nonce_hex: String },
    NullCipher,
    JsonRpcError { message: String },
    MalformedJson,
    CloseWithoutResponse,
    InvalidServerNonce,
}

pub(super) struct BtspServerConfig {
    pub phase2: Phase2Behavior,
    pub negotiate: NegotiateBehavior,
}

pub(super) async fn run_btsp_server(stream: UnixStream, config: BtspServerConfig) {
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

pub(super) async fn with_security_provider<F, Fut>(shared_secret_hex: &str, f: F)
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

#[path = "btsp_client_phase3_tests_decode.rs"]
mod decode;

#[path = "btsp_client_phase3_tests_negotiate.rs"]
mod negotiate;

#[path = "btsp_client_phase3_tests_challenge_response.rs"]
mod challenge_response;

#[path = "btsp_client_phase3_tests_handshake.rs"]
mod handshake;
