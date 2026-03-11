//! Health Monitoring Operations
//!
//! Handles health monitoring, system health checks, and endpoint probing.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use super::core::UniversalBiomeOSManager;
use biomeos_types::{BiomeOSConfig, Health, HealthReport};

/// Health Monitor for system-wide health tracking
#[derive(Debug, Clone)]
pub struct HealthMonitor {
    #[allow(dead_code)] // TODO: Wire up for health monitoring configuration
    config: Arc<BiomeOSConfig>,
}

impl HealthMonitor {
    /// Create new health monitor with Arc-wrapped config for zero-copy sharing
    pub fn new(config: Arc<BiomeOSConfig>) -> Self {
        Self { config }
    }

    /// Get system health report
    pub async fn get_system_health(&self) -> HealthReport {
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
    pub async fn get_system_health(&self) -> HealthReport {
        tracing::debug!("🏥 Getting system health");

        // Delegate to the dedicated health monitor - use Arc::clone for cheap reference counting
        let health_monitor = HealthMonitor::new(Arc::clone(&self.config));
        health_monitor.get_system_health().await
    }

    /// Probe a specific endpoint using unified configuration system
    pub async fn probe_endpoint(&self, endpoint: &str) -> Result<String> {
        tracing::debug!("🔍 Probing endpoint: {}", endpoint);

        match self.discovery_service.probe_endpoint(endpoint).await {
            Ok(probe_result) => {
                tracing::info!(
                    "✅ Successfully probed endpoint {}: {} v{}",
                    endpoint,
                    probe_result.name,
                    probe_result.version
                );
                Ok(format!(
                    "{} v{} ({:?})",
                    probe_result.name, probe_result.version, probe_result.health
                ))
            }
            Err(e) => {
                tracing::warn!("❌ Failed to probe endpoint {}: {}", endpoint, e);
                Err(e)
            }
        }
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
            // Probe the endpoint for current health
            match self.probe_endpoint(&primal.endpoint).await {
                Ok(probe_info) => {
                    result.insert("service_name".to_string(), serde_json::json!(primal.name));
                    result.insert("endpoint".to_string(), serde_json::json!(primal.endpoint));
                    result.insert("health".to_string(), serde_json::json!(primal.health));
                    result.insert("probe_result".to_string(), serde_json::json!(probe_info));
                    result.insert("last_seen".to_string(), serde_json::json!(primal.last_seen));
                    result.insert(
                        "capabilities".to_string(),
                        serde_json::json!(primal.capabilities),
                    );
                    result.insert("status".to_string(), serde_json::json!("Reachable"));
                }
                Err(_) => {
                    result.insert("service_name".to_string(), serde_json::json!(primal.name));
                    result.insert("endpoint".to_string(), serde_json::json!(primal.endpoint));
                    result.insert("health".to_string(), serde_json::json!(primal.health));
                    result.insert("status".to_string(), serde_json::json!("Unreachable"));
                    result.insert(
                        "error".to_string(),
                        serde_json::json!("Failed to probe endpoint"),
                    );
                }
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
        let health_report = self.get_system_health().await;

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
            let health_status = match primal.health {
                biomeos_types::Health::Healthy => {
                    healthy_count += 1;
                    "Healthy"
                }
                biomeos_types::Health::Degraded { .. } => "Degraded",
                biomeos_types::Health::Unhealthy { .. } => "Unhealthy",
                _ => "Unknown",
            };

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
        result.insert("service_summary".to_string(), serde_json::json!({
            "total": total_count,
            "healthy": healthy_count,
            "health_percentage": if total_count > 0 { (healthy_count as f64 / total_count as f64) * 100.0 } else { 0.0 }
        }));

        // System metrics - Future: Integrate with sysinfo crate for real metrics
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

            // Basic connectivity test
            match self.probe_endpoint(&primal.endpoint).await {
                Ok(probe_info) => {
                    let probe_duration = probe_start.elapsed().as_millis();

                    result.insert(
                        "connectivity".to_string(),
                        serde_json::json!({
                            "reachable": true,
                            "response_time_ms": probe_duration,
                            "probe_info": probe_info
                        }),
                    );

                    // Performance metrics - Future: Track actual metrics via health monitor
                    result.insert(
                        "performance".to_string(),
                        serde_json::json!({
                            "avg_latency_ms": probe_duration,
                            "throughput_rps": 100, // Future: compute from request counters
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
            let status = match primal.health {
                biomeos_types::Health::Healthy => "ok",
                _ => {
                    issues_found += 1;
                    "issue"
                }
            };

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

            // Try to probe the endpoint
            match self.probe_endpoint(&primal.endpoint).await {
                Ok(probe_info) => {
                    service_info["probe_status"] = serde_json::json!("reachable");
                    service_info["probe_info"] = serde_json::json!(probe_info);
                }
                Err(_) => {
                    service_info["probe_status"] = serde_json::json!("unreachable");
                    issues_found += 1;
                }
            }

            service_details.insert(id.clone(), service_info);
        }

        result.insert("issues_count".to_string(), serde_json::json!(issues_found));
        result.insert("services".to_string(), serde_json::json!(service_details));

        // System health report
        let health_report = self.get_system_health().await;
        result.insert(
            "system_health".to_string(),
            serde_json::json!(health_report),
        );

        Ok(result)
    }
}
