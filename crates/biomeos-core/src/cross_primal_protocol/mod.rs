//! Cross-Primal Protocol Implementation
//!
//! This module implements the standardized communication protocol for all primals
//! in the biomeOS ecosystem, providing universal message formats, routing,
//! and coordination capabilities.

pub mod types;
pub mod config;
pub mod handler;
pub mod coordinator;
pub mod utils;

// Re-export the main types and traits
pub use types::{
    CrossPrimalMessage, CrossPrimalResponse, MessageType, MessagePriority, PrimalIdentity,
    ResponseStatus, RoutingStrategy, MessageRouting, MessageSecurity, RetryConfig,
    EncryptionMetadata, ProtocolStatistics,
};

pub use config::{CrossPrimalProtocolConfig, QueueLimits};

pub use handler::{
    CrossPrimalMessageHandler, HandlerRegistry, HandlerInfo, HandlerStatistics, BaseHandler,
};

pub use coordinator::{CrossPrimalCoordinator, CoordinatorStatus};

// Re-export utility modules
pub use utils::{message_utils, response_utils, validation_utils, routing_utils};

// Re-export commonly used functions
pub use utils::message_utils::{
    create_capability_query, create_health_query, create_resource_request,
    create_service_discovery, create_authentication_request, create_compute_request,
    create_coordination_request,
};

pub use utils::response_utils::{
    create_success_response, create_error_response, create_timeout_response,
    create_rejected_response, create_partial_success_response,
};

pub use utils::validation_utils::{
    validate_message_format, is_message_expired, validate_message_size,
    validate_response_format,
};

pub use utils::routing_utils::{
    get_routing_path, add_routing_hop, should_retry, calculate_retry_delay,
    matches_routing_strategy,
}; 