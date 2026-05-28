// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use crate::neural_api_server::NeuralApiServer;
use crate::neural_api_server::btsp_negotiate;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};

fn create_test_server() -> NeuralApiServer {
    let temp = tempfile::tempdir().expect("temp dir");
    NeuralApiServer::new(temp.path(), "test_family", temp.path().join("neural.sock"))
}

#[tokio::test]
async fn test_encrypted_frame_roundtrip_via_handle_encrypted_stream() {
    let (server_stream, mut client_stream) =
        tokio::net::UnixStream::pair().expect("UnixStream::pair");
    let server = create_test_server();

    let keys = btsp_negotiate::derive_session_keys(&[0xAAu8; 32], &[1u8; 12], &[2u8; 12]);
    let client_keys = keys.clone();

    let client_task = tokio::spawn(async move {
        let request = r#"{"jsonrpc":"2.0","method":"health.liveness","id":1}"#;
        let frame =
            btsp_negotiate::encrypt_frame(&client_keys.client_to_server, request.as_bytes())
                .expect("encrypt");
        client_stream.write_all(&frame).await.expect("write frame");
        client_stream.flush().await.expect("flush");

        let mut len_buf = [0u8; 4];
        client_stream
            .read_exact(&mut len_buf)
            .await
            .expect("read len");
        let resp_len = u32::from_be_bytes(len_buf) as usize;
        let mut payload = vec![0u8; resp_len];
        client_stream
            .read_exact(&mut payload)
            .await
            .expect("read payload");
        let plaintext = btsp_negotiate::decrypt_frame(&client_keys.server_to_client, &payload)
            .expect("decrypt");
        String::from_utf8(plaintext).expect("utf8")
    });

    let reader = tokio::io::BufReader::new(server_stream);
    server
        .handle_encrypted_stream(reader, keys)
        .await
        .expect("handle_encrypted_stream");

    let response = client_task.await.expect("client task");
    assert!(response.contains("jsonrpc"), "response: {response}");
}

#[tokio::test]
async fn test_encrypted_stream_wrong_key_drops_connection() {
    let (server_stream, mut client_stream) =
        tokio::net::UnixStream::pair().expect("UnixStream::pair");
    let server = create_test_server();

    let server_keys =
        btsp_negotiate::derive_session_keys(&[0xAAu8; 32], &[1u8; 12], &[2u8; 12]);
    let wrong_keys = btsp_negotiate::derive_session_keys(&[0xBBu8; 32], &[3u8; 12], &[4u8; 12]);

    let client_task = tokio::spawn(async move {
        let request = r#"{"jsonrpc":"2.0","method":"health.liveness","id":1}"#;
        let frame =
            btsp_negotiate::encrypt_frame(&wrong_keys.client_to_server, request.as_bytes())
                .expect("encrypt");
        client_stream.write_all(&frame).await.expect("write frame");
        client_stream.flush().await.expect("flush");

        let mut buf = [0u8; 4];
        let result = client_stream.read_exact(&mut buf).await;
        result.is_err() || buf == [0u8; 4]
    });

    let reader = tokio::io::BufReader::new(server_stream);
    server
        .handle_encrypted_stream(reader, server_keys)
        .await
        .expect("should gracefully close");

    let dropped = client_task.await.expect("client task");
    assert!(dropped, "connection should be dropped on bad key");
}

#[tokio::test]
async fn test_encrypted_stream_oversized_frame_drops_connection() {
    let (server_stream, mut client_stream) =
        tokio::net::UnixStream::pair().expect("UnixStream::pair");
    let server = create_test_server();

    let keys = btsp_negotiate::derive_session_keys(&[0xCCu8; 32], &[5u8; 12], &[6u8; 12]);

    let client_task = tokio::spawn(async move {
        let huge_len: u32 = 20_000_000;
        client_stream
            .write_all(&huge_len.to_be_bytes())
            .await
            .expect("write oversized len");
        client_stream.flush().await.expect("flush");
    });

    let reader = tokio::io::BufReader::new(server_stream);
    server
        .handle_encrypted_stream(reader, keys)
        .await
        .expect("should gracefully close on oversized frame");

    client_task.await.expect("client task");
}

#[tokio::test]
async fn test_negotiate_then_encrypted_stream_end_to_end() {
    let (server_stream, mut client_stream) =
        tokio::net::UnixStream::pair().expect("UnixStream::pair");
    let server = create_test_server();

    let handshake_key = [0xDDu8; 32];
    let session_id = "e2e-test-session";

    btsp_negotiate::register_session(&server.btsp_sessions, session_id, Some(handshake_key))
        .await;

    let client_nonce = [0x11u8; 32];
    let client_nonce_b64 = {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(client_nonce)
    };

    let client_task = tokio::spawn(async move {
        let negotiate_req = format!(
            r#"{{"jsonrpc":"2.0","method":"btsp.negotiate","params":{{"session_id":"{}","preferred_cipher":"chacha20-poly1305","client_nonce":"{}","bond_type":"Covalent"}},"id":1}}"#,
            session_id, client_nonce_b64
        );
        client_stream
            .write_all((negotiate_req + "\n").as_bytes())
            .await
            .expect("write negotiate");
        client_stream.flush().await.expect("flush");

        let mut reader = tokio::io::BufReader::new(&mut client_stream);
        let mut negotiate_resp = String::new();
        reader
            .read_line(&mut negotiate_resp)
            .await
            .expect("read negotiate response");

        let resp: serde_json::Value =
            serde_json::from_str(&negotiate_resp).expect("parse negotiate response");
        let result = resp.get("result").expect("has result");
        assert_eq!(result["cipher"], "chacha20-poly1305");

        let server_nonce_b64 = result["server_nonce"].as_str().expect("server_nonce");
        let server_nonce = {
            use base64::Engine;
            base64::engine::general_purpose::STANDARD
                .decode(server_nonce_b64)
                .expect("decode server nonce")
        };

        let keys =
            btsp_negotiate::derive_session_keys(&handshake_key, &client_nonce, &server_nonce);

        let request = r#"{"jsonrpc":"2.0","method":"health.liveness","id":2}"#;
        let frame = btsp_negotiate::encrypt_frame(&keys.client_to_server, request.as_bytes())
            .expect("encrypt");

        let client_stream = reader.into_inner();
        client_stream.write_all(&frame).await.expect("write frame");
        client_stream.flush().await.expect("flush");

        let mut len_buf = [0u8; 4];
        client_stream
            .read_exact(&mut len_buf)
            .await
            .expect("read len");
        let resp_len = u32::from_be_bytes(len_buf) as usize;
        let mut payload = vec![0u8; resp_len];
        client_stream
            .read_exact(&mut payload)
            .await
            .expect("read payload");
        let plaintext =
            btsp_negotiate::decrypt_frame(&keys.server_to_client, &payload).expect("decrypt");
        String::from_utf8(plaintext).expect("utf8")
    });

    let reader = tokio::io::BufReader::new(server_stream);
    server
        .handle_stream_with_negotiate(reader)
        .await
        .expect("handle_stream_with_negotiate");

    let response = client_task.await.expect("client task");
    assert!(
        response.contains("jsonrpc"),
        "encrypted response should be valid JSON-RPC: {response}"
    );
}

#[tokio::test]
async fn test_negotiate_null_cipher_stays_on_ndjson() {
    let (server_stream, mut client_stream) =
        tokio::net::UnixStream::pair().expect("UnixStream::pair");
    let server = create_test_server();

    btsp_negotiate::register_session(&server.btsp_sessions, "null-sess", None).await;

    let client_task = tokio::spawn(async move {
        let negotiate_req = r#"{"jsonrpc":"2.0","method":"btsp.negotiate","params":{"session_id":"null-sess","preferred_cipher":"chacha20-poly1305","client_nonce":"AAAA"},"id":1}"#;
        client_stream
            .write_all((negotiate_req.to_string() + "\n").as_bytes())
            .await
            .expect("write negotiate");
        client_stream.flush().await.expect("flush");

        let mut reader = tokio::io::BufReader::new(&mut client_stream);
        let mut negotiate_resp = String::new();
        reader
            .read_line(&mut negotiate_resp)
            .await
            .expect("read negotiate response");

        let resp: serde_json::Value =
            serde_json::from_str(&negotiate_resp).expect("parse negotiate response");
        assert_eq!(
            resp["result"]["cipher"], "null",
            "should fall back to null without handshake key"
        );

        negotiate_resp
    });

    let reader = tokio::io::BufReader::new(server_stream);
    server
        .handle_stream_with_negotiate(reader)
        .await
        .expect("handle_stream_with_negotiate");

    client_task.await.expect("client task");
}

#[tokio::test]
async fn test_non_negotiate_first_line_stays_on_ndjson() {
    let (server_stream, mut client_stream) =
        tokio::net::UnixStream::pair().expect("UnixStream::pair");
    let server = create_test_server();

    let client_task = tokio::spawn(async move {
        let request = r#"{"jsonrpc":"2.0","method":"health.liveness","id":1}"#;
        client_stream
            .write_all((request.to_string() + "\n").as_bytes())
            .await
            .expect("write");
        client_stream.flush().await.expect("flush");

        let mut reader = tokio::io::BufReader::new(&mut client_stream);
        let mut resp = String::new();
        reader.read_line(&mut resp).await.expect("read response");
        resp
    });

    let reader = tokio::io::BufReader::new(server_stream);
    server
        .handle_stream_with_negotiate(reader)
        .await
        .expect("handle_stream_with_negotiate");

    let response = client_task.await.expect("client task");
    assert!(
        response.contains("jsonrpc"),
        "NDJSON response expected: {response}"
    );
}

#[tokio::test]
async fn test_handle_connection_unknown_method_returns_error_response() {
    let (server_stream, mut client_stream) =
        tokio::net::UnixStream::pair().expect("UnixStream::pair");
    let server = create_test_server();

    let request = r#"{"jsonrpc":"2.0","method":"nonexistent","id":1}"#;
    client_stream
        .write_all((request.to_string() + "\n").as_bytes())
        .await
        .expect("write request");
    client_stream.flush().await.expect("flush");

    let mut buf = String::new();
    let (read_result, conn_result) = tokio::join!(
        async {
            let mut reader = tokio::io::BufReader::new(&mut client_stream);
            reader.read_line(&mut buf).await
        },
        server.handle_connection(server_stream)
    );

    let _ = read_result.expect("read response");
    conn_result.expect("handle_connection should succeed");
    assert!(buf.contains("jsonrpc"));
    assert!(buf.contains("error"));
    assert!(buf.contains("nonexistent"));
}

#[tokio::test]
async fn test_handle_connection_processes_request_and_returns_response() {
    let (server_stream, mut client_stream) =
        tokio::net::UnixStream::pair().expect("UnixStream::pair");
    let server = create_test_server();

    let request = r#"{"jsonrpc":"2.0","method":"nonexistent","id":42}"#;
    client_stream
        .write_all((request.to_string() + "\n").as_bytes())
        .await
        .expect("write request");
    client_stream.flush().await.expect("flush");

    let mut buf = String::new();
    let (read_result, conn_result) = tokio::join!(
        async {
            let mut reader = tokio::io::BufReader::new(&mut client_stream);
            reader.read_line(&mut buf).await
        },
        server.handle_connection(server_stream)
    );

    let _ = read_result.expect("read response");
    conn_result.expect("handle_connection");
    assert!(buf.contains("jsonrpc"));
    assert!(buf.contains("error") || buf.contains("Method not found"));
}

#[tokio::test]
async fn test_handle_connection_invalid_json_returns_parse_error() {
    let (server_stream, mut client_stream) =
        tokio::net::UnixStream::pair().expect("UnixStream::pair");
    let server = create_test_server();

    client_stream
        .write_all(b"{invalid\n")
        .await
        .expect("write invalid json");
    client_stream.flush().await.expect("flush");

    let mut buf = String::new();
    let (read_result, conn_result) = tokio::join!(
        async {
            let mut reader = tokio::io::BufReader::new(&mut client_stream);
            reader.read_line(&mut buf).await
        },
        server.handle_connection(server_stream)
    );

    let _ = read_result.expect("read");
    conn_result.expect("connection handler");
    assert!(
        buf.contains("Parse error") || buf.contains("-32700"),
        "invalid JSON should return parse error, got: {buf}"
    );
}

#[tokio::test]
async fn test_handle_connection_batch_request() {
    let (server_stream, mut client_stream) =
        tokio::net::UnixStream::pair().expect("UnixStream::pair");
    let server = create_test_server();

    let batch = r#"[{"jsonrpc":"2.0","method":"topology.get","id":1},{"jsonrpc":"2.0","method":"topology.primals","id":2}]"#;
    client_stream
        .write_all((batch.to_string() + "\n").as_bytes())
        .await
        .expect("write batch");
    client_stream.flush().await.expect("flush");

    let mut buf = String::new();
    let (read_result, conn_result) = tokio::join!(
        async {
            let mut reader = tokio::io::BufReader::new(&mut client_stream);
            reader.read_line(&mut buf).await
        },
        server.handle_connection(server_stream)
    );

    let _ = read_result.expect("read");
    conn_result.expect("handle batch");
    let parsed: serde_json::Value = serde_json::from_str(&buf).expect("response is valid json");
    assert!(parsed.is_array(), "batch response must be an array");
    assert_eq!(parsed.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_handle_connection_empty_batch_returns_invalid_request() {
    let (server_stream, mut client_stream) =
        tokio::net::UnixStream::pair().expect("UnixStream::pair");
    let server = create_test_server();

    client_stream
        .write_all(b"[]\n")
        .await
        .expect("write empty batch");
    client_stream.flush().await.expect("flush");

    let mut buf = String::new();
    let (read_result, conn_result) = tokio::join!(
        async {
            let mut reader = tokio::io::BufReader::new(&mut client_stream);
            reader.read_line(&mut buf).await
        },
        server.handle_connection(server_stream)
    );

    let _ = read_result.expect("read");
    conn_result.expect("handle empty batch");
    assert!(
        buf.contains("Invalid Request") || buf.contains("-32600"),
        "empty batch should return invalid request, got: {buf}"
    );
}
