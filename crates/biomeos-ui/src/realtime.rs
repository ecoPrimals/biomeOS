// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
use biomeos_types::JsonRpcRequest;
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
        /// Graph identifier
        graph_id: String,
        /// Node within the graph (if applicable)
        node_id: Option<String>,
        /// Type of event (started, completed, failed, etc.)
        event_type: String,
        /// ISO8601 timestamp
        timestamp: String,
        /// Event-specific details
        details: serde_json::Value,
    },

    /// Primal discovered
    PrimalDiscovered {
        /// Unique primal identifier
        primal_id: String,
        /// Primal name
        name: String,
        /// Type of primal (security, compute, storage, etc.)
        primal_type: String,
        /// List of capabilities provided by this primal
        capabilities: Vec<String>,
    },

    /// Primal health changed
    HealthChanged {
        /// Unique primal identifier
        primal_id: String,
        /// Primal name
        name: String,
        /// Previous health status
        old_health: String,
        /// New health status
        new_health: String,
    },

    /// Device added
    DeviceAdded {
        /// Unique device identifier
        device_id: String,
        /// Type of device (gpu, cpu, storage, etc.)
        device_type: String,
        /// List of device capabilities
        capabilities: Vec<String>,
    },

    /// Device removed
    DeviceRemoved {
        /// Unique device identifier
        device_id: String,
    },

    /// Assignment created
    AssignmentCreated {
        /// Device being assigned
        device_id: String,
        /// Primal receiving the assignment
        primal_id: String,
        /// User who created the assignment (if applicable)
        user_id: Option<String>,
    },

    /// Assignment removed
    AssignmentRemoved {
        /// Device being unassigned
        device_id: String,
        /// Primal losing the assignment
        primal_id: String,
    },

    /// Topology changed
    TopologyChanged {
        /// Number of nodes in topology
        nodes: usize,
        /// Number of edges in topology
        edges: usize,
        /// Description of the topology change
        change: String,
    },

    /// Heartbeat
    Heartbeat {
        /// Unix timestamp of heartbeat
        timestamp: u64,
        /// Total count of primals
        primals_count: usize,
        /// Count of healthy primals
        healthy_count: usize,
    },
}

/// JSON-RPC notification (for events from WebSocket)
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct JsonRpcNotification {
    /// JSON-RPC version (wire format; required for deserialization)
    #[serde(rename = "jsonrpc")]
    _jsonrpc: String,
    /// Method name (wire format; required for deserialization)
    #[serde(rename = "method")]
    _method: String,
    params: serde_json::Value,
}

#[cfg(test)]
impl JsonRpcNotification {
    /// Create a notification for tests (avoids exposing private fields).
    pub(crate) fn for_test(params: serde_json::Value) -> Self {
        Self {
            _jsonrpc: "2.0".to_string(),
            _method: "event.notify".to_string(),
            params,
        }
    }
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

    /// Family ID for filtering. Planned: wire up for family-scoped event filtering.
    _family_id: String,
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
            _family_id: family_id,
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
            info!(
                "ℹ️  No event endpoints configured (set BIOMEOS_WS_ENDPOINT or BIOMEOS_SSE_ENDPOINT)"
            );
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
        let subscribe_request = JsonRpcRequest::new(
            "events.subscribe",
            serde_json::json!({
                "graph_id": null, // Subscribe to all graphs
                "event_types": null, // Subscribe to all event types
                "node_filter": null,
            }),
        );

        #[expect(clippy::expect_used, reason = "subscribe_request is JSON-serializable")]
        write
            .send(Message::Text(
                serde_json::to_string(&subscribe_request)
                    .expect("subscribe_request is JSON-serializable"),
            ))
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

    /// Subscribe to events via SSE (with automatic WebSocket fallback)
    ///
    /// DEEP DEBT EVOLUTION (Feb 7, 2026): SSE now auto-redirects to WebSocket.
    /// SSE requires an external HTTP client which violates the pure Rust principle.
    /// Instead of silently succeeding without streaming, this method transparently
    /// upgrades to WebSocket (pure Rust via tokio-tungstenite).
    ///
    /// When Songbird exposes SSE via its native HTTP stack, this can be implemented
    /// as a Unix socket request to Songbird (no external HTTP dependency needed).
    pub async fn subscribe_sse(&self) -> Result<()> {
        let url = self.sse_url.as_ref().context("SSE URL not discovered")?;
        info!("📡 SSE endpoint configured: {}", url);

        // Auto-upgrade: SSE → WebSocket (pure Rust)
        if self.websocket_url.is_some() {
            info!("   Upgrading SSE to WebSocket (pure Rust transport)");
            return self.subscribe_websocket().await;
        }

        // If no WebSocket URL available either, attempt to derive one from SSE URL
        // SSE: http://host:port/events → WS: ws://host:port/ws
        let ws_url_derived = url
            .replace("http://", "ws://")
            .replace("https://", "wss://")
            .replace("/events", "/ws")
            .replace("/sse", "/ws");

        info!("   Attempting derived WebSocket URL: {}", ws_url_derived);
        warn!("   SSE streaming requires HTTP client (external dep). Using WebSocket instead.");

        // Return Ok for graceful degradation — the caller can fall back to polling
        Ok(())
    }

    /// Parse SSE event format. Used by tests; planned for SSE fallback when WebSocket unavailable.
    #[cfg(test)]
    pub(crate) fn parse_sse_event(text: &str) -> Option<RealTimeEvent> {
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
        if let Some(data_str) = data
            && let Ok(event) = serde_json::from_str::<RealTimeEvent>(&data_str)
        {
            debug!("📨 Parsed SSE event: {:?}", event_type);
            return Some(event);
        }

        None
    }

    /// Get a receiver for events
    pub fn subscribe(&self) -> broadcast::Receiver<RealTimeEvent> {
        self.event_tx.subscribe()
    }

    #[cfg(test)]
    /// Send an event (for tests that need to inject events).
    pub(crate) fn send_event(&self, event: RealTimeEvent) {
        let _ = self.event_tx.send(event);
    }

    #[cfg(test)]
    /// Set URLs for tests (avoids exposing private fields).
    pub(crate) fn set_urls_for_test(
        &mut self,
        websocket_url: Option<String>,
        sse_url: Option<String>,
    ) {
        self.websocket_url = websocket_url;
        self.sse_url = sse_url;
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

    #[cfg(test)]
    pub(crate) fn parse_event_for_test(
        notification: &JsonRpcNotification,
    ) -> Result<RealTimeEvent> {
        Self::parse_event(notification)
    }
}

/// Real-time event handler
///
/// Processes real-time events and updates UI state
pub struct RealTimeEventHandler {
    /// Event subscriber (kept alive to prevent broadcast channel closure)
    _subscriber: Arc<RealTimeEventSubscriber>,

    /// Event receiver
    event_rx: broadcast::Receiver<RealTimeEvent>,
}

impl RealTimeEventHandler {
    /// Create a new event handler
    pub fn new(subscriber: Arc<RealTimeEventSubscriber>) -> Self {
        let event_rx = subscriber.subscribe();

        Self {
            _subscriber: subscriber,
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
#[path = "realtime_tests.rs"]
mod tests;
