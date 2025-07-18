//! Core types and data structures for cross-primal protocol
//!
//! This module contains all the core message types, enums, and data structures
//! used in the cross-primal communication protocol.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{PrimalCapability, PrimalContext, PrimalHealth, SecurityLevel};

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
    pub security_level: SecurityLevel,
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// Protocol statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolStatistics {
    /// Total messages sent
    pub messages_sent: u64,
    /// Total messages received
    pub messages_received: u64,
    /// Total responses sent
    pub responses_sent: u64,
    /// Total responses received
    pub responses_received: u64,
    /// Total errors encountered
    pub errors: u64,
    /// Total timeouts
    pub timeouts: u64,
    /// Average processing time in milliseconds
    pub avg_processing_time: f64,
    /// Success rate percentage
    pub success_rate: f64,
}

impl Default for MessagePriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            retry_delay: 1000, // 1 second
            backoff_factor: 2.0,
        }
    }
}

impl Default for ProtocolStatistics {
    fn default() -> Self {
        Self {
            messages_sent: 0,
            messages_received: 0,
            responses_sent: 0,
            responses_received: 0,
            errors: 0,
            timeouts: 0,
            avg_processing_time: 0.0,
            success_rate: 100.0,
        }
    }
}

impl MessageType {
    /// Get all available message types
    pub fn all() -> Vec<Self> {
        vec![
            Self::CapabilityQuery,
            Self::CapabilityResponse,
            Self::ServiceDiscovery,
            Self::ServiceRegistration,
            Self::HealthQuery,
            Self::HealthReport,
            Self::ResourceRequest,
            Self::ResourceResponse,
            Self::ResourceAllocation,
            Self::AuthenticationRequest,
            Self::AuthenticationResponse,
            Self::AuthorizationCheck,
            Self::DataRequest,
            Self::DataResponse,
            Self::DataTransfer,
            Self::ComputeRequest,
            Self::ComputeResponse,
            Self::ComputeResult,
            Self::CoordinationRequest,
            Self::CoordinationResponse,
            Self::CoordinationUpdate,
            Self::ErrorReport,
            Self::StatusUpdate,
        ]
    }

    /// Check if this is a query message type
    pub fn is_query(&self) -> bool {
        matches!(
            self,
            Self::CapabilityQuery
                | Self::ServiceDiscovery
                | Self::HealthQuery
                | Self::ResourceRequest
                | Self::AuthenticationRequest
                | Self::AuthorizationCheck
                | Self::DataRequest
                | Self::ComputeRequest
                | Self::CoordinationRequest
        )
    }

    /// Check if this is a response message type
    pub fn is_response(&self) -> bool {
        matches!(
            self,
            Self::CapabilityResponse
                | Self::ServiceRegistration
                | Self::HealthReport
                | Self::ResourceResponse
                | Self::ResourceAllocation
                | Self::AuthenticationResponse
                | Self::DataResponse
                | Self::ComputeResponse
                | Self::ComputeResult
                | Self::CoordinationResponse
                | Self::CoordinationUpdate
        )
    }
}

impl ResponseStatus {
    /// Check if status indicates success
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success | Self::PartialSuccess)
    }

    /// Check if status indicates failure
    pub fn is_failure(&self) -> bool {
        !self.is_success()
    }

    /// Check if status indicates a retryable error
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Timeout | Self::Unavailable | Self::RateLimited | Self::Degraded
        )
    }
} 