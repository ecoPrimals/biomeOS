//! BiomeOS Core Library
//!
//! Universal operating system and orchestration layer

use serde::{Deserialize, Serialize};

pub mod ai_first_api;
pub mod byob;
pub mod config;
pub mod config_builder;
pub mod ecosystem_integration;
pub mod ecosystem_licensing;
pub mod integration;
pub mod sovereignty_guardian;
pub mod types;
pub mod universal_biomeos_manager;
pub mod universal_service_registration;

// Re-export key types
pub use byob::ByobManager;
pub use config::{BiomeOSConfig, DiscoveryMethod, OrganizationScale};
pub use config_builder::BiomeOSConfigBuilder;
pub use universal_biomeos_manager::{HealthStatus, SystemHealth, UniversalBiomeOSManager};

// Re-export selected types to avoid ambiguous glob conflicts
pub use ai_first_api::{
    AIErrorCategory, AIFirstError, AIFirstResponse, AIResponseMetadata, HumanInteractionContext,
    SuggestedAction,
};
pub use types::{BiomeMetadata, PrimalConfiguration, ServiceConfiguration, UniversalBiomeManifest};

/// Build information
pub const BUILD_INFO: &str = concat!(
    "BiomeOS Core v",
    env!("CARGO_PKG_VERSION"),
    " built on ",
    "2024"
);

/// BiomeOS error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BiomeError {
    /// Configuration error
    Configuration { message: String },

    /// Invalid input error
    InvalidInput(String),

    /// Primal discovery error  
    Discovery { message: String },

    /// Network communication error
    Network { message: String },

    /// Security/authentication error
    Security { message: String },

    /// Resource management error
    Resource { message: String },

    /// Integration error
    Integration { message: String },

    /// Internal system error
    Internal { message: String },

    /// Unknown error
    Unknown { message: String },
}

impl std::fmt::Display for BiomeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BiomeError::Configuration { message } => write!(f, "Configuration error: {}", message),
            BiomeError::InvalidInput(message) => write!(f, "Invalid input: {}", message),
            BiomeError::Discovery { message } => write!(f, "Discovery error: {}", message),
            BiomeError::Network { message } => write!(f, "Network error: {}", message),
            BiomeError::Security { message } => write!(f, "Security error: {}", message),
            BiomeError::Resource { message } => write!(f, "Resource error: {}", message),
            BiomeError::Integration { message } => write!(f, "Integration error: {}", message),
            BiomeError::Internal { message } => write!(f, "Internal error: {}", message),
            BiomeError::Unknown { message } => write!(f, "Unknown error: {}", message),
        }
    }
}

impl std::error::Error for BiomeError {}

impl From<anyhow::Error> for BiomeError {
    fn from(err: anyhow::Error) -> Self {
        BiomeError::Internal {
            message: err.to_string(),
        }
    }
}

impl From<std::io::Error> for BiomeError {
    fn from(err: std::io::Error) -> Self {
        BiomeError::Internal {
            message: format!("IO error: {}", err),
        }
    }
}

/// Standard result type for BiomeOS operations
pub type BiomeResult<T> = Result<T, BiomeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_creation() {
        let config = BiomeOSConfig::default();
        assert!(!config.primals.discovery.scan_hosts.is_empty());
        assert!(!config.primals.discovery.scan_ports.is_empty());
    }

    #[tokio::test]
    async fn test_manager_creation() {
        let config = BiomeOSConfig::default();
        let manager = UniversalBiomeOSManager::new(config);

        // Test that manager can be created without panicking
        let _result = manager.initialize().await;
    }
}
