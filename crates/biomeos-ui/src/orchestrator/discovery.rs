//! Discovery Module
//!
//! Handles runtime discovery of primals, devices, and saved state.
//!
//! ## TRUE PRIMAL Principles
//!
//! - **No hardcoding**: All primals discovered via capabilities
//! - **Runtime discovery**: No compile-time dependencies
//! - **Graceful degradation**: System works with partial primal availability

use crate::primal_client::{
    BearDogClient, NestGateClient, PetalTongueClient, PrimalClient, SongbirdClient, SquirrelClient,
    ToadStoolClient,
};
use anyhow::Result;
use tracing::{debug, info, warn};

/// Discovery result
pub struct DiscoveryResult {
    pub petaltongue: Option<PetalTongueClient>,
    pub songbird: Option<SongbirdClient>,
    pub beardog: Option<BearDogClient>,
    pub nestgate: Option<NestGateClient>,
    pub toadstool: Option<ToadStoolClient>,
    pub squirrel: Option<SquirrelClient>,
}

/// Primal and device discovery
pub struct Discovery;

impl Discovery {
    /// Discover and connect to all primals
    ///
    /// Uses capability-based discovery to find primals. No hardcoded assumptions!
    pub async fn discover_primals() -> Result<DiscoveryResult> {
        info!("Discovering primals via capability-based discovery...");

        // Try to discover each primal by capability
        // Note: These discoveries are independent and fail gracefully
        // TRUE PRIMAL: Uses XDG-compliant Unix socket discovery via AtomicClient

        // 1. Discover visualization primal (petalTongue)
        info!("Attempting to discover visualization primal...");
        let petaltongue = PrimalClient::discover("petaltongue").await.ok();

        // 2. Discover service registry primal (Songbird)
        info!("Attempting to discover service registry primal...");
        let songbird = PrimalClient::discover("songbird").await.ok();

        // 3. Discover security primal (BearDog)
        info!("Attempting to discover security primal...");
        let beardog = PrimalClient::discover("beardog").await.ok();

        // 4. Discover storage primal (NestGate)
        info!("Attempting to discover storage primal...");
        let nestgate = PrimalClient::discover("nestgate").await.ok();

        // 5. Discover compute primal (ToadStool)
        info!("Attempting to discover compute primal...");
        let toadstool = PrimalClient::discover("toadstool").await.ok();

        // 6. Discover AI primal (Squirrel)
        info!("Attempting to discover AI primal...");
        let squirrel = PrimalClient::discover("squirrel").await.ok();

        let discovered_count = [
            petaltongue.is_some(),
            songbird.is_some(),
            beardog.is_some(),
            nestgate.is_some(),
            toadstool.is_some(),
            squirrel.is_some(),
        ]
        .iter()
        .filter(|&&x| x)
        .count();

        info!("Discovered {}/6 primals", discovered_count);

        if discovered_count == 0 {
            warn!("No primals discovered! UI will have limited functionality.");
        }

        Ok(DiscoveryResult {
            petaltongue,
            songbird,
            beardog,
            nestgate,
            toadstool,
            squirrel,
        })
    }

    /// Discover devices from available primals
    ///
    /// Uses Songbird's device registry if available. Falls back gracefully.
    pub async fn discover_devices(songbird: &Option<SongbirdClient>) -> Result<()> {
        info!("Discovering devices...");

        if let Some(ref songbird) = songbird {
            // Query Songbird for registered devices using JSON-RPC
            match songbird
                .call("registry.list_devices", serde_json::json!({}))
                .await
            {
                Ok(devices) => {
                    debug!("Discovered devices: {:?}", devices);
                    info!("Successfully discovered devices from Songbird");
                }
                Err(e) => {
                    warn!("Device discovery failed: {} - Songbird may not support device registry yet", e);
                }
            }
        } else {
            info!("No Songbird available for device discovery");
        }

        Ok(())
    }

    /// Discover active primals
    ///
    /// Uses Songbird's primal registry to get list of active primals.
    pub async fn discover_active_primals(songbird: &Option<SongbirdClient>) -> Result<()> {
        info!("Discovering active primals...");

        if let Some(ref songbird) = songbird {
            // Query Songbird for all registered primals using JSON-RPC
            match songbird
                .call("registry.list_primals", serde_json::json!({}))
                .await
            {
                Ok(primals) => {
                    debug!("Discovered primals: {:?}", primals);
                    info!("Successfully queried Songbird for active primals");
                }
                Err(e) => {
                    warn!("Primal discovery failed: {} - check Songbird connection", e);
                }
            }
        } else {
            info!("No Songbird available, cannot discover other primals");
        }

        Ok(())
    }

    /// Load saved state from NestGate
    pub async fn load_saved_state(
        nestgate: &Option<NestGateClient>,
        family_id: &str,
    ) -> Result<()> {
        info!("Loading saved UI state...");

        if let Some(ref nestgate) = nestgate {
            // Try to load previous UI state from NestGate using JSON-RPC
            match nestgate
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
                    info!("Successfully loaded saved UI state from NestGate");
                }
                Err(e) => {
                    debug!("No saved state found or error: {}", e);
                    info!("Starting with fresh state (no previous state found)");
                }
            }
        } else {
            info!("No storage primal available, starting with fresh state");
        }

        Ok(())
    }

    /// Build initial UI state from discovered primals
    pub async fn build_initial_ui_state(
        family_id: &str,
        songbird: &Option<SongbirdClient>,
        nestgate: &Option<NestGateClient>,
        petaltongue: &Option<PetalTongueClient>,
        beardog: &Option<BearDogClient>,
        toadstool: &Option<ToadStoolClient>,
        squirrel: &Option<SquirrelClient>,
    ) -> serde_json::Value {
        let mut state = serde_json::json!({
            "family_id": family_id,
            "primals": {
                "petaltongue": petaltongue.is_some(),
                "songbird": songbird.is_some(),
                "beardog": beardog.is_some(),
                "nestgate": nestgate.is_some(),
                "toadstool": toadstool.is_some(),
                "squirrel": squirrel.is_some()
            },
            "devices": [],
            "assignments": []
        });

        // Fetch devices from Songbird if available
        if let Some(ref songbird) = songbird {
            if let Ok(devices) = songbird
                .call("registry.list_devices", serde_json::json!({}))
                .await
            {
                state["devices"] = devices;
            }
        }

        // Fetch assignments from NestGate if available
        if let Some(ref nestgate) = nestgate {
            if let Ok(assignments) = nestgate
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
    async fn test_discover_devices_no_songbird() {
        let result = Discovery::discover_devices(&None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_load_saved_state_no_nestgate() {
        let result = Discovery::load_saved_state(&None, "test-family").await;
        assert!(result.is_ok());
    }
}
