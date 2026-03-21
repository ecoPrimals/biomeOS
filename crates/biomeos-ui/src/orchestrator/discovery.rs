// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Discovery Module
//!
//! Handles runtime discovery of primals, devices, and saved state.
//!
//! ## DEEP DEBT EVOLUTION (Feb 7, 2026)
//!
//! - **Dynamic discovery**: Scans runtime socket directory for ANY primal
//! - **No hardcoded primal list**: Unknown primals are discovered and accessible
//! - **Capability-based**: Uses `PrimalConnections` dynamic registry
//! - **Graceful degradation**: System works with partial primal availability

use crate::primal_client::PrimalConnections;
use anyhow::Result;
use biomeos_types::CapabilityTaxonomy;
use tracing::{debug, info, warn};

/// Runtime-captured discovery configuration.
/// Production code uses `DiscoveryConfig::from_env()`.
/// Tests construct explicit configs to avoid env var races.
pub struct DiscoveryConfig {
    /// Explicit registry provider name override.
    pub registry_provider: Option<String>,
    /// Explicit storage provider name override.
    pub storage_provider: Option<String>,
}

impl DiscoveryConfig {
    /// Capture configuration from environment variables.
    pub fn from_env() -> Self {
        Self {
            registry_provider: std::env::var("BIOMEOS_REGISTRY_PROVIDER").ok(),
            storage_provider: std::env::var("BIOMEOS_STORAGE_PROVIDER").ok(),
        }
    }
}

/// Resolve a capability to its provider primal name at runtime.
pub fn resolve_capability_provider(
    env_var: &str,
    capability: &CapabilityTaxonomy,
) -> Option<String> {
    resolve_provider(std::env::var(env_var).ok(), capability)
}

fn resolve_provider(env_value: Option<String>, capability: &CapabilityTaxonomy) -> Option<String> {
    env_value.or_else(|| capability.default_primal().map(String::from))
}

/// Discovery result — wraps PrimalConnections for dynamic primal access
///
/// DEEP DEBT EVOLUTION: Replaced fixed-field struct with dynamic registry.
/// Access primals by name: `result.connections.get("beardog")`
/// Or via typed accessors: `result.connections.beardog()`
pub struct DiscoveryResult {
    /// Dynamic primal connections registry
    pub connections: PrimalConnections,
}

/// Primal and device discovery
pub struct Discovery;

impl Discovery {
    /// Discover and connect to all primals
    ///
    /// DEEP DEBT EVOLUTION: Uses dynamic socket scanning instead of hardcoded list.
    /// Discovers ANY primal with a socket in the runtime directory.
    pub async fn discover_primals() -> Result<DiscoveryResult> {
        info!("Discovering primals via dynamic socket scanning...");

        let family_id = std::env::var("FAMILY_ID")
            .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
            .unwrap_or_else(|_| "default".to_string());

        let connections = PrimalConnections::discover_all(&family_id).await;

        let discovered_count = connections.count_available();
        info!(
            "Discovered {} primals: {:?}",
            discovered_count,
            connections.available_primals()
        );

        if discovered_count == 0 {
            warn!("No primals discovered! UI will have limited functionality.");
        }

        Ok(DiscoveryResult { connections })
    }

    /// Discover devices from registry provider
    ///
    /// Uses the registry provider (resolved by name) if available. Falls back gracefully.
    pub async fn discover_devices(connections: &PrimalConnections) -> Result<()> {
        Self::discover_devices_with_config(connections, &DiscoveryConfig::from_env()).await
    }

    /// Discover devices using explicit config (no env var reads).
    pub async fn discover_devices_with_config(
        connections: &PrimalConnections,
        config: &DiscoveryConfig,
    ) -> Result<()> {
        info!("Discovering devices...");

        let registry_name = config.registry_provider.as_ref();

        if let Some(name) = registry_name {
            if let Some(registry) = connections.get(name) {
                match registry
                    .call("registry.list_devices", serde_json::json!({}))
                    .await
                {
                    Ok(devices) => {
                        debug!("Discovered devices: {:?}", devices);
                        info!("Successfully discovered devices from {}", name);
                    }
                    Err(e) => {
                        warn!(
                            "Device discovery failed: {} - {} may not support device registry yet",
                            e, name
                        );
                    }
                }
            } else {
                info!("Registry provider '{}' not available", name);
            }
        } else {
            info!("No registry provider configured (strict discovery mode)");
        }

        Ok(())
    }

    /// Discover active primals via registry
    pub async fn discover_active_primals(connections: &PrimalConnections) -> Result<()> {
        Self::discover_active_primals_with_config(connections, &DiscoveryConfig::from_env()).await
    }

    /// Discover active primals using explicit config.
    pub async fn discover_active_primals_with_config(
        connections: &PrimalConnections,
        config: &DiscoveryConfig,
    ) -> Result<()> {
        info!("Discovering active primals...");

        let registry_name = config.registry_provider.as_ref();

        if let Some(name) = registry_name {
            if let Some(registry) = connections.get(name) {
                match registry
                    .call("registry.list_primals", serde_json::json!({}))
                    .await
                {
                    Ok(primals) => {
                        debug!("Discovered primals: {:?}", primals);
                        info!("Successfully queried {} for active primals", name);
                    }
                    Err(e) => {
                        warn!("Primal discovery failed: {} - check {} connection", e, name);
                    }
                }
            } else {
                info!("Registry provider '{}' not available", name);
            }
        } else {
            info!("No registry provider configured (strict discovery mode)");
        }

        Ok(())
    }

    /// Load saved state from storage provider
    pub async fn load_saved_state(connections: &PrimalConnections, family_id: &str) -> Result<()> {
        Self::load_saved_state_with_config(connections, family_id, &DiscoveryConfig::from_env())
            .await
    }

    /// Load saved state using explicit config.
    pub async fn load_saved_state_with_config(
        connections: &PrimalConnections,
        family_id: &str,
        config: &DiscoveryConfig,
    ) -> Result<()> {
        info!("Loading saved UI state...");

        let storage_name = if let Some(name) = &config.storage_provider {
            name.clone()
        } else {
            info!("No DataStorage provider available; skipping state load");
            return Ok(());
        };

        if let Some(storage) = connections.get(&storage_name) {
            match storage
                .call(
                    "storage.retrieve",
                    serde_json::json!({
                        "key": format!("ui_state:{}", family_id)
                    }),
                )
                .await
            {
                Ok(state) => {
                    debug!("Loaded saved state: {:?}", state);
                    info!("Successfully loaded saved UI state from {}", storage_name);
                }
                Err(e) => {
                    debug!("No saved state found or error: {}", e);
                    info!("Starting with fresh state (no previous state found)");
                }
            }
        } else {
            info!("No storage provider available, starting with fresh state");
        }

        Ok(())
    }

    /// Build initial UI state from discovered primals
    ///
    /// DEEP DEBT EVOLUTION: Takes `PrimalConnections` instead of individual
    /// primal references. State includes ALL discovered primals dynamically.
    pub async fn build_initial_ui_state(
        family_id: &str,
        connections: &PrimalConnections,
    ) -> serde_json::Value {
        Self::build_initial_ui_state_with_config(
            family_id,
            connections,
            &DiscoveryConfig::from_env(),
        )
        .await
    }

    /// Build initial UI state using explicit config.
    pub async fn build_initial_ui_state_with_config(
        family_id: &str,
        connections: &PrimalConnections,
        config: &DiscoveryConfig,
    ) -> serde_json::Value {
        let mut primals_map = serde_json::Map::new();
        for name in connections.available_primals() {
            primals_map.insert(name.to_string(), serde_json::Value::Bool(true));
        }

        let mut state = serde_json::json!({
            "family_id": family_id,
            "primals": primals_map,
            "primal_count": connections.count_available(),
            "devices": [],
            "assignments": []
        });

        if let Some(registry) = config
            .registry_provider
            .as_deref()
            .and_then(|n| connections.get(n))
            && let Ok(devices) = registry
                .call("registry.list_devices", serde_json::json!({}))
                .await
        {
            state["devices"] = devices;
        }

        if let Some(storage) = config
            .storage_provider
            .as_deref()
            .and_then(|n| connections.get(n))
            && let Ok(assignments) = storage
                .call(
                    "storage.list",
                    serde_json::json!({ "key_prefix": "assignment:" }),
                )
                .await
        {
            state["assignments"] = assignments;
        }

        state
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::{resolve_provider, *};
    use crate::primal_client::{PrimalClient, PrimalConnections};
    use biomeos_test_utils::ready_signal;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

    #[test]
    fn test_resolve_capability_provider_env_var_takes_priority() {
        let result = resolve_provider(
            Some("custom-registry".to_string()),
            &CapabilityTaxonomy::Discovery,
        );
        assert_eq!(result, Some("custom-registry".to_string()));
    }

    #[test]
    fn test_resolve_capability_provider_taxonomy_fallback() {
        let result = resolve_provider(None, &CapabilityTaxonomy::Discovery);
        assert_eq!(result, Some("songbird".to_string()));
    }

    #[tokio::test]
    async fn test_discover_primals() {
        let result = Discovery::discover_primals().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_discover_primals_result_has_connections() {
        let result = Discovery::discover_primals()
            .await
            .expect("discover_primals should succeed");
        let _ = result.connections.count_available();
    }

    #[tokio::test]
    async fn test_discover_devices_no_provider() {
        let config = DiscoveryConfig {
            registry_provider: None,
            storage_provider: None,
        };
        let connections = PrimalConnections::default();
        let result = Discovery::discover_devices_with_config(&connections, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_discover_devices_registry_not_available() {
        let config = DiscoveryConfig {
            registry_provider: Some("nonexistent-registry".to_string()),
            storage_provider: None,
        };
        let connections = PrimalConnections::default();
        let result = Discovery::discover_devices_with_config(&connections, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_discover_devices_registry_call_fails() {
        let config = DiscoveryConfig {
            registry_provider: Some("test-registry".to_string()),
            storage_provider: None,
        };
        let mut connections = PrimalConnections::default();
        connections.add_client(
            "test-registry",
            PrimalClient::with_socket("test-registry", "/nonexistent/socket.sock"),
        );
        let result = Discovery::discover_devices_with_config(&connections, &config).await;
        assert!(
            result.is_ok(),
            "graceful degradation when registry.call fails"
        );
    }

    #[tokio::test]
    async fn test_discover_active_primals_no_provider() {
        let config = DiscoveryConfig {
            registry_provider: None,
            storage_provider: None,
        };
        let connections = PrimalConnections::default();
        let result = Discovery::discover_active_primals_with_config(&connections, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_discover_active_primals_registry_call_fails() {
        let config = DiscoveryConfig {
            registry_provider: Some("test-registry".to_string()),
            storage_provider: None,
        };
        let mut connections = PrimalConnections::default();
        connections.add_client(
            "test-registry",
            PrimalClient::with_socket("test-registry", "/nonexistent/socket.sock"),
        );
        let result = Discovery::discover_active_primals_with_config(&connections, &config).await;
        assert!(
            result.is_ok(),
            "graceful degradation when registry.list_primals fails"
        );
    }

    #[tokio::test]
    async fn test_load_saved_state_no_storage_in_connections() {
        let config = DiscoveryConfig {
            registry_provider: None,
            storage_provider: Some("nonexistent-storage".to_string()),
        };
        let connections = PrimalConnections::default();
        let result =
            Discovery::load_saved_state_with_config(&connections, "test-family", &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_load_saved_state_storage_call_fails() {
        let config = DiscoveryConfig {
            registry_provider: None,
            storage_provider: Some("test-storage".to_string()),
        };
        let mut connections = PrimalConnections::default();
        connections.add_client(
            "test-storage",
            PrimalClient::with_socket("test-storage", "/nonexistent/socket.sock"),
        );
        let result =
            Discovery::load_saved_state_with_config(&connections, "test-family", &config).await;
        assert!(
            result.is_ok(),
            "graceful degradation - starts with fresh state"
        );
    }

    #[tokio::test]
    async fn test_load_saved_state_no_provider() {
        let config = DiscoveryConfig {
            registry_provider: None,
            storage_provider: None,
        };
        let connections = PrimalConnections::default();
        let result =
            Discovery::load_saved_state_with_config(&connections, "test-family", &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_build_initial_ui_state_empty() {
        let config = DiscoveryConfig {
            registry_provider: None,
            storage_provider: None,
        };
        let connections = PrimalConnections::default();
        let state =
            Discovery::build_initial_ui_state_with_config("test-family", &connections, &config)
                .await;
        assert_eq!(state["family_id"], "test-family");
        assert_eq!(state["primal_count"], 0);
        assert!(state["primals"].as_object().expect("primals").is_empty());
        assert!(state["devices"].as_array().expect("devices").is_empty());
        assert!(
            state["assignments"]
                .as_array()
                .expect("assignments")
                .is_empty()
        );
    }

    #[tokio::test]
    async fn test_build_initial_ui_state_with_primals() {
        let config = DiscoveryConfig {
            registry_provider: None,
            storage_provider: None,
        };
        let mut connections = PrimalConnections::default();
        connections.add_client(
            "beardog",
            PrimalClient::with_socket("beardog", "/tmp/beardog.sock"),
        );
        connections.add_client(
            "songbird",
            PrimalClient::with_socket("songbird", "/tmp/songbird.sock"),
        );
        let state =
            Discovery::build_initial_ui_state_with_config("test-family", &connections, &config)
                .await;
        assert_eq!(state["family_id"], "test-family");
        assert_eq!(state["primal_count"], 2);
        let primals = state["primals"].as_object().expect("primals");
        assert!(primals.contains_key("beardog"));
        assert!(primals.contains_key("songbird"));
    }

    #[tokio::test]
    async fn test_build_initial_ui_state_registry_and_storage_unavailable() {
        let config = DiscoveryConfig {
            registry_provider: Some("test-reg".to_string()),
            storage_provider: Some("test-stor".to_string()),
        };
        let mut connections = PrimalConnections::default();
        connections.add_client(
            "test-reg",
            PrimalClient::with_socket("test-reg", "/nonexistent/reg.sock"),
        );
        connections.add_client(
            "test-stor",
            PrimalClient::with_socket("test-stor", "/nonexistent/stor.sock"),
        );
        let state =
            Discovery::build_initial_ui_state_with_config("test-family", &connections, &config)
                .await;
        assert_eq!(state["family_id"], "test-family");
        assert!(state["devices"].as_array().expect("devices").is_empty());
        assert!(
            state["assignments"]
                .as_array()
                .expect("assignments")
                .is_empty()
        );
    }

    #[test]
    fn test_resolve_capability_provider_unset_returns_taxonomy() {
        let result = resolve_provider(None, &CapabilityTaxonomy::Discovery);
        assert_eq!(result, Some("songbird".to_string()));
    }

    #[test]
    fn test_discovery_result_has_connections() {
        let result = DiscoveryResult {
            connections: PrimalConnections::default(),
        };
        assert_eq!(result.connections.count_available(), 0);
    }

    #[tokio::test]
    async fn test_discover_active_primals_registry_not_available() {
        let config = DiscoveryConfig {
            registry_provider: Some("nonexistent-registry".to_string()),
            storage_provider: None,
        };
        let connections = PrimalConnections::default();
        let result = Discovery::discover_active_primals_with_config(&connections, &config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_load_saved_state_no_provider_strict() {
        let config = DiscoveryConfig {
            registry_provider: None,
            storage_provider: None,
        };
        let connections = PrimalConnections::default();
        let result =
            Discovery::load_saved_state_with_config(&connections, "test-family", &config).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_capability_provider_data_storage() {
        let result = resolve_provider(None, &CapabilityTaxonomy::DataStorage);
        assert_eq!(result, Some("nestgate".to_string()));
    }

    #[tokio::test]
    async fn test_build_initial_ui_state_registry_success_with_mock_server() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let socket_path = temp_dir.path().join("registry.sock");
        let path_clone = socket_path.clone();
        let (mut ready_tx, ready_rx) = ready_signal();

        let server = tokio::spawn(async move {
            let listener = tokio::net::UnixListener::bind(&path_clone).expect("bind");
            ready_tx.signal();
            if let Ok((stream, _)) = listener.accept().await {
                let (read_half, mut write_half) = stream.into_split();
                let mut reader = tokio::io::BufReader::new(read_half);
                let mut line = String::new();
                let _ = reader.read_line(&mut line).await;
                let req: serde_json::Value =
                    serde_json::from_str(line.trim()).unwrap_or_else(|_| serde_json::json!({}));
                let id = req
                    .get("id")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(1);
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "result": [{"id": "gpu-0", "name": "Test GPU"}],
                    "id": id
                });
                let response_str = serde_json::to_string(&response).unwrap() + "\n";
                let _ = write_half.write_all(response_str.as_bytes()).await;
            }
        });

        ready_rx.wait().await.unwrap();

        let config = DiscoveryConfig {
            registry_provider: Some("test-registry".to_string()),
            storage_provider: None,
        };
        let mut connections = PrimalConnections::default();
        connections.add_client(
            "test-registry",
            PrimalClient::with_socket("test-registry", &socket_path),
        );

        let state =
            Discovery::build_initial_ui_state_with_config("test-family", &connections, &config)
                .await;

        server.abort();

        let devices = state["devices"].as_array().expect("devices array");
        assert!(!devices.is_empty());
    }
}
