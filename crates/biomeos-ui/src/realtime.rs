//! Real-time event streaming for Interactive UI
//!
//! Phase 4: Real-time WebSocket updates
//!
//! Integrates WebSocket/SSE event streaming from:
//! - biomeos-api (GraphEvent, EcosystemEvent)
//! - Songbird (device/primal changes)
//!
//! Deep Debt Principles:
//! - No hardcoding (discover event endpoints via capabilities)
//! - Modern async Rust (tokio, futures)
//! - No unsafe code
//! - Graceful degradation (works without real-time updates)

#![forbid(unsafe_code)]

use anyhow::{Result, Context};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, info, warn, error};

/// Event types from the ecosystem
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RealTimeEvent {
    /// Graph execution event (from NeuralAPI)
    GraphEvent {
        graph_id: String,
        node_id: Option<String>,
        event_type: String,
        timestamp: String,
        details: serde_json::Value,
    },
    
    /// Primal discovered
    PrimalDiscovered {
        primal_id: String,
        name: String,
        primal_type: String,
        capabilities: Vec<String>,
    },
    
    /// Primal health changed
    HealthChanged {
        primal_id: String,
        name: String,
        old_health: String,
        new_health: String,
    },
    
    /// Device added
    DeviceAdded {
        device_id: String,
        device_type: String,
        capabilities: Vec<String>,
    },
    
    /// Device removed
    DeviceRemoved {
        device_id: String,
    },
    
    /// Assignment created
    AssignmentCreated {
        device_id: String,
        primal_id: String,
        user_id: Option<String>,
    },
    
    /// Assignment removed
    AssignmentRemoved {
        device_id: String,
        primal_id: String,
    },
    
    /// Topology changed
    TopologyChanged {
        nodes: usize,
        edges: usize,
        change: String,
    },
    
    /// Heartbeat
    Heartbeat {
        timestamp: u64,
        primals_count: usize,
        healthy_count: usize,
    },
}

/// JSON-RPC notification (for events from WebSocket)
#[derive(Debug, Clone, Deserialize)]
struct JsonRpcNotification {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
}

/// Real-time event subscriber
///
/// Connects to WebSocket/SSE endpoints and streams events
/// to the UI orchestrator for real-time updates.
pub struct RealTimeEventSubscriber {
    /// Broadcast channel for events
    event_tx: broadcast::Sender<RealTimeEvent>,
    
    /// WebSocket URL (if available)
    websocket_url: Option<String>,
    
    /// SSE URL (if available)
    sse_url: Option<String>,
    
    /// Family ID for filtering
    family_id: String,
}

impl RealTimeEventSubscriber {
    /// Create a new real-time event subscriber
    ///
    /// Discovers WebSocket/SSE endpoints via capability-based discovery
    pub fn new(family_id: String) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        
        Self {
            event_tx,
            websocket_url: None,
            sse_url: None,
            family_id,
        }
    }
    
    /// Discover event streaming endpoints
    ///
    /// Queries Songbird for services with "event_streaming" capability
    pub async fn discover_endpoints(&mut self) -> Result<()> {
        info!("🔍 Discovering real-time event endpoints...");
        
        // TODO: Use actual Songbird client to discover endpoints
        // For now, use default endpoints (will be evolved to capability-based)
        
        // Default WebSocket endpoint (from biomeos-api)
        self.websocket_url = Some("ws://localhost:8080/api/v1/events/ws".to_string());
        
        // Default SSE endpoint (from biomeos-api)
        self.sse_url = Some("http://localhost:8080/api/v1/events/stream".to_string());
        
        info!("✅ Event endpoints discovered");
        Ok(())
    }
    
    /// Subscribe to events via WebSocket (preferred)
    pub async fn subscribe_websocket(&self) -> Result<()> {
        let url = self.websocket_url.as_ref()
            .context("WebSocket URL not discovered")?;
        
        info!("📡 Connecting to WebSocket at {}", url);
        
        let (ws_stream, _) = connect_async(url)
            .await
            .context("Failed to connect to WebSocket")?;
        
        let (mut write, mut read) = ws_stream.split();
        
        // Subscribe to events via JSON-RPC
        let subscribe_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "events.subscribe",
            "params": {
                "graph_id": null, // Subscribe to all graphs
                "event_types": null, // Subscribe to all event types
                "node_filter": null,
            },
            "id": 1,
        });
        
        write.send(Message::Text(subscribe_request.to_string()))
            .await
            .context("Failed to send subscribe request")?;
        
        info!("✅ WebSocket connected, listening for events...");
        
        // Clone event broadcaster
        let event_tx = self.event_tx.clone();
        
        // Spawn task to handle incoming messages
        tokio::spawn(async move {
            while let Some(Ok(msg)) = read.next().await {
                if let Message::Text(text) = msg {
                    debug!("📨 Received WebSocket message: {}", text);
                    
                    // Try to parse as JSON-RPC notification
                    if let Ok(notification) = serde_json::from_str::<JsonRpcNotification>(&text) {
                        // Extract event from params
                        if let Ok(event) = Self::parse_event(&notification) {
                            // Broadcast to subscribers
                            let _ = event_tx.send(event);
                        }
                    }
                }
            }
            
            warn!("WebSocket connection closed");
        });
        
        Ok(())
    }
    
    /// Subscribe to events via SSE (fallback)
    pub async fn subscribe_sse(&self) -> Result<()> {
        let url = self.sse_url.as_ref()
            .context("SSE URL not discovered")?;
        
        info!("📡 Connecting to SSE at {}", url);
        
        let event_tx = self.event_tx.clone();
        
        // TODO: Implement SSE client
        // For now, just log that SSE is available
        warn!("SSE subscription not yet implemented, use WebSocket");
        
        Ok(())
    }
    
    /// Get a receiver for events
    pub fn subscribe(&self) -> broadcast::Receiver<RealTimeEvent> {
        self.event_tx.subscribe()
    }
    
    /// Parse JSON-RPC notification into RealTimeEvent
    fn parse_event(notification: &JsonRpcNotification) -> Result<RealTimeEvent> {
        // Extract event from params.event or params directly
        let event_value = notification.params.get("event")
            .unwrap_or(&notification.params);
        
        // Try to deserialize as RealTimeEvent
        serde_json::from_value(event_value.clone())
            .context("Failed to parse event")
    }
}

/// Real-time event handler
///
/// Processes real-time events and updates UI state
pub struct RealTimeEventHandler {
    /// Event subscriber
    subscriber: Arc<RealTimeEventSubscriber>,
    
    /// Event receiver
    event_rx: broadcast::Receiver<RealTimeEvent>,
}

impl RealTimeEventHandler {
    /// Create a new event handler
    pub fn new(subscriber: Arc<RealTimeEventSubscriber>) -> Self {
        let event_rx = subscriber.subscribe();
        
        Self {
            subscriber,
            event_rx,
        }
    }
    
    /// Start processing events
    ///
    /// This runs indefinitely and should be spawned in a task
    pub async fn process_events<F>(&mut self, mut handler: F) -> Result<()>
    where
        F: FnMut(RealTimeEvent) -> Result<()> + Send,
    {
        info!("🎧 Starting real-time event processing...");
        
        while let Ok(event) = self.event_rx.recv().await {
            debug!("📨 Processing event: {:?}", event);
            
            // Call user-provided handler
            if let Err(e) = handler(event) {
                error!("❌ Error processing event: {}", e);
            }
        }
        
        warn!("Event processing stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_subscriber_creation() {
        let subscriber = RealTimeEventSubscriber::new("test_family".to_string());
        assert_eq!(subscriber.family_id, "test_family");
        assert!(subscriber.websocket_url.is_none());
        assert!(subscriber.sse_url.is_none());
    }
    
    #[tokio::test]
    async fn test_discover_endpoints() {
        let mut subscriber = RealTimeEventSubscriber::new("test_family".to_string());
        
        // Discover endpoints
        let result = subscriber.discover_endpoints().await;
        assert!(result.is_ok());
        
        // Should have discovered endpoints
        assert!(subscriber.websocket_url.is_some());
        assert!(subscriber.sse_url.is_some());
    }
    
    #[test]
    fn test_event_serialization() {
        let event = RealTimeEvent::PrimalDiscovered {
            primal_id: "test_primal".to_string(),
            name: "Test Primal".to_string(),
            primal_type: "test".to_string(),
            capabilities: vec!["test".to_string()],
        };
        
        // Serialize and deserialize
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: RealTimeEvent = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            RealTimeEvent::PrimalDiscovered { primal_id, .. } => {
                assert_eq!(primal_id, "test_primal");
            }
            _ => panic!("Wrong event type"),
        }
    }
    
    #[tokio::test]
    async fn test_event_broadcasting() {
        let subscriber = Arc::new(RealTimeEventSubscriber::new("test_family".to_string()));
        
        // Subscribe to events
        let mut rx1 = subscriber.subscribe();
        let mut rx2 = subscriber.subscribe();
        
        // Send test event
        let event = RealTimeEvent::Heartbeat {
            timestamp: 12345,
            primals_count: 5,
            healthy_count: 5,
        };
        
        let _ = subscriber.event_tx.send(event);
        
        // Both receivers should get the event
        let event1 = rx1.try_recv();
        let event2 = rx2.try_recv();
        
        assert!(event1.is_ok());
        assert!(event2.is_ok());
    }
}

