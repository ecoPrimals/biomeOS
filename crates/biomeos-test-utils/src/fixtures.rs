//! Test Fixtures
//!
//! Provides pre-configured test data and configurations for testing.

use serde_json::json;

/// Create a test configuration with sensible defaults
pub fn create_test_config() -> serde_json::Value {
    json!({
        "network": {
            "bind_address": "127.0.0.1",
            "bind_port": 0,
        },
        "primals": {
            "discovery_enabled": true,
            "health_check_interval": 5,
        },
        "security": {
            "tls_enabled": false,
        },
    })
}

/// Create a test biome manifest
pub fn create_test_manifest(name: &str) -> serde_json::Value {
    json!({
        "name": name,
        "version": "1.0.0",
        "description": "Test biome manifest",
        "services": [],
        "resources": {
            "compute": {
                "cpus": 1,
                "memory": "512M",
            },
        },
    })
}

/// Create a test primal registration
pub fn create_test_primal_registration(name: &str, port: u16) -> serde_json::Value {
    json!({
        "name": name,
        "endpoint": format!("http://localhost:{}", port),
        "capabilities": ["test"],
        "version": "1.0.0",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_config() {
        let config = create_test_config();
        assert!(config.get("network").is_some());
        assert!(config.get("primals").is_some());
    }

    #[test]
    fn test_create_manifest() {
        let manifest = create_test_manifest("test-biome");
        assert_eq!(manifest["name"], "test-biome");
        assert_eq!(manifest["version"], "1.0.0");
    }
}

