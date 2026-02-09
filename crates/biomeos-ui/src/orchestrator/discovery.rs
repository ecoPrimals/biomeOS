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
use tracing::{debug, info, warn};

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
        info!("Discovered {} primals: {:?}", discovered_count, connections.available_primals());

        if discovered_count == 0 {
            warn!("No primals discovered! UI will have limited functionality.");
        }

        Ok(DiscoveryResult { connections })
    }

    /// Discover devices from registry provider
    ///
    /// Uses the registry provider (resolved by name) if available. Falls back gracefully.
    pub async fn discover_devices(connections: &PrimalConnections) -> Result<()> {
        info!("Discovering devices...");

        let registry_name = std::env::var("BIOMEOS_REGISTRY_PROVIDER")
            .unwrap_or_else(|_| "songbird".to_string());

        if let Some(registry) = connections.get(&registry_name) {
            match registry
                .call("registry.list_devices", serde_json::json!({}))
                .await
            {
                Ok(devices) => {
                    debug!("Discovered devices: {:?}", devices);
                    info!("Successfully discovered devices from {}", registry_name);
                }
                Err(e) => {
                    warn!("Device discovery failed: {} - {} may not support device registry yet", e, registry_name);
                }
            }
        } else {
            info!("No registry provider available for device discovery");
        }

        Ok(())
    }

    /// Discover active primals via registry
    pub async fn discover_active_primals(connections: &PrimalConnections) -> Result<()> {
        info!("Discovering active primals...");

        let registry_name = std::env::var("BIOMEOS_REGISTRY_PROVIDER")
            .unwrap_or_else(|_| "songbird".to_string());

        if let Some(registry) = connections.get(&registry_name) {
            match registry
                .call("registry.list_primals", serde_json::json!({}))
                .await
            {
                Ok(primals) => {
                    debug!("Discovered primals: {:?}", primals);
                    info!("Successfully queried {} for active primals", registry_name);
                }
                Err(e) => {
                    warn!("Primal discovery failed: {} - check {} connection", e, registry_name);
                }
            }
        } else {
            info!("No registry provider available, cannot discover other primals");
        }

        Ok(())
    }

    /// Load saved state from storage provider
    pub async fn load_saved_state(
        connections: &PrimalConnections,
        family_id: &str,
    ) -> Result<()> {
        info!("Loading saved UI state...");

        let storage_name = std::env::var("BIOMEOS_STORAGE_PROVIDER")
            .unwrap_or_else(|_| "nestgate".to_string());

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
        // Build dynamic primals map
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

        // Fetch devices from registry provider if available
        let registry_name = std::env::var("BIOMEOS_REGISTRY_PROVIDER")
            .unwrap_or_else(|_| "songbird".to_string());
        if let Some(registry) = connections.get(&registry_name) {
            if let Ok(devices) = registry
                .call("registry.list_devices", serde_json::json!({}))
                .await
            {
                state["devices"] = devices;
            }
        }

        // Fetch assignments from storage provider if available
        let storage_name = std::env::var("BIOMEOS_STORAGE_PROVIDER")
            .unwrap_or_else(|_| "nestgate".to_string());
        if let Some(storage) = connections.get(&storage_name) {
            if let Ok(assignments) = storage
                .call(
                    "storage.list",
                    serde_json::json!({ "key_prefix": "assignment:" }),
                )
                .await
            {
                state["assignments"] = assignments;
            }
        }

        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discover_primals() {
        let result = Discovery::discover_primals().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_discover_devices_no_provider() {
        let connections = PrimalConnections::default();
        let result = Discovery::discover_devices(&connections).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_load_saved_state_no_provider() {
        let connections = PrimalConnections::default();
        let result = Discovery::load_saved_state(&connections, "test-family").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_build_initial_ui_state_empty() {
        let connections = PrimalConnections::default();
        let state = Discovery::build_initial_ui_state("test-family", &connections).await;
        assert_eq!(state["family_id"], "test-family");
        assert_eq!(state["primal_count"], 0);
    }
}
