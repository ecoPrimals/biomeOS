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
    /// Most verbose — captures everything
    Trace,
    /// Debug-level diagnostics
    Debug,
    /// Informational messages
    Info,
    /// Potentially harmful situations
    Warn,
    /// Error events
    Error,
    /// Logging disabled
    Off,
}

/// Log formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    /// Structured JSON output
    Json,
    /// Plain text output
    Plain,
    /// Pretty-printed (human-readable)
    Pretty,
    /// Compact single-line format
    Compact,
    /// Custom format string
    Custom(String),
}

/// Log destinations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogDestination {
    /// Standard output
    Stdout,
    /// Standard error
    Stderr,
    /// Log to a file
    File(PathBuf),
    /// Forward to syslog
    Syslog(SyslogConfig),
    /// Send to a network endpoint
    Network(NetworkLogConfig),
    /// Fan-out to multiple destinations
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
    /// UDP transport
    Udp,
    /// TCP transport
    Tcp,
    /// TLS-encrypted transport
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
    /// HTTP transport
    Http,
    /// HTTPS transport
    Https,
    /// Raw TCP transport
    Tcp,
    /// Raw UDP transport
    Udp,
    /// Custom protocol
    Custom(String),
}

/// Network log authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkLogAuth {
    /// Bearer token authentication
    Bearer(String),
    /// HTTP basic authentication
    Basic {
        /// Username
        username: String,
        /// Password
        password: String,
    },
    /// API key authentication
    ApiKey {
        /// API key value
        key: String,
        /// Header name for the key
        header: String,
    },
    /// Custom authentication parameters
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
    /// Rotate every hour
    Hourly,
    /// Rotate every day
    Daily,
    /// Rotate every week
    Weekly,
    /// Rotate every month
    Monthly,
    /// Custom cron-like expression
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
    /// Allow matching log entries
    Allow,
    /// Deny matching log entries
    Deny,
    /// Transform matching log entries with the given template
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
    /// Random probabilistic sampling
    Random,
    /// Deterministic hash-based sampling
    Deterministic,
    /// Rate-limited sampling
    RateLimited {
        /// Maximum events per second
        rate: u32,
    },
    /// Custom sampling implementation
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
    /// Console output
    Console,
    /// Jaeger distributed tracing
    Jaeger(JaegerConfig),
    /// Zipkin distributed tracing
    Zipkin(ZipkinConfig),
    /// OpenTelemetry Protocol (OTLP)
    Otlp(OtlpConfig),
    /// Custom exporter
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
    /// gRPC transport
    Grpc,
    /// HTTP transport
    Http,
}

/// OTLP compression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OtlpCompression {
    /// Gzip compression
    Gzip,
    /// No compression
    None,
}

/// Tracing authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TracingAuth {
    /// Bearer token
    Bearer(String),
    /// HTTP basic auth
    Basic {
        /// Username
        username: String,
        /// Password
        password: String,
    },
    /// API key auth
    ApiKey {
        /// API key value
        key: String,
        /// Header name
        header: String,
    },
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
    /// Always sample
    Always,
    /// Never sample
    Never,
    /// Sample based on trace ID ratio
    TraceIdRatio,
    /// Rate-limited sampling
    RateLimited {
        /// Maximum traces per second
        rate: u32,
    },
    /// Custom sampling strategy
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
    /// Prometheus exposition format
    Prometheus,
    /// JSON format
    Json,
    /// StatsD protocol
    StatsD,
    /// InfluxDB line protocol
    InfluxDB,
    /// Custom format
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
    /// Monotonically increasing counter
    Counter,
    /// Value that can go up and down
    Gauge,
    /// Distribution of values in buckets
    Histogram,
    /// Statistical summary with quantiles
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
    /// Informational alert
    Info,
    /// Warning-level alert
    Warning,
    /// Critical-level alert
    Critical,
    /// Emergency-level alert (requires immediate action)
    Emergency,
}

/// Notification channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    /// Email notifications
    Email(EmailNotificationConfig),
    /// Slack notifications
    Slack(SlackNotificationConfig),
    /// Generic webhook notifications
    Webhook(WebhookNotificationConfig),
    /// PagerDuty notifications
    PagerDuty(PagerDutyNotificationConfig),
    /// Custom notification handler
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
    /// Bearer token
    Bearer(String),
    /// HTTP basic auth
    Basic {
        /// Username
        username: String,
        /// Password
        password: String,
    },
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

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════════════
    // Default Implementations
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_observability_config_default() {
        let config = ObservabilityConfig::default();
        assert!(config.logging.structured);
        assert!(!config.tracing.enabled);
        assert!(config.metrics.enabled);
        assert!(config.alerting.is_none());
    }

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert!(matches!(config.level, LogLevel::Info));
        assert!(matches!(config.format, LogFormat::Json));
        assert!(matches!(config.destination, LogDestination::Stdout));
        assert!(config.structured);
        assert!(config.filtering.is_none());
        assert!(config.sampling.is_none());
    }

    #[test]
    fn test_log_rotation_default() {
        let config = LogRotationConfig::default();
        assert_eq!(config.max_size, 100 * 1024 * 1024);
        assert_eq!(config.max_files, 10);
        assert!(config.compress);
        assert!(config.schedule.is_none());
    }

    #[test]
    fn test_tracing_config_default() {
        let config = TracingConfig::default();
        assert!(!config.enabled);
        assert!(matches!(config.exporter, TracingExporter::Console));
        assert!(config.sampling.parent_based);
    }

    #[test]
    fn test_tracing_sampling_default() {
        let config = TracingSamplingConfig::default();
        assert!((config.rate - 1.0).abs() < f64::EPSILON);
        assert!(matches!(config.strategy, TracingSamplingStrategy::Always));
        assert!(config.parent_based);
    }

    #[test]
    fn test_span_limits_default() {
        let config = SpanLimitsConfig::default();
        assert_eq!(config.max_attributes, Some(128));
        assert_eq!(config.max_events, Some(128));
        assert_eq!(config.max_links, Some(128));
        assert_eq!(config.max_attribute_value_length, Some(4096));
    }

    #[test]
    fn test_tracing_resource_default() {
        let config = TracingResourceConfig::default();
        assert_eq!(config.service_name, "biomeos");
        assert!(config.service_version.is_none());
        assert!(config.attributes.is_empty());
    }

    #[test]
    fn test_metrics_config_default() {
        let config = MetricsConfig::default();
        assert!(config.enabled);
        assert!(matches!(config.format, MetricsFormat::Prometheus));
        assert!(config.endpoint.is_none());
        assert_eq!(config.interval, Duration::from_secs(60));
        assert!(config.labels.is_empty());
        assert!(config.custom.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Enum Serialization
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_log_level_serialization() {
        for level in [
            LogLevel::Trace,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
            LogLevel::Off,
        ] {
            let json = serde_json::to_string(&level).expect("serialize");
            let _: LogLevel = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_log_format_serialization() {
        for format in [
            LogFormat::Json,
            LogFormat::Plain,
            LogFormat::Pretty,
            LogFormat::Compact,
            LogFormat::Custom("%{time} %{level} %{message}".to_string()),
        ] {
            let json = serde_json::to_string(&format).expect("serialize");
            let _: LogFormat = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_syslog_protocol_serialization() {
        for protocol in [
            SyslogProtocol::Udp,
            SyslogProtocol::Tcp,
            SyslogProtocol::Tls,
        ] {
            let json = serde_json::to_string(&protocol).expect("serialize");
            let _: SyslogProtocol = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_rotation_schedule_serialization() {
        for schedule in [
            RotationSchedule::Hourly,
            RotationSchedule::Daily,
            RotationSchedule::Weekly,
            RotationSchedule::Monthly,
            RotationSchedule::Custom("0 0 * * *".to_string()),
        ] {
            let json = serde_json::to_string(&schedule).expect("serialize");
            let _: RotationSchedule = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_log_filter_action_serialization() {
        for action in [
            LogFilterAction::Allow,
            LogFilterAction::Deny,
            LogFilterAction::Transform("{{level}}: {{message}}".to_string()),
        ] {
            let json = serde_json::to_string(&action).expect("serialize");
            let _: LogFilterAction = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_sampling_strategy_serialization() {
        for strategy in [
            SamplingStrategy::Random,
            SamplingStrategy::Deterministic,
            SamplingStrategy::RateLimited { rate: 100 },
            SamplingStrategy::Custom("adaptive".to_string()),
        ] {
            let json = serde_json::to_string(&strategy).expect("serialize");
            let _: SamplingStrategy = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_otlp_protocol_serialization() {
        for protocol in [OtlpProtocol::Grpc, OtlpProtocol::Http] {
            let json = serde_json::to_string(&protocol).expect("serialize");
            let _: OtlpProtocol = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_otlp_compression_serialization() {
        for compression in [OtlpCompression::Gzip, OtlpCompression::None] {
            let json = serde_json::to_string(&compression).expect("serialize");
            let _: OtlpCompression = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_tracing_sampling_strategy_serialization() {
        for strategy in [
            TracingSamplingStrategy::Always,
            TracingSamplingStrategy::Never,
            TracingSamplingStrategy::TraceIdRatio,
            TracingSamplingStrategy::RateLimited { rate: 50 },
            TracingSamplingStrategy::Custom("head-based".to_string()),
        ] {
            let json = serde_json::to_string(&strategy).expect("serialize");
            let _: TracingSamplingStrategy = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_metrics_format_serialization() {
        for format in [
            MetricsFormat::Prometheus,
            MetricsFormat::Json,
            MetricsFormat::StatsD,
            MetricsFormat::InfluxDB,
            MetricsFormat::Custom("datadog".to_string()),
        ] {
            let json = serde_json::to_string(&format).expect("serialize");
            let _: MetricsFormat = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_metric_type_serialization() {
        for mt in [
            MetricType::Counter,
            MetricType::Gauge,
            MetricType::Histogram,
            MetricType::Summary,
        ] {
            let json = serde_json::to_string(&mt).expect("serialize");
            let _: MetricType = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_alert_severity_serialization() {
        for severity in [
            AlertSeverity::Info,
            AlertSeverity::Warning,
            AlertSeverity::Critical,
            AlertSeverity::Emergency,
        ] {
            let json = serde_json::to_string(&severity).expect("serialize");
            let _: AlertSeverity = serde_json::from_str(&json).expect("deserialize");
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Complex Configuration Types
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_syslog_config_creation() {
        let config = SyslogConfig {
            server: "syslog.local".to_string(),
            port: 514,
            facility: "local0".to_string(),
            protocol: SyslogProtocol::Tls,
        };
        assert_eq!(config.port, 514);
    }

    #[test]
    fn test_network_log_config_creation() {
        let config = NetworkLogConfig {
            endpoint: "https://logs.example.com".to_string(),
            protocol: NetworkLogProtocol::Https,
            auth: Some(NetworkLogAuth::Bearer("token123".to_string())),
            buffer_size: Some(8192),
            batch_size: Some(100),
            flush_interval: Some(Duration::from_secs(5)),
        };
        assert!(config.auth.is_some());
    }

    #[test]
    fn test_log_filter_config_creation() {
        let mut modules = HashMap::new();
        modules.insert("hyper".to_string(), LogLevel::Warn);
        modules.insert("tokio".to_string(), LogLevel::Info);

        let config = LogFilterConfig {
            modules,
            targets: HashMap::new(),
            custom: vec![CustomLogFilter {
                name: "exclude-health".to_string(),
                expression: "path == '/health'".to_string(),
                action: LogFilterAction::Deny,
            }],
        };
        assert_eq!(config.modules.len(), 2);
        assert_eq!(config.custom.len(), 1);
    }

    #[test]
    fn test_log_sampling_config_creation() {
        let config = LogSamplingConfig {
            enabled: true,
            rate: 0.1,
            strategy: SamplingStrategy::RateLimited { rate: 100 },
        };
        assert!(config.enabled);
        assert!((config.rate - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn test_jaeger_config_creation() {
        let config = JaegerConfig {
            endpoint: "http://jaeger:14268/api/traces".to_string(),
            service_name: "biomeos".to_string(),
            auth: Some(TracingAuth::Bearer("jaeger-token".to_string())),
        };
        assert!(config.auth.is_some());
    }

    #[test]
    fn test_otlp_config_creation() {
        let mut headers = HashMap::new();
        headers.insert("x-api-key".to_string(), "key123".to_string());

        let config = OtlpConfig {
            endpoint: "http://otel-collector:4317".to_string(),
            protocol: OtlpProtocol::Grpc,
            headers,
            compression: Some(OtlpCompression::Gzip),
        };
        assert_eq!(config.headers.len(), 1);
        assert!(config.compression.is_some());
    }

    #[test]
    fn test_custom_metric_config_creation() {
        let config = CustomMetricConfig {
            name: "request_duration_seconds".to_string(),
            metric_type: MetricType::Histogram,
            description: "Request duration in seconds".to_string(),
            labels: vec![
                "method".to_string(),
                "path".to_string(),
                "status".to_string(),
            ],
        };
        assert_eq!(config.labels.len(), 3);
    }

    #[test]
    fn test_alert_rule_creation() {
        let mut labels = HashMap::new();
        labels.insert("team".to_string(), "platform".to_string());
        let mut annotations = HashMap::new();
        annotations.insert("summary".to_string(), "High error rate".to_string());

        let config = AlertRule {
            name: "high-error-rate".to_string(),
            expression: "rate(errors[5m]) > 0.1".to_string(),
            severity: AlertSeverity::Critical,
            interval: Duration::from_secs(60),
            duration: Duration::from_secs(300),
            labels,
            annotations,
        };
        assert_eq!(config.severity, AlertSeverity::Critical);
    }

    #[test]
    fn test_slack_notification_config_creation() {
        let config = SlackNotificationConfig {
            webhook_url: "https://hooks.slack.com/services/xxx".to_string(),
            channel: "#alerts".to_string(),
            username: Some("BiomeOS".to_string()),
            message_template: "Alert: {{.AlertName}}".to_string(),
        };
        assert!(config.username.is_some());
    }

    #[test]
    fn test_email_notification_config_creation() {
        let config = EmailNotificationConfig {
            smtp_server: "smtp.example.com".to_string(),
            smtp_port: 587,
            username: "alerts@example.com".to_string(),
            password: "secret".to_string(),
            from: "alerts@example.com".to_string(),
            to: vec!["oncall@example.com".to_string()],
            subject_template: "[{{.Severity}}] {{.AlertName}}".to_string(),
            body_template: "{{.Description}}".to_string(),
        };
        assert_eq!(config.smtp_port, 587);
    }

    #[test]
    fn test_alert_grouping_config_creation() {
        let config = AlertGroupingConfig {
            group_by: vec!["alertname".to_string(), "service".to_string()],
            group_wait: Duration::from_secs(30),
            group_interval: Duration::from_secs(60),
            repeat_interval: Duration::from_secs(3600),
        };
        assert_eq!(config.group_by.len(), 2);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Serialization Roundtrip
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_observability_config_serialization() {
        let config = ObservabilityConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: ObservabilityConfig = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.logging.structured);
    }

    #[test]
    fn test_logging_config_serialization() {
        let config = LoggingConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: LoggingConfig = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.structured);
    }

    #[test]
    fn test_tracing_config_serialization() {
        let config = TracingConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: TracingConfig = serde_json::from_str(&json).expect("deserialize");
        assert!(!deserialized.enabled);
    }

    #[test]
    fn test_metrics_config_serialization() {
        let config = MetricsConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: MetricsConfig = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.enabled);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Clone & Debug
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_observability_config_clone() {
        let original = ObservabilityConfig::default();
        let cloned = original.clone();
        assert!(cloned.metrics.enabled);
    }

    #[test]
    fn test_log_level_debug() {
        let level = LogLevel::Info;
        let debug = format!("{:?}", level);
        assert!(debug.contains("Info"));
    }

    #[test]
    fn test_alert_severity_ordering() {
        assert!(AlertSeverity::Info < AlertSeverity::Warning);
        assert!(AlertSeverity::Warning < AlertSeverity::Critical);
        assert!(AlertSeverity::Critical < AlertSeverity::Emergency);
    }
}
