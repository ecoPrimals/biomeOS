//! Configuration for predictive health analytics

use super::types::*;
use serde::{Deserialize, Serialize};

/// Configuration for health analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAnalyticsConfig {
    /// Health check interval in seconds
    pub health_check_interval: u64,
    /// Number of historical snapshots to maintain
    pub history_size: usize,
    /// Prediction horizon in seconds
    pub prediction_horizon: u64,
    /// Trend analysis window size
    pub trend_window_size: usize,
    /// Alerting thresholds
    pub alert_thresholds: AlertThresholds,
    /// Analysis algorithms to use
    pub analysis_algorithms: Vec<AnalysisAlgorithm>,
}

/// Alert threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// Critical health score threshold
    pub critical_threshold: f64,
    /// Warning health score threshold
    pub warning_threshold: f64,
    /// Performance degradation threshold
    pub performance_threshold: f64,
    /// Resource utilization threshold
    pub resource_threshold: f64,
    /// Trend degradation threshold
    pub trend_threshold: f64,
}

/// Analysis algorithms available
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnalysisAlgorithm {
    /// Simple moving average
    MovingAverage,
    /// Exponential smoothing
    ExponentialSmoothing,
    /// Linear regression
    LinearRegression,
    /// Seasonal decomposition
    SeasonalDecomposition,
    /// Anomaly detection
    AnomalyDetection,
    /// Resource correlation analysis
    ResourceCorrelation,
}

/// Alert condition configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCondition {
    /// Metric to monitor
    pub metric: String,
    /// Condition operator
    pub operator: ConditionOperator,
    /// Threshold value
    pub threshold: f64,
    /// Duration in seconds
    pub duration: u64,
    /// Alert severity
    pub severity: AlertSeverity,
}

/// Analysis preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisPreferences {
    /// Preferred algorithms
    pub algorithms: Vec<AnalysisAlgorithm>,
    /// Analysis depth
    pub depth: AnalysisDepth,
    /// Prediction accuracy requirements
    pub accuracy_requirements: AccuracyRequirements,
}

/// Accuracy requirements for predictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyRequirements {
    /// Minimum confidence level
    pub min_confidence: f64,
    /// Maximum acceptable error rate
    pub max_error_rate: f64,
    /// Prediction horizon accuracy
    pub horizon_accuracy: f64,
}

/// Monitoring session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Monitoring interval in seconds
    pub interval: u64,
    /// Session duration in seconds
    pub duration: u64,
    /// Auto-restart on completion
    pub auto_restart: bool,
}

impl Default for HealthAnalyticsConfig {
    fn default() -> Self {
        Self {
            health_check_interval: 30,
            history_size: 1000,
            prediction_horizon: 3600,
            trend_window_size: 50,
            alert_thresholds: AlertThresholds::default(),
            analysis_algorithms: vec![
                AnalysisAlgorithm::MovingAverage,
                AnalysisAlgorithm::ExponentialSmoothing,
                AnalysisAlgorithm::AnomalyDetection,
            ],
        }
    }
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            critical_threshold: 20.0,
            warning_threshold: 50.0,
            performance_threshold: 60.0,
            resource_threshold: 80.0,
            trend_threshold: 10.0,
        }
    }
}

impl Default for AnalysisPreferences {
    fn default() -> Self {
        Self {
            algorithms: vec![AnalysisAlgorithm::MovingAverage],
            depth: AnalysisDepth::Standard,
            accuracy_requirements: AccuracyRequirements::default(),
        }
    }
}

impl Default for AccuracyRequirements {
    fn default() -> Self {
        Self {
            min_confidence: 0.8,
            max_error_rate: 0.1,
            horizon_accuracy: 0.7,
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            interval: 60,
            duration: 3600,
            auto_restart: false,
        }
    }
}
