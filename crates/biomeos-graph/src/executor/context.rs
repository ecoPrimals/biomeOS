// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Execution Context Module
//!
//! Manages shared state during graph execution including:
//! - Environment variables
//! - Node outputs and status
//! - Rollback action tracking
//! - Checkpoint management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Execution status for a node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    Pending,
    Running,
    Completed(serde_json::Value),
    Failed(String),
    Skipped,
}

/// Rollback action recorded during execution
#[derive(Debug, Clone)]
pub enum RollbackAction {
    /// Stop a launched process
    StopProcess {
        primal: String,
        pid: u32,
        socket: String,
    },
    /// Remove a created file
    RemoveFile { path: PathBuf },
    /// Remove a created directory
    RemoveDir { path: PathBuf },
    /// Custom rollback via JSON-RPC
    JsonRpc {
        socket: String,
        method: String,
        params: serde_json::Value,
    },
}

/// Execution context shared across nodes
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Node outputs (for dependency resolution)
    pub outputs: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    /// Execution status of nodes
    pub status: Arc<Mutex<HashMap<String, NodeStatus>>>,
    /// Checkpoint directory
    pub checkpoint_dir: Option<PathBuf>,
    /// Rollback actions (in execution order - will be reversed for rollback)
    pub rollback_actions: Arc<Mutex<Vec<(String, RollbackAction)>>>,
}

impl ExecutionContext {
    /// Create new execution context
    pub fn new(env: HashMap<String, String>) -> Self {
        Self {
            env,
            outputs: Arc::new(Mutex::new(HashMap::new())),
            status: Arc::new(Mutex::new(HashMap::new())),
            checkpoint_dir: None,
            rollback_actions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Record a rollback action for a node
    pub async fn record_rollback(&self, node_id: &str, action: RollbackAction) {
        let mut actions = self.rollback_actions.lock().await;
        actions.push((node_id.to_string(), action));
    }

    /// Get all rollback actions in reverse order
    pub async fn get_rollback_actions(&self) -> Vec<(String, RollbackAction)> {
        let actions = self.rollback_actions.lock().await;
        actions.iter().rev().cloned().collect()
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

    /// Set node status
    pub async fn set_status(&self, node_id: &str, status: NodeStatus) {
        let mut statuses = self.status.lock().await;
        statuses.insert(node_id.to_string(), status);
    }

    /// Get node status
    pub async fn get_status(&self, node_id: &str) -> Option<NodeStatus> {
        let statuses = self.status.lock().await;
        statuses.get(node_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execution_context_new() {
        let env = HashMap::new();
        let context = ExecutionContext::new(env);
        assert!(context.checkpoint_dir.is_none());
    }

    #[tokio::test]
    async fn test_context_output_storage() {
        let context = ExecutionContext::new(HashMap::new());
        let value = serde_json::json!({"result": "success"});
        
        context.set_output("node1", value.clone()).await;
        let retrieved = context.get_output("node1").await;
        
        assert_eq!(retrieved, Some(value));
    }

    #[tokio::test]
    async fn test_context_status_tracking() {
        let context = ExecutionContext::new(HashMap::new());
        
        context
            .set_status("node1", NodeStatus::Running)
            .await;
        
        let status = context.get_status("node1").await;
        assert_eq!(status, Some(NodeStatus::Running));
    }

    #[tokio::test]
    async fn test_rollback_action_recording() {
        let context = ExecutionContext::new(HashMap::new());
        
        let action = RollbackAction::RemoveFile {
            path: PathBuf::from("/tmp/test.txt"),
        };
        
        context.record_rollback("node1", action).await;
        
        let actions = context.get_rollback_actions().await;
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].0, "node1");
    }
}
