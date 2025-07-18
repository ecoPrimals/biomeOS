//! Main Federation Optimizer
//!
//! This module implements the core federation optimization logic, coordinating
//! resource allocation, load balancing, and predictive optimization across primals.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    ecosystem_integration::EcosystemServiceRegistry, BiomeResult, CrossPrimalCoordinator,
    CrossPrimalMessage, MessagePriority, MessageType, PrimalContext, PrimalHealth, PrimalIdentity,
    PrimalType, HealthStatus, NetworkLocation, SecurityLevel, MessageRouting, RoutingStrategy,
    RetryConfig, MessageSecurity,
};

use super::types::*;
use super::resource_state::FederationResourceState;
use super::load_balancer::{FederationLoadBalancer, LoadBalancingRequest};
use super::resource_predictor::{ResourcePredictor, ResourcePrediction};

/// Federation-wide optimization coordinator
pub struct FederationOptimizer {
    /// Cross-primal coordinator for communication
    coordinator: Arc<CrossPrimalCoordinator>,
    /// Service registry for discovery
    service_registry: Arc<EcosystemServiceRegistry>,
    /// Optimization configuration
    config: OptimizationConfig,
    /// Resource allocation state
    resource_state: Arc<RwLock<FederationResourceState>>,
    /// Load balancing strategies
    load_balancer: Arc<FederationLoadBalancer>,
    /// Resource predictor
    resource_predictor: Arc<ResourcePredictor>,
    /// Optimization history
    optimization_history: Arc<RwLock<Vec<OptimizationEvent>>>,
}

/// Resource analysis results
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceAnalysis {
    /// Identified bottlenecks
    pub bottlenecks: Vec<String>,
    /// Underutilized resources
    pub underutilized_resources: Vec<String>,
    /// Optimization opportunities
    pub optimization_opportunities: Vec<String>,
    /// Predicted demand
    pub predicted_demand: HashMap<String, ResourcePrediction>,
}

/// Optimization recommendation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OptimizationRecommendation {
    /// Recommendation type
    pub recommendation_type: String,
    /// Target primal
    pub target_primal: String,
    /// Recommended action
    pub action: String,
    /// Expected impact
    pub expected_impact: OptimizationImpact,
    /// Priority level
    pub priority: f64,
}

impl FederationOptimizer {
    /// Create new federation optimizer
    pub fn new(
        coordinator: Arc<CrossPrimalCoordinator>,
        service_registry: Arc<EcosystemServiceRegistry>,
        config: OptimizationConfig,
    ) -> Self {
        let load_balancer = Arc::new(FederationLoadBalancer::new(
            config.load_balancing_strategy.clone(),
        ));

        let resource_predictor = Arc::new(ResourcePredictor::new());

        Self {
            coordinator,
            service_registry,
            config,
            resource_state: Arc::new(RwLock::new(FederationResourceState::new())),
            load_balancer,
            resource_predictor,
            optimization_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start federation optimization loop
    pub async fn start_optimization(&self) -> BiomeResult<()> {
        let mut interval =
            tokio::time::interval(Duration::from_secs(self.config.optimization_interval));

        loop {
            interval.tick().await;

            if let Err(e) = self.run_optimization_cycle().await {
                eprintln!("Optimization cycle failed: {}", e);
            }
        }
    }

    /// Run single optimization cycle
    async fn run_optimization_cycle(&self) -> BiomeResult<()> {
        // Collect current federation state
        let state = self.collect_federation_state().await?;

        // Analyze resource utilization
        let analysis = self.analyze_resource_utilization(&state).await?;

        // Generate optimization recommendations
        let recommendations = self
            .generate_optimization_recommendations(&state, &analysis)
            .await?;

        // Apply optimizations
        let impact = self.apply_optimizations(&recommendations).await?;

        // Record optimization event
        self.record_optimization_event(&state, &impact).await?;

        Ok(())
    }

    /// Collect current federation state
    async fn collect_federation_state(&self) -> BiomeResult<FederationResourceState> {
        // Query all registered services for their current state
        let services = self.service_registry.list_services().await?;
        let mut primal_resources = HashMap::new();

        for service in services {
            // Query primal for resource information
            let resource_info = self
                .query_primal_resources(&service.primal_type, &service.service_id)
                .await?;
            primal_resources.insert(service.service_id.clone(), resource_info);
        }

        // Calculate global metrics
        let global_metrics = self.calculate_global_metrics(&primal_resources).await?;

        Ok(FederationResourceState {
            primal_resources,
            global_metrics,
            allocations: HashMap::new(),
            load_balancing: HashMap::new(),
            last_optimization: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    /// Query primal for resource information
    async fn query_primal_resources(
        &self,
        primal_type: &PrimalType,
        service_id: &str,
    ) -> BiomeResult<PrimalResourceInfo> {
        let message = CrossPrimalMessage {
            id: Uuid::new_v4(),
            message_type: MessageType::ResourceRequest,
            source: PrimalIdentity {
                primal_type: "biomeos".to_string(),
                instance_id: "federation-optimizer".to_string(),
                version: "1.0.0".to_string(),
                endpoint: "internal".to_string(),
                capabilities: vec![],
                health: PrimalHealth {
                    status: HealthStatus::Healthy,
                    health_score: 1.0,
                    last_check: chrono::Utc::now(),
                    details: HashMap::new(),
                    metrics: crate::universal_primal_provider::HealthMetrics {
                        cpu_usage: 0.0,
                        memory_mb: 0.0,
                        response_time_ms: 0.0,
                        error_rate: 0.0,
                        active_connections: 0,
                    },
                },
            },
            target: PrimalIdentity {
                primal_type: primal_type.clone(),
                instance_id: service_id.to_string(),
                version: "1.0.0".to_string(),
                endpoint: "internal".to_string(),
                capabilities: vec![],
                health: PrimalHealth {
                    status: HealthStatus::Unknown,
                    health_score: 0.0,
                    last_check: chrono::Utc::now(),
                    details: HashMap::new(),
                    metrics: crate::universal_primal_provider::HealthMetrics {
                        cpu_usage: 0.0,
                        memory_mb: 0.0,
                        response_time_ms: 0.0,
                        error_rate: 0.0,
                        active_connections: 0,
                    },
                },
            },
            payload: serde_json::json!({
                "request_type": "resource_info",
                "include_capacity": true,
                "include_utilization": true,
                "include_performance": true,
                "include_cost": true
            }),
            context: PrimalContext {
                user_id: "system".to_string(),
                device_id: "federation-optimizer".to_string(),
                session_id: Uuid::new_v4().to_string(),
                network_location: NetworkLocation {
                    ip_address: "127.0.0.1".to_string(),
                    subnet: None,
                    network_id: None,
                    geo_location: None,
                },
                security_level: SecurityLevel::High,
                biome_id: None,
                team_id: None,
                metadata: HashMap::new(),
            },
            priority: MessagePriority::High,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl: 30,
            routing: MessageRouting {
                strategy: RoutingStrategy::Direct,
                path: vec![],
                metadata: HashMap::new(),
                retry: RetryConfig {
                    max_attempts: 3,
                    retry_delay: 1000,
                    backoff_factor: 2.0,
                },
            },
            security: MessageSecurity {
                signature: None,
                encryption: None,
                auth_token: None,
                security_level: SecurityLevel::High,
            },
        };

        // Send message and await response
        let response = self.coordinator.send_message(message).await?;

        // Parse response into resource info
        let resource_info = serde_json::from_value(response.payload)?;
        Ok(resource_info)
    }

    /// Calculate global metrics from primal resources
    async fn calculate_global_metrics(
        &self,
        primal_resources: &HashMap<String, PrimalResourceInfo>,
    ) -> BiomeResult<GlobalResourceMetrics> {
        let mut total_cpu = 0.0;
        let mut total_memory = 0u64;
        let mut total_storage = 0u64;
        let mut total_network = 0u64;
        let mut total_cpu_usage = 0.0;
        let mut total_memory_usage = 0.0;
        let mut total_storage_usage = 0.0;
        let mut total_network_usage = 0.0;

        for resource_info in primal_resources.values() {
            total_cpu += resource_info.capacity.cpu_cores;
            total_memory += resource_info.capacity.memory_mb;
            total_storage += resource_info.capacity.storage_gb;
            total_network += resource_info.capacity.network_mbps;

            total_cpu_usage += resource_info.utilization.cpu_usage;
            total_memory_usage += resource_info.utilization.memory_usage;
            total_storage_usage += resource_info.utilization.storage_usage;
            total_network_usage += resource_info.utilization.network_usage;
        }

        let count = primal_resources.len() as f64;
        let avg_utilization = ResourceUtilization {
            cpu_usage: total_cpu_usage / count,
            memory_usage: total_memory_usage / count,
            storage_usage: total_storage_usage / count,
            network_usage: total_network_usage / count,
            gpu_usage: 0.0, // TODO: Add GPU tracking
        };

        Ok(GlobalResourceMetrics {
            total_cpu_cores: total_cpu,
            total_memory_mb: total_memory,
            total_storage_gb: total_storage,
            total_network_mbps: total_network,
            avg_utilization: avg_utilization.clone(),
            efficiency_score: self.calculate_efficiency_score(&avg_utilization),
            cost_efficiency_score: self.calculate_cost_efficiency_score(primal_resources),
        })
    }

    /// Calculate efficiency score
    fn calculate_efficiency_score(&self, utilization: &ResourceUtilization) -> f64 {
        // Ideal utilization is around 70-80%
        let target = self.config.target_utilization;
        let cpu_efficiency = 1.0 - (utilization.cpu_usage - target).abs();
        let memory_efficiency = 1.0 - (utilization.memory_usage - target).abs();
        let storage_efficiency = 1.0 - (utilization.storage_usage - target).abs();
        let network_efficiency = 1.0 - (utilization.network_usage - target).abs();

        (cpu_efficiency + memory_efficiency + storage_efficiency + network_efficiency) / 4.0
    }

    /// Calculate cost efficiency score
    fn calculate_cost_efficiency_score(
        &self,
        primal_resources: &HashMap<String, PrimalResourceInfo>,
    ) -> f64 {
        let total_cost: f64 = primal_resources
            .values()
            .map(|r| r.cost.total_operational_cost)
            .sum();
        let total_throughput: f64 = primal_resources
            .values()
            .map(|r| r.performance.throughput_rps)
            .sum();

        if total_cost > 0.0 && total_throughput > 0.0 {
            total_throughput / total_cost
        } else {
            0.0
        }
    }

    /// Analyze resource utilization
    async fn analyze_resource_utilization(
        &self,
        _state: &FederationResourceState,
    ) -> BiomeResult<ResourceAnalysis> {
        // TODO: Implement resource analysis
        Ok(ResourceAnalysis {
            bottlenecks: vec![],
            underutilized_resources: vec![],
            optimization_opportunities: vec![],
            predicted_demand: HashMap::new(),
        })
    }

    /// Generate optimization recommendations
    async fn generate_optimization_recommendations(
        &self,
        _state: &FederationResourceState,
        _analysis: &ResourceAnalysis,
    ) -> BiomeResult<Vec<OptimizationRecommendation>> {
        // TODO: Implement recommendation generation
        Ok(vec![])
    }

    /// Apply optimizations
    async fn apply_optimizations(
        &self,
        _recommendations: &[OptimizationRecommendation],
    ) -> BiomeResult<OptimizationImpact> {
        // TODO: Implement optimization application
        Ok(OptimizationImpact {
            performance_improvement: 0.0,
            cost_savings: 0.0,
            efficiency_gain: 0.0,
            sovereignty_improvement: 0.0,
        })
    }

    /// Record optimization event
    async fn record_optimization_event(
        &self,
        state: &FederationResourceState,
        impact: &OptimizationImpact,
    ) -> BiomeResult<()> {
        let event = OptimizationEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: OptimizationEventType::ResourceRebalancing,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            description: "Automated resource optimization".to_string(),
            before_state: OptimizationState {
                global_utilization: state.global_metrics.avg_utilization.clone(),
                performance: PerformanceMetrics {
                    avg_response_time_ms: 0.0,
                    throughput_rps: 0.0,
                    error_rate: 0.0,
                    latency_percentiles: LatencyPercentiles {
                        p50: 0.0,
                        p90: 0.0,
                        p95: 0.0,
                        p99: 0.0,
                    },
                },
                cost: 0.0,
                efficiency_score: state.global_metrics.efficiency_score,
            },
            after_state: OptimizationState {
                global_utilization: state.global_metrics.avg_utilization.clone(),
                performance: PerformanceMetrics {
                    avg_response_time_ms: 0.0,
                    throughput_rps: 0.0,
                    error_rate: 0.0,
                    latency_percentiles: LatencyPercentiles {
                        p50: 0.0,
                        p90: 0.0,
                        p95: 0.0,
                        p99: 0.0,
                    },
                },
                cost: 0.0,
                efficiency_score: state.global_metrics.efficiency_score,
            },
            impact: impact.clone(),
        };

        let mut history = self.optimization_history.write().await;
        history.push(event);

        // Keep only recent events
        if history.len() > 1000 {
            history.remove(0);
        }

        Ok(())
    }

    /// Get current resource state
    pub async fn get_resource_state(&self) -> BiomeResult<FederationResourceState> {
        Ok(self.resource_state.read().await.clone())
    }

    /// Update resource state
    pub async fn update_resource_state(&self, state: FederationResourceState) -> BiomeResult<()> {
        *self.resource_state.write().await = state;
        Ok(())
    }

    /// Get load balancer for external use
    pub fn get_load_balancer(&self) -> Arc<FederationLoadBalancer> {
        self.load_balancer.clone()
    }

    /// Get resource predictor for external use
    pub fn get_resource_predictor(&self) -> Arc<ResourcePredictor> {
        self.resource_predictor.clone()
    }

    /// Use resource predictor to predict future needs
    pub async fn predict_resource_needs(&self, primal_id: &str, horizon: u64) -> BiomeResult<ResourcePrediction> {
        self.resource_predictor.predict_demand(primal_id, horizon).await
    }

    /// Use load balancer to route request
    pub async fn route_request(&self, request: LoadBalancingRequest) -> BiomeResult<String> {
        self.load_balancer.select_primal(&request).await
    }

    /// Get optimization history
    pub async fn get_optimization_history(&self) -> Vec<OptimizationEvent> {
        self.optimization_history.read().await.clone()
    }

    /// Get optimization configuration
    pub fn get_config(&self) -> &OptimizationConfig {
        &self.config
    }

    /// Update optimization configuration
    pub fn update_config(&mut self, config: OptimizationConfig) {
        self.config = config;
    }
} 