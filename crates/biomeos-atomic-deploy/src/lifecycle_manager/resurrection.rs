// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Resurrection: restart degraded/dead primals from deployment graph

use anyhow::Result;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{info, warn};

use crate::executor::context::ExecutionContext;
use crate::executor::primal_spawner;
use crate::neural_graph::GraphNode;

use super::{ApoptosisReason, LifecycleManager, LifecycleState};

impl LifecycleManager {
    /// Attempt to resurrect a degraded/dead primal
    pub(crate) async fn attempt_resurrection(&self, name: &str) -> Result<()> {
        let mut primals = self.primals.write().await;
        let primal = primals
            .get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Primal not found: {}", name))?;

        // Check if resurrection is enabled
        if !primal.resurrection_config.enabled {
            warn!("⚠️ Resurrection disabled for {}", name);
            return Ok(());
        }

        // Get resurrection attempt count
        let attempts = match &primal.state {
            LifecycleState::Degraded {
                resurrection_attempts,
                ..
            } => *resurrection_attempts,
            LifecycleState::Dead { .. } => 0,
            _ => return Ok(()), // Not in a state that needs resurrection
        };

        // Check max attempts
        if attempts >= primal.resurrection_config.max_attempts {
            tracing::error!("💀 {} resurrection exhausted ({} attempts)", name, attempts);
            primal.state = LifecycleState::Apoptosis {
                reason: ApoptosisReason::ResurrectionExhausted,
                started_at: chrono::Utc::now(),
            };
            return Ok(());
        }

        // Calculate backoff delay
        let delay = std::cmp::min(
            primal.resurrection_config.base_delay * 2u32.pow(attempts),
            primal.resurrection_config.max_delay,
        );

        info!(
            "🔄 Resurrecting {} (attempt {}/{}, delay: {:?})",
            name,
            attempts + 1,
            primal.resurrection_config.max_attempts,
            delay
        );

        // Get deployment node for resurrection
        let deployment_node = primal.deployment_node.clone();
        let socket_path = primal.socket_path.clone();

        // Update state
        if let LifecycleState::Degraded {
            resurrection_attempts,
            ..
        } = &mut primal.state
        {
            *resurrection_attempts = attempts + 1;
        }

        primal.metrics.resurrection_count += 1;

        // Drop lock before async operations
        drop(primals);

        // Wait backoff delay
        tokio::time::sleep(delay).await;

        // Kill old process if still running
        self.kill_primal_process(name).await?;

        // Clean up old socket
        if socket_path.exists() {
            tokio::fs::remove_file(&socket_path).await.ok();
        }

        // Respawn from deployment node
        if let Some(node) = deployment_node {
            self.respawn_primal(name, &node).await?;
        } else {
            warn!("⚠️ No deployment node for {} - cannot resurrect", name);
        }

        Ok(())
    }

    /// Kill primal process
    pub(crate) async fn kill_primal_process(&self, name: &str) -> Result<()> {
        let primals = self.primals.read().await;
        if let Some(primal) = primals.get(name) {
            if let Some(pid) = primal.pid {
                info!("🔪 Killing {} (PID: {})", name, pid);

                // Send SIGTERM first
                #[cfg(unix)]
                {
                    use nix::sys::signal::{kill, Signal};
                    use nix::unistd::Pid;

                    let pid = Pid::from_raw(pid as i32);

                    // Try graceful SIGTERM
                    if kill(pid, Signal::SIGTERM).is_ok() {
                        // Wait up to 5 seconds for graceful shutdown
                        for _ in 0..50 {
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            if kill(pid, None).is_err() {
                                return Ok(()); // Process dead
                            }
                        }

                        // Force SIGKILL if still running
                        warn!("⚠️ {} didn't terminate gracefully, sending SIGKILL", name);
                        kill(pid, Signal::SIGKILL).ok();
                    }
                }
            }
        }
        Ok(())
    }

    /// Respawn a primal from its deployment node
    async fn respawn_primal(&self, name: &str, node: &GraphNode) -> Result<()> {
        info!("🌱 Respawning {} from deployment node", name);

        // Create execution context
        let mut env = HashMap::new();
        env.insert("FAMILY_ID".to_string(), self.family_id.clone());

        let context = ExecutionContext::new(env).with_nucleation(self.nucleation.clone());

        // Get operation mode
        let mode = node
            .operation
            .as_ref()
            .and_then(|op| op.params.get("mode"))
            .and_then(|v| v.as_str())
            .unwrap_or("server");

        // Spawn primal
        let child = primal_spawner::spawn_primal_process(name, mode, &context, node).await?;

        let pid = child.id();

        // Update primal state
        let mut primals = self.primals.write().await;
        if let Some(primal) = primals.get_mut(name) {
            primal.pid = pid;
            primal.state = LifecycleState::Incubating {
                started_at: chrono::Utc::now(),
                timeout_ms: 30000,
            };
            primal.metrics.health_failures = 0;
        }

        // Relay output streams
        primal_spawner::relay_output_streams(child, name.to_string());

        info!("✅ {} respawned (PID: {:?})", name, pid);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifecycle_manager::{
        HealthConfig, ManagedPrimal, PrimalMetrics, ResurrectionConfig,
    };
    use std::path::PathBuf;
    use std::time::Duration;

    fn test_managed_primal(name: &str, state: LifecycleState) -> ManagedPrimal {
        ManagedPrimal {
            name: name.to_string(),
            family_id: "test-family".to_string(),
            socket_path: PathBuf::from(format!("/tmp/test-{}.sock", name)),
            pid: None,
            state,
            deployment_node: None,
            depends_on: vec![],
            depended_by: vec![],
            health_config: HealthConfig::default(),
            resurrection_config: ResurrectionConfig::default(),
            metrics: PrimalMetrics::default(),
        }
    }

    #[allow(dead_code)]
    #[tokio::test]
    async fn test_attempt_resurrection_nonexistent_primal() {
        let manager = LifecycleManager::new("test-family");
        let result = manager.attempt_resurrection("nonexistent").await;
        assert!(result.is_err(), "Expected error for nonexistent primal");
        assert!(
            result.unwrap_err().to_string().contains("not found"),
            "Error should mention primal not found"
        );
    }

    #[tokio::test]
    async fn test_attempt_resurrection_disabled() {
        let manager = LifecycleManager::new("test-family");
        {
            let mut primals = manager.primals.write().await;
            let mut primal = test_managed_primal(
                "disabled-primal",
                LifecycleState::Degraded {
                    since: chrono::Utc::now(),
                    reason: "test".to_string(),
                    resurrection_attempts: 0,
                },
            );
            primal.resurrection_config.enabled = false;
            primals.insert("disabled-primal".to_string(), primal);
        }

        let result = manager.attempt_resurrection("disabled-primal").await;
        result.expect("should succeed (no-op when disabled)");

        let status = manager.get_status().await;
        assert!(
            matches!(
                status.get("disabled-primal"),
                Some(LifecycleState::Degraded { .. })
            ),
            "State should remain Degraded when resurrection disabled"
        );
    }

    #[tokio::test]
    async fn test_attempt_resurrection_skips_non_degraded() {
        let manager = LifecycleManager::new("test-family");
        {
            let mut primals = manager.primals.write().await;
            primals.insert(
                "active-primal".to_string(),
                test_managed_primal(
                    "active-primal",
                    LifecycleState::Active {
                        since: chrono::Utc::now(),
                        last_health_check: chrono::Utc::now(),
                    },
                ),
            );
            primals.insert(
                "incubating-primal".to_string(),
                test_managed_primal(
                    "incubating-primal",
                    LifecycleState::Incubating {
                        started_at: chrono::Utc::now(),
                        timeout_ms: 30000,
                    },
                ),
            );
        }

        for name in &["active-primal", "incubating-primal"] {
            let result = manager.attempt_resurrection(name).await;
            result.expect("should succeed (no-op for non-degraded)");
        }
    }

    #[tokio::test]
    async fn test_attempt_resurrection_max_attempts_exhausted() {
        let manager = LifecycleManager::new("test-family");
        {
            let mut primals = manager.primals.write().await;
            let mut primal = test_managed_primal(
                "exhausted-primal",
                LifecycleState::Degraded {
                    since: chrono::Utc::now(),
                    reason: "test".to_string(),
                    resurrection_attempts: 5,
                },
            );
            primal.resurrection_config.max_attempts = 5;
            primals.insert("exhausted-primal".to_string(), primal);
        }

        let result = manager.attempt_resurrection("exhausted-primal").await;
        result.expect("should succeed (transitions to Apoptosis)");

        let status = manager.get_status().await;
        assert!(
            matches!(
                status.get("exhausted-primal"),
                Some(LifecycleState::Apoptosis {
                    reason: ApoptosisReason::ResurrectionExhausted,
                    ..
                })
            ),
            "Should transition to Apoptosis when max attempts exhausted"
        );
    }

    #[tokio::test]
    async fn test_attempt_resurrection_no_deployment_node() {
        let manager = LifecycleManager::new("test-family");
        {
            let mut primals = manager.primals.write().await;
            let mut primal = test_managed_primal(
                "no-node-primal",
                LifecycleState::Degraded {
                    since: chrono::Utc::now(),
                    reason: "test".to_string(),
                    resurrection_attempts: 0,
                },
            );
            primal.resurrection_config.base_delay = Duration::from_millis(1);
            primal.resurrection_config.max_delay = Duration::from_millis(10);
            primals.insert("no-node-primal".to_string(), primal);
        }

        let result = manager.attempt_resurrection("no-node-primal").await;
        result.expect("should succeed (warns but no failure)");

        let info = manager.get_primal_info("no-node-primal").await.unwrap();
        assert_eq!(
            info.metrics.resurrection_count, 1,
            "Should increment resurrection count even when no deployment node"
        );
    }

    #[tokio::test]
    async fn test_kill_primal_process_no_primal() {
        let manager = LifecycleManager::new("test-family");
        let result = manager.kill_primal_process("nonexistent").await;
        result.expect("should succeed (no-op)");
    }

    #[tokio::test]
    async fn test_kill_primal_process_no_pid() {
        let manager = LifecycleManager::new("test-family");
        {
            let mut primals = manager.primals.write().await;
            primals.insert(
                "no-pid-primal".to_string(),
                test_managed_primal(
                    "no-pid-primal",
                    LifecycleState::Degraded {
                        since: chrono::Utc::now(),
                        reason: "test".to_string(),
                        resurrection_attempts: 0,
                    },
                ),
            );
        }

        let result = manager.kill_primal_process("no-pid-primal").await;
        result.expect("should succeed (no-op when no pid)");
    }

    #[test]
    fn test_backoff_delay_calculation() {
        let base = Duration::from_secs(2);
        let max = Duration::from_secs(60);

        assert_eq!(
            std::cmp::min(base * 2u32.pow(0), max),
            Duration::from_secs(2)
        );
        assert_eq!(
            std::cmp::min(base * 2u32.pow(1), max),
            Duration::from_secs(4)
        );
        assert_eq!(
            std::cmp::min(base * 2u32.pow(2), max),
            Duration::from_secs(8)
        );
        assert_eq!(
            std::cmp::min(base * 2u32.pow(10), max),
            Duration::from_secs(60)
        );
    }
}
