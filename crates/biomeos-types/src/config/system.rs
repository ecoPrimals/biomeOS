// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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

#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════════════
    // Environment Enum Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_environment_variants() {
        assert_eq!(Environment::Development, Environment::Development);
        assert_eq!(Environment::Testing, Environment::Testing);
        assert_eq!(Environment::Staging, Environment::Staging);
        assert_eq!(Environment::Production, Environment::Production);
        assert_eq!(
            Environment::Custom("beta".to_string()),
            Environment::Custom("beta".to_string())
        );
    }

    #[test]
    fn test_environment_serialization() {
        for env in [
            Environment::Development,
            Environment::Testing,
            Environment::Staging,
            Environment::Production,
            Environment::Custom("custom-env".to_string()),
        ] {
            let json = serde_json::to_string(&env).expect("serialize");
            let deserialized: Environment = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(env, deserialized);
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // OrganizationScale Enum Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_organization_scale_variants() {
        assert_eq!(OrganizationScale::Individual, OrganizationScale::Individual);
        assert_eq!(OrganizationScale::Team, OrganizationScale::Team);
        assert_eq!(OrganizationScale::Department, OrganizationScale::Department);
        assert_eq!(OrganizationScale::Enterprise, OrganizationScale::Enterprise);
        assert_eq!(OrganizationScale::Global, OrganizationScale::Global);
    }

    #[test]
    fn test_organization_scale_serialization() {
        for scale in [
            OrganizationScale::Individual,
            OrganizationScale::Team,
            OrganizationScale::Department,
            OrganizationScale::Enterprise,
            OrganizationScale::Global,
        ] {
            let json = serde_json::to_string(&scale).expect("serialize");
            let deserialized: OrganizationScale = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(scale, deserialized);
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SystemConfig Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_system_config_default() {
        let config = SystemConfig::default();
        assert_eq!(config.name, "biomeos");
        assert_eq!(config.environment, Environment::Development);
        assert_eq!(config.organization_scale, OrganizationScale::Individual);
        assert!(config.temp_dir.is_none());
        assert_eq!(config.data_dir, PathBuf::from("./data"));
        assert_eq!(config.config_dir, PathBuf::from("./config"));
        assert_eq!(config.log_dir, PathBuf::from("./logs"));
    }

    #[test]
    fn test_system_config_serialization() {
        let config = SystemConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: SystemConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.name, "biomeos");
    }

    #[test]
    fn test_system_config_clone() {
        let original = SystemConfig::default();
        let cloned = original.clone();
        assert_eq!(cloned.name, original.name);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // TimeoutConfig Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_timeout_config_default() {
        let config = TimeoutConfig::default();
        assert_eq!(config.default_request_timeout, Duration::from_secs(30));
        assert_eq!(config.connection_timeout, Duration::from_secs(10));
        assert_eq!(config.read_timeout, Duration::from_secs(30));
        assert_eq!(config.write_timeout, Duration::from_secs(30));
        assert_eq!(config.health_check_timeout, Duration::from_secs(5));
        assert_eq!(config.discovery_timeout, Duration::from_secs(10));
        assert_eq!(config.shutdown_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_timeout_config_serialization() {
        let config = TimeoutConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: TimeoutConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.connection_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_timeout_config_custom_values() {
        let config = TimeoutConfig {
            default_request_timeout: Duration::from_secs(60),
            connection_timeout: Duration::from_secs(5),
            read_timeout: Duration::from_secs(15),
            write_timeout: Duration::from_secs(15),
            health_check_timeout: Duration::from_secs(2),
            discovery_timeout: Duration::from_secs(20),
            shutdown_timeout: Duration::from_secs(60),
        };
        assert_eq!(config.default_request_timeout.as_secs(), 60);
        assert_eq!(config.health_check_timeout.as_secs(), 2);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // WorkerConfig Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_worker_config_default() {
        let config = WorkerConfig::default();
        assert!(config.worker_threads.is_none());
        assert!(config.max_blocking_threads.is_none());
        assert!(config.thread_stack_size.is_none());
        assert!(config.thread_keep_alive.is_none());
        assert!(config.queue_size.is_none());
    }

    #[test]
    fn test_worker_config_serialization() {
        let config = WorkerConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: WorkerConfig = serde_json::from_str(&json).expect("deserialize");
        assert!(deserialized.worker_threads.is_none());
    }

    #[test]
    fn test_worker_config_custom_values() {
        let config = WorkerConfig {
            worker_threads: Some(8),
            max_blocking_threads: Some(512),
            thread_stack_size: Some(2 * 1024 * 1024),
            thread_keep_alive: Some(Duration::from_secs(60)),
            queue_size: Some(1024),
        };
        assert_eq!(config.worker_threads, Some(8));
        assert_eq!(config.max_blocking_threads, Some(512));
        assert_eq!(config.queue_size, Some(1024));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SystemLimits Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_system_limits_default() {
        let limits = SystemLimits::default();
        assert_eq!(limits.max_connections, 1000);
        assert_eq!(limits.max_request_size, 10 * 1024 * 1024);
        assert_eq!(limits.max_response_size, 10 * 1024 * 1024);
        assert_eq!(limits.max_upload_size, 100 * 1024 * 1024);
        assert!(limits.max_memory_usage.is_none());
        assert!(limits.max_cpu_usage.is_none());
        assert!(limits.max_disk_usage.is_none());
    }

    #[test]
    fn test_system_limits_serialization() {
        let limits = SystemLimits::default();
        let json = serde_json::to_string(&limits).expect("serialize");
        let deserialized: SystemLimits = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.max_connections, 1000);
    }

    #[test]
    fn test_system_limits_custom_values() {
        let limits = SystemLimits {
            max_connections: 5000,
            max_request_size: 50 * 1024 * 1024,
            max_response_size: 50 * 1024 * 1024,
            max_upload_size: 500 * 1024 * 1024,
            max_memory_usage: Some(8 * 1024 * 1024 * 1024),
            max_cpu_usage: Some(0.8),
            max_disk_usage: Some(100 * 1024 * 1024 * 1024),
        };
        assert_eq!(limits.max_connections, 5000);
        assert_eq!(limits.max_cpu_usage, Some(0.8));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Debug Trait Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_environment_debug() {
        let env = Environment::Production;
        let debug = format!("{env:?}");
        assert!(debug.contains("Production"));
    }

    #[test]
    fn test_organization_scale_debug() {
        let scale = OrganizationScale::Enterprise;
        let debug = format!("{scale:?}");
        assert!(debug.contains("Enterprise"));
    }

    #[test]
    fn test_system_config_debug() {
        let config = SystemConfig::default();
        let debug = format!("{config:?}");
        assert!(debug.contains("biomeos"));
    }
}
