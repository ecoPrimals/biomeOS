// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Event streaming system for real-time graph execution visibility
//!
//! This module provides event emission and broadcasting for graph execution,
//! enabling real-time UI updates and monitoring.
//!
//! Deep Debt Principles:
//! - Modern idiomatic Rust (async/await, channels)
//! - No unsafe code
//! - Type-safe events
//! - Multiple subscriber support

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, broadcast};

/// Event emitted during graph execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GraphEvent {
    /// Graph execution started
    GraphStarted {
        /// Unique identifier for this graph execution
        graph_id: String,
        /// Human-readable name of the graph
        graph_name: String,
        /// Total number of nodes in the graph
        total_nodes: usize,
        /// Coordination mode (sequential/parallel)
        coordination: String,
        /// When this event occurred
        timestamp: DateTime<Utc>,
    },

    /// Node started executing
    NodeStarted {
        /// Graph execution this node belongs to
        graph_id: String,
        /// Unique identifier for this node
        node_id: String,
        /// Primal executing this node
        primal: String,
        /// Operation being performed
        operation: String,
        /// When this event occurred
        timestamp: DateTime<Utc>,
    },

    /// Node completed successfully
    NodeCompleted {
        /// Graph execution this node belongs to
        graph_id: String,
        /// Unique identifier for this node
        node_id: String,
        /// Duration of execution in milliseconds
        duration_ms: u64,
        /// Optional output data from the node
        output: Option<serde_json::Value>,
        /// When this event occurred
        timestamp: DateTime<Utc>,
    },

    /// Node failed
    NodeFailed {
        /// Graph execution this node belongs to
        graph_id: String,
        /// Unique identifier for this node
        node_id: String,
        /// Error message describing the failure
        error: String,
        /// Which retry attempt this was (0 for first attempt)
        retry_attempt: usize,
        /// Whether the node will be retried
        will_retry: bool,
        /// When this event occurred
        timestamp: DateTime<Utc>,
    },

    /// AI decision made during execution
    DecisionMade {
        /// Graph execution this decision belongs to
        graph_id: String,
        /// Type of decision (e.g., "node_selection", "parameter_adjustment")
        decision_type: String,
        /// AI reasoning steps that led to the decision
        reasoning: Vec<String>,
        /// Confidence score (0.0 to 1.0)
        confidence: f64,
        /// When this event occurred
        timestamp: DateTime<Utc>,
    },

    /// Graph execution paused
    GraphPaused {
        /// Graph execution that was paused
        graph_id: String,
        /// Reason for pausing
        reason: String,
        /// Node that was executing when paused
        current_node: Option<String>,
        /// When this event occurred
        timestamp: DateTime<Utc>,
    },

    /// Graph execution resumed
    GraphResumed {
        /// Graph execution that was resumed
        graph_id: String,
        /// When this event occurred
        timestamp: DateTime<Utc>,
    },

    /// Graph execution completed
    GraphCompleted {
        /// Graph execution that completed
        graph_id: String,
        /// Total execution duration in milliseconds
        duration_ms: u64,
        /// Number of nodes that were executed
        nodes_executed: usize,
        /// Number of nodes that failed
        nodes_failed: usize,
        /// Whether the graph completed successfully
        success: bool,
        /// When this event occurred
        timestamp: DateTime<Utc>,
    },

    /// Graph execution cancelled
    GraphCancelled {
        /// Graph execution that was cancelled
        graph_id: String,
        /// Reason for cancellation
        reason: String,
        /// Number of nodes that completed before cancellation
        nodes_completed: usize,
        /// When this event occurred
        timestamp: DateTime<Utc>,
    },

    /// Continuous session started (tick loop begins)
    SessionStarted {
        /// Graph running continuously
        graph_id: String,
        /// Target tick rate in Hz
        target_hz: f64,
        /// When this event occurred
        timestamp: DateTime<Utc>,
    },

    /// One tick of a continuous graph completed
    TickCompleted {
        /// Graph running continuously
        graph_id: String,
        /// Tick number (monotonically increasing)
        tick: u64,
        /// Actual tick duration in microseconds
        duration_us: u64,
        /// Number of nodes that exceeded their budget
        budget_overruns: usize,
        /// When this event occurred
        timestamp: DateTime<Utc>,
    },

    /// Continuous session state changed (paused, resumed, stopped)
    SessionStateChanged {
        /// Graph running continuously
        graph_id: String,
        /// New session state
        new_state: String,
        /// Tick count at time of state change
        tick_at_change: u64,
        /// When this event occurred
        timestamp: DateTime<Utc>,
    },
}

impl GraphEvent {
    /// Get the graph ID for this event
    pub fn graph_id(&self) -> &str {
        match self {
            GraphEvent::GraphStarted { graph_id, .. }
            | GraphEvent::NodeStarted { graph_id, .. }
            | GraphEvent::NodeCompleted { graph_id, .. }
            | GraphEvent::NodeFailed { graph_id, .. }
            | GraphEvent::DecisionMade { graph_id, .. }
            | GraphEvent::GraphPaused { graph_id, .. }
            | GraphEvent::GraphResumed { graph_id, .. }
            | GraphEvent::GraphCompleted { graph_id, .. }
            | GraphEvent::GraphCancelled { graph_id, .. }
            | GraphEvent::SessionStarted { graph_id, .. }
            | GraphEvent::TickCompleted { graph_id, .. }
            | GraphEvent::SessionStateChanged { graph_id, .. } => graph_id,
        }
    }

    /// Get the timestamp for this event
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            GraphEvent::GraphStarted { timestamp, .. }
            | GraphEvent::NodeStarted { timestamp, .. }
            | GraphEvent::NodeCompleted { timestamp, .. }
            | GraphEvent::NodeFailed { timestamp, .. }
            | GraphEvent::DecisionMade { timestamp, .. }
            | GraphEvent::GraphPaused { timestamp, .. }
            | GraphEvent::GraphResumed { timestamp, .. }
            | GraphEvent::GraphCompleted { timestamp, .. }
            | GraphEvent::GraphCancelled { timestamp, .. }
            | GraphEvent::SessionStarted { timestamp, .. }
            | GraphEvent::TickCompleted { timestamp, .. }
            | GraphEvent::SessionStateChanged { timestamp, .. } => *timestamp,
        }
    }
}

/// Event broadcaster for graph execution
///
/// Uses tokio broadcast channels for efficient multi-subscriber support.
/// Each subscriber gets their own receiver that can be read independently.
#[derive(Clone)]
pub struct GraphEventBroadcaster {
    sender: Arc<broadcast::Sender<GraphEvent>>,
    stats: Arc<RwLock<BroadcasterStats>>,
}

#[derive(Debug, Default)]
struct BroadcasterStats {
    events_sent: u64,
    active_subscribers: usize,
}

impl GraphEventBroadcaster {
    /// Create a new event broadcaster with specified capacity
    ///
    /// Capacity determines how many events can be buffered per subscriber.
    /// If a subscriber falls behind, older events will be dropped.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);

        Self {
            sender: Arc::new(sender),
            stats: Arc::new(RwLock::new(BroadcasterStats::default())),
        }
    }

    /// Subscribe to graph events
    ///
    /// Returns a receiver that will receive all future events.
    /// Each subscriber is independent and can consume events at their own pace.
    pub fn subscribe(&self) -> broadcast::Receiver<GraphEvent> {
        let receiver = self.sender.subscribe();

        // Update stats
        let stats = self.stats.clone();
        tokio::spawn(async move {
            let mut stats = stats.write().await;
            stats.active_subscribers += 1;
        });

        receiver
    }

    /// Broadcast an event to all subscribers
    ///
    /// This is non-blocking and will not wait for subscribers to process the event.
    /// If a subscriber's buffer is full, the oldest event in their buffer will be dropped.
    pub async fn broadcast(&self, event: GraphEvent) -> Result<usize, String> {
        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.events_sent += 1;
        }

        // Send to all subscribers
        match self.sender.send(event) {
            Ok(count) => Ok(count),
            Err(e) => Err(format!("Failed to broadcast event: {e}")),
        }
    }

    /// Get broadcaster statistics
    pub async fn stats(&self) -> (u64, usize) {
        let stats = self.stats.read().await;
        (stats.events_sent, stats.active_subscribers)
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

/// Helper for collecting events from a receiver
///
/// Useful for testing and debugging.
pub struct EventCollector {
    receiver: broadcast::Receiver<GraphEvent>,
    events: Vec<GraphEvent>,
}

impl EventCollector {
    /// Create a new event collector from a receiver
    pub fn new(receiver: broadcast::Receiver<GraphEvent>) -> Self {
        Self {
            receiver,
            events: Vec::new(),
        }
    }

    /// Collect events for a specified duration
    pub async fn collect_for(&mut self, duration: Duration) -> Vec<GraphEvent> {
        let deadline = tokio::time::Instant::now() + duration;

        loop {
            tokio::select! {
                result = self.receiver.recv() => {
                    match result {
                        Ok(event) => self.events.push(event),
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            tracing::warn!("Event collector lagged, skipped {} events", skipped);
                        }
                        Err(broadcast::error::RecvError::Closed) => break,
                    }
                }
                () = tokio::time::sleep_until(deadline) => break,
            }
        }

        self.events.clone()
    }

    /// Collect a specific number of events
    pub async fn collect_count(&mut self, count: usize) -> Vec<GraphEvent> {
        while self.events.len() < count {
            match self.receiver.recv().await {
                Ok(event) => self.events.push(event),
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    tracing::warn!("Event collector lagged, skipped {} events", skipped);
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }

        self.events.clone()
    }

    /// Get all collected events
    pub fn events(&self) -> &[GraphEvent] {
        &self.events
    }
}

#[allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_broadcaster_creation() {
        let broadcaster = GraphEventBroadcaster::new(100);
        assert_eq!(broadcaster.subscriber_count(), 0);
    }

    #[tokio::test]
    async fn test_single_subscriber() {
        let broadcaster = GraphEventBroadcaster::new(100);
        let mut receiver = broadcaster.subscribe();

        assert_eq!(broadcaster.subscriber_count(), 1);

        let event = GraphEvent::GraphStarted {
            graph_id: "test".to_string(),
            graph_name: "Test Graph".to_string(),
            total_nodes: 5,
            coordination: "sequential".to_string(),
            timestamp: Utc::now(),
        };

        broadcaster.broadcast(event.clone()).await.unwrap();

        let received = receiver.recv().await.unwrap();
        assert_eq!(received.graph_id(), "test");
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let broadcaster = GraphEventBroadcaster::new(100);
        let mut receiver1 = broadcaster.subscribe();
        let mut receiver2 = broadcaster.subscribe();
        let mut receiver3 = broadcaster.subscribe();

        assert_eq!(broadcaster.subscriber_count(), 3);

        let event = GraphEvent::NodeStarted {
            graph_id: "test".to_string(),
            node_id: "node1".to_string(),
            primal: "test_primal".to_string(),
            operation: "test_op".to_string(),
            timestamp: Utc::now(),
        };

        let count = broadcaster.broadcast(event.clone()).await.unwrap();
        assert_eq!(count, 3); // All 3 subscribers received it

        // All receivers should get the event
        assert_eq!(receiver1.recv().await.unwrap().graph_id(), "test");
        assert_eq!(receiver2.recv().await.unwrap().graph_id(), "test");
        assert_eq!(receiver3.recv().await.unwrap().graph_id(), "test");
    }

    #[tokio::test]
    async fn test_event_ordering() {
        let broadcaster = GraphEventBroadcaster::new(100);
        let mut receiver = broadcaster.subscribe();

        // Send multiple events
        for i in 0..5 {
            let event = GraphEvent::NodeStarted {
                graph_id: format!("graph_{i}"),
                node_id: format!("node_{i}"),
                primal: "test".to_string(),
                operation: "test".to_string(),
                timestamp: Utc::now(),
            };
            broadcaster.broadcast(event).await.unwrap();
        }

        // Receive in order
        for i in 0..5 {
            let received = receiver.recv().await.unwrap();
            assert_eq!(received.graph_id(), format!("graph_{i}"));
        }
    }

    #[tokio::test]
    async fn test_event_collector() {
        let broadcaster = GraphEventBroadcaster::new(100);
        let receiver = broadcaster.subscribe();
        let mut collector = EventCollector::new(receiver);

        // Broadcast events
        for i in 0..3 {
            let event = GraphEvent::NodeCompleted {
                graph_id: format!("graph_{i}"),
                node_id: format!("node_{i}"),
                duration_ms: 100,
                output: None,
                timestamp: Utc::now(),
            };
            broadcaster.broadcast(event).await.unwrap();
        }

        // Collect events
        let events = collector.collect_count(3).await;
        assert_eq!(events.len(), 3);
    }

    #[tokio::test]
    async fn test_broadcaster_stats() {
        let broadcaster = GraphEventBroadcaster::new(100);
        let _receiver1 = broadcaster.subscribe();
        let _receiver2 = broadcaster.subscribe();

        // Give time for stats to update
        sleep(Duration::from_millis(10)).await;

        let event = GraphEvent::GraphStarted {
            graph_id: "test".to_string(),
            graph_name: "Test".to_string(),
            total_nodes: 1,
            coordination: "sequential".to_string(),
            timestamp: Utc::now(),
        };

        broadcaster.broadcast(event).await.unwrap();

        let (events_sent, _) = broadcaster.stats().await;
        assert_eq!(events_sent, 1);
    }

    #[tokio::test]
    async fn test_graph_event_accessors() {
        let event = GraphEvent::NodeFailed {
            graph_id: "test_graph".to_string(),
            node_id: "node1".to_string(),
            error: "Test error".to_string(),
            retry_attempt: 1,
            will_retry: true,
            timestamp: Utc::now(),
        };

        assert_eq!(event.graph_id(), "test_graph");
        assert!(event.timestamp() <= Utc::now());
    }

    #[tokio::test]
    async fn test_concurrent_broadcasting() {
        let broadcaster = GraphEventBroadcaster::new(1000);
        let receiver = broadcaster.subscribe();

        // Spawn multiple tasks broadcasting events
        let mut handles = vec![];
        for i in 0..10 {
            let bc = broadcaster.clone();
            let handle = tokio::spawn(async move {
                for j in 0..10 {
                    let event = GraphEvent::NodeStarted {
                        graph_id: format!("graph_{i}_{j}"),
                        node_id: format!("node_{i}_{j}"),
                        primal: "test".to_string(),
                        operation: "test".to_string(),
                        timestamp: Utc::now(),
                    };
                    bc.broadcast(event).await.unwrap();
                }
            });
            handles.push(handle);
        }

        // Wait for all broadcasts
        for handle in handles {
            handle.await.unwrap();
        }

        // Collect all events
        let mut collector = EventCollector::new(receiver);
        let events = collector.collect_count(100).await;
        assert_eq!(events.len(), 100);
    }
}
