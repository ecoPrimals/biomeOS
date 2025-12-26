//! Manifest Extensions and Dependencies
//!
//! This module contains manifest extension types including BiomeDependency,
//! ConfigSpec, SecretSpec, health monitoring, and other extension specifications.

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
    Registry {
        url: String,
    },
    Git {
        url: String,
        branch: Option<String>,
        tag: Option<String>,
        commit: Option<String>,
    },
    Local {
        path: String,
    },
}

/// Configuration specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSpec {
    /// Configuration data
    pub data: HashMap<String, String>,

    /// Binary data
    pub binary_data: HashMap<String, Vec<u8>>,

    /// Immutable flag
    pub immutable: bool,
}

/// Secret specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretSpec {
    /// Secret type
    pub secret_type: SecretType,

    /// Secret data
    pub data: HashMap<String, Vec<u8>>,

    /// String data
    pub string_data: HashMap<String, String>,

    /// Immutable flag
    pub immutable: bool,
}

/// Secret types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecretType {
    Opaque,
    ServiceAccountToken,
    DockerConfigJson,
    DockerConfig,
    BasicAuth,
    SshAuth,
    Tls,
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
    Http {
        path: String,
        port: u16,
        scheme: HttpScheme,
        headers: HashMap<String, String>,
    },
    Tcp {
        port: u16,
    },
    Exec {
        command: Vec<String>,
    },
    Grpc {
        port: u16,
        service: Option<String>,
    },
}

/// HTTP schemes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpScheme {
    Http,
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
    Prometheus,
    OpenMetrics,
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
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

/// Log formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Text,
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
    Stdout,
    Stderr,
    File,
    Syslog,
    ElasticSearch,
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
    Jaeger { endpoint: String },
    Zipkin { endpoint: String },
    Otlp { endpoint: String },
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
    Critical,
    Warning,
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
    Email,
    Slack,
    Webhook,
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
    Exec {
        command: Vec<String>,
    },
    Http {
        url: String,
        method: String,
        headers: HashMap<String, String>,
    },
}

/// Enforcement modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnforcementMode {
    Enforce,
    Warn,
    Disabled,
}
