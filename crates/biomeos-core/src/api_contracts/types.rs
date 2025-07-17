//! Core types and data structures for API contracts

use crate::{HealthMetrics, HealthStatus, ResourceUtilization};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Standard response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Response status
    pub status: ResponseStatus,
    /// Response data
    pub data: Option<T>,
    /// Error information
    pub error: Option<ApiError>,
    /// Response metadata
    pub metadata: ResponseMetadata,
}

/// Response status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseStatus {
    /// Success
    Success,
    /// Partial success
    PartialSuccess,
    /// Error
    Error,
    /// Timeout
    Timeout,
    /// Unauthorized
    Unauthorized,
    /// Forbidden
    Forbidden,
    /// Not found
    NotFound,
    /// Internal error
    InternalError,
}

/// API error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Error details
    pub details: Option<serde_json::Value>,
    /// Error trace ID
    pub trace_id: Option<String>,
    /// Error category
    pub category: ErrorCategory,
    /// Retry information
    pub retry_info: Option<RetryInfo>,
}

/// Error categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Validation error
    Validation,
    /// Authentication error
    Authentication,
    /// Authorization error
    Authorization,
    /// Resource not found
    NotFound,
    /// Resource conflict
    Conflict,
    /// Rate limit exceeded
    RateLimit,
    /// Service unavailable
    ServiceUnavailable,
    /// Internal server error
    InternalError,
    /// Network error
    NetworkError,
    /// Timeout error
    Timeout,
    /// Configuration error
    Configuration,
    /// Resource exhausted
    ResourceExhausted,
}

/// Retry information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryInfo {
    /// Whether retry is recommended
    pub retry_recommended: bool,
    /// Retry after seconds
    pub retry_after_seconds: Option<u64>,
    /// Maximum retry attempts
    pub max_retries: Option<u32>,
    /// Retry strategy
    pub retry_strategy: Option<RetryStrategy>,
}

/// Retry strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryStrategy {
    /// Fixed delay
    FixedDelay { delay_seconds: u64 },
    /// Exponential backoff
    ExponentialBackoff { base_delay: u64, max_delay: u64 },
    /// Linear backoff
    LinearBackoff { increment: u64, max_delay: u64 },
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// Request ID
    pub request_id: String,
    /// Timestamp
    pub timestamp: u64,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// API version
    pub api_version: String,
    /// Primal identifier
    pub primal_id: String,
    /// Rate limit information
    pub rate_limit: Option<RateLimitInfo>,
}

/// Rate limit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    /// Rate limit
    pub limit: u64,
    /// Remaining requests
    pub remaining: u64,
    /// Reset time
    pub reset_time: u64,
}

/// Endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointInfo {
    /// Endpoint name
    pub name: String,
    /// Endpoint URL
    pub url: String,
    /// HTTP method
    pub method: String,
    /// Endpoint description
    pub description: String,
    /// Request schema
    pub request_schema: Option<serde_json::Value>,
    /// Response schema
    pub response_schema: Option<serde_json::Value>,
    /// Authentication required
    pub auth_required: bool,
    /// Rate limit
    pub rate_limit: Option<RateLimitInfo>,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    /// Dependency name
    pub name: String,
    /// Dependency type
    pub dependency_type: DependencyType,
    /// Dependency version
    pub version: String,
    /// Whether dependency is optional
    pub optional: bool,
    /// Dependency status
    pub status: DependencyStatus,
}

/// Dependency types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    /// Primal dependency
    Primal,
    /// Service dependency
    Service,
    /// Library dependency
    Library,
    /// External service dependency
    ExternalService,
}

/// Dependency status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyStatus {
    /// Available
    Available,
    /// Unavailable
    Unavailable,
    /// Degraded
    Degraded,
    /// Unknown
    Unknown,
}

/// Component health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Component status
    pub status: HealthStatus,
    /// Component health score
    pub health_score: f64,
    /// Component metrics
    pub metrics: Option<HealthMetrics>,
    /// Component last check
    pub last_check: u64,
    /// Component details
    pub details: HashMap<String, serde_json::Value>,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Check name
    pub name: String,
    /// Check status
    pub status: HealthStatus,
    /// Check message
    pub message: String,
    /// Check duration
    pub duration_ms: u64,
    /// Check timestamp
    pub timestamp: u64,
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// System uptime
    pub uptime: u64,
    /// System load
    pub load: f64,
    /// Memory usage
    pub memory_usage: ResourceUtilization,
    /// CPU usage
    pub cpu_usage: f64,
    /// Disk usage
    pub disk_usage: ResourceUtilization,
    /// Network metrics
    pub network_metrics: HashMap<String, f64>,
}

/// Capabilities information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInfo {
    /// Capability name
    pub name: String,
    /// Capability version
    pub version: String,
    /// Capability description
    pub description: String,
    /// Capability enabled
    pub enabled: bool,
    /// Capability configuration
    pub configuration: HashMap<String, serde_json::Value>,
}

/// Resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    /// Resource name
    pub name: String,
    /// Resource type
    pub resource_type: String,
    /// Resource status
    pub status: String,
    /// Resource utilization
    pub utilization: ResourceUtilization,
    /// Resource limits
    pub limits: HashMap<String, f64>,
    /// Resource metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Operation priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationPriority {
    /// Low priority
    Low,
    /// Normal priority
    Normal,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

/// Retry policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum retries
    pub max_retries: u32,
    /// Retry strategy
    pub strategy: RetryStrategy,
    /// Retry conditions
    pub retry_conditions: Vec<RetryCondition>,
}

/// Retry condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryCondition {
    /// Error category
    pub error_category: ErrorCategory,
    /// Retry recommended
    pub retry_recommended: bool,
}

/// Tracing options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingOptions {
    /// Trace ID
    pub trace_id: String,
    /// Span ID
    pub span_id: String,
    /// Sampling rate
    pub sampling_rate: f64,
}

/// Message type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// Request message
    Request,
    /// Response message
    Response,
    /// Event message
    Event,
    /// Notification message
    Notification,
}

/// Log level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    /// Trace
    Trace,
    /// Debug
    Debug,
    /// Info
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
    /// Critical
    Critical,
}

/// Time range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    /// Start time
    pub start: u64,
    /// End time
    pub end: u64,
}

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Log timestamp
    pub timestamp: u64,
    /// Log level
    pub level: LogLevel,
    /// Log component
    pub component: String,
    /// Log message
    pub message: String,
    /// Log metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Metrics aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsAggregation {
    /// Aggregation type
    pub aggregation_type: AggregationType,
    /// Aggregation interval
    pub interval: Option<u64>,
    /// Grouping
    pub group_by: Option<Vec<String>>,
}

/// Aggregation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationType {
    /// Sum
    Sum,
    /// Average
    Average,
    /// Count
    Count,
    /// Maximum
    Maximum,
    /// Minimum
    Minimum,
    /// Percentile
    Percentile { percentile: f64 },
}

/// Metric data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricData {
    /// Metric name
    pub name: String,
    /// Metric values
    pub values: Vec<MetricValue>,
    /// Metric labels
    pub labels: HashMap<String, String>,
}

/// Metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricValue {
    /// Value timestamp
    pub timestamp: u64,
    /// Value
    pub value: f64,
}

/// Metrics metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsMetadata {
    /// Query duration
    pub query_duration_ms: u64,
    /// Data points
    pub data_points: u64,
    /// Sampling rate
    pub sampling_rate: f64,
}

/// Configuration metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationMetadata {
    /// Configuration version
    pub version: String,
    /// Last updated
    pub last_updated: u64,
    /// Configuration source
    pub source: String,
    /// Configuration validation
    pub validation: ConfigurationValidation,
}

/// Configuration validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationValidation {
    /// Validation status
    pub status: ValidationStatus,
    /// Validation errors
    pub errors: Vec<ValidationError>,
    /// Validation warnings
    pub warnings: Vec<ValidationWarning>,
}

/// Validation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStatus {
    /// Valid
    Valid,
    /// Invalid
    Invalid,
    /// Warning
    Warning,
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error field
    pub field: String,
    /// Error message
    pub message: String,
    /// Error code
    pub code: String,
}

/// Validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning field
    pub field: String,
    /// Warning message
    pub message: String,
    /// Warning code
    pub code: String,
}

/// Update metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMetadata {
    /// Updated by
    pub updated_by: String,
    /// Update reason
    pub reason: String,
    /// Update timestamp
    pub timestamp: u64,
}

/// Validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Rule condition
    pub condition: String,
    /// Rule action
    pub action: ValidationAction,
}

/// Validation action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationAction {
    /// Allow
    Allow,
    /// Warn
    Warn,
    /// Reject
    Reject,
    /// Transform
    Transform { transformation: String },
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Validation passed
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<ValidationError>,
    /// Validation warnings
    pub warnings: Vec<ValidationWarning>,
}
