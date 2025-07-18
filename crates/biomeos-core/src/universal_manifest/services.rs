//! Service Definitions Module
//!
//! This module defines service specifications for biomes including
//! service definitions, runtime specs, and health checks.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Service definition in the biome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDefinition {
    /// Service name
    pub name: String,
    /// Service description
    pub description: String,
    /// Service version
    pub version: String,
    /// Service configuration
    pub config: ServiceConfig,
    /// Runtime specification
    pub runtime: RuntimeSpec,
    /// Security context
    pub security: SecurityContext,
    /// Networking configuration
    pub networking: ServiceNetworking,
    /// Storage configuration
    pub storage: ServiceStorage,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Dependencies
    pub dependencies: Vec<ServiceDependency>,
}

/// Service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Environment variables
    pub environment: HashMap<String, String>,
    /// Command line arguments
    pub args: Vec<String>,
    /// Working directory
    pub working_dir: Option<PathBuf>,
    /// User to run as
    pub user: Option<String>,
    /// Group to run as
    pub group: Option<String>,
    /// Resource limits
    pub limits: ResourceLimits,
}

/// Runtime specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSpec {
    /// Runtime type
    pub runtime_type: RuntimeType,
    /// Image or executable path
    pub image: String,
    /// Command to run
    pub command: Vec<String>,
    /// Restart policy
    pub restart_policy: RestartPolicy,
}

/// Runtime types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeType {
    /// Container runtime
    Container,
    /// Native process
    Native,
    /// WebAssembly runtime
    Wasm,
    /// Virtual machine
    VM,
    /// Serverless function
    Serverless,
}

/// Restart policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestartPolicy {
    /// Never restart
    Never,
    /// Always restart
    Always,
    /// Restart on failure
    OnFailure,
    /// Restart unless stopped
    UnlessStopped,
}

/// Security context for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Run as privileged
    pub privileged: bool,
    /// User ID
    pub user_id: Option<u32>,
    /// Group ID
    pub group_id: Option<u32>,
    /// Capabilities
    pub capabilities: Vec<String>,
    /// Security profiles
    pub security_profiles: Vec<String>,
}

/// Service networking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceNetworking {
    /// Port specifications
    pub ports: Vec<PortSpec>,
    /// Network policies
    pub policies: Vec<NetworkPolicy>,
    /// Load balancing configuration
    pub load_balancing: LoadBalancingConfig,
}

/// Port specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortSpec {
    /// Port name
    pub name: String,
    /// Port number
    pub port: u16,
    /// Target port
    pub target_port: u16,
    /// Protocol
    pub protocol: String,
}

/// Network policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    /// Policy name
    pub name: String,
    /// Rules
    pub rules: Vec<NetworkRule>,
}

/// Network rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRule {
    /// Rule action
    pub action: String,
    /// Source
    pub source: String,
    /// Destination
    pub destination: String,
    /// Protocol
    pub protocol: String,
    /// Port range
    pub port_range: Option<String>,
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// Load balancing algorithm
    pub algorithm: String,
    /// Health check path
    pub health_check_path: String,
    /// Session affinity
    pub session_affinity: bool,
}

/// Service storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStorage {
    /// Volume mounts
    pub volumes: Vec<VolumeMount>,
    /// Persistent storage
    pub persistent: Vec<PersistentStorage>,
    /// Temporary storage
    pub temporary: Vec<TemporaryStorage>,
}

/// Volume mount specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    /// Mount name
    pub name: String,
    /// Mount path
    pub path: PathBuf,
    /// Read only
    pub read_only: bool,
    /// Volume type
    pub volume_type: String,
    /// Volume source
    pub source: String,
}

/// Persistent storage specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentStorage {
    /// Storage name
    pub name: String,
    /// Storage size in MB
    pub size_mb: u64,
    /// Storage class
    pub storage_class: String,
    /// Access mode
    pub access_mode: String,
    /// Mount path
    pub mount_path: PathBuf,
}

/// Temporary storage specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryStorage {
    /// Path
    pub path: PathBuf,
    /// Size limit in MB
    pub size_limit_mb: u64,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check type
    pub check_type: HealthCheckType,
    /// Check interval in seconds
    pub interval_seconds: u64,
    /// Timeout in seconds
    pub timeout_seconds: u64,
    /// Retry count
    pub retry_count: u32,
    /// Initial delay in seconds
    pub initial_delay_seconds: u64,
    /// Failure threshold
    pub failure_threshold: u32,
    /// Success threshold
    pub success_threshold: u32,
}

/// Health check types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthCheckType {
    /// HTTP health check
    Http {
        /// Path to check
        path: String,
        /// Port to check
        port: u16,
        /// HTTP method
        method: String,
        /// Expected status code
        expected_status: u16,
    },
    /// TCP health check
    Tcp {
        /// Port to check
        port: u16,
    },
    /// Command health check
    Command {
        /// Command to run
        command: Vec<String>,
    },
    /// Custom health check
    Custom {
        /// Custom check specification
        spec: HashMap<String, String>,
    },
}

/// Service dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependency {
    /// Service name
    pub service: String,
    /// Dependency type
    pub dependency_type: DependencyType,
    /// Required condition
    pub condition: String,
}

/// Dependency types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    /// Hard dependency - must be available
    Hard,
    /// Soft dependency - nice to have
    Soft,
    /// Startup dependency - required for startup
    Startup,
}

/// Resource limits for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// CPU limit
    pub cpu: f64,
    /// Memory limit in MB
    pub memory_mb: u64,
    /// Disk limit in MB
    pub disk_mb: u64,
    /// Network bandwidth limit in Mbps
    pub network_mbps: u64,
}

/// Load balancing specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingSpec {
    /// Load balancing algorithm
    pub algorithm: String,
    /// Load balancing configuration
    pub config: LoadBalancingConfig,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
}

/// Fault tolerance level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FaultToleranceLevel {
    /// No fault tolerance
    None,
    /// Basic fault tolerance
    Basic,
    /// High fault tolerance
    High,
    /// Maximum fault tolerance
    Maximum,
}

/// Validation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStrategy {
    /// Strict validation
    Strict,
    /// Lenient validation
    Lenient,
    /// Custom validation
    Custom(String),
}

/// Service specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpec {
    /// Service definition
    pub definition: ServiceDefinition,
    /// Service configuration
    pub config: ServiceConfig,
    /// Service networking
    pub networking: ServiceNetworking,
    /// Service storage
    pub storage: ServiceStorage,
    /// Service dependencies
    pub dependencies: Vec<ServiceDependency>,
}

/// Service mounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMounts {
    /// Volume mounts
    pub volumes: Vec<VolumeMount>,
    /// Config mounts
    pub configs: Vec<ConfigMount>,
}

/// Configuration mount
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMount {
    /// Config name
    pub name: String,
    /// Mount path
    pub path: String,
    /// Mount mode
    pub mode: String,
} 