//! Storage types for biomeOS

use serde::{Deserialize, Serialize};

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Enable storage
    pub enabled: bool,
    /// Data directory
    pub data_dir: String,
    /// Storage type
    pub storage_type: StorageType,
}

/// Storage types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageType {
    /// Local filesystem
    Local,
    /// Distributed storage
    Distributed,
    /// Cloud storage
    Cloud,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            data_dir: "/var/lib/biomeos".to_string(),
            storage_type: StorageType::Local,
        }
    }
}
