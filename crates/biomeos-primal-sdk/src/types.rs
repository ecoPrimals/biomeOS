// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Primal SDK Types - UNIFIED TYPES IMPLEMENTATION
//!
//! ✅ UNIFICATION COMPLETE: Now uses the unified type system exclusively
//!
//! This module provides a clean interface to the unified types from biomeos-types
//! optimized for primal development with direct access to all capabilities.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Direct re-export of unified types - no compatibility layers
pub use biomeos_types::{
    // Core error and result types
    BiomeError,
    // Configuration
    BiomeOSConfig,
    BiomeResult,

    ComponentHealth,

    Environment,
    // Health system
    Health,
    HealthReport,
    NetworkIoMetrics,

    PrimalCapability,
    PrimalConfiguration,

    // Primal system types
    PrimalType,
    // Resource and metrics
    ResourceMetrics,
    SystemConfig,
    health::HealthMetrics,
};

/// Extended Request type with primal-specific features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRequest {
    /// Request unique identifier
    pub request_id: uuid::Uuid,

    /// Request method/operation
    pub method: String,

    /// Request payload
    pub payload: serde_json::Value,

    /// Request metadata and headers
    pub metadata: HashMap<String, String>,

    /// Request timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Source primal or service
    pub source: Option<String>,

    /// Request timeout in milliseconds
    pub timeout_ms: Option<u64>,

    /// Request priority level
    pub priority: RequestPriority,

    /// Correlation ID for request tracing
    pub correlation_id: Option<uuid::Uuid>,
}

/// Extended Response type with primal-specific features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResponse {
    /// Response to request ID
    pub request_id: uuid::Uuid,

    /// Response status
    pub status: ResponseStatus,

    /// Response payload
    pub payload: serde_json::Value,

    /// Response metadata and headers
    pub metadata: HashMap<String, String>,

    /// Response timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,

    /// Error information if status is error
    pub error: Option<BiomeError>,
}

/// Request priority levels for primal operations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestPriority {
    /// Low priority — background tasks, deferred operations
    Low = 1,
    /// Normal priority — standard user-initiated operations
    Normal = 2,
    /// High priority — time-sensitive operations
    High = 3,
    /// Critical priority — safety, security, or system-integrity operations
    Critical = 4,
}

/// Response status codes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResponseStatus {
    /// Request completed successfully
    Success,
    /// Request accepted and processing
    Accepted,
    /// Request failed with error
    Error,
    /// Request timed out
    Timeout,
    /// Request was cancelled
    Cancelled,
    /// Service temporarily unavailable
    Unavailable,
}

/// Metadata about a primal service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalMetadata {
    /// Primal identifier
    pub id: String,

    /// Primal type using unified system
    pub primal_type: PrimalType,

    /// Human-readable name
    pub name: String,

    /// Primal description
    pub description: String,

    /// Primal version
    pub version: String,

    /// Primal capabilities
    pub capabilities: Vec<PrimalCapability>,

    /// Supported API versions
    pub api_versions: Vec<String>,

    /// Service endpoints
    pub endpoints: HashMap<String, String>,

    /// Custom metadata
    pub custom: HashMap<String, serde_json::Value>,

    /// When this metadata was created
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// When this metadata was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Default for PrimalRequest {
    fn default() -> Self {
        Self {
            request_id: uuid::Uuid::new_v4(),
            method: String::new(),
            payload: serde_json::Value::Null,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
            source: None,
            timeout_ms: Some(30000), // 30 second default timeout
            priority: RequestPriority::Normal,
            correlation_id: None,
        }
    }
}

impl Default for PrimalResponse {
    fn default() -> Self {
        Self {
            request_id: uuid::Uuid::new_v4(),
            status: ResponseStatus::Success,
            payload: serde_json::Value::Null,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
            processing_time_ms: 0,
            error: None,
        }
    }
}

impl Default for PrimalMetadata {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            primal_type: PrimalType::community("unknown".to_string(), "generic".to_string()),
            name: "Unknown Primal".to_string(),
            description: "A primal service".to_string(),
            version: "0.1.0".to_string(),
            capabilities: vec![],
            api_versions: vec!["v1".to_string()],
            endpoints: HashMap::new(),
            custom: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

impl PrimalRequest {
    /// Create a new request with method
    pub fn new(method: impl Into<String>) -> Self {
        Self {
            method: method.into(),
            ..Default::default()
        }
    }

    /// Set request payload
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = payload;
        self
    }

    /// Set request priority
    pub fn with_priority(mut self, priority: RequestPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// Set source primal
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Add metadata entry
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set correlation ID for request tracing
    pub fn with_correlation_id(mut self, correlation_id: uuid::Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
}

impl PrimalResponse {
    /// Create a successful response
    pub fn success(request_id: uuid::Uuid, payload: serde_json::Value) -> Self {
        Self {
            request_id,
            status: ResponseStatus::Success,
            payload,
            timestamp: chrono::Utc::now(),
            ..Default::default()
        }
    }

    /// Create an error response
    pub fn error(request_id: uuid::Uuid, error: BiomeError) -> Self {
        Self {
            request_id,
            status: ResponseStatus::Error,
            error: Some(error),
            timestamp: chrono::Utc::now(),
            ..Default::default()
        }
    }

    /// Create an accepted response
    pub fn accepted(request_id: uuid::Uuid) -> Self {
        Self {
            request_id,
            status: ResponseStatus::Accepted,
            timestamp: chrono::Utc::now(),
            ..Default::default()
        }
    }

    /// Set processing time
    pub fn with_processing_time(mut self, processing_time_ms: u64) -> Self {
        self.processing_time_ms = processing_time_ms;
        self
    }

    /// Add metadata entry
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_priority_ordering() {
        assert!(RequestPriority::Critical > RequestPriority::Low);
        assert!(RequestPriority::High > RequestPriority::Normal);
        assert_eq!(RequestPriority::Low as i32, 1);
        assert_eq!(RequestPriority::Critical as i32, 4);
    }

    #[test]
    fn test_response_status_variants() {
        assert!(matches!(ResponseStatus::Success, ResponseStatus::Success));
        assert!(matches!(ResponseStatus::Error, ResponseStatus::Error));
        assert!(matches!(ResponseStatus::Timeout, ResponseStatus::Timeout));
    }

    #[test]
    fn test_primal_request_default() {
        let req = PrimalRequest::default();
        assert_eq!(req.priority, RequestPriority::Normal);
        assert_eq!(req.timeout_ms, Some(30000));
        assert!(req.request_id != uuid::Uuid::nil());
    }

    #[test]
    fn test_primal_request_new() {
        let req = PrimalRequest::new("health_check");
        assert_eq!(req.method, "health_check");
    }

    #[test]
    fn test_primal_request_builder() {
        let req = PrimalRequest::new("test")
            .with_payload(serde_json::json!({"key": "value"}))
            .with_priority(RequestPriority::High)
            .with_timeout(5000)
            .with_source("beardog")
            .with_metadata("trace", "123")
            .with_correlation_id(uuid::Uuid::new_v4());
        assert_eq!(req.method, "test");
        assert_eq!(req.priority, RequestPriority::High);
        assert_eq!(req.timeout_ms, Some(5000));
        assert_eq!(req.source.as_deref(), Some("beardog"));
    }

    #[test]
    fn test_primal_response_success() {
        let id = uuid::Uuid::new_v4();
        let resp = PrimalResponse::success(id, serde_json::json!({"ok": true}));
        assert_eq!(resp.request_id, id);
        assert_eq!(resp.status, ResponseStatus::Success);
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_primal_response_error() {
        let id = uuid::Uuid::new_v4();
        let err = BiomeError::internal_error("validation failed", None::<String>);
        let resp = PrimalResponse::error(id, err);
        assert_eq!(resp.status, ResponseStatus::Error);
        assert!(resp.error.is_some());
    }

    #[test]
    fn test_primal_response_accepted() {
        let id = uuid::Uuid::new_v4();
        let resp = PrimalResponse::accepted(id);
        assert_eq!(resp.status, ResponseStatus::Accepted);
    }

    #[test]
    fn test_primal_response_builder() {
        let resp = PrimalResponse::default()
            .with_processing_time(100)
            .with_metadata("key", "value");
        assert_eq!(resp.processing_time_ms, 100);
    }

    #[test]
    fn test_primal_metadata_default() {
        let meta = PrimalMetadata::default();
        assert_eq!(meta.name, "Unknown Primal");
        assert_eq!(meta.version, "0.1.0");
        assert!(meta.capabilities.is_empty());
    }

    #[test]
    fn test_helpers_health_check_request() {
        let req = helpers::health_check_request();
        assert_eq!(req.method, "health_check");
        assert_eq!(req.priority, RequestPriority::High);
        assert_eq!(req.timeout_ms, Some(5000));
    }

    #[test]
    fn test_helpers_capability_discovery_request() {
        let req = helpers::capability_discovery_request();
        assert_eq!(req.method, "get_capabilities");
    }

    #[test]
    fn test_helpers_healthy_response() {
        let id = uuid::Uuid::new_v4();
        let resp = helpers::healthy_response(id);
        assert_eq!(resp.request_id, id);
        assert_eq!(resp.status, ResponseStatus::Success);
    }

    #[test]
    fn test_helpers_degraded_response() {
        let id = uuid::Uuid::new_v4();
        let resp = helpers::degraded_response(id, &["issue1".to_string()]);
        assert_eq!(resp.request_id, id);
        let issues = resp
            .payload
            .get("issues")
            .and_then(|v| v.as_array())
            .expect("issues array");
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn test_helpers_critical_response() {
        let id = uuid::Uuid::new_v4();
        let resp = helpers::critical_response(id, &["critical".to_string()]);
        assert_eq!(resp.request_id, id);
        assert_eq!(
            resp.payload.get("health").and_then(|v| v.as_str()),
            Some("critical")
        );
    }

    #[test]
    fn test_types_serialization() {
        let req = PrimalRequest::new("test");
        let json = serde_json::to_string(&req).expect("serialize");
        let _: PrimalRequest = serde_json::from_str(&json).expect("deserialize");
    }
}

/// Helper functions for creating common primal operations
pub mod helpers {
    use super::{PrimalRequest, PrimalResponse, RequestPriority};

    /// Create a health check request
    pub fn health_check_request() -> PrimalRequest {
        PrimalRequest::new("health_check")
            .with_priority(RequestPriority::High)
            .with_timeout(5000) // 5 second timeout for health checks
    }

    /// Create a capability discovery request
    pub fn capability_discovery_request() -> PrimalRequest {
        PrimalRequest::new("get_capabilities")
            .with_priority(RequestPriority::Normal)
            .with_timeout(10000) // 10 second timeout
    }

    /// Create a configuration request
    pub fn get_config_request() -> PrimalRequest {
        PrimalRequest::new("get_configuration").with_priority(RequestPriority::Normal)
    }

    /// Create a successful health response
    pub fn healthy_response(request_id: uuid::Uuid) -> PrimalResponse {
        PrimalResponse::success(
            request_id,
            serde_json::json!({"health": "healthy", "status": "operational"}),
        )
    }

    /// Create a degraded health response with issues
    pub fn degraded_response(request_id: uuid::Uuid, issues: &[String]) -> PrimalResponse {
        PrimalResponse::success(
            request_id,
            serde_json::json!({
                "health": "degraded",
                "status": "degraded",
                "issues": issues.to_vec()
            }),
        )
    }

    /// Create a critical health response
    pub fn critical_response(request_id: uuid::Uuid, critical_issues: &[String]) -> PrimalResponse {
        PrimalResponse::success(
            request_id,
            serde_json::json!({
                "health": "critical",
                "status": "critical",
                "critical_issues": critical_issues.to_vec()
            }),
        )
    }
}
