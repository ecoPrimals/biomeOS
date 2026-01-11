//! UI state management
//!
//! Manages the current state of devices, primals, and assignments.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

/// UI state - represents the current view of the system
#[derive(Debug, Clone, Default)]
pub struct UIState {
    /// Available hardware devices
    pub devices: HashMap<DeviceId, Device>,
    
    /// Active primals
    pub primals: HashMap<PrimalId, PrimalInfo>,
    
    /// Device → Primal assignments
    pub assignments: HashMap<DeviceId, Assignment>,
    
    /// Event log (last 1000 entries)
    pub logs: VecDeque<LogEntry>,
    
    /// Topology graph (nodes + edges)
    pub topology: Topology,
}

/// Device ID (unique identifier)
pub type DeviceId = String;

/// Primal ID (unique identifier)
pub type PrimalId = String;

/// Hardware device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    /// Unique device ID
    pub id: DeviceId,
    
    /// Device type (gpu, cpu, tpm, network, storage, etc.)
    pub device_type: String,
    
    /// Human-readable name
    pub name: String,
    
    /// Device capabilities
    pub capabilities: Vec<String>,
    
    /// Resource information
    pub resources: HashMap<String, serde_json::Value>,
    
    /// Current status
    pub status: DeviceStatus,
}

/// Device status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceStatus {
    /// Available for assignment
    Available,
    
    /// Assigned to a primal
    Assigned,
    
    /// Offline or unavailable
    Offline,
    
    /// Error state
    Error,
}

/// Primal information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Unique primal ID
    pub id: PrimalId,
    
    /// Primal name (beardog, songbird, etc.)
    pub name: String,
    
    /// Primal capabilities
    pub capabilities: Vec<String>,
    
    /// Current status
    pub status: PrimalStatus,
    
    /// Health metrics
    pub health: HealthMetrics,
}

/// Primal status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrimalStatus {
    /// Running and healthy
    Running,
    
    /// Starting up
    Starting,
    
    /// Stopping
    Stopping,
    
    /// Stopped
    Stopped,
    
    /// Error state
    Error,
}

/// Health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// Health status
    pub status: String,
    
    /// Uptime in seconds
    pub uptime: u64,
    
    /// CPU usage (0-100)
    pub cpu_usage: f32,
    
    /// Memory usage in MB
    pub memory_usage: u64,
}

/// Device assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    /// Device ID
    pub device_id: DeviceId,
    
    /// Primal ID
    pub primal_id: PrimalId,
    
    /// When assigned
    pub assigned_at: chrono::DateTime<chrono::Utc>,
    
    /// Assignment status
    pub status: AssignmentStatus,
}

/// Assignment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssignmentStatus {
    /// Assignment pending validation
    Pending,
    
    /// Assignment active
    Active,
    
    /// Assignment failed
    Failed,
}

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    
    /// Source primal
    pub source: String,
    
    /// Log level
    pub level: LogLevel,
    
    /// Message
    pub message: String,
}

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    /// Info message
    Info,
    
    /// Warning message
    Warning,
    
    /// Error message
    Error,
}

/// Topology graph
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Topology {
    /// Graph nodes (devices + primals)
    pub nodes: Vec<TopologyNode>,
    
    /// Graph edges (assignments + connections)
    pub edges: Vec<TopologyEdge>,
}

/// Topology node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyNode {
    /// Node ID
    pub id: String,
    
    /// Node type (device or primal)
    pub node_type: String,
    
    /// Node name
    pub name: String,
    
    /// Node status
    pub status: String,
}

/// Topology edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyEdge {
    /// Source node ID
    pub from: String,
    
    /// Target node ID
    pub to: String,
    
    /// Edge type (assigned, connected, etc.)
    pub edge_type: String,
}

impl UIState {
    /// Create a new empty UI state
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a device
    pub fn add_device(&mut self, device: Device) {
        self.devices.insert(device.id.clone(), device);
    }
    
    /// Add a primal
    pub fn add_primal(&mut self, primal: PrimalInfo) {
        self.primals.insert(primal.id.clone(), primal);
    }
    
    /// Add an assignment
    pub fn add_assignment(&mut self, assignment: Assignment) {
        self.assignments.insert(assignment.device_id.clone(), assignment);
    }
    
    /// Add a log entry
    pub fn add_log(&mut self, entry: LogEntry) {
        self.logs.push_back(entry);
        
        // Keep only last 1000 entries
        if self.logs.len() > 1000 {
            self.logs.pop_front();
        }
    }
    
    /// Get device by ID
    pub fn get_device(&self, id: &str) -> Option<&Device> {
        self.devices.get(id)
    }
    
    /// Get primal by ID
    pub fn get_primal(&self, id: &str) -> Option<&PrimalInfo> {
        self.primals.get(id)
    }
    
    /// Get assignment for device
    pub fn get_assignment(&self, device_id: &str) -> Option<&Assignment> {
        self.assignments.get(device_id)
    }
}

