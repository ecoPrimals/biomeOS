// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

use super::*;
use biomeos_types::{JsonRpcRequest, JsonRpcResponse};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

#[test]
fn test_capability_client_new() {
    let _client = CapabilityClient::new("/tmp/neural-api.sock");
    // Construction succeeds with path
}

#[test]
fn test_capability_client_with_timeout() {
    let _client = CapabilityClient::new("/tmp/sock").with_timeout(Duration::from_secs(5));
    // Builder pattern compiles and works
}

#[test]
fn test_capability_call_request_format() {
    // Verify the capability.call params structure
    let params = json!({
        "capability": "storage",
        "operation": "put",
        "args": { "key": "k", "value": "dmFsdWU=" }
    });
    assert_eq!(params["capability"], "storage");
    assert_eq!(params["operation"], "put");
    assert!(params["args"].is_object());
}

#[test]
fn test_base64_encode_decode_roundtrip() {
    let data = b"hello world";
    let encoded = base64_encode(data);
    let decoded = base64_decode(&encoded).unwrap();
    assert_eq!(decoded.as_ref(), data);
}

#[test]
fn test_base64_encode_empty() {
    let encoded = base64_encode(b"");
    assert_eq!(encoded, "");
}

#[test]
fn test_base64_encode_single_byte() {
    let encoded = base64_encode(b"a");
    assert_eq!(encoded.len(), 4);
    assert!(encoded.ends_with("=="));
    let decoded = base64_decode(&encoded).unwrap();
    assert_eq!(decoded.as_ref(), b"a");
}

#[test]
fn test_base64_encode_two_bytes() {
    let encoded = base64_encode(b"ab");
    assert_eq!(encoded.len(), 4);
    assert!(encoded.ends_with('='));
    let decoded = base64_decode(&encoded).unwrap();
    assert_eq!(decoded.as_ref(), b"ab");
}

#[test]
fn test_base64_encode_three_bytes() {
    let encoded = base64_encode(b"abc");
    assert_eq!(encoded.len(), 4);
    assert!(!encoded.ends_with('='));
    let decoded = base64_decode(&encoded).unwrap();
    assert_eq!(decoded.as_ref(), b"abc");
}

#[test]
fn test_base64_decode_with_padding() {
    let decoded = base64_decode("YQ==").unwrap();
    assert_eq!(decoded.as_ref(), b"a");
}

#[test]
fn test_base64_decode_without_padding() {
    let decoded = base64_decode("YQ").unwrap();
    assert_eq!(decoded.as_ref(), b"a");
}

#[test]
fn test_base64_decode_ignores_invalid_chars() {
    // Invalid chars are filtered out
    let decoded = base64_decode("YQ==\n\t ").unwrap();
    assert_eq!(decoded.as_ref(), b"a");
}

#[test]
fn test_base64_decode_empty() {
    let decoded = base64_decode("").unwrap();
    assert!(decoded.is_empty());
}

#[test]
fn test_resolve_neural_api_socket_no_env() {
    // Without NEURAL_API_SOCKET and no running biomeOS, discover fails
    let result = resolve_neural_api_socket();
    // May succeed if biomeOS happens to be running in test env
    let _ = result;
}

#[test]
fn test_capability_client_discover() {
    // discover() may fail if no socket exists
    let result = CapabilityClient::discover();
    let _ = result;
}

#[test]
fn test_capability_client_path_impl() {
    let client = CapabilityClient::new("/var/run/neural.sock");
    let client2 = CapabilityClient::new(PathBuf::from("/var/run/neural.sock"));
    drop(client);
    drop(client2);
}

#[test]
fn test_base64_encode_decode_large_data() {
    let data: Vec<u8> = (0u8..200).collect();
    let encoded = base64_encode(&data);
    let decoded = base64_decode(&encoded).unwrap();
    assert_eq!(decoded.as_ref(), data.as_slice());
}

#[test]
fn test_base64_decode_invalid_characters_filtered() {
    let decoded = base64_decode("Y\nQ\t=\r\n=").unwrap();
    assert_eq!(decoded.as_ref(), b"a");
}

#[test]
fn test_base64_decode_plus_slash() {
    let decoded = base64_decode("+/+").unwrap();
    assert!(!decoded.is_empty());
}

#[test]
fn test_http_request_params_structure() {
    let args = json!({
        "method": "GET",
        "url": "https://example.com",
        "headers": {"Authorization": "Bearer x"},
        "body": "request body"
    });
    assert_eq!(args["method"], "GET");
    assert_eq!(args["url"], "https://example.com");
    assert!(args["headers"].is_object());
    assert_eq!(args["body"], "request body");
}

#[test]
fn test_storage_put_params() {
    let args = json!({
        "key": "my-key",
        "value": base64_encode(b"value bytes")
    });
    assert_eq!(args["key"], "my-key");
    assert!(args["value"].as_str().is_some());
}

#[test]
fn test_storage_get_params() {
    let args = json!({ "key": "lookup-key" });
    assert_eq!(args["key"], "lookup-key");
}

#[test]
fn test_crypto_sign_params() {
    let args = json!({
        "data": base64_encode(b"data to sign")
    });
    assert!(args["data"].as_str().is_some());
}

#[test]
fn test_crypto_verify_params() {
    let args = json!({
        "data": base64_encode(b"data"),
        "signature": base64_encode(b"sig"),
        "public_key": base64_encode(b"pubkey")
    });
    assert!(args["data"].as_str().is_some());
    assert!(args["signature"].as_str().is_some());
    assert!(args["public_key"].as_str().is_some());
}

#[test]
fn test_crypto_hash_params() {
    let args = json!({
        "data": base64_encode(b"data"),
        "algorithm": "sha256"
    });
    assert_eq!(args["algorithm"], "sha256");
}

#[test]
fn test_compute_execute_params() {
    let args = json!({
        "task": "inference",
        "params": {"model": "test"}
    });
    assert_eq!(args["task"], "inference");
    assert!(args["params"].is_object());
}

#[test]
fn test_health_check_params() {
    let args = json!({ "primal": "beardog" });
    assert_eq!(args["primal"], "beardog");
}

#[tokio::test]
async fn test_capability_client_connection_refused() {
    let client = CapabilityClient::new("/nonexistent/socket/path/12345.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.crypto_sign(b"test").await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Failed")
            || err.contains("connect")
            || err.contains("timeout")
            || err.contains("Connection"),
        "expected connection error, got: {err}"
    );
}

#[tokio::test]
async fn test_storage_put_connection_refused() {
    let client = CapabilityClient::new("/nonexistent/socket/456.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.storage_put("key", b"value").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_storage_get_connection_refused() {
    let client = CapabilityClient::new("/nonexistent/socket/789.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.storage_get("key").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_storage_exists_connection_refused() {
    let client = CapabilityClient::new("/nonexistent/socket/exists.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.storage_exists("key").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_http_request_connection_refused() {
    let client = CapabilityClient::new("/nonexistent/socket/http.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client
        .http_request("GET", "https://example.com", None, None)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_compute_execute_connection_refused() {
    let client = CapabilityClient::new("/nonexistent/socket/compute.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.compute_execute("task", json!({})).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_discover_capability_connection_refused() {
    let client = CapabilityClient::new("/nonexistent/socket/discover.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.discover_capability("crypto").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_translations_connection_refused() {
    let client = CapabilityClient::new("/nonexistent/socket/list.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.list_translations().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_health_check_connection_refused() {
    let client = CapabilityClient::new("/nonexistent/socket/health.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.health_check("beardog").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_crypto_verify_connection_refused() {
    let client = CapabilityClient::new("/nonexistent/socket/verify.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.crypto_verify(b"data", b"sig", b"pubkey").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_crypto_hash_connection_refused() {
    let client = CapabilityClient::new("/nonexistent/socket/hash.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.crypto_hash(b"data", "sha256").await;
    assert!(result.is_err());
}

#[test]
fn test_resolve_neural_api_socket_invocation() {
    let result = resolve_neural_api_socket();
    match &result {
        Ok(p) => assert!(!p.as_os_str().is_empty()),
        Err(e) => {
            assert!(e.to_string().contains("not found") || e.to_string().contains("Neural"));
        }
    }
}

#[test]
fn test_resolve_neural_api_socket_from_env() {
    let tmp = tempfile::NamedTempFile::new().expect("temp file");
    let path = tmp.path().to_path_buf();
    let got =
        resolve_neural_api_socket_with(Some(tmp.path())).expect("explicit path should resolve");
    assert_eq!(got, path);
}

async fn serve_one_jsonrpc_response(listener: UnixListener, reply: serde_json::Value) {
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

#[test]
fn test_base64_encode_all_padding_cases() {
    assert_eq!(base64_encode(&[0xFF, 0xFF, 0xFF]).len(), 4);
}
