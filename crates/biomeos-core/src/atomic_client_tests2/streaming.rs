// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::atomic_client::AtomicClient;
use biomeos_test_utils::ready_signal;
use serde_json::json;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn test_call_stream_http_yields_transport_error() {
    use biomeos_types::StreamItem;
    let client = AtomicClient::http("127.0.0.1", 59997).with_timeout(Duration::from_millis(200));
    let mut rx = client.call_stream("stream", json!({})).expect("receiver");
    let first = rx.recv().await.expect("event");
    assert!(
        matches!(first, StreamItem::Error { .. }),
        "expected StreamItem::Error, got {first:?}"
    );
}

#[tokio::test]
async fn test_call_stream_unix_jsonrpc_single_line_wrapped() {
    use biomeos_types::StreamItem;
    let temp = tempfile::tempdir().expect("temp");
    let socket_path = temp.path().join("stream_wrap.sock");
    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "result": {"wrapped": 7},
                "id": 1
            });
            let _ = stream
                .write_all(format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes())
                .await;
        }
    });
    ready_rx.wait().await.expect("ready");
    let client = AtomicClient::unix(&socket_path).with_timeout(Duration::from_secs(2));
    let mut rx = client.call_stream("m", json!({})).expect("stream");
    let mut saw_data = false;
    while let Some(item) = rx.recv().await {
        if matches!(item, StreamItem::Data(_)) {
            saw_data = true;
        }
        if matches!(item, StreamItem::End) {
            break;
        }
    }
    assert!(saw_data);
}

#[tokio::test]
async fn test_call_stream_unix_raw_non_json_line_becomes_string_data() {
    use biomeos_types::StreamItem;
    let temp = tempfile::tempdir().expect("temp");
    let socket_path = temp.path().join("stream_raw.sock");
    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            let _ = stream.write_all(b"plain-text-not-json\n").await;
            drop(stream);
        }
    });
    ready_rx.wait().await.expect("ready");
    let client = AtomicClient::unix(&socket_path).with_timeout(Duration::from_secs(2));
    let mut rx = client.call_stream("m", json!({})).expect("stream");
    let mut saw_plain = false;
    while let Some(item) = rx.recv().await {
        if let StreamItem::Data(v) = &item {
            if v.as_str() == Some("plain-text-not-json") {
                saw_plain = true;
            }
        }
        if matches!(item, StreamItem::End) {
            break;
        }
    }
    assert!(saw_plain);
}
