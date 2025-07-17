//! Universal Primal Communication Protocol
//!
//! This module defines the standard communication protocol that any primal
//! can implement to work with biomeOS. It provides a universal interface
//! that works regardless of the specific primal implementation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard API endpoints that any primal should implement
pub const STANDARD_ENDPOINTS: &[&str] = &[
    "/api/v1/health",
    "/api/v1/capabilities",
    "/api/v1/status",
    "/api/v1/operation",
    "/api/v1/events",
    "/api/v1/metrics",
];

/// Standard health response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardHealthResponse {
    /// Health status
    pub status: String, // "healthy", "degraded", "unhealthy"
    /// Health score (0.0-1.0)
    pub health_score: f64,
    /// Last check timestamp
    pub last_check: String, // ISO 8601 format
    /// Detailed health information
    pub details: HashMap<String, serde_json::Value>,
    /// Performance metrics
    pub metrics: StandardHealthMetrics,
}

/// Standard health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardHealthMetrics {
    /// CPU usage percentage (0.0-100.0)
    pub cpu_usage: f64,
    /// Memory usage in MB
    pub memory_mb: f64,
    /// Response time in milliseconds
    pub response_time_ms: f64,
    /// Error rate (0.0-1.0)
    pub error_rate: f64,
    /// Active connections
    pub active_connections: u64,
}

/// Standard capabilities response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardCapabilitiesResponse {
    /// List of capabilities
    pub capabilities: Vec<StandardCapability>,
    /// Primal metadata
    pub metadata: StandardPrimalMetadata,
}

/// Standard capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardCapability {
    /// Capability name
    pub name: String,
    /// Capability version
    pub version: String,
    /// Capability type/category
    pub capability_type: String,
    /// Human-readable description
    pub description: String,
    /// Parameters this capability accepts
    pub parameters: HashMap<String, StandardParameter>,
    /// Current status
    pub status: String, // "active", "inactive", "deprecated"
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Standard parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardParameter {
    /// Parameter type
    pub param_type: String, // "string", "number", "boolean", "object", "array"
    /// Whether required
    pub required: bool,
    /// Default value
    pub default: Option<serde_json::Value>,
    /// Description
    pub description: String,
    /// Validation rules
    pub validation: Option<HashMap<String, serde_json::Value>>,
}

/// Standard primal metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardPrimalMetadata {
    /// Primal type (e.g., "toadstool", "songbird", "custom-ai")
    pub primal_type: String,
    /// Primal version
    pub version: String,
    /// Instance identifier
    pub instance_id: String,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: String,
    /// Supported protocols
    pub supported_protocols: Vec<String>,
    /// Endpoints
    pub endpoints: StandardEndpoints,
    /// Dependencies
    pub dependencies: Vec<StandardDependency>,
}

/// Standard endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardEndpoints {
    /// Base URL
    pub base_url: String,
    /// Health check endpoint
    pub health: String,
    /// Capabilities endpoint
    pub capabilities: String,
    /// Status endpoint
    pub status: String,
    /// Operation execution endpoint
    pub operation: String,
    /// Events endpoint
    pub events: String,
    /// Metrics endpoint
    pub metrics: String,
    /// WebSocket endpoint (if supported)
    pub websocket: Option<String>,
    /// tRPC endpoint (if supported)
    pub trpc: Option<String>,
}

/// Standard dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardDependency {
    /// Dependency name
    pub name: String,
    /// Required capability
    pub capability: String,
    /// Minimum version
    pub min_version: String,
    /// Whether optional
    pub optional: bool,
    /// Fallback strategy
    pub fallback: Option<String>,
}

/// Standard operation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardOperationRequest {
    /// Request ID
    pub request_id: String,
    /// Operation name
    pub operation: String,
    /// Operation parameters
    pub payload: serde_json::Value,
    /// Request context
    pub context: StandardContext,
    /// Request metadata
    pub metadata: HashMap<String, String>,
    /// Request priority
    pub priority: String, // "low", "normal", "high", "critical"
}

/// Standard operation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardOperationResponse {
    /// Request ID
    pub request_id: String,
    /// Success status
    pub success: bool,
    /// Response payload
    pub payload: serde_json::Value,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Response metadata
    pub metadata: HashMap<String, String>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Standard context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardContext {
    /// User ID
    pub user_id: String,
    /// Team ID
    pub team_id: String,
    /// Device ID
    pub device_id: String,
    /// Security level
    pub security_level: String, // "low", "standard", "high", "maximum"
    /// Additional context
    pub additional_context: HashMap<String, serde_json::Value>,
}

/// Standard event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardEvent {
    /// Event ID
    pub id: String,
    /// Event type
    pub event_type: String,
    /// Event payload
    pub payload: serde_json::Value,
    /// Event context
    pub context: StandardContext,
    /// Event timestamp (ISO 8601)
    pub timestamp: String,
    /// Event metadata
    pub metadata: HashMap<String, String>,
}

/// Standard status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardStatusResponse {
    /// Overall status
    pub status: String, // "running", "starting", "stopping", "stopped", "error"
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Version information
    pub version: String,
    /// Configuration summary
    pub config: HashMap<String, serde_json::Value>,
    /// Resource usage
    pub resources: StandardResourceUsage,
    /// Active connections
    pub connections: u64,
    /// Last activity timestamp
    pub last_activity: String,
}

/// Standard resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardResourceUsage {
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// Disk usage in bytes
    pub disk_bytes: u64,
    /// Network bytes received
    pub network_rx_bytes: u64,
    /// Network bytes transmitted
    pub network_tx_bytes: u64,
}

/// Protocol validation
pub struct ProtocolValidator;

impl ProtocolValidator {
    /// Validate health response
    pub fn validate_health_response(response: &serde_json::Value) -> Result<(), String> {
        // Check required fields
        if !response["status"].is_string() {
            return Err("Missing or invalid 'status' field".to_string());
        }

        if !response["health_score"].is_number() {
            return Err("Missing or invalid 'health_score' field".to_string());
        }

        if !response["metrics"].is_object() {
            return Err("Missing or invalid 'metrics' field".to_string());
        }

        // Validate health score range
        if let Some(score) = response["health_score"].as_f64() {
            if !(0.0..=1.0).contains(&score) {
                return Err("Health score must be between 0.0 and 1.0".to_string());
            }
        }

        Ok(())
    }

    /// Validate capabilities response
    pub fn validate_capabilities_response(response: &serde_json::Value) -> Result<(), String> {
        if !response["capabilities"].is_array() {
            return Err("Missing or invalid 'capabilities' field".to_string());
        }

        if !response["metadata"].is_object() {
            return Err("Missing or invalid 'metadata' field".to_string());
        }

        Ok(())
    }

    /// Validate operation request
    pub fn validate_operation_request(request: &serde_json::Value) -> Result<(), String> {
        let required_fields = ["request_id", "operation", "payload", "context"];

        for field in required_fields {
            if request[field].is_null() {
                return Err(format!("Missing required field: {}", field));
            }
        }

        Ok(())
    }

    /// Validate operation response
    pub fn validate_operation_response(response: &serde_json::Value) -> Result<(), String> {
        if !response["request_id"].is_string() {
            return Err("Missing or invalid 'request_id' field".to_string());
        }

        if !response["success"].is_boolean() {
            return Err("Missing or invalid 'success' field".to_string());
        }

        if response["payload"].is_null() {
            return Err("Missing 'payload' field".to_string());
        }

        Ok(())
    }
}

/// Protocol constants
pub mod constants {
    /// Standard API version
    pub const API_VERSION: &str = "v1";

    /// Standard HTTP methods
    pub const HTTP_METHODS: &[&str] = &["GET", "POST", "PUT", "DELETE", "PATCH"];

    /// Standard content types
    pub const CONTENT_TYPE_JSON: &str = "application/json";
    pub const CONTENT_TYPE_FORM: &str = "application/x-www-form-urlencoded";

    /// Standard headers
    pub const HEADER_REQUEST_ID: &str = "X-Request-ID";
    pub const HEADER_PRIMAL_TYPE: &str = "X-Primal-Type";
    pub const HEADER_API_VERSION: &str = "X-API-Version";
    pub const HEADER_CONTEXT: &str = "X-Context";

    /// Standard status codes
    pub const STATUS_HEALTHY: &str = "healthy";
    pub const STATUS_DEGRADED: &str = "degraded";
    pub const STATUS_UNHEALTHY: &str = "unhealthy";
    pub const STATUS_UNKNOWN: &str = "unknown";

    /// Standard priority levels
    pub const PRIORITY_LOW: &str = "low";
    pub const PRIORITY_NORMAL: &str = "normal";
    pub const PRIORITY_HIGH: &str = "high";
    pub const PRIORITY_CRITICAL: &str = "critical";

    /// Standard security levels
    pub const SECURITY_LOW: &str = "low";
    pub const SECURITY_STANDARD: &str = "standard";
    pub const SECURITY_HIGH: &str = "high";
    pub const SECURITY_MAXIMUM: &str = "maximum";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_health_response_serialization() {
        let response = StandardHealthResponse {
            status: "healthy".to_string(),
            health_score: 0.95,
            last_check: "2025-01-15T10:30:00Z".to_string(),
            details: HashMap::new(),
            metrics: StandardHealthMetrics {
                cpu_usage: 25.5,
                memory_mb: 512.0,
                response_time_ms: 12.3,
                error_rate: 0.001,
                active_connections: 42,
            },
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: StandardHealthResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.status, "healthy");
        assert_eq!(deserialized.health_score, 0.95);
    }

    #[test]
    fn test_protocol_validation() {
        let valid_health = serde_json::json!({
            "status": "healthy",
            "health_score": 0.9,
            "metrics": {
                "cpu_usage": 10.0,
                "memory_mb": 256.0
            }
        });

        assert!(ProtocolValidator::validate_health_response(&valid_health).is_ok());

        let invalid_health = serde_json::json!({
            "status": "healthy",
            "health_score": 1.5, // Invalid score > 1.0
            "metrics": {}
        });

        assert!(ProtocolValidator::validate_health_response(&invalid_health).is_err());
    }
}
