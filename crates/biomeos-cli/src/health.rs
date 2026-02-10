//! Health monitoring utilities for CLI
//!
//! Specialized health functions: analysis, scoring, trends, conditions.

use anyhow::Result;
use biomeos_core::UniversalBiomeOSManager;
use biomeos_types::Health;

/// Extended health monitoring utilities
pub struct HealthUtils;

/// System health representation for CLI
#[derive(Debug, Clone)]
pub struct SystemHealth {
    /// Overall health status
    pub overall_status: Health,
    /// CPU utilization percentage (0.0–100.0)
    pub cpu_usage: f64,
    /// Memory utilization percentage (0.0–100.0)
    pub memory_usage: f64,
    /// Disk utilization percentage (0.0–100.0)
    pub disk_usage: f64,
    /// Network connectivity status
    pub network_status: String,
}

impl HealthUtils {
    /// Get comprehensive health report
    pub async fn comprehensive_health_report(
        manager: &UniversalBiomeOSManager,
    ) -> Result<CLIHealthReport> {
        let health_report = manager.get_system_health().await;

        // Convert unified health report to CLI format
        let system_health = SystemHealth {
            overall_status: health_report.health.clone(),
            cpu_usage: health_report
                .metrics
                .resources
                .as_ref()
                .and_then(|r| r.cpu_usage)
                .map(|u| u * 100.0)
                .unwrap_or(0.0),
            memory_usage: health_report
                .metrics
                .resources
                .as_ref()
                .and_then(|r| r.memory_usage)
                .map(|u| u * 100.0)
                .unwrap_or(0.0),
            disk_usage: health_report
                .metrics
                .resources
                .as_ref()
                .and_then(|r| r.disk_usage)
                .map(|u| u * 100.0)
                .unwrap_or(0.0),
            network_status: "OK".to_string(),
        };

        // Collect service health from discovered services
        let mut services = Vec::new();

        // Discover available services
        if let Ok(discovered_endpoints) = manager.discover_network_scan().await {
            for endpoint in discovered_endpoints {
                // Probe each service for detailed health info
                let start_time = std::time::Instant::now();
                match manager.probe_endpoint(&endpoint).await {
                    Ok(probe_result) => {
                        let response_time = start_time.elapsed().as_millis() as u64;
                        services.push(ServiceHealth {
                            name: probe_result, // probe_result is the service name/description
                            endpoint: endpoint.clone(),
                            status: biomeos_types::Health::Healthy,
                            response_time_ms: response_time,
                        });
                    }
                    Err(_) => {
                        services.push(ServiceHealth {
                            name: "Unknown Service".to_string(),
                            endpoint: endpoint.clone(),
                            status: biomeos_types::Health::Unhealthy {
                                issues: vec![],
                                failed_at: chrono::Utc::now(),
                            },
                            response_time_ms: 0,
                        });
                    }
                }
            }
        };

        Ok(CLIHealthReport {
            system: system_health,
            services,
            timestamp: chrono::Utc::now(),
        })
    }
}

/// CLI-formatted health report
#[derive(Debug)]
pub struct CLIHealthReport {
    /// System-level health information
    pub system: SystemHealth,
    /// Health of individual services
    pub services: Vec<ServiceHealth>,
    /// When this report was generated
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Health information for a single service
#[derive(Debug)]
pub struct ServiceHealth {
    /// Service display name
    pub name: String,
    /// Service endpoint URL
    pub endpoint: String,
    /// Service health status
    pub status: biomeos_primal_sdk::Health,
    /// Response time in milliseconds
    pub response_time_ms: u64,
}

impl HealthUtils {
    /// Perform health check with detailed analysis
    pub async fn analyze_system_health(
        manager: &UniversalBiomeOSManager,
    ) -> Result<HealthAnalysis> {
        let health_report = manager.get_system_health().await;
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        // Convert to CLI SystemHealth format
        let system_health = SystemHealth {
            overall_status: health_report.health.clone(),
            cpu_usage: health_report
                .metrics
                .resources
                .as_ref()
                .and_then(|r| r.cpu_usage)
                .map(|u| u * 100.0)
                .unwrap_or(0.0),
            memory_usage: health_report
                .metrics
                .resources
                .as_ref()
                .and_then(|r| r.memory_usage)
                .map(|u| u * 100.0)
                .unwrap_or(0.0),
            disk_usage: health_report
                .metrics
                .resources
                .as_ref()
                .and_then(|r| r.disk_usage)
                .map(|u| u * 100.0)
                .unwrap_or(0.0),
            network_status: "OK".to_string(),
        };

        // Analyze memory usage
        if system_health.memory_usage > 90.0 {
            issues.push("Critical: Memory usage above 90%".to_string());
            recommendations.push("Consider restarting services or upgrading memory".to_string());
        } else if system_health.memory_usage > 75.0 {
            issues.push("Warning: Memory usage above 75%".to_string());
            recommendations.push("Monitor memory usage trends".to_string());
        }

        // Analyze CPU usage
        if system_health.cpu_usage > 95.0 {
            issues.push("Critical: CPU usage above 95%".to_string());
            recommendations.push("Check for runaway processes or reduce load".to_string());
        } else if system_health.cpu_usage > 80.0 {
            issues.push("Warning: CPU usage above 80%".to_string());
        }

        // Analyze disk usage
        if system_health.disk_usage > 95.0 {
            issues.push("Critical: Disk usage above 95%".to_string());
            recommendations.push("Clean up disk space immediately".to_string());
        } else if system_health.disk_usage > 85.0 {
            issues.push("Warning: Disk usage above 85%".to_string());
            recommendations.push("Plan disk cleanup or expansion".to_string());
        }

        let overall_score = Self::calculate_health_score(&system_health);

        Ok(HealthAnalysis {
            overall_score,
            issues,
            recommendations,
            system_health,
        })
    }

    /// Calculate overall health score (0-100)
    fn calculate_health_score(health: &SystemHealth) -> f64 {
        let mut score: f64 = 100.0;

        // Deduct for high resource usage
        if health.memory_usage > 75.0 {
            score -= (health.memory_usage - 75.0) * 2.0;
        }

        if health.cpu_usage > 75.0 {
            score -= (health.cpu_usage - 75.0) * 1.5;
        }

        if health.disk_usage > 85.0 {
            score -= (health.disk_usage - 85.0) * 3.0;
        }

        score.clamp(0.0, 100.0)
    }

    /// Get health trend analysis
    pub async fn health_trend_analysis(_manager: &UniversalBiomeOSManager) -> Result<HealthTrend> {
        // In production, this would analyze historical health data
        Ok(HealthTrend {
            trend: "Stable".to_string(),
            confidence: 85.0,
            prediction: "System health expected to remain stable".to_string(),
        })
    }

    /// Check for specific health conditions
    pub async fn check_health_conditions(
        manager: &UniversalBiomeOSManager,
    ) -> Result<Vec<HealthCondition>> {
        let health_report = manager.get_system_health().await;
        let mut conditions = Vec::new();

        // Check uptime from availability metrics
        let uptime_hours = if let Some(availability) = &health_report.metrics.availability {
            (availability.uptime_seconds / 3600) as i64
        } else {
            0
        };
        if uptime_hours > 24 * 30 {
            // 30 days
            conditions.push(HealthCondition {
                condition: "LongUptime".to_string(),
                severity: "Info".to_string(),
                description: format!("System uptime: {} hours", uptime_hours),
                action: "Consider scheduled reboot for updates".to_string(),
            });
        }

        // Check resource patterns
        let total_usage = if let Some(resources) = &health_report.metrics.resources {
            (resources.memory_usage.unwrap_or(0.0)
                + resources.cpu_usage.unwrap_or(0.0)
                + resources.disk_usage.unwrap_or(0.0))
                * 100.0 // Convert to percentage
        } else {
            0.0
        };

        if total_usage > 240.0 {
            // 80% average
            conditions.push(HealthCondition {
                condition: "HighResourceUsage".to_string(),
                severity: "Warning".to_string(),
                description: "Overall resource usage is high".to_string(),
                action: "Review running services and optimize".to_string(),
            });
        }

        Ok(conditions)
    }
}

/// Health analysis results
#[derive(Debug)]
pub struct HealthAnalysis {
    /// Overall health score (0.0–100.0)
    pub overall_score: f64,
    /// Detected health issues
    pub issues: Vec<String>,
    /// Suggested recommendations
    pub recommendations: Vec<String>,
    /// Current system health snapshot
    pub system_health: SystemHealth,
}

/// Health trend analysis
#[derive(Debug)]
pub struct HealthTrend {
    /// Trend direction (e.g., "Stable", "Improving", "Declining")
    pub trend: String,
    /// Confidence in the trend prediction (0.0–100.0)
    pub confidence: f64,
    /// Human-readable prediction
    pub prediction: String,
}

/// A specific health condition detected in the system
#[derive(Debug)]
pub struct HealthCondition {
    /// Condition identifier
    pub condition: String,
    /// Severity level
    pub severity: String,
    /// Description of the condition
    pub description: String,
    /// Recommended action to address the condition
    pub action: String,
}
