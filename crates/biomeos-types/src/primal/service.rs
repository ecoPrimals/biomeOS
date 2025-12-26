//! Primal Service System
//!
//! This module contains the UniversalPrimalService trait, request/response types,
//! service registration, and all service-related functionality.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{BiomeError, BiomeResult};
use crate::health::Health;

use super::capabilities::{CapabilityMetadata, PrimalCapability};
use super::configuration::{HealthCheckConfig, PrimalConfiguration};
use super::core::{PrimalType, ResourceRequirements};

/// Universal Primal Service Trait
///
/// This trait defines the standard interface that all primal services must implement
/// to participate in the biomeOS ecosystem.
#[async_trait::async_trait]
pub trait UniversalPrimalService: Send + Sync {
    // === Core Identity & Metadata ===

    /// Get the unique identifier for this primal service
    fn primal_id(&self) -> &str;

    /// Get the primal type classification
    fn primal_type(&self) -> &PrimalType;

    /// Get comprehensive service metadata
    fn metadata(&self) -> &PrimalServiceMetadata;

    /// Get the service version
    fn version(&self) -> &str;

    // === Capabilities Management ===

    /// Get all capabilities this service provides
    fn capabilities(&self) -> &[PrimalCapability];

    /// Check if this service can handle a specific capability
    async fn can_handle_capability(&self, capability: &PrimalCapability) -> bool;

    /// Get detailed metadata for a specific capability
    async fn get_capability_metadata(&self, capability: &str) -> Option<CapabilityMetadata>;

    // === Lifecycle Management ===

    /// Initialize the service with configuration
    async fn initialize(&mut self, config: &PrimalConfiguration) -> BiomeResult<()>;

    /// Gracefully shutdown the service
    async fn shutdown(&mut self) -> BiomeResult<()>;

    /// Update service configuration at runtime
    async fn update_configuration(&mut self, config: serde_json::Value) -> BiomeResult<()>;

    // === Request Handling ===

    /// Handle a universal service request
    async fn handle_request(&self, request: UniversalServiceRequest) -> UniversalServiceResponse;

    // === Health & Monitoring ===

    /// Get comprehensive health status
    async fn health_check(&self) -> BiomeResult<Health>;

    /// Get detailed health report with metrics
    async fn health_report(&self) -> BiomeResult<crate::health::HealthReport>;

    /// Get current resource usage metrics
    async fn resource_metrics(&self) -> BiomeResult<crate::ResourceMetrics>;

    // === Service Registration & Discovery ===

    /// Get service registration information
    fn get_registration(&self) -> UniversalServiceRegistration;

    /// Register with the ecosystem discovery system
    async fn register_with_ecosystem(&self, discovery_endpoint: &str) -> BiomeResult<()>;

    /// Notify ecosystem of status changes
    async fn notify_status_change(&self, status: ServiceStatus) -> BiomeResult<()>;

    // === Dynamic Configuration ===

    /// Get dynamic configuration that can change at runtime
    fn get_dynamic_config(&self) -> Option<serde_json::Value>;

    /// Validate a potential configuration change
    async fn validate_config_change(
        &self,
        config: &serde_json::Value,
    ) -> BiomeResult<ConfigValidationResult>;
}

/// Comprehensive primal service metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalServiceMetadata {
    /// Service identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Service description
    pub description: String,

    /// Service version
    pub version: String,

    /// Service author/organization
    pub author: String,

    /// Service homepage URL
    pub homepage: Option<String>,

    /// Service documentation URL
    pub documentation: Option<String>,

    /// Service license
    pub license: Option<String>,

    /// Service keywords for discovery
    pub keywords: Vec<String>,

    /// Service endpoints
    pub endpoints: HashMap<String, String>,

    /// Custom metadata
    pub custom: HashMap<String, serde_json::Value>,

    /// When this service was created
    pub created_at: DateTime<Utc>,

    /// When this service was last updated
    pub updated_at: DateTime<Utc>,
}

/// Universal service request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalServiceRequest {
    /// Unique request identifier
    pub request_id: Uuid,

    /// Request method/operation
    pub method: String,

    /// Request parameters
    pub parameters: HashMap<String, serde_json::Value>,

    /// Request payload (for complex data)
    pub payload: serde_json::Value,

    /// Request context and metadata
    pub context: ServiceRequestContext,

    /// Request timestamp
    pub timestamp: DateTime<Utc>,

    /// Required capabilities for this request
    pub required_capabilities: Vec<PrimalCapability>,

    /// Request timeout in milliseconds
    pub timeout_ms: Option<u64>,

    /// Request priority level
    pub priority: RequestPriority,
}

/// Service request context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequestContext {
    /// Source service or user
    pub source: Option<String>,

    /// User identifier
    pub user_id: Option<String>,

    /// Session identifier
    pub session_id: Option<Uuid>,

    /// Trace identifier for distributed tracing
    pub trace_id: Option<String>,

    /// Correlation identifier
    pub correlation_id: Option<Uuid>,

    /// Security context
    pub security: SecurityContext,

    /// Custom context metadata
    pub metadata: HashMap<String, String>,
}

/// Security context for requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Authentication token
    pub auth_token: Option<String>,

    /// User permissions
    pub permissions: Vec<String>,

    /// Access level
    pub access_level: AccessLevel,

    /// Security constraints
    pub constraints: HashMap<String, serde_json::Value>,
}

/// Access levels for security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessLevel {
    /// Public access, no authentication required
    Public,

    /// Authenticated user required
    Authenticated,

    /// Specific authorization required
    Authorized(Vec<String>),

    /// Administrative access required
    Administrative,

    /// System-level access required
    System,
}

/// Request priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
    Emergency = 5,
}

/// Universal service response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalServiceResponse {
    /// Response to request ID
    pub request_id: Uuid,

    /// Response status
    pub status: ResponseStatus,

    /// Response data
    pub data: serde_json::Value,

    /// Response metadata
    pub metadata: ServiceResponseMetadata,

    /// Response timestamp
    pub timestamp: DateTime<Utc>,

    /// Capabilities used to fulfill this request
    pub capabilities_used: Vec<PrimalCapability>,

    /// Error information if status indicates error
    pub error: Option<BiomeError>,
}

/// Service response metadata for universal responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceResponseMetadata {
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Resource usage during request processing
    pub resource_usage: HashMap<String, f64>,
    /// Non-critical warnings
    pub warnings: Vec<String>,
    /// Debug information (only in debug builds)
    pub debug_info: Option<String>,
    /// Custom metadata fields
    pub custom: HashMap<String, serde_json::Value>,
}

/// Response status codes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResponseStatus {
    /// Request completed successfully
    Success,

    /// Request accepted and processing asynchronously
    Accepted,

    /// Request partially successful
    PartialSuccess,

    /// Request failed with error
    Error,

    /// Request timed out
    Timeout,

    /// Request was cancelled
    Cancelled,

    /// Service temporarily unavailable
    Unavailable,

    /// Request not supported
    NotSupported,

    /// Request rate limited
    RateLimited,
}

/// Service registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalServiceRegistration {
    /// Service metadata
    pub metadata: PrimalServiceMetadata,

    /// Service capabilities
    pub capabilities: Vec<PrimalCapability>,

    /// Service endpoints
    pub endpoints: Vec<ServiceEndpoint>,

    /// Health check configuration
    pub health_check: HealthCheckConfig,

    /// Service constraints and requirements
    pub constraints: ServiceConstraints,

    /// Registration timestamp
    pub registered_at: DateTime<Utc>,
}

/// Service endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    /// Endpoint name/identifier
    pub name: String,

    /// Endpoint URL
    pub url: String,

    /// Endpoint protocol (HTTP, gRPC, WebSocket, etc.)
    pub protocol: String,

    /// Whether this endpoint is publicly accessible
    pub public: bool,

    /// Endpoint health status
    pub health: Health,
}

/// Service deployment constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConstraints {
    /// Minimum resource requirements
    pub min_resources: ResourceRequirements,
    /// Maximum resource limits
    pub max_resources: Option<ResourceRequirements>,
    /// Network requirements
    pub network: NetworkRequirements,
    /// Security requirements  
    pub security: SecurityRequirements,
    /// Additional deployment constraints
    pub deployment_constraints: HashMap<String, serde_json::Value>,
}

/// Network requirements for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequirements {
    /// Required network ports
    pub required_ports: Vec<u16>,
    /// Network security requirements
    pub security: NetworkSecurity,
    /// Bandwidth requirements per connection type
    pub bandwidth_requirements: HashMap<String, u64>,
}

/// Network security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSecurity {
    /// Whether TLS is required
    pub tls_required: bool,
    /// Allowed origins for CORS
    pub allowed_origins: Vec<String>,
    /// Rate limiting configuration
    pub rate_limiting: HashMap<String, u32>,
}

/// Security requirements for services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirements {
    /// Whether authentication is required
    pub authentication_required: bool,
    /// Required authorization scopes
    pub authorization_scopes: Vec<String>,
    /// Encryption requirements
    pub encryption: EncryptionRequirements,
    /// Compliance requirements
    pub compliance_requirements: Vec<String>,
}

/// Encryption requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionRequirements {
    /// Encryption at rest required
    pub at_rest: bool,
    /// Encryption in transit required
    pub in_transit: bool,
    /// Key management system
    pub key_management: Option<String>,
}

/// Service status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceStatus {
    /// Service is starting up
    Starting,

    /// Service is running and healthy
    Running,

    /// Service is running but degraded
    Degraded,

    /// Service is stopping
    Stopping,

    /// Service is stopped
    Stopped,

    /// Service encountered an error
    Error(String),

    /// Service is under maintenance
    Maintenance,
}

/// Configuration validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValidationResult {
    /// Whether the configuration is valid
    pub valid: bool,

    /// Validation errors
    pub errors: Vec<String>,

    /// Validation warnings
    pub warnings: Vec<String>,

    /// Suggested improvements
    pub suggestions: Vec<String>,
}
