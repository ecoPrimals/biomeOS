// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::types::{PrimalInfo, RegisterParams};
use crate::Capability;
use biomeos_types::paths::SystemPaths;
use biomeos_types::{BiomeError, PrimalId};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// biomeOS Capability Registry
///
/// Central registry for primal capabilities. Maintains a mapping of
/// capabilities to providers, enabling O(N) capability resolution.
pub struct CapabilityRegistry {
    /// Family ID
    pub(crate) family_id: String,

    /// Registered primals (`PrimalId` -> `PrimalInfo`)
    pub(crate) primals: Arc<RwLock<HashMap<PrimalId, PrimalInfo>>>,

    /// Capability index (`Capability` -> `Vec<PrimalId>`)
    pub(crate) capability_index: Arc<RwLock<HashMap<Capability, Vec<PrimalId>>>>,

    /// Unix socket path
    pub(crate) socket_path: PathBuf,
}

impl CapabilityRegistry {
    /// Create a new capability registry with XDG-resolved socket path.
    #[must_use]
    pub fn new(family_id: String) -> Self {
        let paths = SystemPaths::new_lazy();
        let socket_path = paths
            .runtime_dir()
            .join(format!("biomeos-registry-{family_id}.sock"));
        Self::with_socket_path(family_id, socket_path)
    }

    /// Create a registry with an explicit socket path (useful for tests).
    pub fn with_socket_path(family_id: String, socket_path: PathBuf) -> Self {
        info!("🔧 Creating biomeOS capability registry");
        info!("   Family: {}", family_id);
        info!("   Socket: {:?}", socket_path);

        Self {
            family_id,
            primals: Arc::new(RwLock::new(HashMap::new())),
            capability_index: Arc::new(RwLock::new(HashMap::new())),
            socket_path,
        }
    }

    /// Register a primal
    pub async fn register(&self, id: PrimalId, params: RegisterParams) -> Result<(), BiomeError> {
        info!("📝 Registering primal: {:?}", id);
        debug!("   Provides: {:?}", params.provides);
        debug!("   Requires: {:?}", params.requires);

        let now = chrono::Utc::now();

        let info = PrimalInfo {
            id: id.clone(),
            provides: params.provides.clone(),
            requires: params.requires.clone(),
            socket_path: params.socket_path,
            http_endpoint: params.http_endpoint,
            metadata: params.metadata.unwrap_or_default(),
            registered_at: now,
            last_heartbeat: now,
        };

        // Add to primals map
        {
            let mut primals = self.primals.write().await;
            primals.insert(id.clone(), info);
        }

        // Update capability index
        {
            let mut index = self.capability_index.write().await;
            for capability in params.provides {
                index.entry(capability).or_default().push(id.clone());
            }
        }

        info!("✅ Primal registered: {:?}", id);

        Ok(())
    }

    /// Get provider for a capability
    pub async fn get_provider(
        &self,
        capability: &Capability,
    ) -> Result<Option<PrimalInfo>, BiomeError> {
        debug!("🔍 Looking for provider of: {:?}", capability);

        let index = self.capability_index.read().await;

        if let Some(providers) = index.get(capability)
            && let Some(primal_id) = providers.first()
        {
            let primals = self.primals.read().await;
            if let Some(info) = primals.get(primal_id) {
                info!("✅ Found provider: {:?} for {:?}", primal_id, capability);
                return Ok(Some(info.clone()));
            }
        }

        warn!("❌ No provider found for: {:?}", capability);
        Ok(None)
    }

    /// List all registered primals
    pub async fn list_primals(&self) -> Vec<PrimalInfo> {
        let primals = self.primals.read().await;
        primals.values().cloned().collect()
    }

    /// Update heartbeat for a primal
    pub async fn heartbeat(&self, primal_id: &PrimalId) -> Result<(), BiomeError> {
        let mut primals = self.primals.write().await;

        if let Some(info) = primals.get_mut(primal_id) {
            info.last_heartbeat = chrono::Utc::now();
            debug!("💓 Heartbeat received from: {:?}", primal_id);
            Ok(())
        } else {
            Err(BiomeError::resource_error(
                format!("Primal not found: {primal_id:?}"),
                "registry",
                None::<String>,
                None::<String>,
            ))
        }
    }

    /// Unregister a primal
    pub async fn unregister(&self, primal_id: &PrimalId) -> Result<(), BiomeError> {
        info!("🗑️  Unregistering primal: {:?}", primal_id);

        // Remove from primals map
        let info = {
            let mut primals = self.primals.write().await;
            primals.remove(primal_id)
        };

        if let Some(info) = info {
            // Remove from capability index
            let mut index = self.capability_index.write().await;
            for capability in &info.provides {
                if let Some(providers) = index.get_mut(capability) {
                    providers.retain(|id| id != primal_id);
                    if providers.is_empty() {
                        index.remove(capability);
                    }
                }
            }

            info!("✅ Primal unregistered: {:?}", primal_id);
            Ok(())
        } else {
            Err(BiomeError::resource_error(
                format!("Primal not found: {primal_id:?}"),
                "registry",
                None::<String>,
                None::<String>,
            ))
        }
    }
}

impl Clone for CapabilityRegistry {
    fn clone(&self) -> Self {
        Self {
            family_id: self.family_id.clone(),
            primals: Arc::clone(&self.primals),
            capability_index: Arc::clone(&self.capability_index),
            socket_path: self.socket_path.clone(),
        }
    }
}
