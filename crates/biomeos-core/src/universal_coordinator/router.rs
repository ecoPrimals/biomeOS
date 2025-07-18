//! Capability router for routing requests to primals
//!
//! This module contains the capability router implementation that routes
//! capability requests to appropriate primal instances.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use chrono::Utc;

use crate::universal_primal::{
    CapabilityRequest, CapabilityResponse, DiscoveredPrimal,
};
use crate::{BiomeError, BiomeResult};
use super::types::{CapabilityRouter, RoutingStrategy};

/// Connection tracking for load balancing
#[derive(Debug, Clone)]
pub struct ConnectionMetrics {
    /// Active connections count
    pub active_connections: u32,
    /// Total requests processed
    pub total_requests: u64,
    /// Average response time in milliseconds
    pub avg_response_time: f64,
    /// Last request timestamp
    pub last_request: Instant,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Weight for weighted routing
    pub weight: f64,
}

impl Default for ConnectionMetrics {
    fn default() -> Self {
        Self {
            active_connections: 0,
            total_requests: 0,
            avg_response_time: 0.0,
            last_request: Instant::now(),
            success_rate: 1.0,
            weight: 1.0,
        }
    }
}

/// Enhanced capability router with connection tracking
pub struct EnhancedCapabilityRouter {
    /// Capability map
    capability_map: Arc<RwLock<HashMap<String, Vec<String>>>>,
    /// Primal map
    primal_map: Arc<RwLock<HashMap<String, DiscoveredPrimal>>>,
    /// Connection metrics for each primal
    connection_metrics: Arc<RwLock<HashMap<String, ConnectionMetrics>>>,
    /// Routing strategy
    routing_strategy: RoutingStrategy,
    /// Round-robin counter
    round_robin_counter: Arc<RwLock<usize>>,
}

impl CapabilityRouter {
    /// Create new capability router
    pub fn new() -> Self {
        Self {
            capability_map: Arc::new(RwLock::new(HashMap::new())),
            primal_map: Arc::new(RwLock::new(HashMap::new())),
            routing_strategy: RoutingStrategy::default(),
        }
    }

    /// Create router with specific routing strategy
    pub fn with_strategy(strategy: RoutingStrategy) -> Self {
        Self {
            capability_map: Arc::new(RwLock::new(HashMap::new())),
            primal_map: Arc::new(RwLock::new(HashMap::new())),
            routing_strategy: strategy,
        }
    }

    /// Update primals in the router
    pub async fn update_primals(&self, primals: &[DiscoveredPrimal]) -> BiomeResult<()> {
        let mut capability_map = self.capability_map.write().await;
        let mut primal_map = self.primal_map.write().await;

        // Clear existing data
        capability_map.clear();
        primal_map.clear();

        // Update with new primals
        for primal in primals {
            // Add to primal map
            primal_map.insert(primal.id.clone(), primal.clone());

            // Add to capability map
            for capability in &primal.capabilities {
                let capability_primals = capability_map
                    .entry(capability.name.clone())
                    .or_insert_with(Vec::new);
                capability_primals.push(primal.id.clone());
            }
        }

        Ok(())
    }

    /// Route capability request to appropriate primal
    pub async fn route_capability_request(
        &self,
        request: &CapabilityRequest,
    ) -> BiomeResult<CapabilityResponse> {
        let capability_map = self.capability_map.read().await;
        let primal_map = self.primal_map.read().await;

        // Find primals that can handle this capability
        let capability_name = &request.capability_name;
        let primal_ids = capability_map.get(capability_name).ok_or_else(|| {
            BiomeError::RuntimeError(format!(
                "No primals found for capability: {}",
                capability_name
            ))
        })?;

        if primal_ids.is_empty() {
            return Err(BiomeError::RuntimeError(format!(
                "No available primals for capability: {}",
                capability_name
            )));
        }

        // Select primal based on routing strategy
        let selected_primal_id = self.select_primal(primal_ids, &self.routing_strategy).await;
        let selected_primal = primal_map.get(&selected_primal_id).ok_or_else(|| {
            BiomeError::RuntimeError(format!("Primal not found: {}", selected_primal_id))
        })?;

        // Send request to selected primal
        let response = self.send_request_to_primal(selected_primal, request).await?;

        Ok(response)
    }

    /// Select primal based on routing strategy
    async fn select_primal(&self, primal_ids: &[String], strategy: &RoutingStrategy) -> String {
        match strategy {
            RoutingStrategy::RoundRobin => {
                self.select_round_robin(primal_ids).await
            }
            RoutingStrategy::Random => {
                self.select_random(primal_ids).await
            }
            RoutingStrategy::LeastConnections => {
                self.select_least_connections(primal_ids).await
            }
            RoutingStrategy::LeastLatency => {
                self.select_least_latency(primal_ids).await
            }
            RoutingStrategy::Weighted => {
                self.select_weighted(primal_ids).await
            }
        }
    }

    /// Round-robin selection
    async fn select_round_robin(&self, primal_ids: &[String]) -> String {
        if primal_ids.is_empty() {
            return "".to_string();
        }

        // Simple round-robin implementation
        let index = (std::process::id() as usize + chrono::Utc::now().timestamp() as usize) % primal_ids.len();
        primal_ids[index].clone()
    }

    /// Random selection
    async fn select_random(&self, primal_ids: &[String]) -> String {
        if primal_ids.is_empty() {
            return "".to_string();
        }

        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..primal_ids.len());
        primal_ids[index].clone()
    }

    /// Least connections selection
    async fn select_least_connections(&self, primal_ids: &[String]) -> String {
        if primal_ids.is_empty() {
            return "".to_string();
        }

        // Mock implementation - in reality would query connection metrics
        let mut best_primal = &primal_ids[0];
        let mut min_connections = u32::MAX;

        for primal_id in primal_ids {
            // Simulate connection count based on primal ID hash
            let connection_count = self.get_mock_connection_count(primal_id).await;
            
            if connection_count < min_connections {
                min_connections = connection_count;
                best_primal = primal_id;
            }
        }

        best_primal.clone()
    }

    /// Least latency selection
    async fn select_least_latency(&self, primal_ids: &[String]) -> String {
        if primal_ids.is_empty() {
            return "".to_string();
        }

        // Mock implementation - in reality would query latency metrics
        let mut best_primal = &primal_ids[0];
        let mut min_latency = f64::MAX;

        for primal_id in primal_ids {
            // Simulate latency based on primal ID characteristics
            let latency = self.get_mock_latency(primal_id).await;
            
            if latency < min_latency {
                min_latency = latency;
                best_primal = primal_id;
            }
        }

        best_primal.clone()
    }

    /// Weighted selection
    async fn select_weighted(&self, primal_ids: &[String]) -> String {
        if primal_ids.is_empty() {
            return "".to_string();
        }

        // Mock implementation - in reality would use configured weights
        let mut weighted_selection = Vec::new();
        
        for primal_id in primal_ids {
            let weight = self.get_mock_weight(primal_id).await;
            
            // Add primal to selection based on weight
            for _ in 0..(weight as u32) {
                weighted_selection.push(primal_id.clone());
            }
        }

        if weighted_selection.is_empty() {
            return primal_ids[0].clone();
        }

        // Random selection from weighted list
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..weighted_selection.len());
        weighted_selection[index].clone()
    }

    /// Send request to selected primal
    async fn send_request_to_primal(
        &self,
        primal: &DiscoveredPrimal,
        request: &CapabilityRequest,
    ) -> BiomeResult<CapabilityResponse> {
        // Mock implementation - in reality would make actual network call
        tracing::info!(
            primal_id = primal.id,
            capability = request.capability_name,
            "Sending request to primal"
        );

        // Simulate processing time
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Return mock response
        let response = CapabilityResponse {
            request_id: request.id.clone(),
            success: true,
            data: Some(serde_json::json!({
                "result": "success",
                "primal_id": primal.id,
                "capability": request.capability_name,
                "processed_at": Utc::now().to_rfc3339(),
            })),
            error: None,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        };

        Ok(response)
    }

    /// Get mock connection count for a primal
    async fn get_mock_connection_count(&self, primal_id: &str) -> u32 {
        // Hash-based mock connection count
        let hash = primal_id.len() as u32 * 13;
        hash % 100
    }

    /// Get mock latency for a primal
    async fn get_mock_latency(&self, primal_id: &str) -> f64 {
        // Hash-based mock latency
        let hash = primal_id.len() as f64 * 7.3;
        (hash % 500.0) + 10.0 // 10-510ms range
    }

    /// Get mock weight for a primal
    async fn get_mock_weight(&self, primal_id: &str) -> f64 {
        // Hash-based mock weight
        let hash = primal_id.len() as f64 * 11.7;
        (hash % 10.0) + 1.0 // 1-11 weight range
    }
}

impl Default for CapabilityRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Router statistics
#[derive(Debug, Clone)]
pub struct RouterStats {
    /// Total number of capabilities
    pub total_capabilities: usize,
    /// Total number of primals
    pub total_primals: usize,
    /// Average capabilities per primal
    pub capabilities_per_primal: f64,
    /// Current routing strategy
    pub routing_strategy: RoutingStrategy,
} 