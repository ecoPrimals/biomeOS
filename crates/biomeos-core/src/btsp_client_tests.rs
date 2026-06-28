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
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
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

fn decode_session_key_hex() -> [u8; 32] {
    let mut key = [0u8; 32];
    for (i, byte) in key.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&VALID_SESSION_KEY_HEX[i * 2..i * 2 + 2], 16).unwrap();
    }
    key
}

#[path = "btsp_client_tests_family.rs"]
mod family;

#[path = "btsp_client_tests_types.rs"]
mod types;

#[path = "btsp_client_tests_provider.rs"]
mod provider;

#[path = "btsp_client_tests_server.rs"]
mod server;

#[path = "btsp_client_tests_client.rs"]
mod client;
