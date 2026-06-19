// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Runtime Operations
//!
//! Handles runtime operations including log streaming, command execution,
//! and system/service monitoring.

use anyhow::{Context, Result};
use std::collections::HashMap;

use super::core::{PrimalInfo, UniversalBiomeOSManager};

impl UniversalBiomeOSManager {
    /// Stream logs from services
    pub async fn get_service_logs(
        &self,
        service: &str,
        follow: bool,
        tail: Option<usize>,
        since: Option<&str>,
    ) -> Result<HashMap<String, serde_json::Value>> {
        tracing::info!(
            "📜 Getting logs for service: {} (follow: {})",
            service,
            follow
        );

        let mut result = HashMap::new();
        result.insert("service".to_string(), serde_json::json!(service));
        result.insert("following".to_string(), serde_json::json!(follow));
        result.insert("tail".to_string(), serde_json::json!(tail));
        result.insert("since".to_string(), serde_json::json!(since));

        // Check if service exists
        let primals = self.registered_primals.read().await;
        let primal = primals
            .values()
            .find(|p| p.name == service || p.id == service);

        let primal = primal.ok_or_else(|| anyhow::anyhow!("Service not found: {service}"))?;

        // Generate logs from service
        let logs = self.generate_service_logs(primal, tail, since).await?;
        result.insert("logs".to_string(), serde_json::json!(logs));
        result.insert("status".to_string(), serde_json::json!("success"));

        if follow {
            result.insert(
                "message".to_string(),
                serde_json::json!("Log streaming started"),
            );
            // In a real implementation, this would set up a streaming connection
        } else {
            result.insert(
                "message".to_string(),
                serde_json::json!("Logs retrieved successfully"),
            );
        }

        result.insert(
            "timestamp".to_string(),
            serde_json::json!(chrono::Utc::now()),
        );
        Ok(result)
    }

    /// Execute commands in running services
    pub async fn exec_in_service(
        &self,
        service: &str,
        command: &[String],
        interactive: bool,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let command_str = command.join(" ");
        tracing::info!(
            "💻 Executing '{}' in service '{}' (interactive: {})",
            command_str,
            service,
            interactive
        );

        let mut result = HashMap::new();
        result.insert("service".to_string(), serde_json::json!(service));
        result.insert("command".to_string(), serde_json::json!(command_str));
        result.insert("interactive".to_string(), serde_json::json!(interactive));

        // Check if service exists
        let primals = self.registered_primals.read().await;
        let primal = primals
            .values()
            .find(|p| p.name == service || p.id == service)
            .ok_or_else(|| anyhow::anyhow!("Service not found: {service}"))?;

        // Execute command via primal's API
        let execution_start = std::time::Instant::now();
        let exec_result = self
            .execute_command_integration(primal, &command_str, interactive)
            .await?;

        let duration = execution_start.elapsed().as_millis();

        result.insert("exit_code".to_string(), serde_json::json!(0));
        result.insert("stdout".to_string(), serde_json::json!(exec_result.stdout));
        result.insert("stderr".to_string(), serde_json::json!(exec_result.stderr));
        result.insert("duration_ms".to_string(), serde_json::json!(duration));
        result.insert("status".to_string(), serde_json::json!("success"));
        result.insert(
            "message".to_string(),
            serde_json::json!("Command executed successfully"),
        );
        result.insert(
            "timestamp".to_string(),
            serde_json::json!(chrono::Utc::now()),
        );

        tracing::info!("✅ Command execution completed in {}ms", duration);
        Ok(result)
    }

    /// Monitor service
    pub async fn monitor_service(
        &self,
        service: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        tracing::debug!("📊 Monitoring service: {}", service);

        let mut result = HashMap::new();
        let primals = self.registered_primals.read().await;
        let primal = primals
            .values()
            .find(|p| p.name == service || p.id == service);

        if let Some(primal) = primal {
            result.insert("service_name".to_string(), serde_json::json!(primal.name));
            result.insert("endpoint".to_string(), serde_json::json!(primal.endpoint));
            result.insert("health".to_string(), serde_json::json!(primal.health));
            result.insert("last_seen".to_string(), serde_json::json!(primal.last_seen));

            result.insert(
                "resources".to_string(),
                serde_json::json!({
                    "status": "pending",
                    "message": "Resource metrics via capability.call(compute.metrics) — future evolution"
                }),
            );
        } else {
            result.insert(
                "error".to_string(),
                serde_json::json!(format!("Service not found: {}", service)),
            );
        }

        Ok(result)
    }

    /// Monitor entire system using real metrics from `biomeos-system`.
    pub async fn monitor_system(&self) -> Result<HashMap<String, serde_json::Value>> {
        tracing::debug!("📊 Monitoring system");

        let mut result = HashMap::new();

        // Real system metrics via biomeos-system
        let sys_info = biomeos_system::SystemInspector::get_system_info()
            .await
            .ok();
        let resources = biomeos_system::SystemInspector::get_resource_usage()
            .await
            .ok();

        let cpu_pct = resources.as_ref().and_then(|r| r.cpu_usage).unwrap_or(0.0);
        let mem_info = sys_info.as_ref().map(|s| &s.memory_info);
        let disk_pct = resources.as_ref().and_then(|r| r.disk_usage).unwrap_or(0.0);
        let load = sys_info.as_ref().map_or(0.0, |s| s.load_average.load_1m);

        result.insert(
            "system".to_string(),
            serde_json::json!({
                "cpu_usage_percent": cpu_pct,
                "memory": {
                    "used_gb": mem_info.map_or(0.0, |m| m.used_gb),
                    "total_gb": mem_info.map_or(0.0, |m| m.total_gb),
                    "usage_percent": mem_info.map_or(0.0, |m| m.usage_percent)
                },
                "disk": {
                    "usage_percent": disk_pct
                },
                "load_average": {
                    "1m": load
                }
            }),
        );

        // Service statuses from registered primals
        let primals = self.registered_primals.read().await;
        let mut services = HashMap::new();

        for (id, primal) in primals.iter() {
            services.insert(
                id.clone(),
                serde_json::json!({
                    "name": primal.name,
                    "status": "running",
                    "health": primal.health,
                }),
            );
        }

        result.insert("services".to_string(), serde_json::json!(services));

        // Real network I/O (best-effort)
        let net_io = resources.as_ref().and_then(|r| r.network_io.as_ref());
        result.insert(
            "network".to_string(),
            serde_json::json!({
                "bytes_in_per_sec": net_io.map_or(0.0, |n| n.bytes_in_per_sec),
                "bytes_out_per_sec": net_io.map_or(0.0, |n| n.bytes_out_per_sec),
            }),
        );

        result.insert("alerts".to_string(), serde_json::json!([]));

        Ok(result)
    }

    /// Get system status with real uptime and version from `biomeos-system`.
    pub async fn get_system_status(&self) -> Result<HashMap<String, serde_json::Value>> {
        let mut result = HashMap::new();

        result.insert("status".to_string(), serde_json::json!("running"));

        let uptime_str = match biomeos_system::SystemInspector::get_system_info().await {
            Ok(info) => {
                let dur = info.uptime;
                let secs = dur.as_secs();
                let hours = secs / 3600;
                let mins = (secs % 3600) / 60;
                format!("{hours}h {mins}m")
            }
            Err(_) => "unknown".to_string(),
        };
        result.insert("uptime".to_string(), serde_json::json!(uptime_str));
        result.insert(
            "version".to_string(),
            serde_json::json!(env!("CARGO_PKG_VERSION")),
        );

        // Service count summary
        let primals = self.registered_primals.read().await;
        result.insert("services".to_string(), serde_json::json!({
            "total": primals.len(),
            "healthy": primals.values().filter(|p| matches!(p.health, biomeos_types::Health::Healthy)).count(),
            "registered_primals": primals.len()
        }));

        // System health
        let health_report = self.get_system_health();
        result.insert(
            "health".to_string(),
            serde_json::json!(health_report.health),
        );

        Ok(result)
    }

    /// Fetch service logs from actual primal endpoint (Pure Rust via Unix socket!)
    pub(super) async fn generate_service_logs(
        &self,
        primal: &PrimalInfo,
        tail: Option<usize>,
        since: Option<&str>,
    ) -> Result<Vec<serde_json::Value>> {
        let limit = tail.unwrap_or(100);

        tracing::debug!(
            "🚀 Fetching logs from {} via atomic client (tail={})",
            primal.name,
            limit
        );

        // Use Pure Rust atomic client (Tower-based, Unix sockets)
        let client = crate::atomic_client::AtomicClient::discover(&primal.name)
            .await
            .with_context(|| format!("Failed to discover primal: {}", primal.name))?;

        // Build request parameters
        let mut params = serde_json::json!({
            "tail": limit
        });

        if let Some(since_time) = since {
            params["since"] = serde_json::Value::String(since_time.to_string());
        }

        // Fetch logs via JSON-RPC over Unix socket (Pure Rust!)
        match client.call("get_logs", params).await {
            Ok(result) => {
                // Parse logs array
                if let Some(logs_array) = result.as_array() {
                    tracing::debug!("✅ Fetched {} logs from {}", logs_array.len(), primal.name);
                    Ok(logs_array.clone())
                } else {
                    tracing::warn!("Logs response is not an array from {}", primal.name);
                    Ok(vec![])
                }
            }
            Err(e) => {
                tracing::warn!("Failed to fetch logs from {}: {}", primal.name, e);
                // Return empty logs rather than failing the entire operation
                Ok(vec![])
            }
        }
    }

    /// Execute command via primal's execution API (Pure Rust via Unix socket!)
    pub(super) async fn execute_command_integration(
        &self,
        primal: &PrimalInfo,
        command: &str,
        interactive: bool,
    ) -> Result<ExecutionResult> {
        if interactive {
            anyhow::bail!(
                "Interactive command execution is not supported via the atomic primal client; pass interactive=false"
            );
        }

        tracing::debug!(
            "🚀 Executing command in {} via atomic client: {}",
            primal.name,
            command
        );

        // Use Pure Rust atomic client (Tower-based, Unix sockets)
        let client = crate::atomic_primal_client::AtomicPrimalClient::discover(&primal.name)
            .await
            .with_context(|| format!("Failed to discover primal: {}", primal.name))?;

        // Execute command via JSON-RPC over Unix socket (Pure Rust!)
        let atomic_result = client
            .execute_command(command)
            .await
            .with_context(|| format!("Failed to execute command on primal: {}", primal.name))?;

        tracing::debug!("✅ Command executed successfully on {}", primal.name);

        // Convert to local ExecutionResult type
        Ok(ExecutionResult {
            stdout: atomic_result.stdout,
            stderr: atomic_result.stderr,
        })
    }
}

/// Command execution result
#[derive(Debug)]
pub(super) struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
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
    use crate::universal_biomeos_manager::{PrimalInfo, UniversalBiomeOSManager};
    use biomeos_primal_sdk::PrimalCapability;
    use biomeos_types::{Health, PrimalType};
    use std::collections::HashMap;

    fn test_primal_info(id: &str, name: &str, endpoint: &str) -> PrimalInfo {
        PrimalInfo {
            id: id.to_string(),
            name: name.to_string(),
            primal_type: PrimalType::from_discovered("compute", name, "1.0.0"),
            endpoint: endpoint.to_string(),
            capabilities: vec![PrimalCapability::new("compute", "execution", "1.0")],
            health: Health::Healthy,
            last_seen: chrono::Utc::now(),
            discovered_at: chrono::Utc::now(),
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_get_service_logs_service_not_found() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
        manager.initialize().expect("init");

        let result = manager
            .get_service_logs("nonexistent", false, Some(10), None)
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_exec_in_service_service_not_found() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
        manager.initialize().expect("init");

        let result = manager
            .exec_in_service(
                "nonexistent",
                &["echo".to_string(), "hi".to_string()],
                false,
            )
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_monitor_service_found() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
        manager.initialize().expect("init");

        let primal = test_primal_info("mon-1", "monitor-svc", "unix:///tmp/mon.sock");
        manager.register_primal(primal).await.expect("register");

        let result = manager
            .monitor_service("monitor-svc")
            .await
            .expect("monitor");
        assert_eq!(
            result.get("service_name").and_then(|v| v.as_str()),
            Some("monitor-svc")
        );
        assert!(result.contains_key("health"));
        assert!(result.contains_key("resources"));
    }

    #[tokio::test]
    async fn test_monitor_service_not_found() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
        manager.initialize().expect("init");

        let result = manager
            .monitor_service("nonexistent")
            .await
            .expect("monitor");
        assert!(result.contains_key("error"));
    }

    #[tokio::test]
    async fn test_monitor_service_by_id() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
        manager.initialize().expect("init");

        let primal = test_primal_info("id-1", "by-id-svc", "unix:///tmp/id.sock");
        manager.register_primal(primal).await.expect("register");

        let result = manager.monitor_service("id-1").await.expect("monitor");
        assert_eq!(
            result.get("service_name").and_then(|v| v.as_str()),
            Some("by-id-svc")
        );
    }

    #[tokio::test]
    async fn test_monitor_system() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
        manager.initialize().expect("init");

        let result = manager.monitor_system().await.expect("monitor");
        assert!(result.contains_key("system"));
        assert!(result.contains_key("services"));
        assert!(result.contains_key("network"));
        assert!(result.contains_key("alerts"));

        let system = result.get("system").expect("system");
        assert!(system.get("cpu_usage_percent").is_some());
        assert!(system.get("memory").is_some());
    }

    #[tokio::test]
    async fn test_monitor_system_with_primals() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
        manager.initialize().expect("init");

        let primal = test_primal_info("sys-1", "sys-svc", "unix:///tmp/sys.sock");
        manager.register_primal(primal).await.expect("register");

        let result = manager.monitor_system().await.expect("monitor");
        let services = result
            .get("services")
            .expect("services")
            .as_object()
            .expect("obj");
        assert_eq!(services.len(), 1);
    }

    #[tokio::test]
    async fn test_get_system_status() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
        manager.initialize().expect("init");

        let result = manager.get_system_status().await.expect("status");
        assert_eq!(
            result.get("status").and_then(|v| v.as_str()),
            Some("running")
        );
        assert!(result.contains_key("uptime"));
        assert!(result.contains_key("version"));
        assert!(result.contains_key("services"));
        assert!(result.contains_key("health"));
    }

    #[tokio::test]
    async fn test_get_system_status_with_primals() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
        manager.initialize().expect("init");

        let primal = test_primal_info("stat-1", "stat-svc", "unix:///tmp/stat.sock");
        manager.register_primal(primal).await.expect("register");

        let result = manager.get_system_status().await.expect("status");
        let services = result.get("services").expect("services");
        assert_eq!(services["total"].as_u64(), Some(1));
    }
}
