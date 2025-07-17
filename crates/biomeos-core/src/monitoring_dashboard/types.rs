//! Core types for the monitoring dashboard

use crate::{
    federation_optimization::PerformanceMetrics, federation_optimization::ResourceUtilization,
    HealthStatus, PrimalHealth, PrimalType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Dashboard metrics state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetricsState {
    /// Current timestamp
    pub timestamp: u64,
    /// Cross-primal metrics
    pub primal_metrics: HashMap<String, PrimalMetrics>,
    /// Global federation metrics
    pub federation_metrics: FederationMetrics,
    /// Health summary
    pub health_summary: HealthSummary,
    /// Performance overview
    pub performance_overview: PerformanceOverview,
    /// Resource utilization
    pub resource_utilization: ResourceUtilizationOverview,
    /// Cost analysis
    pub cost_analysis: CostAnalysis,
    /// Alerts summary
    pub alerts_summary: AlertsSummary,
}

/// Primal-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalMetrics {
    /// Primal identifier
    pub primal_id: String,
    /// Primal type
    pub primal_type: PrimalType,
    /// Current health status
    pub health: PrimalHealth,
    /// Performance metrics
    pub performance: PerformanceMetrics,
    /// Resource utilization
    pub resources: ResourceUtilization,
    /// Service-specific metrics
    pub services: HashMap<String, ServiceMetrics>,
    /// Custom metrics
    pub custom_metrics: HashMap<String, serde_json::Value>,
    /// Trend indicators
    pub trends: TrendIndicators,
}

/// Service-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMetrics {
    /// Service identifier
    pub service_id: String,
    /// Service health
    pub health: HealthStatus,
    /// Request metrics
    pub requests: RequestMetrics,
    /// Response time metrics
    pub response_times: ResponseTimeMetrics,
    /// Error metrics
    pub errors: ErrorMetrics,
    /// Dependency metrics
    pub dependencies: DependencyMetrics,
}

/// Request metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    /// Total requests
    pub total_requests: u64,
    /// Requests per second
    pub requests_per_second: f64,
    /// Request rate change
    pub rate_change: f64,
    /// Peak request rate
    pub peak_rate: f64,
    /// Request types distribution
    pub request_types: HashMap<String, u64>,
}

/// Response time metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeMetrics {
    /// Average response time
    pub avg_response_time: f64,
    /// Median response time
    pub median_response_time: f64,
    /// 90th percentile
    pub p90_response_time: f64,
    /// 95th percentile
    pub p95_response_time: f64,
    /// 99th percentile
    pub p99_response_time: f64,
    /// Response time trend
    pub trend: TrendDirection,
}

/// Error metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    /// Total errors
    pub total_errors: u64,
    /// Error rate
    pub error_rate: f64,
    /// Error types distribution
    pub error_types: HashMap<String, u64>,
    /// Recent errors
    pub recent_errors: Vec<ErrorEvent>,
}

/// Error event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    /// Error timestamp
    pub timestamp: u64,
    /// Error type
    pub error_type: String,
    /// Error message
    pub message: String,
    /// Error severity
    pub severity: ErrorSeverity,
    /// Error context
    pub context: HashMap<String, serde_json::Value>,
}

/// Error severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

/// Dependency metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyMetrics {
    /// Dependency health
    pub dependencies: HashMap<String, DependencyHealth>,
    /// Dependency response times
    pub response_times: HashMap<String, f64>,
    /// Dependency availability
    pub availability: HashMap<String, f64>,
}

/// Dependency health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyHealth {
    /// Dependency name
    pub name: String,
    /// Health status
    pub status: HealthStatus,
    /// Last check time
    pub last_check: u64,
    /// Response time
    pub response_time: f64,
    /// Availability percentage
    pub availability: f64,
}

/// Trend indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendIndicators {
    /// Performance trend
    pub performance_trend: TrendDirection,
    /// Resource trend
    pub resource_trend: TrendDirection,
    /// Error trend
    pub error_trend: TrendDirection,
    /// Health trend
    pub health_trend: TrendDirection,
}

/// Trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Improving
    Improving,
    /// Stable
    Stable,
    /// Degrading
    Degrading,
    /// Unknown
    Unknown,
}

/// Federation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationMetrics {
    /// Total primals
    pub total_primals: u32,
    /// Active primals
    pub active_primals: u32,
    /// Federation health score
    pub health_score: f64,
    /// Cross-primal communication metrics
    pub communication_metrics: CommunicationMetrics,
    /// Resource sharing metrics
    pub resource_sharing: ResourceSharingMetrics,
    /// Federation efficiency
    pub efficiency: FederationEfficiency,
}

/// Communication metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationMetrics {
    /// Total messages
    pub total_messages: u64,
    /// Messages per second
    pub messages_per_second: f64,
    /// Average message latency
    pub avg_latency: f64,
    /// Message types distribution
    pub message_types: HashMap<String, u64>,
    /// Communication errors
    pub errors: u64,
}

/// Resource sharing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSharingMetrics {
    /// Shared resources
    pub shared_resources: HashMap<String, ResourceSharingInfo>,
    /// Resource utilization efficiency
    pub utilization_efficiency: f64,
    /// Resource conflicts
    pub conflicts: u64,
}

/// Resource sharing info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSharingInfo {
    /// Resource type
    pub resource_type: String,
    /// Sharing primals
    pub sharing_primals: Vec<String>,
    /// Utilization percentage
    pub utilization: f64,
    /// Conflicts count
    pub conflicts: u64,
}

/// Federation efficiency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationEfficiency {
    /// Overall efficiency score
    pub overall_score: f64,
    /// Resource efficiency
    pub resource_efficiency: f64,
    /// Communication efficiency
    pub communication_efficiency: f64,
    /// Coordination efficiency
    pub coordination_efficiency: f64,
}

/// Health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSummary {
    /// Overall health score
    pub overall_score: f64,
    /// Health by primal
    pub health_by_primal: HashMap<String, f64>,
    /// Health trends
    pub health_trends: HealthTrends,
    /// Critical issues
    pub critical_issues: Vec<HealthIssue>,
    /// Health recommendations
    pub recommendations: Vec<HealthRecommendation>,
}

/// Health trends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthTrends {
    /// Short-term trend (last hour)
    pub short_term: TrendDirection,
    /// Medium-term trend (last day)
    pub medium_term: TrendDirection,
    /// Long-term trend (last week)
    pub long_term: TrendDirection,
}

/// Health issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    /// Issue ID
    pub id: String,
    /// Issue severity
    pub severity: ErrorSeverity,
    /// Issue description
    pub description: String,
    /// Affected primal
    pub affected_primal: String,
    /// Issue timestamp
    pub timestamp: u64,
    /// Suggested actions
    pub suggested_actions: Vec<String>,
}

/// Health recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthRecommendation {
    /// Recommendation ID
    pub id: String,
    /// Recommendation type
    pub recommendation_type: String,
    /// Recommendation description
    pub description: String,
    /// Priority level
    pub priority: u32,
    /// Estimated impact
    pub estimated_impact: f64,
    /// Implementation effort
    pub implementation_effort: String,
}

/// Performance overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceOverview {
    /// Overall performance score
    pub overall_score: f64,
    /// Performance by primal
    pub performance_by_primal: HashMap<String, f64>,
    /// Performance trends
    pub performance_trends: PerformanceTrends,
    /// Bottlenecks
    pub bottlenecks: Vec<PerformanceBottleneck>,
    /// Optimization opportunities
    pub optimizations: Vec<OptimizationOpportunity>,
}

/// Performance trends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrends {
    /// Throughput trend
    pub throughput_trend: TrendDirection,
    /// Latency trend
    pub latency_trend: TrendDirection,
    /// Resource utilization trend
    pub resource_trend: TrendDirection,
    /// Error rate trend
    pub error_rate_trend: TrendDirection,
}

/// Performance bottleneck
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBottleneck {
    /// Bottleneck ID
    pub id: String,
    /// Bottleneck type
    pub bottleneck_type: String,
    /// Affected component
    pub affected_component: String,
    /// Severity level
    pub severity: f64,
    /// Impact description
    pub impact: String,
    /// Suggested solutions
    pub solutions: Vec<String>,
}

/// Optimization opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationOpportunity {
    /// Opportunity ID
    pub id: String,
    /// Opportunity type
    pub opportunity_type: String,
    /// Description
    pub description: String,
    /// Potential improvement
    pub potential_improvement: f64,
    /// Implementation complexity
    pub complexity: String,
    /// Priority score
    pub priority: f64,
}

/// Resource utilization overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilizationOverview {
    /// CPU utilization
    pub cpu: ResourceUtilizationDetail,
    /// Memory utilization
    pub memory: ResourceUtilizationDetail,
    /// Storage utilization
    pub storage: ResourceUtilizationDetail,
    /// Network utilization
    pub network: ResourceUtilizationDetail,
    /// Overall efficiency score
    pub efficiency_score: f64,
}

/// Resource utilization detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilizationDetail {
    /// Current utilization percentage
    pub current: f64,
    /// Average utilization
    pub average: f64,
    /// Peak utilization
    pub peak: f64,
    /// Utilization trend
    pub trend: TrendDirection,
    /// Optimization recommendations
    pub recommendations: Vec<String>,
}

/// Cost analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAnalysis {
    /// Total cost
    pub total_cost: f64,
    /// Cost per hour
    pub cost_per_hour: f64,
    /// Cost by primal
    pub cost_by_primal: HashMap<String, f64>,
    /// Cost by resource type
    pub cost_by_resource: HashMap<String, f64>,
    /// Cost trends
    pub cost_trends: CostTrends,
    /// Optimization opportunities
    pub optimization_opportunities: Vec<CostOptimization>,
}

/// Cost trends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostTrends {
    /// Daily cost trend
    pub daily_trend: TrendDirection,
    /// Weekly cost trend
    pub weekly_trend: TrendDirection,
    /// Monthly cost trend
    pub monthly_trend: TrendDirection,
    /// Cost efficiency trend
    pub efficiency_trend: TrendDirection,
}

/// Cost optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostOptimization {
    /// Optimization ID
    pub id: String,
    /// Optimization type
    pub optimization_type: String,
    /// Description
    pub description: String,
    /// Potential savings
    pub potential_savings: f64,
    /// Implementation effort
    pub effort: String,
    /// Risk level
    pub risk: String,
}

/// Alerts summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertsSummary {
    /// Total active alerts
    pub total_alerts: u32,
    /// Alerts by severity
    pub alerts_by_severity: HashMap<String, u32>,
    /// Recent alerts
    pub recent_alerts: Vec<String>,
    /// Alert trends
    pub alert_trends: AlertTrends,
}

/// Alert trends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertTrends {
    /// Alert frequency trend
    pub frequency_trend: TrendDirection,
    /// Alert severity trend
    pub severity_trend: TrendDirection,
    /// Alert resolution time trend
    pub resolution_time_trend: TrendDirection,
}

/// Dashboard subscriber
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSubscriber {
    /// Subscriber ID
    pub id: String,
    /// Subscriber name
    pub name: String,
    /// Subscription filters
    pub filters: Vec<SubscriptionFilter>,
    /// Update frequency
    pub update_frequency: u64,
    /// Last update time
    pub last_update: u64,
}

/// Subscription filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionFilter {
    /// Filter type
    pub filter_type: String,
    /// Filter value
    pub value: String,
    /// Filter operator
    pub operator: String,
}

/// Dashboard event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardEvent {
    /// Event ID
    pub id: String,
    /// Event type
    pub event_type: DashboardEventType,
    /// Event timestamp
    pub timestamp: u64,
    /// Event data
    pub data: serde_json::Value,
}

/// Dashboard event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DashboardEventType {
    /// Metrics updated
    MetricsUpdated,
    /// Alert triggered
    AlertTriggered,
    /// Alert resolved
    AlertResolved,
    /// Health status changed
    HealthStatusChanged,
    /// Performance threshold exceeded
    PerformanceThresholdExceeded,
    /// Resource utilization changed
    ResourceUtilizationChanged,
}

impl Default for DashboardMetricsState {
    fn default() -> Self {
        Self {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            primal_metrics: HashMap::new(),
            federation_metrics: FederationMetrics {
                total_primals: 0,
                active_primals: 0,
                health_score: 0.0,
                communication_metrics: CommunicationMetrics {
                    total_messages: 0,
                    messages_per_second: 0.0,
                    avg_latency: 0.0,
                    message_types: HashMap::new(),
                    errors: 0,
                },
                resource_sharing: ResourceSharingMetrics {
                    shared_resources: HashMap::new(),
                    utilization_efficiency: 0.0,
                    conflicts: 0,
                },
                efficiency: FederationEfficiency {
                    overall_score: 0.0,
                    resource_efficiency: 0.0,
                    communication_efficiency: 0.0,
                    coordination_efficiency: 0.0,
                },
            },
            health_summary: HealthSummary {
                overall_score: 0.0,
                health_by_primal: HashMap::new(),
                health_trends: HealthTrends {
                    short_term: TrendDirection::Stable,
                    medium_term: TrendDirection::Stable,
                    long_term: TrendDirection::Stable,
                },
                critical_issues: Vec::new(),
                recommendations: Vec::new(),
            },
            performance_overview: PerformanceOverview {
                overall_score: 0.0,
                performance_by_primal: HashMap::new(),
                performance_trends: PerformanceTrends {
                    throughput_trend: TrendDirection::Stable,
                    latency_trend: TrendDirection::Stable,
                    resource_trend: TrendDirection::Stable,
                    error_rate_trend: TrendDirection::Stable,
                },
                bottlenecks: Vec::new(),
                optimizations: Vec::new(),
            },
            resource_utilization: ResourceUtilizationOverview {
                cpu: ResourceUtilizationDetail {
                    current: 0.0,
                    average: 0.0,
                    peak: 0.0,
                    trend: TrendDirection::Stable,
                    recommendations: Vec::new(),
                },
                memory: ResourceUtilizationDetail {
                    current: 0.0,
                    average: 0.0,
                    peak: 0.0,
                    trend: TrendDirection::Stable,
                    recommendations: Vec::new(),
                },
                storage: ResourceUtilizationDetail {
                    current: 0.0,
                    average: 0.0,
                    peak: 0.0,
                    trend: TrendDirection::Stable,
                    recommendations: Vec::new(),
                },
                network: ResourceUtilizationDetail {
                    current: 0.0,
                    average: 0.0,
                    peak: 0.0,
                    trend: TrendDirection::Stable,
                    recommendations: Vec::new(),
                },
                efficiency_score: 0.0,
            },
            cost_analysis: CostAnalysis {
                total_cost: 0.0,
                cost_per_hour: 0.0,
                cost_by_primal: HashMap::new(),
                cost_by_resource: HashMap::new(),
                cost_trends: CostTrends {
                    daily_trend: TrendDirection::Stable,
                    weekly_trend: TrendDirection::Stable,
                    monthly_trend: TrendDirection::Stable,
                    efficiency_trend: TrendDirection::Stable,
                },
                optimization_opportunities: Vec::new(),
            },
            alerts_summary: AlertsSummary {
                total_alerts: 0,
                alerts_by_severity: HashMap::new(),
                recent_alerts: Vec::new(),
                alert_trends: AlertTrends {
                    frequency_trend: TrendDirection::Stable,
                    severity_trend: TrendDirection::Stable,
                    resolution_time_trend: TrendDirection::Stable,
                },
            },
        }
    }
}
