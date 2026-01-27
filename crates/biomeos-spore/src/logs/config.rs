//! Log configuration types.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for log management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// Where to store active logs
    pub active_dir: PathBuf,

    /// Where to archive fossil logs
    pub fossil_dir: PathBuf,

    /// Maximum age before auto-archival (seconds)
    pub max_active_age_secs: u64,

    /// Whether to enable BearDog encryption (future)
    pub enable_encryption: bool,

    /// Compression for fossil logs
    pub compress_fossils: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            active_dir: PathBuf::from("/var/biomeos/logs/active"),
            fossil_dir: PathBuf::from("/var/biomeos/logs/fossil"),
            max_active_age_secs: 86400, // 24 hours
            enable_encryption: false,   // Future feature
            compress_fossils: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LogConfig::default();
        assert_eq!(config.max_active_age_secs, 86400);
        assert!(config.compress_fossils);
    }
}
