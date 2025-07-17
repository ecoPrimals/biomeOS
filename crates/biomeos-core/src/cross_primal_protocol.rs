//! Cross-Primal Protocol Implementation
//!
//! This module implements the standardized communication protocol for all primals
//! in the biomeOS ecosystem, providing universal message formats, routing,
//! and coordination capabilities.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{BiomeError, BiomeResult, PrimalCapability, PrimalContext, PrimalHealth};

/// Universal message format for cross-primal communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPrimalMessage {
    /// Unique message identifier
    pub id: Uuid,
    /// Message type for routing and processing
    pub message_type: MessageType,
    /// Source primal information
    pub source: PrimalIdentity,
    /// Target primal information
    pub target: PrimalIdentity,
    /// Message payload
    pub payload: serde_json::Value,
    /// Message context for routing decisions
    pub context: PrimalContext,
    /// Message priority for processing order
    pub priority: MessagePriority,
    /// Message timestamp
    pub timestamp: u64,
    /// Message TTL (time-to-live) in seconds
    pub ttl: u64,
    /// Message routing metadata
    pub routing: MessageRouting,
    /// Message security metadata
    pub security: MessageSecurity,
}

/// Message types for different cross-primal operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MessageType {
    /// Capability query and response
    CapabilityQuery,
    CapabilityResponse,

    /// Service discovery and registration
    ServiceDiscovery,
    ServiceRegistration,

    /// Health monitoring and reporting
    HealthQuery,
    HealthReport,

    /// Resource coordination
    ResourceRequest,
    ResourceResponse,
    ResourceAllocation,

    /// Security and authentication
    AuthenticationRequest,
    AuthenticationResponse,
    AuthorizationCheck,

    /// Data and storage operations
    DataRequest,
    DataResponse,
    DataTransfer,

    /// AI and compute operations
    ComputeRequest,
    ComputeResponse,
    ComputeResult,

    /// Coordination and orchestration
    CoordinationRequest,
    CoordinationResponse,
    CoordinationUpdate,

    /// Error and status reporting
    ErrorReport,
    StatusUpdate,

    /// Custom message types for primal-specific operations
    Custom(String),
}

/// Message priority levels for processing order
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    /// Critical system operations
    Critical,
    /// High priority operations
    High,
    /// Normal priority operations
    Normal,
    /// Low priority operations
    Low,
    /// Background operations
    Background,
}

/// Primal identity information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalIdentity {
    /// Primal type (toadstool, songbird, nestgate, etc.)
    pub primal_type: String,
    /// Unique instance identifier
    pub instance_id: String,
    /// Primal version information
    pub version: String,
    /// Network endpoint information
    pub endpoint: String,
    /// Primal capabilities
    pub capabilities: Vec<PrimalCapability>,
    /// Current health status
    pub health: PrimalHealth,
}

/// Message routing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRouting {
    /// Routing strategy
    pub strategy: RoutingStrategy,
    /// Routing path taken by the message
    pub path: Vec<String>,
    /// Routing metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Retry configuration
    pub retry: RetryConfig,
}

/// Message security information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSecurity {
    /// Message signature for integrity
    pub signature: Option<String>,
    /// Encryption metadata
    pub encryption: Option<EncryptionMetadata>,
    /// Authentication token
    pub auth_token: Option<String>,
    /// Security level required
    pub security_level: crate::SecurityLevel,
}

/// Routing strategies for message delivery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingStrategy {
    /// Direct routing to specific primal
    Direct,
    /// Broadcast to all primals of a type
    Broadcast,
    /// Multicast to selected primals
    Multicast(Vec<String>),
    /// Round-robin routing
    RoundRobin,
    /// Load-balanced routing
    LoadBalanced,
    /// Capability-based routing
    CapabilityBased(PrimalCapability),
}

/// Retry configuration for message delivery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,
    /// Retry delay in milliseconds
    pub retry_delay: u64,
    /// Exponential backoff factor
    pub backoff_factor: f64,
}

/// Encryption metadata for secure messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionMetadata {
    /// Encryption algorithm used
    pub algorithm: String,
    /// Key identifier
    pub key_id: String,
    /// Initialization vector
    pub iv: Option<String>,
}

/// Response to a cross-primal message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPrimalResponse {
    /// Response identifier
    pub id: Uuid,
    /// Original message ID this responds to
    pub in_response_to: Uuid,
    /// Response status
    pub status: ResponseStatus,
    /// Response payload
    pub payload: serde_json::Value,
    /// Processing time in milliseconds
    pub processing_time: u64,
    /// Response metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Response status codes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)] // Add PartialEq derive
pub enum ResponseStatus {
    /// Successful response
    Success,
    /// Partial success with warnings
    PartialSuccess,
    /// Request failed
    Error,
    /// Request timed out
    Timeout,
    /// Request was rejected
    Rejected,
    /// Service unavailable
    Unavailable,
    /// Unauthorized access
    Unauthorized,
    /// Rate limited
    RateLimited,
    /// Degraded performance
    Degraded,
}

/// Trait for cross-primal message handlers
#[async_trait]
pub trait CrossPrimalMessageHandler: Send + Sync {
    /// Handle incoming cross-primal message
    async fn handle_message(&self, message: CrossPrimalMessage)
        -> BiomeResult<CrossPrimalResponse>;

    /// Get supported message types
    fn supported_message_types(&self) -> Vec<MessageType>;

    /// Get handler capabilities
    fn handler_capabilities(&self) -> Vec<PrimalCapability>;
}

/// Cross-primal protocol coordinator
pub struct CrossPrimalCoordinator {
    /// Registered message handlers
    handlers: Arc<RwLock<HashMap<MessageType, Arc<dyn CrossPrimalMessageHandler>>>>,
    /// Active message tracking
    active_messages: Arc<RwLock<HashMap<Uuid, CrossPrimalMessage>>>,
    /// Response tracking
    response_tracking: Arc<RwLock<HashMap<Uuid, CrossPrimalResponse>>>,
    /// Primal registry
    primal_registry: Arc<RwLock<HashMap<String, PrimalIdentity>>>,
    /// Protocol configuration
    config: CrossPrimalProtocolConfig,
}

/// Configuration for cross-primal protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPrimalProtocolConfig {
    /// Default message TTL in seconds
    pub default_ttl: u64,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Default retry configuration
    pub default_retry: RetryConfig,
    /// Protocol timeout in seconds
    pub protocol_timeout: u64,
    /// Enable message encryption
    pub enable_encryption: bool,
    /// Message queue limits
    pub queue_limits: QueueLimits,
}

/// Message queue limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueLimits {
    /// Maximum pending messages
    pub max_pending: usize,
    /// Maximum processing time per message
    pub max_processing_time: u64,
    /// Maximum queue size per priority
    pub max_queue_per_priority: usize,
}

impl Default for CrossPrimalProtocolConfig {
    fn default() -> Self {
        Self {
            default_ttl: 300,              // 5 minutes
            max_message_size: 1024 * 1024, // 1 MB
            default_retry: RetryConfig {
                max_attempts: 3,
                retry_delay: 1000, // 1 second
                backoff_factor: 2.0,
            },
            protocol_timeout: 30, // 30 seconds
            enable_encryption: true,
            queue_limits: QueueLimits {
                max_pending: 1000,
                max_processing_time: 60, // 1 minute
                max_queue_per_priority: 200,
            },
        }
    }
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
        }
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

    /// Register a primal in the registry
    pub async fn register_primal(&self, identity: PrimalIdentity) -> BiomeResult<()> {
        let mut registry = self.primal_registry.write().await;
        registry.insert(identity.instance_id.clone(), identity);
        Ok(())
    }

    /// Send a cross-primal message
    pub async fn send_message(
        &self,
        mut message: CrossPrimalMessage,
    ) -> BiomeResult<CrossPrimalResponse> {
        // Validate message
        self.validate_message(&message)?;

        // Set timestamp if not provided
        if message.timestamp == 0 {
            message.timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }

        // Track active message
        {
            let mut active = self.active_messages.write().await;
            active.insert(message.id, message.clone());
        }

        // Route and process message
        let response = self.route_message(message).await?;

        // Track response
        {
            let mut responses = self.response_tracking.write().await;
            responses.insert(response.id, response.clone());
        }

        Ok(response)
    }

    /// Validate message format and content
    fn validate_message(&self, message: &CrossPrimalMessage) -> BiomeResult<()> {
        // Check message size
        let serialized = serde_json::to_string(message).map_err(|e| {
            BiomeError::ValidationError(format!("Failed to serialize message: {}", e))
        })?;

        if serialized.len() > self.config.max_message_size {
            return Err(BiomeError::ValidationError(
                "Message exceeds maximum size limit".to_string(),
            ));
        }

        // Check TTL
        if message.ttl == 0 {
            return Err(BiomeError::ValidationError(
                "Message TTL must be greater than 0".to_string(),
            ));
        }

        // Check routing strategy
        if let RoutingStrategy::Multicast(targets) = &message.routing.strategy {
            if targets.is_empty() {
                return Err(BiomeError::ValidationError(
                    "Multicast routing requires at least one target".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Route message to appropriate handler
    async fn route_message(&self, message: CrossPrimalMessage) -> BiomeResult<CrossPrimalResponse> {
        let handlers = self.handlers.read().await;

        // Find handler for message type
        if let Some(handler) = handlers.get(&message.message_type) {
            // Process message with timeout
            let response = tokio::time::timeout(
                Duration::from_secs(self.config.protocol_timeout),
                handler.handle_message(message.clone()),
            )
            .await
            .map_err(|_| BiomeError::RuntimeError("Message processing timeout".to_string()))?;

            response
        } else {
            // No handler found
            Ok(CrossPrimalResponse {
                id: Uuid::new_v4(),
                in_response_to: message.id,
                status: ResponseStatus::Error,
                payload: serde_json::json!({
                    "error": "No handler found for message type",
                    "message_type": message.message_type
                }),
                processing_time: 0,
                metadata: HashMap::new(),
            })
        }
    }

    /// Broadcast message to all primals of a specific type
    pub async fn broadcast_message(
        &self,
        message_type: MessageType,
        payload: serde_json::Value,
        target_primal_type: String,
        context: PrimalContext,
    ) -> BiomeResult<Vec<CrossPrimalResponse>> {
        let registry = self.primal_registry.read().await;
        let mut responses = Vec::new();

        // Find all primals of the target type
        for identity in registry.values() {
            if identity.primal_type == target_primal_type {
                let message = CrossPrimalMessage {
                    id: Uuid::new_v4(),
                    message_type: message_type.clone(),
                    source: PrimalIdentity {
                        primal_type: "biomeos".to_string(),
                        instance_id: "coordinator".to_string(),
                        version: "1.0.0".to_string(),
                        endpoint: "localhost".to_string(),
                        capabilities: vec![],
                        health: PrimalHealth {
                            status: crate::HealthStatus::Healthy,
                            health_score: 100.0,
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
                    target: identity.clone(),
                    payload: payload.clone(),
                    context: context.clone(),
                    priority: MessagePriority::Normal,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    ttl: self.config.default_ttl,
                    routing: MessageRouting {
                        strategy: RoutingStrategy::Broadcast,
                        path: vec![],
                        metadata: HashMap::new(),
                        retry: self.config.default_retry.clone(),
                    },
                    security: MessageSecurity {
                        signature: None,
                        encryption: None,
                        auth_token: None,
                        security_level: crate::SecurityLevel::Standard,
                    },
                };

                let response = self.send_message(message).await?;
                responses.push(response);
            }
        }

        Ok(responses)
    }

    /// Get protocol statistics
    pub async fn get_statistics(&self) -> BiomeResult<ProtocolStatistics> {
        let active_count = self.active_messages.read().await.len();
        let response_count = self.response_tracking.read().await.len();
        let primal_count = self.primal_registry.read().await.len();

        Ok(ProtocolStatistics {
            active_messages: active_count,
            total_responses: response_count,
            registered_primals: primal_count,
            protocol_version: "1.0.0".to_string(),
            uptime: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
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
            responses.retain(|_, response| {
                // Keep responses for 1 hour after creation
                current_time
                    < response
                        .id
                        .get_timestamp()
                        .unwrap_or(uuid::Timestamp::from_unix(uuid::NoContext, 0, 0))
                        .to_unix()
                        .0
                        + 3600
            });
        }

        Ok(())
    }
}

/// Protocol statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolStatistics {
    /// Number of active messages
    pub active_messages: usize,
    /// Total number of responses
    pub total_responses: usize,
    /// Number of registered primals
    pub registered_primals: usize,
    /// Protocol version
    pub protocol_version: String,
    /// Protocol uptime in seconds
    pub uptime: u64,
}

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
                security_level: crate::SecurityLevel::Standard,
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
                security_level: crate::SecurityLevel::Standard,
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
                security_level: crate::SecurityLevel::Standard,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::universal_primal_provider::HealthMetrics;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn create_health_metrics() -> HealthMetrics {
        HealthMetrics {
            cpu_usage: 0.5,
            memory_mb: 512.0,
            response_time_ms: 50.0,
            error_rate: 0.0,
            active_connections: 10,
        }
    }

    struct TestMessageHandler {
        message_count: AtomicUsize,
    }

    impl TestMessageHandler {
        fn new() -> Self {
            Self {
                message_count: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait::async_trait]
    impl CrossPrimalMessageHandler for TestMessageHandler {
        async fn handle_message(
            &self,
            message: CrossPrimalMessage,
        ) -> BiomeResult<CrossPrimalResponse> {
            self.message_count.fetch_add(1, Ordering::SeqCst);

            Ok(CrossPrimalResponse {
                id: Uuid::new_v4(),
                in_response_to: message.id,
                status: ResponseStatus::Success,
                payload: serde_json::json!({
                    "test": "response",
                    "message_type": message.message_type
                }),
                processing_time: 10,
                metadata: HashMap::new(),
            })
        }

        fn supported_message_types(&self) -> Vec<MessageType> {
            vec![MessageType::CapabilityQuery, MessageType::HealthQuery]
        }

        fn handler_capabilities(&self) -> Vec<PrimalCapability> {
            vec![]
        }
    }

    #[tokio::test]
    async fn test_message_routing() {
        let config = CrossPrimalProtocolConfig::default();
        let coordinator = CrossPrimalCoordinator::new(config);

        // Register test handler
        let handler = Arc::new(TestMessageHandler::new());
        coordinator
            .register_handler(MessageType::CapabilityQuery, handler.clone())
            .await
            .unwrap();

        // Create test message
        let message = message_utils::create_capability_query(
            PrimalIdentity {
                primal_type: "test".to_string(),
                instance_id: "test-1".to_string(),
                version: "1.0.0".to_string(),
                endpoint: "localhost".to_string(),
                capabilities: vec![],
                health: PrimalHealth {
                    status: crate::HealthStatus::Healthy,
                    health_score: 100.0,
                    last_check: Utc::now(),
                    details: HashMap::new(),
                    metrics: create_health_metrics(),
                },
            },
            PrimalIdentity {
                primal_type: "target".to_string(),
                instance_id: "target-1".to_string(),
                version: "1.0.0".to_string(),
                endpoint: "localhost".to_string(),
                capabilities: vec![],
                health: PrimalHealth {
                    status: crate::HealthStatus::Healthy,
                    health_score: 100.0,
                    last_check: Utc::now(),
                    details: HashMap::new(),
                    metrics: create_health_metrics(),
                },
            },
            PrimalContext {
                user_id: "test".to_string(),
                device_id: "test-device".to_string(),
                session_id: Uuid::new_v4().to_string(),
                network_location: crate::NetworkLocation {
                    ip_address: "127.0.0.1".to_string(),
                    subnet: None,
                    network_id: None,
                    geo_location: None,
                },
                security_level: crate::SecurityLevel::Standard,
                biome_id: None,
                team_id: None,
                metadata: HashMap::new(),
            },
        );

        // Send message
        let response = coordinator.send_message(message).await.unwrap();

        // Verify response
        assert_eq!(response.status, ResponseStatus::Success);
        assert_eq!(handler.message_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_message_validation() {
        let config = CrossPrimalProtocolConfig::default();
        let coordinator = CrossPrimalCoordinator::new(config);

        // Test invalid message with TTL = 0
        let mut message = message_utils::create_capability_query(
            PrimalIdentity {
                primal_type: "test".to_string(),
                instance_id: "test-1".to_string(),
                version: "1.0.0".to_string(),
                endpoint: "localhost".to_string(),
                capabilities: vec![],
                health: PrimalHealth {
                    status: crate::HealthStatus::Healthy,
                    health_score: 100.0,
                    last_check: Utc::now(),
                    details: HashMap::new(),
                    metrics: create_health_metrics(),
                },
            },
            PrimalIdentity {
                primal_type: "target".to_string(),
                instance_id: "target-1".to_string(),
                version: "1.0.0".to_string(),
                endpoint: "localhost".to_string(),
                capabilities: vec![],
                health: PrimalHealth {
                    status: crate::HealthStatus::Healthy,
                    health_score: 100.0,
                    last_check: Utc::now(),
                    details: HashMap::new(),
                    metrics: create_health_metrics(),
                },
            },
            PrimalContext {
                user_id: "test".to_string(),
                device_id: "test-device".to_string(),
                session_id: Uuid::new_v4().to_string(),
                network_location: crate::NetworkLocation {
                    ip_address: "127.0.0.1".to_string(),
                    subnet: None,
                    network_id: None,
                    geo_location: None,
                },
                security_level: crate::SecurityLevel::Standard,
                biome_id: None,
                team_id: None,
                metadata: HashMap::new(),
            },
        );

        // Set invalid TTL
        message.ttl = 0;

        // Should fail validation
        let result = coordinator.send_message(message).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_broadcast_message() {
        let config = CrossPrimalProtocolConfig::default();
        let coordinator = CrossPrimalCoordinator::new(config);

        // Register test primals
        let primal1 = PrimalIdentity {
            primal_type: "test".to_string(),
            instance_id: "test-1".to_string(),
            version: "1.0.0".to_string(),
            endpoint: "localhost:8001".to_string(),
            capabilities: vec![],
            health: PrimalHealth {
                status: crate::HealthStatus::Healthy,
                health_score: 100.0,
                last_check: Utc::now(),
                details: HashMap::new(),
                metrics: create_health_metrics(),
            },
        };

        let primal2 = PrimalIdentity {
            primal_type: "test".to_string(),
            instance_id: "test-2".to_string(),
            version: "1.0.0".to_string(),
            endpoint: "localhost:8002".to_string(),
            capabilities: vec![],
            health: PrimalHealth {
                status: crate::HealthStatus::Healthy,
                health_score: 100.0,
                last_check: Utc::now(),
                details: HashMap::new(),
                metrics: create_health_metrics(),
            },
        };

        coordinator.register_primal(primal1).await.unwrap();
        coordinator.register_primal(primal2).await.unwrap();

        // Register handler
        let handler = Arc::new(TestMessageHandler::new());
        coordinator
            .register_handler(MessageType::CapabilityQuery, handler.clone())
            .await
            .unwrap();

        // Test broadcast
        let context = PrimalContext {
            user_id: "test".to_string(),
            device_id: "test-device".to_string(),
            session_id: Uuid::new_v4().to_string(),
            network_location: crate::NetworkLocation {
                ip_address: "127.0.0.1".to_string(),
                subnet: None,
                network_id: None,
                geo_location: None,
            },
            security_level: crate::SecurityLevel::Standard,
            biome_id: None,
            team_id: None,
            metadata: HashMap::new(),
        };

        let responses = coordinator
            .broadcast_message(
                MessageType::CapabilityQuery,
                serde_json::json!({"test": "broadcast"}),
                "test".to_string(),
                context,
            )
            .await
            .unwrap();

        // Should have 2 responses (one for each primal)
        assert_eq!(responses.len(), 2);
        assert_eq!(handler.message_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_response_status_equality() {
        let response = CrossPrimalResponse {
            id: Uuid::new_v4(),
            in_response_to: Uuid::new_v4(),
            status: ResponseStatus::Success,
            payload: serde_json::json!({}),
            processing_time: 0,
            metadata: HashMap::new(),
        };

        assert_eq!(response.status, ResponseStatus::Success);
    }

    #[tokio::test]
    async fn test_cross_primal_health_aggregation() {
        let config = CrossPrimalProtocolConfig::default();
        let coordinator = CrossPrimalCoordinator::new(config);

        // Mock multiple health responses
        let health1 = PrimalHealth {
            status: crate::HealthStatus::Healthy,
            health_score: 100.0,
            last_check: Utc::now(),
            details: HashMap::new(),
            metrics: create_health_metrics(),
        };

        let health2 = PrimalHealth {
            status: crate::HealthStatus::Healthy,
            health_score: 100.0,
            last_check: Utc::now(),
            details: HashMap::new(),
            metrics: create_health_metrics(),
        };

        coordinator
            .register_primal(PrimalIdentity {
                primal_type: "test".to_string(),
                instance_id: "test-1".to_string(),
                version: "1.0.0".to_string(),
                endpoint: "localhost".to_string(),
                capabilities: vec![],
                health: health1,
            })
            .await
            .unwrap();

        coordinator
            .register_primal(PrimalIdentity {
                primal_type: "test".to_string(),
                instance_id: "test-2".to_string(),
                version: "1.0.0".to_string(),
                endpoint: "localhost".to_string(),
                capabilities: vec![],
                health: health2,
            })
            .await
            .unwrap();

        // Register handler
        let handler = Arc::new(TestMessageHandler::new());
        coordinator
            .register_handler(MessageType::HealthQuery, handler.clone())
            .await
            .unwrap();

        // Test broadcast
        let context = PrimalContext {
            user_id: "test".to_string(),
            device_id: "test-device".to_string(),
            session_id: Uuid::new_v4().to_string(),
            network_location: crate::NetworkLocation {
                ip_address: "127.0.0.1".to_string(),
                subnet: None,
                network_id: None,
                geo_location: None,
            },
            security_level: crate::SecurityLevel::Standard,
            biome_id: None,
            team_id: None,
            metadata: HashMap::new(),
        };

        let responses = coordinator
            .broadcast_message(
                MessageType::HealthQuery,
                serde_json::json!({"test": "health_broadcast"}),
                "test".to_string(),
                context,
            )
            .await
            .unwrap();

        // Should have 2 responses (one for each primal)
        assert_eq!(responses.len(), 2);
        assert_eq!(handler.message_count.load(Ordering::SeqCst), 2);

        // Check aggregated health
        let aggregated_health = coordinator
            .primal_registry
            .read()
            .await
            .values()
            .map(|p| p.health.clone())
            .reduce(|acc, h| {
                let mut new_health = acc.clone();
                new_health.health_score = (new_health.health_score + h.health_score) / 2.0;
                new_health.last_check = Utc::now();
                new_health
            })
            .unwrap();

        assert_eq!(aggregated_health.status, crate::HealthStatus::Healthy);
        assert_eq!(aggregated_health.health_score, 100.0);
    }

    #[tokio::test]
    async fn test_cross_primal_message_routing() {
        let config = CrossPrimalProtocolConfig::default();
        let coordinator = CrossPrimalCoordinator::new(config);

        // Register test handler
        let handler = Arc::new(TestMessageHandler::new());
        coordinator
            .register_handler(MessageType::CapabilityQuery, handler.clone())
            .await
            .unwrap();

        // Create test message
        let message = message_utils::create_capability_query(
            PrimalIdentity {
                primal_type: "test".to_string(),
                instance_id: "test-1".to_string(),
                version: "1.0.0".to_string(),
                endpoint: "localhost".to_string(),
                capabilities: vec![],
                health: PrimalHealth {
                    status: crate::HealthStatus::Healthy,
                    health_score: 100.0,
                    last_check: Utc::now(),
                    details: HashMap::new(),
                    metrics: create_health_metrics(),
                },
            },
            PrimalIdentity {
                primal_type: "target".to_string(),
                instance_id: "target-1".to_string(),
                version: "1.0.0".to_string(),
                endpoint: "localhost".to_string(),
                capabilities: vec![],
                health: PrimalHealth {
                    status: crate::HealthStatus::Healthy,
                    health_score: 100.0,
                    last_check: Utc::now(),
                    details: HashMap::new(),
                    metrics: create_health_metrics(),
                },
            },
            PrimalContext {
                user_id: "test".to_string(),
                device_id: "test-device".to_string(),
                session_id: Uuid::new_v4().to_string(),
                network_location: crate::NetworkLocation {
                    ip_address: "127.0.0.1".to_string(),
                    subnet: None,
                    network_id: None,
                    geo_location: None,
                },
                security_level: crate::SecurityLevel::Standard,
                biome_id: None,
                team_id: None,
                metadata: HashMap::new(),
            },
        );

        // Send message
        let response = coordinator.send_message(message).await.unwrap();

        // Verify response
        assert_eq!(response.status, ResponseStatus::Success);
        assert_eq!(handler.message_count.load(Ordering::SeqCst), 1);
    }
}
