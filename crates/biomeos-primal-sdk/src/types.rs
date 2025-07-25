//! Primal SDK Types
//!
//! Core types for the biomeOS Primal SDK

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Primal health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrimalHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl PrimalHealth {
    /// Create a healthy status
    pub fn healthy() -> Self {
        Self::Healthy
    }

    /// Create a degraded status
    pub fn degraded() -> Self {
        Self::Degraded
    }

    /// Create an unhealthy status
    pub fn unhealthy() -> Self {
        Self::Unhealthy
    }

    /// Create an unknown status
    pub fn unknown() -> Self {
        Self::Unknown
    }

    /// Check if healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }
}

/// Universal primal type system - NO HARDCODED NAMES
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimalType {
    /// Service category (compute, storage, security, etc.)
    pub category: String,

    /// Service name (discovered dynamically)
    pub name: String,

    /// Service version
    pub version: String,

    /// Additional metadata for classification
    pub metadata: HashMap<String, String>,
}

impl PrimalType {
    /// Create a new primal type
    pub fn new(category: &str, name: &str, version: &str) -> Self {
        Self {
            category: category.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            metadata: HashMap::new(),
        }
    }

    /// Create with metadata
    pub fn with_metadata(
        category: &str,
        name: &str,
        version: &str,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            category: category.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            metadata,
        }
    }

    /// Check if this primal provides a specific category
    pub fn is_category(&self, category: &str) -> bool {
        self.category == category
    }

    /// Check if this primal has a specific name
    pub fn is_name(&self, name: &str) -> bool {
        self.name == name
    }

    // Convenience constructors for common primal types (replacing enum variants)
    pub fn toadstool() -> Self {
        Self::new("compute", "toadstool", "1.0.0")
    }

    pub fn songbird() -> Self {
        Self::new("orchestration", "songbird", "1.0.0")
    }

    pub fn nestgate() -> Self {
        Self::new("storage", "nestgate", "1.0.0")
    }

    pub fn beardog() -> Self {
        Self::new("security", "beardog", "1.0.0")
    }

    pub fn squirrel() -> Self {
        Self::new("ai", "squirrel", "1.0.0")
    }

    pub fn community(name: impl Into<String>, category: impl Into<String>) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("community".to_string(), "true".to_string());
        metadata.insert("category".to_string(), category.into());
        Self::with_metadata("community", &name.into(), "1.0.0", metadata)
    }

    // Check methods for type matching (replacing enum pattern matching)
    pub fn is_toadstool(&self) -> bool {
        self.name == "toadstool" && self.category == "compute"
    }

    pub fn is_songbird(&self) -> bool {
        self.name == "songbird" && self.category == "orchestration"
    }

    pub fn is_community(&self) -> bool {
        self.category == "community"
    }

    pub fn is_beardog(&self) -> bool {
        self.name == "beardog" && self.category == "security"
    }

    pub fn is_nestgate(&self) -> bool {
        self.name == "nestgate" && self.category == "storage"
    }

    pub fn is_squirrel(&self) -> bool {
        self.name == "squirrel" && self.category == "ai"
    }
}

/// Universal primal capabilities - fully extensible
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimalCapability {
    /// Capability domain (system, security, compute, etc.)
    pub domain: String,

    /// Specific capability name
    pub name: String,

    /// Capability version/level
    pub version: String,

    /// Capability parameters and constraints
    pub parameters: HashMap<String, serde_json::Value>,
}

impl PrimalCapability {
    /// Create a new capability
    pub fn new(domain: &str, name: &str, version: &str) -> Self {
        Self {
            domain: domain.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            parameters: HashMap::new(),
        }
    }

    /// Create with parameters
    pub fn with_parameters(
        domain: &str,
        name: &str,
        version: &str,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            domain: domain.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            parameters,
        }
    }

    /// Check if this capability matches a domain
    pub fn matches_domain(&self, domain: &str) -> bool {
        self.domain == domain
    }

    /// Check if this capability matches a specific name
    pub fn matches_name(&self, name: &str) -> bool {
        self.name == name
    }

    /// Check if this capability satisfies requirements
    pub fn satisfies(&self, required_domain: &str, required_name: &str) -> bool {
        self.domain == required_domain && self.name == required_name
    }

    // Convenience constructors for common capabilities
    pub fn system_management() -> Self {
        Self::new("system", "management", "v1")
    }

    pub fn storage_provider() -> Self {
        Self::new("storage", "provider", "v1")
    }

    pub fn compute_provider() -> Self {
        Self::new("compute", "provider", "v1")
    }

    // Additional static methods for backward compatibility
    pub fn orchestration(name: &str, description: &str) -> Self {
        let mut capability = Self::new("orchestration", name, "v1");
        capability.parameters.insert("description".to_string(), serde_json::Value::String(description.to_string()));
        capability
    }

    pub fn communication(name: &str, description: &str) -> Self {
        let mut capability = Self::new("communication", name, "v1");
        capability.parameters.insert("description".to_string(), serde_json::Value::String(description.to_string()));
        capability
    }
}

impl PrimalCapability {
    pub fn security_provider() -> Self {
        Self::new("security", "provider", "v1")
    }

    pub fn networking_provider() -> Self {
        Self::new("networking", "provider", "v1")
    }

    pub fn orchestration_provider() -> Self {
        Self::new("orchestration", "provider", "v1")
    }

    pub fn ai_provider() -> Self {
        Self::new("ai", "provider", "v1")
    }

    // Additional capabilities for examples
    pub fn gaming() -> Self {
        Self::new("entertainment", "gaming", "v1")
    }

    pub fn machine_learning() -> Self {
        Self::new("ai", "machine_learning", "v1")
    }

    pub fn iot() -> Self {
        Self::new("iot", "device_management", "v1")
    }

    pub fn device_management() -> Self {
        Self::new("system", "device_management", "v1")
    }

    pub fn process_management() -> Self {
        Self::new("system", "process_management", "v1")
    }

    pub fn encryption() -> Self {
        Self::new("security", "encryption", "v1")
    }

    pub fn authentication() -> Self {
        Self::new("security", "authentication", "v1")
    }

    pub fn authorization() -> Self {
        Self::new("security", "authorization", "v1")
    }

    pub fn key_management() -> Self {
        Self::new("security", "key_management", "v1")
    }

    pub fn service_discovery() -> Self {
        Self::new("networking", "service_discovery", "v1")
    }

    pub fn message_routing() -> Self {
        Self::new("networking", "message_routing", "v1")
    }

    pub fn load_balancing() -> Self {
        Self::new("orchestration", "load_balancing", "v1")
    }

    pub fn service_mesh() -> Self {
        Self::new("networking", "service_mesh", "v1")
    }

    pub fn network_management() -> Self {
        Self::new("networking", "network_management", "v1")
    }

    pub fn plugin_management() -> Self {
        Self::new("system", "plugin_management", "v1")
    }

    pub fn code_execution() -> Self {
        Self::new("compute", "code_execution", "v1")
    }

    pub fn sandboxing() -> Self {
        Self::new("security", "sandboxing", "v1")
    }

    // Custom capability constructor
    pub fn custom(name: impl Into<String>, description: impl Into<String>) -> Self {
        let mut capability = Self::new("custom", &name.into(), "v1");
        capability.parameters.insert(
            "description".to_string(),
            serde_json::Value::String(description.into()),
        );
        capability
    }
}

/// Primal configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalConfig {
    pub name: String,
    pub primal_type: PrimalType,
    pub capabilities: Vec<PrimalCapability>,
    pub configuration: HashMap<String, serde_json::Value>,
}

/// Primal metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub primal_type: PrimalType,
    pub capabilities: Vec<PrimalCapability>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub homepage: Option<String>,
    pub tags: Vec<String>,
    pub keywords: Vec<String>,
    pub min_biomeos_version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl PrimalMetadata {
    /// Create new metadata with minimal required fields
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        primal_type: PrimalType,
        capabilities: Vec<PrimalCapability>,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            name: name.into(),
            version: version.into(),
            description: description.into(),
            primal_type,
            capabilities,
            author: None,
            license: None,
            repository: None,
            documentation: None,
            homepage: None,
            tags: Vec::new(),
            keywords: Vec::new(),
            min_biomeos_version: "0.1.0".to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Builder pattern methods
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    pub fn with_license(mut self, license: impl Into<String>) -> Self {
        self.license = Some(license.into());
        self
    }

    pub fn with_homepage(mut self, homepage: impl Into<String>) -> Self {
        self.homepage = Some(homepage.into());
        self
    }

    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    pub fn with_min_biomeos_version(mut self, version: impl Into<String>) -> Self {
        self.min_biomeos_version = version.into();
        self
    }
}

/// Primal request format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRequest {
    pub request_id: uuid::Uuid,
    pub method: String,
    pub payload: serde_json::Value,
    pub metadata: HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source: Option<String>,
    pub timeout_ms: Option<u64>,
    pub required_capabilities: Vec<PrimalCapability>,
}

impl PrimalRequest {
    /// Create a new primal request
    pub fn new(method: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            request_id: uuid::Uuid::new_v4(),
            method: method.into(),
            payload,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
            source: None,
            timeout_ms: Some(30000), // Default 30s timeout
            required_capabilities: Vec::new(),
        }
    }

    /// Create with source identification
    pub fn with_source(
        method: impl Into<String>,
        payload: serde_json::Value,
        source: impl Into<String>,
    ) -> Self {
        let mut req = Self::new(method, payload);
        req.source = Some(source.into());
        req
    }

    /// Add required capability
    pub fn require_capability(mut self, capability: PrimalCapability) -> Self {
        self.required_capabilities.push(capability);
        self
    }
}

/// Primal response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResponse {
    pub request_id: uuid::Uuid,
    pub status: ResponseStatus,
    pub payload: serde_json::Value,
    pub metadata: HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub processing_time_ms: Option<u64>,
    pub provided_capabilities: Vec<PrimalCapability>,
}

impl PrimalResponse {
    /// Create a successful response
    pub fn success(request_id: uuid::Uuid, payload: serde_json::Value) -> Self {
        Self {
            request_id,
            status: ResponseStatus::Success,
            payload,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
            processing_time_ms: None,
            provided_capabilities: Vec::new(),
        }
    }

    /// Create an error response
    pub fn error(
        request_id: uuid::Uuid,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            request_id,
            status: ResponseStatus::Error {
                code: code.into(),
                message: message.into(),
            },
            payload: serde_json::Value::Null,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
            processing_time_ms: None,
            provided_capabilities: Vec::new(),
        }
    }

    /// Create with processing time
    pub fn success_with_timing(
        request_id: uuid::Uuid,
        payload: serde_json::Value,
        processing_time_ms: u64,
    ) -> Self {
        let mut response = Self::success(request_id, payload);
        response.processing_time_ms = Some(processing_time_ms);
        response
    }
}

/// Response status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseStatus {
    Success,
    Error { code: String, message: String },
    PartialSuccess,
}

/// Primal error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalError {
    pub code: String,
    pub message: String,
    pub details: Option<HashMap<String, serde_json::Value>>,
}

impl PrimalError {
    /// Create a new error with just a message
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    /// Create with details
    pub fn with_details(
        code: impl Into<String>,
        message: impl Into<String>,
        details: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: Some(details),
        }
    }

    /// Common error constructors
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new("INVALID_REQUEST", message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("NOT_FOUND", message)
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new("INTERNAL_ERROR", message)
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new("TIMEOUT", message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new("UNAUTHORIZED", message)
    }
}

impl std::fmt::Display for PrimalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for PrimalError {}

/// Standard result type for primal operations
pub type PrimalResult<T> = Result<T, PrimalError>;
