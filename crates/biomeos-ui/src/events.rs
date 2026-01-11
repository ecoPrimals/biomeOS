//! UI event system
//!
//! Events represent state changes that need to be propagated to the UI.

use crate::state::{Device, PrimalInfo, Assignment, LogEntry, Topology};
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

