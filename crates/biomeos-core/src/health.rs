//! Health monitoring for biomeOS

use crate::{BiomeError, BiomeResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Health status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub enum HealthStatus {
    /// System is healthy and functioning normally
    Healthy,
    /// System has minor issues but is still functional
    Warning,
    /// System is partially degraded
    Degraded,
    /// System is unhealthy
    Unhealthy,
    /// System has serious issues affecting functionality
    Critical,
    /// System is not responding or has failed
    Failed,
    /// Health status is unknown
    #[default]
    Unknown,
}

/// Detailed health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthInfo {
    /// Overall health status
    pub status: HealthStatus,
    /// Health check timestamp
    pub timestamp: DateTime<Utc>,
    /// Health score (0.0 = failed, 1.0 = perfect health)
    pub score: f64,
    /// Detailed health metrics
    pub metrics: HealthMetrics,
    /// Health issues if any
    pub issues: Vec<HealthIssue>,
}

/// Health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// CPU usage percentage (0.0-1.0)
    pub cpu_usage: f64,
    /// Memory usage percentage (0.0-1.0)
    pub memory_usage: f64,
    /// Disk usage percentage (0.0-1.0)
    pub disk_usage: f64,
    /// Network latency in milliseconds
    pub network_latency_ms: f64,
    /// Request success rate (0.0-1.0)
    pub success_rate: f64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
}

/// Health issue description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    /// Issue severity
    pub severity: IssueSeverity,
    /// Component affected
    pub component: String,
    /// Issue description
    pub description: String,
    /// When the issue was first detected
    pub first_detected: DateTime<Utc>,
    /// Suggested resolution
    pub resolution: Option<String>,
}

/// Issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IssueSeverity {
    /// Informational message
    Info,
    /// Warning that should be addressed
    Warning,
    /// Error affecting functionality
    Error,
    /// Critical issue requiring immediate attention
    Critical,
}

/// Comprehensive health monitoring system
pub struct HealthMonitor {
    /// Component health states
    component_health: Arc<RwLock<HashMap<String, ComponentHealthInfo>>>,
    /// Health metrics history
    metrics_history: Arc<RwLock<HashMap<String, Vec<HealthMetrics>>>>,
    /// Health alerts
    alerts: Arc<RwLock<Vec<HealthAlert>>>,
    /// Monitoring configuration
    config: HealthMonitorConfig,
}

/// Component-specific health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealthInfo {
    /// Component name
    pub name: String,
    /// Component type (service, primal, resource, etc.)
    pub component_type: ComponentType,
    /// Current health status
    pub status: HealthStatus,
    /// Latest health metrics
    pub metrics: HealthMetrics,
    /// Health issues
    pub issues: Vec<HealthIssue>,
    /// Last health check timestamp
    pub last_check: DateTime<Utc>,
    /// Health check frequency in seconds
    pub check_interval: u64,
    /// Health history (limited to recent entries)
    pub history: Vec<HealthSnapshot>,
}

/// Component type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComponentType {
    /// Primal service (Toadstool, Songbird, etc.)
    Primal,
    /// Biome instance
    Biome,
    /// System resource (CPU, memory, etc.)
    Resource,
    /// Network component
    Network,
    /// Storage component
    Storage,
    /// Security component
    Security,
}

/// Health snapshot for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSnapshot {
    /// Timestamp of the snapshot
    pub timestamp: DateTime<Utc>,
    /// Health status at the time
    pub status: HealthStatus,
    /// Health score at the time
    pub score: f64,
    /// Key metrics at the time
    pub metrics: HealthMetrics,
}

/// Health alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAlert {
    /// Alert ID
    pub id: String,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Component that triggered the alert
    pub component: String,
    /// Alert message
    pub message: String,
    /// When the alert was triggered
    pub triggered_at: DateTime<Utc>,
    /// Alert acknowledgment status
    pub acknowledged: bool,
    /// Alert resolution status
    pub resolved: bool,
    /// Resolution timestamp
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    /// Informational alert
    Info,
    /// Warning alert
    Warning,
    /// Error alert
    Error,
    /// Critical alert requiring immediate attention
    Critical,
}

/// Health monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitorConfig {
    /// Default health check interval in seconds
    pub default_check_interval: u64,
    /// Maximum history entries per component
    pub max_history_entries: usize,
    /// Health score thresholds
    pub thresholds: HealthThresholds,
    /// Alert configuration
    pub alerts: AlertConfig,
    /// Metrics collection configuration
    pub metrics: MetricsConfig,
}

/// Health score thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthThresholds {
    /// Healthy threshold (above this is healthy)
    pub healthy: f64,
    /// Warning threshold (above this is warning)
    pub warning: f64,
    /// Critical threshold (above this is critical, below is failed)
    pub critical: f64,
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Enable alerting
    pub enabled: bool,
    /// Alert cooldown period in seconds
    pub cooldown_seconds: u64,
    /// Maximum alerts per component
    pub max_alerts_per_component: usize,
    /// Auto-resolve alerts after this many seconds
    pub auto_resolve_seconds: u64,
}

/// Metrics collection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,
    /// Metrics retention period in seconds
    pub retention_seconds: u64,
    /// Metrics aggregation interval in seconds
    pub aggregation_interval: u64,
}

/// Health monitoring report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Overall system health
    pub overall_health: HealthStatus,
    /// Overall health score
    pub overall_score: f64,
    /// Component health summary
    pub components: HashMap<String, ComponentHealthSummary>,
    /// Active alerts
    pub active_alerts: Vec<HealthAlert>,
    /// Health trends
    pub trends: HealthTrends,
    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,
}

/// Component health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealthSummary {
    /// Component name
    pub name: String,
    /// Component type
    pub component_type: ComponentType,
    /// Current health status
    pub status: HealthStatus,
    /// Current health score
    pub score: f64,
    /// Issue count by severity
    pub issues: HashMap<IssueSeverity, usize>,
    /// Last check timestamp
    pub last_check: DateTime<Utc>,
}

/// Health trends analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthTrends {
    /// Overall health trend (improving, declining, stable)
    pub overall_trend: HealthTrend,
    /// Component-specific trends
    pub component_trends: HashMap<String, HealthTrend>,
    /// Trend analysis period
    pub analysis_period: chrono::Duration,
}

/// Health trend enumeration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthTrend {
    /// Health is improving
    Improving,
    /// Health is declining
    Declining,
    /// Health is stable
    Stable,
    /// Not enough data to determine trend
    Unknown,
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthMonitor {
    /// Create a new health monitor with default configuration
    pub fn new() -> Self {
        Self::with_config(HealthMonitorConfig::default())
    }

    /// Create a new health monitor with custom configuration
    pub fn with_config(config: HealthMonitorConfig) -> Self {
        Self {
            component_health: Arc::new(RwLock::new(HashMap::new())),
            metrics_history: Arc::new(RwLock::new(HashMap::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Register a component for health monitoring
    pub async fn register_component(
        &self,
        name: String,
        component_type: ComponentType,
        check_interval: Option<u64>,
    ) -> BiomeResult<()> {
        let mut components = self.component_health.write().await;

        let component_info = ComponentHealthInfo {
            name: name.clone(),
            component_type,
            status: HealthStatus::Unknown,
            metrics: HealthMetrics::default(),
            issues: Vec::new(),
            last_check: Utc::now(),
            check_interval: check_interval.unwrap_or(self.config.default_check_interval),
            history: Vec::new(),
        };

        components.insert(name, component_info);
        Ok(())
    }

    /// Update component health information
    pub async fn update_component_health(
        &self,
        name: &str,
        status: HealthStatus,
        metrics: HealthMetrics,
        issues: Vec<HealthIssue>,
    ) -> BiomeResult<()> {
        let mut components = self.component_health.write().await;

        if let Some(component) = components.get_mut(name) {
            let now = Utc::now();

            // Create health snapshot for history
            let snapshot = HealthSnapshot {
                timestamp: now,
                status: status.clone(),
                score: Self::calculate_health_score(&metrics, &issues),
                metrics: metrics.clone(),
            };

            // Update component information
            component.status = status.clone();
            component.metrics = metrics;
            component.issues = issues.clone();
            component.last_check = now;

            // Add to history (keep only recent entries)
            component.history.push(snapshot);
            if component.history.len() > self.config.max_history_entries {
                component.history.remove(0);
            }

            // Check for alerts
            self.check_for_alerts(name, &status, &issues).await?;

            Ok(())
        } else {
            Err(BiomeError::InvalidInput(format!(
                "Component '{}' not registered",
                name
            )))
        }
    }

    /// Generate comprehensive health report
    pub async fn generate_health_report(&self) -> BiomeResult<HealthReport> {
        let components = self.component_health.read().await;
        let alerts = self.alerts.read().await;

        let mut component_summaries = HashMap::new();
        let mut overall_scores = Vec::new();
        let mut overall_statuses = Vec::new();

        // Generate component summaries
        for (name, component) in components.iter() {
            let score = Self::calculate_health_score(&component.metrics, &component.issues);
            overall_scores.push(score);
            overall_statuses.push(component.status.clone());

            let mut issue_counts = HashMap::new();
            for issue in &component.issues {
                *issue_counts.entry(issue.severity.clone()).or_insert(0) += 1;
            }

            component_summaries.insert(
                name.clone(),
                ComponentHealthSummary {
                    name: name.clone(),
                    component_type: component.component_type.clone(),
                    status: component.status.clone(),
                    score,
                    issues: issue_counts,
                    last_check: component.last_check,
                },
            );
        }

        // Calculate overall health
        let overall_score = if overall_scores.is_empty() {
            0.0
        } else {
            overall_scores.iter().sum::<f64>() / overall_scores.len() as f64
        };

        let overall_health = Self::score_to_status(overall_score, &self.config.thresholds);

        // Generate trends analysis
        let trends = self.analyze_health_trends().await?;

        // Filter active alerts
        let active_alerts: Vec<HealthAlert> = alerts
            .iter()
            .filter(|alert| !alert.resolved)
            .cloned()
            .collect();

        Ok(HealthReport {
            overall_health,
            overall_score,
            components: component_summaries,
            active_alerts,
            trends,
            generated_at: Utc::now(),
        })
    }

    /// Start health monitoring background task
    pub async fn start_monitoring(&self) -> BiomeResult<()> {
        let component_health = self.component_health.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                config.default_check_interval,
            ));

            loop {
                interval.tick().await;

                let components = component_health.read().await;
                for (name, component) in components.iter() {
                    let time_since_last_check = Utc::now() - component.last_check;
                    let check_interval = chrono::Duration::seconds(component.check_interval as i64);

                    if time_since_last_check >= check_interval {
                        // Trigger health check for this component
                        // This would typically call the component's health check endpoint
                        tracing::debug!("Health check needed for component: {}", name);
                    }
                }
            }
        });

        Ok(())
    }

    /// Calculate health score based on metrics and issues
    fn calculate_health_score(metrics: &HealthMetrics, issues: &[HealthIssue]) -> f64 {
        let mut score = 1.0;

        // Factor in resource usage
        score *= (1.0 - metrics.cpu_usage).max(0.0);
        score *= (1.0 - metrics.memory_usage).max(0.0);
        score *= (1.0 - metrics.disk_usage).max(0.0);

        // Factor in performance metrics
        score *= metrics.success_rate;

        // Factor in network latency (penalty for high latency)
        if metrics.network_latency_ms > 100.0 {
            score *= (100.0 / metrics.network_latency_ms).min(1.0);
        }

        // Factor in response time (penalty for slow responses)
        if metrics.avg_response_time_ms > 1000.0 {
            score *= (1000.0 / metrics.avg_response_time_ms).min(1.0);
        }

        // Factor in issues
        for issue in issues {
            match issue.severity {
                IssueSeverity::Info => score *= 0.98,
                IssueSeverity::Warning => score *= 0.9,
                IssueSeverity::Error => score *= 0.7,
                IssueSeverity::Critical => score *= 0.3,
            }
        }

        score.max(0.0)
    }

    /// Convert health score to status
    fn score_to_status(score: f64, thresholds: &HealthThresholds) -> HealthStatus {
        if score >= thresholds.healthy {
            HealthStatus::Healthy
        } else if score >= thresholds.warning {
            HealthStatus::Warning
        } else if score >= thresholds.critical {
            HealthStatus::Critical
        } else {
            HealthStatus::Failed
        }
    }

    /// Check for alert conditions
    async fn check_for_alerts(
        &self,
        component_name: &str,
        status: &HealthStatus,
        issues: &[HealthIssue],
    ) -> BiomeResult<()> {
        if !self.config.alerts.enabled {
            return Ok(());
        }

        let mut alerts = self.alerts.write().await;

        // Check for status-based alerts
        if matches!(status, HealthStatus::Critical | HealthStatus::Failed) {
            let alert = HealthAlert {
                id: format!("{}_{}", component_name, Utc::now().timestamp()),
                severity: AlertSeverity::Critical,
                component: component_name.to_string(),
                message: format!(
                    "Component '{}' is in {} state",
                    component_name,
                    match status {
                        HealthStatus::Critical => "Critical",
                        HealthStatus::Failed => "Failed",
                        _ => "Unknown",
                    }
                ),
                triggered_at: Utc::now(),
                acknowledged: false,
                resolved: false,
                resolved_at: None,
            };

            alerts.push(alert);
        }

        // Check for issue-based alerts
        for issue in issues {
            if matches!(
                issue.severity,
                IssueSeverity::Critical | IssueSeverity::Error
            ) {
                let alert = HealthAlert {
                    id: format!(
                        "{}_{}_issue_{}",
                        component_name,
                        issue.component,
                        Utc::now().timestamp()
                    ),
                    severity: match issue.severity {
                        IssueSeverity::Critical => AlertSeverity::Critical,
                        IssueSeverity::Error => AlertSeverity::Error,
                        _ => AlertSeverity::Warning,
                    },
                    component: component_name.to_string(),
                    message: format!(
                        "Issue in component '{}': {}",
                        issue.component, issue.description
                    ),
                    triggered_at: Utc::now(),
                    acknowledged: false,
                    resolved: false,
                    resolved_at: None,
                };

                alerts.push(alert);
            }
        }

        // Limit alerts per component
        let component_alerts_count = alerts
            .iter()
            .filter(|alert| alert.component == component_name && !alert.resolved)
            .count();

        if component_alerts_count > self.config.alerts.max_alerts_per_component {
            // Remove oldest unresolved alerts for this component
            let mut component_alerts: Vec<_> = alerts
                .iter_mut()
                .filter(|alert| alert.component == component_name && !alert.resolved)
                .collect();

            component_alerts.sort_by(|a, b| a.triggered_at.cmp(&b.triggered_at));

            for alert in component_alerts
                .iter_mut()
                .take(component_alerts_count - self.config.alerts.max_alerts_per_component)
            {
                alert.resolved = true;
                alert.resolved_at = Some(Utc::now());
            }
        }

        Ok(())
    }

    /// Analyze health trends
    async fn analyze_health_trends(&self) -> BiomeResult<HealthTrends> {
        let components = self.component_health.read().await;
        let mut component_trends = HashMap::new();

        for (name, component) in components.iter() {
            let trend = if component.history.len() < 3 {
                HealthTrend::Unknown
            } else {
                let recent_scores: Vec<f64> = component
                    .history
                    .iter()
                    .rev()
                    .take(5)
                    .map(|snapshot| snapshot.score)
                    .collect();

                if recent_scores.len() >= 3 {
                    let first_avg = recent_scores[0..2].iter().sum::<f64>() / 2.0;
                    let last_avg =
                        recent_scores[recent_scores.len() - 2..].iter().sum::<f64>() / 2.0;

                    let diff = last_avg - first_avg;
                    if diff > 0.05 {
                        HealthTrend::Improving
                    } else if diff < -0.05 {
                        HealthTrend::Declining
                    } else {
                        HealthTrend::Stable
                    }
                } else {
                    HealthTrend::Unknown
                }
            };

            component_trends.insert(name.clone(), trend);
        }

        // Calculate overall trend
        let improving_count = component_trends
            .values()
            .filter(|&&trend| trend == HealthTrend::Improving)
            .count();
        let declining_count = component_trends
            .values()
            .filter(|&&trend| trend == HealthTrend::Declining)
            .count();
        let stable_count = component_trends
            .values()
            .filter(|&&trend| trend == HealthTrend::Stable)
            .count();

        let overall_trend = if improving_count > declining_count && improving_count > stable_count {
            HealthTrend::Improving
        } else if declining_count > improving_count && declining_count > stable_count {
            HealthTrend::Declining
        } else if stable_count > 0 {
            HealthTrend::Stable
        } else {
            HealthTrend::Unknown
        };

        Ok(HealthTrends {
            overall_trend,
            component_trends,
            analysis_period: chrono::Duration::minutes(30),
        })
    }

    /// Get metrics history for a component
    pub async fn get_metrics_history(&self, component_id: &str) -> Option<Vec<HealthMetrics>> {
        let history = self.metrics_history.read().await;
        history.get(component_id).cloned()
    }

    /// Add metrics to history
    pub async fn add_metrics_to_history(&self, component_id: &str, metrics: HealthMetrics) {
        let mut history = self.metrics_history.write().await;
        let component_history = history.entry(component_id.to_string()).or_insert_with(Vec::new);
        component_history.push(metrics);

        // Keep only recent entries
        if component_history.len() > self.config.max_history_entries {
            component_history.remove(0);
        }
    }

    /// Clear old metrics history
    pub async fn clear_old_metrics(&self, retention_hours: u64) {
        let mut history = self.metrics_history.write().await;
        let _cutoff = Utc::now() - chrono::Duration::hours(retention_hours as i64);
        
        for component_history in history.values_mut() {
            // This is a simplified cleanup - in a real implementation,
            // we'd store timestamps with metrics to do proper cleanup
            if component_history.len() > 50 {
                component_history.drain(0..25);
            }
        }
    }
}

impl Default for HealthInfo {
    fn default() -> Self {
        Self {
            status: HealthStatus::Unknown,
            timestamp: Utc::now(),
            score: 0.0,
            metrics: HealthMetrics::default(),
            issues: Vec::new(),
        }
    }
}

impl Default for HealthMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            disk_usage: 0.0,
            network_latency_ms: 0.0,
            success_rate: 1.0,
            avg_response_time_ms: 0.0,
        }
    }
}

impl Default for HealthMonitorConfig {
    fn default() -> Self {
        Self {
            default_check_interval: 30,
            max_history_entries: 100,
            thresholds: HealthThresholds {
                healthy: 0.8,
                warning: 0.6,
                critical: 0.3,
            },
            alerts: AlertConfig {
                enabled: true,
                cooldown_seconds: 300,
                max_alerts_per_component: 10,
                auto_resolve_seconds: 3600,
            },
            metrics: MetricsConfig {
                enabled: true,
                retention_seconds: 86400,  // 24 hours
                aggregation_interval: 300, // 5 minutes
            },
        }
    }
}
