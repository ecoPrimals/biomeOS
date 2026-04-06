// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Observability Configuration
//!
//! This module contains observability-related configuration types including
//! logging, tracing, metrics, and alerting configuration.

use serde::{Deserialize, Serialize};

pub mod alerting;
pub mod logging;
pub mod metrics;
pub mod tracing;

pub use alerting::{
    AlertGroupingConfig, AlertManagerAuth, AlertManagerConfig, AlertRule, AlertSeverity,
    AlertingConfig, CustomNotificationConfig, EmailNotificationConfig, PagerDutyNotificationConfig,
    SlackNotificationConfig, WebhookNotificationConfig,
};
pub use logging::{
    CustomLogFilter, LogDestination, LogFilterAction, LogFilterConfig, LogFormat, LogLevel,
    LogRotationConfig, LogSamplingConfig, LoggingConfig, NetworkLogAuth, NetworkLogConfig,
    NetworkLogProtocol, RotationSchedule, SamplingStrategy, SyslogConfig, SyslogProtocol,
};
pub use metrics::{CustomMetricConfig, MetricType, MetricsConfig, MetricsFormat};
pub use tracing::{
    JaegerConfig, OtlpCompression, OtlpConfig, OtlpProtocol, SpanLimitsConfig, TracingAuth,
    TracingConfig, TracingExporter, TracingResourceConfig, TracingSamplingConfig,
    TracingSamplingStrategy, ZipkinConfig,
};

/// Observability configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

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
    fn test_alert_severity_ordering() {
        assert!(AlertSeverity::Info < AlertSeverity::Warning);
        assert!(AlertSeverity::Warning < AlertSeverity::Critical);
        assert!(AlertSeverity::Critical < AlertSeverity::Emergency);
    }

    #[test]
    fn test_observability_config_serialization() {
        let config = ObservabilityConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: ObservabilityConfig = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.logging.structured);
    }

    #[test]
    fn test_observability_config_clone() {
        let original = ObservabilityConfig::default();
        let cloned = original;
        assert!(cloned.metrics.enabled);
    }

    #[test]
    fn test_log_level_debug() {
        let level = LogLevel::Info;
        let debug = format!("{level:?}");
        assert!(debug.contains("Info"));
    }
}
