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

/// UI theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub primary_color: String,
    pub secondary_color: String,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            name: "dark".to_string(),
            primary_color: "#1a1a1a".to_string(),
            secondary_color: "#2d2d2d".to_string(),
        }
    }

    pub const DARK: &'static str = "dark";
}

/// UI mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIMode {
    Light,
    Dark,
    Auto,
    Terminal,
    CLI,
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
