//! Configuration types for biomeOS

use serde::{Deserialize, Serialize};

/// Configuration loader trait
pub trait ConfigLoader {
    /// Load configuration from source
    fn load_config(&self) -> Result<Config, ConfigError>;
}

/// Configuration error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigError {
    /// Error message
    pub message: String,
}

/// Main configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Configuration name
    pub name: String,
    /// Configuration data
    pub data: serde_json::Value,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            data: serde_json::Value::Null,
        }
    }
}
