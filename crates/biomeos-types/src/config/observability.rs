//! Observability Configuration
//!
//! This module contains observability-related configuration types including
//! logging, tracing, metrics, and alerting configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Logging configuration
    pub logging: LoggingConfig,

    /// Tracing configuration
    pub tracing: TracingConfig,

    /// Metrics configuration
    pub metrics: MetricsConfig,

    /// Alerting configuration
    pub alerting: Option<AlertingConfig>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: LogLevel,

    /// Log format
    pub format: LogFormat,

    /// Log destination
    pub destination: LogDestination,

    /// Log rotation
    pub rotation: LogRotationConfig,

    /// Structured logging
    pub structured: bool,

    /// Enable log filtering
    pub filtering: Option<LogFilterConfig>,

    /// Log sampling configuration
    pub sampling: Option<LogSamplingConfig>,
}

/// Log levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Off,
}

/// Log formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Plain,
    Pretty,
    Compact,
    Custom(String),
}

/// Log destinations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogDestination {
    Stdout,
    Stderr,
    File(PathBuf),
    Syslog(SyslogConfig),
    Network(NetworkLogConfig),
    Multiple(Vec<LogDestination>),
}

/// Syslog configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyslogConfig {
    /// Syslog server
    pub server: String,

    /// Syslog port
    pub port: u16,

    /// Syslog facility
    pub facility: String,

    /// Syslog protocol
    pub protocol: SyslogProtocol,
}

/// Syslog protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyslogProtocol {
    Udp,
    Tcp,
    Tls,
}

/// Network log configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkLogConfig {
    /// Network endpoint
    pub endpoint: String,

    /// Network protocol
    pub protocol: NetworkLogProtocol,

    /// Authentication
    pub auth: Option<NetworkLogAuth>,

    /// Buffer size
    pub buffer_size: Option<usize>,

    /// Batch size
    pub batch_size: Option<usize>,

    /// Flush interval
    pub flush_interval: Option<Duration>,
}

/// Network log protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkLogProtocol {
    Http,
    Https,
    Tcp,
    Udp,
    Custom(String),
}

/// Network log authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkLogAuth {
    Bearer(String),
    Basic { username: String, password: String },
    ApiKey { key: String, header: String },
    Custom(HashMap<String, String>),
}

/// Log rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    /// Max file size before rotation
    pub max_size: usize,

    /// Max number of files to keep
    pub max_files: usize,

    /// Compress rotated files
    pub compress: bool,

    /// Rotation schedule
    pub schedule: Option<RotationSchedule>,
}

/// Rotation schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationSchedule {
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Custom(String),
}

/// Log filter configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFilterConfig {
    /// Module filters
    pub modules: HashMap<String, LogLevel>,

    /// Target filters
    pub targets: HashMap<String, LogLevel>,

    /// Custom filters
    pub custom: Vec<CustomLogFilter>,
}

/// Custom log filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomLogFilter {
    /// Filter name
    pub name: String,

    /// Filter expression
    pub expression: String,

    /// Filter action
    pub action: LogFilterAction,
}

/// Log filter actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFilterAction {
    Allow,
    Deny,
    Transform(String),
}

/// Log sampling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSamplingConfig {
    /// Enable sampling
    pub enabled: bool,

    /// Sampling rate (0.0-1.0)
    pub rate: f64,

    /// Sampling strategy
    pub strategy: SamplingStrategy,
}

/// Sampling strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SamplingStrategy {
    Random,
    Deterministic,
    RateLimited { rate: u32 },
    Custom(String),
}

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Enable tracing
    pub enabled: bool,

    /// Tracing level
    pub level: LogLevel,

    /// Tracing exporter
    pub exporter: TracingExporter,

    /// Tracing sampling
    pub sampling: TracingSamplingConfig,

    /// Span limits
    pub span_limits: SpanLimitsConfig,

    /// Resource configuration
    pub resource: TracingResourceConfig,
}

/// Tracing exporters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TracingExporter {
    Console,
    Jaeger(JaegerConfig),
    Zipkin(ZipkinConfig),
    Otlp(OtlpConfig),
    Custom(String),
}

/// Jaeger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JaegerConfig {
    /// Jaeger endpoint
    pub endpoint: String,

    /// Service name
    pub service_name: String,

    /// Authentication
    pub auth: Option<TracingAuth>,
}

/// Zipkin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZipkinConfig {
    /// Zipkin endpoint
    pub endpoint: String,

    /// Service name
    pub service_name: String,
}

/// OTLP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpConfig {
    /// OTLP endpoint
    pub endpoint: String,

    /// Protocol
    pub protocol: OtlpProtocol,

    /// Headers
    pub headers: HashMap<String, String>,

    /// Compression
    pub compression: Option<OtlpCompression>,
}

/// OTLP protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OtlpProtocol {
    Grpc,
    Http,
}

/// OTLP compression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OtlpCompression {
    Gzip,
    None,
}

/// Tracing authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TracingAuth {
    Bearer(String),
    Basic { username: String, password: String },
    ApiKey { key: String, header: String },
}

/// Tracing sampling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingSamplingConfig {
    /// Sampling rate (0.0-1.0)
    pub rate: f64,

    /// Sampling strategy
    pub strategy: TracingSamplingStrategy,

    /// Parent-based sampling
    pub parent_based: bool,
}

/// Tracing sampling strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TracingSamplingStrategy {
    Always,
    Never,
    TraceIdRatio,
    RateLimited { rate: u32 },
    Custom(String),
}

/// Span limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanLimitsConfig {
    /// Max attributes per span
    pub max_attributes: Option<u32>,

    /// Max events per span
    pub max_events: Option<u32>,

    /// Max links per span
    pub max_links: Option<u32>,

    /// Max attribute value length
    pub max_attribute_value_length: Option<u32>,
}

/// Tracing resource configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingResourceConfig {
    /// Service name
    pub service_name: String,

    /// Service version
    pub service_version: Option<String>,

    /// Service namespace
    pub service_namespace: Option<String>,

    /// Additional attributes
    pub attributes: HashMap<String, String>,
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,

    /// Metrics export format
    pub format: MetricsFormat,

    /// Metrics export endpoint
    pub endpoint: Option<String>,

    /// Metrics collection interval
    pub interval: Duration,

    /// Metrics retention period
    pub retention: Duration,

    /// Metrics labels
    pub labels: HashMap<String, String>,

    /// Custom metrics
    pub custom: Vec<CustomMetricConfig>,
}

/// Metrics formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricsFormat {
    Prometheus,
    Json,
    StatsD,
    InfluxDB,
    Custom(String),
}

/// Custom metric configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMetricConfig {
    /// Metric name
    pub name: String,

    /// Metric type
    pub metric_type: MetricType,

    /// Metric description
    pub description: String,

    /// Metric labels
    pub labels: Vec<String>,
}

/// Metric types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Summary,
}

/// Alerting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingConfig {
    /// Enable alerting
    pub enabled: bool,

    /// Alert rules
    pub rules: Vec<AlertRule>,

    /// Notification channels
    pub channels: Vec<NotificationChannel>,

    /// Alert manager configuration
    pub manager: Option<AlertManagerConfig>,
}

/// Alert rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Rule name
    pub name: String,

    /// Rule expression
    pub expression: String,

    /// Rule severity
    pub severity: AlertSeverity,

    /// Evaluation interval
    pub interval: Duration,

    /// Alert duration
    pub duration: Duration,

    /// Rule labels
    pub labels: HashMap<String, String>,

    /// Rule annotations
    pub annotations: HashMap<String, String>,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Notification channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Email(EmailNotificationConfig),
    Slack(SlackNotificationConfig),
    Webhook(WebhookNotificationConfig),
    PagerDuty(PagerDutyNotificationConfig),
    Custom(CustomNotificationConfig),
}

/// Email notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailNotificationConfig {
    /// SMTP server
    pub smtp_server: String,

    /// SMTP port
    pub smtp_port: u16,

    /// SMTP username
    pub username: String,

    /// SMTP password
    pub password: String,

    /// From address
    pub from: String,

    /// To addresses
    pub to: Vec<String>,

    /// Subject template
    pub subject_template: String,

    /// Body template
    pub body_template: String,
}

/// Slack notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackNotificationConfig {
    /// Webhook URL
    pub webhook_url: String,

    /// Channel
    pub channel: String,

    /// Username
    pub username: Option<String>,

    /// Message template
    pub message_template: String,
}

/// Webhook notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookNotificationConfig {
    /// Webhook URL
    pub url: String,

    /// HTTP method
    pub method: String,

    /// Headers
    pub headers: HashMap<String, String>,

    /// Body template
    pub body_template: String,
}

/// PagerDuty notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagerDutyNotificationConfig {
    /// Integration key
    pub integration_key: String,

    /// Severity mapping
    pub severity_mapping: HashMap<AlertSeverity, String>,
}

/// Custom notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomNotificationConfig {
    /// Handler name
    pub handler: String,

    /// Configuration parameters
    pub config: HashMap<String, String>,
}

/// Alert manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertManagerConfig {
    /// Alert manager URL
    pub url: String,

    /// Authentication
    pub auth: Option<AlertManagerAuth>,

    /// Grouping configuration
    pub grouping: AlertGroupingConfig,
}

/// Alert manager authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertManagerAuth {
    Bearer(String),
    Basic { username: String, password: String },
}

/// Alert grouping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertGroupingConfig {
    /// Group by labels
    pub group_by: Vec<String>,

    /// Group wait time
    pub group_wait: Duration,

    /// Group interval
    pub group_interval: Duration,

    /// Repeat interval
    pub repeat_interval: Duration,
}

/// Default implementations
impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            logging: LoggingConfig::default(),
            tracing: TracingConfig::default(),
            metrics: MetricsConfig::default(),
            alerting: None,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            format: LogFormat::Json,
            destination: LogDestination::Stdout,
            rotation: LogRotationConfig::default(),
            structured: true,
            filtering: None,
            sampling: None,
        }
    }
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        Self {
            max_size: 100 * 1024 * 1024, // 100MB
            max_files: 10,
            compress: true,
            schedule: None,
        }
    }
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            level: LogLevel::Info,
            exporter: TracingExporter::Console,
            sampling: TracingSamplingConfig::default(),
            span_limits: SpanLimitsConfig::default(),
            resource: TracingResourceConfig::default(),
        }
    }
}

impl Default for TracingSamplingConfig {
    fn default() -> Self {
        Self {
            rate: 1.0,
            strategy: TracingSamplingStrategy::Always,
            parent_based: true,
        }
    }
}

impl Default for SpanLimitsConfig {
    fn default() -> Self {
        Self {
            max_attributes: Some(128),
            max_events: Some(128),
            max_links: Some(128),
            max_attribute_value_length: Some(4096),
        }
    }
}

impl Default for TracingResourceConfig {
    fn default() -> Self {
        Self {
            service_name: "biomeos".to_string(),
            service_version: None,
            service_namespace: None,
            attributes: HashMap::new(),
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            format: MetricsFormat::Prometheus,
            endpoint: None,
            interval: Duration::from_secs(60),
            retention: Duration::from_secs(24 * 60 * 60), // 24 hours
            labels: HashMap::new(),
            custom: Vec::new(),
        }
    }
} 