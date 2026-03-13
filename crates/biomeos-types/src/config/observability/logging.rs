// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Logging configuration types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

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
