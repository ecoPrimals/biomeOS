//! Lifecycle event types and handling

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Lifecycle event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleEvent {
    /// Event type
    pub event_type: LifecycleEventType,
    /// Event data
    pub data: serde_json::Value,
    /// Event metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Lifecycle event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleEventType {
    /// Primal starting
    Starting,
    /// Primal started
    Started,
    /// Primal stopping
    Stopping,
    /// Primal stopped
    Stopped,
    /// Primal restarting
    Restarting,
    /// Configuration updated
    ConfigurationUpdated,
    /// Health status changed
    HealthStatusChanged,
    /// Resource status changed
    ResourceStatusChanged,
}

/// Lifecycle response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleResponse {
    /// Event acknowledged
    pub acknowledged: bool,
    /// Response data
    pub data: Option<serde_json::Value>,
    /// Response metadata
    pub metadata: HashMap<String, serde_json::Value>,
}
