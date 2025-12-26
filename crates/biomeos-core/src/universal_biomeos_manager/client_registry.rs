//! Client Registry for Primal Clients
//!
//! Manages the lifecycle of primal clients, including discovery,
//! initialization, health checking, and graceful degradation.

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::clients::*;
use crate::discovery_bootstrap::DiscoveryBootstrap;
use crate::primal_client::PrimalClient;
use biomeos_types::constants::capabilities;

/// Registry for all primal clients
///
/// Manages client lifecycle and provides access to discovered primals.
/// Supports graceful degradation when primals are unavailable.
#[derive(Debug, Clone)]
pub struct ClientRegistry {
    /// Songbird client (discovery & coordination)
    songbird: Arc<RwLock<Option<SongbirdClient>>>,

    /// ToadStool client (compute & metrics)
    toadstool: Arc<RwLock<Option<ToadStoolClient>>>,

    /// Squirrel client (AI & intelligence)
    squirrel: Arc<RwLock<Option<SquirrelClient>>>,

    /// NestGate client (storage & persistence)
    nestgate: Arc<RwLock<Option<NestGateClient>>>,

    /// BearDog client (security & cryptography)
    beardog: Arc<RwLock<Option<BearDogClient>>>,
}

impl ClientRegistry {
    /// Create a new empty client registry
    pub fn new() -> Self {
        Self {
            songbird: Arc::new(RwLock::new(None)),
            toadstool: Arc::new(RwLock::new(None)),
            squirrel: Arc::new(RwLock::new(None)),
            nestgate: Arc::new(RwLock::new(None)),
            beardog: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize all clients through discovery
    ///
    /// This performs zero-knowledge startup:
    /// 1. Find universal adapter (Songbird)
    /// 2. Use Songbird to discover other primals
    /// 3. Initialize clients for discovered primals
    ///
    /// # Errors
    /// Returns an error if Songbird cannot be found. Other primal failures
    /// are logged but don't fail initialization (graceful degradation).
    pub async fn initialize(&self) -> Result<()> {
        tracing::info!("🔍 Initializing client registry with zero-knowledge discovery");

        // Step 1: Find universal adapter (Songbird)
        let bootstrap = DiscoveryBootstrap::new("universal-adapter");

        match bootstrap.find_universal_adapter().await {
            Ok(endpoint) => {
                tracing::info!("✅ Found universal adapter at: {}", endpoint);

                // Initialize Songbird client
                let songbird_client = SongbirdClient::new(endpoint);

                // Verify Songbird is available
                if songbird_client.is_available().await {
                    *self.songbird.write().await = Some(songbird_client.clone());
                    tracing::info!("✅ Songbird client initialized");

                    // Step 2: Use Songbird to discover other primals
                    self.discover_and_initialize_primals(&songbird_client).await;
                } else {
                    tracing::warn!("⚠️ Songbird endpoint found but not responding");
                }
            }
            Err(e) => {
                tracing::warn!(
                    "⚠️ No universal adapter found: {}. Running in degraded mode.",
                    e
                );
                tracing::info!(
                    "💡 Set DISCOVERY_ENDPOINT environment variable to enable discovery"
                );
            }
        }

        self.log_initialization_status().await;
        Ok(())
    }

    /// Discover and initialize all other primal clients
    async fn discover_and_initialize_primals(&self, songbird: &SongbirdClient) {
        // Discover ToadStool (compute)
        if let Ok(services) = songbird.discover_by_capability(capabilities::COMPUTE).await {
            if let Some(service) = services.first() {
                let client = ToadStoolClient::new(&service.endpoint);
                if client.is_available().await {
                    *self.toadstool.write().await = Some(client);
                    tracing::info!("✅ ToadStool client initialized at {}", service.endpoint);
                }
            }
        } else {
            tracing::debug!("No compute services discovered");
        }

        // Discover Squirrel (AI)
        if let Ok(services) = songbird.discover_by_capability(capabilities::AI).await {
            if let Some(service) = services.first() {
                let client = SquirrelClient::new(&service.endpoint);
                if client.is_available().await {
                    *self.squirrel.write().await = Some(client);
                    tracing::info!("✅ Squirrel client initialized at {}", service.endpoint);
                }
            }
        } else {
            tracing::debug!("No AI services discovered");
        }

        // Discover NestGate (storage)
        if let Ok(services) = songbird.discover_by_capability(capabilities::STORAGE).await {
            if let Some(service) = services.first() {
                let client = NestGateClient::new(&service.endpoint);
                if client.is_available().await {
                    *self.nestgate.write().await = Some(client);
                    tracing::info!("✅ NestGate client initialized at {}", service.endpoint);
                }
            }
        } else {
            tracing::debug!("No storage services discovered");
        }

        // Discover BearDog (security)
        if let Ok(services) = songbird
            .discover_by_capability(capabilities::SECURITY)
            .await
        {
            if let Some(service) = services.first() {
                let client = BearDogClient::new(&service.endpoint);
                if client.is_available().await {
                    *self.beardog.write().await = Some(client);
                    tracing::info!("✅ BearDog client initialized at {}", service.endpoint);
                }
            }
        } else {
            tracing::debug!("No security services discovered");
        }
    }

    /// Log the initialization status of all clients
    async fn log_initialization_status(&self) {
        let songbird_status = if self.songbird.read().await.is_some() {
            "✅"
        } else {
            "❌"
        };
        let toadstool_status = if self.toadstool.read().await.is_some() {
            "✅"
        } else {
            "⚠️"
        };
        let squirrel_status = if self.squirrel.read().await.is_some() {
            "✅"
        } else {
            "⚠️"
        };
        let nestgate_status = if self.nestgate.read().await.is_some() {
            "✅"
        } else {
            "⚠️"
        };
        let beardog_status = if self.beardog.read().await.is_some() {
            "✅"
        } else {
            "⚠️"
        };

        tracing::info!("📊 Client Registry Status:");
        tracing::info!("  {} Songbird  (Discovery)", songbird_status);
        tracing::info!("  {} ToadStool (Compute)", toadstool_status);
        tracing::info!("  {} Squirrel  (AI)", squirrel_status);
        tracing::info!("  {} NestGate  (Storage)", nestgate_status);
        tracing::info!("  {} BearDog   (Security)", beardog_status);
    }

    /// Get Songbird client (discovery & coordination)
    ///
    /// # Errors
    /// Returns an error if Songbird is not available.
    pub async fn songbird(&self) -> Result<SongbirdClient> {
        self.songbird.read().await.clone().ok_or_else(|| {
            anyhow::anyhow!(
                "Songbird not available. Set DISCOVERY_ENDPOINT or ensure Songbird is running."
            )
        })
    }

    /// Get ToadStool client (compute & metrics)
    ///
    /// # Errors
    /// Returns an error if ToadStool is not available.
    pub async fn toadstool(&self) -> Result<ToadStoolClient> {
        self.toadstool.read().await.clone().ok_or_else(|| {
            anyhow::anyhow!(
                "ToadStool not available. Ensure a compute primal is registered with Songbird."
            )
        })
    }

    /// Get Squirrel client (AI & intelligence)
    ///
    /// # Errors
    /// Returns an error if Squirrel is not available.
    pub async fn squirrel(&self) -> Result<SquirrelClient> {
        self.squirrel.read().await.clone().ok_or_else(|| {
            anyhow::anyhow!(
                "Squirrel not available. Ensure an AI primal is registered with Songbird."
            )
        })
    }

    /// Get NestGate client (storage & persistence)
    ///
    /// # Errors
    /// Returns an error if NestGate is not available.
    pub async fn nestgate(&self) -> Result<NestGateClient> {
        self.nestgate.read().await.clone().ok_or_else(|| {
            anyhow::anyhow!(
                "NestGate not available. Ensure a storage primal is registered with Songbird."
            )
        })
    }

    /// Get BearDog client (security & cryptography)
    ///
    /// # Errors
    /// Returns an error if BearDog is not available.
    pub async fn beardog(&self) -> Result<BearDogClient> {
        self.beardog.read().await.clone().ok_or_else(|| {
            anyhow::anyhow!(
                "BearDog not available. Ensure a security primal is registered with Songbird."
            )
        })
    }

    /// Check if Songbird is available
    pub async fn has_songbird(&self) -> bool {
        self.songbird.read().await.is_some()
    }

    /// Check if ToadStool is available
    pub async fn has_toadstool(&self) -> bool {
        self.toadstool.read().await.is_some()
    }

    /// Check if Squirrel is available
    pub async fn has_squirrel(&self) -> bool {
        self.squirrel.read().await.is_some()
    }

    /// Check if NestGate is available
    pub async fn has_nestgate(&self) -> bool {
        self.nestgate.read().await.is_some()
    }

    /// Check if BearDog is available
    pub async fn has_beardog(&self) -> bool {
        self.beardog.read().await.is_some()
    }

    /// Get count of available clients
    pub async fn available_client_count(&self) -> usize {
        let mut count = 0;
        if self.has_songbird().await {
            count += 1;
        }
        if self.has_toadstool().await {
            count += 1;
        }
        if self.has_squirrel().await {
            count += 1;
        }
        if self.has_nestgate().await {
            count += 1;
        }
        if self.has_beardog().await {
            count += 1;
        }
        count
    }
}

impl Default for ClientRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_registry_creation() {
        let registry = ClientRegistry::new();
        assert!(registry.songbird.try_read().is_ok());
    }

    #[tokio::test]
    async fn test_empty_registry() {
        let registry = ClientRegistry::new();
        assert!(!registry.has_songbird().await);
        assert!(!registry.has_toadstool().await);
        assert_eq!(registry.available_client_count().await, 0);
    }

    #[tokio::test]
    async fn test_client_getters_fail_when_empty() {
        let registry = ClientRegistry::new();

        assert!(registry.songbird().await.is_err());
        assert!(registry.toadstool().await.is_err());
        assert!(registry.squirrel().await.is_err());
        assert!(registry.nestgate().await.is_err());
        assert!(registry.beardog().await.is_err());
    }
}
