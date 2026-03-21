// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Health monitoring: interval-based health checks and state transitions

use anyhow::Result;
use std::time::Instant;
use tracing::{debug, error, info, warn};

use super::{LifecycleManager, LifecycleState};

impl LifecycleManager {
    /// Start the health monitoring loop
    ///
    /// This runs continuously, checking primal health and triggering
    /// resurrection when needed.
    pub async fn start_monitoring(&self) -> Result<()> {
        info!(
            "🔍 Starting lifecycle monitoring (interval: {:?})",
            self.health_check_interval
        );

        let primals = self.primals.clone();
        let shutdown = self.shutdown.clone();
        let _health_checker = self.health_checker.clone();
        let interval = self.health_check_interval;
        let manager = self.clone_for_task();

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                // Check shutdown flag
                if *shutdown.read().await {
                    info!("🛑 Monitoring loop shutting down");
                    break;
                }

                // Check health of all primals
                let primal_names: Vec<String> = {
                    let primals = primals.read().await;
                    primals.keys().cloned().collect()
                };

                for name in primal_names {
                    if let Err(e) = manager.check_primal_health(&name).await {
                        warn!("Health check failed for {}: {}", name, e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Check health of a single primal
    ///
    /// Uses deep JSON-RPC health checks for active primals (validates responsiveness),
    /// and socket-only checks during incubation (lighter weight).
    pub(crate) async fn check_primal_health(&self, name: &str) -> Result<()> {
        let mut primals = self.primals.write().await;
        let primal = primals
            .get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Primal not found: {name}"))?;

        // Skip if not in active or incubating state
        match &primal.state {
            LifecycleState::Active { .. } | LifecycleState::Incubating { .. } => {}
            LifecycleState::Degraded { .. } => {
                // Already degraded, resurrection handler will manage
                return Ok(());
            }
            _ => return Ok(()),
        }

        let start = Instant::now();

        // Deep health check for active primals (JSON-RPC ping),
        // socket-only for incubating (primal may not have RPC ready yet)
        let health_result = match &primal.state {
            LifecycleState::Active { .. } => {
                self.health_checker
                    .check_primal_deep(&primal.socket_path, &primal.health_config.health_method)
                    .await?
            }
            _ => {
                self.health_checker
                    .check_primal(&primal.socket_path)
                    .await?
            }
        };

        primal.metrics.last_health_latency_ms = start.elapsed().as_millis() as u64;

        if health_result.is_healthy {
            // Transition to active if incubating
            match &primal.state {
                LifecycleState::Incubating { .. } => {
                    info!("✅ {} transitioned to ACTIVE", name);
                    primal.state = LifecycleState::Active {
                        since: chrono::Utc::now(),
                        last_health_check: chrono::Utc::now(),
                    };
                }
                LifecycleState::Active { since, .. } => {
                    primal.state = LifecycleState::Active {
                        since: *since,
                        last_health_check: chrono::Utc::now(),
                    };
                }
                _ => {}
            }

            debug!(
                "💚 {} healthy ({}ms)",
                name, primal.metrics.last_health_latency_ms
            );
        } else {
            // Health check failed
            primal.metrics.health_failures += 1;

            if primal.metrics.health_failures >= primal.health_config.failure_threshold {
                warn!(
                    "🔴 {} DEGRADED after {} failures: {:?}",
                    name, primal.metrics.health_failures, health_result.message
                );

                primal.state = LifecycleState::Degraded {
                    since: chrono::Utc::now(),
                    reason: health_result
                        .message
                        .unwrap_or_else(|| "Health check failed".to_string()),
                    resurrection_attempts: 0,
                };

                // Drop lock before spawning resurrection
                let name_clone = name.to_string();
                let manager = self.clone_for_task();
                drop(primals);

                // Trigger resurrection
                tokio::spawn(async move {
                    if let Err(e) = manager.attempt_resurrection(&name_clone).await {
                        error!("Resurrection failed for {}: {}", name_clone, e);
                    }
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use crate::lifecycle_manager::ApoptosisReason;
    use std::os::unix::net::UnixListener;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_check_primal_health_not_found() {
        let manager = LifecycleManager::new("test-family");
        let result = manager.check_primal_health("nonexistent").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_check_primal_health_skips_dead_state() {
        let manager = LifecycleManager::new("test-family");
        manager
            .register_primal("dead-primal", PathBuf::from("/tmp/dead.sock"), None, None)
            .await
            .unwrap();

        manager
            .apoptosis("dead-primal", ApoptosisReason::UserRequest)
            .await
            .unwrap();

        let result = manager.check_primal_health("dead-primal").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_start_monitoring_returns_ok() {
        let manager = LifecycleManager::new("test-family");
        let result = manager.start_monitoring().await;
        assert!(result.is_ok());
        manager.shutdown_all().await.unwrap();
    }

    #[tokio::test]
    async fn test_check_primal_health_incubating_becomes_active_with_socket() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let sock = tmp.path().join("incubating.sock");
        let _listener = UnixListener::bind(&sock).expect("bind socket");

        let manager = LifecycleManager::new("test-family");
        manager
            .register_primal("incu", sock, None, None)
            .await
            .expect("register");

        manager
            .check_primal_health("incu")
            .await
            .expect("health check");

        let state = manager.get_primal_info("incu").await.expect("info");
        assert!(matches!(state.state, LifecycleState::Active { .. }));
    }

    #[tokio::test]
    async fn test_check_primal_health_skips_degraded_early_return() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let sock = tmp.path().join("deg.sock");
        let _listener = UnixListener::bind(&sock).expect("bind");

        let manager = LifecycleManager::new("test-family");
        manager
            .register_primal("deg-p", sock, None, None)
            .await
            .expect("register");

        {
            let mut primals = manager.primals.write().await;
            let p = primals.get_mut("deg-p").expect("primal");
            p.state = LifecycleState::Degraded {
                since: chrono::Utc::now(),
                reason: "test".to_string(),
                resurrection_attempts: 0,
            };
        }

        manager
            .check_primal_health("deg-p")
            .await
            .expect("degraded should no-op ok");
    }

    #[tokio::test]
    async fn test_check_primal_health_germinating_skipped() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let sock = tmp.path().join("germ.sock");
        let _listener = UnixListener::bind(&sock).expect("bind");

        let manager = LifecycleManager::new("test-family");
        manager
            .register_primal("germ", sock, None, None)
            .await
            .expect("register");

        {
            let mut primals = manager.primals.write().await;
            let p = primals.get_mut("germ").expect("primal");
            p.state = LifecycleState::Germinating;
        }

        manager
            .check_primal_health("germ")
            .await
            .expect("germinating skip");
    }

    #[tokio::test]
    async fn test_check_primal_health_active_increments_failures_toward_degraded() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let sock = tmp.path().join("active.sock");
        let _listener = UnixListener::bind(&sock).expect("bind");

        let manager = LifecycleManager::new("test-family");
        manager
            .register_primal("act", sock.clone(), None, None)
            .await
            .expect("register");

        // Incubating → Active (socket-only check succeeds)
        manager.check_primal_health("act").await.expect("first");
        // Deep checks fail (no JSON-RPC server) but socket exists
        manager.check_primal_health("act").await.expect("f1");
        manager.check_primal_health("act").await.expect("f2");
        let info = manager.get_primal_info("act").await.expect("info");
        assert_eq!(info.metrics.health_failures, 2);

        manager.check_primal_health("act").await.expect("f3");
        let info = manager
            .get_primal_info("act")
            .await
            .expect("info after threshold");
        assert!(matches!(info.state, LifecycleState::Degraded { .. }));
        assert!(info.metrics.health_failures >= 3);
    }
}
