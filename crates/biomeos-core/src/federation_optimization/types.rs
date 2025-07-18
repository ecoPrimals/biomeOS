//! Core Types and Structures for Federation Optimization
//!
//! This module contains all the fundamental types, enums, and structures used
//! throughout the federation optimization system.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::PrimalType;

/// Configuration for federation optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Optimization interval in seconds
    pub optimization_interval: u64,
    /// Resource utilization target (0.0-1.0)
    pub target_utilization: f64,
    /// Load balancing strategy
    pub load_balancing_strategy: LoadBalancingStrategy,
    /// Resource allocation strategy
    pub resource_allocation_strategy: ResourceAllocationStrategy,
    /// Enable predictive optimization
    pub predictive_optimization: bool,
    /// Cost optimization priority
    pub cost_optimization_priority: f64,
    /// Performance optimization priority
    pub performance_optimization_priority: f64,
    /// Sovereignty compliance requirements
    pub sovereignty_requirements: SovereigntyRequirements,
}

/// Load balancing strategies for federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// Round-robin across healthy primals
    RoundRobin,
    /// Least connections
    LeastConnections,
    /// Weighted round-robin based on capacity
    WeightedRoundRobin,
    /// Health-based routing
    HealthBased,
    /// Resource-aware routing
    ResourceAware,
    /// Latency-based routing
    LatencyBased,
    /// Cost-optimized routing
    CostOptimized,
    /// Hybrid strategy combining multiple factors
    Hybrid(HybridStrategy),
}

/// Hybrid load balancing strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridStrategy {
    /// Weight for health factor
    pub health_weight: f64,
    /// Weight for resource utilization
    pub resource_weight: f64,
    /// Weight for latency
    pub latency_weight: f64,
    /// Weight for cost
    pub cost_weight: f64,
    /// Weight for sovereignty compliance
    pub sovereignty_weight: f64,
}

/// Resource allocation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceAllocationStrategy {
    /// Fair share allocation
    FairShare,
    /// Priority-based allocation
    PriorityBased,
    /// Demand-based allocation
    DemandBased,
    /// Predictive allocation
    Predictive,
    /// Cost-optimized allocation
    CostOptimized,
    /// Performance-first allocation
    PerformanceFirst,
}

/// Sovereignty requirements for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereigntyRequirements {
    /// Require sovereign-compliant primals
    pub require_sovereign_primals: bool,
    /// Data residency requirements
    pub data_residency: Vec<String>,
    /// Prohibited vendors
    pub prohibited_vendors: Vec<String>,
    /// Preferred sovereign providers
    pub preferred_sovereign_providers: Vec<String>,
}

/// Resource capacity for a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCapacity {
    /// CPU cores available
    pub cpu_cores: f64,
    /// Memory in MB
    pub memory_mb: u64,
    /// Storage in GB
    pub storage_gb: u64,
    /// Network bandwidth in Mbps
    pub network_mbps: u64,
    /// GPU count
    pub gpu_count: u32,
    /// Custom resources
    pub custom_resources: HashMap<String, f64>,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    /// CPU utilization (0.0-1.0)
    pub cpu_usage: f64,
    /// Memory utilization (0.0-1.0)
    pub memory_usage: f64,
    /// Storage utilization (0.0-1.0)
    pub storage_usage: f64,
    /// Network utilization (0.0-1.0)
    pub network_usage: f64,
    /// GPU utilization (0.0-1.0)
    pub gpu_usage: f64,
}

/// Performance metrics for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Throughput in requests per second
    pub throughput_rps: f64,
    /// Error rate (0.0-1.0)
    pub error_rate: f64,
    /// Latency percentiles
    pub latency_percentiles: LatencyPercentiles,
}

/// Latency percentile metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyPercentiles {
    /// 50th percentile (median)
    pub p50: f64,
    /// 90th percentile
    pub p90: f64,
    /// 95th percentile
    pub p95: f64,
    /// 99th percentile
    pub p99: f64,
}

/// Cost metrics for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostMetrics {
    /// Cost per hour
    pub cost_per_hour: f64,
    /// Cost per request
    pub cost_per_request: f64,
    /// Cost per GB storage
    pub cost_per_gb_storage: f64,
    /// Cost per GB network
    pub cost_per_gb_network: f64,
    /// Total operational cost
    pub total_operational_cost: f64,
}

/// Sovereignty compliance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereigntyCompliance {
    /// Compliance level
    pub compliance_level: ComplianceLevel,
    /// Data residency compliance
    pub data_residency_compliant: bool,
    /// Vendor independence score
    pub vendor_independence_score: f64,
    /// Sovereignty certifications
    pub certifications: Vec<String>,
}

/// Compliance levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceLevel {
    /// Fully compliant
    Full,
    /// Partially compliant
    Partial,
    /// Non-compliant
    NonCompliant,
    /// Unknown compliance
    Unknown,
}

/// Resource information for a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResourceInfo {
    /// Primal identity
    pub primal_id: String,
    /// Primal type
    pub primal_type: PrimalType,
    /// Current health status
    pub health: crate::PrimalHealth,
    /// Resource capacity
    pub capacity: ResourceCapacity,
    /// Current utilization
    pub utilization: ResourceUtilization,
    /// Performance metrics
    pub performance: PerformanceMetrics,
    /// Cost metrics
    pub cost: CostMetrics,
    /// Sovereignty compliance
    pub sovereignty_compliance: SovereigntyCompliance,
}

/// Global resource metrics across federation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalResourceMetrics {
    /// Total CPU cores
    pub total_cpu_cores: f64,
    /// Total memory in MB
    pub total_memory_mb: u64,
    /// Total storage in GB
    pub total_storage_gb: u64,
    /// Total network bandwidth in Mbps
    pub total_network_mbps: u64,
    /// Average utilization across federation
    pub avg_utilization: ResourceUtilization,
    /// Resource efficiency score
    pub efficiency_score: f64,
    /// Cost efficiency score
    pub cost_efficiency_score: f64,
}

/// Allocated resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocatedResources {
    /// CPU cores allocated
    pub cpu_cores: f64,
    /// Memory allocated in MB
    pub memory_mb: u64,
    /// Storage allocated in GB
    pub storage_gb: u64,
    /// Network bandwidth allocated in Mbps
    pub network_mbps: u64,
    /// Custom resource allocations
    pub custom_allocations: HashMap<String, f64>,
}

/// Allocation reason
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AllocationReason {
    /// Initial allocation
    Initial,
    /// Scale up due to high demand
    ScaleUp,
    /// Scale down due to low demand
    ScaleDown,
    /// Rebalancing for optimization
    Rebalancing,
    /// Failover to healthy primal
    Failover,
    /// Cost optimization
    CostOptimization,
    /// Performance optimization
    PerformanceOptimization,
}

/// Resource allocation decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    /// Target primal
    pub target_primal: String,
    /// Allocated resources
    pub allocated_resources: AllocatedResources,
    /// Allocation reason
    pub reason: AllocationReason,
    /// Allocation timestamp
    pub timestamp: u64,
    /// Expected utilization
    pub expected_utilization: f64,
}

/// Factors influencing load balancing decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingFactors {
    /// Health score factor
    pub health_score: f64,
    /// Resource utilization factor
    pub resource_factor: f64,
    /// Latency factor
    pub latency_factor: f64,
    /// Cost factor
    pub cost_factor: f64,
    /// Sovereignty factor
    pub sovereignty_factor: f64,
}

/// Load balancing decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingDecision {
    /// Service identifier
    pub service_id: String,
    /// Selected primal for routing
    pub selected_primal: String,
    /// Decision weight/score
    pub decision_score: f64,
    /// Decision factors
    pub factors: LoadBalancingFactors,
    /// Decision timestamp
    pub timestamp: u64,
}

/// Connection state for load balancing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionState {
    /// Active connections
    pub active_connections: u64,
    /// Total requests
    pub total_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Last request timestamp
    pub last_request: u64,
}

/// Health snapshot for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSnapshot {
    /// Timestamp
    pub timestamp: u64,
    /// Health score
    pub health_score: f64,
    /// Performance metrics
    pub performance: PerformanceMetrics,
}

/// Trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Improving health
    Improving,
    /// Stable health
    Stable,
    /// Declining health
    Declining,
    /// Unknown trend
    Unknown,
}

/// Health trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthTrend {
    /// Trend direction
    pub direction: TrendDirection,
    /// Trend strength
    pub strength: f64,
    /// Predicted future health
    pub predicted_health: f64,
    /// Confidence in prediction
    pub confidence: f64,
}

/// Model types for prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    /// Linear regression
    LinearRegression,
    /// Moving average
    MovingAverage,
    /// Exponential smoothing
    ExponentialSmoothing,
    /// Time series decomposition
    TimeSeriesDecomposition,
    /// Machine learning model
    MachineLearning { algorithm: String },
}

/// Prediction model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionModel {
    /// Model type
    pub model_type: ModelType,
    /// Model parameters
    pub parameters: HashMap<String, f64>,
    /// Model accuracy
    pub accuracy: f64,
    /// Last training timestamp
    pub last_training: u64,
}

/// Demand metrics for prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemandMetrics {
    /// Request volume
    pub request_volume: f64,
    /// Data volume
    pub data_volume: f64,
    /// Compute demand
    pub compute_demand: f64,
    /// Storage demand
    pub storage_demand: f64,
}

/// Resource snapshot for prediction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    /// Timestamp
    pub timestamp: u64,
    /// Resource utilization
    pub utilization: ResourceUtilization,
    /// Performance metrics
    pub performance: PerformanceMetrics,
    /// Demand metrics
    pub demand: DemandMetrics,
}

/// Optimization event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationEventType {
    /// Resource rebalancing
    ResourceRebalancing,
    /// Load balancing adjustment
    LoadBalancingAdjustment,
    /// Scaling decision
    ScalingDecision,
    /// Failover event
    FailoverEvent,
    /// Cost optimization
    CostOptimization,
    /// Performance optimization
    PerformanceOptimization,
}

/// Optimization state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationState {
    /// Global resource utilization
    pub global_utilization: ResourceUtilization,
    /// Performance metrics
    pub performance: PerformanceMetrics,
    /// Cost metrics
    pub cost: f64,
    /// Efficiency score
    pub efficiency_score: f64,
}

/// Optimization impact metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationImpact {
    /// Performance improvement
    pub performance_improvement: f64,
    /// Cost savings
    pub cost_savings: f64,
    /// Resource efficiency gain
    pub efficiency_gain: f64,
    /// Sovereignty compliance improvement
    pub sovereignty_improvement: f64,
}

/// Optimization event for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationEvent {
    /// Event ID
    pub event_id: String,
    /// Event type
    pub event_type: OptimizationEventType,
    /// Event timestamp
    pub timestamp: u64,
    /// Event description
    pub description: String,
    /// Before state
    pub before_state: OptimizationState,
    /// After state
    pub after_state: OptimizationState,
    /// Optimization impact
    pub impact: OptimizationImpact,
}

/// Default implementation for OptimizationConfig
impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            optimization_interval: 300, // 5 minutes
            target_utilization: 0.8,
            load_balancing_strategy: LoadBalancingStrategy::Hybrid(HybridStrategy {
                health_weight: 0.3,
                resource_weight: 0.25,
                latency_weight: 0.2,
                cost_weight: 0.15,
                sovereignty_weight: 0.1,
            }),
            resource_allocation_strategy: ResourceAllocationStrategy::DemandBased,
            predictive_optimization: true,
            cost_optimization_priority: 0.3,
            performance_optimization_priority: 0.7,
            sovereignty_requirements: SovereigntyRequirements {
                require_sovereign_primals: false,
                data_residency: Vec::new(),
                prohibited_vendors: Vec::new(),
                preferred_sovereign_providers: Vec::new(),
            },
        }
    }
} 