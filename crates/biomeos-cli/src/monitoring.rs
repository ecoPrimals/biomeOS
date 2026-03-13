// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

// Monitoring utilities for CLI
// Specialized monitoring functions implemented: performance monitoring, resource monitoring, alerting

use anyhow::Result;
use biomeos_core::UniversalBiomeOSManager;
use std::time::Duration;
use tokio::time::Interval;

/// Extended monitoring utilities
pub struct MonitoringUtils;

impl MonitoringUtils {
    /// Create a monitoring interval
    pub fn create_interval(seconds: u64) -> Interval {
        tokio::time::interval(Duration::from_secs(seconds))
    }

    /// Monitor multiple endpoints
    pub async fn monitor_endpoints(
        manager: &UniversalBiomeOSManager,
        endpoints: &[String],
    ) -> Result<MonitoringSnapshot> {
        let mut services = Vec::new();

        let start_time = std::time::Instant::now();
        for endpoint in endpoints {
            match manager.probe_endpoint(endpoint).await {
                Ok(probe_result) => {
                    services.push(ServiceStatus {
                        endpoint: endpoint.clone(),
                        name: probe_result, // probe_result is already a String
                        health: biomeos_types::Health::Healthy, // Successfully probed, assume healthy
                        response_time_ms: start_time.elapsed().as_millis() as u64,
                    });
                }
                Err(_) => {
                    services.push(ServiceStatus {
                        endpoint: endpoint.clone(),
                        name: "Unknown".to_string(),
                        health: biomeos_types::Health::Unknown { 
                            reason: "Connection failed".to_string(),
                            last_known: None 
                        },
                        response_time_ms: 0,
                    });
                }
            }
        }

        let system_health = manager.get_system_health().await;

        Ok(MonitoringSnapshot {
            timestamp: chrono::Utc::now(),
            system_health,
            services,
        })
    }
}

#[derive(Debug)]
pub struct MonitoringSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub system_health: biomeos_types::HealthReport,
    pub services: Vec<ServiceStatus>,
}

#[derive(Debug)]
pub struct ServiceStatus {
    pub endpoint: String,
    pub name: String,
    pub health: biomeos_primal_sdk::Health,
    pub response_time_ms: u64,
}

impl MonitoringUtils {
    /// Monitor service performance over time
    pub async fn performance_monitoring(
        manager: &UniversalBiomeOSManager,
        endpoint: &str,
        duration_secs: u64,
        interval_secs: u64,
    ) -> Result<PerformanceReport> {
        let mut measurements = Vec::new();
        let mut interval = Self::create_interval(interval_secs);
        let start_time = std::time::Instant::now();

        while start_time.elapsed().as_secs() < duration_secs {
            interval.tick().await;

            let measurement_start = std::time::Instant::now();
            match manager.probe_endpoint(endpoint).await {
                Ok(_) => {
                    measurements.push(PerformanceMeasurement {
                        timestamp: chrono::Utc::now(),
                        response_time_ms: measurement_start.elapsed().as_millis() as u64,
                        success: true,
                    });
                }
                Err(_) => {
                    measurements.push(PerformanceMeasurement {
                        timestamp: chrono::Utc::now(),
                        response_time_ms: 0,
                        success: false,
                    });
                }
            }
        }

        Ok(PerformanceReport {
            endpoint: endpoint.to_string(),
            measurements,
            duration_secs,
        })
    }

    /// Monitor system resources continuously
    pub async fn resource_monitoring(
        manager: &UniversalBiomeOSManager,
        samples: usize,
        interval_secs: u64,
    ) -> Result<ResourceReport> {
        let mut snapshots = Vec::new();
        let mut interval = Self::create_interval(interval_secs);

        for _ in 0..samples {
            interval.tick().await;
            let health = manager.get_system_health().await;
            snapshots.push(ResourceSnapshot {
                timestamp: chrono::Utc::now(),
                cpu_percent: health.metrics.resources
                    .as_ref()
                    .and_then(|r| r.cpu_usage)
                    .map(|u| u * 100.0)
                    .unwrap_or(0.0),
                memory_percent: health.metrics.resources
                    .as_ref()
                    .and_then(|r| r.memory_usage)
                    .map(|u| u * 100.0)
                    .unwrap_or(0.0),
                disk_percent: health.metrics.resources
                    .as_ref()
                    .and_then(|r| r.disk_usage)
                    .map(|u| u * 100.0)
                    .unwrap_or(0.0),
                network_mbps: health.metrics.resources
                    .as_ref()
                    .and_then(|r| r.network_io.as_ref())
                    .map(|n| (n.bytes_in_per_sec + n.bytes_out_per_sec) / (1024.0 * 1024.0))
                    .unwrap_or(0.0),
            });
        }

        Ok(ResourceReport {
            snapshots,
            sample_count: samples,
            interval_secs,
        })
    }

    /// Create alerting conditions
    pub fn create_alert_conditions() -> Vec<AlertCondition> {
        vec![
            AlertCondition {
                name: "High CPU Usage".to_string(),
                threshold: 90.0,
                metric: "cpu_percent".to_string(),
                severity: "Critical".to_string(),
            },
            AlertCondition {
                name: "High Memory Usage".to_string(),
                threshold: 85.0,
                metric: "memory_percent".to_string(),
                severity: "Warning".to_string(),
            },
            AlertCondition {
                name: "High Disk Usage".to_string(),
                threshold: 90.0,
                metric: "disk_percent".to_string(),
                severity: "Critical".to_string(),
            },
        ]
    }

    /// Check for alert conditions
    pub fn check_alerts(
        snapshot: &MonitoringSnapshot,
        conditions: &[AlertCondition],
    ) -> Vec<Alert> {
        let mut alerts = Vec::new();

        for condition in conditions {
            let current_value = match condition.metric.as_str() {
                "cpu_percent" => snapshot.system_health.metrics.resources
                    .as_ref()
                    .and_then(|r| r.cpu_usage)
                    .map(|u| u * 100.0)
                    .unwrap_or(0.0),
                "memory_percent" => snapshot.system_health.metrics.resources
                    .as_ref()
                    .and_then(|r| r.memory_usage)
                    .map(|u| u * 100.0)
                    .unwrap_or(0.0),
                "disk_percent" => snapshot.system_health.metrics.resources
                    .as_ref()
                    .and_then(|r| r.disk_usage)
                    .map(|u| u * 100.0)
                    .unwrap_or(0.0),
                _ => 0.0,
            };

            if current_value > condition.threshold {
                alerts.push(Alert {
                    condition: condition.name.clone(),
                    current_value,
                    threshold: condition.threshold,
                    severity: condition.severity.clone(),
                    timestamp: chrono::Utc::now(),
                });
            }
        }

        alerts
    }
}

#[derive(Debug)]
pub struct PerformanceReport {
    pub endpoint: String,
    pub measurements: Vec<PerformanceMeasurement>,
    pub duration_secs: u64,
}

#[derive(Debug)]
pub struct PerformanceMeasurement {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub response_time_ms: u64,
    pub success: bool,
}

#[derive(Debug)]
pub struct ResourceReport {
    pub snapshots: Vec<ResourceSnapshot>,
    pub sample_count: usize,
    pub interval_secs: u64,
}

#[derive(Debug)]
pub struct ResourceSnapshot {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub disk_percent: f64,
    pub network_mbps: f64,
}

#[derive(Debug)]
pub struct AlertCondition {
    pub name: String,
    pub threshold: f64,
    pub metric: String,
    pub severity: String,
}

#[derive(Debug)]
pub struct Alert {
    pub condition: String,
    pub current_value: f64,
    pub threshold: f64,
    pub severity: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
