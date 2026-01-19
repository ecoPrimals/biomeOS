//! UI event system
//!
//! Events represent state changes that need to be propagated to the UI.

use crate::state::{Assignment, Device, LogEntry, PrimalInfo, Topology};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

/// UI event - represents a state change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIEvent {
    /// A new device was discovered
    DeviceDiscovered(Device),

    /// A device was removed
    DeviceRemoved(String),

    /// A device status changed
    DeviceStatusChanged { device_id: String, status: String },

    /// A new primal was registered
    PrimalRegistered(PrimalInfo),

    /// A primal was removed
    PrimalRemoved(String),

    /// A primal status changed
    PrimalStatusChanged { primal_id: String, status: String },

    /// An assignment was created
    AssignmentCreated(Assignment),

    /// An assignment was removed
    AssignmentRemoved { device_id: String },

    /// A log entry was added
    LogEntry(LogEntry),

    /// The topology graph changed
    TopologyChanged(Topology),

    /// An error occurred
    Error { message: String },
}

/// Event stream - broadcasts UI events
pub type EventStream = broadcast::Receiver<UIEvent>;

/// Event broadcaster
#[derive(Clone)]
pub struct EventBroadcaster {
    tx: broadcast::Sender<UIEvent>,
}

impl EventBroadcaster {
    /// Create a new event broadcaster
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self { tx }
    }

    /// Emit an event
    pub fn emit(&self, event: UIEvent) {
        // Ignore send errors (no receivers = no problem)
        let _ = self.tx.send(event);
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> EventStream {
        self.tx.subscribe()
    }
}

impl Default for EventBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{
        Assignment, AssignmentStatus, Device, DeviceStatus, HealthMetrics, LogEntry, LogLevel,
        PrimalInfo, PrimalStatus, Topology,
    };
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn test_event_broadcaster_creation() {
        let broadcaster = EventBroadcaster::new();
        let _ = broadcaster.subscribe();
    }

    #[test]
    fn test_event_broadcaster_default() {
        let broadcaster = EventBroadcaster::default();
        let _ = broadcaster.subscribe();
    }

    #[tokio::test]
    async fn test_emit_device_discovered() {
        let broadcaster = EventBroadcaster::new();
        let mut rx = broadcaster.subscribe();

        let device = Device {
            id: "gpu0".to_string(),
            device_type: "gpu".to_string(),
            name: "Test GPU".to_string(),
            capabilities: vec![],
            resources: HashMap::new(),
            status: DeviceStatus::Available,
        };

        broadcaster.emit(UIEvent::DeviceDiscovered(device.clone()));

        let event = rx.recv().await.expect("Should receive event");
        match event {
            UIEvent::DeviceDiscovered(d) => assert_eq!(d.id, "gpu0"),
            _ => panic!("Wrong event type"),
        }
    }

    #[tokio::test]
    async fn test_emit_device_removed() {
        let broadcaster = EventBroadcaster::new();
        let mut rx = broadcaster.subscribe();

        broadcaster.emit(UIEvent::DeviceRemoved("gpu0".to_string()));

        let event = rx.recv().await.expect("Should receive event");
        match event {
            UIEvent::DeviceRemoved(id) => assert_eq!(id, "gpu0"),
            _ => panic!("Wrong event type"),
        }
    }

    #[tokio::test]
    async fn test_emit_device_status_changed() {
        let broadcaster = EventBroadcaster::new();
        let mut rx = broadcaster.subscribe();

        broadcaster.emit(UIEvent::DeviceStatusChanged {
            device_id: "gpu0".to_string(),
            status: "assigned".to_string(),
        });

        let event = rx.recv().await.expect("Should receive event");
        match event {
            UIEvent::DeviceStatusChanged { device_id, status } => {
                assert_eq!(device_id, "gpu0");
                assert_eq!(status, "assigned");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[tokio::test]
    async fn test_emit_primal_registered() {
        let broadcaster = EventBroadcaster::new();
        let mut rx = broadcaster.subscribe();

        let primal = PrimalInfo {
            id: "beardog-1".to_string(),
            name: "beardog".to_string(),
            capabilities: vec!["security".to_string()],
            status: PrimalStatus::Running,
            health: HealthMetrics {
                status: "healthy".to_string(),
                uptime: 3600,
                cpu_usage: 10.0,
                memory_usage: 256,
            },
        };

        broadcaster.emit(UIEvent::PrimalRegistered(primal.clone()));

        let event = rx.recv().await.expect("Should receive event");
        match event {
            UIEvent::PrimalRegistered(p) => assert_eq!(p.name, "beardog"),
            _ => panic!("Wrong event type"),
        }
    }

    #[tokio::test]
    async fn test_emit_assignment_created() {
        let broadcaster = EventBroadcaster::new();
        let mut rx = broadcaster.subscribe();

        let assignment = Assignment {
            device_id: "gpu0".to_string(),
            primal_id: "toadstool-1".to_string(),
            assigned_at: Utc::now(),
            status: AssignmentStatus::Active,
        };

        broadcaster.emit(UIEvent::AssignmentCreated(assignment.clone()));

        let event = rx.recv().await.expect("Should receive event");
        match event {
            UIEvent::AssignmentCreated(a) => {
                assert_eq!(a.device_id, "gpu0");
                assert_eq!(a.primal_id, "toadstool-1");
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[tokio::test]
    async fn test_emit_log_entry() {
        let broadcaster = EventBroadcaster::new();
        let mut rx = broadcaster.subscribe();

        let log = LogEntry {
            timestamp: Utc::now(),
            source: "beardog".to_string(),
            level: LogLevel::Info,
            message: "Test message".to_string(),
        };

        broadcaster.emit(UIEvent::LogEntry(log.clone()));

        let event = rx.recv().await.expect("Should receive event");
        match event {
            UIEvent::LogEntry(l) => assert_eq!(l.source, "beardog"),
            _ => panic!("Wrong event type"),
        }
    }

    #[tokio::test]
    async fn test_emit_topology_changed() {
        let broadcaster = EventBroadcaster::new();
        let mut rx = broadcaster.subscribe();

        let topology = Topology::default();
        broadcaster.emit(UIEvent::TopologyChanged(topology));

        let event = rx.recv().await.expect("Should receive event");
        match event {
            UIEvent::TopologyChanged(_) => {}
            _ => panic!("Wrong event type"),
        }
    }

    #[tokio::test]
    async fn test_emit_error() {
        let broadcaster = EventBroadcaster::new();
        let mut rx = broadcaster.subscribe();

        broadcaster.emit(UIEvent::Error {
            message: "Test error".to_string(),
        });

        let event = rx.recv().await.expect("Should receive event");
        match event {
            UIEvent::Error { message } => assert_eq!(message, "Test error"),
            _ => panic!("Wrong event type"),
        }
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let broadcaster = EventBroadcaster::new();
        let mut rx1 = broadcaster.subscribe();
        let mut rx2 = broadcaster.subscribe();

        broadcaster.emit(UIEvent::DeviceRemoved("test".to_string()));

        let event1 = rx1.recv().await.expect("Sub 1 should receive");
        let event2 = rx2.recv().await.expect("Sub 2 should receive");

        match (event1, event2) {
            (UIEvent::DeviceRemoved(id1), UIEvent::DeviceRemoved(id2)) => {
                assert_eq!(id1, "test");
                assert_eq!(id2, "test");
            }
            _ => panic!("Wrong event types"),
        }
    }

    #[test]
    fn test_event_serialization() {
        let event = UIEvent::Error {
            message: "test error".to_string(),
        };

        let json = serde_json::to_string(&event).expect("Should serialize");
        assert!(json.contains("test error"));

        let deserialized: UIEvent = serde_json::from_str(&json).expect("Should deserialize");
        match deserialized {
            UIEvent::Error { message } => assert_eq!(message, "test error"),
            _ => panic!("Wrong event type after deserialization"),
        }
    }
}
