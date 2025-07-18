//! Federation Load Balancer
//!
//! This module implements intelligent load balancing across primals in the federation,
//! supporting multiple strategies and health tracking.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::BiomeResult;
use super::types::*;

/// Federation load balancer
pub struct FederationLoadBalancer {
    /// Load balancing strategy
    strategy: LoadBalancingStrategy,
    /// Connection tracking
    connections: Arc<RwLock<HashMap<String, ConnectionState>>>,
    /// Health tracking
    health_tracker: Arc<HealthTracker>,
}

/// Health tracker for load balancing
pub struct HealthTracker {
    /// Health history
    health_history: Arc<RwLock<HashMap<String, Vec<HealthSnapshot>>>>,
    /// Health trends
    health_trends: Arc<RwLock<HashMap<String, HealthTrend>>>,
}

/// Load balancing request
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoadBalancingRequest {
    /// Service identifier
    pub service_id: String,
    /// Request type
    pub request_type: String,
    /// Resource requirements
    pub resource_requirements: AllocatedResources,
    /// Latency requirements
    pub latency_requirements: Option<f64>,
    /// Cost constraints
    pub cost_constraints: Option<f64>,
    /// Sovereignty requirements
    pub sovereignty_requirements: Option<SovereigntyRequirements>,
}

impl FederationLoadBalancer {
    /// Create new federation load balancer
    pub fn new(strategy: LoadBalancingStrategy) -> Self {
        Self {
            strategy,
            connections: Arc::new(RwLock::new(HashMap::new())),
            health_tracker: Arc::new(HealthTracker::new()),
        }
    }

    /// Select best primal for request
    pub async fn select_primal(&self, request: &LoadBalancingRequest) -> BiomeResult<String> {
        match &self.strategy {
            LoadBalancingStrategy::RoundRobin => self.round_robin_selection(request).await,
            LoadBalancingStrategy::LeastConnections => {
                self.least_connections_selection(request).await
            }
            LoadBalancingStrategy::WeightedRoundRobin => {
                self.weighted_round_robin_selection(request).await
            }
            LoadBalancingStrategy::HealthBased => self.health_based_selection(request).await,
            LoadBalancingStrategy::ResourceAware => self.resource_aware_selection(request).await,
            LoadBalancingStrategy::LatencyBased => self.latency_based_selection(request).await,
            LoadBalancingStrategy::CostOptimized => self.cost_optimized_selection(request).await,
            LoadBalancingStrategy::Hybrid(hybrid) => self.hybrid_selection(request, hybrid).await,
        }
    }

    /// Round-robin selection
    async fn round_robin_selection(&self, request: &LoadBalancingRequest) -> BiomeResult<String> {
        // Get available primals for this service
        let available_primals = self.get_available_primals(&request.service_id).await?;
        
        if available_primals.is_empty() {
            return Err(crate::BiomeError::ResourceNotFound(
                "No available primals for service".to_string()
            ));
        }

        // Simple round-robin: use request hash to distribute evenly
        let index = self.hash_request(request) % available_primals.len();
        Ok(available_primals[index].clone())
    }

    /// Least connections selection
    async fn least_connections_selection(
        &self,
        request: &LoadBalancingRequest,
    ) -> BiomeResult<String> {
        let available_primals = self.get_available_primals(&request.service_id).await?;
        
        if available_primals.is_empty() {
            return Err(crate::BiomeError::ResourceNotFound(
                "No available primals for service".to_string()
            ));
        }

        let connections = self.connections.read().await;
        let mut min_connections = u64::MAX;
        let mut selected_primal = &available_primals[0];

        for primal in &available_primals {
            let connection_count = connections
                .get(primal)
                .map(|state| state.active_connections)
                .unwrap_or(0);
            
            if connection_count < min_connections {
                min_connections = connection_count;
                selected_primal = primal;
            }
        }

        Ok(selected_primal.clone())
    }

    /// Weighted round-robin selection
    async fn weighted_round_robin_selection(
        &self,
        request: &LoadBalancingRequest,
    ) -> BiomeResult<String> {
        let available_primals = self.get_available_primals(&request.service_id).await?;
        
        if available_primals.is_empty() {
            return Err(crate::BiomeError::ResourceNotFound(
                "No available primals for service".to_string()
            ));
        }

        // Calculate weights based on health and capacity
        let mut weighted_primals = Vec::new();
        let health_history = self.health_tracker.health_history.read().await;

        for primal in &available_primals {
            let health_score = health_history
                .get(primal)
                .and_then(|history| history.last())
                .map(|snapshot| snapshot.health_score)
                .unwrap_or(0.5);

            // Higher health score = higher weight
            let weight = (health_score * 100.0) as u32;
            for _ in 0..weight {
                weighted_primals.push(primal.clone());
            }
        }

        if weighted_primals.is_empty() {
            return Ok(available_primals[0].clone());
        }

        let index = self.hash_request(request) % weighted_primals.len();
        Ok(weighted_primals[index].clone())
    }

    /// Health-based selection
    async fn health_based_selection(&self, request: &LoadBalancingRequest) -> BiomeResult<String> {
        let available_primals = self.get_available_primals(&request.service_id).await?;
        
        if available_primals.is_empty() {
            return Err(crate::BiomeError::ResourceNotFound(
                "No available primals for service".to_string()
            ));
        }

        let health_history = self.health_tracker.health_history.read().await;
        let mut best_health = 0.0;
        let mut selected_primal = &available_primals[0];

        for primal in &available_primals {
            let health_score = health_history
                .get(primal)
                .and_then(|history| history.last())
                .map(|snapshot| snapshot.health_score)
                .unwrap_or(0.0);

            if health_score > best_health {
                best_health = health_score;
                selected_primal = primal;
            }
        }

        Ok(selected_primal.clone())
    }

    /// Resource-aware selection
    async fn resource_aware_selection(
        &self,
        request: &LoadBalancingRequest,
    ) -> BiomeResult<String> {
        let available_primals = self.get_available_primals(&request.service_id).await?;
        
        if available_primals.is_empty() {
            return Err(crate::BiomeError::ResourceNotFound(
                "No available primals for service".to_string()
            ));
        }

        // Calculate resource suitability score
        let mut best_score = 0.0;
        let mut selected_primal = &available_primals[0];

        for primal in &available_primals {
            let resource_score = self.calculate_resource_score(primal, request).await?;
            if resource_score > best_score {
                best_score = resource_score;
                selected_primal = primal;
            }
        }

        Ok(selected_primal.clone())
    }

    /// Latency-based selection
    async fn latency_based_selection(
        &self,
        request: &LoadBalancingRequest,
    ) -> BiomeResult<String> {
        let available_primals = self.get_available_primals(&request.service_id).await?;
        
        if available_primals.is_empty() {
            return Err(crate::BiomeError::ResourceNotFound(
                "No available primals for service".to_string()
            ));
        }

        let health_history = self.health_tracker.health_history.read().await;
        let mut best_latency = f64::MAX;
        let mut selected_primal = &available_primals[0];

        for primal in &available_primals {
            let avg_latency = health_history
                .get(primal)
                .and_then(|history| history.last())
                .map(|snapshot| snapshot.performance.avg_response_time_ms)
                .unwrap_or(1000.0);

            if avg_latency < best_latency {
                best_latency = avg_latency;
                selected_primal = primal;
            }
        }

        Ok(selected_primal.clone())
    }

    /// Cost-optimized selection
    async fn cost_optimized_selection(
        &self,
        request: &LoadBalancingRequest,
    ) -> BiomeResult<String> {
        let available_primals = self.get_available_primals(&request.service_id).await?;
        
        if available_primals.is_empty() {
            return Err(crate::BiomeError::ResourceNotFound(
                "No available primals for service".to_string()
            ));
        }

        // Calculate cost score based on estimated costs
        let mut best_cost_score = 0.0;
        let mut selected_primal = &available_primals[0];

        for primal in &available_primals {
            let cost_score = self.calculate_cost_score(primal, request).await?;
            if cost_score > best_cost_score {
                best_cost_score = cost_score;
                selected_primal = primal;
            }
        }

        Ok(selected_primal.clone())
    }

    /// Hybrid selection
    async fn hybrid_selection(
        &self,
        request: &LoadBalancingRequest,
        hybrid: &HybridStrategy,
    ) -> BiomeResult<String> {
        let available_primals = self.get_available_primals(&request.service_id).await?;
        
        if available_primals.is_empty() {
            return Err(crate::BiomeError::ResourceNotFound(
                "No available primals for service".to_string()
            ));
        }

        let health_history = self.health_tracker.health_history.read().await;
        let mut best_score = 0.0;
        let mut selected_primal = &available_primals[0];

        for primal in &available_primals {
            let mut total_score = 0.0;

            // Health factor
            let health_score = health_history
                .get(primal)
                .and_then(|history| history.last())
                .map(|snapshot| snapshot.health_score)
                .unwrap_or(0.5);
            total_score += health_score * hybrid.health_weight;

            // Resource factor
            let resource_score = self.calculate_resource_score(primal, request).await.unwrap_or(0.5);
            total_score += resource_score * hybrid.resource_weight;

            // Latency factor (inverted - lower latency = higher score)
            let latency = health_history
                .get(primal)
                .and_then(|history| history.last())
                .map(|snapshot| snapshot.performance.avg_response_time_ms)
                .unwrap_or(1000.0);
            let latency_score = 1.0 - (latency / 2000.0).min(1.0); // Normalize to 0-1
            total_score += latency_score * hybrid.latency_weight;

            // Cost factor (higher cost efficiency = higher score)
            let cost_score = self.calculate_cost_score(primal, request).await.unwrap_or(0.5);
            total_score += cost_score * hybrid.cost_weight;

            // Sovereignty factor
            let sovereignty_score = self.calculate_sovereignty_score(primal, request).await.unwrap_or(0.5);
            total_score += sovereignty_score * hybrid.sovereignty_weight;

            if total_score > best_score {
                best_score = total_score;
                selected_primal = primal;
            }
        }

        Ok(selected_primal.clone())
    }

    /// Track connection for a primal
    pub async fn track_connection(&self, primal_id: &str, connection_state: ConnectionState) -> BiomeResult<()> {
        let mut connections = self.connections.write().await;
        connections.insert(primal_id.to_string(), connection_state);
        Ok(())
    }

    /// Get connection state for a primal
    pub async fn get_connection_state(&self, primal_id: &str) -> Option<ConnectionState> {
        let connections = self.connections.read().await;
        connections.get(primal_id).cloned()
    }

    /// Get all active connections
    pub async fn get_active_connections(&self) -> HashMap<String, ConnectionState> {
        self.connections.read().await.clone()
    }

    /// Use health tracker to track primal health
    pub async fn track_primal_health(&self, primal_id: &str, snapshot: HealthSnapshot) -> BiomeResult<()> {
        self.health_tracker.track_health(primal_id, snapshot).await
    }

    /// Get health tracker for external use
    pub fn get_health_tracker(&self) -> Arc<HealthTracker> {
        self.health_tracker.clone()
    }

    /// Update load balancing strategy
    pub fn update_strategy(&mut self, strategy: LoadBalancingStrategy) {
        self.strategy = strategy;
    }

    /// Get current strategy
    pub fn get_strategy(&self) -> &LoadBalancingStrategy {
        &self.strategy
    }

    /// Get connection count for a primal
    pub async fn get_connection_count(&self, primal_id: &str) -> u64 {
        let connections = self.connections.read().await;
        connections.get(primal_id)
            .map(|state| state.active_connections)
            .unwrap_or(0)
    }

    /// Get total connection count
    pub async fn get_total_connections(&self) -> u64 {
        let connections = self.connections.read().await;
        connections.values().map(|state| state.active_connections).sum()
    }

    /// Clear connections for a primal
    pub async fn clear_primal_connections(&self, primal_id: &str) {
        let mut connections = self.connections.write().await;
        connections.remove(primal_id);
    }

    /// Clear all connections
    pub async fn clear_all_connections(&self) {
        let mut connections = self.connections.write().await;
        connections.clear();
    }
}

impl Default for HealthTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthTracker {
    /// Create new health tracker
    pub fn new() -> Self {
        Self {
            health_history: Arc::new(RwLock::new(HashMap::new())),
            health_trends: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Track health snapshot
    pub async fn track_health(&self, primal_id: &str, snapshot: HealthSnapshot) -> BiomeResult<()> {
        let mut history = self.health_history.write().await;
        let primal_history = history
            .entry(primal_id.to_string())
            .or_insert_with(Vec::new);
        primal_history.push(snapshot);

        // Keep only recent snapshots
        if primal_history.len() > 100 {
            primal_history.remove(0);
        }

        // Update trends
        self.update_health_trends(primal_id, primal_history).await?;

        Ok(())
    }

    /// Get health history for a primal
    pub async fn get_health_history(&self, primal_id: &str) -> Option<Vec<HealthSnapshot>> {
        let history = self.health_history.read().await;
        history.get(primal_id).cloned()
    }

    /// Get health trend for a primal
    pub async fn get_health_trend(&self, primal_id: &str) -> Option<HealthTrend> {
        let trends = self.health_trends.read().await;
        trends.get(primal_id).cloned()
    }

    /// Get current health score for a primal
    pub async fn get_current_health_score(&self, primal_id: &str) -> Option<f64> {
        let history = self.health_history.read().await;
        history.get(primal_id)?
            .last()
            .map(|snapshot| snapshot.health_score)
    }

    /// Get average health score for a primal
    pub async fn get_average_health_score(&self, primal_id: &str) -> Option<f64> {
        let history = self.health_history.read().await;
        let primal_history = history.get(primal_id)?;
        
        if primal_history.is_empty() {
            return None;
        }

        let total: f64 = primal_history.iter().map(|s| s.health_score).sum();
        Some(total / primal_history.len() as f64)
    }

    /// Check if primal is healthy
    pub async fn is_primal_healthy(&self, primal_id: &str, threshold: f64) -> bool {
        match self.get_current_health_score(primal_id).await {
            Some(score) => score >= threshold,
            None => false,
        }
    }

    /// Get all healthy primals
    pub async fn get_healthy_primals(&self, threshold: f64) -> Vec<String> {
        let history = self.health_history.read().await;
        let mut healthy_primals = Vec::new();

        for (primal_id, primal_history) in history.iter() {
            if let Some(latest) = primal_history.last() {
                if latest.health_score >= threshold {
                    healthy_primals.push(primal_id.clone());
                }
            }
        }

        healthy_primals
    }

    /// Clear health history for a primal
    pub async fn clear_primal_health(&self, primal_id: &str) {
        let mut history = self.health_history.write().await;
        let mut trends = self.health_trends.write().await;
        history.remove(primal_id);
        trends.remove(primal_id);
    }

    /// Clear all health data
    pub async fn clear_all_health_data(&self) {
        let mut history = self.health_history.write().await;
        let mut trends = self.health_trends.write().await;
        history.clear();
        trends.clear();
    }

    /// Update health trends
    async fn update_health_trends(
        &self,
        primal_id: &str,
        history: &[HealthSnapshot],
    ) -> BiomeResult<()> {
        if history.len() < 2 {
            return Ok(());
        }

        let trend = self.calculate_health_trend(history);
        let mut trends = self.health_trends.write().await;
        trends.insert(primal_id.to_string(), trend);

        Ok(())
    }

    /// Calculate health trend
    fn calculate_health_trend(&self, history: &[HealthSnapshot]) -> HealthTrend {
        // Simple trend calculation - can be enhanced with more sophisticated algorithms
        let recent_scores: Vec<f64> = history
            .iter()
            .rev()
            .take(10)
            .map(|s| s.health_score)
            .collect();

        if recent_scores.len() < 2 {
            return HealthTrend {
                direction: TrendDirection::Unknown,
                strength: 0.0,
                predicted_health: 0.0,
                confidence: 0.0,
            };
        }

        let first_score = recent_scores[recent_scores.len() - 1];
        let last_score = recent_scores[0];
        let trend_strength = (last_score - first_score).abs();

        let direction = if last_score > first_score {
            TrendDirection::Improving
        } else if last_score < first_score {
            TrendDirection::Declining
        } else {
            TrendDirection::Stable
        };

        HealthTrend {
            direction,
            strength: trend_strength,
            predicted_health: last_score,
            confidence: 0.8, // TODO: Calculate real confidence
        }
    }

    /// Get available primals for a service
    async fn get_available_primals(&self, service_id: &str) -> BiomeResult<Vec<String>> {
        // Integration with songbird's service discovery
        // Instead of hardcoding primal names, query the universal service registry
        let service_capabilities = self.get_service_capabilities(service_id).await?;
        
        // Query the universal primal registry for capable instances
        let available_instances = self.query_universal_primal_registry(&service_capabilities).await?;
        
        if available_instances.is_empty() {
            // Fallback to any available primal instances
            self.get_fallback_primals().await
        } else {
            Ok(available_instances)
        }
    }

    /// Get service capabilities for capability-based routing
    async fn get_service_capabilities(&self, service_id: &str) -> BiomeResult<Vec<String>> {
        // Map service types to required capabilities (agnostic)
        let capabilities = match service_id {
            "compute" => vec!["cpu_processing".to_string(), "memory_management".to_string()],
            "storage" => vec!["data_storage".to_string(), "file_system".to_string()],
            "networking" => vec!["network_routing".to_string(), "load_balancing".to_string()],
            "security" => vec!["authentication".to_string(), "encryption".to_string()],
            "collaboration" => vec!["messaging".to_string(), "file_sharing".to_string()],
            "ai" => vec!["ai_processing".to_string(), "model_inference".to_string()],
            _ => vec!["general_purpose".to_string()],
        };
        Ok(capabilities)
    }

    /// Query universal primal registry for capable instances
    async fn query_universal_primal_registry(&self, capabilities: &[String]) -> BiomeResult<Vec<String>> {
        // This would integrate with the actual universal primal registry
        // For now, return a mock implementation that shows the pattern
        
        // In a real implementation, this would:
        // 1. Query the universal primal registry
        // 2. Filter by required capabilities
        // 3. Check health status
        // 4. Return instance IDs (not hardcoded names)
        
        let mut available_instances = Vec::new();
        
        // Mock capability-based discovery
        for capability in capabilities {
            match capability.as_str() {
                "cpu_processing" | "memory_management" => {
                    // Query for compute-capable primals
                    available_instances.extend(self.discover_compute_primals().await?);
                }
                "data_storage" | "file_system" => {
                    // Query for storage-capable primals  
                    available_instances.extend(self.discover_storage_primals().await?);
                }
                "network_routing" | "load_balancing" => {
                    // Query for networking-capable primals
                    available_instances.extend(self.discover_networking_primals().await?);
                }
                "authentication" | "encryption" => {
                    // Query for security-capable primals
                    available_instances.extend(self.discover_security_primals().await?);
                }
                "messaging" | "file_sharing" => {
                    // Query for collaboration-capable primals
                    available_instances.extend(self.discover_collaboration_primals().await?);
                }
                _ => {
                    // Query for general-purpose primals
                    available_instances.extend(self.discover_general_primals().await?);
                }
            }
        }
        
        // Remove duplicates and return
        available_instances.sort();
        available_instances.dedup();
        Ok(available_instances)
    }

    /// Discover compute-capable primal instances
    async fn discover_compute_primals(&self) -> BiomeResult<Vec<String>> {
        // Integration point with songbird's service discovery
        // This would query the service registry for compute-capable instances
        Ok(vec!["compute-instance-1".to_string(), "compute-instance-2".to_string()])
    }

    /// Discover storage-capable primal instances
    async fn discover_storage_primals(&self) -> BiomeResult<Vec<String>> {
        // Integration point with songbird's service discovery
        Ok(vec!["storage-instance-1".to_string(), "storage-instance-2".to_string()])
    }

    /// Discover networking-capable primal instances
    async fn discover_networking_primals(&self) -> BiomeResult<Vec<String>> {
        // Integration point with songbird's service discovery
        Ok(vec!["network-instance-1".to_string(), "network-instance-2".to_string()])
    }

    /// Discover security-capable primal instances
    async fn discover_security_primals(&self) -> BiomeResult<Vec<String>> {
        // Integration point with songbird's service discovery
        Ok(vec!["security-instance-1".to_string(), "security-instance-2".to_string()])
    }

    /// Discover collaboration-capable primal instances
    async fn discover_collaboration_primals(&self) -> BiomeResult<Vec<String>> {
        // Integration point with songbird's service discovery
        Ok(vec!["collab-instance-1".to_string(), "collab-instance-2".to_string()])
    }

    /// Discover general-purpose primal instances
    async fn discover_general_primals(&self) -> BiomeResult<Vec<String>> {
        // Integration point with songbird's service discovery
        Ok(vec!["general-instance-1".to_string(), "general-instance-2".to_string()])
    }

    /// Get fallback primals when no specific instances found
    async fn get_fallback_primals(&self) -> BiomeResult<Vec<String>> {
        // Query for any available healthy primal instances
        // This would integrate with the universal primal registry
        Ok(vec!["fallback-instance-1".to_string()])
    }

    /// Integration point with songbird's load balancing
    pub async fn delegate_to_songbird(&self, request: &LoadBalancingRequest) -> BiomeResult<String> {
        // This would delegate to songbird's load balancing for actual routing
        // songbird already has sophisticated load balancing algorithms
        
        // Mock songbird integration - in reality this would use songbird's traits
        let songbird_request = self.convert_to_songbird_request(request).await?;
        let songbird_response = self.call_songbird_load_balancer(songbird_request).await?;
        
        Ok(songbird_response.selected_instance)
    }

    /// Convert federation request to songbird request format
    async fn convert_to_songbird_request(&self, request: &LoadBalancingRequest) -> BiomeResult<SongbirdRequest> {
        // Convert federation optimization request to songbird's format
        Ok(SongbirdRequest {
            service_id: request.service_id.clone(),
            capabilities: self.get_service_capabilities(&request.service_id).await?,
            resource_requirements: request.resource_requirements.clone(),
            // Map federation parameters to songbird parameters
        })
    }

    /// Call songbird's load balancer
    async fn call_songbird_load_balancer(&self, _request: SongbirdRequest) -> BiomeResult<SongbirdResponse> {
        // This would call songbird's actual load balancing API
        // For now, return a mock response
        Ok(SongbirdResponse {
            selected_instance: "songbird-selected-instance".to_string(),
            routing_metadata: std::collections::HashMap::new(),
        })
    }

    /// Hash request for consistent routing
    fn hash_request(&self, request: &LoadBalancingRequest) -> usize {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        request.service_id.hash(&mut hasher);
        request.request_type.hash(&mut hasher);
        hasher.finish() as usize
    }

    /// Calculate resource score for a primal (now using universal metrics)
    async fn calculate_resource_score(&self, primal_id: &str, request: &LoadBalancingRequest) -> BiomeResult<f64> {
        // Instead of hardcoded logic, query universal metrics
        let metrics = self.get_universal_metrics(primal_id).await?;
        
        let cpu_needed = request.resource_requirements.cpu_cores;
        let memory_needed = request.resource_requirements.memory_mb;
        
        // Calculate resource availability score
        let cpu_available = metrics.cpu_capacity - metrics.cpu_usage;
        let memory_available = metrics.memory_capacity - metrics.memory_usage;
        
        let cpu_score = if cpu_available >= cpu_needed { 1.0 } else { cpu_available / cpu_needed };
        let memory_score = if memory_available >= memory_needed as f64 { 1.0 } else { memory_available / memory_needed as f64 };
        
        // Factor in current load and health
        let load_score = 1.0 - metrics.load_percentage;
        let health_score = metrics.health_score;
        
        Ok((cpu_score + memory_score + load_score + health_score) / 4.0)
    }

    /// Get universal metrics for any primal instance
    async fn get_universal_metrics(&self, primal_id: &str) -> BiomeResult<UniversalMetrics> {
        // This would query the universal monitoring system
        // For now, return mock metrics
        Ok(UniversalMetrics {
            cpu_capacity: 8.0,
            cpu_usage: 2.0,
            memory_capacity: 16384.0,
            memory_usage: 4096.0,
            load_percentage: 0.3,
            health_score: 0.9,
        })
    }

    /// Calculate cost score using universal pricing
    async fn calculate_cost_score(&self, primal_id: &str, request: &LoadBalancingRequest) -> BiomeResult<f64> {
        // Query universal pricing information instead of hardcoded costs
        let pricing = self.get_universal_pricing(primal_id).await?;
        
        let resource_multiplier = (request.resource_requirements.cpu_cores + 
                                 request.resource_requirements.memory_mb as f64 / 1024.0) / 8.0;
        
        let estimated_cost = pricing.base_cost_per_hour * resource_multiplier;
        
        // Apply cost constraints if specified
        if let Some(max_cost) = request.cost_constraints {
            if estimated_cost > max_cost {
                return Ok(0.0); // Not viable due to cost constraints
            }
        }
        
        // Convert to score (lower cost = higher score)
        Ok((pricing.max_reasonable_cost - estimated_cost) / pricing.max_reasonable_cost)
    }

    /// Get universal pricing information
    async fn get_universal_pricing(&self, _primal_id: &str) -> BiomeResult<UniversalPricing> {
        // This would query the universal pricing system
        Ok(UniversalPricing {
            base_cost_per_hour: 0.10,
            max_reasonable_cost: 1.0,
        })
    }

    /// Calculate sovereignty score using universal compliance
    async fn calculate_sovereignty_score(&self, primal_id: &str, request: &LoadBalancingRequest) -> BiomeResult<f64> {
        let Some(sovereignty_req) = &request.sovereignty_requirements else {
            return Ok(1.0); // No sovereignty requirements
        };

        // Query universal compliance system
        let compliance = self.get_universal_compliance(primal_id).await?;
        
        let mut score = 1.0;

        // Check prohibited vendors
        for prohibited in &sovereignty_req.prohibited_vendors {
            if compliance.vendor_info.contains(prohibited) {
                return Ok(0.0); // Prohibited vendor
            }
        }

        // Check preferred sovereign providers
        let is_preferred = sovereignty_req.preferred_sovereign_providers
            .iter()
            .any(|provider| compliance.sovereign_providers.contains(provider));
        
        if is_preferred {
            score += 0.5; // Boost for preferred provider
        }

        // Check data residency requirements
        if !sovereignty_req.data_residency.is_empty() {
            let meets_residency = sovereignty_req.data_residency
                .iter()
                .all(|requirement| compliance.data_residency.contains(requirement));
            if !meets_residency {
                score *= 0.7; // Penalty for not meeting residency
            }
        }

        Ok(score.min(1.0))
    }

    /// Get universal compliance information
    async fn get_universal_compliance(&self, _primal_id: &str) -> BiomeResult<UniversalCompliance> {
        // This would query the universal compliance system
        Ok(UniversalCompliance {
            vendor_info: "sovereign-provider".to_string(),
            sovereign_providers: vec!["sovereign-provider".to_string()],
            data_residency: vec!["domestic".to_string(), "eu".to_string()],
        })
    }
}

/// Songbird integration types
#[derive(Debug, Clone)]
struct SongbirdRequest {
    service_id: String,
    capabilities: Vec<String>,
    resource_requirements: AllocatedResources,
}

#[derive(Debug, Clone)]
struct SongbirdResponse {
    selected_instance: String,
    routing_metadata: HashMap<String, String>,
}

/// Universal metrics for any primal instance
#[derive(Debug, Clone)]
struct UniversalMetrics {
    cpu_capacity: f64,
    cpu_usage: f64,
    memory_capacity: f64,
    memory_usage: f64,
    load_percentage: f64,
    health_score: f64,
}

/// Universal pricing information
#[derive(Debug, Clone)]
struct UniversalPricing {
    base_cost_per_hour: f64,
    max_reasonable_cost: f64,
}

/// Universal compliance information
#[derive(Debug, Clone)]
struct UniversalCompliance {
    vendor_info: String,
    sovereign_providers: Vec<String>,
    data_residency: Vec<String>,
} 