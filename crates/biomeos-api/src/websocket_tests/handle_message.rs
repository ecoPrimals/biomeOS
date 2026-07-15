// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use chrono::Utc;
use std::sync::Arc;

use super::common::test_empty_subscriptions;
use super::super::*;

#[tokio::test]
async fn test_handle_message_invalid_json() {
    let subscriptions = test_empty_subscriptions();
    let broadcaster = Arc::new(GraphEventBroadcaster::new(16));
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

    let resp = GraphEventWebSocketServer::handle_message(
        "not valid json",
        &subscriptions,
        &broadcaster,
        tx,
    )
    .await;

    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32700);
}

#[tokio::test]
async fn test_handle_message_invalid_version() {
    let subscriptions = test_empty_subscriptions();
    let broadcaster = Arc::new(GraphEventBroadcaster::new(16));
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

    let req = r#"{"jsonrpc":"1.0","id":1,"method":"events.subscribe","params":{}}"#;
    let resp =
        GraphEventWebSocketServer::handle_message(req, &subscriptions, &broadcaster, tx).await;

    assert!(resp.error.is_some());
    // Wrong jsonrpc fails during JsonRpcVersion deserialization → parse_error path.
    assert_eq!(resp.error.as_ref().unwrap().code, -32700);
}

#[tokio::test]
async fn test_handle_message_method_not_found() {
    let subscriptions = test_empty_subscriptions();
    let broadcaster = Arc::new(GraphEventBroadcaster::new(16));
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

    let req = r#"{"jsonrpc":"2.0","id":1,"method":"unknown.method","params":{}}"#;
    let resp =
        GraphEventWebSocketServer::handle_message(req, &subscriptions, &broadcaster, tx).await;

    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32601);
}

#[tokio::test]
async fn test_handle_message_subscribe_and_list() {
    let subscriptions = test_empty_subscriptions();
    let broadcaster = Arc::new(GraphEventBroadcaster::new(16));
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

    let req = r#"{"jsonrpc":"2.0","id":1,"method":"events.subscribe","params":{"graph_id":"g1"}}"#;
    let resp =
        GraphEventWebSocketServer::handle_message(req, &subscriptions, &broadcaster, tx.clone())
            .await;

    assert!(resp.error.is_none());
    let result = resp.result.expect("result");
    assert!(
        result
            .get("subscription_id")
            .and_then(serde_json::Value::as_str)
            .is_some()
    );
    assert!(
        result
            .get("success")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
    );

    let list_req = r#"{"jsonrpc":"2.0","id":2,"method":"events.list_subscriptions","params":{}}"#;
    let list_resp =
        GraphEventWebSocketServer::handle_message(list_req, &subscriptions, &broadcaster, tx).await;

    assert!(list_resp.error.is_none());
    let list_result = list_resp.result.expect("result");
    let count = list_result
        .get("count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_handle_message_unsubscribe_invalid_params() {
    let subscriptions = test_empty_subscriptions();
    let broadcaster = Arc::new(GraphEventBroadcaster::new(16));
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

    let req = r#"{"jsonrpc":"2.0","id":1,"method":"events.unsubscribe","params":{}}"#;
    let resp =
        GraphEventWebSocketServer::handle_message(req, &subscriptions, &broadcaster, tx).await;

    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
}

#[tokio::test]
async fn test_handle_message_subscribe_invalid_params_type() {
    let subscriptions = test_empty_subscriptions();
    let broadcaster = Arc::new(GraphEventBroadcaster::new(16));
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

    let req = r#"{"jsonrpc":"2.0","id":1,"method":"events.subscribe","params":{"graph_id":[]}}"#;
    let resp =
        GraphEventWebSocketServer::handle_message(req, &subscriptions, &broadcaster, tx).await;

    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
}

#[tokio::test]
async fn test_handle_message_unsubscribe_success() {
    let subscriptions = test_empty_subscriptions();
    let broadcaster = Arc::new(GraphEventBroadcaster::new(16));
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

    let sub_req =
        r#"{"jsonrpc":"2.0","id":1,"method":"events.subscribe","params":{"graph_id":"g1"}}"#;
    let sub_resp = GraphEventWebSocketServer::handle_message(
        sub_req,
        &subscriptions,
        &broadcaster,
        tx.clone(),
    )
    .await;
    let sub_id = sub_resp
        .result
        .as_ref()
        .and_then(|r| r.get("subscription_id"))
        .and_then(|v| v.as_str())
        .expect("subscription id");

    let unsub = format!(
        r#"{{"jsonrpc":"2.0","id":2,"method":"events.unsubscribe","params":{{"subscription_id":"{sub_id}"}}}}"#
    );
    let unsub_resp =
        GraphEventWebSocketServer::handle_message(&unsub, &subscriptions, &broadcaster, tx).await;
    assert!(unsub_resp.error.is_none());
    assert_eq!(
        unsub_resp
            .result
            .as_ref()
            .and_then(|r| r.get("success"))
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
}

#[tokio::test]
async fn test_handle_message_unsubscribe_unknown_id_returns_false() {
    let subscriptions = test_empty_subscriptions();
    let broadcaster = Arc::new(GraphEventBroadcaster::new(16));
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

    let req = r#"{"jsonrpc":"2.0","id":1,"method":"events.unsubscribe","params":{"subscription_id":"sub_does_not_exist"}}"#;
    let resp =
        GraphEventWebSocketServer::handle_message(req, &subscriptions, &broadcaster, tx).await;

    assert!(resp.error.is_none());
    assert_eq!(
        resp.result
            .as_ref()
            .and_then(|r| r.get("success"))
            .and_then(serde_json::Value::as_bool),
        Some(false)
    );
}

#[tokio::test]
async fn test_handle_message_list_subscriptions_empty() {
    let subscriptions = test_empty_subscriptions();
    let broadcaster = Arc::new(GraphEventBroadcaster::new(16));
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

    let req = r#"{"jsonrpc":"2.0","id":null,"method":"events.list_subscriptions","params":{}}"#;
    let resp =
        GraphEventWebSocketServer::handle_message(req, &subscriptions, &broadcaster, tx).await;

    assert!(resp.error.is_none());
    assert_eq!(resp.id, serde_json::Value::Null);
    let count = resp
        .result
        .as_ref()
        .and_then(|r| r.get("count"))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0);
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_handle_message_subscribe_params_null_invalid() {
    let subscriptions = test_empty_subscriptions();
    let broadcaster = Arc::new(GraphEventBroadcaster::new(16));
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

    let req = r#"{"jsonrpc":"2.0","id":1,"method":"events.subscribe","params":null}"#;
    let resp =
        GraphEventWebSocketServer::handle_message(req, &subscriptions, &broadcaster, tx).await;

    assert!(resp.error.is_some());
    assert_eq!(resp.error.as_ref().unwrap().code, -32602);
}

#[tokio::test]
async fn test_handle_message_subscribe_empty_params_object() {
    let subscriptions = test_empty_subscriptions();
    let broadcaster = Arc::new(GraphEventBroadcaster::new(16));
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();

    let req = r#"{"jsonrpc":"2.0","id":1,"method":"events.subscribe","params":{}}"#;
    let resp =
        GraphEventWebSocketServer::handle_message(req, &subscriptions, &broadcaster, tx).await;

    assert!(resp.error.is_none());
    assert!(
        resp.result
            .as_ref()
            .and_then(|r| r.get("subscription_id"))
            .and_then(|v| v.as_str())
            .is_some()
    );
}

#[tokio::test]
async fn test_handle_message_broadcast_two_subscribers_same_graph_both_notify() {
    let subscriptions = test_empty_subscriptions();
    let broadcaster = Arc::new(GraphEventBroadcaster::new(16));
    let (tx1, mut rx1) = tokio::sync::mpsc::unbounded_channel();
    let (tx2, mut rx2) = tokio::sync::mpsc::unbounded_channel();

    let sub1 = r#"{"jsonrpc":"2.0","id":1,"method":"events.subscribe","params":{"graph_id":"g1"}}"#;
    GraphEventWebSocketServer::handle_message(sub1, &subscriptions, &broadcaster, tx1.clone())
        .await;

    let sub2 = r#"{"jsonrpc":"2.0","id":2,"method":"events.subscribe","params":{"graph_id":"g1"}}"#;
    GraphEventWebSocketServer::handle_message(sub2, &subscriptions, &broadcaster, tx2.clone())
        .await;

    let event = GraphEvent::GraphStarted {
        graph_id: "g1".to_string(),
        graph_name: "G1".to_string(),
        total_nodes: 1,
        coordination: "sequential".to_string(),
        timestamp: Utc::now(),
    };
    broadcaster.broadcast(event).await.expect("broadcast");

    tokio::task::yield_now().await;

    let n1 = rx1.recv().await.expect("subscriber 1 notification");
    let n2 = rx2.recv().await.expect("subscriber 2 notification");
    assert!(n1.contains("subscription_id"));
    assert!(n2.contains("subscription_id"));
    assert!(n1.contains("g1"));
    assert!(n2.contains("g1"));
}

#[tokio::test]
async fn test_handle_message_broadcast_filtered_out_for_mismatched_graph() {
    let subscriptions = test_empty_subscriptions();
    let broadcaster = Arc::new(GraphEventBroadcaster::new(16));
    let (tx1, mut rx1) = tokio::sync::mpsc::unbounded_channel();
    let (tx2, mut rx2) = tokio::sync::mpsc::unbounded_channel();

    let sub1 = r#"{"jsonrpc":"2.0","id":1,"method":"events.subscribe","params":{"graph_id":"g1"}}"#;
    GraphEventWebSocketServer::handle_message(sub1, &subscriptions, &broadcaster, tx1.clone())
        .await;

    let sub2 = r#"{"jsonrpc":"2.0","id":2,"method":"events.subscribe","params":{"graph_id":"g2"}}"#;
    GraphEventWebSocketServer::handle_message(sub2, &subscriptions, &broadcaster, tx2.clone())
        .await;

    let event = GraphEvent::GraphStarted {
        graph_id: "g1".to_string(),
        graph_name: "G1".to_string(),
        total_nodes: 1,
        coordination: "sequential".to_string(),
        timestamp: Utc::now(),
    };
    broadcaster.broadcast(event).await.expect("broadcast");

    tokio::task::yield_now().await;

    let n1 = rx1.recv().await.expect("g1 filter should match");
    assert!(n1.contains("g1"));

    let n2 = tokio::time::timeout(std::time::Duration::from_millis(100), rx2.recv()).await;
    assert!(
        n2.is_err(),
        "g2 subscription should not receive notification for g1 event"
    );
}
