//! Primal abstraction and plugin system
//!
//! This module defines the core Primal abstraction that allows biomeOS to be
//! completely agnostic about which Primals exist. Any system that implements
//! the Primal trait can participate in the biomeOS ecosystem.

use crate::{PrimalId, BiomeResult, BiomeError, HealthStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Universal Primal identifier - can be any string
pub type PrimalType = String;

/// Capability that a Primal can provide
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Capability {
    /// Capability name (e.g., "storage.zfs", "orchestration.service", "ai.inference")
    pub name: String,
    /// Version of this capability
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// Optional parameters this capability accepts
    pub parameters: HashMap<String, CapabilityParameter>,
}

/// Parameter definition for a capability
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityParameter {
    /// Parameter type (string, number, boolean, array, object)
    pub param_type: String,
    /// Whether this parameter is required
    pub required: bool,
    /// Default value if not provided
    pub default: Option<serde_json::Value>,
    /// Human-readable description
    pub description: String,
    /// Validation rules (regex, min/max, enum values, etc.)
    pub validation: Option<HashMap<String, serde_json::Value>>,
}

/// Universal Primal trait - any system can implement this
#[async_trait]
pub trait Primal: Send + Sync {
    /// Unique identifier for this primal instance
    fn id(&self) -> PrimalId;
    
    /// Type identifier for this primal (e.g., "toadstool", "songbird", "custom-ai")
    fn primal_type(&self) -> PrimalType;
    
    /// Human-readable name
    fn name(&self) -> String;
    
    /// Version of this primal implementation
    fn version(&self) -> String;
    
    /// Capabilities this primal provides
    fn capabilities(&self) -> Vec<Capability>;
    
    /// Dependencies this primal requires from other primals
    fn dependencies(&self) -> Vec<PrimalDependency>;
    
    /// Initialize the primal with configuration
    async fn initialize(&mut self, config: PrimalConfig) -> BiomeResult<()>;
    
    /// Start the primal services
    async fn start(&mut self) -> BiomeResult<()>;
    
    /// Stop the primal services gracefully
    async fn stop(&mut self) -> BiomeResult<()>;
    
    /// Check health status
    async fn health_check(&self) -> BiomeResult<HealthStatus>;
    
    /// Execute a capability request
    async fn execute_capability(
        &self,
        capability: &str,
        request: CapabilityRequest,
    ) -> BiomeResult<CapabilityResponse>;
    
    /// Handle events from other primals
    async fn handle_event(&mut self, event: PrimalEvent) -> BiomeResult<()>;
    
    /// Get current metrics/telemetry
    async fn get_metrics(&self) -> BiomeResult<PrimalMetrics>;
}

/// Dependency that a primal requires
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalDependency {
    /// Required capability name
    pub capability: String,
    /// Minimum version required
    pub min_version: String,
    /// Whether this dependency is optional
    pub optional: bool,
    /// Fallback behavior if dependency unavailable
    pub fallback: Option<String>,
}

/// Configuration for a primal instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalConfig {
    /// Instance-specific settings
    pub settings: HashMap<String, serde_json::Value>,
    /// Network configuration
    pub network: NetworkConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Resource limits
    pub resources: ResourceLimits,
}

/// Network configuration for a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Host to bind to
    pub host: String,
    /// Port to bind to (if applicable)
    pub port: Option<u16>,
    /// TLS configuration
    pub tls: Option<TlsConfig>,
    /// Discovery endpoints for finding other primals
    pub discovery: Vec<String>,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Path to certificate file
    pub cert_path: String,
    /// Path to private key file
    pub key_path: String,
    /// Path to CA certificate file
    pub ca_path: Option<String>,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Authentication method
    pub auth_method: String,
    /// API keys or tokens
    pub credentials: HashMap<String, String>,
    /// Access control rules
    pub access_rules: Vec<AccessRule>,
}

/// Access control rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRule {
    /// Subject (user, service, primal)
    pub subject: String,
    /// Action (read, write, execute)
    pub action: String,
    /// Resource pattern
    pub resource: String,
    /// Allow or deny
    pub effect: String,
}

/// Resource limits for a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in MB
    pub max_memory_mb: Option<u64>,
    /// Maximum CPU usage (0.0-1.0)
    pub max_cpu: Option<f64>,
    /// Maximum disk usage in MB
    pub max_disk_mb: Option<u64>,
    /// Maximum network bandwidth in Mbps
    pub max_bandwidth_mbps: Option<u64>,
}

/// Request to execute a capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRequest {
    /// Request ID for tracking
    pub request_id: Uuid,
    /// Parameters for the capability
    pub parameters: HashMap<String, serde_json::Value>,
    /// Context information
    pub context: RequestContext,
}

/// Context for a capability request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    /// ID of the requesting primal
    pub requesting_primal: PrimalId,
    /// User or service making the request
    pub principal: String,
    /// Request timestamp
    pub timestamp: DateTime<Utc>,
    /// Correlation ID for request tracing
    pub correlation_id: Option<String>,
}

/// Response from executing a capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityResponse {
    /// Request ID this response corresponds to
    pub request_id: Uuid,
    /// Success status
    pub success: bool,
    /// Response data
    pub data: Option<serde_json::Value>,
    /// Error message if failed
    pub error: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Event between primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEvent {
    /// Event ID
    pub id: Uuid,
    /// Event type
    pub event_type: String,
    /// Source primal
    pub source: PrimalId,
    /// Target primal (None for broadcast)
    pub target: Option<PrimalId>,
    /// Event payload
    pub payload: serde_json::Value,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
}

/// Metrics from a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalMetrics {
    /// Primal ID
    pub primal_id: PrimalId,
    /// Timestamp of metrics collection
    pub timestamp: DateTime<Utc>,
    /// Resource usage metrics
    pub resources: ResourceMetrics,
    /// Performance metrics
    pub performance: PerformanceMetrics,
    /// Custom metrics specific to this primal
    pub custom: HashMap<String, serde_json::Value>,
}

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// Current memory usage in MB
    pub memory_mb: f64,
    /// Current CPU usage (0.0-1.0)
    pub cpu_usage: f64,
    /// Current disk usage in MB
    pub disk_mb: f64,
    /// Current network bandwidth usage in Mbps
    pub bandwidth_mbps: f64,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Requests per second
    pub requests_per_second: f64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Error rate (0.0-1.0)
    pub error_rate: f64,
    /// Number of active connections
    pub active_connections: u64,
}

/// Registry for discovering and managing primals
pub struct PrimalRegistry {
    /// Known primal types and their factories
    factories: HashMap<PrimalType, Box<dyn PrimalFactory>>,
    /// Running primal instances
    instances: HashMap<PrimalId, Box<dyn Primal>>,
}

/// Factory for creating primal instances
#[async_trait]
pub trait PrimalFactory: Send + Sync {
    /// Create a new instance of this primal type
    async fn create_primal(&self, config: PrimalConfig) -> BiomeResult<Box<dyn Primal>>;
    
    /// Get metadata about this primal type
    fn get_metadata(&self) -> PrimalTypeMetadata;
}

/// Metadata about a primal type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalTypeMetadata {
    /// Primal type identifier
    pub primal_type: PrimalType,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: String,
    /// Version
    pub version: String,
    /// Author/maintainer
    pub author: String,
    /// Capabilities this type provides
    pub capabilities: Vec<Capability>,
    /// Configuration schema
    pub config_schema: serde_json::Value,
}

impl PrimalRegistry {
    /// Create a new primal registry
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
            instances: HashMap::new(),
        }
    }
    
    /// Register a primal factory
    pub fn register_factory(
        &mut self,
        primal_type: PrimalType,
        factory: Box<dyn PrimalFactory>,
    ) {
        self.factories.insert(primal_type, factory);
    }
    
    /// Create a new primal instance
    pub async fn create_primal(
        &mut self,
        primal_type: &str,
        config: PrimalConfig,
    ) -> BiomeResult<PrimalId> {
        let factory = self
            .factories
            .get(primal_type)
            .ok_or_else(|| BiomeError::PrimalNotFound(primal_type.to_string()))?;
        
        let mut primal = factory.create_primal(config.clone()).await?;
        primal.initialize(config).await?;
        
        let primal_id = primal.id().clone();
        self.instances.insert(primal_id.clone(), primal);
        
        Ok(primal_id)
    }
    
    /// Get a reference to a running primal
    pub fn get_primal(&self, primal_id: &PrimalId) -> Option<&dyn Primal> {
        self.instances.get(primal_id).map(|p| p.as_ref())
    }
    
    /// Get all available primal types
    pub fn get_available_types(&self) -> Vec<PrimalTypeMetadata> {
        self.factories.values()
            .map(|f| f.get_metadata())
            .collect()
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            auth_method: "none".to_string(),
            credentials: HashMap::new(),
            access_rules: Vec::new(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: None,
            tls: None,
            discovery: Vec::new(),
        }
    }
} 