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
    Multiple(Vec<Self>),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_log_level_serde() {
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
    fn test_log_format_serde() {
        for format in [
            LogFormat::Json,
            LogFormat::Plain,
            LogFormat::Pretty,
            LogFormat::Compact,
            LogFormat::Custom("%t %m".to_string()),
        ] {
            let json = serde_json::to_string(&format).expect("serialize");
            let _: LogFormat = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_log_destination_serde() {
        let destinations = [
            LogDestination::Stdout,
            LogDestination::Stderr,
            LogDestination::File(PathBuf::from("/var/log/app.log")),
        ];
        for dest in destinations {
            let json = serde_json::to_string(&dest).expect("serialize");
            let _: LogDestination = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_syslog_protocol_serde() {
        for proto in [
            SyslogProtocol::Udp,
            SyslogProtocol::Tcp,
            SyslogProtocol::Tls,
        ] {
            let json = serde_json::to_string(&proto).expect("serialize");
            let _: SyslogProtocol = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_rotation_schedule_serde() {
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
    fn test_log_filter_action_serde() {
        for action in [
            LogFilterAction::Allow,
            LogFilterAction::Deny,
            LogFilterAction::Transform("template".to_string()),
        ] {
            let json = serde_json::to_string(&action).expect("serialize");
            let _: LogFilterAction = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_sampling_strategy_serde() {
        for strategy in [
            SamplingStrategy::Random,
            SamplingStrategy::Deterministic,
            SamplingStrategy::RateLimited { rate: 100 },
            SamplingStrategy::Custom("custom".to_string()),
        ] {
            let json = serde_json::to_string(&strategy).expect("serialize");
            let _: SamplingStrategy = serde_json::from_str(&json).expect("deserialize");
        }
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
    fn test_logging_config_serde_roundtrip() {
        let config = LoggingConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: LoggingConfig = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(deserialized.level, LogLevel::Info));
        assert!(deserialized.structured);
    }

    #[test]
    fn test_log_rotation_config_default() {
        let config = LogRotationConfig::default();
        assert_eq!(config.max_size, 100 * 1024 * 1024);
        assert_eq!(config.max_files, 10);
        assert!(config.compress);
        assert!(config.schedule.is_none());
    }
}
