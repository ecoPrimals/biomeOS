//! Core Error Types
//!
//! This module contains the main BiomeError enum and core error types
//! that form the foundation of the unified error handling system.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::ai_context::AIErrorContext;
use super::operations::{NetworkOperation, SecurityViolationType, ResourceOperation, DataOperation};

/// Universal biomeOS Error
/// 
/// This consolidates all error types from across the ecosystem into a single,
/// comprehensive error system that supports both human and AI interaction.
#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum BiomeError {
    /// Configuration error with context
    #[error("Configuration error: {message}")]
    Configuration {
        /// Error message
        message: String,
        /// Configuration key that caused the error
        key: Option<String>,
        /// Configuration file path
        config_path: Option<String>,
        /// AI-specific context
        #[serde(flatten)]
        ai_context: AIErrorContext,
    },

    /// Invalid input error
    #[error("Invalid input: {message}")]
    InvalidInput {
        /// Error message
        message: String,
        /// Input field that was invalid
        field: Option<String>,
        /// Expected format or value
        expected: Option<String>,
        /// Actual value received
        actual: Option<String>,
        /// AI-specific context
        #[serde(flatten)]
        ai_context: AIErrorContext,
    },

    /// Primal discovery error
    #[error("Discovery error: {message}")]
    Discovery {
        /// Error message
        message: String,
        /// Endpoint that failed discovery
        endpoint: Option<String>,
        /// HTTP status code if applicable
        status_code: Option<u16>,
        /// Discovery method used
        discovery_method: Option<String>,
        /// AI-specific context
        #[serde(flatten)]
        ai_context: AIErrorContext,
    },

    /// Network communication error
    #[error("Network error: {message}")]
    Network {
        /// Error message
        message: String,
        /// Endpoint that failed
        endpoint: Option<String>,
        /// HTTP status code
        status_code: Option<u16>,
        /// Timeout duration if applicable
        timeout_ms: Option<u64>,
        /// Network operation type
        operation: Option<NetworkOperation>,
        /// AI-specific context
        #[serde(flatten)]
        ai_context: AIErrorContext,
    },

    /// Security/authentication error
    #[error("Security error: {message}")]
    Security {
        /// Error message
        message: String,
        /// Security context
        context: Option<String>,
        /// Authentication method used
        auth_method: Option<String>,
        /// Security violation type
        violation_type: Option<SecurityViolationType>,
        /// AI-specific context
        #[serde(flatten)]
        ai_context: AIErrorContext,
    },

    /// Resource management error
    #[error("Resource error: {message}")]
    Resource {
        /// Error message
        message: String,
        /// Resource type (cpu, memory, disk, etc.)
        resource_type: Option<String>,
        /// Requested amount
        requested: Option<String>,
        /// Available amount
        available: Option<String>,
        /// Resource operation
        operation: Option<ResourceOperation>,
        /// AI-specific context
        #[serde(flatten)]
        ai_context: AIErrorContext,
    },

    /// Integration error
    #[error("Integration error: {message}")]
    Integration {
        /// Error message
        message: String,
        /// Component that failed integration
        component: Option<String>,
        /// Integration type
        integration_type: Option<String>,
        /// AI-specific context
        #[serde(flatten)]
        ai_context: AIErrorContext,
    },

    /// Internal system error
    #[error("Internal error: {message}")]
    Internal {
        /// Error message
        message: String,
        /// Internal error code
        error_code: Option<String>,
        /// Stack trace if available
        stack_trace: Option<String>,
        /// AI-specific context
        #[serde(flatten)]
        ai_context: AIErrorContext,
    },

    /// Timeout error
    #[error("Timeout error: {message}")]
    Timeout {
        /// Error message
        message: String,
        /// Timeout duration in milliseconds
        timeout_ms: u64,
        /// Operation that timed out
        operation: Option<String>,
        /// AI-specific context
        #[serde(flatten)]
        ai_context: AIErrorContext,
    },

    /// Authorization error
    #[error("Authorization error: {message}")]
    Authorization {
        /// Error message
        message: String,
        /// Required permission
        required_permission: Option<String>,
        /// User or service identifier
        subject: Option<String>,
        /// AI-specific context
        #[serde(flatten)]
        ai_context: AIErrorContext,
    },

    /// Validation error
    #[error("Validation error: {message}")]
    Validation {
        /// Error message
        message: String,
        /// Field that failed validation
        field: Option<String>,
        /// Validation rule that failed
        rule: Option<String>,
        /// All validation errors
        errors: Vec<ValidationError>,
        /// AI-specific context
        #[serde(flatten)]
        ai_context: AIErrorContext,
    },

    /// External service error
    #[error("External service error: {message}")]
    ExternalService {
        /// Error message
        message: String,
        /// Service name
        service: Option<String>,
        /// Service endpoint
        endpoint: Option<String>,
        /// HTTP status code
        status_code: Option<u16>,
        /// AI-specific context
        #[serde(flatten)]
        ai_context: AIErrorContext,
    },

    /// Data error (corruption, inconsistency, etc.)
    #[error("Data error: {message}")]
    Data {
        /// Error message
        message: String,
        /// Data type or table affected
        data_type: Option<String>,
        /// Data identifier
        data_id: Option<String>,
        /// Operation that failed
        operation: Option<DataOperation>,
        /// AI-specific context
        #[serde(flatten)]
        ai_context: AIErrorContext,
    },

    /// Unknown error
    #[error("Unknown error: {message}")]
    Unknown {
        /// Error message
        message: String,
        /// AI-specific context
        #[serde(flatten)]
        ai_context: AIErrorContext,
    },
}

/// Individual validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Field that failed validation
    pub field: String,
    
    /// Error message
    pub message: String,
    
    /// Validation code
    pub code: String,
    
    /// Rejected value
    pub rejected_value: Option<serde_json::Value>,
} 