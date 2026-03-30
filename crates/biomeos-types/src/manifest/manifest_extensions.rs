// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Manifest Extensions and Dependencies
//!
//! This module contains manifest extension types including `BiomeDependency`,
//! `ConfigSpec`, `SecretSpec`, health monitoring, and other extension specifications.

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Biome dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeDependency {
    /// Dependency name
    pub name: String,

    /// Version constraint
    pub version: Option<String>,

    /// Optional dependency
    pub optional: bool,

    /// Dependency source
    pub source: DependencySource,
}

/// Dependency sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencySource {
    /// Registry-based dependency
    Registry {
        /// Registry URL
        url: String,
    },
    /// Git repository dependency
    Git {
        /// Repository URL
        url: String,
        /// Branch name
        branch: Option<String>,
        /// Tag name
        tag: Option<String>,
        /// Commit hash
        commit: Option<String>,
    },
    /// Local filesystem dependency
    Local {
        /// Filesystem path
        path: String,
    },
}

/// Configuration specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSpec {
    /// Configuration data
    pub data: HashMap<String, String>,

    /// Binary data
    pub binary_data: HashMap<String, Bytes>,

    /// Immutable flag
    pub immutable: bool,
}

/// Secret specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretSpec {
    /// Secret type
    pub secret_type: SecretType,

    /// Secret data
    pub data: HashMap<String, Bytes>,

    /// String data
    pub string_data: HashMap<String, String>,

    /// Immutable flag
    pub immutable: bool,
}

/// Secret types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecretType {
    /// Opaque (generic) secret
    Opaque,
    /// Service account token
    ServiceAccountToken,
    /// Docker config JSON
    DockerConfigJson,
    /// Legacy Docker config
    DockerConfig,
    /// HTTP basic auth credentials
    BasicAuth,
    /// SSH authentication key
    SshAuth,
    /// TLS certificate and key
    Tls,
    /// Custom secret type
    Custom(String),
}

/// Health monitoring specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitoringSpec {
    /// Health checks
    pub health_checks: Vec<HealthCheckSpec>,

    /// Metrics collection
    pub metrics: Option<MetricsSpec>,

    /// Logging configuration
    pub logging: Option<LoggingSpec>,

    /// Tracing configuration
    pub tracing: Option<TracingSpec>,

    /// Alerting configuration
    pub alerting: Option<AlertingSpec>,
}

/// Health check specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckSpec {
    /// Check name
    pub name: String,

    /// Check type
    pub check_type: HealthCheckType,

    /// Check interval
    pub interval: u32,

    /// Check timeout
    pub timeout: u32,

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
        /// URL path to check
        path: String,
        /// Port number
        port: u16,
        /// HTTP or HTTPS
        scheme: HttpScheme,
        /// Additional headers
        headers: HashMap<String, String>,
    },
    /// TCP connectivity check
    Tcp {
        /// Port to connect to
        port: u16,
    },
    /// Execute a command
    Exec {
        /// Command and arguments
        command: Vec<String>,
    },
    /// gRPC health check
    Grpc {
        /// gRPC port
        port: u16,
        /// Service name (optional)
        service: Option<String>,
    },
}

/// HTTP schemes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpScheme {
    /// Plain HTTP
    Http,
    /// HTTPS (TLS)
    Https,
}

/// Metrics specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSpec {
    /// Enable metrics collection
    pub enabled: bool,

    /// Metrics port
    pub port: u16,

    /// Metrics path
    pub path: String,

    /// Metrics format
    pub format: MetricsFormat,

    /// Collection interval
    pub interval: u32,
}

/// Metrics formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricsFormat {
    /// Prometheus exposition format
    Prometheus,
    /// `OpenMetrics` format
    OpenMetrics,
    /// JSON format
    Json,
}

/// Logging specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSpec {
    /// Log level
    pub level: LogLevel,

    /// Log format
    pub format: LogFormat,

    /// Log outputs
    pub outputs: Vec<LogOutputSpec>,

    /// Log rotation
    pub rotation: Option<LogRotationSpec>,
}

/// Log levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    /// Most verbose
    Trace,
    /// Debug diagnostics
    Debug,
    /// Informational
    Info,
    /// Warnings
    Warn,
    /// Errors
    Error,
    /// Fatal (process will exit)
    Fatal,
}

/// Log formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    /// JSON output
    Json,
    /// Plain text
    Text,
    /// Structured key-value
    Structured,
}

/// Log output specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogOutputSpec {
    /// Output type
    pub output_type: LogOutputType,

    /// Output configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Log output types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutputType {
    /// Standard output
    Stdout,
    /// Standard error
    Stderr,
    /// File output
    File,
    /// Syslog output
    Syslog,
    /// Elasticsearch output
    ElasticSearch,
    /// Fluentd output
    Fluentd,
}

/// Log rotation specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationSpec {
    /// Maximum file size in MB
    pub max_size_mb: u32,

    /// Maximum number of files
    pub max_files: u32,

    /// Maximum age in days
    pub max_age_days: u32,

    /// Compress old files
    pub compress: bool,
}

/// Tracing specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingSpec {
    /// Enable tracing
    pub enabled: bool,

    /// Sampling rate
    pub sampling_rate: f64,

    /// Exporter configuration
    pub exporter: TracingExporter,
}

/// Tracing exporters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TracingExporter {
    /// Jaeger distributed tracing
    Jaeger {
        /// Jaeger collector endpoint
        endpoint: String,
    },
    /// Zipkin distributed tracing
    Zipkin {
        /// Zipkin collector endpoint
        endpoint: String,
    },
    /// OpenTelemetry Protocol
    Otlp {
        /// OTLP collector endpoint
        endpoint: String,
    },
}

/// Alerting specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingSpec {
    /// Alert rules
    pub rules: Vec<AlertRuleSpec>,

    /// Notification channels
    pub channels: Vec<NotificationChannelSpec>,
}

/// Alert rule specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRuleSpec {
    /// Rule name
    pub name: String,

    /// Condition
    pub condition: String,

    /// Severity
    pub severity: AlertSeverity,

    /// Duration
    pub duration: u32,
}

/// Alert severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Critical severity — immediate action required
    Critical,
    /// Warning severity — attention needed
    Warning,
    /// Informational severity
    Info,
}

/// Notification channel specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannelSpec {
    /// Channel name
    pub name: String,

    /// Channel type
    pub channel_type: NotificationChannelType,

    /// Configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Notification channel types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannelType {
    /// Email notification
    Email,
    /// Slack notification
    Slack,
    /// Generic webhook
    Webhook,
    /// `PagerDuty` integration
    PagerDuty,
}

/// Biome networking specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeNetworkingSpec {
    /// Network policies - reference to networking policies
    pub policies: Vec<super::networking_policies::NetworkPolicySpec>,

    /// Service mesh configuration
    pub service_mesh: Option<super::networking_services::ServiceMeshSpec>,

    /// DNS configuration
    pub dns: Option<DnsSpec>,
}

/// DNS specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsSpec {
    /// Nameservers
    pub nameservers: Vec<String>,

    /// Search domains
    pub search: Vec<String>,

    /// Options
    pub options: Vec<DnsOptionSpec>,
}

/// DNS option specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsOptionSpec {
    /// Name
    pub name: String,

    /// Value
    pub value: Option<String>,
}

/// Scaling specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingSpec {
    /// Minimum replicas
    pub min_replicas: u32,

    /// Maximum replicas
    pub max_replicas: u32,

    /// Target CPU utilization
    pub target_cpu_utilization: Option<u32>,

    /// Target memory utilization
    pub target_memory_utilization: Option<u32>,

    /// Custom metrics
    pub custom_metrics: Vec<CustomMetricSpec>,
}

/// Custom metric specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMetricSpec {
    /// Metric name
    pub name: String,

    /// Target value
    pub target_value: f64,

    /// Metric selector
    pub selector: HashMap<String, String>,
}

/// Lifecycle specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleSpec {
    /// Pre-start hooks
    pub pre_start: Vec<LifecycleHookSpec>,

    /// Post-start hooks
    pub post_start: Vec<LifecycleHookSpec>,

    /// Pre-stop hooks
    pub pre_stop: Vec<LifecycleHookSpec>,

    /// Post-stop hooks
    pub post_stop: Vec<LifecycleHookSpec>,
}

/// Lifecycle hook specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleHookSpec {
    /// Hook name
    pub name: String,

    /// Hook action
    pub action: LifecycleAction,

    /// Timeout
    pub timeout: Option<u32>,
}

/// Lifecycle actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleAction {
    /// Execute a command
    Exec {
        /// Command and arguments
        command: Vec<String>,
    },
    /// Make an HTTP request
    Http {
        /// Request URL
        url: String,
        /// HTTP method
        method: String,
        /// Request headers
        headers: HashMap<String, String>,
    },
}

/// Enforcement modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnforcementMode {
    /// Strictly enforce the policy
    Enforce,
    /// Warn on violations but allow
    Warn,
    /// Policy disabled
    Disabled,
}
