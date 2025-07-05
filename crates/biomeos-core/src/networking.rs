//! Networking types for biomeOS

use serde::{Deserialize, Serialize};

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Enable networking
    pub enabled: bool,
    /// Bind address
    pub bind_address: String,
    /// Port
    pub port: u16,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bind_address: "0.0.0.0".to_string(),
            port: 8080,
        }
    }
}
