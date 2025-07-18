//! Cross-primal coordinator implementation
//!
//! This module contains the main coordinator that manages cross-primal
//! communication, message routing, and primal registry.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{BiomeResult, PrimalHealth};
use super::types::{
    CrossPrimalMessage, CrossPrimalResponse, MessageType, PrimalIdentity,
    ResponseStatus, ProtocolStatistics,
};
use super::config::CrossPrimalProtocolConfig;
use super::handler::CrossPrimalMessageHandler;

/// Response tracking information
#[derive(Debug, Clone)]
pub struct ResponseTrackingInfo {
    pub response: CrossPrimalResponse,
    pub created_at: u64,
}

/// Cross-primal protocol coordinator
pub struct CrossPrimalCoordinator {
    /// Registered message handlers
    handlers: Arc<RwLock<HashMap<MessageType, Arc<dyn CrossPrimalMessageHandler>>>>,
    /// Active message tracking
    active_messages: Arc<RwLock<HashMap<Uuid, CrossPrimalMessage>>>,
    /// Response tracking with timestamps
    response_tracking: Arc<RwLock<HashMap<Uuid, ResponseTrackingInfo>>>,
    /// Primal registry
    primal_registry: Arc<RwLock<HashMap<String, PrimalIdentity>>>,
    /// Protocol configuration
    config: CrossPrimalProtocolConfig,
    /// Protocol statistics
    statistics: Arc<RwLock<ProtocolStatistics>>,
}

impl CrossPrimalCoordinator {
    /// Create new cross-primal coordinator
    pub fn new(config: CrossPrimalProtocolConfig) -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            active_messages: Arc::new(RwLock::new(HashMap::new())),
            response_tracking: Arc::new(RwLock::new(HashMap::new())),
            primal_registry: Arc::new(RwLock::new(HashMap::new())),
            config,
            statistics: Arc::new(RwLock::new(ProtocolStatistics::default())),
        }
    }

    /// Create coordinator with default configuration
    pub fn with_default_config() -> Self {
        Self::new(CrossPrimalProtocolConfig::default())
    }

    /// Register a message handler
    pub async fn register_handler(
        &self,
        message_type: MessageType,
        handler: Arc<dyn CrossPrimalMessageHandler>,
    ) -> BiomeResult<()> {
        let mut handlers = self.handlers.write().await;
        handlers.insert(message_type, handler);
        Ok(())
    }

    /// Unregister a message handler
    pub async fn unregister_handler(&self, message_type: &MessageType) -> BiomeResult<()> {
        let mut handlers = self.handlers.write().await;
        handlers.remove(message_type);
        Ok(())
    }

    /// Register a primal in the registry
    pub async fn register_primal(&self, identity: PrimalIdentity) -> BiomeResult<()> {
        let mut registry = self.primal_registry.write().await;
        registry.insert(identity.instance_id.clone(), identity);
        Ok(())
    }

    /// Unregister a primal from the registry
    pub async fn unregister_primal(&self, instance_id: &str) -> BiomeResult<()> {
        let mut registry = self.primal_registry.write().await;
        registry.remove(instance_id);
        Ok(())
    }

    /// Get primal by instance ID
    pub async fn get_primal(&self, instance_id: &str) -> Option<PrimalIdentity> {
        let registry = self.primal_registry.read().await;
        registry.get(instance_id).cloned()
    }

    /// Get all registered primals
    pub async fn get_all_primals(&self) -> Vec<PrimalIdentity> {
        let registry = self.primal_registry.read().await;
        registry.values().cloned().collect()
    }

    /// Get primals by type
    pub async fn get_primals_by_type(&self, primal_type: &str) -> Vec<PrimalIdentity> {
        let registry = self.primal_registry.read().await;
        registry
            .values()
            .filter(|identity| identity.primal_type == primal_type)
            .cloned()
            .collect()
    }

    /// Send message to target primal
    pub async fn send_message(
        &self,
        message: CrossPrimalMessage,
    ) -> BiomeResult<CrossPrimalResponse> {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // Track the message
        {
            let mut active = self.active_messages.write().await;
            active.insert(message.id, message.clone());
        }

        // Update statistics - message sent
        {
            let mut stats = self.statistics.write().await;
            stats.messages_sent += 1;
        }

        // Route the message
        match self.route_message(message).await {
            Ok(response) => {
                let processing_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64 - start_time;

                // Track the response
                {
                    let mut responses = self.response_tracking.write().await;
                    responses.insert(response.id, ResponseTrackingInfo {
                        response: response.clone(),
                        created_at: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    });
                }

                // Update statistics - response received
                {
                    let mut stats = self.statistics.write().await;
                    stats.responses_received += 1;
                    
                    // Update average processing time
                    let total_messages = stats.messages_sent;
                    let current_avg = stats.avg_processing_time;
                    stats.avg_processing_time = (current_avg * (total_messages - 1) as f64 + processing_time as f64) / total_messages as f64;
                    
                    // Update success rate based on response status
                    match response.status {
                        ResponseStatus::Success => {
                            // Success rate calculation is handled in get_statistics
                        }
                        ResponseStatus::Error | ResponseStatus::Timeout => {
                            stats.errors += 1;
                        }
                        ResponseStatus::Timeout => {
                            stats.timeouts += 1;
                        }
                        _ => {}
                    }
                }

                Ok(response)
            }
            Err(e) => {
                // Update error statistics
                {
                    let mut stats = self.statistics.write().await;
                    stats.errors += 1;
                }
                Err(e)
            }
        }
    }

    /// Route message to appropriate handler
    async fn route_message(&self, message: CrossPrimalMessage) -> BiomeResult<CrossPrimalResponse> {
        // Update statistics - message received
        {
            let mut stats = self.statistics.write().await;
            stats.messages_received += 1;
        }

        let handlers = self.handlers.read().await;
        match handlers.get(&message.message_type) {
            Some(handler) => {
                let response = handler.handle_message(message.clone()).await?;
                
                // Update statistics - response sent
                {
                    let mut stats = self.statistics.write().await;
                    stats.responses_sent += 1;
                }

                Ok(response)
            }
            None => {
                let response = CrossPrimalResponse {
                    id: Uuid::new_v4(),
                    in_response_to: message.id,
                    status: ResponseStatus::Error,
                    payload: serde_json::json!({
                        "error": "No handler registered for message type",
                        "message_type": message.message_type
                    }),
                    processing_time: 0,
                    metadata: HashMap::new(),
                };

                // Update statistics - response sent (error)
                {
                    let mut stats = self.statistics.write().await;
                    stats.responses_sent += 1;
                    stats.errors += 1;
                }

                Ok(response)
            }
        }
    }

    /// Send message to multiple primals
    pub async fn broadcast_message(
        &self,
        message: CrossPrimalMessage,
    ) -> BiomeResult<Vec<CrossPrimalResponse>> {
        let mut responses = Vec::new();
        
        // For now, just send to all registered primals
        // In a real implementation, this would use actual network communication
        let primals = self.get_all_primals().await;
        
        for _primal in primals {
            // This would be actual network communication
            let response = self.send_message(message.clone()).await?;
            responses.push(response);
        }
        
        Ok(responses)
    }

    /// Get message status
    pub async fn get_message_status(&self, message_id: &Uuid) -> Option<CrossPrimalMessage> {
        let active = self.active_messages.read().await;
        active.get(message_id).cloned()
    }

    /// Get response by ID
    pub async fn get_response(&self, response_id: &Uuid) -> Option<CrossPrimalResponse> {
        let responses = self.response_tracking.read().await;
        responses.get(response_id).map(|info| info.response.clone())
    }

    /// Health check for the coordinator
    pub async fn health_check(&self) -> BiomeResult<PrimalHealth> {
        let active_count = self.active_messages.read().await.len();
        let response_count = self.response_tracking.read().await.len();
        let primal_count = self.primal_registry.read().await.len();
        let handler_count = self.handlers.read().await.len();

        // Basic health assessment
        let health_status = if active_count < 1000 && response_count < 5000 {
            crate::HealthStatus::Healthy
        } else if active_count < 5000 && response_count < 20000 {
            crate::HealthStatus::Degraded
        } else {
            crate::HealthStatus::Unhealthy
        };

        let health_score = if active_count < 1000 && response_count < 5000 {
            1.0
        } else if active_count < 5000 && response_count < 20000 {
            0.7
        } else {
            0.3
        };

        Ok(PrimalHealth {
            status: health_status,
            health_score,
            last_check: chrono::Utc::now(),
            details: {
                let mut details = HashMap::new();
                details.insert("active_messages".to_string(), serde_json::Value::Number(active_count.into()));
                details.insert("response_count".to_string(), serde_json::Value::Number(response_count.into()));
                details.insert("primal_count".to_string(), serde_json::Value::Number(primal_count.into()));
                details.insert("handler_count".to_string(), serde_json::Value::Number(handler_count.into()));
                details.insert("version".to_string(), serde_json::Value::String(env!("CARGO_PKG_VERSION").to_string()));
                details
            },
            metrics: crate::universal_primal_provider::HealthMetrics {
                cpu_usage: 0.0, // Would be actual CPU usage
                memory_mb: 0.0, // Would be actual memory usage
                response_time_ms: 0.0, // Would be actual response time
                error_rate: 0.0, // Would be calculated from statistics
                active_connections: active_count as u64,
            },
        })
    }

    /// Get protocol statistics
    pub async fn get_statistics(&self) -> BiomeResult<ProtocolStatistics> {
        let stats = self.statistics.read().await;
        
        // Calculate success rate
        let total_responses = stats.responses_received + stats.responses_sent;
        let success_rate = if total_responses > 0 {
            ((total_responses - stats.errors) as f64 / total_responses as f64) * 100.0
        } else {
            100.0
        };

        Ok(ProtocolStatistics {
            messages_sent: stats.messages_sent,
            messages_received: stats.messages_received,
            responses_sent: stats.responses_sent,
            responses_received: stats.responses_received,
            errors: stats.errors,
            timeouts: stats.timeouts,
            avg_processing_time: stats.avg_processing_time,
            success_rate,
        })
    }

    /// Get coordinator status
    pub async fn get_coordinator_status(&self) -> CoordinatorStatus {
        let active_count = self.active_messages.read().await.len();
        let response_count = self.response_tracking.read().await.len();
        let primal_count = self.primal_registry.read().await.len();
        let handler_count = self.handlers.read().await.len();

        CoordinatorStatus {
            active_messages: active_count,
            tracked_responses: response_count,
            registered_primals: primal_count,
            registered_handlers: handler_count,
            uptime: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Cleanup expired messages and responses
    pub async fn cleanup_expired(&self) -> BiomeResult<()> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Clean up expired messages
        {
            let mut active = self.active_messages.write().await;
            active.retain(|_, message| current_time < message.timestamp + message.ttl);
        }

        // Clean up old responses (keep for 1 hour)
        {
            let mut responses = self.response_tracking.write().await;
            responses.retain(|_, info| {
                // Keep responses for 1 hour (3600 seconds) after creation
                current_time < info.created_at + 3600
            });
        }

        Ok(())
    }

    /// Start periodic cleanup task
    pub async fn start_cleanup_task(&self) -> BiomeResult<()> {
        // This would spawn a background task for cleanup
        // For now, just a placeholder
        Ok(())
    }

    /// Shutdown the coordinator
    pub async fn shutdown(&self) -> BiomeResult<()> {
        // Clear all tracking maps
        {
            let mut handlers = self.handlers.write().await;
            handlers.clear();
        }
        
        {
            let mut active = self.active_messages.write().await;
            active.clear();
        }
        
        {
            let mut responses = self.response_tracking.write().await;
            responses.clear();
        }
        
        {
            let mut registry = self.primal_registry.write().await;
            registry.clear();
        }

        Ok(())
    }
}

/// Coordinator status information
#[derive(Debug, Clone)]
pub struct CoordinatorStatus {
    pub active_messages: usize,
    pub tracked_responses: usize,
    pub registered_primals: usize,
    pub registered_handlers: usize,
    pub uptime: u64,
} 