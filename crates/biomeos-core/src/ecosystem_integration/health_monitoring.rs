//! # Ecosystem Health Monitoring
//!
//! Comprehensive health monitoring and coordination for the ecosystem.
//! This module provides advanced health monitoring capabilities including
//! service health tracking, performance metrics, and trend analysis.

use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

use super::service_registry::EcosystemServiceRegistry;
use super::types::*;
use crate::health::{ComponentType, HealthIssue, HealthMetrics, HealthMonitor, IssueSeverity};
use crate::{BiomeError, BiomeResult, HealthStatus};

/// Ecosystem health monitoring and coordination
pub struct EcosystemHealthCoordinator {
    /// Comprehensive health monitor
    health_monitor: Arc<HealthMonitor>,
    /// Service registry
    service_registry: Arc<EcosystemServiceRegistry>,
    /// Health check configuration
    config: EcosystemHealthConfig,
}

impl EcosystemHealthCoordinator {
    /// Create new health coordinator with default configuration
    pub fn new(service_registry: Arc<EcosystemServiceRegistry>) -> Self {
        Self::with_config(service_registry, EcosystemHealthConfig::default())
    }

    /// Create new health coordinator with custom configuration
    pub fn with_config(
        service_registry: Arc<EcosystemServiceRegistry>,
        config: EcosystemHealthConfig,
    ) -> Self {
        let health_monitor = Arc::new(HealthMonitor::new());

        Self {
            health_monitor,
            service_registry,
            config,
        }
    }

    /// Initialize health monitoring for all registered services
    pub async fn initialize(&self) -> BiomeResult<()> {
        info!("Initializing ecosystem health monitoring");

        // Register all known services with the health monitor
        let services = self.service_registry.list_services().await?;

        for service in services {
            self.health_monitor
                .register_component(
                    service.service_id.clone(),
                    ComponentType::Primal,
                    Some(self.config.service_check_interval),
                )
                .await?;
        }

        // Start health monitoring background task
        self.health_monitor.start_monitoring().await?;

        // Start ecosystem health monitoring task
        self.start_ecosystem_monitoring().await?;

        Ok(())
    }

    /// Start ecosystem-wide health monitoring
    async fn start_ecosystem_monitoring(&self) -> BiomeResult<()> {
        let health_monitor = self.health_monitor.clone();
        let service_registry = self.service_registry.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(Duration::from_secs(config.service_check_interval));

            loop {
                interval.tick().await;

                // Get all registered services
                if let Ok(services) = service_registry.list_services().await {
                    for service in services {
                        // Perform health check for each service
                        if let Err(e) =
                            Self::perform_service_health_check(&health_monitor, &service, &config)
                                .await
                        {
                            warn!(
                                "Health check failed for service {}: {}",
                                service.service_id, e
                            );
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Perform health check for a specific service
    async fn perform_service_health_check(
        health_monitor: &HealthMonitor,
        service: &EcosystemServiceRegistration,
        config: &EcosystemHealthConfig,
    ) -> BiomeResult<()> {
        let start_time = std::time::Instant::now();

        // Check service health
        let (status, metrics, issues) =
            match Self::check_service_health_detailed(service, config).await {
                Ok(result) => result,
                Err(e) => {
                    // Create health issue for the failed check
                    let issue = HealthIssue {
                        severity: IssueSeverity::Error,
                        component: service.service_id.clone(),
                        description: format!("Health check failed: {}", e),
                        first_detected: Utc::now(),
                        resolution: Some(
                            "Check service availability and network connectivity".to_string(),
                        ),
                    };

                    (HealthStatus::Failed, HealthMetrics::default(), vec![issue])
                }
            };

        // Update health metrics with response time
        let response_time = start_time.elapsed().as_millis() as f64;
        let mut updated_metrics = metrics;
        updated_metrics.avg_response_time_ms = response_time;

        // Update component health
        health_monitor
            .update_component_health(&service.service_id, status, updated_metrics, issues)
            .await?;

        Ok(())
    }

    /// Perform detailed health check for a service
    async fn check_service_health_detailed(
        service: &EcosystemServiceRegistration,
        config: &EcosystemHealthConfig,
    ) -> BiomeResult<(HealthStatus, HealthMetrics, Vec<HealthIssue>)> {
        let health_url = format!("{}/health", service.endpoints.primary);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.health_check_timeout))
            .build()
            .map_err(|e| {
                BiomeError::NetworkError(format!("Failed to create HTTP client: {}", e))
            })?;

        let response =
            client.get(&health_url).send().await.map_err(|e| {
                BiomeError::NetworkError(format!("Health check request failed: {}", e))
            })?;

        if response.status().is_success() {
            // Try to parse detailed health response
            match response.json::<serde_json::Value>().await {
                Ok(health_data) => {
                    let status = Self::parse_health_status(&health_data);
                    let metrics = Self::parse_health_metrics(&health_data);
                    let issues = Self::parse_health_issues(&health_data);

                    Ok((status, metrics, issues))
                }
                Err(_) => {
                    // Basic health check without detailed metrics
                    Ok((HealthStatus::Healthy, HealthMetrics::default(), Vec::new()))
                }
            }
        } else {
            let issue = HealthIssue {
                severity: IssueSeverity::Error,
                component: service.service_id.clone(),
                description: format!("Health endpoint returned HTTP {}", response.status()),
                first_detected: Utc::now(),
                resolution: Some("Check service health endpoint implementation".to_string()),
            };

            Ok((HealthStatus::Failed, HealthMetrics::default(), vec![issue]))
        }
    }

    /// Parse health status from JSON response
    fn parse_health_status(data: &serde_json::Value) -> HealthStatus {
        if let Some(status) = data.get("status").and_then(|s| s.as_str()) {
            match status.to_lowercase().as_str() {
                "healthy" | "up" | "ok" => HealthStatus::Healthy,
                "warning" | "degraded" => HealthStatus::Warning,
                "critical" | "error" => HealthStatus::Critical,
                "failed" | "down" => HealthStatus::Failed,
                _ => HealthStatus::Unknown,
            }
        } else {
            HealthStatus::Healthy // Default to healthy if no status field
        }
    }

    /// Parse health metrics from JSON response
    fn parse_health_metrics(data: &serde_json::Value) -> HealthMetrics {
        let metrics = data.get("metrics").unwrap_or(data);

        HealthMetrics {
            cpu_usage: metrics
                .get("cpu_usage")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            memory_usage: metrics
                .get("memory_usage")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            disk_usage: metrics
                .get("disk_usage")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            network_latency_ms: metrics
                .get("network_latency_ms")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            success_rate: metrics
                .get("success_rate")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0),
            avg_response_time_ms: metrics
                .get("avg_response_time_ms")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
        }
    }

    /// Parse health issues from JSON response
    fn parse_health_issues(data: &serde_json::Value) -> Vec<HealthIssue> {
        let mut issues = Vec::new();

        if let Some(issues_array) = data.get("issues").and_then(|v| v.as_array()) {
            for issue_data in issues_array {
                if let (Some(severity), Some(component), Some(description)) = (
                    issue_data.get("severity").and_then(|v| v.as_str()),
                    issue_data.get("component").and_then(|v| v.as_str()),
                    issue_data.get("description").and_then(|v| v.as_str()),
                ) {
                    let severity = match severity.to_lowercase().as_str() {
                        "critical" => IssueSeverity::Critical,
                        "error" => IssueSeverity::Error,
                        "warning" => IssueSeverity::Warning,
                        _ => IssueSeverity::Info,
                    };

                    issues.push(HealthIssue {
                        severity,
                        component: component.to_string(),
                        description: description.to_string(),
                        first_detected: Utc::now(),
                        resolution: issue_data
                            .get("resolution")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                    });
                }
            }
        }

        issues
    }

    /// Generate comprehensive ecosystem health status
    pub async fn get_ecosystem_health(&self) -> BiomeResult<EnhancedEcosystemHealthStatus> {
        // Get comprehensive health report
        let health_report = self.health_monitor.generate_health_report().await?;

        // Generate service health information
        let service_health = self.generate_service_health_info().await?;

        // Generate primal health information
        let primal_health = self.generate_primal_health_info(&service_health).await?;

        // Generate performance metrics
        let performance_metrics = self.generate_performance_metrics(&service_health).await?;

        // Generate health trends
        let health_trends = self.generate_health_trends(&service_health).await?;

        Ok(EnhancedEcosystemHealthStatus {
            overall_health: health_report.overall_health.clone(),
            health_report,
            service_health,
            primal_health,
            performance_metrics,
            health_trends,
        })
    }

    /// Generate service health information
    async fn generate_service_health_info(
        &self,
    ) -> BiomeResult<HashMap<String, ServiceHealthInfo>> {
        let services = self.service_registry.list_services().await?;
        let mut service_health = HashMap::new();

        for service in services {
            // Get health info from health monitor
            let health_info = ServiceHealthInfo {
                service_id: service.service_id.clone(),
                status: HealthStatus::Unknown, // This would be populated from health monitor
                metrics: HealthMetrics::default(),
                issues: Vec::new(),
                last_successful_check: None,
                failed_check_count: 0,
                response_times: Vec::new(),
            };

            service_health.insert(service.service_id, health_info);
        }

        Ok(service_health)
    }

    /// Generate primal health information
    async fn generate_primal_health_info(
        &self,
        service_health: &HashMap<String, ServiceHealthInfo>,
    ) -> BiomeResult<HashMap<String, PrimalHealthInfo>> {
        let mut primal_health = HashMap::new();

        // Group services by primal type
        let services = self.service_registry.list_services().await?;
        let mut primal_services: HashMap<String, Vec<&EcosystemServiceRegistration>> =
            HashMap::new();

        for service in &services {
            primal_services
                .entry(service.primal_type.clone())
                .or_default()
                .push(service);
        }

        // Calculate health for each primal type
        for (primal_type, primal_service_list) in primal_services {
            let mut healthy_count = 0;
            let total_count = primal_service_list.len();

            for service in primal_service_list {
                if let Some(health_info) = service_health.get(&service.service_id) {
                    if matches!(health_info.status, HealthStatus::Healthy) {
                        healthy_count += 1;
                    }
                }
            }

            let health = if healthy_count == total_count {
                HealthStatus::Healthy
            } else if healthy_count >= (total_count * 2 / 3) {
                HealthStatus::Warning
            } else {
                HealthStatus::Critical
            };

            primal_health.insert(
                primal_type,
                PrimalHealthInfo {
                    health,
                    healthy_count,
                    total_count,
                },
            );
        }

        Ok(primal_health)
    }

    /// Generate performance metrics
    async fn generate_performance_metrics(
        &self,
        service_health: &HashMap<String, ServiceHealthInfo>,
    ) -> BiomeResult<EcosystemPerformanceMetrics> {
        let mut total_response_time = 0.0;
        let mut total_services = 0;
        let mut total_errors = 0;

        for health_info in service_health.values() {
            total_response_time += health_info.metrics.avg_response_time_ms;
            total_services += 1;

            if matches!(
                health_info.status,
                HealthStatus::Failed | HealthStatus::Critical
            ) {
                total_errors += 1;
            }
        }

        let avg_response_time = if total_services > 0 {
            total_response_time / total_services as f64
        } else {
            0.0
        };

        let error_rate = if total_services > 0 {
            (total_errors as f64 / total_services as f64) * 100.0
        } else {
            0.0
        };

        Ok(EcosystemPerformanceMetrics {
            avg_response_time_ms: avg_response_time,
            throughput_rps: 0.0, // Would be calculated from actual metrics
            error_rate,
            resource_utilization: ResourceUtilizationSummary {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                disk_usage: 0.0,
                network_usage: 0.0,
            },
            network_metrics: NetworkPerformanceMetrics {
                avg_latency_ms: 0.0,
                packet_loss_rate: 0.0,
                bandwidth_utilization: 0.0,
                connection_success_rate: 100.0,
            },
        })
    }

    /// Generate health trends
    async fn generate_health_trends(
        &self,
        _service_health: &HashMap<String, ServiceHealthInfo>,
    ) -> BiomeResult<EcosystemHealthTrends> {
        // This would analyze historical data to determine trends
        // For now, return stable trends
        Ok(EcosystemHealthTrends {
            overall_trend: HealthTrendDirection::Stable,
            service_trends: HashMap::new(),
            performance_trend: PerformanceTrend {
                response_time_trend: HealthTrendDirection::Stable,
                throughput_trend: HealthTrendDirection::Stable,
                error_rate_trend: HealthTrendDirection::Stable,
                resource_usage_trend: HealthTrendDirection::Stable,
            },
            analysis_period: chrono::Duration::hours(1),
        })
    }
}
