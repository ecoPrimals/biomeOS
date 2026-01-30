//! Rollback Management Module
//!
//! Handles rollback of failed deployments by executing recorded actions
//! in reverse order. Supports multiple rollback types:
//! - Process termination (graceful then forceful)
//! - File removal
//! - Directory removal
//! - Custom JSON-RPC rollback calls

use anyhow::Result;
use std::path::PathBuf;
use tracing::{debug, info, warn};

use super::context::{ExecutionContext, RollbackAction};

/// Manages rollback of failed deployments
pub struct RollbackManager<'a> {
    context: &'a ExecutionContext,
}

impl<'a> RollbackManager<'a> {
    /// Create new rollback manager
    pub fn new(context: &'a ExecutionContext) -> Self {
        Self { context }
    }

    /// Execute all recorded rollback actions in reverse order
    pub async fn execute_rollback(&self) -> Result<()> {
        info!("🔄 Starting rollback...");

        let actions = self.context.get_rollback_actions().await;
        let total = actions.len();

        if total == 0 {
            info!("✅ No actions to rollback");
            return Ok(());
        }

        info!("   Rolling back {} actions", total);

        let mut errors = Vec::new();

        for (i, (node_id, action)) in actions.iter().enumerate() {
            debug!("   [{}/{}] Rolling back: {}", i + 1, total, node_id);

            let result = match action {
                RollbackAction::StopProcess {
                    primal,
                    pid,
                    socket,
                } => self.rollback_stop_process(primal, *pid, socket).await,
                RollbackAction::RemoveFile { path } => self.rollback_remove_file(path).await,
                RollbackAction::RemoveDir { path } => self.rollback_remove_dir(path).await,
                RollbackAction::JsonRpc {
                    socket,
                    method,
                    params,
                } => {
                    self.rollback_jsonrpc(socket, method, params.clone())
                        .await
                }
            };

            if let Err(e) = result {
                warn!("   ⚠️ Rollback action failed for {}: {}", node_id, e);
                errors.push((node_id.clone(), e.to_string()));
                // Continue with other rollback actions
            }
        }

        if errors.is_empty() {
            info!("✅ Rollback completed successfully ({} actions)", total);
        } else {
            warn!(
                "⚠️ Rollback completed with {} errors out of {} actions",
                errors.len(),
                total
            );
        }

        Ok(())
    }

    /// Stop a launched process (graceful then forceful)
    async fn rollback_stop_process(&self, primal: &str, pid: u32, socket: &str) -> Result<()> {
        info!("   Stopping {} (PID {})", primal, pid);

        // First try graceful shutdown via socket if available
        if std::path::Path::new(socket).exists() {
            if self.send_shutdown_signal(socket).await.is_ok() {
                // Wait a bit for graceful shutdown
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }

        // Check if process is still running, send SIGTERM
        #[cfg(unix)]
        {
            use std::process::Command;

            // Check if process exists
            let check = Command::new("kill")
                .args(["-0", &pid.to_string()])
                .output();

            if check.is_ok() && check.unwrap().status.success() {
                // Process still running, send SIGTERM
                let _ = Command::new("kill")
                    .args(["-15", &pid.to_string()]) // SIGTERM
                    .output();

                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                // Check again, send SIGKILL if still running
                let check = Command::new("kill")
                    .args(["-0", &pid.to_string()])
                    .output();

                if check.is_ok() && check.unwrap().status.success() {
                    let _ = Command::new("kill")
                        .args(["-9", &pid.to_string()]) // SIGKILL
                        .output();
                }
            }
        }

        // Remove socket file if it exists
        let _ = std::fs::remove_file(socket);

        Ok(())
    }

    /// Send graceful shutdown signal via JSON-RPC
    async fn send_shutdown_signal(&self, socket: &str) -> Result<()> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        let stream = tokio::time::timeout(
            tokio::time::Duration::from_secs(2),
            UnixStream::connect(socket),
        )
        .await??;

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "shutdown",
            "params": { "graceful": true },
            "id": 1
        });
        let request_str = serde_json::to_string(&request)? + "\n";
        writer.write_all(request_str.as_bytes()).await?;
        writer.flush().await?;

        // Wait for acknowledgment
        let mut response = String::new();
        let _ = tokio::time::timeout(
            tokio::time::Duration::from_secs(2),
            reader.read_line(&mut response),
        )
        .await;

        Ok(())
    }

    /// Remove a created file
    async fn rollback_remove_file(&self, path: &PathBuf) -> Result<()> {
        if path.exists() {
            std::fs::remove_file(path)?;
            debug!("   Removed file: {}", path.display());
        }
        Ok(())
    }

    /// Remove a created directory
    async fn rollback_remove_dir(&self, path: &PathBuf) -> Result<()> {
        if path.exists() && path.is_dir() {
            std::fs::remove_dir_all(path)?;
            debug!("   Removed directory: {}", path.display());
        }
        Ok(())
    }

    /// Execute custom JSON-RPC rollback
    async fn rollback_jsonrpc(
        &self,
        socket: &str,
        method: &str,
        params: serde_json::Value,
    ) -> Result<()> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        if !std::path::Path::new(socket).exists() {
            debug!("   Socket {} not available for rollback", socket);
            return Ok(());
        }

        let stream = tokio::time::timeout(
            tokio::time::Duration::from_secs(5),
            UnixStream::connect(socket),
        )
        .await??;

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });
        let request_str = serde_json::to_string(&request)? + "\n";
        writer.write_all(request_str.as_bytes()).await?;
        writer.flush().await?;

        // Read response
        let mut response = String::new();
        let _ = tokio::time::timeout(
            tokio::time::Duration::from_secs(5),
            reader.read_line(&mut response),
        )
        .await?;

        debug!("   Rollback {} completed", method);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_rollback_manager_creation() {
        let context = ExecutionContext::new(HashMap::new());
        let manager = RollbackManager::new(&context);
        
        // Should be able to create manager
        let result = manager.execute_rollback().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rollback_no_actions() {
        let context = ExecutionContext::new(HashMap::new());
        let manager = RollbackManager::new(&context);
        
        let result = manager.execute_rollback().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_rollback_file_removal() {
        use std::fs::File;
        use tempfile::tempdir;

        let context = ExecutionContext::new(HashMap::new());
        let manager = RollbackManager::new(&context);
        
        // Create a temporary file
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        File::create(&file_path).unwrap();
        
        // Record rollback action
        context
            .record_rollback("test", RollbackAction::RemoveFile { path: file_path.clone() })
            .await;
        
        // Execute rollback
        let result = manager.execute_rollback().await;
        assert!(result.is_ok());
        
        // File should be removed
        assert!(!file_path.exists());
    }
}
