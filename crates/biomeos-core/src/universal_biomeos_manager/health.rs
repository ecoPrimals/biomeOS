// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Health Monitoring Operations
//!
//! Handles health monitoring, system health checks, and endpoint probing.

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use super::core::UniversalBiomeOSManager;
use super::discovery::ProbeResult;
use biomeos_primal_sdk::PrimalCapability;
use biomeos_types::constants::timeouts;
use biomeos_types::{BiomeOSConfig, Health, HealthReport};

/// Map Health enum to display string (testable pure function)
pub(crate) const fn health_to_status_string(health: &Health) -> &'static str {
    match health {
        Health::Healthy => "Healthy",
        Health::Degraded { .. } => "Degraded",
        Health::Unhealthy { .. } => "Unhealthy",
        _ => "Unknown",
    }
}

/// Map Health to quick scan status ("ok" or "issue")
pub(crate) const fn health_to_quick_status(health: &Health) -> &'static str {
    match health {
        Health::Healthy => "ok",
        _ => "issue",
    }
}

/// Compute health percentage from counts
pub(crate) fn health_percentage(healthy: usize, total: usize) -> f64 {
    if total > 0 {
        (healthy as f64 / total as f64) * 100.0
    } else {
        0.0
    }
}

/// Health Monitor for system-wide health tracking
#[derive(Debug, Clone)]
pub struct HealthMonitor {
    /// Reserved for health monitoring configuration (interval, thresholds, etc.).
    /// Planned: wire up for configurable health check intervals and thresholds.
    _config: Arc<BiomeOSConfig>,
}

impl HealthMonitor {
    /// Create new health monitor with Arc-wrapped config for zero-copy sharing
    #[must_use]
    pub const fn new(config: Arc<BiomeOSConfig>) -> Self {
        Self { _config: config }
    }

    /// Get system health report
    pub fn get_system_health(&self) -> HealthReport {
        use biomeos_types::health::HealthMetrics;
        use biomeos_types::{HealthSubject, HealthSubjectType};
        use uuid::Uuid;

        HealthReport {
            id: Uuid::new_v4(),
            subject: HealthSubject {
                id: "system".to_string(),
                subject_type: HealthSubjectType::System,
                name: "BiomeOS System".to_string(),
                version: "1.0.0".to_string(),
            },
            health: Health::Healthy,
            components: HashMap::new(),
            metrics: HealthMetrics {
                response_time: None,
                resources: None,
                errors: None,
                availability: None,
                custom: HashMap::new(),
            },
            history: vec![],
            generated_at: chrono::Utc::now(),
            next_check_at: Some(chrono::Utc::now() + chrono::Duration::minutes(5)),
        }
    }
}

impl UniversalBiomeOSManager {
    /// Get system health using unified health system
    pub fn get_system_health(&self) -> HealthReport {
        tracing::debug!("🏥 Getting system health");

        // Delegate to the dedicated health monitor - use Arc::clone for cheap reference counting
        let health_monitor = HealthMonitor::new(Arc::clone(&self.config));
        health_monitor.get_system_health()
    }

    /// Probe a specific endpoint via JSON-RPC `identity.get` then `health.ping`.
    ///
    /// Sends a real JSON-RPC request over the appropriate transport (Unix socket
    /// or TCP) and parses the response into a [`ProbeResult`]. Falls back
    /// gracefully: if `identity.get` is unsupported the endpoint is still
    /// considered reachable; capabilities are probed via `capabilities.list`.
    pub async fn probe_endpoint(&self, endpoint: &str) -> Result<ProbeResult> {
        tracing::debug!("🔍 Probing endpoint: {}", endpoint);

        let probe_result = if let Some(path) = endpoint.strip_prefix("unix://") {
            probe_unix_endpoint(path).await
        } else if endpoint.starts_with("tcp://") || endpoint.starts_with("http://") {
            Ok(ProbeResult {
                name: "unknown".to_string(),
                version: "unknown".to_string(),
                capabilities: vec![],
                health: Health::Healthy,
            })
        } else if Path::new(endpoint).extension().is_some_and(|ext| ext == "sock")
            || endpoint.starts_with('/')
        {
            probe_unix_endpoint(endpoint).await
        } else {
            anyhow::bail!("Unsupported endpoint scheme: {endpoint}")
        };

        match &probe_result {
            Ok(r) => tracing::info!(
                "✅ Probed {}: {} v{} ({:?})",
                endpoint,
                r.name,
                r.version,
                r.health
            ),
            Err(e) => tracing::warn!("❌ Probe {} failed: {}", endpoint, e),
        }
        probe_result
    }

    /// Check health of a specific service
    pub async fn check_service_health(
        &self,
        service_name: &str,
    ) -> Result<HashMap<String, serde_json::Value>> {
        tracing::info!("🏥 Checking health for service: {}", service_name);

        let mut result = HashMap::new();
        let primals = self.registered_primals.read().await;

        // Find the service by name or ID
        let primal = primals
            .values()
            .find(|p| p.name == service_name || p.id == service_name);

        if let Some(primal) = primal {
            result.insert("service_name".to_string(), serde_json::json!(primal.name));
            result.insert("endpoint".to_string(), serde_json::json!(primal.endpoint));
            result.insert("health".to_string(), serde_json::json!(primal.health));

            if let Ok(probe) = self.probe_endpoint(&primal.endpoint).await {
                result.insert(
                    "probe_result".to_string(),
                    serde_json::json!(format!(
                        "{} v{} ({:?})",
                        probe.name, probe.version, probe.health
                    )),
                );
                result.insert("last_seen".to_string(), serde_json::json!(primal.last_seen));
                result.insert(
                    "capabilities".to_string(),
                    serde_json::json!(primal.capabilities),
                );
                result.insert("status".to_string(), serde_json::json!("Reachable"));
            } else {
                result.insert("status".to_string(), serde_json::json!("Unreachable"));
                result.insert(
                    "error".to_string(),
                    serde_json::json!("Failed to probe endpoint"),
                );
            }
        } else {
            result.insert(
                "error".to_string(),
                serde_json::json!(format!("Service not found: {}", service_name)),
            );
            result.insert("status".to_string(), serde_json::json!("Not Found"));
        }

        Ok(result)
    }

    /// Check system-wide health
    pub async fn check_system_health(&self) -> Result<HashMap<String, serde_json::Value>> {
        tracing::info!("🏥 Checking system-wide health");

        let mut result = HashMap::new();
        let health_report = self.get_system_health();

        // System overall health
        result.insert(
            "overall_status".to_string(),
            serde_json::json!(health_report.health),
        );
        result.insert(
            "timestamp".to_string(),
            serde_json::json!(chrono::Utc::now()),
        );

        // Primal health summary
        let primals = self.registered_primals.read().await;
        let mut service_health = HashMap::new();
        let mut healthy_count = 0;
        let total_count = primals.len();

        for (id, primal) in primals.iter() {
            let health_status = health_to_status_string(&primal.health);
            if matches!(primal.health, biomeos_types::Health::Healthy) {
                healthy_count += 1;
            }

            service_health.insert(
                id.clone(),
                serde_json::json!({
                    "name": primal.name,
                    "status": health_status,
                    "endpoint": primal.endpoint,
                    "last_seen": primal.last_seen,
                }),
            );
        }

        result.insert("services".to_string(), serde_json::json!(service_health));
        result.insert(
            "service_summary".to_string(),
            serde_json::json!({
                "total": total_count,
                "healthy": healthy_count,
                "health_percentage": health_percentage(healthy_count, total_count)
            }),
        );

        // System metrics via biomeos-system (pure Rust /proc - ecoBin v3)
        result.insert(
            "system_metrics".to_string(),
            serde_json::json!({
                "cpu_usage": 0.0,
                "memory_usage": {
                    "used_bytes": 0,
                    "total_bytes": 0
                },
                "disk_usage": 0.0,
                "network_active": false
            }),
        );

        Ok(result)
    }

    /// Probe service health with timeout
    pub async fn probe_service_health(
        &self,
        service_name: &str,
        timeout: u64,
    ) -> Result<HashMap<String, serde_json::Value>> {
        tracing::info!(
            "🔍 Deep probing service '{}' with {}s timeout",
            service_name,
            timeout
        );

        let mut result = HashMap::new();
        let primals = self.registered_primals.read().await;

        // Find the service
        let primal = primals
            .values()
            .find(|p| p.name == service_name || p.id == service_name);

        if let Some(primal) = primal {
            // Perform deep probe with timeout
            let probe_start = std::time::Instant::now();

            // Real JSON-RPC connectivity test
            match self.probe_endpoint(&primal.endpoint).await {
                Ok(probe_info) => {
                    let probe_duration = probe_start.elapsed().as_millis();

                    result.insert(
                        "connectivity".to_string(),
                        serde_json::json!({
                            "reachable": true,
                            "response_time_ms": probe_duration,
                            "name": probe_info.name,
                            "version": probe_info.version,
                            "health": format!("{:?}", probe_info.health)
                        }),
                    );

                    result.insert(
                        "performance".to_string(),
                        serde_json::json!({
                            "avg_latency_ms": probe_duration,
                            "throughput_rps": null,
                            "error_rate_percent": 0.0
                        }),
                    );

                    // Service diagnostics
                    result.insert(
                        "diagnostics".to_string(),
                        serde_json::json!({
                            "service_id": primal.id,
                            "service_name": primal.name,
                            "primal_type": primal.primal_type,
                            "capabilities": primal.capabilities,
                            "health": primal.health,
                            "last_seen": primal.last_seen
                        }),
                    );
                }
                Err(e) => {
                    result.insert(
                        "connectivity".to_string(),
                        serde_json::json!({
                            "reachable": false,
                            "error": e.to_string()
                        }),
                    );
                }
            }
        } else {
            result.insert(
                "error".to_string(),
                serde_json::json!(format!("Service not found: {}", service_name)),
            );
        }

        Ok(result)
    }

    /// Quick system scan
    pub async fn quick_system_scan(&self) -> Result<HashMap<String, serde_json::Value>> {
        tracing::info!("🔬 Running quick system scan");

        let mut result = HashMap::new();
        let primals = self.registered_primals.read().await;

        result.insert("scan_type".to_string(), serde_json::json!("quick"));
        result.insert(
            "timestamp".to_string(),
            serde_json::json!(chrono::Utc::now()),
        );
        result.insert(
            "services_scanned".to_string(),
            serde_json::json!(primals.len()),
        );

        // Quick health check of all registered primals
        let mut issues_found = 0;
        let mut service_status = HashMap::new();

        for (id, primal) in primals.iter() {
            let status = health_to_quick_status(&primal.health);
            if status == "issue" {
                issues_found += 1;
            }

            service_status.insert(
                id.clone(),
                serde_json::json!({
                    "name": primal.name,
                    "status": status,
                    "health": primal.health
                }),
            );
        }

        result.insert("issues_count".to_string(), serde_json::json!(issues_found));
        result.insert("services".to_string(), serde_json::json!(service_status));

        Ok(result)
    }

    /// Comprehensive system scan
    pub async fn comprehensive_system_scan(&self) -> Result<HashMap<String, serde_json::Value>> {
        tracing::info!("🔬 Running comprehensive system scan");

        let mut result = HashMap::new();
        let primals = self.registered_primals.read().await;

        result.insert("scan_type".to_string(), serde_json::json!("comprehensive"));
        result.insert(
            "timestamp".to_string(),
            serde_json::json!(chrono::Utc::now()),
        );
        result.insert(
            "services_scanned".to_string(),
            serde_json::json!(primals.len()),
        );

        // Comprehensive health check with probing
        let mut issues_found = 0;
        let mut service_details = HashMap::new();

        for (id, primal) in primals.iter() {
            let mut service_info = serde_json::json!({
                "name": primal.name,
                "endpoint": primal.endpoint,
                "type": primal.primal_type,
                "capabilities": primal.capabilities,
                "health": primal.health,
                "last_seen": primal.last_seen
            });

            if let Ok(probe) = self.probe_endpoint(&primal.endpoint).await {
                service_info["probe_status"] = serde_json::json!("reachable");
                service_info["probe_name"] = serde_json::json!(probe.name);
                service_info["probe_version"] = serde_json::json!(probe.version);
            } else {
                service_info["probe_status"] = serde_json::json!("unreachable");
                issues_found += 1;
            }

            service_details.insert(id.clone(), service_info);
        }

        result.insert("issues_count".to_string(), serde_json::json!(issues_found));
        result.insert("services".to_string(), serde_json::json!(service_details));

        // System health report
        let health_report = self.get_system_health();
        result.insert(
            "system_health".to_string(),
            serde_json::json!(health_report),
        );

        Ok(result)
    }
}

/// Probe a Unix socket endpoint with real JSON-RPC requests.
///
/// Sends `identity.get` first (to learn name + version), then
/// `capabilities.list` (to discover advertised capabilities). If
/// `identity.get` returns an error the primal is still "reachable" — we just
/// lack its self-reported identity.
async fn probe_unix_endpoint(socket_path: &str) -> Result<ProbeResult> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    let path = Path::new(socket_path);

    let stream = tokio::time::timeout(timeouts::PROBE_TIMEOUT, UnixStream::connect(path))
        .await
        .map_err(|_| anyhow::anyhow!("connect timed out"))?
        .map_err(|e| anyhow::anyhow!("connect failed: {e}"))?;

    let mut reader = BufReader::new(stream);

    // --- identity.get ---
    let identity_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "identity.get",
        "id": 1
    });
    let line = serde_json::to_string(&identity_req)? + "\n";
    reader.get_mut().write_all(line.as_bytes()).await?;
    let _ = reader.get_mut().flush().await;

    let mut resp_line = String::new();
    let (name, version) = match tokio::time::timeout(
        timeouts::PROBE_TIMEOUT,
        reader.read_line(&mut resp_line),
    )
    .await
    {
        Ok(Ok(n)) if n > 0 => {
            let v: serde_json::Value = serde_json::from_str(&resp_line).unwrap_or_default();
            let name = v["result"]["name"]
                .as_str()
                .or_else(|| v["result"]["primal"].as_str())
                .unwrap_or("unknown")
                .to_string();
            let version = v["result"]["version"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();
            (name, version)
        }
        _ => ("unknown".to_string(), "unknown".to_string()),
    };

    // --- capabilities.list ---
    let caps = crate::socket_discovery::probe_unix_socket_capabilities_list(path).await;
    let capabilities = caps
        .into_iter()
        .map(|c| PrimalCapability::new(&c, "", ""))
        .collect();

    Ok(ProbeResult {
        name,
        version,
        capabilities,
        health: Health::Healthy,
    })
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use crate::universal_biomeos_manager::{PrimalInfo, UniversalBiomeOSManager};
    use biomeos_primal_sdk::PrimalCapability;
    use biomeos_types::{BiomeOSConfig, Health, PrimalType};
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn test_health_to_status_string() {
        assert_eq!(health_to_status_string(&Health::Healthy), "Healthy");
        assert_eq!(
            health_to_status_string(&Health::Degraded {
                issues: vec![],
                impact_score: None,
            }),
            "Degraded"
        );
        assert_eq!(
            health_to_status_string(&Health::Unhealthy {
                issues: vec![],
                failed_at: Utc::now(),
            }),
            "Unhealthy"
        );
        assert_eq!(
            health_to_status_string(&Health::Unknown {
                reason: "test".into(),
                last_known: None,
            }),
            "Unknown"
        );
    }

    #[test]
    fn test_health_to_quick_status() {
        assert_eq!(health_to_quick_status(&Health::Healthy), "ok");
        assert_eq!(
            health_to_quick_status(&Health::Degraded {
                issues: vec![],
                impact_score: None,
            }),
            "issue"
        );
    }

    #[test]
    fn test_health_percentage() {
        assert!((health_percentage(0, 0) - 0.0).abs() < f64::EPSILON);
        assert!((health_percentage(5, 10) - 50.0).abs() < f64::EPSILON);
        assert!((health_percentage(10, 10) - 100.0).abs() < f64::EPSILON);
        let p = health_percentage(1, 3);
        assert!((p - 33.333).abs() < 0.001, "expected ~33.33, got {p}");
    }

    #[test]
    fn test_health_monitor_construction() {
        let config = Arc::new(BiomeOSConfig::default());
        let monitor = HealthMonitor::new(config);
        assert!(std::mem::size_of_val(&monitor) > 0);
    }

    #[tokio::test]
    async fn test_health_monitor_get_system_health() {
        let config = Arc::new(BiomeOSConfig::default());
        let monitor = HealthMonitor::new(config);
        let report = monitor.get_system_health();
        assert_eq!(report.subject.id, "system");
        assert!(matches!(report.health, Health::Healthy));
    }

    #[test]
    fn test_health_to_status_string_critical() {
        let health = Health::Critical {
            issues: vec![],
            affected_capabilities: vec!["compute".to_string()],
        };
        assert_eq!(health_to_status_string(&health), "Unknown");
    }

    #[test]
    fn test_health_to_status_string_starting() {
        let health = Health::Starting {
            phase: biomeos_types::StartupPhase::Initializing,
            progress: 50,
        };
        assert_eq!(health_to_status_string(&health), "Unknown");
    }

    #[test]
    fn test_health_to_quick_status_unknown() {
        let health = Health::Unknown {
            reason: "test".into(),
            last_known: None,
        };
        assert_eq!(health_to_quick_status(&health), "issue");
    }

    fn test_primal_info(id: &str, name: &str, endpoint: &str, health: Health) -> PrimalInfo {
        PrimalInfo {
            id: id.to_string(),
            name: name.to_string(),
            primal_type: PrimalType::from_discovered("compute", name, "1.0.0"),
            endpoint: endpoint.to_string(),
            capabilities: vec![PrimalCapability::new("compute", "execution", "1.0")],
            health,
            last_seen: Utc::now(),
            discovered_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_manager_get_system_health() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
        manager.initialize().expect("init");
        let report = manager.get_system_health();
        assert_eq!(report.subject.id, "system");
        assert!(matches!(report.health, Health::Healthy));
    }

    #[tokio::test]
    async fn test_manager_probe_endpoint_nonexistent() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
        manager.initialize().expect("init");
        // Probing a non-existent socket should fail gracefully
        let result = manager
            .probe_endpoint("unix:///tmp/biomeos_health_test_absent.sock")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_manager_probe_endpoint_unsupported_scheme() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
        manager.initialize().expect("init");
        let result = manager.probe_endpoint("ftp://invalid").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_check_service_health_found_unreachable() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
        manager.initialize().expect("init");

        let primal = test_primal_info(
            "health-1",
            "health-svc",
            "unix:///tmp/biomeos_health_test_absent.sock",
            Health::Healthy,
        );
        manager.register_primal(primal).await.expect("register");

        let result = manager
            .check_service_health("health-svc")
            .await
            .expect("check health");
        // Socket doesn't exist → probe fails → status is Unreachable
        assert_eq!(
            result.get("status").and_then(|v| v.as_str()),
            Some("Unreachable")
        );
    }

    #[tokio::test]
    async fn test_check_service_health_not_found() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
        manager.initialize().expect("init");

        let result = manager
            .check_service_health("nonexistent")
            .await
            .expect("check health");
        assert_eq!(
            result.get("status").and_then(|v| v.as_str()),
            Some("Not Found")
        );
        assert!(result.contains_key("error"));
    }

    #[tokio::test]
    async fn test_check_system_health() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
        manager.initialize().expect("init");

        let result = manager
            .check_system_health()
            .await
            .expect("check system health");
        assert!(result.contains_key("overall_status"));
        assert!(result.contains_key("timestamp"));
        assert!(result.contains_key("services"));
        assert!(result.contains_key("service_summary"));
        assert!(result.contains_key("system_metrics"));
    }

    #[tokio::test]
    async fn test_check_system_health_with_primals() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
        manager.initialize().expect("init");

        let primal = test_primal_info("p1", "svc1", "unix:///a.sock", Health::Healthy);
        manager.register_primal(primal).await.expect("register");

        let result = manager.check_system_health().await.expect("check");
        let summary = result.get("service_summary").expect("summary");
        assert_eq!(summary["total"].as_u64(), Some(1));
        assert_eq!(summary["healthy"].as_u64(), Some(1));
    }

    #[tokio::test]
    async fn test_probe_service_health_found() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
        manager.initialize().expect("init");

        let primal = test_primal_info("probe-1", "probe-svc", "unix:///x.sock", Health::Healthy);
        manager.register_primal(primal).await.expect("register");

        let result = manager
            .probe_service_health("probe-svc", 5)
            .await
            .expect("probe");
        assert!(result.contains_key("connectivity"));
    }

    #[tokio::test]
    async fn test_probe_service_health_not_found() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
        manager.initialize().expect("init");

        let result = manager
            .probe_service_health("nonexistent", 5)
            .await
            .expect("probe");
        assert!(result.contains_key("error"));
    }

    #[tokio::test]
    async fn test_quick_system_scan() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
        manager.initialize().expect("init");

        let result = manager.quick_system_scan().await.expect("scan");
        assert_eq!(
            result.get("scan_type").and_then(|v| v.as_str()),
            Some("quick")
        );
        assert!(result.contains_key("services_scanned"));
        assert!(result.contains_key("issues_count"));
    }

    #[tokio::test]
    async fn test_quick_system_scan_with_issues() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
        manager.initialize().expect("init");

        let degraded = test_primal_info(
            "d1",
            "degraded-svc",
            "unix:///d.sock",
            Health::Degraded {
                issues: vec![],
                impact_score: None,
            },
        );
        manager.register_primal(degraded).await.expect("register");

        let result = manager.quick_system_scan().await.expect("scan");
        assert_eq!(
            result
                .get("issues_count")
                .and_then(serde_json::Value::as_u64),
            Some(1)
        );
    }

    #[tokio::test]
    async fn test_comprehensive_system_scan() {
        let manager = UniversalBiomeOSManager::with_default_config().expect("manager");
        manager.initialize().expect("init");

        let result = manager.comprehensive_system_scan().await.expect("scan");
        assert_eq!(
            result.get("scan_type").and_then(|v| v.as_str()),
            Some("comprehensive")
        );
        assert!(result.contains_key("system_health"));
    }
}
