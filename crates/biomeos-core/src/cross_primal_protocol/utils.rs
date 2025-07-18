//! Utility functions for cross-primal protocol
//!
//! This module contains utility functions for creating and manipulating
//! cross-primal messages and responses.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::{PrimalContext, SecurityLevel};
use super::types::{
    CrossPrimalMessage, CrossPrimalResponse, MessageType, MessagePriority, PrimalIdentity,
    ResponseStatus, RoutingStrategy, MessageRouting, MessageSecurity, RetryConfig,
};

/// Utility functions for message creation
pub mod message_utils {
    use super::*;

    /// Create a capability query message
    pub fn create_capability_query(
        source: PrimalIdentity,
        target: PrimalIdentity,
        context: PrimalContext,
    ) -> CrossPrimalMessage {
        CrossPrimalMessage {
            id: Uuid::new_v4(),
            message_type: MessageType::CapabilityQuery,
            source,
            target,
            payload: serde_json::json!({}),
            context,
            priority: MessagePriority::Normal,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl: 300, // 5 minutes
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
                security_level: SecurityLevel::Standard,
            },
        }
    }

    /// Create a health query message
    pub fn create_health_query(
        source: PrimalIdentity,
        target: PrimalIdentity,
        context: PrimalContext,
    ) -> CrossPrimalMessage {
        CrossPrimalMessage {
            id: Uuid::new_v4(),
            message_type: MessageType::HealthQuery,
            source,
            target,
            payload: serde_json::json!({}),
            context,
            priority: MessagePriority::High,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl: 60, // 1 minute
            routing: MessageRouting {
                strategy: RoutingStrategy::Direct,
                path: vec![],
                metadata: HashMap::new(),
                retry: RetryConfig {
                    max_attempts: 2,
                    retry_delay: 500,
                    backoff_factor: 1.5,
                },
            },
            security: MessageSecurity {
                signature: None,
                encryption: None,
                auth_token: None,
                security_level: SecurityLevel::Standard,
            },
        }
    }

    /// Create a resource request message
    pub fn create_resource_request(
        source: PrimalIdentity,
        target: PrimalIdentity,
        context: PrimalContext,
        resource_type: String,
        resource_spec: serde_json::Value,
    ) -> CrossPrimalMessage {
        CrossPrimalMessage {
            id: Uuid::new_v4(),
            message_type: MessageType::ResourceRequest,
            source,
            target,
            payload: serde_json::json!({
                "resource_type": resource_type,
                "resource_spec": resource_spec
            }),
            context,
            priority: MessagePriority::Normal,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl: 600, // 10 minutes
            routing: MessageRouting {
                strategy: RoutingStrategy::Direct,
                path: vec![],
                metadata: HashMap::new(),
                retry: RetryConfig {
                    max_attempts: 5,
                    retry_delay: 2000,
                    backoff_factor: 2.0,
                },
            },
            security: MessageSecurity {
                signature: None,
                encryption: None,
                auth_token: None,
                security_level: SecurityLevel::Standard,
            },
        }
    }

    /// Create a service discovery message
    pub fn create_service_discovery(
        source: PrimalIdentity,
        target: PrimalIdentity,
        context: PrimalContext,
        service_type: Option<String>,
    ) -> CrossPrimalMessage {
        let mut payload = serde_json::json!({});
        if let Some(service_type) = service_type {
            payload["service_type"] = serde_json::Value::String(service_type);
        }

        CrossPrimalMessage {
            id: Uuid::new_v4(),
            message_type: MessageType::ServiceDiscovery,
            source,
            target,
            payload,
            context,
            priority: MessagePriority::Normal,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl: 180, // 3 minutes
            routing: MessageRouting {
                strategy: RoutingStrategy::Broadcast,
                path: vec![],
                metadata: HashMap::new(),
                retry: RetryConfig {
                    max_attempts: 2,
                    retry_delay: 1000,
                    backoff_factor: 1.5,
                },
            },
            security: MessageSecurity {
                signature: None,
                encryption: None,
                auth_token: None,
                security_level: SecurityLevel::Standard,
            },
        }
    }

    /// Create an authentication request message
    pub fn create_authentication_request(
        source: PrimalIdentity,
        target: PrimalIdentity,
        context: PrimalContext,
        credentials: serde_json::Value,
    ) -> CrossPrimalMessage {
        CrossPrimalMessage {
            id: Uuid::new_v4(),
            message_type: MessageType::AuthenticationRequest,
            source,
            target,
            payload: serde_json::json!({
                "credentials": credentials
            }),
            context,
            priority: MessagePriority::High,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl: 120, // 2 minutes
            routing: MessageRouting {
                strategy: RoutingStrategy::Direct,
                path: vec![],
                metadata: HashMap::new(),
                retry: RetryConfig {
                    max_attempts: 2,
                    retry_delay: 1000,
                    backoff_factor: 1.0,
                },
            },
            security: MessageSecurity {
                signature: None,
                encryption: None,
                auth_token: None,
                security_level: SecurityLevel::High,
            },
        }
    }

    /// Create a compute request message
    pub fn create_compute_request(
        source: PrimalIdentity,
        target: PrimalIdentity,
        context: PrimalContext,
        compute_spec: serde_json::Value,
    ) -> CrossPrimalMessage {
        CrossPrimalMessage {
            id: Uuid::new_v4(),
            message_type: MessageType::ComputeRequest,
            source,
            target,
            payload: serde_json::json!({
                "compute_spec": compute_spec
            }),
            context,
            priority: MessagePriority::Normal,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl: 3600, // 1 hour
            routing: MessageRouting {
                strategy: RoutingStrategy::CapabilityBased(
                    crate::PrimalCapability {
                        name: "compute".to_string(),
                        version: "1.0.0".to_string(),
                        description: "Compute capability for processing tasks".to_string(),
                        parameters: HashMap::new(),
                    }
                ),
                path: vec![],
                metadata: HashMap::new(),
                retry: RetryConfig {
                    max_attempts: 3,
                    retry_delay: 5000,
                    backoff_factor: 2.0,
                },
            },
            security: MessageSecurity {
                signature: None,
                encryption: None,
                auth_token: None,
                security_level: SecurityLevel::Standard,
            },
        }
    }

    /// Create a coordination request message
    pub fn create_coordination_request(
        source: PrimalIdentity,
        target: PrimalIdentity,
        context: PrimalContext,
        coordination_type: String,
        coordination_data: serde_json::Value,
    ) -> CrossPrimalMessage {
        CrossPrimalMessage {
            id: Uuid::new_v4(),
            message_type: MessageType::CoordinationRequest,
            source,
            target,
            payload: serde_json::json!({
                "coordination_type": coordination_type,
                "coordination_data": coordination_data
            }),
            context,
            priority: MessagePriority::High,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            ttl: 900, // 15 minutes
            routing: MessageRouting {
                strategy: RoutingStrategy::Direct,
                path: vec![],
                metadata: HashMap::new(),
                retry: RetryConfig {
                    max_attempts: 3,
                    retry_delay: 2000,
                    backoff_factor: 1.8,
                },
            },
            security: MessageSecurity {
                signature: None,
                encryption: None,
                auth_token: None,
                security_level: SecurityLevel::Standard,
            },
        }
    }
}

/// Utility functions for response creation
pub mod response_utils {
    use super::*;

    /// Create a successful response
    pub fn create_success_response(
        original_message_id: Uuid,
        payload: serde_json::Value,
        processing_time: u64,
    ) -> CrossPrimalResponse {
        CrossPrimalResponse {
            id: Uuid::new_v4(),
            in_response_to: original_message_id,
            status: ResponseStatus::Success,
            payload,
            processing_time,
            metadata: HashMap::new(),
        }
    }

    /// Create an error response
    pub fn create_error_response(
        original_message_id: Uuid,
        error_message: String,
        processing_time: u64,
    ) -> CrossPrimalResponse {
        CrossPrimalResponse {
            id: Uuid::new_v4(),
            in_response_to: original_message_id,
            status: ResponseStatus::Error,
            payload: serde_json::json!({
                "error": error_message
            }),
            processing_time,
            metadata: HashMap::new(),
        }
    }

    /// Create a timeout response
    pub fn create_timeout_response(
        original_message_id: Uuid,
        timeout_duration: u64,
    ) -> CrossPrimalResponse {
        CrossPrimalResponse {
            id: Uuid::new_v4(),
            in_response_to: original_message_id,
            status: ResponseStatus::Timeout,
            payload: serde_json::json!({
                "error": "Request timed out",
                "timeout_duration": timeout_duration
            }),
            processing_time: timeout_duration,
            metadata: HashMap::new(),
        }
    }

    /// Create a rejected response
    pub fn create_rejected_response(
        original_message_id: Uuid,
        reason: String,
        processing_time: u64,
    ) -> CrossPrimalResponse {
        CrossPrimalResponse {
            id: Uuid::new_v4(),
            in_response_to: original_message_id,
            status: ResponseStatus::Rejected,
            payload: serde_json::json!({
                "error": "Request rejected",
                "reason": reason
            }),
            processing_time,
            metadata: HashMap::new(),
        }
    }

    /// Create a partial success response
    pub fn create_partial_success_response(
        original_message_id: Uuid,
        payload: serde_json::Value,
        warnings: Vec<String>,
        processing_time: u64,
    ) -> CrossPrimalResponse {
        CrossPrimalResponse {
            id: Uuid::new_v4(),
            in_response_to: original_message_id,
            status: ResponseStatus::PartialSuccess,
            payload,
            processing_time,
            metadata: HashMap::from([
                ("warnings".to_string(), serde_json::json!(warnings)),
            ]),
        }
    }
}

/// Utility functions for message validation
pub mod validation_utils {
    use super::*;
    use crate::BiomeError;

    /// Validate message format
    pub fn validate_message_format(message: &CrossPrimalMessage) -> Result<(), BiomeError> {
        // Check required fields
        if message.source.instance_id.is_empty() {
            return Err(BiomeError::ValidationError(
                "Source instance ID cannot be empty".to_string(),
            ));
        }

        if message.target.instance_id.is_empty() {
            return Err(BiomeError::ValidationError(
                "Target instance ID cannot be empty".to_string(),
            ));
        }

        if message.ttl == 0 {
            return Err(BiomeError::ValidationError(
                "TTL must be greater than 0".to_string(),
            ));
        }

        // Check routing strategy
        match &message.routing.strategy {
            RoutingStrategy::Multicast(targets) => {
                if targets.is_empty() {
                    return Err(BiomeError::ValidationError(
                        "Multicast routing requires at least one target".to_string(),
                    ));
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Check if message has expired
    pub fn is_message_expired(message: &CrossPrimalMessage) -> bool {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        current_time > message.timestamp + message.ttl
    }

    /// Validate message size
    pub fn validate_message_size(
        message: &CrossPrimalMessage,
        max_size: usize,
    ) -> Result<(), BiomeError> {
        let serialized = serde_json::to_vec(message)
            .map_err(|e| BiomeError::Serialization(e.to_string()))?;

        if serialized.len() > max_size {
            return Err(BiomeError::ValidationError(format!(
                "Message size {} exceeds maximum size {}",
                serialized.len(),
                max_size
            )));
        }

        Ok(())
    }

    /// Validate response format
    pub fn validate_response_format(response: &CrossPrimalResponse) -> Result<(), BiomeError> {
        // Check if response ID is valid
        if response.id.is_nil() {
            return Err(BiomeError::ValidationError(
                "Response ID cannot be nil".to_string(),
            ));
        }

        // Check if in_response_to is valid
        if response.in_response_to.is_nil() {
            return Err(BiomeError::ValidationError(
                "in_response_to field cannot be nil".to_string(),
            ));
        }

        Ok(())
    }
}

/// Utility functions for message routing
pub mod routing_utils {
    use super::*;

    /// Extract routing path from message
    pub fn get_routing_path(message: &CrossPrimalMessage) -> Vec<String> {
        message.routing.path.clone()
    }

    /// Add hop to routing path
    pub fn add_routing_hop(message: &mut CrossPrimalMessage, hop: String) {
        message.routing.path.push(hop);
    }

    /// Check if message should be retried
    pub fn should_retry(message: &CrossPrimalMessage, attempt: u32) -> bool {
        attempt < message.routing.retry.max_attempts
    }

    /// Calculate retry delay
    pub fn calculate_retry_delay(message: &CrossPrimalMessage, attempt: u32) -> u64 {
        let base_delay = message.routing.retry.retry_delay;
        let backoff_factor = message.routing.retry.backoff_factor;

        (base_delay as f64 * backoff_factor.powi(attempt as i32)) as u64
    }

    /// Check if target matches routing strategy
    pub fn matches_routing_strategy(
        strategy: &RoutingStrategy,
        primal: &PrimalIdentity,
    ) -> bool {
        match strategy {
            RoutingStrategy::Direct => true,
            RoutingStrategy::Broadcast => true,
            RoutingStrategy::Multicast(targets) => {
                targets.contains(&primal.instance_id)
            }
            RoutingStrategy::RoundRobin => true,
            RoutingStrategy::LoadBalanced => true,
            RoutingStrategy::CapabilityBased(required_capability) => {
                primal.capabilities.iter().any(|cap| {
                    cap.name == required_capability.name
                        && cap.version == required_capability.version
                })
            }
        }
    }
} 