// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! JSON-RPC 2.0 WebSocket server for real-time graph events
//!
//! This module provides a WebSocket server that streams graph execution events
//! to clients using JSON-RPC 2.0 as the primary protocol, aligned with the
//! primal ecosystem.
//!
//! Protocol Priority:
//! 1. JSON-RPC 2.0 over WebSocket (PRIMARY - universal access)
//! 2. tarpc over WebSocket (HIGH-PERFORMANCE - Rust-to-Rust, future)
//!
//! Deep Debt Principles:
//! - No hardcoded endpoints (capability-based discovery)
//! - Modern async Rust (tokio, futures)
//! - No unsafe code
//! - Zero mocks in production (mocks only in tests)

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;

// Re-export JSON-RPC types and graph types
pub use biomeos_graph::{GraphEvent, GraphEventBroadcaster};
#[cfg(test)]
use biomeos_types::JsonRpcVersion;
pub use biomeos_types::{JSONRPC_VERSION, JsonRpcError, JsonRpcRequest, JsonRpcResponse};

/// Subscription filter parameters
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SubscriptionFilter {
    /// Filter by specific graph ID
    #[serde(default)]
    pub graph_id: Option<String>,

    /// Filter by event types
    #[serde(default)]
    pub event_types: Option<Vec<String>>,

    /// Filter by node ID pattern (simple string matching)
    #[serde(default)]
    pub node_filter: Option<String>,
}

impl SubscriptionFilter {
    /// Check if an event matches this filter
    pub fn matches(&self, event: &GraphEvent) -> bool {
        // Check graph_id filter
        if let Some(ref filter_graph_id) = self.graph_id {
            if event.graph_id() != filter_graph_id {
                return false;
            }
        }

        // Check event_types filter
        if let Some(ref event_types) = self.event_types {
            let event_debug = format!("{event:?}");
            let event_type = event_debug.split('{').next().unwrap_or("").trim();
            if !event_types.iter().any(|t| t == event_type) {
                return false;
            }
        }

        // Check node_filter (simple string matching)
        if let Some(ref node_pattern) = self.node_filter {
            // Extract node_id from event if available
            let has_node = match event {
                GraphEvent::NodeStarted { node_id, .. }
                | GraphEvent::NodeCompleted { node_id, .. }
                | GraphEvent::NodeFailed { node_id, .. } => node_id.contains(node_pattern),
                _ => true, // Non-node events pass node filter
            };

            if !has_node {
                return false;
            }
        }

        true
    }
}

/// Active subscription
pub(crate) struct Subscription {
    /// Subscription ID (used in list_subscriptions and event notifications)
    id: String,
    filter: SubscriptionFilter,
    /// Channel sender (held to keep subscription alive; future: event forwarding)
    _sender: tokio::sync::mpsc::UnboundedSender<GraphEvent>,
}

/// JSON-RPC 2.0 WebSocket server for graph events
pub struct GraphEventWebSocketServer {
    /// Active subscriptions (subscription_id -> Subscription)
    subscriptions: Arc<RwLock<HashMap<String, Subscription>>>,

    /// Event broadcaster from graph executor
    event_broadcaster: Arc<GraphEventBroadcaster>,

    /// Server bind address
    bind_addr: SocketAddr,
}

impl GraphEventWebSocketServer {
    /// Create a new WebSocket server
    pub fn new(bind_addr: SocketAddr, event_broadcaster: Arc<GraphEventBroadcaster>) -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            event_broadcaster,
            bind_addr,
        }
    }

    /// Start the WebSocket server (runs indefinitely)
    pub async fn start(&self) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(self.bind_addr)
            .await
            .context("Failed to bind WebSocket server")?;

        tracing::info!("WebSocket server listening on {}", self.bind_addr);

        loop {
            let (stream, addr) = listener.accept().await?;
            tracing::debug!("New WebSocket connection from {}", addr);

            // Upgrade to WebSocket
            let ws_stream = tokio_tungstenite::accept_async(stream)
                .await
                .context("Failed to accept WebSocket connection")?;

            // Spawn handler for this connection
            let subscriptions = self.subscriptions.clone();
            let event_broadcaster = self.event_broadcaster.clone();

            tokio::spawn(async move {
                if let Err(e) =
                    Self::handle_connection(ws_stream, subscriptions, event_broadcaster).await
                {
                    tracing::error!("WebSocket connection error from {}: {}", addr, e);
                }
            });
        }
    }

    /// Handle a single WebSocket connection
    async fn handle_connection(
        ws_stream: WebSocketStream<tokio::net::TcpStream>,
        subscriptions: Arc<RwLock<HashMap<String, Subscription>>>,
        event_broadcaster: Arc<GraphEventBroadcaster>,
    ) -> Result<()> {
        let (mut write, mut read) = ws_stream.split();

        // Channel for sending responses back to client
        let (response_tx, mut response_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

        // Spawn task to send responses
        let send_task = tokio::spawn(async move {
            while let Some(msg) = response_rx.recv().await {
                if write.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        });

        // Handle incoming messages
        while let Some(msg) = read.next().await {
            let msg = msg?;

            if let Message::Text(text) = msg {
                let response = Self::handle_message(
                    &text,
                    &subscriptions,
                    &event_broadcaster,
                    response_tx.clone(),
                )
                .await;

                // Send response
                let _ = response_tx.send(serde_json::to_string(&response)?);
            } else if msg.is_close() {
                break;
            }
        }

        // Clean up: remove all subscriptions for this connection
        // (In production, we'd track which subscriptions belong to which connection)

        send_task.abort();
        Ok(())
    }

    /// Handle a JSON-RPC message (pub for testing)
    pub(crate) async fn handle_message(
        text: &str,
        subscriptions: &Arc<RwLock<HashMap<String, Subscription>>>,
        event_broadcaster: &Arc<GraphEventBroadcaster>,
        response_tx: tokio::sync::mpsc::UnboundedSender<String>,
    ) -> JsonRpcResponse {
        // Parse JSON-RPC request
        let request: JsonRpcRequest = match serde_json::from_str(text) {
            Ok(req) => req,
            Err(_) => {
                return JsonRpcResponse::error(
                    serde_json::Value::Null,
                    JsonRpcError::parse_error(),
                );
            }
        };

        // Validate JSON-RPC version
        if request.jsonrpc != JSONRPC_VERSION {
            return JsonRpcResponse::error(
                request.id.unwrap_or(serde_json::Value::Null),
                JsonRpcError::invalid_request(),
            );
        }

        let params = request.params.unwrap_or(serde_json::Value::Null);

        // Handle method
        let result = match request.method.as_ref() {
            "events.subscribe" => {
                Self::handle_subscribe(params, subscriptions, event_broadcaster, response_tx).await
            }
            "events.unsubscribe" => Self::handle_unsubscribe(params, subscriptions).await,
            "events.list_subscriptions" => Self::handle_list_subscriptions(subscriptions).await,
            _ => Err(JsonRpcError::method_not_found()),
        };

        let id = request.id.unwrap_or(serde_json::Value::Null);

        // Build response
        match result {
            Ok(value) => JsonRpcResponse::success(id, value),
            Err(error) => JsonRpcResponse::error(id, error),
        }
    }

    /// Handle events.subscribe method
    async fn handle_subscribe(
        params: serde_json::Value,
        subscriptions: &Arc<RwLock<HashMap<String, Subscription>>>,
        event_broadcaster: &Arc<GraphEventBroadcaster>,
        response_tx: tokio::sync::mpsc::UnboundedSender<String>,
    ) -> Result<serde_json::Value, JsonRpcError> {
        // Parse filter parameters
        let filter: SubscriptionFilter = serde_json::from_value(params)
            .map_err(|e| JsonRpcError::invalid_params(Some(e.to_string())))?;

        // Generate subscription ID
        let sub_id = format!("sub_{}", uuid::Uuid::new_v4());

        // Create channel for this subscription
        let (event_tx, _event_rx) = tokio::sync::mpsc::unbounded_channel();

        // Subscribe to event broadcaster
        let mut event_receiver = event_broadcaster.subscribe();
        let filter_clone = filter.clone();
        let sub_id_clone = sub_id.clone();

        // Spawn task to forward filtered events
        tokio::spawn(async move {
            while let Ok(event) = event_receiver.recv().await {
                // Apply filter
                if filter_clone.matches(&event) {
                    // Send event as JSON-RPC notification
                    let notification = JsonRpcResponse::success(
                        serde_json::Value::Null, // Notifications have no ID
                        serde_json::json!({
                            "subscription_id": sub_id_clone,
                            "event": event,
                        }),
                    );

                    if let Ok(json) = serde_json::to_string(&notification) {
                        let _ = response_tx.send(json);
                    }
                }
            }
        });

        // Store subscription
        let subscription = Subscription {
            id: sub_id.clone(),
            filter,
            _sender: event_tx,
        };

        subscriptions
            .write()
            .await
            .insert(sub_id.clone(), subscription);

        Ok(serde_json::json!({
            "subscription_id": sub_id,
            "success": true,
        }))
    }

    /// Handle events.unsubscribe method
    async fn handle_unsubscribe(
        params: serde_json::Value,
        subscriptions: &Arc<RwLock<HashMap<String, Subscription>>>,
    ) -> Result<serde_json::Value, JsonRpcError> {
        #[derive(Deserialize)]
        struct UnsubscribeParams {
            subscription_id: String,
        }

        let params: UnsubscribeParams = serde_json::from_value(params)
            .map_err(|e| JsonRpcError::invalid_params(Some(e.to_string())))?;

        let existed = subscriptions
            .write()
            .await
            .remove(&params.subscription_id)
            .is_some();

        Ok(serde_json::json!({
            "success": existed,
        }))
    }

    /// Handle events.list_subscriptions method
    async fn handle_list_subscriptions(
        subscriptions: &Arc<RwLock<HashMap<String, Subscription>>>,
    ) -> Result<serde_json::Value, JsonRpcError> {
        let subs = subscriptions.read().await;

        let sub_list: Vec<serde_json::Value> = subs
            .values()
            .map(|sub| {
                serde_json::json!({
                    "subscription_id": sub.id,
                    "filter": sub.filter,
                })
            })
            .collect();

        Ok(serde_json::json!({
            "subscriptions": sub_list,
            "count": sub_list.len(),
        }))
    }
}

#[cfg(test)]
fn test_empty_subscriptions() -> Arc<RwLock<HashMap<String, Subscription>>> {
    Arc::new(RwLock::new(HashMap::new()))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_subscription_filter_graph_id() {
        let filter = SubscriptionFilter {
            graph_id: Some("test_graph".to_string()),
            event_types: None,
            node_filter: None,
        };

        let event = GraphEvent::GraphStarted {
            graph_id: "test_graph".to_string(),
            graph_name: "Test Graph".to_string(),
            total_nodes: 1,
            coordination: "sequential".to_string(),
            timestamp: Utc::now(),
        };

        assert!(filter.matches(&event));

        let event2 = GraphEvent::GraphStarted {
            graph_id: "other_graph".to_string(),
            graph_name: "Other Graph".to_string(),
            total_nodes: 1,
            coordination: "sequential".to_string(),
            timestamp: Utc::now(),
        };

        assert!(!filter.matches(&event2));
    }

    #[test]
    fn test_subscription_filter_node() {
        let filter = SubscriptionFilter {
            graph_id: None,
            event_types: None,
            node_filter: Some("node1".to_string()),
        };

        let event = GraphEvent::NodeStarted {
            graph_id: "test".to_string(),
            node_id: "node1".to_string(),
            primal: "test_primal".to_string(),
            operation: "test_op".to_string(),
            timestamp: Utc::now(),
        };

        assert!(filter.matches(&event));

        let event2 = GraphEvent::NodeStarted {
            graph_id: "test".to_string(),
            node_id: "node2".to_string(),
            primal: "test_primal".to_string(),
            operation: "test_op".to_string(),
            timestamp: Utc::now(),
        };

        assert!(!filter.matches(&event2));
    }

    #[test]
    fn test_json_rpc_error_codes() {
        assert_eq!(JsonRpcError::parse_error().code, -32700);
        assert_eq!(JsonRpcError::invalid_request().code, -32600);
        assert_eq!(JsonRpcError::method_not_found().code, -32601);
        assert_eq!(JsonRpcError::invalid_params(None).code, -32602);
        assert_eq!(JsonRpcError::internal_error(None).code, -32603);
    }

    #[tokio::test]
    async fn test_subscription_filter_empty() {
        let filter = SubscriptionFilter {
            graph_id: None,
            event_types: None,
            node_filter: None,
        };

        // Empty filter matches everything
        let event = GraphEvent::GraphStarted {
            graph_id: "any".to_string(),
            graph_name: "Any Graph".to_string(),
            total_nodes: 1,
            coordination: "sequential".to_string(),
            timestamp: Utc::now(),
        };

        assert!(filter.matches(&event));
    }

    #[test]
    fn test_json_rpc_request_deserialization() {
        let json =
            r#"{"jsonrpc": "2.0", "id": 1, "method": "test.method", "params": {"key": "value"}}"#;
        let request: JsonRpcRequest = serde_json::from_str(json).expect("deserialize");

        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, Some(serde_json::json!(1)));
        assert_eq!(request.method.as_ref(), "test.method");
    }

    #[test]
    fn test_json_rpc_response_serialization() {
        let response = JsonRpcResponse {
            jsonrpc: JsonRpcVersion,
            result: Some(serde_json::json!({"success": true})),
            error: None,
            id: serde_json::json!(1),
        };

        let json = serde_json::to_string(&response).expect("serialize");
        assert!(json.contains("2.0"));
        assert!(json.contains("success"));
    }

    #[test]
    fn test_json_rpc_error_with_data() {
        let error = JsonRpcError::invalid_params(Some("missing required field".to_string()));
        assert_eq!(error.code, -32602);
        assert!(error.data.is_some());
    }

    #[test]
    fn test_subscription_filter_serialization() {
        let filter = SubscriptionFilter {
            graph_id: Some("test-graph".to_string()),
            event_types: Some(vec!["graph_started".to_string()]),
            node_filter: Some("node*".to_string()),
        };

        let json = serde_json::to_string(&filter).expect("serialize");
        assert!(json.contains("test-graph"));
        assert!(json.contains("graph_started"));
        assert!(json.contains("node*"));
    }

    #[test]
    fn test_subscription_filter_deserialization() {
        let json = r#"{"graph_id": "g1", "event_types": ["a", "b"], "node_filter": "n*"}"#;
        let filter: SubscriptionFilter = serde_json::from_str(json).expect("deserialize");

        assert_eq!(filter.graph_id, Some("g1".to_string()));
        assert_eq!(filter.event_types.as_ref().map(Vec::len), Some(2));
        assert_eq!(filter.node_filter, Some("n*".to_string()));
    }

    #[test]
    fn test_graph_event_serialization() {
        let event = GraphEvent::GraphCompleted {
            graph_id: "test".to_string(),
            success: true,
            duration_ms: 1234,
            nodes_executed: 5,
            nodes_failed: 0,
            timestamp: Utc::now(),
        };

        let json = serde_json::to_string(&event).expect("serialize");
        assert!(json.contains("test"));
        assert!(json.contains("1234"));
        assert!(json.contains("true"));
    }

    #[test]
    fn test_subscription_filter_event_types() {
        let filter = SubscriptionFilter {
            graph_id: None,
            event_types: Some(vec!["NodeStarted".to_string(), "NodeCompleted".to_string()]),
            node_filter: None,
        };

        let started_event = GraphEvent::NodeStarted {
            graph_id: "g1".to_string(),
            node_id: "n1".to_string(),
            primal: "p1".to_string(),
            operation: "op1".to_string(),
            timestamp: Utc::now(),
        };
        assert!(filter.matches(&started_event));

        let completed_event = GraphEvent::NodeCompleted {
            graph_id: "g1".to_string(),
            node_id: "n1".to_string(),
            duration_ms: 100,
            output: None,
            timestamp: Utc::now(),
        };
        assert!(filter.matches(&completed_event));

        let graph_started = GraphEvent::GraphStarted {
            graph_id: "g1".to_string(),
            graph_name: "G1".to_string(),
            total_nodes: 1,
            coordination: "sequential".to_string(),
            timestamp: Utc::now(),
        };
        assert!(!filter.matches(&graph_started));
    }

    #[test]
    fn test_subscription_filter_node_failed() {
        let filter = SubscriptionFilter {
            graph_id: None,
            event_types: None,
            node_filter: Some("target_node".to_string()),
        };

        let matching_event = GraphEvent::NodeFailed {
            graph_id: "g1".to_string(),
            node_id: "target_node".to_string(),
            error: "err".to_string(),
            retry_attempt: 0,
            will_retry: false,
            timestamp: Utc::now(),
        };
        assert!(filter.matches(&matching_event));

        let non_matching = GraphEvent::NodeFailed {
            graph_id: "g1".to_string(),
            node_id: "other_node".to_string(),
            error: "err".to_string(),
            retry_attempt: 0,
            will_retry: false,
            timestamp: Utc::now(),
        };
        assert!(!filter.matches(&non_matching));
    }

    #[test]
    fn test_subscription_filter_combined() {
        let filter = SubscriptionFilter {
            graph_id: Some("my_graph".to_string()),
            event_types: Some(vec!["NodeCompleted".to_string()]),
            node_filter: Some("node_a".to_string()),
        };

        let matching = GraphEvent::NodeCompleted {
            graph_id: "my_graph".to_string(),
            node_id: "node_a".to_string(),
            duration_ms: 50,
            output: None,
            timestamp: Utc::now(),
        };
        assert!(filter.matches(&matching));

        let wrong_graph = GraphEvent::NodeCompleted {
            graph_id: "other_graph".to_string(),
            node_id: "node_a".to_string(),
            duration_ms: 50,
            output: None,
            timestamp: Utc::now(),
        };
        assert!(!filter.matches(&wrong_graph));
    }

    #[test]
    fn test_json_rpc_error_serialization() {
        let error = JsonRpcError::internal_error(Some("detail".to_string()));
        let json = serde_json::to_string(&error).expect("serialize");
        assert!(json.contains("-32603"));
        assert!(json.contains("Internal error"));
        assert!(json.contains("detail"));
    }

    #[test]
    fn test_json_rpc_response_error_serialization() {
        let response = JsonRpcResponse {
            jsonrpc: JsonRpcVersion,
            result: None,
            error: Some(JsonRpcError::invalid_params(Some(
                "missing field".to_string(),
            ))),
            id: serde_json::json!("req-1"),
        };
        let json = serde_json::to_string(&response).expect("serialize");
        assert!(json.contains("error"));
        assert!(json.contains("-32602"));
    }

    #[test]
    fn test_graph_event_websocket_server_construction() {
        use std::net::SocketAddr;
        use std::str::FromStr;

        let addr = SocketAddr::from_str("127.0.0.1:0").expect("parse addr");
        let broadcaster = Arc::new(GraphEventBroadcaster::new(16));
        let server = GraphEventWebSocketServer::new(addr, broadcaster);
        // Server should be constructible without panicking
        drop(server);
    }

    #[test]
    fn test_subscription_filter_default() {
        let filter = SubscriptionFilter::default();
        assert!(filter.graph_id.is_none());
        assert!(filter.event_types.is_none());
        assert!(filter.node_filter.is_none());

        let event = GraphEvent::GraphStarted {
            graph_id: "any".to_string(),
            graph_name: "Any".to_string(),
            total_nodes: 1,
            coordination: "sequential".to_string(),
            timestamp: Utc::now(),
        };
        assert!(filter.matches(&event));
    }

    #[test]
    fn test_subscription_filter_non_node_events_pass_node_filter() {
        // GraphStarted has no node_id - node_filter should pass (returns true for non-node events)
        let filter = SubscriptionFilter {
            graph_id: None,
            event_types: None,
            node_filter: Some("some_node".to_string()),
        };
        let event = GraphEvent::GraphStarted {
            graph_id: "g1".to_string(),
            graph_name: "G1".to_string(),
            total_nodes: 1,
            coordination: "sequential".to_string(),
            timestamp: Utc::now(),
        };
        assert!(filter.matches(&event));
    }

    #[test]
    fn test_json_rpc_parse_error_response() {
        let err = JsonRpcError::parse_error();
        let resp = JsonRpcResponse::error(serde_json::Value::Null, err);
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("-32700"));
        assert!(json.contains("Parse error"));
    }

    #[test]
    fn test_json_rpc_invalid_request_response() {
        let err = JsonRpcError::invalid_request();
        let resp = JsonRpcResponse::error(serde_json::json!(1), err);
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("-32600"));
    }

    #[test]
    fn test_subscription_filter_event_type_empty_list() {
        let filter = SubscriptionFilter {
            graph_id: None,
            event_types: Some(vec![]),
            node_filter: None,
        };
        let event = GraphEvent::GraphStarted {
            graph_id: "g1".to_string(),
            graph_name: "G1".to_string(),
            total_nodes: 1,
            coordination: "sequential".to_string(),
            timestamp: Utc::now(),
        };
        assert!(!filter.matches(&event));
    }

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

        let req =
            r#"{"jsonrpc":"2.0","id":1,"method":"events.subscribe","params":{"graph_id":"g1"}}"#;
        let resp = GraphEventWebSocketServer::handle_message(
            req,
            &subscriptions,
            &broadcaster,
            tx.clone(),
        )
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

        let list_req =
            r#"{"jsonrpc":"2.0","id":2,"method":"events.list_subscriptions","params":{}}"#;
        let list_resp =
            GraphEventWebSocketServer::handle_message(list_req, &subscriptions, &broadcaster, tx)
                .await;

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

        let req =
            r#"{"jsonrpc":"2.0","id":1,"method":"events.subscribe","params":{"graph_id":[]}}"#;
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
            GraphEventWebSocketServer::handle_message(&unsub, &subscriptions, &broadcaster, tx)
                .await;
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

        let sub1 =
            r#"{"jsonrpc":"2.0","id":1,"method":"events.subscribe","params":{"graph_id":"g1"}}"#;
        GraphEventWebSocketServer::handle_message(sub1, &subscriptions, &broadcaster, tx1.clone())
            .await;

        let sub2 =
            r#"{"jsonrpc":"2.0","id":2,"method":"events.subscribe","params":{"graph_id":"g1"}}"#;
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

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

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

        let sub1 =
            r#"{"jsonrpc":"2.0","id":1,"method":"events.subscribe","params":{"graph_id":"g1"}}"#;
        GraphEventWebSocketServer::handle_message(sub1, &subscriptions, &broadcaster, tx1.clone())
            .await;

        let sub2 =
            r#"{"jsonrpc":"2.0","id":2,"method":"events.subscribe","params":{"graph_id":"g2"}}"#;
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

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let n1 = rx1.recv().await.expect("g1 filter should match");
        assert!(n1.contains("g1"));

        let n2 = tokio::time::timeout(std::time::Duration::from_millis(100), rx2.recv()).await;
        assert!(
            n2.is_err(),
            "g2 subscription should not receive notification for g1 event"
        );
    }
}
