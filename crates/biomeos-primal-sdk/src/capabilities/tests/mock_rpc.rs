// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use biomeos_types::{JsonRpcRequest, JsonRpcResponse};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

pub(super) async fn serve_one_jsonrpc_response(listener: UnixListener, reply: serde_json::Value) {
    let (stream, _) = listener.accept().await.expect("accept");
    let (mut read_half, mut write_half) = stream.into_split();
    let mut line = String::new();
    BufReader::new(&mut read_half)
        .read_line(&mut line)
        .await
        .expect("read line");
    let req: JsonRpcRequest = serde_json::from_str(line.trim()).expect("parse request");
    let id = req.id.clone().unwrap_or(serde_json::Value::Null);
    let resp = JsonRpcResponse {
        jsonrpc: biomeos_types::JsonRpcVersion,
        result: Some(reply),
        error: None,
        id,
    };
    let body = serde_json::to_string(&resp).expect("serialize");
    write_half.write_all(body.as_bytes()).await.expect("write");
    write_half.write_all(b"\n").await.expect("newline");
    write_half.shutdown().await.ok();
}

#[tokio::test]
async fn test_crypto_sign_success_via_mock_server() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("mock.sock");
    let sig = base64_encode(b"signed-payload");
    let listener = UnixListener::bind(&sock).expect("bind");
    let server = tokio::spawn(async move {
        serve_one_jsonrpc_response(listener, serde_json::json!({ "signature": sig })).await;
    });

    let client = CapabilityClient::new(&sock).with_timeout(Duration::from_secs(5));
    let out = client.crypto_sign(b"payload").await.expect("sign ok");
    assert_eq!(out.as_ref(), b"signed-payload");
    server.await.expect("server join");
}

#[tokio::test]
async fn test_capability_call_rpc_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("err.sock");
    let listener = UnixListener::bind(&sock).expect("bind");
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let (mut read_half, mut write_half) = stream.into_split();
        let mut line = String::new();
        BufReader::new(&mut read_half)
            .read_line(&mut line)
            .await
            .expect("read");
        let req: JsonRpcRequest = serde_json::from_str(line.trim()).expect("parse");
        let id = req.id.clone().unwrap_or(serde_json::Value::Null);
        let resp = JsonRpcResponse {
            jsonrpc: biomeos_types::JsonRpcVersion,
            result: None,
            error: Some(biomeos_types::JsonRpcError {
                code: -32_000,
                message: "boom".to_string(),
                data: None,
            }),
            id,
        };
        let body = serde_json::to_string(&resp).expect("serialize");
        write_half.write_all(body.as_bytes()).await.unwrap();
        write_half.write_all(b"\n").await.unwrap();
        write_half.shutdown().await.ok();
    });

    let client = CapabilityClient::new(&sock).with_timeout(Duration::from_secs(5));
    let err = client.storage_put("k", b"v").await.expect_err("rpc error");
    assert!(err.to_string().contains("boom") || err.to_string().contains("-32000"));
    server.await.expect("join");
}

#[tokio::test]
async fn test_storage_get_null_value() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("null.sock");
    let listener = UnixListener::bind(&sock).expect("bind");
    let server = tokio::spawn(async move {
        serve_one_jsonrpc_response(listener, serde_json::json!({ "value": null })).await;
    });
    let client = CapabilityClient::new(&sock).with_timeout(Duration::from_secs(5));
    let got = client.storage_get("k").await.expect("ok");
    assert!(got.is_none());
    server.await.expect("join");
}

#[tokio::test]
async fn test_storage_get_string_value() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("get.sock");
    let b64 = base64_encode(b"hello");
    let listener = UnixListener::bind(&sock).expect("bind");
    let server = tokio::spawn(async move {
        serve_one_jsonrpc_response(listener, serde_json::json!({ "value": b64 })).await;
    });
    let client = CapabilityClient::new(&sock).with_timeout(Duration::from_secs(5));
    let got = client.storage_get("k").await.expect("get");
    assert_eq!(got.unwrap().as_ref(), b"hello");
    server.await.expect("join");
}

#[tokio::test]
async fn test_crypto_verify_bool_result_alias() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("verify.sock");
    let listener = UnixListener::bind(&sock).expect("bind");
    let server = tokio::spawn(async move {
        serve_one_jsonrpc_response(listener, serde_json::json!({ "result": true })).await;
    });
    let client = CapabilityClient::new(&sock).with_timeout(Duration::from_secs(5));
    let ok = client.crypto_verify(b"a", b"b", b"c").await.expect("v");
    assert!(ok);
    server.await.expect("join");
}

#[tokio::test]
async fn test_crypto_hash_result_field() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("hash.sock");
    let h = base64_encode(b"hashbytes");
    let listener = UnixListener::bind(&sock).expect("bind");
    let server = tokio::spawn(async move {
        serve_one_jsonrpc_response(listener, serde_json::json!({ "result": h })).await;
    });
    let client = CapabilityClient::new(&sock).with_timeout(Duration::from_secs(5));
    let out = client.crypto_hash(b"x", "sha256").await.expect("hash");
    assert_eq!(out.as_ref(), b"hashbytes");
    server.await.expect("join");
}

#[tokio::test]
async fn test_discover_capability_parses_primals() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("disc.sock");
    let listener = UnixListener::bind(&sock).expect("bind");
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let (mut read_half, mut write_half) = stream.into_split();
        let mut line = String::new();
        BufReader::new(&mut read_half)
            .read_line(&mut line)
            .await
            .expect("read");
        let req: JsonRpcRequest = serde_json::from_str(line.trim()).expect("parse");
        let id = req.id.clone().unwrap_or(serde_json::Value::Null);
        let resp = JsonRpcResponse {
            jsonrpc: biomeos_types::JsonRpcVersion,
            result: Some(serde_json::json!({
                "primals": [{"name": "beardog"}, {"name": "songbird"}]
            })),
            error: None,
            id,
        };
        let body = serde_json::to_string(&resp).expect("serialize");
        write_half.write_all(body.as_bytes()).await.unwrap();
        write_half.write_all(b"\n").await.unwrap();
        write_half.shutdown().await.ok();
    });
    let client = CapabilityClient::new(&sock).with_timeout(Duration::from_secs(5));
    let names = client.discover_capability("crypto").await.expect("disc");
    assert_eq!(names, vec!["beardog", "songbird"]);
    server.await.expect("join");
}

#[tokio::test]
async fn test_list_translations_success() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("list.sock");
    let listener = UnixListener::bind(&sock).expect("bind");
    let server = tokio::spawn(async move {
        serve_one_jsonrpc_response(listener, serde_json::json!({ "items": [] })).await;
    });
    let client = CapabilityClient::new(&sock).with_timeout(Duration::from_secs(5));
    let v = client.list_translations().await.expect("list");
    assert!(v.get("items").is_some());
    server.await.expect("join");
}

#[tokio::test]
async fn test_send_request_parse_error_short_response() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("bad.sock");
    let listener = UnixListener::bind(&sock).expect("bind");
    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        let (mut read_half, mut write_half) = stream.into_split();
        let mut line = String::new();
        BufReader::new(&mut read_half)
            .read_line(&mut line)
            .await
            .expect("read");
        write_half.write_all(b"not-json").await.unwrap();
        write_half.shutdown().await.ok();
    });
    let client = CapabilityClient::new(&sock).with_timeout(Duration::from_secs(5));
    let err = client.health_check("x").await.expect_err("parse fail");
    assert!(err.to_string().contains("parse") || err.to_string().contains("JSON"));
    server.await.expect("join");
}

#[tokio::test]
async fn test_storage_exists_result_alias() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("ex.sock");
    let listener = UnixListener::bind(&sock).expect("bind");
    let server = tokio::spawn(async move {
        serve_one_jsonrpc_response(listener, serde_json::json!({ "result": false })).await;
    });
    let client = CapabilityClient::new(&sock).with_timeout(Duration::from_secs(5));
    assert!(!client.storage_exists("k").await.expect("exists"));
    server.await.expect("join");
}
