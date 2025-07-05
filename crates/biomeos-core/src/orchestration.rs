use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use crate::BiomeResult;

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
    async fn cluster_events(&self, since: Option<chrono::DateTime<chrono::Utc>>) -> BiomeResult<Vec<ClusterEvent>>;
    
    /// Execute command in workload
    async fn exec_workload(&self, id: &WorkloadId, command: &[String]) -> BiomeResult<ExecResult>;
    
    /// Get workload logs
    async fn workload_logs(&self, id: &WorkloadId, options: &LogOptions) -> BiomeResult<Vec<LogEntry>>;
}

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
    Kubernetes { distribution: KubernetesDistribution },
    /// HashiCorp Nomad
    Nomad { enterprise: bool },
    /// Docker Swarm
    DockerSwarm,
    /// Apache Mesos
    Mesos { framework: String },
    /// Red Hat OpenShift
    OpenShift { version: String },
    /// Rancher
    Rancher { k8s_version: String },
    /// Amazon EKS
    Eks { aws_region: String },
    /// Google GKE
    Gke { gcp_region: String },
    /// Microsoft AKS
    Aks { azure_region: String },
    /// DigitalOcean Kubernetes
    DigitalOceanK8s,
    /// Lightweight orchestrators
    K3s,
    K0s,
    MicroK8s,
    /// Service mesh orchestrators
    Istio { version: String },
    Linkerd { version: String },
    Consul { connect_enabled: bool },
    /// Edge orchestrators
    KubeEdge,
    OpenYurt,
    SuperEdge,
    /// Container-native orchestrators
    Podman { pods_enabled: bool },
    Containerd { cri_enabled: bool },
    /// Serverless orchestrators
    Knative { serving_version: String, eventing_version: String },
    OpenFaas,
    Fission,
    /// Custom orchestrator
    Custom { name: String, api_version: String },
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
    Custom { name: String, parameters: HashMap<String, String> },
}

/// Orchestration sovereignty compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestrationSovereignty {
    /// Fully sovereign - no external dependencies
    FullSovereignty,
    /// Partial sovereignty - some cloud provider integration
    PartialSovereignty { cloud_dependencies: Vec<String> },
    /// Cloud managed - vendor controlled
    CloudManaged { provider: String, control_plane_location: String },
}

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

/// Workload types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkloadType {
    /// Long-running services
    Deployment,
    /// Stateful services
    StatefulSet,
    /// Background jobs
    Job,
    /// Scheduled jobs
    CronJob { schedule: String },
    /// Daemon sets
    DaemonSet,
    /// Serverless functions
    Function { trigger: FunctionTrigger },
    /// Batch processing
    Batch { parallelism: u32 },
    /// Custom workload type
    Custom { type_name: String, parameters: HashMap<String, String> },
}

/// Function triggers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FunctionTrigger {
    Http { path: String, methods: Vec<String> },
    Event { source: String, event_type: String },
    Schedule { cron: String },
    Queue { queue_name: String },
    Custom { trigger_type: String, config: HashMap<String, String> },
}

/// Container specification (reusing from universal.rs)
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

/// Port protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortProtocol {
    Tcp,
    Udp,
    Sctp,
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

/// Probe types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProbeType {
    Http { path: String, port: u16, headers: HashMap<String, String> },
    Tcp { port: u16 },
    Exec { command: Vec<String> },
    Grpc { port: u16, service: Option<String> },
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

/// Node affinity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAffinitySpec {
    pub required: Vec<NodeSelectorTerm>,
    pub preferred: Vec<PreferredSchedulingTerm>,
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

/// Match operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchOperator {
    In,
    NotIn,
    Exists,
    DoesNotExist,
    Gt,
    Lt,
}

/// Preferred scheduling term
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferredSchedulingTerm {
    pub weight: u32,
    pub preference: NodeSelectorTerm,
}

/// Pod affinity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodAffinitySpec {
    pub required: Vec<PodAffinityTerm>,
    pub preferred: Vec<WeightedPodAffinityTerm>,
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
    pub weight: u32,
    pub pod_affinity_term: PodAffinityTerm,
}

/// Toleration specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TolerationSpec {
    pub key: String,
    pub operator: TolerationOperator,
    pub value: Option<String>,
    pub effect: TaintEffect,
    pub toleration_seconds: Option<u64>,
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

/// Networking specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkingSpec {
    pub dns_policy: DnsPolicy,
    pub hostname: Option<String>,
    pub subdomain: Option<String>,
    pub host_network: bool,
    pub dns_config: Option<DnsConfig>,
    pub service_account: Option<String>,
}

/// DNS policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DnsPolicy {
    Default,
    ClusterFirst,
    ClusterFirstWithHostNet,
    None,
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
    pub volume_source: VolumeSource,
}

/// Volume sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeSource {
    EmptyDir { size_limit: Option<String> },
    HostPath { path: String },
    PersistentVolumeClaim { claim_name: String },
    ConfigMap { name: String },
    Secret { name: String },
    Nfs { server: String, path: String },
    Ceph { monitors: Vec<String>, secret_ref: String },
    Custom { volume_type: String, parameters: HashMap<String, String> },
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
    pub size: String,
    pub storage_class: Option<String>,
    pub selector: Option<LabelSelector>,
}

/// Access modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessMode {
    ReadWriteOnce,
    ReadOnlyMany,
    ReadWriteMany,
    ReadWriteOncePod,
}

/// Security specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySpec {
    pub pod_security_context: PodSecurityContext,
    pub service_account_name: Option<String>,
    pub image_pull_secrets: Vec<String>,
    pub security_policies: Vec<SecurityPolicy>,
}

/// Pod security context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodSecurityContext {
    pub run_as_user: Option<u32>,
    pub run_as_group: Option<u32>,
    pub run_as_non_root: bool,
    pub fs_group: Option<u32>,
    pub supplemental_groups: Vec<u32>,
    pub seccomp_profile: Option<String>,
}

/// Security policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityPolicy {
    PodSecurityPolicy { name: String },
    NetworkPolicy { name: String },
    ServiceMeshPolicy { name: String },
    Custom { policy_type: String, name: String },
}

/// Health check specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckSpec {
    pub name: String,
    pub check_type: HealthCheckType,
    pub interval_seconds: u32,
    pub timeout_seconds: u32,
    pub retries: u32,
}

/// Health check types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthCheckType {
    Http { url: String, expected_status: u16 },
    Tcp { host: String, port: u16 },
    Command { command: Vec<String> },
    Custom { checker: String, config: HashMap<String, String> },
}

// Type aliases and additional types
pub type WorkloadId = String;
pub type ServiceId = String;

/// Workload status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadStatus {
    pub workload_id: WorkloadId,
    pub phase: WorkloadPhase,
    pub replicas: ReplicaStatus,
    pub conditions: Vec<WorkloadCondition>,
    pub resource_usage: WorkloadResourceUsage,
    pub last_updated: chrono::DateTime<chrono::Utc>,
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

/// Replica status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaStatus {
    pub desired: u32,
    pub current: u32,
    pub ready: u32,
    pub available: u32,
    pub unavailable: u32,
}

/// Workload condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadCondition {
    pub condition_type: String,
    pub status: ConditionStatus,
    pub reason: Option<String>,
    pub message: Option<String>,
    pub last_transition_time: chrono::DateTime<chrono::Utc>,
}

/// Condition status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionStatus {
    True,
    False,
    Unknown,
}

/// Workload resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadResourceUsage {
    pub cpu_cores_used: f64,
    pub memory_mb_used: u64,
    pub storage_gb_used: u64,
    pub network_mbps_used: f64,
    pub cost_per_hour: Option<f64>,
}

/// Workload information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadInfo {
    pub id: WorkloadId,
    pub name: String,
    pub workload_type: WorkloadType,
    pub status: WorkloadStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Service specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpec {
    pub name: String,
    pub service_type: ServiceType,
    pub selector: HashMap<String, String>,
    pub ports: Vec<ServicePort>,
    pub load_balancer_config: Option<LoadBalancerConfig>,
    pub session_affinity: SessionAffinity,
}

/// Service types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceType {
    ClusterIP,
    NodePort,
    LoadBalancer,
    ExternalName { external_name: String },
    Headless,
}

/// Service port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePort {
    pub name: Option<String>,
    pub protocol: PortProtocol,
    pub port: u16,
    pub target_port: ServicePortTarget,
    pub node_port: Option<u16>,
}

/// Service port target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServicePortTarget {
    Port(u16),
    Name(String),
}

/// Load balancer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    pub load_balancer_ip: Option<String>,
    pub load_balancer_source_ranges: Vec<String>,
    pub external_traffic_policy: ExternalTrafficPolicy,
    pub health_check_node_port: Option<u16>,
}

/// External traffic policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExternalTrafficPolicy {
    Cluster,
    Local,
}

/// Session affinity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionAffinity {
    None,
    ClientIP { timeout_seconds: u32 },
}

/// Service endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub service_name: String,
    pub namespace: Option<String>,
    pub endpoints: Vec<EndpointAddress>,
    pub ports: Vec<EndpointPort>,
    pub service_type: ServiceType,
}

/// Endpoint address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointAddress {
    pub ip: String,
    pub hostname: Option<String>,
    pub node_name: Option<String>,
    pub ready: bool,
}

/// Endpoint port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointPort {
    pub name: Option<String>,
    pub port: u16,
    pub protocol: PortProtocol,
}

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

/// Node types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    Master,
    Worker,
    Edge,
    Virtual,
    Custom { node_type: String },
}

/// Node status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeStatus {
    Ready,
    NotReady,
    Unknown,
    SchedulingDisabled,
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

/// Event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Normal,
    Warning,
    Error,
}

/// Event severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventSeverity {
    Info,
    Warning,
    Error,
    Critical,
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

/// Log levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

/// Log source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSource {
    pub workload_id: WorkloadId,
    pub container_name: String,
    pub pod_name: Option<String>,
    pub node_name: Option<String>,
}

/// Universal Orchestration Manager
pub struct UniversalOrchestrationManager {
    pub orchestrators: HashMap<String, Box<dyn UniversalOrchestrationInterface>>,
    pub default_orchestrator: Option<String>,
    pub orchestrator_preference: Vec<String>,
    pub sovereignty_requirements: OrchestrationSovereigntyRequirements,
}

/// Orchestration sovereignty requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationSovereigntyRequirements {
    pub require_sovereign_orchestrator: bool,
    pub allow_cloud_managed_orchestrator: bool,
    pub crypto_lock_external_apis: bool,
    pub prefer_self_hosted: bool,
    pub require_air_gapped_deployment: bool,
}

impl UniversalOrchestrationManager {
    /// Create new orchestration manager with sovereignty-first defaults
    pub fn new() -> Self {
        Self {
            orchestrators: HashMap::new(),
            default_orchestrator: None,
            orchestrator_preference: vec![
                "none".to_string(),          // Direct deployment (most sovereign)
                "podman".to_string(),        // Container-native, no orchestrator
                "k3s".to_string(),           // Lightweight K8s
                "nomad".to_string(),         // Simple, sovereign-friendly
                "kubernetes".to_string(),    // Full-featured
            ],
            sovereignty_requirements: OrchestrationSovereigntyRequirements {
                require_sovereign_orchestrator: false,
                allow_cloud_managed_orchestrator: true,
                crypto_lock_external_apis: true,
                prefer_self_hosted: true,
                require_air_gapped_deployment: false,
            },
        }
    }
    
    /// Add orchestrator
    pub fn add_orchestrator(&mut self, name: String, orchestrator: Box<dyn UniversalOrchestrationInterface>) {
        self.orchestrators.insert(name, orchestrator);
    }
    
    /// Get best available orchestrator based on sovereignty requirements
    pub async fn get_best_orchestrator(&self) -> Option<&Box<dyn UniversalOrchestrationInterface>> {
        for orchestrator_name in &self.orchestrator_preference {
            if let Some(orchestrator) = self.orchestrators.get(orchestrator_name) {
                // Check sovereignty compliance
                if let Ok(info) = orchestrator.orchestrator_info().await {
                    if self.meets_sovereignty_requirements(&info.sovereignty_compliance) {
                        return Some(orchestrator);
                    }
                }
            }
        }
        None
    }
    
    fn meets_sovereignty_requirements(&self, compliance: &OrchestrationSovereignty) -> bool {
        match compliance {
            OrchestrationSovereignty::FullSovereignty => true,
            OrchestrationSovereignty::PartialSovereignty { .. } => !self.sovereignty_requirements.require_sovereign_orchestrator,
            OrchestrationSovereignty::CloudManaged { .. } => self.sovereignty_requirements.allow_cloud_managed_orchestrator,
        }
    }
} 