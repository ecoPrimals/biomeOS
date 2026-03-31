// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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
            Self::Security => write!(f, "security"),
            Self::Discovery => write!(f, "discovery"),
            Self::Compute => write!(f, "compute"),
            Self::AI => write!(f, "ai"),
            Self::Storage => write!(f, "storage"),
            Self::Observability => write!(f, "observability"),
            Self::Federation => write!(f, "federation"),
            Self::Network => write!(f, "network"),
            Self::Custom(s) => write!(f, "custom:{s}"),
        }
    }
}

impl std::str::FromStr for Capability {
    type Err = std::convert::Infallible;

    /// Parse capability from string (implements `FromStr` trait)
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "security" => Self::Security,
            "discovery" => Self::Discovery,
            "compute" => Self::Compute,
            "ai" => Self::AI,
            "storage" => Self::Storage,
            "observability" => Self::Observability,
            "federation" => Self::Federation,
            "network" => Self::Network,
            _ => Self::Custom(s.to_string()),
        })
    }
}

impl Capability {
    /// Load from environment variable (comma-separated)
    #[must_use]
    pub fn from_env(var_name: &str) -> Vec<Self> {
        std::env::var(var_name)
            .ok()
            .map(|s| Self::from_csv_list(&s))
            .unwrap_or_default()
    }

    /// Parse comma-separated capability list (same rules as [`Self::from_env`]).
    #[must_use]
    pub fn from_csv_list(raw: &str) -> Vec<Self> {
        raw.split(',')
            .filter_map(|cap| cap.trim().parse().ok())
            .collect()
    }
}

/// Primal configuration loaded from environment
#[derive(Debug, Clone)]
pub struct PrimalConfig {
    /// Unique identifier (from `PRIMAL_ID` or auto-generated)
    pub id: String,

    /// Binary path (from `PRIMAL_BINARY` or `argv[0]`)
    pub binary_path: String,

    /// Capabilities this primal provides
    pub provides: Vec<Capability>,

    /// Capabilities this primal requires
    pub requires: Vec<Capability>,

    /// HTTP port (0 = auto-select, from `HTTP_PORT`)
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

        let hostname = gethostname::gethostname()
            .to_str()
            .map_or_else(|| "unknown".to_string(), String::from);

        // Create unique ID: binary@hostname-random
        let random_suffix = uuid::Uuid::new_v4()
            .to_string()
            .chars()
            .take(8)
            .collect::<String>();

        Ok(format!("{binary_name}@{hostname}-{random_suffix}"))
    }

    /// Create config for a specific primal type (for manual construction)
    #[must_use]
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
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
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
    fn test_capability_parsing_all_variants() {
        assert_eq!(
            Capability::from_str("compute").unwrap(),
            Capability::Compute
        );
        assert_eq!(
            Capability::from_str("storage").unwrap(),
            Capability::Storage
        );
        assert_eq!(
            Capability::from_str("observability").unwrap(),
            Capability::Observability
        );
        assert_eq!(
            Capability::from_str("federation").unwrap(),
            Capability::Federation
        );
        assert_eq!(
            Capability::from_str("network").unwrap(),
            Capability::Network
        );
    }

    #[test]
    fn test_capability_parsing_case_insensitive() {
        assert_eq!(
            Capability::from_str("SECURITY").unwrap(),
            Capability::Security
        );
        assert_eq!(
            Capability::from_str("SeCuRiTy").unwrap(),
            Capability::Security
        );
        assert_eq!(
            Capability::from_str("Network").unwrap(),
            Capability::Network
        );
    }

    #[test]
    fn test_capability_parsing_unknown_becomes_custom() {
        let cap = Capability::from_str("unknown_capability").unwrap();
        assert_eq!(cap, Capability::Custom("unknown_capability".to_string()));
    }

    #[test]
    fn test_capability_display() {
        assert_eq!(Capability::Security.to_string(), "security");
        assert_eq!(Capability::Discovery.to_string(), "discovery");
    }

    #[test]
    fn test_capability_display_all_variants() {
        assert_eq!(Capability::Compute.to_string(), "compute");
        assert_eq!(Capability::AI.to_string(), "ai");
        assert_eq!(Capability::Storage.to_string(), "storage");
        assert_eq!(Capability::Observability.to_string(), "observability");
        assert_eq!(Capability::Federation.to_string(), "federation");
        assert_eq!(Capability::Network.to_string(), "network");
        assert_eq!(
            Capability::Custom("myservice".to_string()).to_string(),
            "custom:myservice"
        );
    }

    #[test]
    fn test_capability_equality() {
        assert_eq!(Capability::Security, Capability::Security);
        assert_ne!(Capability::Security, Capability::Discovery);
        assert_eq!(
            Capability::Custom("test".to_string()),
            Capability::Custom("test".to_string())
        );
        assert_ne!(
            Capability::Custom("test1".to_string()),
            Capability::Custom("test2".to_string())
        );
    }

    #[test]
    fn test_capability_clone() {
        let cap = Capability::Storage;
        let cloned = cap.clone();
        assert_eq!(cap, cloned);

        let custom = Capability::Custom("test".to_string());
        let cloned_custom = custom.clone();
        assert_eq!(custom, cloned_custom);
    }

    #[test]
    fn test_capability_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Capability::Security);
        set.insert(Capability::Discovery);
        set.insert(Capability::Security); // duplicate

        assert_eq!(set.len(), 2);
        assert!(set.contains(&Capability::Security));
        assert!(set.contains(&Capability::Discovery));
    }

    #[test]
    fn test_capability_serialization() {
        let cap = Capability::Security;
        let json = serde_json::to_string(&cap).unwrap();
        assert_eq!(json, "\"Security\"");

        let deserialized: Capability = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, cap);
    }

    #[test]
    fn test_capability_custom_serialization() {
        let cap = Capability::Custom("myservice".to_string());
        let json = serde_json::to_string(&cap).unwrap();
        assert!(json.contains("myservice"));

        let deserialized: Capability = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, cap);
    }

    #[test]
    fn test_capability_from_env_empty() {
        // Unique key — extremely unlikely to exist in the process environment.
        let caps = Capability::from_env("TEST_CAP_EMPTY_1234");
        assert!(caps.is_empty());
    }

    #[test]
    fn test_primal_config_for_capability() {
        let config = PrimalConfig::for_capability(
            vec![Capability::Security, Capability::Compute],
            vec![Capability::Discovery],
        );

        assert_eq!(config.provides.len(), 2);
        assert!(config.provides.contains(&Capability::Security));
        assert!(config.provides.contains(&Capability::Compute));
        assert_eq!(config.requires.len(), 1);
        assert!(config.requires.contains(&Capability::Discovery));
        assert_eq!(config.http_port, 0);
        assert!(config.env_config.is_empty());
    }

    #[test]
    fn test_primal_config_for_capability_empty() {
        let config = PrimalConfig::for_capability(vec![], vec![]);

        assert!(config.provides.is_empty());
        assert!(config.requires.is_empty());
        assert!(!config.id.is_empty()); // Should have a UUID
    }

    #[test]
    fn test_primal_config_id_is_uuid() {
        let config = PrimalConfig::for_capability(vec![], vec![]);

        // UUID format: 8-4-4-4-12 hex characters
        assert!(config.id.len() == 36);
        assert!(config.id.chars().filter(|c| *c == '-').count() == 4);
    }

    #[test]
    fn test_capability_from_env_multiple() {
        let caps = Capability::from_csv_list("security,compute,storage");
        assert_eq!(caps.len(), 3);
        assert!(caps.contains(&Capability::Security));
        assert!(caps.contains(&Capability::Compute));
        assert!(caps.contains(&Capability::Storage));
    }

    #[test]
    fn test_capability_from_env_with_spaces() {
        let caps = Capability::from_csv_list("  ai  ,  network  ");
        assert_eq!(caps.len(), 2);
        assert!(caps.contains(&Capability::AI));
        assert!(caps.contains(&Capability::Network));
    }

    #[test]
    fn test_primal_config_from_env_with_primal_id() {
        let config = PrimalConfig {
            id: "test-primal-123".to_string(),
            binary_path: "/usr/bin/test-primal".to_string(),
            provides: Capability::from_csv_list("security"),
            requires: Capability::from_csv_list("discovery"),
            http_port: 8080,
            env_config: std::collections::HashMap::new(),
        };
        assert_eq!(config.id, "test-primal-123");
        assert_eq!(config.binary_path, "/usr/bin/test-primal");
        assert_eq!(config.http_port, 8080);
        assert!(config.provides.contains(&Capability::Security));
        assert!(config.requires.contains(&Capability::Discovery));
    }
}
