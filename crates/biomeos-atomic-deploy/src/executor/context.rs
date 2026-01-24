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
    /// Family ID for socket path namespacing
    pub family_id: String,
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
            .cloned()
            .unwrap_or_else(|| "nat0".to_string());

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
            // Use nucleation for deterministic assignment
            let mut nuc = nucleation.write().await;
            let path = nuc.assign_socket(primal_name, &self.family_id);
            path.display().to_string()
        } else {
            // Fallback: deterministic path based on family_id
            format!("/tmp/{}-{}.sock", primal_name, self.family_id)
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
                "family_id": self.family_id,
            });

            std::fs::write(checkpoint_path, serde_json::to_string_pretty(&checkpoint_data)?)?;
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
        assert_eq!(ctx.family_id, "test");
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
}

