//! Monitoring Configuration Module

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Metrics configuration
    pub metrics: MetricsSpec,
    /// Logging configuration
    pub logging: LoggingSpec,
    /// Tracing configuration
    pub tracing: TracingSpec,
    /// Alerting configuration
    pub alerting: AlertingSpec,
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
    /// Metrics storage
    pub storage: MetricsStorage,
}

/// Logging specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingSpec {
    /// Enable logging
    pub enabled: bool,
    /// Log level
    pub level: String,
    /// Log format
    pub format: String,
    /// Log storage
    pub storage: LogStorage,
}

/// Tracing specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingSpec {
    /// Enable tracing
    pub enabled: bool,
    /// Tracing endpoint
    pub endpoint: String,
    /// Tracing storage
    pub storage: TracingStorage,
}

/// Alerting specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingSpec {
    /// Enable alerting
    pub enabled: bool,
    /// Alert rules
    pub rules: Vec<AlertRule>,
    /// Alert channels
    pub channels: Vec<AlertChannel>,
}

/// Alert rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Rule name
    pub name: String,
    /// Rule expression
    pub expression: String,
    /// Alert severity
    pub severity: String,
    /// Alert threshold
    pub threshold: f64,
}

/// Alert channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertChannel {
    /// Channel name
    pub name: String,
    /// Channel type
    pub channel_type: String,
    /// Channel configuration
    pub config: HashMap<String, String>,
}

/// Metrics storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsStorage {
    /// Storage type
    pub storage_type: String,
    /// Storage configuration
    pub config: HashMap<String, String>,
}

/// Log storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogStorage {
    /// Storage type
    pub storage_type: String,
    /// Storage configuration
    pub config: HashMap<String, String>,
}

/// Tracing storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingStorage {
    /// Storage type
    pub storage_type: String,
    /// Storage configuration
    pub config: HashMap<String, String>,
}

/// Monitoring specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringSpec {
    /// Monitoring configuration
    pub config: MonitoringConfig,
    /// Enable monitoring
    pub enabled: bool,
} 