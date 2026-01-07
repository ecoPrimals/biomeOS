// biomeOS Tower Configuration
//
// Modern, idiomatic, platform-agnostic configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Tower configuration - loaded from tower.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerConfig {
    /// Tower metadata
    #[serde(default)]
    pub tower: TowerMeta,
    
    /// Primal configurations
    #[serde(default)]
    pub primals: Vec<PrimalConfig>,
    
    /// Discovery settings
    #[serde(default)]
    pub discovery: DiscoveryConfig,
    
    /// Health monitoring settings
    #[serde(default)]
    pub health: HealthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerMeta {
    /// Tower name (defaults to hostname)
    pub name: Option<String>,
    
    /// Family ID for genetic lineage
    pub family: Option<String>,
    
    /// Enable concurrent primal startup
    #[serde(default = "default_true")]
    pub concurrent_startup: bool,
}

impl Default for TowerMeta {
    fn default() -> Self {
        Self {
            name: None,
            family: None,
            concurrent_startup: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalConfig {
    /// Binary path (relative or absolute)
    pub binary: PathBuf,
    
    /// Optional primal ID (defaults to binary name)
    pub id: Option<String>,
    
    /// Capabilities this primal provides
    #[serde(default)]
    pub provides: Vec<String>,
    
    /// Capabilities this primal requires
    #[serde(default)]
    pub requires: Vec<String>,
    
    /// HTTP port (0 = auto, omit for port-free)
    #[serde(default)]
    pub http_port: u16,
    
    /// IPC protocol (optional: "tarpc", "jsonrpc", or auto-detect)
    /// Used for inter-primal communication over Unix sockets
    /// - "tarpc": Type-safe, high-performance (Rust ↔ Rust)
    /// - "jsonrpc": Flexible, debuggable (cross-language, development)
    /// - Auto-detect if not specified (recommended)
    #[serde(default)]
    pub protocol: Option<String>,
    
    /// Environment variables for this primal
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
    
    /// Auto-discover capabilities by querying binary
    #[serde(default = "default_true")]
    pub auto_discover: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Directories to scan for primals
    #[serde(default)]
    pub scan_dirs: Vec<PathBuf>,
    
    /// Auto-register discovered primals
    #[serde(default)]
    pub auto_register: bool,
    
    /// Query binaries for capabilities
    #[serde(default = "default_true")]
    pub query_capabilities: bool,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            scan_dirs: vec![],
            auto_register: false,
            query_capabilities: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Health check interval in seconds
    #[serde(default = "default_health_interval")]
    pub interval_secs: u64,
    
    /// Health check timeout in seconds
    #[serde(default = "default_health_timeout")]
    pub timeout_secs: u64,
    
    /// Max recovery attempts
    #[serde(default = "default_recovery_attempts")]
    pub recovery_attempts: u32,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            interval_secs: 30,
            timeout_secs: 5,
            recovery_attempts: 3,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_health_interval() -> u64 {
    30
}

fn default_health_timeout() -> u64 {
    5
}

fn default_recovery_attempts() -> u32 {
    3
}

impl TowerConfig {
    /// Load configuration from TOML file
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, anyhow::Error> {
        let contents = std::fs::read_to_string(path)?;
        let config: TowerConfig = toml::from_str(&contents)?;
        Ok(config)
    }
    
    /// Load from TOML string
    pub fn from_toml(contents: &str) -> Result<Self, anyhow::Error> {
        let config: TowerConfig = toml::from_str(contents)?;
        Ok(config)
    }
    
    /// Create default configuration
    pub fn default_config() -> Self {
        Self {
            tower: TowerMeta::default(),
            primals: vec![],
            discovery: DiscoveryConfig::default(),
            health: HealthConfig::default(),
        }
    }
    
    /// Get health check interval as Duration
    pub fn health_interval(&self) -> Duration {
        Duration::from_secs(self.health.interval_secs)
    }
    
    /// Get health check timeout as Duration
    pub fn health_timeout(&self) -> Duration {
        Duration::from_secs(self.health.timeout_secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_basic_config() {
        let toml = r#"
[tower]
name = "tower1"
family = "nat0"
concurrent_startup = true

[[primals]]
binary = "./primals/beardog"
provides = ["Security", "Encryption"]
requires = []
http_port = 9000

[[primals]]
binary = "./primals/songbird"
provides = ["Discovery"]
requires = ["Security"]

[health]
interval_secs = 30
timeout_secs = 5
"#;
        
        let config = TowerConfig::from_toml(toml).unwrap();
        assert_eq!(config.tower.name, Some("tower1".to_string()));
        assert_eq!(config.tower.family, Some("nat0".to_string()));
        assert_eq!(config.primals.len(), 2);
        assert_eq!(config.primals[0].provides.len(), 2);
        assert_eq!(config.health.interval_secs, 30);
    }
    
    #[test]
    fn test_default_values() {
        let toml = r#"
[[primals]]
binary = "./primals/test"
"#;
        
        let config = TowerConfig::from_toml(toml).unwrap();
        assert!(config.tower.concurrent_startup);
        assert_eq!(config.health.interval_secs, 30);
        assert!(config.primals[0].auto_discover);
        assert!(config.primals[0].protocol.is_none()); // Default: no protocol (auto-detect)
    }
    
    #[test]
    fn test_protocol_field_tarpc() {
        let toml = r#"
[[primals]]
binary = "./primals/beardog"
provides = ["Security"]
protocol = "tarpc"

[primals.env]
BEARDOG_NODE_ID = "test"
"#;
        
        let config = TowerConfig::from_toml(toml).unwrap();
        assert_eq!(config.primals.len(), 1);
        assert_eq!(config.primals[0].protocol, Some("tarpc".to_string()));
    }
    
    #[test]
    fn test_protocol_field_jsonrpc() {
        let toml = r#"
[[primals]]
binary = "./primals/songbird"
provides = ["Discovery"]
protocol = "jsonrpc"

[primals.env]
SONGBIRD_NODE_ID = "test"
"#;
        
        let config = TowerConfig::from_toml(toml).unwrap();
        assert_eq!(config.primals.len(), 1);
        assert_eq!(config.primals[0].protocol, Some("jsonrpc".to_string()));
    }
    
    #[test]
    fn test_protocol_field_omitted() {
        let toml = r#"
[[primals]]
binary = "./primals/beardog"
provides = ["Security"]

[primals.env]
BEARDOG_NODE_ID = "test"
"#;
        
        let config = TowerConfig::from_toml(toml).unwrap();
        assert_eq!(config.primals.len(), 1);
        assert!(config.primals[0].protocol.is_none()); // Omitted = auto-detect
    }
    
    #[test]
    fn test_dual_protocol_configuration() {
        let toml = r#"
[tower]
family = "nat0"
concurrent_startup = true

[[primals]]
binary = "./primals/beardog"
provides = ["Security"]
protocol = "tarpc"

[primals.env]
BEARDOG_FAMILY_ID = "nat0"
BEARDOG_NODE_ID = "tower1"

[[primals]]
binary = "./primals/songbird"
provides = ["Discovery"]
requires = ["Security"]
protocol = "jsonrpc"

[primals.env]
SONGBIRD_NODE_ID = "tower1"
SECURITY_ENDPOINT = "jsonrpc+unix:///tmp/beardog.sock"
"#;
        
        let config = TowerConfig::from_toml(toml).unwrap();
        assert_eq!(config.primals.len(), 2);
        
        // BearDog with tarpc
        assert_eq!(config.primals[0].protocol, Some("tarpc".to_string()));
        assert_eq!(config.primals[0].provides, vec!["Security"]);
        
        // Songbird with JSON-RPC
        assert_eq!(config.primals[1].protocol, Some("jsonrpc".to_string()));
        assert_eq!(config.primals[1].requires, vec!["Security"]);
        assert_eq!(
            config.primals[1].env.get("SECURITY_ENDPOINT"),
            Some(&"jsonrpc+unix:///tmp/beardog.sock".to_string())
        );
    }
    
    #[test]
    fn test_fractal_deployment_mixed_protocols() {
        let toml = r#"
[tower]
family = "nat0"

[[primals]]
binary = "./primals/beardog"
protocol = "tarpc"

[[primals]]
binary = "./primals/songbird"
protocol = "tarpc"

[[primals]]
binary = "./primals/toadstool"
protocol = "jsonrpc"
"#;
        
        let config = TowerConfig::from_toml(toml).unwrap();
        assert_eq!(config.primals.len(), 3);
        
        // Core primals: tarpc
        assert_eq!(config.primals[0].protocol, Some("tarpc".to_string()));
        assert_eq!(config.primals[1].protocol, Some("tarpc".to_string()));
        
        // Edge primal: JSON-RPC
        assert_eq!(config.primals[2].protocol, Some("jsonrpc".to_string()));
    }
    
    #[test]
    fn test_backward_compatibility_no_protocol_field() {
        // Ensure old configs without protocol field still work
        let toml = r#"
[tower]
name = "tower1"
family = "nat0"

[[primals]]
binary = "./primals/beardog"
provides = ["Security"]
http_port = 9000

[primals.env]
BEARDOG_NODE_ID = "tower1"
"#;
        
        let config = TowerConfig::from_toml(toml).unwrap();
        assert_eq!(config.primals.len(), 1);
        assert!(config.primals[0].protocol.is_none());
        assert_eq!(config.primals[0].http_port, 9000);
    }
}

