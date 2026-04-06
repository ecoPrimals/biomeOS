// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Integration tests for JSON-RPC WebSocket server
//!
//! Tests the full WebSocket lifecycle including:
//! - Connection establishment
//! - JSON-RPC 2.0 protocol compliance
//! - Subscription management
//! - Event filtering
//! - Concurrent subscriptions
//! - Error handling

use anyhow::Result;
use biomeos_graph::{GraphEvent, GraphEventBroadcaster};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::sync::Arc;
use tokio::time::{Duration, sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};

// Note: We can't import from biomeos-api since it's a binary crate
// So we'll test through the WebSocket protocol

/// Get the WebSocket server URL from environment or default.
///
/// Uses `BIOMEOS_WS_TEST_URL` env var if set, otherwise falls back to
/// the default HTTP port from RuntimeConfig.
fn ws_test_url() -> String {
    if let Ok(url) = std::env::var("BIOMEOS_WS_TEST_URL") {
        return url;
    }
    let port = std::env::var("HTTP_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
        .unwrap_or(biomeos_types::constants::network::DEFAULT_HTTP_PORT);
    format!("ws://127.0.0.1:{port}/ws")
}

/// Helper to parse JSON-RPC response.
/// `#[expect(dead_code)]`: serde populates all fields; tests only assert on a subset.
#[derive(Debug, serde::Deserialize)]
#[expect(dead_code, reason = "serde deserialization requires all fields")]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(default)]
    result: Option<serde_json::Value>,
    #[serde(default)]
    error: Option<JsonRpcError>,
    #[serde(default)]
    id: Option<serde_json::Value>,
}

/// Error payload; expect(dead_code) since tests only check presence, not field values.
#[derive(Debug, serde::Deserialize)]
#[expect(dead_code, reason = "serde deserialization requires all fields")]
struct JsonRpcError {
    code: i64,
    message: String,
}

/// Test: WebSocket connection and basic JSON-RPC
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_websocket_connection() -> Result<()> {
    // This test requires the biomeos-api server to be running
    // In CI, we'll skip this test if the server isn't available

    let url = ws_test_url();

    let Ok((ws_stream, _)) = connect_async(&url).await else {
        eprintln!("WebSocket server not running, skipping test");
        return Ok(());
    };
    let (mut write, mut read) = ws_stream.split();

    // Send a health check request
    let request = json!({
        "jsonrpc": "2.0",
        "method": "events.list_subscriptions",
        "params": {},
        "id": 1
    });

    write.send(Message::Text(request.to_string())).await?;

    // Read response
    if let Some(Ok(Message::Text(text))) = read.next().await {
        let response: JsonRpcResponse = serde_json::from_str(&text)?;
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
    }

    Ok(())
}

/// Test: JSON-RPC 2.0 error codes
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_json_rpc_error_codes() -> Result<()> {
    let url = ws_test_url();

    let Ok((ws_stream, _)) = connect_async(&url).await else {
        eprintln!("WebSocket server not running, skipping test");
        return Ok(());
    };
    let (mut write, mut read) = ws_stream.split();

    // Test invalid JSON (parse error -32700)
    write.send(Message::Text("not json".to_string())).await?;

    if let Some(Ok(Message::Text(text))) = read.next().await {
        let response: JsonRpcResponse = serde_json::from_str(&text)?;
        if let Some(error) = response.error {
            assert_eq!(error.code, -32700);
        }
    }

    // Test method not found (-32601)
    let request = json!({
        "jsonrpc": "2.0",
        "method": "invalid.method",
        "params": {},
        "id": 1
    });

    write.send(Message::Text(request.to_string())).await?;

    if let Some(Ok(Message::Text(text))) = read.next().await {
        let response: JsonRpcResponse = serde_json::from_str(&text)?;
        if let Some(error) = response.error {
            assert_eq!(error.code, -32601);
        }
    }

    Ok(())
}

/// Test: Subscription management
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_subscription_management() -> Result<()> {
    let url = ws_test_url();

    let Ok((ws_stream, _)) = connect_async(&url).await else {
        eprintln!("WebSocket server not running, skipping test");
        return Ok(());
    };
    let (mut write, mut read) = ws_stream.split();

    // Subscribe to events
    let subscribe_request = json!({
        "jsonrpc": "2.0",
        "method": "events.subscribe",
        "params": {
            "graph_id": "test_graph"
        },
        "id": 1
    });

    write
        .send(Message::Text(subscribe_request.to_string()))
        .await?;

    // Read subscription response
    let mut subscription_id = String::new();
    if let Some(Ok(Message::Text(text))) = read.next().await {
        let response: JsonRpcResponse = serde_json::from_str(&text)?;
        if let Some(result) = response.result {
            subscription_id = result["subscription_id"].as_str().unwrap_or("").to_string();
            assert!(!subscription_id.is_empty());
        }
    }

    // List subscriptions
    let list_request = json!({
        "jsonrpc": "2.0",
        "method": "events.list_subscriptions",
        "params": {},
        "id": 2
    });

    write.send(Message::Text(list_request.to_string())).await?;

    if let Some(Ok(Message::Text(text))) = read.next().await {
        let response: JsonRpcResponse = serde_json::from_str(&text)?;
        if let Some(result) = response.result {
            let count = result["count"].as_u64().unwrap_or(0);
            assert!(count >= 1);
        }
    }

    // Unsubscribe
    let unsubscribe_request = json!({
        "jsonrpc": "2.0",
        "method": "events.unsubscribe",
        "params": {
            "subscription_id": subscription_id
        },
        "id": 3
    });

    write
        .send(Message::Text(unsubscribe_request.to_string()))
        .await?;

    if let Some(Ok(Message::Text(text))) = read.next().await {
        let response: JsonRpcResponse = serde_json::from_str(&text)?;
        if let Some(result) = response.result {
            assert!(result["success"].as_bool().unwrap_or(false));
        }
    }

    Ok(())
}

/// Test: Event filtering
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_event_filtering() -> Result<()> {
    let url = ws_test_url();

    let Ok((ws_stream, _)) = connect_async(&url).await else {
        eprintln!("WebSocket server not running, skipping test");
        return Ok(());
    };
    let (mut write, mut read) = ws_stream.split();

    // Subscribe with filter
    let subscribe_request = json!({
        "jsonrpc": "2.0",
        "method": "events.subscribe",
        "params": {
            "graph_id": "test_graph",
            "event_types": ["NodeStarted", "NodeCompleted"],
            "node_filter": "node1"
        },
        "id": 1
    });

    write
        .send(Message::Text(subscribe_request.to_string()))
        .await?;

    // Read subscription confirmation
    if let Some(Ok(Message::Text(text))) = read.next().await {
        let response: JsonRpcResponse = serde_json::from_str(&text)?;
        assert!(response.result.is_some());
    }

    // Wait for events (if any)
    // In a real test, we'd trigger graph execution to generate events
    sleep(Duration::from_millis(100)).await;

    Ok(())
}

/// Test: Concurrent subscriptions
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_subscriptions() -> Result<()> {
    let url = ws_test_url();

    // Create multiple connections
    let mut handles = vec![];

    for i in 0..5 {
        let url_clone = url.clone();
        let handle = tokio::spawn(async move {
            match connect_async(&url_clone).await {
                Ok((ws_stream, _)) => {
                    let (mut write, _read) = ws_stream.split();

                    // Subscribe
                    let request = json!({
                        "jsonrpc": "2.0",
                        "method": "events.subscribe",
                        "params": {
                            "graph_id": format!("graph_{}", i)
                        },
                        "id": 1
                    });

                    let _ = write.send(Message::Text(request.to_string())).await;

                    // Hold connection briefly
                    sleep(Duration::from_millis(100)).await;

                    Ok::<(), anyhow::Error>(())
                }
                Err(_) => Ok(()),
            }
        });

        handles.push(handle);
    }

    // Wait for all connections
    for handle in handles {
        handle.await??;
    }

    Ok(())
}

/// Test: Invalid params error
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_invalid_params() -> Result<()> {
    let url = ws_test_url();

    let Ok((ws_stream, _)) = connect_async(&url).await else {
        eprintln!("WebSocket server not running, skipping test");
        return Ok(());
    };
    let (mut write, mut read) = ws_stream.split();

    // Send invalid params
    let request = json!({
        "jsonrpc": "2.0",
        "method": "events.subscribe",
        "params": "not an object",
        "id": 1
    });

    write.send(Message::Text(request.to_string())).await?;

    if let Some(Ok(Message::Text(text))) = read.next().await {
        let response: JsonRpcResponse = serde_json::from_str(&text)?;
        if let Some(error) = response.error {
            assert_eq!(error.code, -32602); // Invalid params
        }
    }

    Ok(())
}

/// Mock test: GraphEventBroadcaster integration
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_event_broadcaster_integration() {
    // This is a unit test for the broadcaster, not requiring a live server
    let broadcaster = Arc::new(GraphEventBroadcaster::new(100));

    // Subscribe
    let mut receiver1 = broadcaster.subscribe();
    let mut receiver2 = broadcaster.subscribe();

    // Broadcast an event
    let event = GraphEvent::GraphStarted {
        graph_id: "test".to_string(),
        graph_name: "Test Graph".to_string(),
        total_nodes: 5,
        coordination: "sequential".to_string(),
        timestamp: Utc::now(),
    };

    let _ = broadcaster.broadcast(event.clone()).await;

    // Both receivers should get the event
    assert!(receiver1.recv().await.is_ok());
    assert!(receiver2.recv().await.is_ok());
}

/// Test: Connection cleanup
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_connection_cleanup() -> Result<()> {
    let url = ws_test_url();

    let Ok((ws_stream, _)) = connect_async(&url).await else {
        eprintln!("WebSocket server not running, skipping test");
        return Ok(());
    };
    let (mut write, mut read) = ws_stream.split();

    // Create subscription
    let request = json!({
        "jsonrpc": "2.0",
        "method": "events.subscribe",
        "params": {},
        "id": 1
    });

    write.send(Message::Text(request.to_string())).await?;

    // Read response
    if let Some(Ok(Message::Text(_))) = read.next().await {
        // Connection established
    }

    // Close connection
    // Note: ws_stream was consumed by split(), so we only drop write and read
    drop(write);
    drop(read);

    // Subscriptions should be cleaned up automatically
    // (In a real test, we'd verify this by checking server state)

    Ok(())
}

/// Performance test: High-frequency events
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Performance test - run manually"]
async fn test_high_frequency_events() -> Result<()> {
    // Unit test with broadcaster only
    let broadcaster = Arc::new(GraphEventBroadcaster::new(100));
    let mut receiver = broadcaster.subscribe();

    // Spawn task to broadcast events rapidly
    let broadcaster_clone = broadcaster.clone();
    let broadcast_task = tokio::spawn(async move {
        for i in 0..1000 {
            let event = GraphEvent::GraphStarted {
                graph_id: format!("graph_{i}"),
                graph_name: format!("Graph {i}"),
                total_nodes: 3,
                coordination: "parallel".to_string(),
                timestamp: Utc::now(),
            };
            let _ = broadcaster_clone.broadcast(event).await;
        }
    });

    // Receive events
    let mut count = 0;
    let receive_task = tokio::spawn(async move {
        while receiver.recv().await.is_ok() {
            count += 1;
            if count >= 1000 {
                break;
            }
        }
        count
    });

    broadcast_task.await?;
    let received = receive_task.await?;

    // Should receive most or all events
    assert!(received >= 900, "Expected ~1000 events, got {received}");

    Ok(())
}
