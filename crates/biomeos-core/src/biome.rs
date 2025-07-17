//! Biome lifecycle and management
//!
//! This module defines the core Biome type and its lifecycle management.
//! A Biome represents a complete biomeOS instance with all its Primals.

use crate::{BiomeId, HealthStatus, PrimalId, PrimalType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Represents a complete biomeOS instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Biome {
    /// Unique identifier for this biome
    pub id: BiomeId,
    /// Human-readable name
    pub name: String,
    /// Description of this biome's purpose
    pub description: String,
    /// Biome metadata
    pub metadata: BiomeMetadata,
    /// Primals running in this biome
    pub primals: HashMap<PrimalType, PrimalInstance>,
    /// Current biome state
    pub state: BiomeState,
    /// Health status
    pub health: BiomeHealth,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Biome metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeMetadata {
    /// Biome name
    pub name: String,

    /// Namespace
    pub namespace: Option<String>,

    /// Labels
    pub labels: HashMap<String, String>,

    /// Annotations
    pub annotations: HashMap<String, String>,
}

/// Biome specialization types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BiomeSpecialization {
    /// General development environment
    Development,
    /// AI research and ML workloads
    AiResearch,
    /// Scientific computing and simulations
    ScientificComputing,
    /// Secure enterprise applications
    SecureEnterprise,
    /// Edge computing deployments
    EdgeComputing,
    /// Quantum computing research
    QuantumComputing,
    /// Genomics and bioinformatics
    Genomics,
    /// Computer vision and image processing
    ComputerVision,
    /// Custom specialization
    Custom { name: String, description: String },
}

/// Represents a Primal instance within a biome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInstance {
    /// Primal identifier
    pub id: PrimalId,
    /// Type of Primal
    pub primal_type: PrimalType,
    /// Instance name
    pub name: String,
    /// Configuration for this Primal
    pub config: serde_json::Value,
    /// Current state
    pub state: PrimalState,
    /// Health status
    pub health: HealthStatus,
    /// Endpoints exposed by this Primal
    pub endpoints: Vec<PrimalEndpoint>,
    /// Capabilities provided
    pub capabilities: Vec<String>,
    /// Resource usage
    pub resources: ResourceUsage,
}

/// Primal endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpoint {
    /// Endpoint name
    pub name: String,
    /// URL or address
    pub url: String,
    /// Protocol (http, grpc, websocket, etc.)
    pub protocol: String,
    /// Whether this endpoint is healthy
    pub healthy: bool,
}

/// Current state of a Primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimalState {
    /// Primal is initializing
    Initializing,
    /// Primal is starting up
    Starting,
    /// Primal is running normally
    Running,
    /// Primal is stopping
    Stopping,
    /// Primal has stopped
    Stopped,
    /// Primal has failed
    Failed { error: String },
    /// Primal is in maintenance mode
    Maintenance,
}

/// Current state of a biome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BiomeState {
    /// Biome is being created
    Creating,
    /// Biome is starting up
    Starting,
    /// Biome is running normally
    Running,
    /// Biome is stopping
    Stopping,
    /// Biome has stopped
    Stopped,
    /// Biome has failed
    Failed { error: String },
    /// Biome is being updated
    Updating,
    /// Biome is in maintenance mode
    Maintenance,
}

/// Health status of the entire biome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeHealth {
    /// Overall health status
    pub status: HealthStatus,
    /// Health of individual Primals
    pub primal_health: HashMap<PrimalType, HealthStatus>,
    /// Health check timestamp
    pub last_check: DateTime<Utc>,
    /// Any health issues
    pub issues: Vec<HealthIssue>,
    /// Performance metrics
    pub metrics: BiomeMetrics,
}

/// Health issue details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    /// Severity level
    pub severity: IssueSeverity,
    /// Affected component
    pub component: String,
    /// Issue description
    pub description: String,
    /// When the issue was detected
    pub detected_at: DateTime<Utc>,
    /// Suggested resolution
    pub resolution: Option<String>,
}

/// Issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Informational
    Info,
    /// Warning that should be addressed
    Warning,
    /// Error that affects functionality
    Error,
    /// Critical issue requiring immediate attention
    Critical,
}

/// Resource usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// Disk usage in bytes
    pub disk_bytes: u64,
    /// Network bytes sent
    pub network_sent_bytes: u64,
    /// Network bytes received
    pub network_received_bytes: u64,
}

/// Biome performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeMetrics {
    /// Total resource usage
    pub total_resources: ResourceUsage,
    /// Request throughput (requests per second)
    pub requests_per_second: f64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Error rate percentage
    pub error_rate_percent: f64,
    /// Uptime in seconds
    pub uptime_seconds: u64,
}

/// Biome specification defining the structure and requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeSpec {
    /// Biome metadata
    pub metadata: BiomeMetadata,

    /// Primal configurations within this biome
    pub primals: Vec<PrimalSpec>,

    /// Resource requirements
    pub resources: ResourceRequirements,

    /// Networking configuration
    pub networking: NetworkingSpec,

    /// Security requirements
    pub security: SecuritySpec,
}

/// Biome manifest for deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeManifest {
    /// API version
    pub api_version: String,

    /// Kind of manifest
    pub kind: String,

    /// Metadata
    pub metadata: BiomeMetadata,

    /// Specification
    pub spec: BiomeSpec,

    /// Current status
    pub status: Option<BiomeStatus>,
}

/// Primal specification within a biome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalSpec {
    /// Primal name
    pub name: String,

    /// Primal type
    pub primal_type: PrimalType,

    /// Configuration
    pub config: HashMap<String, String>,

    /// Dependencies
    pub dependencies: Vec<String>,
}

/// Resource requirements for the biome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// CPU requirements
    pub cpu: CpuRequirements,

    /// Memory requirements
    pub memory: MemoryRequirements,

    /// Storage requirements
    pub storage: StorageRequirements,

    /// Network requirements
    pub network: NetworkRequirements,
}

/// CPU requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuRequirements {
    /// Minimum cores
    pub min_cores: u32,

    /// Maximum cores
    pub max_cores: Option<u32>,

    /// Architecture preferences
    pub architectures: Vec<String>,
}

/// Memory requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRequirements {
    /// Minimum memory in MB
    pub min_mb: u64,

    /// Maximum memory in MB
    pub max_mb: Option<u64>,
}

/// Storage requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRequirements {
    /// Minimum storage in GB
    pub min_gb: u64,

    /// Storage type preferences
    pub storage_types: Vec<String>,
}

/// Network requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequirements {
    /// Required bandwidth in Mbps
    pub bandwidth_mbps: u64,

    /// Latency requirements in milliseconds
    pub max_latency_ms: u32,
}

/// Networking specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkingSpec {
    /// Service mesh enabled
    pub service_mesh: bool,

    /// Load balancing strategy
    pub load_balancing: String,

    /// Service discovery
    pub service_discovery: bool,
}

/// Security specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySpec {
    /// Encryption requirements
    pub encryption: EncryptionSpec,

    /// Access control
    pub access_control: AccessControlSpec,

    /// Network policies
    pub network_policies: Vec<NetworkPolicy>,
}

/// Encryption specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionSpec {
    /// Encrypt data at rest
    pub at_rest: bool,

    /// Encrypt data in transit
    pub in_transit: bool,

    /// Encryption algorithms
    pub algorithms: Vec<String>,
}

/// Access control specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlSpec {
    /// RBAC enabled
    pub rbac: bool,

    /// Authentication methods
    pub auth_methods: Vec<String>,

    /// Authorization policies
    pub policies: Vec<String>,
}

/// Network policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    /// Policy name
    pub name: String,

    /// Ingress rules
    pub ingress: Vec<String>,

    /// Egress rules
    pub egress: Vec<String>,
}

/// Biome status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeStatus {
    /// Current phase
    pub phase: BiomePhase,

    /// Status message
    pub message: Option<String>,

    /// Ready condition
    pub ready: bool,

    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Biome phases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BiomePhase {
    Pending,
    Creating,
    Running,
    Updating,
    Deleting,
    Failed,
}

impl Default for BiomeSpec {
    fn default() -> Self {
        Self {
            metadata: BiomeMetadata {
                name: "default-biome".to_string(),
                namespace: None,
                labels: HashMap::new(),
                annotations: HashMap::new(),
            },
            primals: vec![],
            resources: ResourceRequirements {
                cpu: CpuRequirements {
                    min_cores: 1,
                    max_cores: None,
                    architectures: vec!["x86_64".to_string()],
                },
                memory: MemoryRequirements {
                    min_mb: 512,
                    max_mb: None,
                },
                storage: StorageRequirements {
                    min_gb: 10,
                    storage_types: vec!["ssd".to_string()],
                },
                network: NetworkRequirements {
                    bandwidth_mbps: 100,
                    max_latency_ms: 100,
                },
            },
            networking: NetworkingSpec {
                service_mesh: false,
                load_balancing: "round_robin".to_string(),
                service_discovery: true,
            },
            security: SecuritySpec {
                encryption: EncryptionSpec {
                    at_rest: true,
                    in_transit: true,
                    algorithms: vec!["aes256".to_string()],
                },
                access_control: AccessControlSpec {
                    rbac: true,
                    auth_methods: vec!["oauth2".to_string()],
                    policies: vec!["default".to_string()],
                },
                network_policies: vec![],
            },
        }
    }
}

impl Biome {
    /// Create a new biome
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.clone(),
            description,
            metadata: BiomeMetadata {
                name: name.clone(),
                namespace: None,
                labels: HashMap::new(),
                annotations: HashMap::new(),
            },
            primals: HashMap::new(),
            state: BiomeState::Creating,
            health: BiomeHealth {
                status: HealthStatus::Unknown,
                primal_health: HashMap::new(),
                last_check: Utc::now(),
                issues: Vec::new(),
                metrics: BiomeMetrics {
                    total_resources: ResourceUsage {
                        cpu_percent: 0.0,
                        memory_bytes: 0,
                        disk_bytes: 0,
                        network_sent_bytes: 0,
                        network_received_bytes: 0,
                    },
                    requests_per_second: 0.0,
                    avg_response_time_ms: 0.0,
                    error_rate_percent: 0.0,
                    uptime_seconds: 0,
                },
            },

            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Add a Primal to this biome
    pub fn add_primal(&mut self, primal: PrimalInstance) {
        self.primals.insert(primal.primal_type.clone(), primal);
        self.updated_at = Utc::now();
    }

    /// Get a Primal by type
    pub fn get_primal(&self, primal_type: &PrimalType) -> Option<&PrimalInstance> {
        self.primals.get(primal_type)
    }

    /// Update biome state
    pub fn set_state(&mut self, state: BiomeState) {
        self.state = state;
        self.updated_at = Utc::now();
    }

    /// Check if biome is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.health.status, HealthStatus::Healthy)
    }

    /// Get all running Primals
    pub fn running_primals(&self) -> Vec<&PrimalInstance> {
        self.primals
            .values()
            .filter(|p| matches!(p.state, PrimalState::Running))
            .collect()
    }

    /// Get all failed Primals
    pub fn failed_primals(&self) -> Vec<&PrimalInstance> {
        self.primals
            .values()
            .filter(|p| matches!(p.state, PrimalState::Failed { .. }))
            .collect()
    }
}

impl PrimalInstance {
    /// Create a new Primal instance
    pub fn new(
        id: PrimalId,
        primal_type: PrimalType,
        name: String,
        config: serde_json::Value,
    ) -> Self {
        Self {
            id,
            primal_type,
            name,
            config,
            state: PrimalState::Initializing,
            health: HealthStatus::Unknown,
            endpoints: Vec::new(),
            capabilities: Vec::new(),
            resources: ResourceUsage {
                cpu_percent: 0.0,
                memory_bytes: 0,
                disk_bytes: 0,
                network_sent_bytes: 0,
                network_received_bytes: 0,
            },
        }
    }

    /// Update Primal state
    pub fn set_state(&mut self, state: PrimalState) {
        self.state = state;
    }

    /// Add an endpoint
    pub fn add_endpoint(&mut self, endpoint: PrimalEndpoint) {
        self.endpoints.push(endpoint);
    }

    /// Check if Primal is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.health, HealthStatus::Healthy)
    }

    /// Check if Primal is running
    pub fn is_running(&self) -> bool {
        matches!(self.state, PrimalState::Running)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biome_creation() {
        let biome = Biome::new("test-biome".to_string(), "A test biome".to_string());

        assert_eq!(biome.name, "test-biome");
        assert_eq!(biome.description, "A test biome");
        assert!(matches!(biome.state, BiomeState::Creating));
        assert!(biome.primals.is_empty());
    }

    #[test]
    fn test_primal_instance_creation() {
        let primal = PrimalInstance::new(
            "toadstool-001".to_string(),
            "toadstool".to_string(),
            "Toadstool Runtime".to_string(),
            serde_json::json!({}),
        );

        assert_eq!(primal.id, "toadstool-001");
        assert_eq!(primal.primal_type, "toadstool");
        assert!(matches!(primal.state, PrimalState::Initializing));
        assert!(primal.endpoints.is_empty());
    }

    #[test]
    fn test_biome_add_primal() {
        let mut biome = Biome::new("test-biome".to_string(), "A test biome".to_string());

        let primal = PrimalInstance::new(
            "toadstool-001".to_string(),
            "toadstool".to_string(),
            "Toadstool Runtime".to_string(),
            serde_json::json!({}),
        );

        biome.add_primal(primal);
        assert_eq!(biome.primals.len(), 1);
        assert!(biome.get_primal(&"toadstool".to_string()).is_some());
    }
}
