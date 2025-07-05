//! Security types for biomeOS

use serde::{Deserialize, Serialize};

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Authentication enabled
    pub auth_enabled: bool,
    /// Encryption enabled
    pub encryption_enabled: bool,
    /// Security level
    pub security_level: SecurityLevel,
}

/// Security levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Basic security
    Basic,
    /// High security
    High,
    /// Maximum security
    Maximum,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            auth_enabled: true,
            encryption_enabled: true,
            security_level: SecurityLevel::High,
        }
    }
}
