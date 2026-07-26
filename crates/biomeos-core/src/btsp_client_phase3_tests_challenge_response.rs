// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::or_fun_call,
    reason = "test assertions"
)]

use super::super::*;
use super::{VALID_SHARED_SECRET_HEX, with_security_provider};
use crate::btsp_client::{BTSP_VERSION, ServerHello};
use biomeos_types::JsonRpcResponse;
use serde_json::json;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::oneshot;

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
