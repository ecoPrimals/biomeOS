//! Federation Resource State Management
//!
//! This module manages the global resource state across the federation,
//! tracking primal resources, allocations, and optimization decisions.

use std::collections::HashMap;
use super::types::*;

/// Federation resource state
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FederationResourceState {
    /// Primal resource information
    pub primal_resources: HashMap<String, PrimalResourceInfo>,
    /// Global resource metrics
    pub global_metrics: GlobalResourceMetrics,
    /// Resource allocation decisions
    pub allocations: HashMap<String, ResourceAllocation>,
    /// Load balancing decisions
    pub load_balancing: HashMap<String, LoadBalancingDecision>,
    /// Last optimization timestamp
    pub last_optimization: u64,
}

impl Default for FederationResourceState {
    fn default() -> Self {
        Self::new()
    }
}

impl FederationResourceState {
    /// Create new federation resource state
    pub fn new() -> Self {
        Self {
            primal_resources: HashMap::new(),
            global_metrics: GlobalResourceMetrics {
                total_cpu_cores: 0.0,
                total_memory_mb: 0,
                total_storage_gb: 0,
                total_network_mbps: 0,
                avg_utilization: ResourceUtilization {
                    cpu_usage: 0.0,
                    memory_usage: 0.0,
                    storage_usage: 0.0,
                    network_usage: 0.0,
                    gpu_usage: 0.0,
                },
                efficiency_score: 0.0,
                cost_efficiency_score: 0.0,
            },
            allocations: HashMap::new(),
            load_balancing: HashMap::new(),
            last_optimization: 0,
        }
    }

    /// Update primal resource information
    pub fn update_primal_resources(&mut self, primal_id: String, info: PrimalResourceInfo) {
        self.primal_resources.insert(primal_id, info);
        self.recalculate_global_metrics();
    }

    /// Remove primal from resource state
    pub fn remove_primal(&mut self, primal_id: &str) {
        self.primal_resources.remove(primal_id);
        self.allocations.retain(|_, allocation| allocation.target_primal != primal_id);
        self.load_balancing.retain(|_, decision| decision.selected_primal != primal_id);
        self.recalculate_global_metrics();
    }

    /// Get primal resource information
    pub fn get_primal_info(&self, primal_id: &str) -> Option<&PrimalResourceInfo> {
        self.primal_resources.get(primal_id)
    }

    /// Get all healthy primals
    pub fn get_healthy_primals(&self) -> Vec<&PrimalResourceInfo> {
        self.primal_resources
            .values()
            .filter(|info| matches!(info.health.status, crate::HealthStatus::Healthy))
            .collect()
    }

    /// Get primals with low utilization
    pub fn get_underutilized_primals(&self, threshold: f64) -> Vec<&PrimalResourceInfo> {
        self.primal_resources
            .values()
            .filter(|info| {
                let avg_utilization = (info.utilization.cpu_usage
                    + info.utilization.memory_usage
                    + info.utilization.storage_usage)
                    / 3.0;
                avg_utilization < threshold
            })
            .collect()
    }

    /// Get primals with high utilization
    pub fn get_overutilized_primals(&self, threshold: f64) -> Vec<&PrimalResourceInfo> {
        self.primal_resources
            .values()
            .filter(|info| {
                let avg_utilization = (info.utilization.cpu_usage
                    + info.utilization.memory_usage
                    + info.utilization.storage_usage)
                    / 3.0;
                avg_utilization > threshold
            })
            .collect()
    }

    /// Add resource allocation
    pub fn add_allocation(&mut self, allocation_id: String, allocation: ResourceAllocation) {
        self.allocations.insert(allocation_id, allocation);
    }

    /// Remove resource allocation
    pub fn remove_allocation(&mut self, allocation_id: &str) {
        self.allocations.remove(allocation_id);
    }

    /// Get resource allocation
    pub fn get_allocation(&self, allocation_id: &str) -> Option<&ResourceAllocation> {
        self.allocations.get(allocation_id)
    }

    /// Add load balancing decision
    pub fn add_load_balancing_decision(&mut self, service_id: String, decision: LoadBalancingDecision) {
        self.load_balancing.insert(service_id, decision);
    }

    /// Remove load balancing decision
    pub fn remove_load_balancing_decision(&mut self, service_id: &str) {
        self.load_balancing.remove(service_id);
    }

    /// Get load balancing decision
    pub fn get_load_balancing_decision(&self, service_id: &str) -> Option<&LoadBalancingDecision> {
        self.load_balancing.get(service_id)
    }

    /// Update last optimization timestamp
    pub fn update_optimization_timestamp(&mut self) {
        self.last_optimization = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Get resource utilization summary
    pub fn get_utilization_summary(&self) -> ResourceUtilization {
        self.global_metrics.avg_utilization.clone()
    }

    /// Get total resource capacity
    pub fn get_total_capacity(&self) -> ResourceCapacity {
        let mut total = ResourceCapacity {
            cpu_cores: 0.0,
            memory_mb: 0,
            storage_gb: 0,
            network_mbps: 0,
            gpu_count: 0,
            custom_resources: HashMap::new(),
        };

        for info in self.primal_resources.values() {
            total.cpu_cores += info.capacity.cpu_cores;
            total.memory_mb += info.capacity.memory_mb;
            total.storage_gb += info.capacity.storage_gb;
            total.network_mbps += info.capacity.network_mbps;
            total.gpu_count += info.capacity.gpu_count;

            for (key, value) in &info.capacity.custom_resources {
                *total.custom_resources.entry(key.clone()).or_insert(0.0) += value;
            }
        }

        total
    }

    /// Calculate efficiency score
    pub fn calculate_efficiency_score(&self) -> f64 {
        if self.primal_resources.is_empty() {
            return 0.0;
        }

        let total_utilization = self.global_metrics.avg_utilization.cpu_usage
            + self.global_metrics.avg_utilization.memory_usage
            + self.global_metrics.avg_utilization.storage_usage;

        let avg_utilization = total_utilization / 3.0;

        // Efficiency is higher when utilization is close to target (0.8)
        let target_utilization = 0.8;
        let deviation = (avg_utilization - target_utilization).abs();
        1.0 - deviation
    }

    /// Recalculate global metrics
    fn recalculate_global_metrics(&mut self) {
        let mut total_cpu = 0.0;
        let mut total_memory = 0;
        let mut total_storage = 0;
        let mut total_network = 0;
        let mut total_cpu_usage = 0.0;
        let mut total_memory_usage = 0.0;
        let mut total_storage_usage = 0.0;
        let mut total_network_usage = 0.0;
        let mut total_gpu_usage = 0.0;

        let primal_count = self.primal_resources.len() as f64;

        for info in self.primal_resources.values() {
            total_cpu += info.capacity.cpu_cores;
            total_memory += info.capacity.memory_mb;
            total_storage += info.capacity.storage_gb;
            total_network += info.capacity.network_mbps;
            total_cpu_usage += info.utilization.cpu_usage;
            total_memory_usage += info.utilization.memory_usage;
            total_storage_usage += info.utilization.storage_usage;
            total_network_usage += info.utilization.network_usage;
            total_gpu_usage += info.utilization.gpu_usage;
        }

        self.global_metrics = GlobalResourceMetrics {
            total_cpu_cores: total_cpu,
            total_memory_mb: total_memory,
            total_storage_gb: total_storage,
            total_network_mbps: total_network,
            avg_utilization: ResourceUtilization {
                cpu_usage: if primal_count > 0.0 { total_cpu_usage / primal_count } else { 0.0 },
                memory_usage: if primal_count > 0.0 { total_memory_usage / primal_count } else { 0.0 },
                storage_usage: if primal_count > 0.0 { total_storage_usage / primal_count } else { 0.0 },
                network_usage: if primal_count > 0.0 { total_network_usage / primal_count } else { 0.0 },
                gpu_usage: if primal_count > 0.0 { total_gpu_usage / primal_count } else { 0.0 },
            },
            efficiency_score: self.calculate_efficiency_score(),
            cost_efficiency_score: self.calculate_cost_efficiency_score(),
        };
    }

    /// Calculate cost efficiency score
    fn calculate_cost_efficiency_score(&self) -> f64 {
        if self.primal_resources.is_empty() {
            return 0.0;
        }

        let mut total_cost = 0.0;
        let mut total_performance = 0.0;

        for info in self.primal_resources.values() {
            total_cost += info.cost.total_operational_cost;
            total_performance += info.performance.throughput_rps;
        }

        if total_cost > 0.0 {
            total_performance / total_cost
        } else {
            0.0
        }
    }
} 