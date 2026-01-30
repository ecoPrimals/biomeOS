//! Capability-based primal system
//!
//! Defines capabilities that primals can provide/require.
//! NO hardcoded primal names - only capabilities!

use std::fmt;

use serde::{Deserialize, Serialize};

/// Capabilities that primals can provide or require
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    /// Cryptographic security services (signing, encryption, key management)
    Security,

    /// Discovery and orchestration services
    Discovery,

    /// Compute/execution services
    Compute,

    /// AI/ML inference services
    AI,

    /// Storage services (content-addressed, distributed, etc.)
    Storage,

    /// Observability services (metrics, logging, tracing)
    Observability,

    /// Federation/multi-org coordination
    Federation,

    /// Network services (NAT traversal, routing, etc.)
    Network,

    /// Custom capability (for extension)
    Custom(String),
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Capability::Security => write!(f, "security"),
            Capability::Discovery => write!(f, "discovery"),
            Capability::Compute => write!(f, "compute"),
            Capability::AI => write!(f, "ai"),
            Capability::Storage => write!(f, "storage"),
            Capability::Observability => write!(f, "observability"),
            Capability::Federation => write!(f, "federation"),
            Capability::Network => write!(f, "network"),
            Capability::Custom(s) => write!(f, "custom:{}", s),
        }
    }
}

impl std::str::FromStr for Capability {
    type Err = std::convert::Infallible;

    /// Parse capability from string (implements FromStr trait)
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "security" => Capability::Security,
            "discovery" => Capability::Discovery,
            "compute" => Capability::Compute,
            "ai" => Capability::AI,
            "storage" => Capability::Storage,
            "observability" => Capability::Observability,
            "federation" => Capability::Federation,
            "network" => Capability::Network,
            _ => Capability::Custom(s.to_string()),
        })
    }
}

impl Capability {
    /// Load from environment variable (comma-separated)
    pub fn from_env(var_name: &str) -> Vec<Self> {
        std::env::var(var_name)
            .ok()
            .map(|s| {
                s.split(',')
                    .filter_map(|cap| cap.trim().parse().ok())
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Primal configuration loaded from environment
#[derive(Debug, Clone)]
pub struct PrimalConfig {
    /// Unique identifier (from PRIMAL_ID or auto-generated)
    pub id: String,

    /// Binary path (from PRIMAL_BINARY or argv[0])
    pub binary_path: String,

    /// Capabilities this primal provides
    pub provides: Vec<Capability>,

    /// Capabilities this primal requires
    pub requires: Vec<Capability>,

    /// HTTP port (0 = auto-select, from HTTP_PORT)
    pub http_port: u16,

    /// Additional environment-specific config
    pub env_config: std::collections::HashMap<String, String>,
}

impl PrimalConfig {
    /// Load configuration from environment (infant model - zero hardcoding!)
    pub fn from_env() -> biomeos_types::error::BiomeResult<Self> {
        use biomeos_types::error::BiomeError;

        // Discover own identity
        let id = Self::discover_identity()?;

        // Discover binary path
        let binary_path = std::env::var("PRIMAL_BINARY").or_else(|_| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.to_str().map(String::from))
                .ok_or_else(|| {
                    BiomeError::config_error("Cannot determine binary path", Some("PRIMAL_BINARY"))
                })
        })?;

        // Load capabilities
        let provides = Capability::from_env("PRIMAL_PROVIDES");
        let requires = Capability::from_env("PRIMAL_REQUIRES");

        // Load port (0 = OS auto-select)
        let http_port = std::env::var("HTTP_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        // Collect all environment variables for extensibility
        let env_config = std::env::vars()
            .filter(|(k, _)| k.starts_with("PRIMAL_"))
            .collect();

        Ok(Self {
            id,
            binary_path,
            provides,
            requires,
            http_port,
            env_config,
        })
    }

    /// Discover own identity (infant model)
    fn discover_identity() -> biomeos_types::error::BiomeResult<String> {
        // 1. Check PRIMAL_ID env var
        if let Ok(id) = std::env::var("PRIMAL_ID") {
            return Ok(id);
        }

        // 2. Generate from binary name + hostname
        let binary_name = std::env::current_exe()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "unknown".to_string());

        let hostname = hostname::get()
            .ok()
            .and_then(|h| h.to_str().map(String::from))
            .unwrap_or_else(|| "unknown".to_string());

        // Create unique ID: binary@hostname-random
        let random_suffix = uuid::Uuid::new_v4()
            .to_string()
            .chars()
            .take(8)
            .collect::<String>();

        Ok(format!("{}@{}-{}", binary_name, hostname, random_suffix))
    }

    /// Create config for a specific primal type (for manual construction)
    pub fn for_capability(provides: Vec<Capability>, requires: Vec<Capability>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            binary_path: String::new(),
            provides,
            requires,
            http_port: 0,
            env_config: std::collections::HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_capability_parsing() {
        assert_eq!(
            Capability::from_str("security").unwrap(),
            Capability::Security
        );
        assert_eq!(
            Capability::from_str("DISCOVERY").unwrap(),
            Capability::Discovery
        );
        assert_eq!(
            Capability::from_str("custom").unwrap(),
            Capability::Custom("custom".to_string())
        );

        // Also test via parse()
        assert_eq!("ai".parse::<Capability>().unwrap(), Capability::AI);
    }

    #[test]
    fn test_capability_display() {
        assert_eq!(Capability::Security.to_string(), "security");
        assert_eq!(Capability::Discovery.to_string(), "discovery");
    }
}
