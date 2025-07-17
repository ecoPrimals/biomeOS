//! Core types and data structures for predictive health analytics

use crate::PrimalHealth;
use serde::{Deserialize, Serialize};

/// Health snapshot for a primal at a specific time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSnapshot {
    /// Timestamp of the snapshot
    pub timestamp: u64,
    /// Primal identifier
    pub primal_id: String,
    /// Health information
    pub health: PrimalHealth,
    /// Extended metrics
    pub extended_metrics: ExtendedHealthMetrics,
    /// System context
    pub system_context: SystemContext,
}

/// Extended health metrics beyond basic health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedHealthMetrics {
    /// CPU utilization percentage
    pub cpu_utilization: f64,
    /// Memory utilization percentage
    pub memory_utilization: f64,
    /// Network throughput in bytes/sec
    pub network_throughput: f64,
    /// Disk I/O rate in operations/sec
    pub disk_io_rate: f64,
    /// Request latency in milliseconds
    pub request_latency: f64,
    /// Error rate percentage
    pub error_rate: f64,
    /// Active connections count
    pub active_connections: u32,
    /// Queue depth
    pub queue_depth: u32,
}

/// System context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemContext {
    /// System load average
    pub load_average: f64,
    /// Available memory in bytes
    pub available_memory: u64,
    /// Network conditions
    pub network_conditions: NetworkConditions,
    /// Biome activity metrics
    pub biome_activity: BiomeActivity,
}

/// Network conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConditions {
    /// Network latency in milliseconds
    pub latency: f64,
    /// Packet loss percentage
    pub packet_loss: f64,
    /// Available bandwidth in bits/sec
    pub bandwidth: u64,
    /// Congestion level
    pub congestion_level: CongestionLevel,
}

/// Network congestion levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CongestionLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Biome activity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeActivity {
    /// Number of active services
    pub active_services: u32,
    /// Request volume per second
    pub request_volume: f64,
    /// Data processing rate in bytes/sec
    pub data_processing_rate: f64,
    /// User activity level
    pub user_activity: ActivityLevel,
}

/// Activity level categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityLevel {
    Low,
    Medium,
    High,
    Peak,
}

/// Trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
    Volatile,
    Unknown,
}

/// Trend components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendComponents {
    /// Long-term trend
    pub long_term: f64,
    /// Seasonal component
    pub seasonal: f64,
    /// Cyclical component
    pub cyclical: f64,
    /// Irregular component
    pub irregular: f64,
}

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    /// Anomaly timestamp
    pub timestamp: u64,
    /// Anomaly type
    pub anomaly_type: AnomalyType,
    /// Severity level
    pub severity: f64,
    /// Affected metrics
    pub affected_metrics: Vec<String>,
    /// Anomaly description
    pub description: String,
}

/// Types of anomalies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnomalyType {
    /// Sudden spike in metrics
    Spike,
    /// Sudden drop in metrics
    Drop,
    /// Gradual drift from normal
    Drift,
    /// Cyclical pattern disruption
    PatternDisruption,
    /// Resource exhaustion
    ResourceExhaustion,
    /// Performance degradation
    PerformanceDegradation,
}

/// Prediction model types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    LinearRegression,
    ExponentialSmoothing,
    ARIMA,
    NeuralNetwork,
    RandomForest,
    SupportVectorMachine,
}

/// Model accuracy metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAccuracy {
    /// Mean absolute error
    pub mae: f64,
    /// Root mean square error
    pub rmse: f64,
    /// Coefficient of determination
    pub r_squared: f64,
    /// Prediction accuracy percentage
    pub accuracy: f64,
}

/// Predicted health score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedScore {
    /// Prediction timestamp
    pub timestamp: u64,
    /// Predicted health score
    pub score: f64,
    /// Confidence level
    pub confidence: f64,
}

/// Confidence interval for predictions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    /// Lower bound of confidence interval
    pub lower_bound: f64,
    /// Upper bound of confidence interval
    pub upper_bound: f64,
    /// Confidence level percentage
    pub confidence_level: f64,
}

/// Risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Type of risk
    pub risk_type: RiskType,
    /// Probability of occurrence
    pub probability: f64,
    /// Impact severity
    pub impact: f64,
    /// Timeline when risk might occur
    pub timeline: u64,
    /// Risk description
    pub description: String,
}

/// Types of risks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskType {
    ServiceFailure,
    PerformanceDegradation,
    ResourceExhaustion,
    SecurityBreach,
    DataLoss,
    NetworkPartition,
}

/// Condition operators for alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    GreaterThan,
    LessThan,
    Equals,
    NotEquals,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// Monitoring session status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringStatus {
    Active,
    Paused,
    Stopped,
    Error(String),
}

/// Analysis depth levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisDepth {
    Basic,
    Standard,
    Advanced,
    Comprehensive,
}

/// Overall health status categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverallHealthStatus {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

/// Recommendation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    Optimize,
    ScaleUp,
    ScaleDown,
    Maintenance,
    Configuration,
    Monitoring,
    Security,
}

/// Recommendation priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Implementation effort levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Low,
    Medium,
    High,
    Complex,
}
