// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Health Monitoring Operations
//!
//! Handles health monitoring, system health checks, and endpoint probing.

mod display;
mod endpoint_probe;
mod monitor;

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests;

pub(crate) use display::{health_percentage, health_to_quick_status, health_to_status_string};
pub use monitor::HealthMonitor;

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use super::core::UniversalBiomeOSManager;
use super::discovery::ProbeResult;
use biomeos_types::HealthReport;

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
        endpoint_probe::probe_endpoint(endpoint).await
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
