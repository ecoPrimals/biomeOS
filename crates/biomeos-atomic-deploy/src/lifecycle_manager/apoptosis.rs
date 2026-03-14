// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Apoptosis: graceful shutdown with dependency-aware ordering

use anyhow::Result;
use tracing::{info, warn};

use super::{ApoptosisReason, LifecycleManager, LifecycleState};

impl LifecycleManager {
    /// Initiate apoptosis (graceful shutdown) for a primal
    ///
    /// Uses iterative approach instead of recursion to avoid async boxing complexity.
    pub async fn apoptosis(&self, name: &str, _reason: ApoptosisReason) -> Result<()> {
        // Collect all primals to shut down (transitive dependents)
        let shutdown_list = self.collect_shutdown_order(name).await;

        for (primal_name, shutdown_reason) in shutdown_list {
            self.apoptosis_single(&primal_name, shutdown_reason).await?;
        }

        Ok(())
    }

    /// Collect primals to shut down in order (dependents first)
    pub(crate) async fn collect_shutdown_order(
        &self,
        name: &str,
    ) -> Vec<(String, ApoptosisReason)> {
        let mut result = Vec::new();
        let mut to_process = vec![(name.to_string(), ApoptosisReason::UserRequest)];
        let mut processed = std::collections::HashSet::new();

        while let Some((current, reason)) = to_process.pop() {
            if processed.contains(&current) {
                continue;
            }
            processed.insert(current.clone());

            // Get dependents
            let primals = self.primals.read().await;
            if let Some(primal) = primals.get(&current) {
                // Add dependents first (they get shut down before this primal)
                for dependent in &primal.depended_by {
                    if !processed.contains(dependent) {
                        to_process.push((
                            dependent.clone(),
                            ApoptosisReason::DependencyDeath(current.clone()),
                        ));
                    }
                }
            }
            drop(primals);

            // Add this primal to result
            result.push((current, reason));
        }

        // Reverse so dependents are first
        result.reverse();
        result
    }

    /// Shutdown a single primal (no recursion)
    async fn apoptosis_single(&self, name: &str, reason: ApoptosisReason) -> Result<()> {
        info!("💀 Initiating apoptosis for {}: {:?}", name, reason);

        let mut primals = self.primals.write().await;
        let primal = match primals.get_mut(name) {
            Some(p) => p,
            None => {
                warn!("⚠️ Primal not found for apoptosis: {}", name);
                return Ok(());
            }
        };

        primal.state = LifecycleState::Apoptosis {
            reason: reason.clone(),
            started_at: chrono::Utc::now(),
        };

        let _pid = primal.pid;
        drop(primals);

        // Kill the process
        self.kill_primal_process(name).await?;

        // Update state to dead
        let mut primals = self.primals.write().await;
        if let Some(primal) = primals.get_mut(name) {
            primal.state = LifecycleState::Dead {
                since: chrono::Utc::now(),
                reason: format!("{reason:?}"),
            };
            primal.pid = None;
        }

        info!("💀 {} apoptosis complete", name);

        Ok(())
    }

    /// Initiate system-wide shutdown
    pub async fn shutdown_all(&self) -> Result<()> {
        info!("🛑 Initiating system-wide shutdown");

        // Set shutdown flag
        *self.shutdown.write().await = true;

        // Get all primals in dependency order (reverse topological)
        let primals = self.primals.read().await;
        let mut shutdown_order: Vec<String> = primals.keys().cloned().collect();

        // Sort by number of dependents (most dependents first = shutdown last)
        shutdown_order.sort_by_key(|name| {
            primals
                .get(name)
                .map(|p| std::cmp::Reverse(p.depended_by.len()))
                .unwrap_or(std::cmp::Reverse(0))
        });

        drop(primals);

        // Shutdown in order
        for name in shutdown_order {
            self.apoptosis(&name, ApoptosisReason::SystemShutdown)
                .await
                .ok();
        }

        info!("✅ System shutdown complete");

        Ok(())
    }
}
