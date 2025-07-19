//! Configuration for the monitoring dashboard

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for monitoring dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Metrics collection interval in seconds
    pub metrics_interval: u64,
    /// Alert evaluation interval in seconds
    pub alert_interval: u64,
    /// Metrics retention period in seconds
    pub metrics_retention: u64,
    /// Maximum concurrent metric collectors
    pub max_collectors: usize,
    /// Dashboard update frequency in seconds
    pub update_frequency: u64,
    /// Enable real-time streaming
    pub real_time_streaming: bool,
    /// Alert configuration
    /// Notification configuration
    /// Metrics configuration
    /// Alert configurations
    pub alert_configs: Vec<AlertConfig>,
    /// Visualization preferences
    pub visualization: VisualizationConfig,
}

/// Alert configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Alert name
    pub name: String,
    /// Alert condition
    pub condition: AlertCondition,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert frequency
    pub frequency: AlertFrequency,
    /// Alert destinations
    pub destinations: Vec<AlertDestination>,
    /// Alert enabled
    pub enabled: bool,
}

/// Alert condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertCondition {
    /// Metric name
    pub metric_name: String,
    /// Comparison operator
    pub operator: ComparisonOperator,
    /// Threshold value
    pub threshold: f64,
    /// Evaluation window in seconds
    pub window: u64,
    /// Consecutive evaluations required
    pub consecutive_evaluations: u32,
}

/// Comparison operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    /// Greater than
    GreaterThan,
    /// Greater than or equal
    GreaterThanOrEqual,
    /// Less than
    LessThan,
    /// Less than or equal
    LessThanOrEqual,
    /// Equal
    Equal,
    /// Not equal
    NotEqual,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Info alert
    Info,
    /// Warning alert
    Warning,
    /// Error alert
    Error,
    /// Critical alert
    Critical,
}

/// Alert frequency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertFrequency {
    /// Once
    Once,
    /// Every evaluation
    EveryEvaluation,
    /// Hourly
    Hourly,
    /// Daily
    Daily,
    /// Custom frequency
    Custom { interval: u64 },
}

/// Alert destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertDestination {
    /// Destination type
    pub destination_type: AlertDestinationType,
    /// Destination configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Alert destination types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertDestinationType {
    /// Email notification
    Email,
    /// Slack notification
    Slack,
    /// Webhook notification
    Webhook,
    /// SMS notification
    Sms,
    /// PagerDuty notification
    PagerDuty,
    /// Dashboard notification
    Dashboard,
}

/// Visualization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    /// Chart types to generate
    pub chart_types: Vec<ChartType>,
    /// Time ranges for charts
    pub time_ranges: Vec<TimeRange>,
    /// Aggregation intervals
    pub aggregation_intervals: Vec<AggregationInterval>,
    /// Color schemes
    pub color_schemes: HashMap<String, ColorScheme>,
}

/// Chart types for visualization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ChartType {
    /// Line chart for time series
    TimeSeries,
    /// Bar chart for comparisons
    Bar,
    /// Pie chart for distributions
    Pie,
    /// Heatmap for correlation
    Heatmap,
    /// Gauge for current values
    Gauge,
    /// Histogram for distributions
    Histogram,
    /// Scatter plot for relationships
    Scatter,
}

/// Time ranges for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeRange {
    /// Last 5 minutes
    Last5Minutes,
    /// Last 15 minutes
    Last15Minutes,
    /// Last hour
    LastHour,
    /// Last 6 hours
    Last6Hours,
    /// Last 24 hours
    LastDay,
    /// Last week
    LastWeek,
    /// Last month
    LastMonth,
    /// Custom time range
    Custom { start: u64, end: u64 },
}

/// Aggregation intervals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationInterval {
    /// 1 second
    Second,
    /// 5 seconds
    FiveSeconds,
    /// 30 seconds
    ThirtySeconds,
    /// 1 minute
    Minute,
    /// 5 minutes
    FiveMinutes,
    /// 15 minutes
    FifteenMinutes,
    /// 1 hour
    Hour,
    /// 1 day
    Day,
}

/// Color scheme for visualizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    /// Primary color
    pub primary: String,
    /// Secondary color
    pub secondary: String,
    /// Success color
    pub success: String,
    /// Warning color
    pub warning: String,
    /// Error color
    pub error: String,
    /// Info color
    pub info: String,
    /// Background color
    pub background: String,
    /// Text color
    pub text: String,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            metrics_interval: 30,      // 30 seconds
            alert_interval: 60,        // 1 minute
            metrics_retention: 604800, // 7 days
            max_collectors: 100,
            update_frequency: 5, // 5 seconds
            real_time_streaming: true,
            alert_configs: vec![],
            visualization: VisualizationConfig {
                chart_types: vec![ChartType::TimeSeries, ChartType::Gauge, ChartType::Bar],
                time_ranges: vec![TimeRange::LastHour, TimeRange::LastDay, TimeRange::LastWeek],
                aggregation_intervals: vec![
                    AggregationInterval::Minute,
                    AggregationInterval::FiveMinutes,
                    AggregationInterval::Hour,
                ],
                color_schemes: HashMap::new(),
            },
        }
    }
}

impl std::fmt::Display for AlertDestinationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertDestinationType::Email => write!(f, "email"),
            AlertDestinationType::Slack => write!(f, "slack"),
            AlertDestinationType::Webhook => write!(f, "webhook"),
            AlertDestinationType::Sms => write!(f, "sms"),
            AlertDestinationType::PagerDuty => write!(f, "pagerduty"),
            AlertDestinationType::Dashboard => write!(f, "dashboard"),
        }
    }
}



#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]

pub struct NotificationConfig {
    pub enabled: bool,
    pub channels: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricsConfig {
    pub enabled: bool,
    pub collection_interval: u64,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            channels: vec![],
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            collection_interval: 300,
        }
    }
}


impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            condition: AlertCondition {
                metric_name: "health".to_string(),
                operator: ComparisonOperator::LessThan,
                threshold: 0.5,
                window: 300,
                consecutive_evaluations: 1,
            },
            severity: AlertSeverity::Warning,
            frequency: AlertFrequency::Once,
            destinations: vec![],
            enabled: true,
        }
    }
}

