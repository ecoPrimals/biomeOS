// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Execution context and shared state
//!
//! This module contains the execution context that is shared across all nodes
//! during graph execution, including environment variables, outputs, and status tracking.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Execution status for a node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    /// Node is waiting to be executed
    Pending,
    /// Node is currently executing
    Running,
    /// Node completed successfully with output
    Completed(serde_json::Value),
    /// Node failed with error message
    Failed(String),
    /// Node was skipped (dependency failed)
    Skipped,
}

/// Execution context shared across nodes
///
/// This context maintains the shared state during graph execution, including:
/// - Environment variables
/// - Node outputs (for dependency resolution)
/// - Node execution status
/// - Socket path assignment (via nucleation)
#[derive(Clone)]
pub struct ExecutionContext {
    /// Environment variables available to all nodes
    pub env: HashMap<String, String>,
    /// Outputs from completed nodes (for dependency resolution)
    pub outputs: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    /// Execution status of all nodes
    pub status: Arc<Mutex<HashMap<String, NodeStatus>>>,
    /// Checkpoint directory for state persistence
    pub checkpoint_dir: Option<PathBuf>,
    /// Socket nucleation for deterministic socket path assignment
    pub nucleation: Option<Arc<tokio::sync::RwLock<crate::nucleation::SocketNucleation>>>,
    /// Family ID for socket path namespacing (`Arc<str>` for zero-copy clone across tasks)
    pub family_id: Arc<str>,
}

impl std::fmt::Debug for ExecutionContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionContext")
            .field("env", &self.env)
            .field("checkpoint_dir", &self.checkpoint_dir)
            .field("family_id", &self.family_id)
            .field("nucleation", &self.nucleation.is_some())
            .finish()
    }
}

impl ExecutionContext {
    /// Create new execution context
    ///
    /// # Arguments
    ///
    /// * `env` - Environment variables to make available to all nodes
    ///
    /// # Example
    ///
    /// ```rust
    /// use std::collections::HashMap;
    /// # use biomeos_atomic_deploy::executor::context::ExecutionContext;
    ///
    /// let mut env = HashMap::new();
    /// env.insert("FAMILY_ID".to_string(), "test_family".to_string());
    /// let ctx = ExecutionContext::new(env);
    /// ```
    pub fn new(env: HashMap<String, String>) -> Self {
        let family_id = env
            .get("FAMILY_ID")
            .or_else(|| env.get("BIOMEOS_FAMILY_ID"))
            .map(|s| Arc::from(s.as_str()))
            .unwrap_or_else(|| Arc::from(biomeos_core::family_discovery::get_family_id().as_str()));

        Self {
            env,
            outputs: Arc::new(Mutex::new(HashMap::new())),
            status: Arc::new(Mutex::new(HashMap::new())),
            checkpoint_dir: None,
            nucleation: None,
            family_id,
        }
    }

    /// Set socket nucleation for deterministic socket path assignment
    pub fn with_nucleation(
        mut self,
        nucleation: Arc<tokio::sync::RwLock<crate::nucleation::SocketNucleation>>,
    ) -> Self {
        self.nucleation = Some(nucleation);
        self
    }

    /// Get or assign socket path for a primal
    ///
    /// Uses nucleation if available for deterministic assignment,
    /// otherwise falls back to family-based path.
    pub async fn get_socket_path(&self, primal_name: &str) -> String {
        if let Some(ref nucleation) = self.nucleation {
            // Use shared nucleation for coordinated assignment
            let mut nuc = nucleation.write().await;
            let path = nuc.assign_socket(primal_name, self.family_id.as_ref());
            path.display().to_string()
        } else {
            // Fallback: create local nucleation for deterministic path
            let mut nuc = crate::nucleation::SocketNucleation::default();
            nuc.assign_socket(primal_name, self.family_id.as_ref())
                .display()
                .to_string()
        }
    }

    /// Get environment variables
    pub fn env(&self) -> &HashMap<String, String> {
        &self.env
    }

    /// Set output for a node
    pub async fn set_output(&self, node_id: &str, value: serde_json::Value) {
        let mut outputs = self.outputs.lock().await;
        outputs.insert(node_id.to_string(), value);
    }

    /// Get output from a node
    pub async fn get_output(&self, node_id: &str) -> Option<serde_json::Value> {
        let outputs = self.outputs.lock().await;
        outputs.get(node_id).cloned()
    }

    /// Set status for a node
    pub async fn set_status(&self, node_id: &str, status: NodeStatus) {
        let mut statuses = self.status.lock().await;
        statuses.insert(node_id.to_string(), status);
    }

    /// Get status of a node
    pub async fn get_status(&self, node_id: &str) -> Option<NodeStatus> {
        let statuses = self.status.lock().await;
        statuses.get(node_id).cloned()
    }

    /// Get all node statuses
    pub async fn all_statuses(&self) -> HashMap<String, NodeStatus> {
        let statuses = self.status.lock().await;
        statuses.clone()
    }

    /// Set checkpoint directory
    pub fn with_checkpoint_dir(mut self, dir: PathBuf) -> Self {
        self.checkpoint_dir = Some(dir);
        self
    }

    /// Save checkpoint to disk
    pub async fn save_checkpoint(&self) -> Result<()> {
        if let Some(ref checkpoint_dir) = self.checkpoint_dir {
            std::fs::create_dir_all(checkpoint_dir)?;

            let statuses = self.status.lock().await;
            let checkpoint_path = checkpoint_dir.join("execution_state.json");

            let checkpoint_data = serde_json::json!({
                "statuses": *statuses,
                "family_id": self.family_id.as_ref(),
            });

            std::fs::write(
                checkpoint_path,
                serde_json::to_string_pretty(&checkpoint_data)?,
            )?;
        }
        Ok(())
    }

    /// Load checkpoint from disk
    pub async fn load_checkpoint(&self) -> Result<()> {
        if let Some(ref checkpoint_dir) = self.checkpoint_dir {
            let checkpoint_path = checkpoint_dir.join("execution_state.json");

            if checkpoint_path.exists() {
                let data = std::fs::read_to_string(checkpoint_path)?;
                let checkpoint: serde_json::Value = serde_json::from_str(&data)?;

                if let Some(statuses) = checkpoint.get("statuses") {
                    let loaded_statuses: HashMap<String, NodeStatus> =
                        serde_json::from_value(statuses.clone())?;

                    let mut status_map = self.status.lock().await;
                    *status_map = loaded_statuses;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execution_context_new() {
        let mut env = HashMap::new();
        env.insert("FAMILY_ID".to_string(), "test".to_string());

        let ctx = ExecutionContext::new(env);
        assert_eq!(ctx.family_id.as_ref(), "test");
    }

    #[tokio::test]
    async fn test_execution_context_outputs() {
        let ctx = ExecutionContext::new(HashMap::new());

        ctx.set_output("node1", serde_json::json!({"result": "success"}))
            .await;

        let output = ctx.get_output("node1").await;
        assert!(output.is_some());
        assert_eq!(output.unwrap()["result"], "success");
    }

    #[tokio::test]
    async fn test_execution_context_status() {
        let ctx = ExecutionContext::new(HashMap::new());

        ctx.set_status("node1", NodeStatus::Running).await;

        let status = ctx.get_status("node1").await;
        assert_eq!(status, Some(NodeStatus::Running));
    }

    // --- New tests for comprehensive coverage ---

    #[tokio::test]
    async fn test_get_output_missing_key() {
        let ctx = ExecutionContext::new(HashMap::new());
        let output = ctx.get_output("nonexistent").await;
        assert!(output.is_none());
    }

    #[tokio::test]
    async fn test_get_status_missing_key() {
        let ctx = ExecutionContext::new(HashMap::new());
        let status = ctx.get_status("nonexistent").await;
        assert!(status.is_none());
    }

    #[tokio::test]
    async fn test_all_statuses() {
        let ctx = ExecutionContext::new(HashMap::new());
        ctx.set_status(
            "node1",
            NodeStatus::Completed(serde_json::json!({"ok": true})),
        )
        .await;
        ctx.set_status("node2", NodeStatus::Running).await;
        ctx.set_status("node3", NodeStatus::Failed("oops".to_string()))
            .await;
        ctx.set_status("node4", NodeStatus::Pending).await;
        ctx.set_status("node5", NodeStatus::Skipped).await;

        let all = ctx.all_statuses().await;
        assert_eq!(all.len(), 5);
        assert_eq!(
            all.get("node1"),
            Some(&NodeStatus::Completed(serde_json::json!({"ok": true})))
        );
        assert_eq!(all.get("node2"), Some(&NodeStatus::Running));
        assert_eq!(
            all.get("node3"),
            Some(&NodeStatus::Failed("oops".to_string()))
        );
        assert_eq!(all.get("node4"), Some(&NodeStatus::Pending));
        assert_eq!(all.get("node5"), Some(&NodeStatus::Skipped));
    }

    #[tokio::test]
    async fn test_status_overwrite() {
        let ctx = ExecutionContext::new(HashMap::new());
        ctx.set_status("node1", NodeStatus::Pending).await;
        assert_eq!(ctx.get_status("node1").await, Some(NodeStatus::Pending));

        ctx.set_status("node1", NodeStatus::Running).await;
        assert_eq!(ctx.get_status("node1").await, Some(NodeStatus::Running));

        ctx.set_status("node1", NodeStatus::Completed(serde_json::json!("done")))
            .await;
        assert_eq!(
            ctx.get_status("node1").await,
            Some(NodeStatus::Completed(serde_json::json!("done")))
        );
    }

    #[tokio::test]
    async fn test_output_overwrite() {
        let ctx = ExecutionContext::new(HashMap::new());
        ctx.set_output("node1", serde_json::json!(1)).await;
        assert_eq!(ctx.get_output("node1").await, Some(serde_json::json!(1)));

        ctx.set_output("node1", serde_json::json!(2)).await;
        assert_eq!(ctx.get_output("node1").await, Some(serde_json::json!(2)));
    }

    #[tokio::test]
    async fn test_checkpoint_save_and_load() {
        let temp_dir = tempfile::tempdir().unwrap();

        let ctx = ExecutionContext::new(HashMap::new())
            .with_checkpoint_dir(temp_dir.path().to_path_buf());

        ctx.set_status(
            "beardog",
            NodeStatus::Completed(serde_json::json!({"pid": 123})),
        )
        .await;
        ctx.set_status("songbird", NodeStatus::Running).await;

        // Save checkpoint
        ctx.save_checkpoint().await.unwrap();

        // Verify checkpoint file exists
        let checkpoint_path = temp_dir.path().join("execution_state.json");
        assert!(checkpoint_path.exists());

        // Create a fresh context and load
        let ctx2 = ExecutionContext::new(HashMap::new())
            .with_checkpoint_dir(temp_dir.path().to_path_buf());

        ctx2.load_checkpoint().await.unwrap();

        let loaded = ctx2.all_statuses().await;
        assert_eq!(loaded.len(), 2);
        assert_eq!(
            loaded.get("beardog"),
            Some(&NodeStatus::Completed(serde_json::json!({"pid": 123})))
        );
        assert_eq!(loaded.get("songbird"), Some(&NodeStatus::Running));
    }

    #[tokio::test]
    async fn test_checkpoint_load_missing_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let ctx = ExecutionContext::new(HashMap::new())
            .with_checkpoint_dir(temp_dir.path().to_path_buf());

        // Load checkpoint from empty dir — should succeed without error
        ctx.load_checkpoint().await.unwrap();
        let statuses = ctx.all_statuses().await;
        assert!(statuses.is_empty());
    }

    #[tokio::test]
    async fn test_save_checkpoint_no_dir_configured() {
        let ctx = ExecutionContext::new(HashMap::new());
        // No checkpoint dir set — save should be a no-op success
        ctx.save_checkpoint().await.unwrap();
    }

    #[test]
    fn test_context_debug_format() {
        let mut env = HashMap::new();
        env.insert("FAMILY_ID".to_string(), "test_fam".to_string());
        let ctx = ExecutionContext::new(env);
        let debug_str = format!("{ctx:?}");
        assert!(debug_str.contains("ExecutionContext"));
        assert!(debug_str.contains("FAMILY_ID"));
        assert!(debug_str.contains("test_fam"));
    }

    #[test]
    fn test_family_id_from_biomeos_family_id_env() {
        let mut env = HashMap::new();
        env.insert("BIOMEOS_FAMILY_ID".to_string(), "biomeos_fam".to_string());
        let ctx = ExecutionContext::new(env);
        assert_eq!(ctx.family_id.as_ref(), "biomeos_fam");
    }

    #[test]
    fn test_family_id_prefers_family_id_over_biomeos() {
        let mut env = HashMap::new();
        env.insert("FAMILY_ID".to_string(), "primary".to_string());
        env.insert("BIOMEOS_FAMILY_ID".to_string(), "secondary".to_string());
        let ctx = ExecutionContext::new(env);
        assert_eq!(ctx.family_id.as_ref(), "primary");
    }

    #[test]
    fn test_env_accessor() {
        let mut env = HashMap::new();
        env.insert("KEY".to_string(), "VALUE".to_string());
        let ctx = ExecutionContext::new(env);
        assert_eq!(ctx.env().get("KEY"), Some(&"VALUE".to_string()));
    }

    #[test]
    fn test_node_status_serialization_roundtrip() {
        let statuses = vec![
            NodeStatus::Pending,
            NodeStatus::Running,
            NodeStatus::Completed(serde_json::json!({"result": "ok"})),
            NodeStatus::Failed("timeout".to_string()),
            NodeStatus::Skipped,
        ];
        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let parsed: NodeStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, status);
        }
    }

    #[tokio::test]
    async fn test_context_clone_shares_state() {
        let ctx = ExecutionContext::new(HashMap::new());
        let ctx2 = ctx.clone();

        ctx.set_output("node1", serde_json::json!("hello")).await;
        // Cloned context should see the same output (shared Arc)
        let output = ctx2.get_output("node1").await;
        assert_eq!(output, Some(serde_json::json!("hello")));
    }
}
