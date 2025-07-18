//! Workload specifications and related types
//!
//! This module contains all workload-related specifications, including
//! container specs, resource requirements, and workload status.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::*;

/// Workload specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadSpec {
    pub name: String,
    pub workload_type: WorkloadType,
    pub containers: Vec<ContainerSpec>,
    pub replicas: u32,
    pub resources: ResourceRequirements,
    pub scheduling: SchedulingSpec,
    pub networking: NetworkingSpec,
    pub storage: StorageSpec,
    pub security: SecuritySpec,
    pub health_checks: Vec<HealthCheckSpec>,
    pub environment: HashMap<String, String>,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
}

/// Container specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSpec {
    pub name: String,
    pub image: String,
    pub command: Option<Vec<String>>,
    pub args: Option<Vec<String>>,
    pub env: HashMap<String, String>,
    pub ports: Vec<ContainerPort>,
    pub volume_mounts: Vec<VolumeMount>,
    pub resources: ContainerResources,
    pub security_context: SecurityContext,
    pub liveness_probe: Option<ProbeSpec>,
    pub readiness_probe: Option<ProbeSpec>,
}

/// Container port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerPort {
    pub name: Option<String>,
    pub container_port: u16,
    pub protocol: PortProtocol,
}

/// Volume mount
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    pub name: String,
    pub mount_path: String,
    pub read_only: bool,
    pub sub_path: Option<String>,
}

/// Container resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerResources {
    pub requests: ResourceList,
    pub limits: ResourceList,
}

/// Resource list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceList {
    pub cpu: Option<String>,
    pub memory: Option<String>,
    pub storage: Option<String>,
    pub gpu: Option<String>,
    pub custom: HashMap<String, String>,
}

/// Security context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub run_as_user: Option<u32>,
    pub run_as_group: Option<u32>,
    pub run_as_non_root: bool,
    pub read_only_root_filesystem: bool,
    pub allow_privilege_escalation: bool,
    pub capabilities: CapabilitySpec,
    pub seccomp_profile: Option<String>,
    pub se_linux_options: HashMap<String, String>,
}

/// Linux capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySpec {
    pub add: Vec<String>,
    pub drop: Vec<String>,
}

/// Probe specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeSpec {
    pub probe_type: ProbeType,
    pub initial_delay_seconds: u32,
    pub period_seconds: u32,
    pub timeout_seconds: u32,
    pub success_threshold: u32,
    pub failure_threshold: u32,
}

/// Resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: Option<f64>,
    pub memory_mb: Option<u64>,
    pub storage_gb: Option<u64>,
    pub gpu_count: Option<u32>,
    pub network_bandwidth_mbps: Option<u64>,
    pub custom_resources: HashMap<String, String>,
}

/// Scheduling specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingSpec {
    pub node_selector: HashMap<String, String>,
    pub affinity: Option<AffinitySpec>,
    pub tolerations: Vec<TolerationSpec>,
    pub priority_class: Option<String>,
    pub scheduler_name: Option<String>,
}

/// Affinity specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffinitySpec {
    pub node_affinity: Option<NodeAffinitySpec>,
    pub pod_affinity: Option<PodAffinitySpec>,
    pub pod_anti_affinity: Option<PodAffinitySpec>,
}

/// Node affinity specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAffinitySpec {
    pub required_terms: Vec<NodeSelectorTerm>,
    pub preferred_terms: Vec<PreferredSchedulingTerm>,
}

/// Node selector term
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSelectorTerm {
    pub match_expressions: Vec<MatchExpression>,
}

/// Match expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchExpression {
    pub key: String,
    pub operator: MatchOperator,
    pub values: Vec<String>,
}

/// Preferred scheduling term
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferredSchedulingTerm {
    pub weight: i32,
    pub preference: NodeSelectorTerm,
}

/// Pod affinity specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodAffinitySpec {
    pub required_terms: Vec<PodAffinityTerm>,
    pub preferred_terms: Vec<WeightedPodAffinityTerm>,
}

/// Pod affinity term
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodAffinityTerm {
    pub label_selector: LabelSelector,
    pub topology_key: String,
}

/// Label selector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelSelector {
    pub match_labels: HashMap<String, String>,
    pub match_expressions: Vec<MatchExpression>,
}

/// Weighted pod affinity term
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedPodAffinityTerm {
    pub weight: i32,
    pub pod_affinity_term: PodAffinityTerm,
}

/// Toleration specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TolerationSpec {
    pub key: Option<String>,
    pub operator: TolerationOperator,
    pub value: Option<String>,
    pub effect: Option<TaintEffect>,
    pub toleration_seconds: Option<u64>,
}

/// Networking specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkingSpec {
    pub hostname: Option<String>,
    pub subdomain: Option<String>,
    pub dns_policy: DnsPolicy,
    pub dns_config: Option<DnsConfig>,
    pub host_network: bool,
    pub host_pid: bool,
    pub host_ipc: bool,
    pub ip_family: Option<String>,
}

/// DNS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfig {
    pub nameservers: Vec<String>,
    pub searches: Vec<String>,
    pub options: Vec<DnsOption>,
}

/// DNS option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsOption {
    pub name: String,
    pub value: Option<String>,
}

/// Storage specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSpec {
    pub volumes: Vec<VolumeSpec>,
    pub volume_claim_templates: Vec<VolumeClaimTemplate>,
}

/// Volume specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSpec {
    pub name: String,
    pub source: VolumeSource,
    pub mount_options: Vec<String>,
    pub read_only: bool,
}

/// Volume claim template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeClaimTemplate {
    pub metadata: ObjectMeta,
    pub spec: VolumeClaimSpec,
}

/// Object metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMeta {
    pub name: String,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
}

/// Volume claim specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeClaimSpec {
    pub access_modes: Vec<AccessMode>,
    pub resources: ResourceRequirements,
    pub storage_class: Option<String>,
    pub selector: Option<LabelSelector>,
}

/// Security specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySpec {
    pub security_policy: SecurityPolicy,
    pub pod_security_context: PodSecurityContext,
    pub image_pull_secrets: Vec<String>,
    pub service_account: Option<String>,
    pub automount_service_account_token: bool,
}

/// Pod security context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodSecurityContext {
    pub run_as_user: Option<u32>,
    pub run_as_group: Option<u32>,
    pub run_as_non_root: bool,
    pub supplemental_groups: Vec<u32>,
    pub fs_group: Option<u32>,
    pub seccomp_profile: Option<String>,
    pub se_linux_options: HashMap<String, String>,
}

/// Health check specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckSpec {
    pub name: String,
    pub check_type: HealthCheckType,
    pub probe: ProbeSpec,
    pub enabled: bool,
}

/// Workload status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadStatus {
    pub phase: WorkloadPhase,
    pub ready_replicas: u32,
    pub available_replicas: u32,
    pub unavailable_replicas: u32,
    pub updated_replicas: u32,
    pub observed_generation: u64,
    pub conditions: Vec<WorkloadCondition>,
    pub replica_statuses: Vec<ReplicaStatus>,
    pub resource_usage: Option<WorkloadResourceUsage>,
}

/// Replica status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaStatus {
    pub name: String,
    pub ready: bool,
    pub restarts: u32,
    pub phase: WorkloadPhase,
    pub node_name: Option<String>,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Workload condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadCondition {
    pub condition_type: String,
    pub status: ConditionStatus,
    pub reason: Option<String>,
    pub message: Option<String>,
    pub last_transition_time: chrono::DateTime<chrono::Utc>,
    pub last_update_time: chrono::DateTime<chrono::Utc>,
}

/// Workload resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadResourceUsage {
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub network_rx: u64,
    pub network_tx: u64,
    pub storage_usage: u64,
}

/// Workload information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadInfo {
    pub id: WorkloadId,
    pub name: String,
    pub namespace: Option<String>,
    pub workload_type: WorkloadType,
    pub status: WorkloadStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// Default implementations
impl Default for WorkloadSpec {
    fn default() -> Self {
        Self {
            name: "unnamed".to_string(),
            workload_type: WorkloadType::default(),
            containers: vec![],
            replicas: 1,
            resources: ResourceRequirements::default(),
            scheduling: SchedulingSpec::default(),
            networking: NetworkingSpec::default(),
            storage: StorageSpec::default(),
            security: SecuritySpec::default(),
            health_checks: vec![],
            environment: HashMap::new(),
            labels: HashMap::new(),
            annotations: HashMap::new(),
        }
    }
}

impl Default for ContainerSpec {
    fn default() -> Self {
        Self {
            name: "main".to_string(),
            image: "alpine:latest".to_string(),
            command: None,
            args: None,
            env: HashMap::new(),
            ports: vec![],
            volume_mounts: vec![],
            resources: ContainerResources::default(),
            security_context: SecurityContext::default(),
            liveness_probe: None,
            readiness_probe: None,
        }
    }
}

impl Default for ContainerResources {
    fn default() -> Self {
        Self {
            requests: ResourceList::default(),
            limits: ResourceList::default(),
        }
    }
}

impl Default for ResourceList {
    fn default() -> Self {
        Self {
            cpu: Some("100m".to_string()),
            memory: Some("128Mi".to_string()),
            storage: None,
            gpu: None,
            custom: HashMap::new(),
        }
    }
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            run_as_user: None,
            run_as_group: None,
            run_as_non_root: false,
            read_only_root_filesystem: false,
            allow_privilege_escalation: false,
            capabilities: CapabilitySpec::default(),
            seccomp_profile: None,
            se_linux_options: HashMap::new(),
        }
    }
}

impl Default for CapabilitySpec {
    fn default() -> Self {
        Self {
            add: vec![],
            drop: vec!["ALL".to_string()],
        }
    }
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            cpu_cores: Some(0.1),
            memory_mb: Some(128),
            storage_gb: None,
            gpu_count: None,
            network_bandwidth_mbps: None,
            custom_resources: HashMap::new(),
        }
    }
}

impl Default for SchedulingSpec {
    fn default() -> Self {
        Self {
            node_selector: HashMap::new(),
            affinity: None,
            tolerations: vec![],
            priority_class: None,
            scheduler_name: None,
        }
    }
}

impl Default for NetworkingSpec {
    fn default() -> Self {
        Self {
            hostname: None,
            subdomain: None,
            dns_policy: DnsPolicy::default(),
            dns_config: None,
            host_network: false,
            host_pid: false,
            host_ipc: false,
            ip_family: None,
        }
    }
}

impl Default for StorageSpec {
    fn default() -> Self {
        Self {
            volumes: vec![],
            volume_claim_templates: vec![],
        }
    }
}

impl Default for SecuritySpec {
    fn default() -> Self {
        Self {
            security_policy: SecurityPolicy::default(),
            pod_security_context: PodSecurityContext::default(),
            image_pull_secrets: vec![],
            service_account: None,
            automount_service_account_token: true,
        }
    }
}

impl Default for PodSecurityContext {
    fn default() -> Self {
        Self {
            run_as_user: None,
            run_as_group: None,
            run_as_non_root: false,
            supplemental_groups: vec![],
            fs_group: None,
            seccomp_profile: None,
            se_linux_options: HashMap::new(),
        }
    }
} 