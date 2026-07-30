// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Primal registration handler (`lifecycle.register`).

use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::path::PathBuf;
use tracing::info;

use super::LifecycleHandler;
use crate::neural_graph::GraphNode;

impl LifecycleHandler {
    /// Handle `lifecycle.register` - Register a primal for management
    ///
    /// When `binary_path` is provided (e.g. from auto-discovery), uses
    /// `register_primal_binary` so resurrection can respawn the process.
    pub async fn register(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let name = params["name"]
            .as_str()
            .context("Missing 'name' parameter")?;
        let socket_path = params["socket_path"]
            .as_str()
            .context("Missing 'socket_path' parameter")?;
        let pid = params["pid"].as_u64().map(|p| p as u32);
        let binary_path = params
            .get("binary_path")
            .and_then(|v| v.as_str())
            .map(PathBuf::from);

        let manager = self.manager.read().await;

        if let Some(ref binary) = binary_path {
            let node_id = params
                .get("node_id")
                .and_then(|v| v.as_str())
                .unwrap_or(name);
            manager
                .register_primal_binary(
                    name,
                    PathBuf::from(socket_path),
                    pid,
                    binary.clone(),
                    node_id,
                )
                .await?;
        } else {
            let deployment_node: Option<GraphNode> = params
                .get("deployment_node")
                .and_then(|v| serde_json::from_value(v.clone()).ok());
            manager
                .register_primal(name, PathBuf::from(socket_path), pid, deployment_node)
                .await?;
        }

        drop(manager);

        let version = self.bump_topology();
        info!(
            "🌱 Registered primal via API: {} (topology v{version})",
            name
        );

        Ok(json!({
            "registered": name,
            "socket_path": socket_path,
            "pid": pid,
            "binary_path": binary_path.as_ref().map(|p| p.display().to_string()),
            "state": "incubating",
            "topology_version": version,
        }))
    }
}
