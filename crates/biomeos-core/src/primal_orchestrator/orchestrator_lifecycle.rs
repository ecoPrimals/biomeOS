// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Register, start/stop, and capability-provider orchestration.

use std::collections::HashMap;
use std::sync::Arc;

use biomeos_types::error::{BiomeError, BiomeResult};
use biomeos_types::identifiers::PrimalId;
use tokio::time::timeout;
use tracing::{debug, error, info, instrument, warn};

use crate::capabilities::Capability;

use super::orchestrator::PrimalOrchestrator;
use super::orchestrator_health::wait_for_managed_primal_health;
use super::state::{ManagedPrimal, PrimalRecord, PrimalState};

impl PrimalOrchestrator {
    /// Register a primal for orchestration
    #[instrument(skip(self, primal))]
    pub async fn register(&self, primal: Arc<dyn ManagedPrimal>) {
        let id = primal.id().clone();
        info!("Registering primal: {}", id);

        let mut primals = self.primals.write().await;
        primals.insert(
            id,
            PrimalRecord {
                primal,
                state: PrimalState::Pending,
            },
        );
    }

    /// Start all primals in dependency order
    #[instrument(skip(self))]
    pub async fn start_all(&self) -> BiomeResult<()> {
        info!("🚀 Starting all primals in dependency order...");

        let start_order = self.resolve_dependencies().await?;

        info!("📋 Start order: {:?}", start_order);

        for primal_id in start_order {
            self.start_primal(&primal_id).await?;
        }

        info!("✅ All primals started successfully");
        Ok(())
    }

    /// Start a specific primal (with capability-based dependencies)
    #[instrument(skip(self))]
    pub fn start_primal<'a>(
        &'a self,
        id: &'a PrimalId,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = BiomeResult<()>> + Send + 'a>> {
        Box::pin(self.start_primal_inner(id))
    }

    async fn start_primal_inner(&self, id: &PrimalId) -> BiomeResult<()> {
        info!("Starting primal: {}", id);

        let (primal, required_caps) = {
            let primals = self.primals.read().await;
            let record = primals.get(id).ok_or_else(|| {
                BiomeError::discovery_failed(
                    format!("Primal not found: {id}"),
                    Some(id.to_string()),
                )
            })?;

            if record.state == PrimalState::Running {
                info!("Primal {} already running", id);
                return Ok(());
            }

            (record.primal.clone(), record.primal.requires().to_vec())
        };

        for required_cap in &required_caps {
            debug!("Ensuring capability provider for: {}", required_cap);
            self.ensure_capability_provider(required_cap).await?;
        }

        {
            let mut primals = self.primals.write().await;
            if let Some(record) = primals.get_mut(id) {
                record.state = PrimalState::Starting;
            }
        }

        let start_result = self
            .retry_policy
            .execute(|| async {
                primal
                    .start()
                    .await
                    .map_err(|e| anyhow::anyhow!("Start failed: {e}"))
            })
            .await;

        match start_result {
            Ok(()) => {
                info!("✅ Primal {} started", id);

                match timeout(
                    primal.startup_timeout(),
                    wait_for_managed_primal_health(&primal),
                )
                .await
                {
                    Ok(Ok(())) => {
                        if let Some(endpoint) = primal.endpoint().await {
                            self.health_monitor.register(id.clone(), endpoint).await;
                        }

                        let mut primals = self.primals.write().await;
                        if let Some(record) = primals.get_mut(id) {
                            record.state = PrimalState::Running;
                        }

                        info!("✅ Primal {} is healthy and running", id);
                        Ok(())
                    }
                    Ok(Err(e)) => {
                        error!("Primal {} failed health check: {}", id, e);
                        self.mark_failed(id, format!("Health check failed: {e}"))
                            .await;
                        Err(e)
                    }
                    Err(_) => {
                        let msg = format!("Startup timeout after {:?}", primal.startup_timeout());
                        error!("Primal {} {}", id, msg);
                        self.mark_failed(id, msg.clone()).await;
                        Err(BiomeError::timeout_error(msg, 30000, Some("primal_start")))
                    }
                }
            }
            Err(e) => {
                error!("Failed to start primal {}: {}", id, e);
                self.mark_failed(id, e.to_string()).await;
                Err(BiomeError::internal_error(
                    format!("Failed to start {id}: {e}"),
                    Some("primal_start_failure"),
                ))
            }
        }
    }

    /// Ensure at least one provider for a capability is running
    async fn ensure_capability_provider(&self, capability: &Capability) -> BiomeResult<()> {
        let providers: Vec<_> = {
            let primals = self.primals.read().await;
            primals
                .iter()
                .filter(|(_, record)| record.primal.provides().contains(capability))
                .map(|(id, _)| id.clone())
                .collect()
        };

        if providers.is_empty() {
            return Err(BiomeError::discovery_failed(
                format!("No provider found for capability: {capability}"),
                Some(format!("capability:{capability:?}")),
            ));
        }

        for provider_id in providers {
            let state = self.get_state(&provider_id).await;
            if state == Some(PrimalState::Running) {
                debug!(
                    "Capability {} already provided by {}",
                    capability, provider_id
                );
                return Ok(());
            }

            match self.start_primal(&provider_id).await {
                Ok(()) => {
                    info!(
                        "✅ Started capability provider {} for {}",
                        provider_id, capability
                    );
                    return Ok(());
                }
                Err(e) => {
                    warn!(
                        "Failed to start provider {} for {}: {}",
                        provider_id, capability, e
                    );
                }
            }
        }

        Err(BiomeError::internal_error(
            format!("All providers for capability {capability} failed to start"),
            Some("capability_startup_failure"),
        ))
    }

    /// Stop a specific primal
    #[instrument(skip(self))]
    pub async fn stop_primal(&self, id: &PrimalId) -> BiomeResult<()> {
        info!("Stopping primal: {}", id);

        let primal = {
            let primals = self.primals.read().await;
            let record = primals.get(id).ok_or_else(|| {
                BiomeError::discovery_failed(
                    format!("Primal not found: {id}"),
                    Some(id.to_string()),
                )
            })?;

            if record.state == PrimalState::Stopped {
                info!("Primal {} already stopped", id);
                return Ok(());
            }

            record.primal.clone()
        };

        self.health_monitor.unregister(id).await;

        primal.stop().await.map_err(|e| {
            BiomeError::internal_error(
                format!("Failed to stop primal {id}: {e}"),
                Some("primal_stop_failure"),
            )
        })?;

        let mut primals = self.primals.write().await;
        if let Some(record) = primals.get_mut(id) {
            record.state = PrimalState::Stopped;
        }

        info!("✅ Primal {} stopped", id);
        Ok(())
    }

    /// Stop all primals (in reverse dependency order)
    #[instrument(skip(self))]
    pub async fn stop_all(&self) -> BiomeResult<()> {
        info!("🛑 Stopping all primals...");

        let mut stop_order = self.resolve_dependencies().await?;
        stop_order.reverse();

        info!("📋 Stop order: {:?}", stop_order);

        for primal_id in stop_order {
            if let Err(e) = self.stop_primal(&primal_id).await {
                warn!("Error stopping {}: {}", primal_id, e);
            }
        }

        info!("✅ All primals stopped");
        Ok(())
    }

    /// Get the state of a primal
    pub async fn get_state(&self, id: &PrimalId) -> Option<PrimalState> {
        let primals = self.primals.read().await;
        primals.get(id).map(|r| r.state.clone())
    }

    /// Get all primal states
    pub async fn get_all_states(&self) -> HashMap<PrimalId, PrimalState> {
        let primals = self.primals.read().await;
        primals
            .iter()
            .map(|(id, record)| (id.clone(), record.state.clone()))
            .collect()
    }

    async fn mark_failed(&self, id: &PrimalId, reason: String) {
        let mut primals = self.primals.write().await;
        if let Some(record) = primals.get_mut(id) {
            record.state = PrimalState::Failed { reason };
        }
    }
}
