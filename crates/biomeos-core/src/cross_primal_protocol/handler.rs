//! Message handler trait and utilities for cross-primal protocol
//!
//! This module contains the message handler trait definition and utilities
//! for implementing cross-primal message handlers.

use async_trait::async_trait;
use crate::{BiomeResult, PrimalCapability};
use super::types::{CrossPrimalMessage, CrossPrimalResponse, MessageType};

/// Trait for cross-primal message handlers
#[async_trait]
pub trait CrossPrimalMessageHandler: Send + Sync {
    /// Handle incoming cross-primal message
    async fn handle_message(
        &self,
        message: CrossPrimalMessage,
    ) -> BiomeResult<CrossPrimalResponse>;

    /// Get supported message types
    fn supported_message_types(&self) -> Vec<MessageType>;

    /// Get handler capabilities
    fn handler_capabilities(&self) -> Vec<PrimalCapability>;

    /// Get handler name for identification
    fn handler_name(&self) -> &str;

    /// Check if handler can process a specific message type
    fn can_handle(&self, message_type: &MessageType) -> bool {
        self.supported_message_types().contains(message_type)
    }

    /// Get handler priority for message type
    fn handler_priority(&self, _message_type: &MessageType) -> u32 {
        0 // Default priority
    }

    /// Validate incoming message before processing
    async fn validate_message(&self, message: &CrossPrimalMessage) -> BiomeResult<()> {
        // Default validation - can be overridden
        if !self.can_handle(&message.message_type) {
            return Err(crate::BiomeError::RuntimeError(format!(
                "Handler {} cannot process message type {:?}",
                self.handler_name(),
                message.message_type
            )));
        }
        Ok(())
    }

    /// Pre-process message before handling
    async fn pre_process(&self, _message: &mut CrossPrimalMessage) -> BiomeResult<()> {
        // Default: no pre-processing
        Ok(())
    }

    /// Post-process response after handling
    async fn post_process(&self, _response: &mut CrossPrimalResponse) -> BiomeResult<()> {
        // Default: no post-processing
        Ok(())
    }
}

/// Handler registry for managing message handlers
pub struct HandlerRegistry {
    /// Map from message type to list of handlers
    handlers: std::collections::HashMap<MessageType, Vec<HandlerInfo>>,
}

/// Information about a registered handler
#[derive(Debug, Clone)]
pub struct HandlerInfo {
    /// Handler name
    pub name: String,
    /// Handler priority
    pub priority: u32,
    /// Supported capabilities
    pub capabilities: Vec<PrimalCapability>,
}

impl HandlerRegistry {
    /// Create a new handler registry
    pub fn new() -> Self {
        Self {
            handlers: std::collections::HashMap::new(),
        }
    }

    /// Register a handler for specific message types
    pub fn register_handler(
        &mut self,
        handler: &dyn CrossPrimalMessageHandler,
    ) -> BiomeResult<()> {
        let handler_info = HandlerInfo {
            name: handler.handler_name().to_string(),
            priority: 0, // Default priority, can be customized
            capabilities: handler.handler_capabilities(),
        };

        for message_type in handler.supported_message_types() {
            self.handlers
                .entry(message_type.clone())
                .or_insert_with(Vec::new)
                .push(handler_info.clone());
        }

        // Sort handlers by priority (higher priority first)
        for handlers in self.handlers.values_mut() {
            handlers.sort_by(|a, b| b.priority.cmp(&a.priority));
        }

        Ok(())
    }

    /// Get handlers for a specific message type
    pub fn get_handlers(&self, message_type: &MessageType) -> Vec<&HandlerInfo> {
        self.handlers
            .get(message_type)
            .map(|handlers| handlers.iter().collect())
            .unwrap_or_default()
    }

    /// Check if any handler can process a message type
    pub fn can_handle(&self, message_type: &MessageType) -> bool {
        self.handlers.contains_key(message_type) && !self.handlers[message_type].is_empty()
    }

    /// Get all registered message types
    pub fn supported_message_types(&self) -> Vec<MessageType> {
        self.handlers.keys().cloned().collect()
    }

    /// Get handler statistics
    pub fn get_statistics(&self) -> HandlerStatistics {
        let total_handlers = self.handlers.values().map(|v| v.len()).sum();
        let total_message_types = self.handlers.len();
        let avg_handlers_per_type = if total_message_types > 0 {
            total_handlers as f64 / total_message_types as f64
        } else {
            0.0
        };

        HandlerStatistics {
            total_handlers,
            total_message_types,
            avg_handlers_per_type,
        }
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about handler registry
#[derive(Debug, Clone)]
pub struct HandlerStatistics {
    /// Total number of handlers
    pub total_handlers: usize,
    /// Total number of message types supported
    pub total_message_types: usize,
    /// Average handlers per message type
    pub avg_handlers_per_type: f64,
}

/// Base handler implementation for common functionality
pub struct BaseHandler {
    /// Handler name
    pub name: String,
    /// Supported message types
    pub supported_types: Vec<MessageType>,
    /// Handler capabilities
    pub capabilities: Vec<PrimalCapability>,
}

impl BaseHandler {
    /// Create a new base handler
    pub fn new(
        name: String,
        supported_types: Vec<MessageType>,
        capabilities: Vec<PrimalCapability>,
    ) -> Self {
        Self {
            name,
            supported_types,
            capabilities,
        }
    }
}

#[async_trait]
impl CrossPrimalMessageHandler for BaseHandler {
    async fn handle_message(
        &self,
        _message: CrossPrimalMessage,
    ) -> BiomeResult<CrossPrimalResponse> {
        // Default implementation - should be overridden
        Err(crate::BiomeError::RuntimeError(
            "Base handler should be overridden".to_string(),
        ))
    }

    fn supported_message_types(&self) -> Vec<MessageType> {
        self.supported_types.clone()
    }

    fn handler_capabilities(&self) -> Vec<PrimalCapability> {
        self.capabilities.clone()
    }

    fn handler_name(&self) -> &str {
        &self.name
    }
}

/// Utility functions for message handling
pub mod utils {
    use super::*;
    use crate::BiomeError;

    /// Create a standard success response
    pub fn create_success_response(
        original_message_id: uuid::Uuid,
        payload: serde_json::Value,
    ) -> CrossPrimalResponse {
        CrossPrimalResponse {
            id: uuid::Uuid::new_v4(),
            in_response_to: original_message_id,
            status: super::super::types::ResponseStatus::Success,
            payload,
            processing_time: 0, // Should be set by the handler
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create a standard error response
    pub fn create_error_response(
        original_message_id: uuid::Uuid,
        error: &BiomeError,
    ) -> CrossPrimalResponse {
        CrossPrimalResponse {
            id: uuid::Uuid::new_v4(),
            in_response_to: original_message_id,
            status: super::super::types::ResponseStatus::Error,
            payload: serde_json::json!({
                "error": error.to_string()
            }),
            processing_time: 0,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Validate message size
    pub fn validate_message_size(message: &CrossPrimalMessage, max_size: usize) -> BiomeResult<()> {
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

    /// Check if message has expired
    pub fn is_message_expired(message: &CrossPrimalMessage) -> bool {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        current_time > message.timestamp + message.ttl
    }
} 