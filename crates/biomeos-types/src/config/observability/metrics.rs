// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
