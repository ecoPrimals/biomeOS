//! # Universal Biome Manifest System
//!
//! This module provides a universal, primal-agnostic manifest system that replaces
//! the ToadStool-specific manifest. It describes biomes in terms of capabilities
//! and requirements rather than specific Primal implementations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::universal_primal::{
    CapabilityRequirement, Constraint, ResourceRequirements,
};
use crate::{BiomeError, BiomeResult};

/// Universal biome manifest - agnostic to specific Primal implementations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalBiomeManifest {
    /// API version for manifest compatibility
    pub api_version: String,
    /// Manifest type (always "Biome")
    pub kind: String,
    /// Biome metadata
    pub metadata: BiomeMetadata,
    /// Capability requirements for this biome
    pub requirements: BiomeRequirements,
    /// Service definitions
    pub services: Vec<ServiceDefinition>,
    /// Resource specifications
    pub resources: GlobalResourceSpec,
    /// Security requirements
    pub security: SecurityRequirements,
    /// Networking configuration
    pub networking: NetworkingSpec,
    /// Storage requirements
    pub storage: StorageSpec,
    /// Monitoring and observability
    pub monitoring: MonitoringSpec,
    /// Deployment preferences
    pub deployment: DeploymentPreferences,
    /// Validation rules
    pub validation: ValidationSpec,
}

/// Biome metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeMetadata {
    /// Biome name
    pub name: String,
    /// Biome description
    pub description: String,
    /// Version
    pub version: String,
    /// Maintainer information
    pub maintainer: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Labels for organization
    pub labels: HashMap<String, String>,
    /// Annotations for metadata
    pub annotations: HashMap<String, String>,
    /// Creation timestamp
    pub created: Option<DateTime<Utc>>,
    /// Last modified timestamp
    pub modified: Option<DateTime<Utc>>,
}

/// Capability requirements for a biome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeRequirements {
    /// Required capabilities for this biome to function
    pub required: Vec<CapabilityRequirement>,
    /// Optional capabilities that enhance functionality
    pub optional: Vec<CapabilityRequirement>,
    /// Minimum resource requirements
    pub min_resources: ResourceRequirements,
    /// Preferred resource allocations
    pub preferred_resources: Option<ResourceRequirements>,
    /// Maximum resource limits
    pub max_resources: Option<ResourceRequirements>,
    /// Performance requirements
    pub performance: PerformanceRequirements,
    /// Availability requirements
    pub availability: AvailabilityRequirements,
    /// Scaling requirements
    pub scaling: ScalingRequirements,
}

/// Performance requirements for the biome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRequirements {
    /// Maximum acceptable latency
    pub max_latency_ms: Option<u64>,
    /// Minimum required throughput
    pub min_throughput: Option<String>,
    /// Maximum error rate acceptable
    pub max_error_rate: Option<f64>,
    /// Minimum uptime requirement
    pub min_uptime: Option<f64>,
}

/// Availability requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityRequirements {
    /// High availability required
    pub high_availability: bool,
    /// Fault tolerance level
    pub fault_tolerance: FaultToleranceLevel,
    /// Disaster recovery requirements
    pub disaster_recovery: bool,
    /// Backup requirements
    pub backup: BackupRequirements,
}

/// Fault tolerance levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FaultToleranceLevel {
    /// No fault tolerance
    None,
    /// Basic fault tolerance
    Basic,
    /// High fault tolerance
    High,
    /// Mission-critical fault tolerance
    Critical,
}

/// Backup requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRequirements {
    /// Backup required
    pub required: bool,
    /// Backup frequency
    pub frequency: Option<String>,
    /// Retention period
    pub retention: Option<String>,
    /// Backup encryption required
    pub encryption: bool,
}

/// Scaling requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingRequirements {
    /// Auto-scaling enabled
    pub auto_scaling: bool,
    /// Minimum instances
    pub min_instances: u32,
    /// Maximum instances
    pub max_instances: u32,
    /// Scaling triggers
    pub triggers: Vec<ScalingTrigger>,
    /// Scaling policies
    pub policies: Vec<ScalingPolicy>,
}

/// Scaling trigger definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingTrigger {
    /// Trigger type
    pub trigger_type: String,
    /// Metric to monitor
    pub metric: String,
    /// Threshold value
    pub threshold: f64,
    /// Comparison operator
    pub operator: String,
    /// Duration before triggering
    pub duration: Option<String>,
}

/// Scaling policy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingPolicy {
    /// Policy name
    pub name: String,
    /// Scaling direction
    pub direction: ScalingDirection,
    /// Scaling amount
    pub amount: ScalingAmount,
    /// Cooldown period
    pub cooldown: Option<String>,
}

/// Scaling direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingDirection {
    /// Scale up
    Up,
    /// Scale down
    Down,
}

/// Scaling amount specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingAmount {
    /// Fixed number of instances
    Fixed(u32),
    /// Percentage of current instances
    Percentage(f64),
    /// Dynamic based on load
    Dynamic(String),
}

/// Service definition in the biome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDefinition {
    /// Service name
    pub name: String,
    /// Service description
    pub description: String,
    /// Service type/category
    pub service_type: String,
    /// Capabilities required by this service
    pub required_capabilities: Vec<CapabilityRequirement>,
    /// Service configuration
    pub config: ServiceConfig,
    /// Resource requirements
    pub resources: ResourceRequirements,
    /// Networking configuration
    pub networking: ServiceNetworking,
    /// Storage requirements
    pub storage: ServiceStorage,
    /// Health check configuration
    pub health_checks: Vec<HealthCheckConfig>,
    /// Dependencies on other services
    pub dependencies: Vec<ServiceDependency>,
}

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service image or source
    pub source: String,
    /// Runtime environment
    pub runtime: RuntimeSpec,
    /// Environment variables
    pub environment: HashMap<String, String>,
    /// Command to run
    pub command: Option<Vec<String>>,
    /// Working directory
    pub working_dir: Option<String>,
    /// User context
    pub user: Option<String>,
    /// Security context
    pub security_context: Option<SecurityContext>,
}

/// Runtime specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSpec {
    /// Runtime type preference
    pub runtime_type: RuntimeType,
    /// Runtime version
    pub version: Option<String>,
    /// Runtime-specific options
    pub options: HashMap<String, serde_json::Value>,
}

/// Runtime types (generic, not Primal-specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeType {
    /// Container runtime
    Container,
    /// WebAssembly runtime
    Wasm,
    /// Native process
    Native,
    /// Virtual machine
    VM,
    /// Serverless function
    Function,
    /// Custom runtime
    Custom(String),
}

/// Security context for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Run as privileged
    pub privileged: bool,
    /// Capabilities to add
    pub capabilities_add: Vec<String>,
    /// Capabilities to drop
    pub capabilities_drop: Vec<String>,
    /// Security profiles
    pub security_profiles: Vec<String>,
}

/// Service networking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceNetworking {
    /// Network ports to expose
    pub ports: Vec<PortSpec>,
    /// Network policies
    pub policies: Vec<NetworkPolicy>,
    /// Load balancing configuration
    pub load_balancing: Option<LoadBalancingConfig>,
}

/// Port specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortSpec {
    /// Port name
    pub name: String,
    /// Port number
    pub port: u16,
    /// Protocol
    pub protocol: String,
    /// Whether port is exposed externally
    pub external: bool,
}

/// Network policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    /// Policy name
    pub name: String,
    /// Policy type
    pub policy_type: String,
    /// Policy rules
    pub rules: Vec<NetworkRule>,
}

/// Network rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRule {
    /// Rule action
    pub action: String,
    /// Source specification
    pub source: Option<String>,
    /// Destination specification
    pub destination: Option<String>,
    /// Port specification
    pub ports: Option<Vec<String>>,
    /// Protocol specification
    pub protocol: Option<String>,
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// Load balancing algorithm
    pub algorithm: String,
    /// Health check configuration
    pub health_check: Option<HealthCheckConfig>,
    /// Session affinity
    pub session_affinity: Option<String>,
    /// Timeout configuration
    pub timeout: Option<String>,
}

/// Service storage requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStorage {
    /// Volume mounts
    pub volumes: Vec<VolumeMount>,
    /// Persistent storage requirements
    pub persistent: Vec<PersistentStorage>,
    /// Temporary storage requirements
    pub temporary: Option<TemporaryStorage>,
}

/// Volume mount specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    /// Volume name
    pub name: String,
    /// Mount path
    pub path: String,
    /// Read-only mount
    pub read_only: bool,
    /// Volume type
    pub volume_type: String,
    /// Volume options
    pub options: HashMap<String, String>,
}

/// Persistent storage specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentStorage {
    /// Storage name
    pub name: String,
    /// Storage size
    pub size: String,
    /// Storage class
    pub storage_class: Option<String>,
    /// Access mode
    pub access_mode: String,
    /// Backup policy
    pub backup_policy: Option<String>,
}

/// Temporary storage specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryStorage {
    /// Storage size
    pub size: String,
    /// Storage type
    pub storage_type: Option<String>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check name
    pub name: String,
    /// Health check type
    pub check_type: HealthCheckType,
    /// Check interval
    pub interval: String,
    /// Check timeout
    pub timeout: String,
    /// Failure threshold
    pub failure_threshold: u32,
    /// Success threshold
    pub success_threshold: u32,
    /// Initial delay
    pub initial_delay: Option<String>,
}

/// Health check types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthCheckType {
    /// HTTP health check
    Http {
        /// HTTP path
        path: String,
        /// HTTP port
        port: u16,
        /// HTTP headers
        headers: HashMap<String, String>,
    },
    /// TCP health check
    Tcp {
        /// TCP port
        port: u16,
    },
    /// Command health check
    Command {
        /// Command to execute
        command: Vec<String>,
    },
    /// Custom health check
    Custom {
        /// Custom check configuration
        config: HashMap<String, serde_json::Value>,
    },
}

/// Service dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependency {
    /// Dependency name
    pub name: String,
    /// Dependency type
    pub dependency_type: DependencyType,
    /// Whether dependency is optional
    pub optional: bool,
    /// Dependency configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Dependency types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    /// Service dependency
    Service,
    /// Capability dependency
    Capability,
    /// Resource dependency
    Resource,
    /// Network dependency
    Network,
    /// Storage dependency
    Storage,
}

/// Global resource specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalResourceSpec {
    /// Total resource limits
    pub limits: ResourceRequirements,
    /// Resource reservations
    pub reservations: ResourceRequirements,
    /// Resource quotas
    pub quotas: Vec<ResourceQuota>,
    /// Resource pools
    pub pools: Vec<ResourcePool>,
}

/// Resource quota
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuota {
    /// Quota name
    pub name: String,
    /// Resource type
    pub resource_type: String,
    /// Quota limit
    pub limit: String,
    /// Quota scope
    pub scope: String,
}

/// Resource pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePool {
    /// Pool name
    pub name: String,
    /// Pool type
    pub pool_type: String,
    /// Pool resources
    pub resources: ResourceRequirements,
    /// Pool policies
    pub policies: Vec<ResourcePolicy>,
}

/// Resource policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePolicy {
    /// Policy name
    pub name: String,
    /// Policy type
    pub policy_type: String,
    /// Policy rules
    pub rules: Vec<PolicyRule>,
}

/// Policy rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Rule condition
    pub condition: String,
    /// Rule action
    pub action: String,
    /// Rule parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Security requirements for the biome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirements {
    /// Authentication requirements
    pub authentication: AuthenticationSpec,
    /// Authorization requirements
    pub authorization: AuthorizationSpec,
    /// Encryption requirements
    pub encryption: EncryptionSpec,
    /// Network security
    pub network_security: NetworkSecuritySpec,
    /// Audit requirements
    pub audit: AuditSpec,
    /// Compliance requirements
    pub compliance: Vec<ComplianceSpec>,
}

/// Authentication specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationSpec {
    /// Authentication methods
    pub methods: Vec<String>,
    /// Multi-factor authentication
    pub mfa: bool,
    /// Token expiration
    pub token_expiration: Option<String>,
    /// Authentication providers
    pub providers: Vec<AuthProvider>,
}

/// Authentication provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthProvider {
    /// Provider name
    pub name: String,
    /// Provider type
    pub provider_type: String,
    /// Provider configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Authorization specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationSpec {
    /// Authorization model
    pub model: String,
    /// Role definitions
    pub roles: Vec<Role>,
    /// Permission definitions
    pub permissions: Vec<Permission>,
    /// Policy definitions
    pub policies: Vec<AuthPolicy>,
}

/// Role definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Role name
    pub name: String,
    /// Role description
    pub description: String,
    /// Role permissions
    pub permissions: Vec<String>,
}

/// Permission definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// Permission name
    pub name: String,
    /// Permission description
    pub description: String,
    /// Resource type
    pub resource_type: String,
    /// Allowed actions
    pub actions: Vec<String>,
}

/// Authorization policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthPolicy {
    /// Policy name
    pub name: String,
    /// Policy description
    pub description: String,
    /// Policy rules
    pub rules: Vec<AuthRule>,
}

/// Authorization rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthRule {
    /// Rule effect
    pub effect: String,
    /// Rule subjects
    pub subjects: Vec<String>,
    /// Rule resources
    pub resources: Vec<String>,
    /// Rule actions
    pub actions: Vec<String>,
    /// Rule conditions
    pub conditions: Vec<Condition>,
}

/// Rule condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Condition type
    pub condition_type: String,
    /// Condition value
    pub value: serde_json::Value,
}

/// Encryption specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionSpec {
    /// Encryption at rest
    pub at_rest: bool,
    /// Encryption in transit
    pub in_transit: bool,
    /// Encryption algorithms
    pub algorithms: Vec<String>,
    /// Key management
    pub key_management: KeyManagementSpec,
}

/// Key management specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyManagementSpec {
    /// Key provider
    pub provider: String,
    /// Key rotation
    pub rotation: bool,
    /// Key rotation interval
    pub rotation_interval: Option<String>,
    /// Key backup
    pub backup: bool,
}

/// Network security specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSecuritySpec {
    /// Network isolation
    pub isolation: bool,
    /// Firewall rules
    pub firewall: Vec<FirewallRule>,
    /// VPN requirements
    pub vpn: bool,
    /// DDoS protection
    pub ddos_protection: bool,
}

/// Firewall rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    /// Rule name
    pub name: String,
    /// Rule action
    pub action: String,
    /// Source specification
    pub source: Option<String>,
    /// Destination specification
    pub destination: Option<String>,
    /// Port specification
    pub ports: Option<Vec<String>>,
    /// Protocol specification
    pub protocol: Option<String>,
}

/// Audit specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSpec {
    /// Audit enabled
    pub enabled: bool,
    /// Audit events
    pub events: Vec<String>,
    /// Audit storage
    pub storage: AuditStorage,
    /// Audit retention
    pub retention: String,
}

/// Audit storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStorage {
    /// Storage type
    pub storage_type: String,
    /// Storage configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Compliance specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceSpec {
    /// Compliance standard
    pub standard: String,
    /// Compliance version
    pub version: String,
    /// Required controls
    pub controls: Vec<String>,
}

/// Networking specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkingSpec {
    /// Network topology
    pub topology: NetworkTopology,
    /// Network policies
    pub policies: Vec<NetworkPolicy>,
    /// Load balancing
    pub load_balancing: LoadBalancingSpec,
    /// Service mesh
    pub service_mesh: Option<ServiceMeshSpec>,
}

/// Network topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkTopology {
    /// Flat network
    Flat,
    /// Segmented network
    Segmented,
    /// Mesh network
    Mesh,
    /// Hub and spoke
    HubSpoke,
    /// Custom topology
    Custom(String),
}

/// Load balancing specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingSpec {
    /// Load balancing enabled
    pub enabled: bool,
    /// Load balancing algorithm
    pub algorithm: String,
    /// Health check configuration
    pub health_check: Option<HealthCheckConfig>,
}

/// Service mesh specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeshSpec {
    /// Service mesh type
    pub mesh_type: String,
    /// Service mesh configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Storage specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSpec {
    /// Storage classes
    pub storage_classes: Vec<StorageClass>,
    /// Default storage class
    pub default_class: String,
    /// Storage policies
    pub policies: Vec<StoragePolicy>,
    /// Backup configuration
    pub backup: BackupSpec,
}

/// Storage class definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageClass {
    /// Storage class name
    pub name: String,
    /// Storage provisioner
    pub provisioner: String,
    /// Storage parameters
    pub parameters: HashMap<String, String>,
    /// Reclaim policy
    pub reclaim_policy: String,
    /// Volume binding mode
    pub volume_binding_mode: String,
}

/// Storage policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePolicy {
    /// Policy name
    pub name: String,
    /// Policy type
    pub policy_type: String,
    /// Policy rules
    pub rules: Vec<StorageRule>,
}

/// Storage rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRule {
    /// Rule condition
    pub condition: String,
    /// Rule action
    pub action: String,
    /// Rule parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Backup specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSpec {
    /// Backup enabled
    pub enabled: bool,
    /// Backup schedule
    pub schedule: String,
    /// Backup retention
    pub retention: String,
    /// Backup encryption
    pub encryption: bool,
    /// Backup storage
    pub storage: BackupStorage,
}

/// Backup storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupStorage {
    /// Storage type
    pub storage_type: String,
    /// Storage configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Monitoring specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringSpec {
    /// Monitoring enabled
    pub enabled: bool,
    /// Metrics collection
    pub metrics: MetricsSpec,
    /// Logging configuration
    pub logging: LoggingSpec,
    /// Alerting configuration
    pub alerting: AlertingSpec,
    /// Tracing configuration
    pub tracing: TracingSpec,
}

/// Metrics specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSpec {
    /// Metrics enabled
    pub enabled: bool,
    /// Metrics retention
    pub retention: String,
    /// Metrics storage
    pub storage: MetricsStorage,
}

/// Metrics storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsStorage {
    /// Storage type
    pub storage_type: String,
    /// Storage configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Logging specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSpec {
    /// Logging enabled
    pub enabled: bool,
    /// Log level
    pub level: String,
    /// Log retention
    pub retention: String,
    /// Log storage
    pub storage: LogStorage,
}

/// Log storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStorage {
    /// Storage type
    pub storage_type: String,
    /// Storage configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Alerting specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingSpec {
    /// Alerting enabled
    pub enabled: bool,
    /// Alert rules
    pub rules: Vec<AlertRule>,
    /// Alert receivers
    pub receivers: Vec<AlertReceiver>,
}

/// Alert rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Rule name
    pub name: String,
    /// Rule expression
    pub expression: String,
    /// Rule severity
    pub severity: String,
    /// Rule description
    pub description: String,
}

/// Alert receiver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertReceiver {
    /// Receiver name
    pub name: String,
    /// Receiver type
    pub receiver_type: String,
    /// Receiver configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Tracing specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingSpec {
    /// Tracing enabled
    pub enabled: bool,
    /// Sampling rate
    pub sampling_rate: f64,
    /// Tracing storage
    pub storage: TracingStorage,
}

/// Tracing storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingStorage {
    /// Storage type
    pub storage_type: String,
    /// Storage configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Deployment preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentPreferences {
    /// Preferred deployment strategy
    pub strategy: DeploymentStrategy,
    /// Preferred primals (if available)
    pub preferred_primals: Vec<PrimalPreference>,
    /// Deployment constraints
    pub constraints: Vec<DeploymentConstraint>,
    /// Deployment policies
    pub policies: Vec<DeploymentPolicy>,
}

/// Deployment strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStrategy {
    /// Optimize for performance
    Performance,
    /// Optimize for cost
    Cost,
    /// Optimize for reliability
    Reliability,
    /// Optimize for security
    Security,
    /// Balanced approach
    Balanced,
    /// Custom strategy
    Custom(String),
}

/// Primal preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalPreference {
    /// Primal type preference
    pub primal_type: String,
    /// Preference weight
    pub weight: f64,
    /// Preference constraints
    pub constraints: Vec<Constraint>,
}

/// Deployment constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConstraint {
    /// Constraint type
    pub constraint_type: String,
    /// Constraint value
    pub value: serde_json::Value,
    /// Constraint description
    pub description: String,
}

/// Deployment policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentPolicy {
    /// Policy name
    pub name: String,
    /// Policy type
    pub policy_type: String,
    /// Policy rules
    pub rules: Vec<PolicyRule>,
}

/// Validation specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSpec {
    /// Validation rules
    pub rules: Vec<ValidationRule>,
    /// Validation policies
    pub policies: Vec<ValidationPolicy>,
}

/// Validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Rule name
    pub name: String,
    /// Rule type
    pub rule_type: String,
    /// Rule expression
    pub expression: String,
    /// Rule message
    pub message: String,
}

/// Validation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationPolicy {
    /// Policy name
    pub name: String,
    /// Policy type
    pub policy_type: String,
    /// Policy rules
    pub rules: Vec<ValidationRule>,
}

/// Manifest validation and parsing
impl UniversalBiomeManifest {
    /// Parse manifest from YAML string
    pub fn from_yaml(yaml: &str) -> BiomeResult<Self> {
        serde_yaml::from_str(yaml).map_err(|e| BiomeError::ConfigError(e.to_string()))
    }

    /// Parse manifest from JSON string
    pub fn from_json(json: &str) -> BiomeResult<Self> {
        serde_json::from_str(json).map_err(|e| BiomeError::ConfigError(e.to_string()))
    }

    /// Serialize manifest to YAML string
    pub fn to_yaml(&self) -> BiomeResult<String> {
        serde_yaml::to_string(self).map_err(|e| BiomeError::ConfigError(e.to_string()))
    }

    /// Serialize manifest to JSON string
    pub fn to_json(&self) -> BiomeResult<String> {
        serde_json::to_string_pretty(self).map_err(|e| BiomeError::ConfigError(e.to_string()))
    }

    /// Validate manifest structure and requirements
    pub fn validate(&self) -> BiomeResult<Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate API version
        if self.api_version.is_empty() {
            errors.push(ValidationError {
                field: "api_version".to_string(),
                message: "API version is required".to_string(),
                severity: ValidationSeverity::Error,
            });
        }

        // Validate kind
        if self.kind != "Biome" {
            errors.push(ValidationError {
                field: "kind".to_string(),
                message: "Kind must be 'Biome'".to_string(),
                severity: ValidationSeverity::Error,
            });
        }

        // Validate metadata
        if self.metadata.name.is_empty() {
            errors.push(ValidationError {
                field: "metadata.name".to_string(),
                message: "Biome name is required".to_string(),
                severity: ValidationSeverity::Error,
            });
        }

        // Validate requirements
        if self.requirements.required.is_empty() {
            errors.push(ValidationError {
                field: "requirements.required".to_string(),
                message: "At least one required capability must be specified".to_string(),
                severity: ValidationSeverity::Warning,
            });
        }

        // Validate services
        for (i, service) in self.services.iter().enumerate() {
            if service.name.is_empty() {
                errors.push(ValidationError {
                    field: format!("services[{}].name", i),
                    message: "Service name is required".to_string(),
                    severity: ValidationSeverity::Error,
                });
            }

            if service.required_capabilities.is_empty() {
                errors.push(ValidationError {
                    field: format!("services[{}].required_capabilities", i),
                    message: "Service must specify required capabilities".to_string(),
                    severity: ValidationSeverity::Warning,
                });
            }
        }

        // Validate custom validation rules
        for rule in &self.validation.rules {
            // Apply custom validation rules
            if let Err(e) = self.apply_validation_rule(rule) {
                errors.push(ValidationError {
                    field: rule.name.clone(),
                    message: e.to_string(),
                    severity: ValidationSeverity::Error,
                });
            }
        }

        Ok(errors)
    }

    /// Apply a validation rule
    fn apply_validation_rule(&self, _rule: &ValidationRule) -> BiomeResult<()> {
        // TODO: Implement validation rule application
        Ok(())
    }

    /// Get all required capabilities from the manifest
    pub fn get_all_required_capabilities(&self) -> Vec<String> {
        let mut capabilities = Vec::new();

        // Add biome-level requirements
        for req in &self.requirements.required {
            capabilities.push(req.capability.clone());
        }

        // Add service-level requirements
        for service in &self.services {
            for req in &service.required_capabilities {
                capabilities.push(req.capability.clone());
            }
        }

        capabilities.sort();
        capabilities.dedup();
        capabilities
    }

    /// Get all optional capabilities from the manifest
    pub fn get_all_optional_capabilities(&self) -> Vec<String> {
        let mut capabilities = Vec::new();

        // Add biome-level optional capabilities
        for req in &self.requirements.optional {
            capabilities.push(req.capability.clone());
        }

        capabilities.sort();
        capabilities.dedup();
        capabilities
    }

    /// Get resource requirements summary
    pub fn get_resource_summary(&self) -> ResourceSummary {
        let mut summary = ResourceSummary::default();

        // Add biome-level resource requirements
        if let Some(cpu) = &self.requirements.min_resources.cpu {
            summary.total_cpu = cpu.clone();
        }
        if let Some(memory) = &self.requirements.min_resources.memory {
            summary.total_memory = memory.clone();
        }
        if let Some(storage) = &self.requirements.min_resources.storage {
            summary.total_storage = storage.clone();
        }

        // Count services
        summary.service_count = self.services.len() as u32;

        summary
    }
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Field with error
    pub field: String,
    /// Error message
    pub message: String,
    /// Error severity
    pub severity: ValidationSeverity,
}

/// Validation severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Error that prevents deployment
    Error,
    /// Warning that should be addressed
    Warning,
    /// Informational message
    Info,
}

/// Resource summary
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceSummary {
    /// Total CPU requirements
    pub total_cpu: String,
    /// Total memory requirements
    pub total_memory: String,
    /// Total storage requirements
    pub total_storage: String,
    /// Number of services
    pub service_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_validation() {
        let manifest = UniversalBiomeManifest {
            api_version: "biomeOS/v1".to_string(),
            kind: "Biome".to_string(),
            metadata: BiomeMetadata {
                name: "test-biome".to_string(),
                description: "Test biome".to_string(),
                version: "1.0.0".to_string(),
                maintainer: None,
                tags: vec![],
                labels: HashMap::new(),
                annotations: HashMap::new(),
                created: None,
                modified: None,
            },
            requirements: BiomeRequirements {
                required: vec![CapabilityRequirement {
                    capability: "compute.orchestration".to_string(),
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    optional: false,
                    constraints: vec![],
                    fallback: None,
                }],
                optional: vec![],
                min_resources: ResourceRequirements {
                    cpu: Some("100m".to_string()),
                    memory: Some("128Mi".to_string()),
                    storage: Some("1Gi".to_string()),
                    network: None,
                    gpu: None,
                },
                preferred_resources: None,
                max_resources: None,
                performance: PerformanceRequirements {
                    max_latency_ms: None,
                    min_throughput: None,
                    max_error_rate: None,
                    min_uptime: None,
                },
                availability: AvailabilityRequirements {
                    high_availability: false,
                    fault_tolerance: FaultToleranceLevel::None,
                    disaster_recovery: false,
                    backup: BackupRequirements {
                        required: false,
                        frequency: None,
                        retention: None,
                        encryption: false,
                    },
                },
                scaling: ScalingRequirements {
                    auto_scaling: false,
                    min_instances: 1,
                    max_instances: 1,
                    triggers: vec![],
                    policies: vec![],
                },
            },
            services: vec![],
            resources: GlobalResourceSpec {
                limits: ResourceRequirements {
                    cpu: Some("1000m".to_string()),
                    memory: Some("1Gi".to_string()),
                    storage: Some("10Gi".to_string()),
                    network: None,
                    gpu: None,
                },
                reservations: ResourceRequirements {
                    cpu: Some("100m".to_string()),
                    memory: Some("128Mi".to_string()),
                    storage: Some("1Gi".to_string()),
                    network: None,
                    gpu: None,
                },
                quotas: vec![],
                pools: vec![],
            },
            security: SecurityRequirements {
                authentication: AuthenticationSpec {
                    methods: vec!["token".to_string()],
                    mfa: false,
                    token_expiration: None,
                    providers: vec![],
                },
                authorization: AuthorizationSpec {
                    model: "rbac".to_string(),
                    roles: vec![],
                    permissions: vec![],
                    policies: vec![],
                },
                encryption: EncryptionSpec {
                    at_rest: false,
                    in_transit: false,
                    algorithms: vec![],
                    key_management: KeyManagementSpec {
                        provider: "default".to_string(),
                        rotation: false,
                        rotation_interval: None,
                        backup: false,
                    },
                },
                network_security: NetworkSecuritySpec {
                    isolation: false,
                    firewall: vec![],
                    vpn: false,
                    ddos_protection: false,
                },
                audit: AuditSpec {
                    enabled: false,
                    events: vec![],
                    storage: AuditStorage {
                        storage_type: "file".to_string(),
                        config: HashMap::new(),
                    },
                    retention: "30d".to_string(),
                },
                compliance: vec![],
            },
            networking: NetworkingSpec {
                topology: NetworkTopology::Flat,
                policies: vec![],
                load_balancing: LoadBalancingSpec {
                    enabled: false,
                    algorithm: "round_robin".to_string(),
                    health_check: None,
                },
                service_mesh: None,
            },
            storage: StorageSpec {
                storage_classes: vec![],
                default_class: "default".to_string(),
                policies: vec![],
                backup: BackupSpec {
                    enabled: false,
                    schedule: "0 0 * * *".to_string(),
                    retention: "7d".to_string(),
                    encryption: false,
                    storage: BackupStorage {
                        storage_type: "file".to_string(),
                        config: HashMap::new(),
                    },
                },
            },
            monitoring: MonitoringSpec {
                enabled: false,
                metrics: MetricsSpec {
                    enabled: false,
                    retention: "7d".to_string(),
                    storage: MetricsStorage {
                        storage_type: "memory".to_string(),
                        config: HashMap::new(),
                    },
                },
                logging: LoggingSpec {
                    enabled: false,
                    level: "info".to_string(),
                    retention: "7d".to_string(),
                    storage: LogStorage {
                        storage_type: "file".to_string(),
                        config: HashMap::new(),
                    },
                },
                alerting: AlertingSpec {
                    enabled: false,
                    rules: vec![],
                    receivers: vec![],
                },
                tracing: TracingSpec {
                    enabled: false,
                    sampling_rate: 0.1,
                    storage: TracingStorage {
                        storage_type: "memory".to_string(),
                        config: HashMap::new(),
                    },
                },
            },
            deployment: DeploymentPreferences {
                strategy: DeploymentStrategy::Balanced,
                preferred_primals: vec![],
                constraints: vec![],
                policies: vec![],
            },
            validation: ValidationSpec {
                rules: vec![],
                policies: vec![],
            },
        };

        let errors = manifest.validate().unwrap();
        assert!(errors.is_empty());
    }

    #[test]
    fn test_capability_extraction() {
        let manifest = UniversalBiomeManifest {
            api_version: "biomeOS/v1".to_string(),
            kind: "Biome".to_string(),
            metadata: BiomeMetadata {
                name: "test-biome".to_string(),
                description: "Test biome".to_string(),
                version: "1.0.0".to_string(),
                maintainer: None,
                tags: vec![],
                labels: HashMap::new(),
                annotations: HashMap::new(),
                created: None,
                modified: None,
            },
            requirements: BiomeRequirements {
                required: vec![CapabilityRequirement {
                    capability: "compute.orchestration".to_string(),
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    optional: false,
                    constraints: vec![],
                    fallback: None,
                }],
                optional: vec![CapabilityRequirement {
                    capability: "storage.backup".to_string(),
                    min_version: "1.0.0".to_string(),
                    max_version: None,
                    optional: true,
                    constraints: vec![],
                    fallback: None,
                }],
                // ... rest of the fields with default values
                min_resources: ResourceRequirements {
                    cpu: Some("100m".to_string()),
                    memory: Some("128Mi".to_string()),
                    storage: Some("1Gi".to_string()),
                    network: None,
                    gpu: None,
                },
                preferred_resources: None,
                max_resources: None,
                performance: PerformanceRequirements {
                    max_latency_ms: None,
                    min_throughput: None,
                    max_error_rate: None,
                    min_uptime: None,
                },
                availability: AvailabilityRequirements {
                    high_availability: false,
                    fault_tolerance: FaultToleranceLevel::None,
                    disaster_recovery: false,
                    backup: BackupRequirements {
                        required: false,
                        frequency: None,
                        retention: None,
                        encryption: false,
                    },
                },
                scaling: ScalingRequirements {
                    auto_scaling: false,
                    min_instances: 1,
                    max_instances: 1,
                    triggers: vec![],
                    policies: vec![],
                },
            },
            services: vec![],
            // ... rest of the fields with default values
            resources: GlobalResourceSpec {
                limits: ResourceRequirements {
                    cpu: Some("1000m".to_string()),
                    memory: Some("1Gi".to_string()),
                    storage: Some("10Gi".to_string()),
                    network: None,
                    gpu: None,
                },
                reservations: ResourceRequirements {
                    cpu: Some("100m".to_string()),
                    memory: Some("128Mi".to_string()),
                    storage: Some("1Gi".to_string()),
                    network: None,
                    gpu: None,
                },
                quotas: vec![],
                pools: vec![],
            },
            security: SecurityRequirements {
                authentication: AuthenticationSpec {
                    methods: vec!["token".to_string()],
                    mfa: false,
                    token_expiration: None,
                    providers: vec![],
                },
                authorization: AuthorizationSpec {
                    model: "rbac".to_string(),
                    roles: vec![],
                    permissions: vec![],
                    policies: vec![],
                },
                encryption: EncryptionSpec {
                    at_rest: false,
                    in_transit: false,
                    algorithms: vec![],
                    key_management: KeyManagementSpec {
                        provider: "default".to_string(),
                        rotation: false,
                        rotation_interval: None,
                        backup: false,
                    },
                },
                network_security: NetworkSecuritySpec {
                    isolation: false,
                    firewall: vec![],
                    vpn: false,
                    ddos_protection: false,
                },
                audit: AuditSpec {
                    enabled: false,
                    events: vec![],
                    storage: AuditStorage {
                        storage_type: "file".to_string(),
                        config: HashMap::new(),
                    },
                    retention: "30d".to_string(),
                },
                compliance: vec![],
            },
            networking: NetworkingSpec {
                topology: NetworkTopology::Flat,
                policies: vec![],
                load_balancing: LoadBalancingSpec {
                    enabled: false,
                    algorithm: "round_robin".to_string(),
                    health_check: None,
                },
                service_mesh: None,
            },
            storage: StorageSpec {
                storage_classes: vec![],
                default_class: "default".to_string(),
                policies: vec![],
                backup: BackupSpec {
                    enabled: false,
                    schedule: "0 0 * * *".to_string(),
                    retention: "7d".to_string(),
                    encryption: false,
                    storage: BackupStorage {
                        storage_type: "file".to_string(),
                        config: HashMap::new(),
                    },
                },
            },
            monitoring: MonitoringSpec {
                enabled: false,
                metrics: MetricsSpec {
                    enabled: false,
                    retention: "7d".to_string(),
                    storage: MetricsStorage {
                        storage_type: "memory".to_string(),
                        config: HashMap::new(),
                    },
                },
                logging: LoggingSpec {
                    enabled: false,
                    level: "info".to_string(),
                    retention: "7d".to_string(),
                    storage: LogStorage {
                        storage_type: "file".to_string(),
                        config: HashMap::new(),
                    },
                },
                alerting: AlertingSpec {
                    enabled: false,
                    rules: vec![],
                    receivers: vec![],
                },
                tracing: TracingSpec {
                    enabled: false,
                    sampling_rate: 0.1,
                    storage: TracingStorage {
                        storage_type: "memory".to_string(),
                        config: HashMap::new(),
                    },
                },
            },
            deployment: DeploymentPreferences {
                strategy: DeploymentStrategy::Balanced,
                preferred_primals: vec![],
                constraints: vec![],
                policies: vec![],
            },
            validation: ValidationSpec {
                rules: vec![],
                policies: vec![],
            },
        };

        let required_capabilities = manifest.get_all_required_capabilities();
        assert_eq!(required_capabilities, vec!["compute.orchestration"]);

        let optional_capabilities = manifest.get_all_optional_capabilities();
        assert_eq!(optional_capabilities, vec!["storage.backup"]);
    }
}
