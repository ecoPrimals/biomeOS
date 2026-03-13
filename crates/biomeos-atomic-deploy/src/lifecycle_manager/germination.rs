// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Germination: primal registration and deployment graph storage

use anyhow::Result;
use std::path::PathBuf;
use tracing::info;

use crate::neural_graph::{Graph, GraphNode};

use super::{
    HealthConfig, LifecycleManager, LifecycleState, ManagedPrimal, PrimalMetrics,
    ResurrectionConfig,
};

impl LifecycleManager {
    /// Register a primal for lifecycle management
    ///
    /// Called after germination to track the primal
    pub async fn register_primal(
        &self,
        name: impl Into<String>,
        socket_path: PathBuf,
        pid: Option<u32>,
        deployment_node: Option<GraphNode>,
    ) -> Result<()> {
        let name = name.into();
        let depends_on = deployment_node
            .as_ref()
            .map(|n| n.depends_on.clone())
            .unwrap_or_default();

        let primal = ManagedPrimal {
            name: name.clone(),
            family_id: self.family_id.clone(),
            socket_path,
            pid,
            state: LifecycleState::Incubating {
                started_at: chrono::Utc::now(),
                timeout_ms: 30000,
            },
            deployment_node,
            depends_on,
            depended_by: Vec::new(),
            health_config: HealthConfig::default(),
            resurrection_config: ResurrectionConfig::default(),
            metrics: PrimalMetrics::default(),
        };

        {
            let mut primals = self.primals.write().await;
            primals.insert(name.clone(), primal);
            info!("🌱 Registered primal: {} (incubating)", name);
        } // Release write lock before calling update_dependency_graph

        // Update dependency graph (requires its own write lock)
        self.update_dependency_graph().await;

        Ok(())
    }

    /// Set the JSON-RPC health check method for a primal
    ///
    /// Some primals use semantic method naming (e.g., "toadstool.health")
    /// instead of plain "health". Call this after `register_primal` to override.
    pub async fn set_health_method(&self, name: &str, method: impl Into<String>) {
        let mut primals = self.primals.write().await;
        if let Some(primal) = primals.get_mut(name) {
            primal.health_config.health_method = method.into();
        }
    }

    /// Store deployment graph for resurrection
    pub async fn store_deployment_graph(&self, graph_id: impl Into<String>, graph: Graph) {
        let graph_id = graph_id.into();
        let mut graphs = self.deployment_graphs.write().await;
        graphs.insert(graph_id.clone(), graph);
        info!(
            "📋 Stored deployment graph: {} (for resurrection)",
            graph_id
        );
    }
}
