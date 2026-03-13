// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Lifecycle manager helpers: dependency graph, status, and task cloning

use std::collections::HashMap;

use super::{LifecycleManager, LifecycleState, ManagedPrimal};

impl LifecycleManager {
    /// Update dependency graph (depended_by relationships)
    pub(crate) async fn update_dependency_graph(&self) {
        let mut primals = self.primals.write().await;

        // Clear all depended_by lists
        for primal in primals.values_mut() {
            primal.depended_by.clear();
        }

        // Build dependency graph
        let depends_on_map: HashMap<String, Vec<String>> = primals
            .iter()
            .map(|(name, p)| (name.clone(), p.depends_on.clone()))
            .collect();

        // Set depended_by relationships
        for (name, deps) in &depends_on_map {
            for dep in deps {
                if let Some(dep_primal) = primals.get_mut(dep) {
                    dep_primal.depended_by.push(name.clone());
                }
            }
        }
    }

    /// Get status of all managed primals
    pub async fn get_status(&self) -> HashMap<String, LifecycleState> {
        let primals = self.primals.read().await;
        primals
            .iter()
            .map(|(name, p)| (name.clone(), p.state.clone()))
            .collect()
    }

    /// Get detailed info for a primal
    pub async fn get_primal_info(&self, name: &str) -> Option<ManagedPrimal> {
        let primals = self.primals.read().await;
        primals.get(name).cloned()
    }

    /// Clone for spawning async tasks
    pub(crate) fn clone_for_task(&self) -> Self {
        Self {
            primals: self.primals.clone(),
            family_id: self.family_id.clone(),
            nucleation: self.nucleation.clone(),
            health_checker: self.health_checker.clone(),
            deployment_graphs: self.deployment_graphs.clone(),
            health_check_interval: self.health_check_interval,
            shutdown: self.shutdown.clone(),
        }
    }
}
