//! Cluster resources, events, and logging
//!
//! This module contains all cluster-related specifications, including
//! node information, cluster events, and logging functionality.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::*;
use super::workload::ResourceList;

/// Cluster resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterResources {
    pub nodes: Vec<NodeInfo>,
    pub total_capacity: ResourceList,
    pub total_allocatable: ResourceList,
    pub total_allocated: ResourceList,
    pub cluster_utilization: ClusterUtilization,
}

/// Node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub name: String,
    pub node_type: NodeType,
    pub status: NodeStatus,
    pub capacity: ResourceList,
    pub allocatable: ResourceList,
    pub allocated: ResourceList,
    pub conditions: Vec<NodeCondition>,
    pub node_info: NodeSystemInfo,
}

/// Node condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCondition {
    pub condition_type: String,
    pub status: ConditionStatus,
    pub reason: Option<String>,
    pub message: Option<String>,
    pub last_heartbeat_time: chrono::DateTime<chrono::Utc>,
}

/// Node system information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSystemInfo {
    pub machine_id: String,
    pub system_uuid: String,
    pub boot_id: String,
    pub kernel_version: String,
    pub os_image: String,
    pub container_runtime_version: String,
    pub kubelet_version: Option<String>,
    pub kube_proxy_version: Option<String>,
    pub operating_system: String,
    pub architecture: String,
}

/// Cluster utilization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterUtilization {
    pub cpu_utilization_percent: f64,
    pub memory_utilization_percent: f64,
    pub storage_utilization_percent: f64,
    pub network_utilization_percent: f64,
    pub pod_utilization_percent: f64,
}

/// Cluster event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterEvent {
    pub event_id: String,
    pub event_type: EventType,
    pub severity: EventSeverity,
    pub object: EventObject,
    pub reason: String,
    pub message: String,
    pub first_timestamp: chrono::DateTime<chrono::Utc>,
    pub last_timestamp: chrono::DateTime<chrono::Utc>,
    pub count: u32,
}

/// Event object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventObject {
    pub kind: String,
    pub name: String,
    pub namespace: Option<String>,
    pub uid: String,
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
}

/// Log options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogOptions {
    pub follow: bool,
    pub tail_lines: Option<u32>,
    pub since_time: Option<chrono::DateTime<chrono::Utc>>,
    pub since_seconds: Option<u64>,
    pub timestamps: bool,
    pub container: Option<String>,
}

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub message: String,
    pub source: LogSource,
    pub metadata: HashMap<String, String>,
}

/// Log source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSource {
    pub workload_id: WorkloadId,
    pub container_name: String,
    pub pod_name: Option<String>,
    pub node_name: Option<String>,
}

/// Cluster metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterMetrics {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub node_count: u32,
    pub running_pods: u32,
    pub pending_pods: u32,
    pub failed_pods: u32,
    pub cpu_usage: ClusterResourceUsage,
    pub memory_usage: ClusterResourceUsage,
    pub storage_usage: ClusterResourceUsage,
    pub network_usage: ClusterNetworkUsage,
}

/// Cluster resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterResourceUsage {
    pub capacity: f64,
    pub allocatable: f64,
    pub used: f64,
    pub available: f64,
    pub utilization_percent: f64,
}

/// Cluster network usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNetworkUsage {
    pub bytes_in: u64,
    pub bytes_out: u64,
    pub packets_in: u64,
    pub packets_out: u64,
    pub errors_in: u64,
    pub errors_out: u64,
}

/// Node maintenance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMaintenance {
    pub node_name: String,
    pub maintenance_type: MaintenanceType,
    pub scheduled_start: chrono::DateTime<chrono::Utc>,
    pub estimated_duration: chrono::Duration,
    pub drain_strategy: DrainStrategy,
    pub reason: String,
    pub status: MaintenanceStatus,
}

/// Maintenance types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaintenanceType {
    Planned,
    Emergency,
    Upgrade,
    Patch,
    Hardware,
    Custom { maintenance_type: String },
}

/// Drain strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrainStrategy {
    pub force: bool,
    pub delete_local_data: bool,
    pub ignore_daemon_sets: bool,
    pub timeout: chrono::Duration,
    pub grace_period: chrono::Duration,
}

/// Maintenance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaintenanceStatus {
    Scheduled,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Cluster health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterHealthCheck {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub overall_status: HealthStatus,
    pub component_health: Vec<ComponentHealth>,
    pub cluster_score: f64,
    pub recommendations: Vec<String>,
}

/// Health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Component health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub component_name: String,
    pub status: HealthStatus,
    pub message: String,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub metrics: HashMap<String, f64>,
}

/// Cluster scaling recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingRecommendation {
    pub resource_type: String,
    pub current_capacity: f64,
    pub recommended_capacity: f64,
    pub reason: String,
    pub confidence: f64,
    pub estimated_cost_impact: Option<f64>,
}

// Default implementations
impl Default for ClusterResources {
    fn default() -> Self {
        Self {
            nodes: vec![],
            total_capacity: ResourceList::default(),
            total_allocatable: ResourceList::default(),
            total_allocated: ResourceList::default(),
            cluster_utilization: ClusterUtilization::default(),
        }
    }
}

impl Default for ClusterUtilization {
    fn default() -> Self {
        Self {
            cpu_utilization_percent: 0.0,
            memory_utilization_percent: 0.0,
            storage_utilization_percent: 0.0,
            network_utilization_percent: 0.0,
            pod_utilization_percent: 0.0,
        }
    }
}

impl Default for LogOptions {
    fn default() -> Self {
        Self {
            follow: false,
            tail_lines: Some(100),
            since_time: None,
            since_seconds: None,
            timestamps: true,
            container: None,
        }
    }
}

impl Default for DrainStrategy {
    fn default() -> Self {
        Self {
            force: false,
            delete_local_data: false,
            ignore_daemon_sets: true,
            timeout: chrono::Duration::minutes(15),
            grace_period: chrono::Duration::seconds(30),
        }
    }
}

impl Default for ClusterHealthCheck {
    fn default() -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            overall_status: HealthStatus::Unknown,
            component_health: vec![],
            cluster_score: 0.0,
            recommendations: vec![],
        }
    }
}

impl Default for ClusterMetrics {
    fn default() -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            node_count: 0,
            running_pods: 0,
            pending_pods: 0,
            failed_pods: 0,
            cpu_usage: ClusterResourceUsage::default(),
            memory_usage: ClusterResourceUsage::default(),
            storage_usage: ClusterResourceUsage::default(),
            network_usage: ClusterNetworkUsage::default(),
        }
    }
}

impl Default for ClusterResourceUsage {
    fn default() -> Self {
        Self {
            capacity: 0.0,
            allocatable: 0.0,
            used: 0.0,
            available: 0.0,
            utilization_percent: 0.0,
        }
    }
}

impl Default for ClusterNetworkUsage {
    fn default() -> Self {
        Self {
            bytes_in: 0,
            bytes_out: 0,
            packets_in: 0,
            packets_out: 0,
            errors_in: 0,
            errors_out: 0,
        }
    }
} 