//! Core biomeOS Configuration Types
//!
//! This module contains the fundamental configuration structures for biomeOS,
//! including global settings, network configuration, and telemetry settings.

use crate::{PrimalType, SecurityConfig, StorageConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core biomeOS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSConfig {
    /// Global biomeOS settings
    pub global: GlobalConfig,
    /// Primal-specific configurations
    pub primals: HashMap<PrimalType, GlobalConfig>,
    /// Security configuration
    pub security: SecurityConfig,
    /// Network configuration  
    pub networking: NetworkConfig,
    /// Storage configuration
    pub storage: StorageConfig,
}

/// Core network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Network interface to bind to
    pub interface: String,
    /// Port ranges for services
    pub port_ranges: Vec<(u16, u16)>,
    /// Enable IPv6 support
    pub ipv6_enabled: bool,
}

/// Global biomeOS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// biomeOS instance name
    pub instance_name: String,
    /// Data directory for biomeOS
    pub data_dir: String,
    /// Log level
    pub log_level: String,
    /// Enable development mode
    pub dev_mode: bool,
    /// Telemetry settings
    pub telemetry: TelemetryConfig,
}

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Enable telemetry collection
    pub enabled: bool,
    /// Telemetry endpoint
    pub endpoint: Option<String>,
    /// Collection interval in seconds
    pub interval_secs: u64,
}

impl Default for BiomeOSConfig {
    fn default() -> Self {
        Self {
            global: GlobalConfig {
                instance_name: "default-biome".to_string(),
                data_dir: "/var/lib/biomeos".to_string(),
                log_level: "info".to_string(),
                dev_mode: false,
                telemetry: TelemetryConfig {
                    enabled: true,
                    endpoint: None,
                    interval_secs: 60,
                },
            },
            primals: HashMap::new(),
            security: SecurityConfig::default(),
            networking: NetworkConfig {
                interface: "0.0.0.0".to_string(),
                port_ranges: vec![(8080, 8080), (8443, 8443)],
                ipv6_enabled: false,
            },
            storage: StorageConfig::default(),
        }
    }
}
