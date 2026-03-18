// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Sensor event routing for continuous graph execution.
//!
//! Routes real-time input events from sensor sources (e.g. petalTongue's
//! `SensorStreamRegistry`) through graph nodes to consumers (e.g. ludoSpring).
//!
//! Architecture:
//! ```text
//! petalTongue ──► SensorEventBus ──► Graph Node (input) ──► Graph Node (logic)
//!   keyboard         broadcast           poll_sensors          game.tick_logic
//!   mouse
//!   gamepad
//!   tracking
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use tracing::{debug, warn};

/// A sensor event from an input device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorEvent {
    /// Source device type (keyboard, mouse, gamepad, tracking, haptic)
    pub source: SensorSource,
    /// Event payload (device-specific)
    pub payload: serde_json::Value,
    /// Timestamp in microseconds since session start
    pub timestamp_us: u64,
    /// Sequence number (monotonic per source)
    pub sequence: u64,
}

/// Source type for sensor events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SensorSource {
    /// Keyboard key events
    Keyboard,
    /// Mouse position and button events
    Mouse,
    /// Gamepad axes and buttons
    Gamepad,
    /// Motion tracking (6DoF head/hand/tool)
    Tracking,
    /// Touch events
    Touch,
    /// Custom/extension sensor
    Custom,
}

impl std::fmt::Display for SensorSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SensorSource::Keyboard => write!(f, "keyboard"),
            SensorSource::Mouse => write!(f, "mouse"),
            SensorSource::Gamepad => write!(f, "gamepad"),
            SensorSource::Tracking => write!(f, "tracking"),
            SensorSource::Touch => write!(f, "touch"),
            SensorSource::Custom => write!(f, "custom"),
        }
    }
}

/// Bus for broadcasting sensor events to graph nodes.
///
/// Multiple producers (sensor adapters) can push events, and multiple
/// consumers (graph nodes) can subscribe to filtered streams.
#[derive(Clone)]
pub struct SensorEventBus {
    sender: Arc<broadcast::Sender<SensorEvent>>,
    stats: Arc<RwLock<SensorBusStats>>,
}

#[derive(Debug, Default)]
struct SensorBusStats {
    events_published: u64,
    events_by_source: HashMap<String, u64>,
}

impl SensorEventBus {
    /// Create a new sensor event bus with the given buffer capacity.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender: Arc::new(sender),
            stats: Arc::new(RwLock::new(SensorBusStats::default())),
        }
    }

    /// Publish a sensor event to all subscribers.
    pub async fn publish(&self, event: SensorEvent) {
        {
            let mut stats = self.stats.write().await;
            stats.events_published += 1;
            *stats
                .events_by_source
                .entry(event.source.to_string())
                .or_insert(0) += 1;
        }

        if let Ok(n) = self.sender.send(event) {
            debug!("Sensor event delivered to {} subscribers", n);
        }
    }

    /// Subscribe to all sensor events.
    pub fn subscribe(&self) -> broadcast::Receiver<SensorEvent> {
        self.sender.subscribe()
    }

    /// Get the number of active subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }

    /// Get statistics.
    pub async fn stats(&self) -> (u64, HashMap<String, u64>) {
        let stats = self.stats.read().await;
        (stats.events_published, stats.events_by_source.clone())
    }
}

/// Collects sensor events over a tick window for batch delivery to a graph node.
///
/// Used by the `input` node in a continuous graph to gather all events
/// that arrived since the last tick.
pub struct SensorCollector {
    receiver: broadcast::Receiver<SensorEvent>,
    filter: Option<Vec<SensorSource>>,
}

impl SensorCollector {
    /// Create a collector from a bus subscription.
    pub fn new(receiver: broadcast::Receiver<SensorEvent>) -> Self {
        Self {
            receiver,
            filter: None,
        }
    }

    /// Create a collector that only captures events from specific sources.
    pub fn with_filter(
        receiver: broadcast::Receiver<SensorEvent>,
        sources: Vec<SensorSource>,
    ) -> Self {
        Self {
            receiver,
            filter: Some(sources),
        }
    }

    /// Drain all buffered events (non-blocking). Returns events collected since last drain.
    pub fn drain(&mut self) -> Vec<SensorEvent> {
        let mut events = Vec::new();
        loop {
            match self.receiver.try_recv() {
                Ok(event) => {
                    if let Some(ref filter) = self.filter {
                        if filter.contains(&event.source) {
                            events.push(event);
                        }
                    } else {
                        events.push(event);
                    }
                }
                Err(
                    broadcast::error::TryRecvError::Empty | broadcast::error::TryRecvError::Closed,
                ) => break,
                Err(broadcast::error::TryRecvError::Lagged(n)) => {
                    warn!("SensorCollector lagged, lost {} events", n);
                }
            }
        }
        events
    }

    /// Convert collected events to a JSON value suitable for graph node input.
    pub fn drain_as_json(&mut self) -> serde_json::Value {
        let events = self.drain();
        serde_json::json!({
            "sensor_events": events,
            "count": events.len(),
        })
    }
}

/// Route sensor events from the bus into a graph node's input.
///
/// This is a convenience function that creates a collector, drains events,
/// and returns them as a JSON payload for the continuous executor.
pub fn collect_sensor_input(bus: &SensorEventBus, sources: &[SensorSource]) -> serde_json::Value {
    let rx = bus.subscribe();
    let mut collector = if sources.is_empty() {
        SensorCollector::new(rx)
    } else {
        SensorCollector::with_filter(rx, sources.to_vec())
    };
    collector.drain_as_json()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sensor_event_bus_creation() {
        let bus = SensorEventBus::new(100);
        assert_eq!(bus.subscriber_count(), 0);
    }

    #[tokio::test]
    async fn test_sensor_event_publish_subscribe() {
        let bus = SensorEventBus::new(100);
        let mut rx = bus.subscribe();

        let event = SensorEvent {
            source: SensorSource::Keyboard,
            payload: serde_json::json!({"key": "W", "pressed": true}),
            timestamp_us: 1000,
            sequence: 1,
        };

        bus.publish(event.clone()).await;

        let received = rx.recv().await.unwrap();
        assert_eq!(received.source, SensorSource::Keyboard);
        assert_eq!(received.sequence, 1);
    }

    #[tokio::test]
    async fn test_sensor_event_bus_multiple_subscribers() {
        let bus = SensorEventBus::new(100);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 2);

        bus.publish(SensorEvent {
            source: SensorSource::Mouse,
            payload: serde_json::json!({"x": 100, "y": 200}),
            timestamp_us: 2000,
            sequence: 1,
        })
        .await;

        assert!(rx1.recv().await.is_ok());
        assert!(rx2.recv().await.is_ok());
    }

    #[tokio::test]
    async fn test_sensor_event_bus_stats() {
        let bus = SensorEventBus::new(100);
        let _rx = bus.subscribe();

        for i in 0..5 {
            bus.publish(SensorEvent {
                source: SensorSource::Gamepad,
                payload: serde_json::json!({"axis": "left_stick", "x": i}),
                timestamp_us: i * 1000,
                sequence: i,
            })
            .await;
        }

        let (total, by_source) = bus.stats().await;
        assert_eq!(total, 5);
        assert_eq!(by_source.get("gamepad"), Some(&5));
    }

    #[tokio::test]
    async fn test_sensor_collector_drain() {
        let bus = SensorEventBus::new(100);
        let rx = bus.subscribe();
        let mut collector = SensorCollector::new(rx);

        for i in 0..3 {
            bus.publish(SensorEvent {
                source: SensorSource::Keyboard,
                payload: serde_json::json!({"key": format!("key_{}", i)}),
                timestamp_us: i * 1000,
                sequence: i,
            })
            .await;
        }

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let events = collector.drain();
        assert_eq!(events.len(), 3);
    }

    #[tokio::test]
    async fn test_sensor_collector_filter() {
        let bus = SensorEventBus::new(100);
        let rx = bus.subscribe();
        let mut collector = SensorCollector::with_filter(rx, vec![SensorSource::Mouse]);

        bus.publish(SensorEvent {
            source: SensorSource::Keyboard,
            payload: serde_json::json!({}),
            timestamp_us: 0,
            sequence: 0,
        })
        .await;
        bus.publish(SensorEvent {
            source: SensorSource::Mouse,
            payload: serde_json::json!({"x": 1}),
            timestamp_us: 1,
            sequence: 1,
        })
        .await;
        bus.publish(SensorEvent {
            source: SensorSource::Gamepad,
            payload: serde_json::json!({}),
            timestamp_us: 2,
            sequence: 2,
        })
        .await;

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let events = collector.drain();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].source, SensorSource::Mouse);
    }

    #[tokio::test]
    async fn test_sensor_collector_drain_as_json() {
        let bus = SensorEventBus::new(100);
        let rx = bus.subscribe();
        let mut collector = SensorCollector::new(rx);

        bus.publish(SensorEvent {
            source: SensorSource::Touch,
            payload: serde_json::json!({"finger": 0, "x": 50, "y": 50}),
            timestamp_us: 0,
            sequence: 0,
        })
        .await;

        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let json = collector.drain_as_json();
        assert_eq!(json["count"], 1);
    }

    #[test]
    fn test_sensor_source_display() {
        assert_eq!(SensorSource::Keyboard.to_string(), "keyboard");
        assert_eq!(SensorSource::Mouse.to_string(), "mouse");
        assert_eq!(SensorSource::Gamepad.to_string(), "gamepad");
        assert_eq!(SensorSource::Tracking.to_string(), "tracking");
        assert_eq!(SensorSource::Touch.to_string(), "touch");
        assert_eq!(SensorSource::Custom.to_string(), "custom");
    }

    #[test]
    fn test_sensor_source_serde() {
        let source = SensorSource::Tracking;
        let json = serde_json::to_string(&source).unwrap();
        assert_eq!(json, "\"tracking\"");
        let back: SensorSource = serde_json::from_str(&json).unwrap();
        assert_eq!(back, SensorSource::Tracking);
    }

    #[test]
    fn test_sensor_event_serde_roundtrip() {
        let event = SensorEvent {
            source: SensorSource::Gamepad,
            payload: serde_json::json!({"button": "A", "pressed": true}),
            timestamp_us: 12345,
            sequence: 42,
        };
        let json = serde_json::to_string(&event).unwrap();
        let back: SensorEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(back.source, SensorSource::Gamepad);
        assert_eq!(back.sequence, 42);
        assert_eq!(back.timestamp_us, 12345);
    }
}
