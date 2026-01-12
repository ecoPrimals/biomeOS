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

use anyhow::{Result, Context};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;

// Re-export GraphEvent and GraphEventBroadcaster from biomeos-graph
// This maintains proper module boundaries
pub use biomeos_graph::{GraphEvent, GraphEventBroadcaster};

/// JSON-RPC 2.0 request structure
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 response structure
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 error structure
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    pub fn parse_error() -> Self {
        Self {
            code: -32700,
            message: "Parse error".to_string(),
            data: None,
        }
    }
    
    pub fn invalid_request() -> Self {
        Self {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: None,
        }
    }
    
    pub fn method_not_found() -> Self {
        Self {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        }
    }
    
    pub fn invalid_params(details: Option<String>) -> Self {
        Self {
            code: -32602,
            message: "Invalid params".to_string(),
            data: details.map(|d| serde_json::json!({"details": d})),
        }
    }
    
    pub fn internal_error(details: Option<String>) -> Self {
        Self {
            code: -32603,
            message: "Internal error".to_string(),
            data: details.map(|d| serde_json::json!({"details": d})),
        }
    }
}

/// Subscription filter parameters
#[derive(Debug, Clone, Deserialize, Serialize)]
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
            let event_debug = format!("{:?}", event);
            let event_type = event_debug.split('{').next().unwrap_or("").trim();
            if !event_types.iter().any(|t| t == event_type) {
                return false;
            }
        }
        
        // Check node_filter (simple string matching)
        if let Some(ref node_pattern) = self.node_filter {
            // Extract node_id from event if available
            let has_node = match event {
                GraphEvent::NodeStarted { node_id, .. } |
                GraphEvent::NodeCompleted { node_id, .. } |
                GraphEvent::NodeFailed { node_id, .. } => {
                    node_id.contains(node_pattern)
                }
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
struct Subscription {
    id: String,
    filter: SubscriptionFilter,
    sender: tokio::sync::mpsc::UnboundedSender<GraphEvent>,
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
                if let Err(e) = Self::handle_connection(ws_stream, subscriptions, event_broadcaster).await {
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
                ).await;
                
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
    
    /// Handle a JSON-RPC message
    async fn handle_message(
        text: &str,
        subscriptions: &Arc<RwLock<HashMap<String, Subscription>>>,
        event_broadcaster: &Arc<GraphEventBroadcaster>,
        response_tx: tokio::sync::mpsc::UnboundedSender<String>,
    ) -> JsonRpcResponse {
        // Parse JSON-RPC request
        let request: JsonRpcRequest = match serde_json::from_str(text) {
            Ok(req) => req,
            Err(_) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError::parse_error()),
                    id: None,
                };
            }
        };
        
        // Validate JSON-RPC version
        if request.jsonrpc != "2.0" {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError::invalid_request()),
                id: request.id,
            };
        }
        
        // Handle method
        let result = match request.method.as_str() {
            "events.subscribe" => {
                Self::handle_subscribe(
                    request.params,
                    subscriptions,
                    event_broadcaster,
                    response_tx,
                ).await
            }
            "events.unsubscribe" => {
                Self::handle_unsubscribe(request.params, subscriptions).await
            }
            "events.list_subscriptions" => {
                Self::handle_list_subscriptions(subscriptions).await
            }
            _ => Err(JsonRpcError::method_not_found()),
        };
        
        // Build response
        match result {
            Ok(value) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(value),
                error: None,
                id: request.id,
            },
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(error),
                id: request.id,
            },
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
                    let notification = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(serde_json::json!({
                            "subscription_id": sub_id_clone,
                            "event": event,
                        })),
                        error: None,
                        id: None, // Notifications have no ID
                    };
                    
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
            sender: event_tx,
        };
        
        subscriptions.write().await.insert(sub_id.clone(), subscription);
        
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
        
        let existed = subscriptions.write().await.remove(&params.subscription_id).is_some();
        
        Ok(serde_json::json!({
            "success": existed,
        }))
    }
    
    /// Handle events.list_subscriptions method
    async fn handle_list_subscriptions(
        subscriptions: &Arc<RwLock<HashMap<String, Subscription>>>,
    ) -> Result<serde_json::Value, JsonRpcError> {
        let subs = subscriptions.read().await;
        
        let sub_list: Vec<serde_json::Value> = subs.values()
            .map(|sub| serde_json::json!({
                "subscription_id": sub.id,
                "filter": sub.filter,
            }))
            .collect();
        
        Ok(serde_json::json!({
            "subscriptions": sub_list,
            "count": sub_list.len(),
        }))
    }
}

#[cfg(test)]
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
            timestamp: Utc::now(),
        };
        
        assert!(filter.matches(&event));
        
        let event2 = GraphEvent::GraphStarted {
            graph_id: "other_graph".to_string(),
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
            timestamp: Utc::now(),
        };
        
        assert!(filter.matches(&event));
    }
}

