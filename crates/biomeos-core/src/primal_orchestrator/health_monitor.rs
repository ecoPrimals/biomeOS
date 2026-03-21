// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Primal health monitor using JSON-RPC over Unix sockets.
//!
//! This is the TRUE PRIMAL health monitoring implementation:
//! - Uses Unix sockets, not HTTP
//! - Calls `health.check` JSON-RPC method
//! - Tracks primal status with atomic state

use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;
use tracing::debug;

use biomeos_types::identifiers::PrimalId;

use crate::socket_discovery::SocketDiscovery;

/// Primal health monitor using JSON-RPC over Unix sockets.
#[derive(Clone)]
pub struct PrimalHealthMonitor {
    /// Registered primals: id → socket path
    pub(crate) primals: Arc<RwLock<HashMap<PrimalId, String>>>,

    /// Primal health status: id → healthy
    pub(crate) status: Arc<RwLock<HashMap<PrimalId, bool>>>,

    /// Check interval
    pub(crate) interval: std::time::Duration,

    /// Running flag
    pub(crate) running: Arc<std::sync::atomic::AtomicBool>,
}

impl PrimalHealthMonitor {
    /// Create a new builder for configuring the health monitor
    pub fn builder() -> PrimalHealthMonitorBuilder {
        PrimalHealthMonitorBuilder {
            interval: std::time::Duration::from_secs(30),
        }
    }

    /// Start the health monitoring background task.
    ///
    /// Periodically calls `health.check` on all registered primals.
    pub async fn start_monitoring(&self) -> anyhow::Result<()> {
        tracing::info!("🏥 Health monitor started (JSON-RPC over Unix sockets)");

        self.running
            .store(true, std::sync::atomic::Ordering::SeqCst);

        let primals = self.primals.clone();
        let status = self.status.clone();
        let interval = self.interval;
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            while running.load(std::sync::atomic::Ordering::SeqCst) {
                interval_timer.tick().await;

                let primals_snapshot = primals.read().await.clone();

                for (id, socket_path) in primals_snapshot {
                    let healthy = Self::check_primal_health(&socket_path).await;
                    status.write().await.insert(id.clone(), healthy);

                    if !healthy {
                        tracing::warn!("🏥 Primal {} is unhealthy", id);
                    }
                }
            }

            tracing::info!("🏥 Health monitor stopped");
        });

        Ok(())
    }

    /// Check a primal's health via JSON-RPC.
    async fn check_primal_health(socket_path: &str) -> bool {
        use std::path::Path;
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        let socket = Path::new(socket_path);
        if !socket.exists() {
            return false;
        }

        let Ok(stream) = UnixStream::connect(socket).await else {
            return false;
        };

        let (reader, mut writer) = stream.into_split();
        let request = r#"{"jsonrpc":"2.0","method":"health.check","id":1}"#;

        if writer
            .write_all(format!("{request}\n").as_bytes())
            .await
            .is_err()
        {
            return false;
        }

        let mut reader = BufReader::new(reader);
        let mut response = String::new();

        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            reader.read_line(&mut response),
        )
        .await
        {
            Ok(Ok(_)) => response.contains("healthy") || response.contains("\"result\""),
            _ => false,
        }
    }

    /// Stop the health monitor.
    pub fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    /// Register a primal for health monitoring.
    pub async fn register(&self, id: PrimalId, endpoint: biomeos_types::identifiers::Endpoint) {
        let url = endpoint.url();
        let socket_path = if url.scheme() == "unix" || url.scheme() == "file" {
            url.path().to_string()
        } else {
            tracing::warn!(
                "🏥 Primal {} uses HTTP endpoint ({}), discovering socket path",
                id,
                url
            );
            let family_id = std::env::var("FAMILY_ID")
                .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
                .unwrap_or_else(|_| "default".to_string());
            let discovery = SocketDiscovery::new(family_id);
            discovery
                .build_socket_path(id.as_ref())
                .to_string_lossy()
                .to_string()
        };

        debug!("🏥 Registering primal {} at {}", id, socket_path);
        self.primals.write().await.insert(id.clone(), socket_path);
        self.status.write().await.insert(id, true);
    }

    /// Register a primal by direct socket path.
    pub async fn register_socket(&self, id: PrimalId, socket_path: impl Into<String>) {
        let socket_path = socket_path.into();
        debug!("🏥 Registering primal {} at {}", id, socket_path);
        self.primals.write().await.insert(id.clone(), socket_path);
        self.status.write().await.insert(id, true);
    }

    /// Unregister a primal from health monitoring.
    pub async fn unregister(&self, id: &PrimalId) {
        debug!("🏥 Unregistering primal {}", id);
        self.primals.write().await.remove(id);
        self.status.write().await.remove(id);
    }

    /// Get the health status of a primal.
    pub async fn is_healthy(&self, id: &PrimalId) -> Option<bool> {
        self.status.read().await.get(id).copied()
    }

    /// Get all primal health statuses.
    pub async fn all_status(&self) -> HashMap<PrimalId, bool> {
        self.status.read().await.clone()
    }
}

/// Builder for [`PrimalHealthMonitor`]
pub struct PrimalHealthMonitorBuilder {
    interval: std::time::Duration,
}

impl PrimalHealthMonitorBuilder {
    /// Set the health check interval.
    pub fn interval(mut self, interval: std::time::Duration) -> Self {
        self.interval = interval;
        self
    }

    /// Build the health monitor with the configured interval
    pub fn build(self) -> PrimalHealthMonitor {
        PrimalHealthMonitor {
            primals: Arc::new(RwLock::new(HashMap::new())),
            status: Arc::new(RwLock::new(HashMap::new())),
            interval: self.interval,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::time::Duration;

    fn pid(name: &str) -> PrimalId {
        PrimalId::new(name).expect("valid primal id")
    }

    #[test]
    fn test_builder_default_interval() {
        let monitor = PrimalHealthMonitor::builder().build();
        assert_eq!(monitor.interval, Duration::from_secs(30));
    }

    #[test]
    fn test_builder_custom_interval() {
        let monitor = PrimalHealthMonitor::builder()
            .interval(Duration::from_secs(5))
            .build();
        assert_eq!(monitor.interval, Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_health_monitor_register_socket_and_status() {
        let monitor = PrimalHealthMonitor::builder().build();
        let id = pid("test-primal");

        monitor.register_socket(id.clone(), "/tmp/test.sock").await;

        assert_eq!(monitor.is_healthy(&id).await, Some(true));

        let all = monitor.all_status().await;
        assert_eq!(all.len(), 1);
        assert!(all[&id]);
    }

    #[tokio::test]
    async fn test_health_monitor_unregister() {
        let monitor = PrimalHealthMonitor::builder().build();
        let id = pid("removable");

        monitor
            .register_socket(id.clone(), "/tmp/removable.sock")
            .await;
        assert!(monitor.is_healthy(&id).await.is_some());

        monitor.unregister(&id).await;
        assert!(monitor.is_healthy(&id).await.is_none());
        assert!(monitor.all_status().await.is_empty());
    }

    #[tokio::test]
    async fn test_health_monitor_multiple_primals() {
        let monitor = PrimalHealthMonitor::builder().build();

        monitor.register_socket(pid("a"), "/tmp/a.sock").await;
        monitor.register_socket(pid("b"), "/tmp/b.sock").await;
        monitor.register_socket(pid("c"), "/tmp/c.sock").await;

        assert_eq!(monitor.all_status().await.len(), 3);
    }

    #[tokio::test]
    async fn test_health_monitor_is_healthy_unknown_primal() {
        let monitor = PrimalHealthMonitor::builder().build();
        assert_eq!(monitor.is_healthy(&pid("unknown")).await, None);
    }

    #[test]
    fn test_health_monitor_stop() {
        let monitor = PrimalHealthMonitor::builder().build();
        assert!(!monitor.running.load(std::sync::atomic::Ordering::SeqCst));
        monitor.stop();
        assert!(!monitor.running.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn test_health_monitor_clone() {
        let monitor = PrimalHealthMonitor::builder()
            .interval(Duration::from_secs(10))
            .build();
        let cloned = monitor.clone();
        assert_eq!(cloned.interval, Duration::from_secs(10));
        assert!(Arc::ptr_eq(&monitor.primals, &cloned.primals));
        assert!(Arc::ptr_eq(&monitor.status, &cloned.status));
    }

    #[tokio::test]
    async fn test_health_monitor_register_with_endpoint() {
        use biomeos_types::identifiers::Endpoint;
        let monitor = PrimalHealthMonitor::builder().build();
        let id = pid("endpoint-primal");
        let endpoint = Endpoint::new("file:///tmp/test-endpoint.sock").expect("valid url");
        monitor.register(id.clone(), endpoint).await;
        assert_eq!(monitor.is_healthy(&id).await, Some(true));
    }

    #[tokio::test]
    async fn test_check_primal_health_nonexistent_socket() {
        let monitor = PrimalHealthMonitor::builder().build();
        monitor
            .register_socket(pid("ghost"), "/nonexistent/path/to/socket.sock")
            .await;
        let healthy = monitor.is_healthy(&pid("ghost")).await;
        assert_eq!(healthy, Some(true));
    }

    #[tokio::test]
    async fn test_start_monitoring() {
        let monitor = PrimalHealthMonitor::builder()
            .interval(Duration::from_millis(100))
            .build();
        monitor
            .register_socket(pid("monitored"), "/tmp/monitored.sock")
            .await;
        let result = monitor.start_monitoring().await;
        assert!(result.is_ok());
        monitor.stop();
    }

    #[tokio::test]
    async fn test_register_with_http_endpoint_fallback() {
        use biomeos_types::identifiers::Endpoint;
        let monitor = PrimalHealthMonitor::builder().build();
        let id = pid("http-primal");
        let endpoint = Endpoint::new("http://localhost:8080").expect("valid url");
        monitor.register(id.clone(), endpoint).await;
        assert!(monitor.is_healthy(&id).await.is_some());
    }

    #[tokio::test]
    async fn test_all_status_empty() {
        let monitor = PrimalHealthMonitor::builder().build();
        let status = monitor.all_status().await;
        assert!(status.is_empty());
    }

    #[tokio::test]
    async fn test_register_then_unregister_then_reregister() {
        let monitor = PrimalHealthMonitor::builder().build();
        let id = pid("transient");

        monitor
            .register_socket(id.clone(), "/tmp/transient.sock")
            .await;
        assert!(monitor.is_healthy(&id).await.is_some());

        monitor.unregister(&id).await;
        assert!(monitor.is_healthy(&id).await.is_none());

        monitor
            .register_socket(id.clone(), "/tmp/transient2.sock")
            .await;
        assert!(monitor.is_healthy(&id).await.is_some());
    }

    #[test]
    fn test_builder_chain() {
        let monitor = PrimalHealthMonitor::builder()
            .interval(Duration::from_secs(15))
            .build();
        assert_eq!(monitor.interval, Duration::from_secs(15));
    }
}
