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
use tokio::sync::{broadcast, RwLock};
use std::time::Duration;

/// Event emitted during graph execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GraphEvent {
    /// Graph execution started
    GraphStarted {
        graph_id: String,
        graph_name: String,
        total_nodes: usize,
        coordination: String,
        timestamp: DateTime<Utc>,
    },
    
    /// Node started executing
    NodeStarted {
        graph_id: String,
        node_id: String,
        primal: String,
        operation: String,
        timestamp: DateTime<Utc>,
    },
    
    /// Node completed successfully
    NodeCompleted {
        graph_id: String,
        node_id: String,
        duration_ms: u64,
        output: Option<serde_json::Value>,
        timestamp: DateTime<Utc>,
    },
    
    /// Node failed
    NodeFailed {
        graph_id: String,
        node_id: String,
        error: String,
        retry_attempt: usize,
        will_retry: bool,
        timestamp: DateTime<Utc>,
    },
    
    /// AI decision made during execution
    DecisionMade {
        graph_id: String,
        decision_type: String,
        reasoning: Vec<String>,
        confidence: f64,
        timestamp: DateTime<Utc>,
    },
    
    /// Graph execution paused
    GraphPaused {
        graph_id: String,
        reason: String,
        current_node: Option<String>,
        timestamp: DateTime<Utc>,
    },
    
    /// Graph execution resumed
    GraphResumed {
        graph_id: String,
        timestamp: DateTime<Utc>,
    },
    
    /// Graph execution completed
    GraphCompleted {
        graph_id: String,
        duration_ms: u64,
        nodes_executed: usize,
        nodes_failed: usize,
        success: bool,
        timestamp: DateTime<Utc>,
    },
    
    /// Graph execution cancelled
    GraphCancelled {
        graph_id: String,
        reason: String,
        nodes_completed: usize,
        timestamp: DateTime<Utc>,
    },
}

impl GraphEvent {
    /// Get the graph ID for this event
    pub fn graph_id(&self) -> &str {
        match self {
            GraphEvent::GraphStarted { graph_id, .. } => graph_id,
            GraphEvent::NodeStarted { graph_id, .. } => graph_id,
            GraphEvent::NodeCompleted { graph_id, .. } => graph_id,
            GraphEvent::NodeFailed { graph_id, .. } => graph_id,
            GraphEvent::DecisionMade { graph_id, .. } => graph_id,
            GraphEvent::GraphPaused { graph_id, .. } => graph_id,
            GraphEvent::GraphResumed { graph_id, .. } => graph_id,
            GraphEvent::GraphCompleted { graph_id, .. } => graph_id,
            GraphEvent::GraphCancelled { graph_id, .. } => graph_id,
        }
    }
    
    /// Get the timestamp for this event
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            GraphEvent::GraphStarted { timestamp, .. } => *timestamp,
            GraphEvent::NodeStarted { timestamp, .. } => *timestamp,
            GraphEvent::NodeCompleted { timestamp, .. } => *timestamp,
            GraphEvent::NodeFailed { timestamp, .. } => *timestamp,
            GraphEvent::DecisionMade { timestamp, .. } => *timestamp,
            GraphEvent::GraphPaused { timestamp, .. } => *timestamp,
            GraphEvent::GraphResumed { timestamp, .. } => *timestamp,
            GraphEvent::GraphCompleted { timestamp, .. } => *timestamp,
            GraphEvent::GraphCancelled { timestamp, .. } => *timestamp,
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
            Err(e) => Err(format!("Failed to broadcast event: {}", e)),
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
                _ = tokio::time::sleep_until(deadline) => break,
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
                graph_id: format!("graph_{}", i),
                node_id: format!("node_{}", i),
                primal: "test".to_string(),
                operation: "test".to_string(),
                timestamp: Utc::now(),
            };
            broadcaster.broadcast(event).await.unwrap();
        }
        
        // Receive in order
        for i in 0..5 {
            let received = receiver.recv().await.unwrap();
            assert_eq!(received.graph_id(), format!("graph_{}", i));
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
                graph_id: format!("graph_{}", i),
                node_id: format!("node_{}", i),
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
        let mut receiver = broadcaster.subscribe();
        
        // Spawn multiple tasks broadcasting events
        let mut handles = vec![];
        for i in 0..10 {
            let bc = broadcaster.clone();
            let handle = tokio::spawn(async move {
                for j in 0..10 {
                    let event = GraphEvent::NodeStarted {
                        graph_id: format!("graph_{}_{}", i, j),
                        node_id: format!("node_{}_{}", i, j),
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

