// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Metrics configuration types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

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
    /// `StatsD` protocol
    StatsD,
    /// `InfluxDB` line protocol
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

/// Default StatsD UDP endpoint for local metrics export configs.
pub use crate::constants::endpoints::DEFAULT_STATSD_UDP_ENDPOINT;

#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_config_default() {
        let config = MetricsConfig::default();
        assert!(config.enabled);
        assert!(matches!(config.format, MetricsFormat::Prometheus));
        assert!(config.endpoint.is_none());
        assert_eq!(config.interval, Duration::from_secs(60));
        assert_eq!(config.retention, Duration::from_secs(24 * 60 * 60));
        assert!(config.labels.is_empty());
        assert!(config.custom.is_empty());
    }

    #[test]
    fn test_metrics_format_serde() {
        let formats = [
            MetricsFormat::Prometheus,
            MetricsFormat::Json,
            MetricsFormat::StatsD,
            MetricsFormat::InfluxDB,
            MetricsFormat::Custom("custom".to_string()),
        ];
        for format in formats {
            let json = serde_json::to_string(&format).expect("serialize");
            let _: MetricsFormat = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_metric_type_serde() {
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
    fn test_custom_metric_config_serde() {
        let config = CustomMetricConfig {
            name: "requests_total".to_string(),
            metric_type: MetricType::Counter,
            description: "Total requests".to_string(),
            labels: vec!["method".to_string(), "path".to_string()],
        };
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: CustomMetricConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(config.name, deserialized.name);
        assert!(matches!(deserialized.metric_type, MetricType::Counter));
    }

    #[test]
    fn test_metrics_config_serde_roundtrip() {
        let config = MetricsConfig {
            enabled: false,
            format: MetricsFormat::StatsD,
            endpoint: Some("udp://localhost:8125".to_string()),
            interval: Duration::from_secs(30),
            retention: Duration::from_secs(3600),
            labels: {
                let mut m = HashMap::new();
                m.insert("env".to_string(), "test".to_string());
                m
            },
            custom: vec![CustomMetricConfig {
                name: "test".to_string(),
                metric_type: MetricType::Gauge,
                description: "Test".to_string(),
                labels: vec![],
            }],
        };
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: MetricsConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(config.endpoint, deserialized.endpoint);
        assert_eq!(config.custom.len(), deserialized.custom.len());
    }
}
