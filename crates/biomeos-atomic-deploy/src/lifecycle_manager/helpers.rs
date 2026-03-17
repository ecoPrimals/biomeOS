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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_get_status_empty() {
        let manager = LifecycleManager::new("test-family");
        let status = manager.get_status().await;
        assert!(status.is_empty());
    }

    #[tokio::test]
    async fn test_get_status_after_register() {
        let manager = LifecycleManager::new("test-family");
        manager
            .register_primal("beardog", PathBuf::from("/tmp/bd.sock"), Some(42), None)
            .await
            .unwrap();
        let status = manager.get_status().await;
        assert_eq!(status.len(), 1);
        assert!(status.contains_key("beardog"));
    }

    #[tokio::test]
    async fn test_get_primal_info_found() {
        let manager = LifecycleManager::new("test-family");
        manager
            .register_primal("songbird", PathBuf::from("/tmp/sb.sock"), Some(100), None)
            .await
            .unwrap();
        let info = manager.get_primal_info("songbird").await;
        assert!(info.is_some());
        let info = info.unwrap();
        assert_eq!(info.name, "songbird");
        assert_eq!(info.pid, Some(100));
    }

    #[tokio::test]
    async fn test_get_primal_info_not_found() {
        let manager = LifecycleManager::new("test-family");
        let info = manager.get_primal_info("nonexistent").await;
        assert!(info.is_none());
    }

    #[tokio::test]
    async fn test_clone_for_task_shares_state() {
        let manager = LifecycleManager::new("test-family");
        manager
            .register_primal("test", PathBuf::from("/tmp/t.sock"), None, None)
            .await
            .unwrap();
        let cloned = manager.clone_for_task();
        let status = cloned.get_status().await;
        assert_eq!(status.len(), 1);
        assert!(status.contains_key("test"));
    }

    #[tokio::test]
    async fn test_update_dependency_graph() {
        let manager = LifecycleManager::new("test-family");
        let beardog_node = crate::neural_graph::GraphNode {
            id: "beardog".to_string(),
            ..Default::default()
        };
        let songbird_node = crate::neural_graph::GraphNode {
            id: "songbird".to_string(),
            depends_on: vec!["beardog".to_string()],
            ..Default::default()
        };
        manager
            .register_primal(
                "beardog",
                PathBuf::from("/tmp/beardog.sock"),
                None,
                Some(beardog_node),
            )
            .await
            .unwrap();
        manager
            .register_primal(
                "songbird",
                PathBuf::from("/tmp/songbird.sock"),
                None,
                Some(songbird_node),
            )
            .await
            .unwrap();
        manager.update_dependency_graph().await;
        let beardog_info = manager.get_primal_info("beardog").await.unwrap();
        assert!(beardog_info.depended_by.contains(&"songbird".to_string()));
    }
}
