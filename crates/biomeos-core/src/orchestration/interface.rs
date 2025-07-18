//! Universal orchestration interface
//!
//! This module contains the main trait definition for universal orchestration
//! that eliminates vendor lock-in across different orchestration platforms.

use async_trait::async_trait;
use crate::BiomeResult;
use super::types::*;
use super::workload::{WorkloadSpec, WorkloadStatus, WorkloadInfo};
use super::service::{ServiceSpec, ServiceEndpoint};
use super::cluster::{ClusterResources, ClusterEvent, ExecResult, LogOptions, LogEntry};

/// Universal Orchestration Interface - eliminates K8s/Nomad/Swarm vendor lock-in
#[async_trait]
pub trait UniversalOrchestrationInterface {
    /// Get orchestrator information
    async fn orchestrator_info(&self) -> BiomeResult<OrchestratorInfo>;

    /// Deploy workload
    async fn deploy_workload(&self, spec: &WorkloadSpec) -> BiomeResult<WorkloadId>;

    /// Update workload
    async fn update_workload(&self, id: &WorkloadId, spec: &WorkloadSpec) -> BiomeResult<()>;

    /// Scale workload
    async fn scale_workload(&self, id: &WorkloadId, replicas: u32) -> BiomeResult<()>;

    /// Stop workload
    async fn stop_workload(&self, id: &WorkloadId) -> BiomeResult<()>;

    /// Delete workload
    async fn delete_workload(&self, id: &WorkloadId) -> BiomeResult<()>;

    /// Get workload status
    async fn workload_status(&self, id: &WorkloadId) -> BiomeResult<WorkloadStatus>;

    /// List all workloads
    async fn list_workloads(&self) -> BiomeResult<Vec<WorkloadInfo>>;

    /// Create service
    async fn create_service(&self, spec: &ServiceSpec) -> BiomeResult<ServiceId>;

    /// Delete service
    async fn delete_service(&self, id: &ServiceId) -> BiomeResult<()>;

    /// Service discovery
    async fn discover_services(&self) -> BiomeResult<Vec<ServiceEndpoint>>;

    /// Get cluster resources
    async fn cluster_resources(&self) -> BiomeResult<ClusterResources>;

    /// Get cluster events
    async fn cluster_events(
        &self,
        since: Option<chrono::DateTime<chrono::Utc>>,
    ) -> BiomeResult<Vec<ClusterEvent>>;

    /// Execute command in workload
    async fn exec_workload(&self, id: &WorkloadId, command: &[String]) -> BiomeResult<ExecResult>;

    /// Get workload logs
    async fn workload_logs(
        &self,
        id: &WorkloadId,
        options: &LogOptions,
    ) -> BiomeResult<Vec<LogEntry>>;
}

/// Optional trait for orchestrators that support advanced features
#[async_trait]
pub trait ExtendedOrchestrationInterface: UniversalOrchestrationInterface {
    /// Get workload metrics
    async fn workload_metrics(&self, id: &WorkloadId) -> BiomeResult<WorkloadMetrics>;

    /// Stream workload logs
    async fn stream_workload_logs(
        &self,
        id: &WorkloadId,
        options: &LogOptions,
    ) -> BiomeResult<LogStream>;

    /// Port forward to workload
    async fn port_forward(
        &self,
        id: &WorkloadId,
        local_port: u16,
        remote_port: u16,
    ) -> BiomeResult<PortForwardSession>;

    /// Create network policy
    async fn create_network_policy(&self, spec: &NetworkPolicySpec) -> BiomeResult<String>;

    /// Apply resource quota
    async fn apply_resource_quota(&self, spec: &ResourceQuotaSpec) -> BiomeResult<String>;

    /// Create persistent volume
    async fn create_persistent_volume(&self, spec: &PersistentVolumeSpec) -> BiomeResult<String>;

    /// Backup workload
    async fn backup_workload(&self, id: &WorkloadId, backup_spec: &BackupSpec) -> BiomeResult<String>;

    /// Restore workload
    async fn restore_workload(&self, backup_id: &str, restore_spec: &RestoreSpec) -> BiomeResult<WorkloadId>;
}

/// Workload metrics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkloadMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub network_rx: u64,
    pub network_tx: u64,
    pub storage_usage: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Log stream handle
#[derive(Debug)]
pub struct LogStream {
    pub stream_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Port forward session
#[derive(Debug)]
pub struct PortForwardSession {
    pub session_id: String,
    pub local_port: u16,
    pub remote_port: u16,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Network policy specification
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkPolicySpec {
    pub name: String,
    pub namespace: String,
    pub pod_selector: std::collections::HashMap<String, String>,
    pub ingress_rules: Vec<NetworkPolicyRule>,
    pub egress_rules: Vec<NetworkPolicyRule>,
}

/// Network policy rule
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkPolicyRule {
    pub from_selectors: Vec<std::collections::HashMap<String, String>>,
    pub to_selectors: Vec<std::collections::HashMap<String, String>>,
    pub ports: Vec<NetworkPolicyPort>,
}

/// Network policy port
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NetworkPolicyPort {
    pub port: u16,
    pub protocol: String,
}

/// Resource quota specification
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceQuotaSpec {
    pub name: String,
    pub namespace: String,
    pub hard_limits: std::collections::HashMap<String, String>,
    pub scopes: Vec<String>,
}

/// Persistent volume specification
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersistentVolumeSpec {
    pub name: String,
    pub capacity: String,
    pub access_modes: Vec<AccessMode>,
    pub volume_source: VolumeSource,
    pub storage_class: Option<String>,
}

/// Backup specification
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackupSpec {
    pub name: String,
    pub include_volumes: bool,
    pub include_cluster_resources: bool,
    pub backup_location: String,
    pub retention_policy: RetentionPolicy,
}

/// Restore specification
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RestoreSpec {
    pub name: String,
    pub namespace_mapping: std::collections::HashMap<String, String>,
    pub include_volumes: bool,
    pub include_cluster_resources: bool,
}

/// Backup retention policy
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetentionPolicy {
    pub daily: u32,
    pub weekly: u32,
    pub monthly: u32,
    pub yearly: u32,
}

impl Default for WorkloadMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            network_rx: 0,
            network_tx: 0,
            storage_usage: 0,
            timestamp: chrono::Utc::now(),
        }
    }
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            daily: 7,
            weekly: 4,
            monthly: 12,
            yearly: 3,
        }
    }
} 