// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
        self.assignments
            .insert(assignment.device_id.clone(), assignment);
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_ui_state_creation() {
        let state = UIState::new();
        assert!(state.devices.is_empty());
        assert!(state.primals.is_empty());
        assert!(state.assignments.is_empty());
        assert!(state.logs.is_empty());
        assert!(state.topology.nodes.is_empty());
        assert!(state.topology.edges.is_empty());
    }

    #[test]
    fn test_add_device() {
        let mut state = UIState::new();
        let device = Device {
            id: "gpu0".to_string(),
            device_type: "gpu".to_string(),
            name: "NVIDIA RTX 4090".to_string(),
            capabilities: vec!["compute".to_string(), "ml".to_string()],
            resources: HashMap::new(),
            status: DeviceStatus::Available,
        };

        state.add_device(device.clone());
        assert_eq!(state.devices.len(), 1);
        assert_eq!(state.get_device("gpu0").unwrap().id, "gpu0");
        assert_eq!(state.get_device("gpu0").unwrap().device_type, "gpu");
    }

    #[test]
    fn test_add_primal() {
        let mut state = UIState::new();
        let primal = PrimalInfo {
            id: "beardog-1".to_string(),
            name: "beardog".to_string(),
            capabilities: vec!["security".to_string(), "crypto".to_string()],
            status: PrimalStatus::Running,
            health: HealthMetrics {
                status: "healthy".to_string(),
                uptime: 3600,
                cpu_usage: 15.5,
                memory_usage: 256,
            },
        };

        state.add_primal(primal.clone());
        assert_eq!(state.primals.len(), 1);
        assert_eq!(state.get_primal("beardog-1").unwrap().name, "beardog");
        assert_eq!(
            state.get_primal("beardog-1").unwrap().status,
            PrimalStatus::Running
        );
    }

    #[test]
    fn test_add_assignment() {
        let mut state = UIState::new();
        let assignment = Assignment {
            device_id: "gpu0".to_string(),
            primal_id: "toadstool-1".to_string(),
            assigned_at: Utc::now(),
            status: AssignmentStatus::Active,
        };

        state.add_assignment(assignment.clone());
        assert_eq!(state.assignments.len(), 1);
        let retrieved = state.get_assignment("gpu0").unwrap();
        assert_eq!(retrieved.primal_id, "toadstool-1");
        assert_eq!(retrieved.status, AssignmentStatus::Active);
    }

    #[test]
    fn test_add_log_entry() {
        let mut state = UIState::new();
        let log = LogEntry {
            timestamp: Utc::now(),
            source: "beardog".to_string(),
            level: LogLevel::Info,
            message: "Service started".to_string(),
        };

        state.add_log(log.clone());
        assert_eq!(state.logs.len(), 1);
        assert_eq!(state.logs.front().unwrap().source, "beardog");
        assert_eq!(state.logs.front().unwrap().level, LogLevel::Info);
    }

    #[test]
    fn test_log_rotation() {
        let mut state = UIState::new();

        // Add 1100 log entries (exceeds 1000 limit)
        for i in 0..1100 {
            state.add_log(LogEntry {
                timestamp: Utc::now(),
                source: "test".to_string(),
                level: LogLevel::Info,
                message: format!("Log entry {i}"),
            });
        }

        // Should keep only last 1000
        assert_eq!(state.logs.len(), 1000);
        // First entry should be entry #100 (0-99 dropped)
        assert_eq!(state.logs.front().unwrap().message, "Log entry 100");
        // Last entry should be entry #1099
        assert_eq!(state.logs.back().unwrap().message, "Log entry 1099");
    }

    #[test]
    fn test_get_nonexistent_device() {
        let state = UIState::new();
        assert!(state.get_device("nonexistent").is_none());
    }

    #[test]
    fn test_get_nonexistent_primal() {
        let state = UIState::new();
        assert!(state.get_primal("nonexistent").is_none());
    }

    #[test]
    fn test_get_nonexistent_assignment() {
        let state = UIState::new();
        assert!(state.get_assignment("nonexistent").is_none());
    }

    #[test]
    fn test_device_status_values() {
        assert_eq!(DeviceStatus::Available, DeviceStatus::Available);
        assert_ne!(DeviceStatus::Available, DeviceStatus::Assigned);
        assert_ne!(DeviceStatus::Assigned, DeviceStatus::Offline);
        assert_ne!(DeviceStatus::Offline, DeviceStatus::Error);
    }

    #[test]
    fn test_primal_status_values() {
        assert_eq!(PrimalStatus::Running, PrimalStatus::Running);
        assert_ne!(PrimalStatus::Running, PrimalStatus::Starting);
        assert_ne!(PrimalStatus::Starting, PrimalStatus::Stopping);
        assert_ne!(PrimalStatus::Stopping, PrimalStatus::Stopped);
        assert_ne!(PrimalStatus::Stopped, PrimalStatus::Error);
    }

    #[test]
    fn test_assignment_status_values() {
        assert_eq!(AssignmentStatus::Pending, AssignmentStatus::Pending);
        assert_ne!(AssignmentStatus::Pending, AssignmentStatus::Active);
        assert_ne!(AssignmentStatus::Active, AssignmentStatus::Failed);
    }

    #[test]
    fn test_log_level_values() {
        assert_eq!(LogLevel::Info, LogLevel::Info);
        assert_ne!(LogLevel::Info, LogLevel::Warning);
        assert_ne!(LogLevel::Warning, LogLevel::Error);
    }

    #[test]
    fn test_topology_default() {
        let topology = Topology::default();
        assert!(topology.nodes.is_empty());
        assert!(topology.edges.is_empty());
    }

    #[test]
    fn test_device_serialization() {
        let device = Device {
            id: "test-device".to_string(),
            device_type: "gpu".to_string(),
            name: "Test GPU".to_string(),
            capabilities: vec!["compute".to_string()],
            resources: HashMap::new(),
            status: DeviceStatus::Available,
        };

        // Serialize
        let json = serde_json::to_string(&device).expect("Should serialize");
        assert!(json.contains("test-device"));
        assert!(json.contains("gpu"));

        // Deserialize
        let deserialized: Device = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.id, "test-device");
        assert_eq!(deserialized.device_type, "gpu");
    }

    #[test]
    fn test_primal_info_serialization() {
        let primal = PrimalInfo {
            id: "test-primal".to_string(),
            name: "beardog".to_string(),
            capabilities: vec!["security".to_string()],
            status: PrimalStatus::Running,
            health: HealthMetrics {
                status: "healthy".to_string(),
                uptime: 3600,
                cpu_usage: 10.0,
                memory_usage: 512,
            },
        };

        let json = serde_json::to_string(&primal).expect("Should serialize");
        assert!(json.contains("test-primal"));
        assert!(json.contains("beardog"));

        let deserialized: PrimalInfo = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.id, "test-primal");
        assert_eq!(deserialized.name, "beardog");
        assert_eq!(deserialized.health.uptime, 3600);
    }

    #[test]
    fn test_multiple_devices() {
        let mut state = UIState::new();

        for i in 0..10 {
            state.add_device(Device {
                id: format!("device-{i}"),
                device_type: "gpu".to_string(),
                name: format!("GPU {i}"),
                capabilities: vec![],
                resources: HashMap::new(),
                status: DeviceStatus::Available,
            });
        }

        assert_eq!(state.devices.len(), 10);
        assert!(state.get_device("device-0").is_some());
        assert!(state.get_device("device-9").is_some());
        assert!(state.get_device("device-10").is_none());
    }

    #[test]
    fn test_health_metrics_values() {
        let health = HealthMetrics {
            status: "healthy".to_string(),
            uptime: 7200,
            cpu_usage: 25.5,
            memory_usage: 1024,
        };

        assert_eq!(health.status, "healthy");
        assert_eq!(health.uptime, 7200);
        assert_eq!(health.cpu_usage, 25.5);
        assert_eq!(health.memory_usage, 1024);
    }

    #[test]
    fn test_ui_state_default() {
        let state = UIState::default();
        assert!(state.devices.is_empty());
        assert!(state.primals.is_empty());
        assert!(state.assignments.is_empty());
        assert!(state.logs.is_empty());
        assert!(state.topology.nodes.is_empty());
    }

    #[test]
    fn test_assignment_serde_roundtrip() {
        let assignment = Assignment {
            device_id: "gpu0".to_string(),
            primal_id: "beardog-1".to_string(),
            assigned_at: Utc::now(),
            status: AssignmentStatus::Active,
        };
        let json = serde_json::to_string(&assignment).unwrap();
        let parsed: Assignment = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.device_id, assignment.device_id);
        assert_eq!(parsed.status, assignment.status);
    }

    #[test]
    fn test_log_entry_serde_roundtrip() {
        let entry = LogEntry {
            timestamp: Utc::now(),
            source: "beardog".to_string(),
            level: LogLevel::Warning,
            message: "test".to_string(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let parsed: LogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.source, entry.source);
        assert_eq!(parsed.level, entry.level);
    }

    #[test]
    fn test_topology_node_serde_roundtrip() {
        let node = TopologyNode {
            id: "n1".to_string(),
            node_type: "device".to_string(),
            name: "GPU 0".to_string(),
            status: "available".to_string(),
        };
        let json = serde_json::to_string(&node).unwrap();
        let parsed: TopologyNode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, node.id);
    }

    #[test]
    fn test_topology_edge_serde_roundtrip() {
        let edge = TopologyEdge {
            from: "gpu0".to_string(),
            to: "beardog-1".to_string(),
            edge_type: "assigned".to_string(),
        };
        let json = serde_json::to_string(&edge).unwrap();
        let parsed: TopologyEdge = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.from, edge.from);
    }

    #[test]
    fn test_enum_variants_serde() {
        let statuses = [
            DeviceStatus::Available,
            DeviceStatus::Assigned,
            DeviceStatus::Offline,
            DeviceStatus::Error,
        ];
        for s in statuses {
            let json = serde_json::to_string(&s).unwrap();
            let parsed: DeviceStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(s, parsed);
        }
    }
}
