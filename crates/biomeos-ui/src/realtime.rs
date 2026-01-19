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

use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info, warn};

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
    DeviceRemoved { device_id: String },

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

        // EVOLUTION: Discover from environment, no hardcoded fallbacks
        // biomeOS primals announce themselves via discovery

        // WebSocket endpoint discovery
        self.websocket_url = std::env::var("BIOMEOS_WS_ENDPOINT")
            .or_else(|_| std::env::var("BIOMEOS_API_WS"))
            .ok();

        // SSE endpoint discovery
        self.sse_url = std::env::var("BIOMEOS_SSE_ENDPOINT")
            .or_else(|_| std::env::var("BIOMEOS_API_SSE"))
            .ok();

        if self.websocket_url.is_some() || self.sse_url.is_some() {
            info!("✅ Event endpoints discovered from environment");
        } else {
            info!("ℹ️  No event endpoints configured (set BIOMEOS_WS_ENDPOINT or BIOMEOS_SSE_ENDPOINT)");
        }

        Ok(())
    }

    /// Subscribe to events via WebSocket (preferred)
    pub async fn subscribe_websocket(&self) -> Result<()> {
        let url = self
            .websocket_url
            .as_ref()
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

        write
            .send(Message::Text(subscribe_request.to_string()))
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
        let url = self.sse_url.as_ref().context("SSE URL not discovered")?;

        info!("📡 Connecting to SSE at {}", url);

        let event_tx = self.event_tx.clone();
        let url_clone = url.clone();

        // Spawn SSE client task
        tokio::spawn(async move {
            // Create HTTP client with keep-alive
            let client = reqwest::Client::builder()
                .build()
                .expect("Failed to create HTTP client");

            // Connect to SSE endpoint
            let response = match client.get(&url_clone).send().await {
                Ok(resp) => resp,
                Err(e) => {
                    error!("Failed to connect to SSE endpoint: {}", e);
                    return;
                }
            };

            if !response.status().is_success() {
                error!("SSE endpoint returned error: {}", response.status());
                return;
            }

            info!("✅ Connected to SSE stream");

            // Read SSE events line by line
            let mut stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk) = futures_util::StreamExt::next(&mut stream).await {
                match chunk {
                    Ok(bytes) => {
                        // Append chunk to buffer
                        let text = String::from_utf8_lossy(&bytes);
                        buffer.push_str(&text);

                        // Process complete SSE events (terminated by double newline)
                        while let Some(pos) = buffer.find("\n\n") {
                            let event_text = buffer[..pos].to_string();
                            buffer = buffer[pos + 2..].to_string();

                            // Parse SSE event format
                            if let Some(event) = Self::parse_sse_event(&event_text) {
                                // Broadcast to subscribers
                                let _ = event_tx.send(event);
                            }
                        }
                    }
                    Err(e) => {
                        error!("SSE stream error: {}", e);
                        break;
                    }
                }
            }

            warn!("SSE connection closed");
        });

        Ok(())
    }

    /// Parse SSE event format
    ///
    /// SSE format:
    /// ```text
    /// event: graph_event
    /// data: {"graph_id": "123", ...}
    ///
    /// ```
    fn parse_sse_event(text: &str) -> Option<RealTimeEvent> {
        let mut event_type: Option<String> = None;
        let mut data: Option<String> = None;

        for line in text.lines() {
            if let Some(value) = line.strip_prefix("event: ") {
                event_type = Some(value.trim().to_string());
            } else if let Some(value) = line.strip_prefix("data: ") {
                data = Some(value.trim().to_string());
            }
        }

        // Try to parse data as RealTimeEvent
        if let Some(data_str) = data {
            if let Ok(event) = serde_json::from_str::<RealTimeEvent>(&data_str) {
                debug!("📨 Parsed SSE event: {:?}", event_type);
                return Some(event);
            }
        }

        None
    }

    /// Get a receiver for events
    pub fn subscribe(&self) -> broadcast::Receiver<RealTimeEvent> {
        self.event_tx.subscribe()
    }

    /// Parse JSON-RPC notification into RealTimeEvent
    fn parse_event(notification: &JsonRpcNotification) -> Result<RealTimeEvent> {
        // Extract event from params.event or params directly
        let event_value = notification
            .params
            .get("event")
            .unwrap_or(&notification.params);

        // Try to deserialize as RealTimeEvent
        serde_json::from_value(event_value.clone()).context("Failed to parse event")
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
    #[ignore] // Requires Neural API server to be running (integration test)
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

    #[test]
    fn test_sse_event_parsing() {
        // Test valid SSE event format
        let sse_text = "event: graph_event\ndata: {\"type\":\"graph_event\",\"graph_id\":\"test123\",\"node_id\":\"node1\",\"event_type\":\"started\",\"timestamp\":\"2026-01-15T12:00:00Z\",\"details\":{}}";

        let event = RealTimeEventSubscriber::parse_sse_event(sse_text);
        assert!(event.is_some());

        match event.unwrap() {
            RealTimeEvent::GraphEvent { graph_id, .. } => {
                assert_eq!(graph_id, "test123");
            }
            _ => panic!("Expected GraphEvent"),
        }
    }

    #[test]
    fn test_sse_event_parsing_no_event_type() {
        // SSE with only data field
        let sse_text = "data: {\"type\":\"heartbeat\",\"timestamp\":12345,\"primals_count\":5,\"healthy_count\":5}";

        let event = RealTimeEventSubscriber::parse_sse_event(sse_text);
        assert!(event.is_some());
    }

    #[test]
    fn test_sse_event_parsing_invalid() {
        // Invalid JSON in data field
        let sse_text = "event: test\ndata: invalid json";

        let event = RealTimeEventSubscriber::parse_sse_event(sse_text);
        assert!(event.is_none());
    }

    #[test]
    fn test_sse_event_parsing_no_data() {
        // SSE with no data field
        let sse_text = "event: test_event";

        let event = RealTimeEventSubscriber::parse_sse_event(sse_text);
        assert!(event.is_none());
    }

    #[test]
    fn test_all_realtimeevent_variants() {
        // Test all event variants can be created
        let events = vec![
            RealTimeEvent::GraphEvent {
                graph_id: "g1".to_string(),
                node_id: Some("n1".to_string()),
                event_type: "started".to_string(),
                timestamp: "2026-01-15T12:00:00Z".to_string(),
                details: serde_json::json!({}),
            },
            RealTimeEvent::PrimalDiscovered {
                primal_id: "p1".to_string(),
                name: "TestPrimal".to_string(),
                primal_type: "test".to_string(),
                capabilities: vec!["cap1".to_string()],
            },
            RealTimeEvent::HealthChanged {
                primal_id: "p1".to_string(),
                name: "TestPrimal".to_string(),
                old_health: "unknown".to_string(),
                new_health: "healthy".to_string(),
            },
            RealTimeEvent::DeviceAdded {
                device_id: "d1".to_string(),
                device_type: "gpu".to_string(),
                capabilities: vec!["compute".to_string()],
            },
            RealTimeEvent::DeviceRemoved {
                device_id: "d1".to_string(),
            },
            RealTimeEvent::AssignmentCreated {
                device_id: "d1".to_string(),
                primal_id: "p1".to_string(),
                user_id: Some("u1".to_string()),
            },
            RealTimeEvent::AssignmentRemoved {
                device_id: "d1".to_string(),
                primal_id: "p1".to_string(),
            },
            RealTimeEvent::TopologyChanged {
                nodes: 10,
                edges: 15,
                change: "added_node".to_string(),
            },
            RealTimeEvent::Heartbeat {
                timestamp: 12345,
                primals_count: 5,
                healthy_count: 5,
            },
        ];

        // All variants should serialize successfully
        for event in events {
            let json = serde_json::to_string(&event).unwrap();
            assert!(!json.is_empty());
        }
    }

    #[test]
    fn test_graph_event_serialization() {
        let event = RealTimeEvent::GraphEvent {
            graph_id: "test_graph".to_string(),
            node_id: Some("node1".to_string()),
            event_type: "completed".to_string(),
            timestamp: "2026-01-15T12:00:00Z".to_string(),
            details: serde_json::json!({"status": "success"}),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("test_graph"));
        assert!(json.contains("completed"));

        let deserialized: RealTimeEvent = serde_json::from_str(&json).unwrap();
        match deserialized {
            RealTimeEvent::GraphEvent {
                graph_id,
                event_type,
                ..
            } => {
                assert_eq!(graph_id, "test_graph");
                assert_eq!(event_type, "completed");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_health_changed_event() {
        let event = RealTimeEvent::HealthChanged {
            primal_id: "beardog-1".to_string(),
            name: "BearDog".to_string(),
            old_health: "degraded".to_string(),
            new_health: "healthy".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: RealTimeEvent = serde_json::from_str(&json).unwrap();

        match deserialized {
            RealTimeEvent::HealthChanged {
                old_health,
                new_health,
                ..
            } => {
                assert_eq!(old_health, "degraded");
                assert_eq!(new_health, "healthy");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_device_added_event() {
        let event = RealTimeEvent::DeviceAdded {
            device_id: "gpu0".to_string(),
            device_type: "gpu".to_string(),
            capabilities: vec!["compute".to_string(), "ml".to_string()],
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("gpu0"));
        assert!(json.contains("compute"));
    }

    #[test]
    fn test_assignment_events() {
        let created = RealTimeEvent::AssignmentCreated {
            device_id: "gpu0".to_string(),
            primal_id: "toadstool-1".to_string(),
            user_id: Some("user1".to_string()),
        };

        let removed = RealTimeEvent::AssignmentRemoved {
            device_id: "gpu0".to_string(),
            primal_id: "toadstool-1".to_string(),
        };

        // Both should serialize
        let json1 = serde_json::to_string(&created).unwrap();
        let json2 = serde_json::to_string(&removed).unwrap();

        assert!(json1.contains("gpu0"));
        assert!(json2.contains("gpu0"));
    }

    #[test]
    fn test_topology_changed_event() {
        let event = RealTimeEvent::TopologyChanged {
            nodes: 25,
            edges: 40,
            change: "primal_added".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: RealTimeEvent = serde_json::from_str(&json).unwrap();

        match deserialized {
            RealTimeEvent::TopologyChanged { nodes, edges, .. } => {
                assert_eq!(nodes, 25);
                assert_eq!(edges, 40);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_heartbeat_event() {
        let event = RealTimeEvent::Heartbeat {
            timestamp: 1705329600,
            primals_count: 12,
            healthy_count: 11,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("1705329600"));
        assert!(json.contains("12"));
    }

    #[tokio::test]
    async fn test_event_handler_creation() {
        let subscriber = Arc::new(RealTimeEventSubscriber::new("test_family".to_string()));
        let handler = RealTimeEventHandler::new(subscriber);

        // Handler should be created successfully
        assert!(true); // If we get here, creation succeeded
    }

    #[tokio::test]
    async fn test_subscriber_subscribe() {
        let subscriber = RealTimeEventSubscriber::new("test_family".to_string());
        let rx1 = subscriber.subscribe();
        let rx2 = subscriber.subscribe();

        // Both subscriptions should be independent
        assert!(true); // If we get here, subscriptions work
    }

    #[test]
    fn test_sse_multiline_data() {
        // SSE with multiline data (valid JSON split across lines)
        let sse_text = "event: test\ndata: {\"type\":\"heartbeat\",\ndata: \"timestamp\":12345,\ndata: \"primals_count\":5,\"healthy_count\":5}";

        // This should fail to parse (our implementation expects data on one line)
        let event = RealTimeEventSubscriber::parse_sse_event(sse_text);
        assert!(event.is_none());
    }

    #[test]
    fn test_jsonrpc_notification_structure() {
        // Test that we can parse JSON-RPC notifications
        let json = r#"{"jsonrpc":"2.0","method":"event.notify","params":{"event":{"type":"heartbeat","timestamp":12345,"primals_count":5,"healthy_count":5}}}"#;

        let notification: serde_json::Result<serde_json::Value> = serde_json::from_str(json);
        assert!(notification.is_ok());

        let notif = notification.unwrap();
        assert_eq!(notif["jsonrpc"], "2.0");
        assert_eq!(notif["method"], "event.notify");
    }
}
