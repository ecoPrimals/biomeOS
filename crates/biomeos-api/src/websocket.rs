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
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;

// Re-export JSON-RPC types and graph types
pub use biomeos_graph::{GraphEvent, GraphEventBroadcaster};
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
    id: Arc<str>,
    filter: Arc<SubscriptionFilter>,
    /// Channel sender (held to keep subscription alive; events forwarded on graph state changes).
    _sender: tokio::sync::mpsc::UnboundedSender<GraphEvent>,
}

/// JSON-RPC 2.0 WebSocket server for graph events
pub struct GraphEventWebSocketServer {
    /// Active subscriptions (subscription_id -> Subscription)
    subscriptions: Arc<RwLock<HashMap<Arc<str>, Subscription>>>,

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
        subscriptions: Arc<RwLock<HashMap<Arc<str>, Subscription>>>,
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
        subscriptions: &Arc<RwLock<HashMap<Arc<str>, Subscription>>>,
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
        subscriptions: &Arc<RwLock<HashMap<Arc<str>, Subscription>>>,
        event_broadcaster: &Arc<GraphEventBroadcaster>,
        response_tx: tokio::sync::mpsc::UnboundedSender<String>,
    ) -> Result<serde_json::Value, JsonRpcError> {
        // Parse filter parameters
        let filter = Arc::new(
            serde_json::from_value::<SubscriptionFilter>(params)
                .map_err(|e| JsonRpcError::invalid_params(Some(e.to_string())))?,
        );

        // Generate subscription ID
        let sub_id: Arc<str> = format!("sub_{}", uuid::Uuid::new_v4()).into();

        // Create channel for this subscription
        let (event_tx, _event_rx) = tokio::sync::mpsc::unbounded_channel();

        // Subscribe to event broadcaster
        let mut event_receiver = event_broadcaster.subscribe();
        let filter_for_task = Arc::clone(&filter);
        let sub_id_for_task = Arc::clone(&sub_id);

        // Spawn task to forward filtered events
        tokio::spawn(async move {
            while let Ok(event) = event_receiver.recv().await {
                // Apply filter
                if filter_for_task.matches(&event) {
                    // Send event as JSON-RPC notification
                    let notification = JsonRpcResponse::success(
                        serde_json::Value::Null, // Notifications have no ID
                        serde_json::json!({
                            "subscription_id": sub_id_for_task.as_ref(),
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
            id: Arc::clone(&sub_id),
            filter: Arc::clone(&filter),
            _sender: event_tx,
        };

        subscriptions
            .write()
            .await
            .insert(Arc::clone(&sub_id), subscription);

        Ok(serde_json::json!({
            "subscription_id": sub_id.as_ref(),
            "success": true,
        }))
    }

    /// Handle events.unsubscribe method
    async fn handle_unsubscribe(
        params: serde_json::Value,
        subscriptions: &Arc<RwLock<HashMap<Arc<str>, Subscription>>>,
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
            .remove(params.subscription_id.as_str())
            .is_some();

        Ok(serde_json::json!({
            "success": existed,
        }))
    }

    /// Handle events.list_subscriptions method
    async fn handle_list_subscriptions(
        subscriptions: &Arc<RwLock<HashMap<Arc<str>, Subscription>>>,
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

/// Dispatch JSON-RPC methods for the axum WebSocket event stream (`/api/v1/events/ws`).
pub(crate) async fn dispatch_ws_method(
    method: &str,
    params: Option<serde_json::Value>,
    id: Option<serde_json::Value>,
    subscriptions: &Arc<RwLock<HashMap<String, SubscriptionFilter>>>,
    next_sub_id: &AtomicU64,
) -> JsonRpcResponse {
    let id = id.unwrap_or(serde_json::Value::Null);
    match method {
        "events.subscribe" => {
            let params = params.unwrap_or_else(|| serde_json::json!({}));
            let filter: SubscriptionFilter = serde_json::from_value(params).unwrap_or_else(|e| {
                tracing::warn!("JSON parse fallback: {}", e);
                SubscriptionFilter::default()
            });
            let sub_id = format!("sub_{}", next_sub_id.fetch_add(1, Ordering::SeqCst));
            let response_body = serde_json::json!({
                "subscription_id": &sub_id,
                "success": true,
            });
            subscriptions.write().await.insert(sub_id, filter);
            JsonRpcResponse::success(id, response_body)
        }
        "events.unsubscribe" => {
            let sub_id = params
                .as_ref()
                .and_then(|p| p.get("subscription_id"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            let existed = subscriptions.write().await.remove(sub_id).is_some();
            JsonRpcResponse::success(
                id,
                serde_json::json!({
                    "success": existed,
                    "subscription_id": sub_id,
                }),
            )
        }
        "events.list_subscriptions" => {
            let subs = subscriptions.read().await;
            let sub_list: Vec<_> = subs.keys().collect();
            JsonRpcResponse::success(
                id,
                serde_json::json!({
                    "subscriptions": sub_list,
                    "count": sub_list.len(),
                }),
            )
        }
        _ => JsonRpcResponse::error(id, JsonRpcError::method_not_found()),
    }
}

#[cfg(test)]
#[path = "websocket_tests.rs"]
mod websocket_tests;
