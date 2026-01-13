//! Primal handle types and metadata

use serde::{Deserialize, Serialize};
use std::fmt;

use super::schema::ApiSchema;

/// Unique identifier for a primal instance
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PrimalId(String);

impl PrimalId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PrimalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Handle to a discovered primal
#[derive(Debug, Clone)]
pub struct PrimalHandle {
    /// Unique identifier
    pub id: PrimalId,

    /// Human-readable name
    pub name: String,

    /// Available endpoints (in priority order)
    pub endpoints: Vec<Endpoint>,

    /// Advertised capabilities
    pub capabilities: Vec<String>,

    /// Parsed API schema (if available)
    pub schema: Option<ApiSchema>,

    /// Primary protocol
    pub protocol: String,

    /// Hint about response format
    pub format_hint: Option<FormatHint>,
}

impl PrimalHandle {
    /// Create a new primal handle
    pub fn new(id: PrimalId, name: String) -> Self {
        Self {
            id,
            name,
            endpoints: Vec::new(),
            capabilities: Vec::new(),
            schema: None,
            protocol: "http".to_string(),
            format_hint: None,
        }
    }

    /// Get primary endpoint
    pub fn primary_endpoint(&self) -> Option<&Endpoint> {
        self.endpoints.first()
    }

    /// Check if primal has capability
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }
}

/// Endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Endpoint {
    /// Full URL
    pub url: String,

    /// Protocol (http, https, tarpc, grpc, etc.)
    pub protocol: String,

    /// Priority (lower = higher priority)
    pub priority: u8,
}

impl Endpoint {
    pub fn new(url: impl Into<String>, protocol: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            protocol: protocol.into(),
            priority: 100,
        }
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}

/// Hint about primal's response format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormatHint {
    /// Wrapped in ApiResponse { success, data, error }
    Wrapped,

    /// Direct data, no wrapper
    Unwrapped,

    /// Uses HTTP status codes for success/failure
    StatusCodeBased,

    /// Unknown format, will auto-detect
    Unknown,
}

/// Primal metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalMetadata {
    /// Primal name
    pub name: String,

    /// Version string
    pub version: String,

    /// Capabilities with details
    pub capabilities: Vec<Capability>,

    /// API version
    pub api_version: String,

    /// URL to fetch schema
    pub schema_url: Option<String>,

    /// Health check endpoint
    pub health_endpoint: Option<String>,
}

/// Capability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Capability name (e.g., "security", "orchestration")
    pub name: String,

    /// Capability version
    pub version: String,

    /// Operations provided by this capability
    pub operations: Vec<String>,

    /// Additional metadata
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

impl Capability {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            operations: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_operations(mut self, operations: Vec<String>) -> Self {
        self.operations = operations;
        self
    }
}
