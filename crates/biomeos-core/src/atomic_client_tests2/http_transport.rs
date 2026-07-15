// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::atomic_client::AtomicClient;
use serde_json::json;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn test_call_http_jsonrpc_success() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let port = listener.local_addr().expect("addr").port();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 16384];
            let _ = stream.read(&mut buf).await;
            let body = r#"{"jsonrpc":"2.0","result":{"http_ok":true},"id":1}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes()).await;
        }
    });
    tokio::task::yield_now().await;
    let client = AtomicClient::http("127.0.0.1", port);
    let result = client.call("ping", json!({})).await.expect("http call");
    assert_eq!(result["http_ok"], true);
}

#[tokio::test]
async fn test_call_http_malformed_no_separator_fails() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let port = listener.local_addr().expect("addr").port();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            let _ = stream
                .write_all(b"HTTP/1.1 200 OK\r\noops no body sep")
                .await;
        }
    });
    tokio::task::yield_now().await;
    let client = AtomicClient::http("127.0.0.1", port).with_timeout(Duration::from_secs(2));
    let err = client
        .call("m", json!({}))
        .await
        .expect_err("malformed http");
    let s = err.to_string();
    assert!(
        s.contains("Malformed") || s.contains("separator") || s.contains("body"),
        "{s}"
    );
}

#[tokio::test]
async fn test_call_http_jsonrpc_body_after_lf_only_separator() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let port = listener.local_addr().expect("addr").port();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 16384];
            let _ = stream.read(&mut buf).await;
            let body = r#"{"jsonrpc":"2.0","result":{"lf_sep":true},"id":1}"#;
            let response = format!("HTTP/1.1 200 OK\n\n{}", body);
            let _ = stream.write_all(response.as_bytes()).await;
        }
    });
    tokio::task::yield_now().await;
    let client = AtomicClient::http("127.0.0.1", port);
    let result = client.call("ping", json!({})).await.expect("http call");
    assert_eq!(result["lf_sep"], true);
}

#[tokio::test]
async fn test_call_http_jsonrpc_invalid_body_json_fails() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let port = listener.local_addr().expect("addr").port();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            let body = "not-json";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes()).await;
        }
    });
    tokio::task::yield_now().await;
    let client = AtomicClient::http("127.0.0.1", port).with_timeout(Duration::from_secs(2));
    let err = client
        .call("m", json!({}))
        .await
        .expect_err("bad json body");
    let s = err.to_string();
    assert!(
        s.contains("serialization") || s.contains("parse") || s.contains("JSON"),
        "{s}"
    );
}
