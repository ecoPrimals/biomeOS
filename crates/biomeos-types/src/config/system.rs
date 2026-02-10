//! System Configuration
//!
//! This module contains system-level configuration types including
//! environment settings, timeouts, worker configuration, and system limits.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// System environment types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Environment {
    /// Development environment
    Development,
    /// Testing environment
    Testing,
    /// Staging environment
    Staging,
    /// Production environment
    Production,
    /// Custom environment
    Custom(String),
}

/// Organization scale for tuning behavior
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrganizationScale {
    /// Individual user
    Individual,
    /// Small team
    Team,
    /// Department level
    Department,
    /// Enterprise level
    Enterprise,
    /// Global scale
    Global,
}

/// System-wide configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// System name
    pub name: String,

    /// System environment (development, staging, production)
    pub environment: Environment,

    /// Organization scale
    pub organization_scale: OrganizationScale,

    /// Default timeouts
    pub timeouts: TimeoutConfig,

    /// Worker and thread pool configuration
    pub workers: WorkerConfig,

    /// Temporary directory configuration
    pub temp_dir: Option<PathBuf>,

    /// Data directory configuration
    pub data_dir: PathBuf,

    /// Configuration directory
    pub config_dir: PathBuf,

    /// Log directory
    pub log_dir: PathBuf,

    /// System limits
    pub limits: SystemLimits,
}

/// Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Default request timeout
    pub default_request_timeout: Duration,

    /// Connection timeout
    pub connection_timeout: Duration,

    /// Read timeout
    pub read_timeout: Duration,

    /// Write timeout
    pub write_timeout: Duration,

    /// Health check timeout
    pub health_check_timeout: Duration,

    /// Service discovery timeout
    pub discovery_timeout: Duration,

    /// Graceful shutdown timeout
    pub shutdown_timeout: Duration,
}

/// Worker configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkerConfig {
    /// Number of worker threads
    pub worker_threads: Option<usize>,

    /// Maximum blocking threads
    pub max_blocking_threads: Option<usize>,

    /// Thread stack size
    pub thread_stack_size: Option<usize>,

    /// Thread keep-alive time
    pub thread_keep_alive: Option<Duration>,

    /// Work queue size
    pub queue_size: Option<usize>,
}

/// System limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemLimits {
    /// Maximum concurrent connections
    pub max_connections: usize,

    /// Maximum request size in bytes
    pub max_request_size: usize,

    /// Maximum response size in bytes
    pub max_response_size: usize,

    /// Maximum file upload size
    pub max_upload_size: usize,

    /// Maximum memory usage
    pub max_memory_usage: Option<usize>,

    /// Maximum CPU usage (0.0-1.0)
    pub max_cpu_usage: Option<f64>,

    /// Maximum disk usage
    pub max_disk_usage: Option<usize>,
}

/// Default implementations
impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            name: "biomeos".to_string(),
            environment: Environment::Development,
            organization_scale: OrganizationScale::Individual,
            timeouts: TimeoutConfig::default(),
            workers: WorkerConfig::default(),
            temp_dir: None,
            data_dir: PathBuf::from("./data"),
            config_dir: PathBuf::from("./config"),
            log_dir: PathBuf::from("./logs"),
            limits: SystemLimits::default(),
        }
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            default_request_timeout: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(10),
            read_timeout: Duration::from_secs(30),
            write_timeout: Duration::from_secs(30),
            health_check_timeout: Duration::from_secs(5),
            discovery_timeout: Duration::from_secs(10),
            shutdown_timeout: Duration::from_secs(30),
        }
    }
}

// WorkerConfig Default derived via #[derive(Default)]

impl Default for SystemLimits {
    fn default() -> Self {
        Self {
            max_connections: 1000,
            max_request_size: 1024 * 1024 * 10,  // 10MB
            max_response_size: 1024 * 1024 * 10, // 10MB
            max_upload_size: 1024 * 1024 * 100,  // 100MB
            max_memory_usage: None,
            max_cpu_usage: None,
            max_disk_usage: None,
        }
    }
}
