//! Core types and data structures for universal orchestration
//!
//! This module contains all the core data structures, enums, and types
//! used in the universal orchestration system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type alias for workload identifier
pub type WorkloadId = String;

/// Type alias for service identifier
pub type ServiceId = String;

/// Orchestrator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorInfo {
    pub name: String,
    pub orchestrator_type: OrchestratorType,
    pub version: String,
    pub cluster_name: String,
    pub capabilities: Vec<OrchestratorCapability>,
    pub sovereignty_compliance: OrchestrationSovereignty,
}

/// Orchestrator types - vendor agnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestratorType {
    /// Kubernetes
    Kubernetes {
        distribution: KubernetesDistribution,
    },
    /// HashiCorp Nomad
    Nomad {
        enterprise: bool,
    },
    /// Docker Swarm
    DockerSwarm,
    /// Apache Mesos
    Mesos {
        framework: String,
    },
    /// Red Hat OpenShift
    OpenShift {
        version: String,
    },
    /// Rancher
    Rancher {
        k8s_version: String,
    },
    /// Amazon EKS
    Eks {
        aws_region: String,
    },
    /// Google GKE
    Gke {
        gcp_region: String,
    },
    /// Microsoft AKS
    Aks {
        azure_region: String,
    },
    /// DigitalOcean Kubernetes
    DigitalOceanK8s,
    /// Lightweight orchestrators
    K3s,
    K0s,
    MicroK8s,
    /// Service mesh orchestrators
    Istio {
        version: String,
    },
    Linkerd {
        version: String,
    },
    Consul {
        connect_enabled: bool,
    },
    /// Edge orchestrators
    KubeEdge,
    OpenYurt,
    SuperEdge,
    /// Container-native orchestrators
    Podman {
        pods_enabled: bool,
    },
    Containerd {
        cri_enabled: bool,
    },
    /// Serverless orchestrators
    Knative {
        serving_version: String,
        eventing_version: String,
    },
    OpenFaas,
    Fission,
    /// Custom orchestrator
    Custom {
        name: String,
        api_version: String,
    },
    /// No orchestrator (direct deployment)
    None,
}

/// Kubernetes distributions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KubernetesDistribution {
    Vanilla,
    OpenShift,
    Rancher,
    Tanzu,
    Anthos,
    Eks,
    Gke,
    Aks,
    DigitalOcean,
    Linode,
    OvhCloud,
    Scaleway,
    Custom { name: String },
}

/// Orchestrator capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestratorCapability {
    /// Container orchestration
    ContainerOrchestration { runtimes: Vec<String> },
    /// Service mesh
    ServiceMesh { protocols: Vec<String> },
    /// Auto-scaling
    AutoScaling { metrics: Vec<String> },
    /// Load balancing
    LoadBalancing { algorithms: Vec<String> },
    /// Storage orchestration
    StorageOrchestration { types: Vec<String> },
    /// Network policies
    NetworkPolicies { engines: Vec<String> },
    /// Security policies
    SecurityPolicies { frameworks: Vec<String> },
    /// Multi-cluster
    MultiCluster { federation_types: Vec<String> },
    /// Edge computing
    EdgeComputing { edge_nodes: u32 },
    /// Serverless functions
    Serverless { runtimes: Vec<String> },
    /// Batch processing
    BatchProcessing { schedulers: Vec<String> },
    /// ML/AI workloads
    MachineLearning { frameworks: Vec<String> },
    /// Custom capability
    Custom {
        name: String,
        parameters: HashMap<String, String>,
    },
}

/// Orchestration sovereignty compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestrationSovereignty {
    /// Fully sovereign - no external dependencies
    FullSovereignty,
    /// Partial sovereignty - some cloud provider integration
    PartialSovereignty { cloud_dependencies: Vec<String> },
    /// Cloud managed - vendor controlled
    CloudManaged {
        provider: String,
        control_plane_location: String,
    },
}

/// Workload types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkloadType {
    /// Single container
    Container,
    /// Multi-container pod
    Pod,
    /// Deployment with replicas
    Deployment,
    /// StatefulSet
    StatefulSet,
    /// DaemonSet
    DaemonSet,
    /// Job
    Job,
    /// CronJob
    CronJob,
    /// Serverless function
    Function { trigger: FunctionTrigger },
    /// Custom resource
    Custom { api_version: String, kind: String },
}

/// Function trigger types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FunctionTrigger {
    Http,
    Timer,
    Queue,
    Event,
    Custom { trigger_type: String },
}

/// Port protocol types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortProtocol {
    TCP,
    UDP,
    SCTP,
}

/// Probe types for health checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProbeType {
    Http,
    Tcp,
    Exec,
    Grpc,
}

/// Match operators for node selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchOperator {
    In,
    NotIn,
    Exists,
    DoesNotExist,
    Gt,
    Lt,
}

/// Toleration operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TolerationOperator {
    Exists,
    Equal,
}

/// Taint effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaintEffect {
    NoSchedule,
    PreferNoSchedule,
    NoExecute,
}

/// DNS policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DnsPolicy {
    ClusterFirst,
    ClusterFirstWithHostNet,
    Default,
    None,
}

/// Volume source types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeSource {
    EmptyDir,
    HostPath { path: String },
    ConfigMap { name: String },
    Secret { name: String },
    PersistentVolumeClaim { claim_name: String },
    Nfs { server: String, path: String },
    Csi { driver: String, volume_handle: String },
    Custom { source_type: String, parameters: HashMap<String, String> },
}

/// Access modes for volumes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessMode {
    ReadWriteOnce,
    ReadOnlyMany,
    ReadWriteMany,
    ReadWriteOncePod,
}

/// Security policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityPolicy {
    Privileged,
    Restricted,
    Custom { policy_name: String },
}

/// Health check types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthCheckType {
    Readiness,
    Liveness,
    Startup,
}

/// Workload phases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkloadPhase {
    Pending,
    Running,
    Succeeded,
    Failed,
    Unknown,
}

/// Condition status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionStatus {
    True,
    False,
    Unknown,
}

/// Service types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceType {
    ClusterIP,
    NodePort,
    LoadBalancer,
    ExternalName,
    Headless,
}

/// Service port target types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServicePortTarget {
    Port(u32),
    Name(String),
}

/// External traffic policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExternalTrafficPolicy {
    Cluster,
    Local,
}

/// Session affinity types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionAffinity {
    None,
    ClientIP,
}

/// Node types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Master,
    Worker,
    Edge,
    Virtual,
}

/// Node status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Ready,
    NotReady,
    Unknown,
    SchedulingDisabled,
}

/// Event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Normal,
    Warning,
    Error,
}

/// Event severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Log levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Default for WorkloadType {
    fn default() -> Self {
        Self::Container
    }
}

impl Default for PortProtocol {
    fn default() -> Self {
        Self::TCP
    }
}

impl Default for ProbeType {
    fn default() -> Self {
        Self::Http
    }
}

impl Default for MatchOperator {
    fn default() -> Self {
        Self::In
    }
}

impl Default for TolerationOperator {
    fn default() -> Self {
        Self::Equal
    }
}

impl Default for TaintEffect {
    fn default() -> Self {
        Self::NoSchedule
    }
}

impl Default for DnsPolicy {
    fn default() -> Self {
        Self::ClusterFirst
    }
}

impl Default for AccessMode {
    fn default() -> Self {
        Self::ReadWriteOnce
    }
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self::Restricted
    }
}

impl Default for HealthCheckType {
    fn default() -> Self {
        Self::Readiness
    }
}

impl Default for WorkloadPhase {
    fn default() -> Self {
        Self::Pending
    }
}

impl Default for ConditionStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

impl Default for ServiceType {
    fn default() -> Self {
        Self::ClusterIP
    }
}

impl Default for ExternalTrafficPolicy {
    fn default() -> Self {
        Self::Cluster
    }
}

impl Default for SessionAffinity {
    fn default() -> Self {
        Self::None
    }
}

impl Default for NodeType {
    fn default() -> Self {
        Self::Worker
    }
}

impl Default for NodeStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

impl Default for EventType {
    fn default() -> Self {
        Self::Normal
    }
}

impl Default for EventSeverity {
    fn default() -> Self {
        Self::Info
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
} 