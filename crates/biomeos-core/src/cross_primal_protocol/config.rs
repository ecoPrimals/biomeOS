//! Configuration for the cross-primal protocol
//!
//! This module contains all configuration structures and their default implementations
//! for the cross-primal communication protocol.

use serde::{Deserialize, Serialize};
use super::types::RetryConfig;

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
            default_retry: RetryConfig::default(),
            protocol_timeout: 30, // 30 seconds
            enable_encryption: true,
            queue_limits: QueueLimits::default(),
        }
    }
}

impl Default for QueueLimits {
    fn default() -> Self {
        Self {
            max_pending: 1000,
            max_processing_time: 60, // 1 minute
            max_queue_per_priority: 200,
        }
    }
}

impl CrossPrimalProtocolConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration for development environment
    pub fn development() -> Self {
        Self {
            default_ttl: 60,               // 1 minute
            max_message_size: 512 * 1024,  // 512 KB
            default_retry: RetryConfig {
                max_attempts: 2,
                retry_delay: 500, // 0.5 seconds
                backoff_factor: 1.5,
            },
            protocol_timeout: 10, // 10 seconds
            enable_encryption: false,
            queue_limits: QueueLimits {
                max_pending: 100,
                max_processing_time: 30, // 30 seconds
                max_queue_per_priority: 50,
            },
        }
    }

    /// Create a configuration for production environment
    pub fn production() -> Self {
        Self {
            default_ttl: 600,              // 10 minutes
            max_message_size: 2048 * 1024, // 2 MB
            default_retry: RetryConfig {
                max_attempts: 5,
                retry_delay: 2000, // 2 seconds
                backoff_factor: 2.5,
            },
            protocol_timeout: 60, // 1 minute
            enable_encryption: true,
            queue_limits: QueueLimits {
                max_pending: 5000,
                max_processing_time: 120, // 2 minutes
                max_queue_per_priority: 1000,
            },
        }
    }

    /// Create a configuration for high-throughput scenarios
    pub fn high_throughput() -> Self {
        Self {
            default_ttl: 120,             // 2 minutes
            max_message_size: 256 * 1024, // 256 KB
            default_retry: RetryConfig {
                max_attempts: 2,
                retry_delay: 100, // 0.1 seconds
                backoff_factor: 1.2,
            },
            protocol_timeout: 5, // 5 seconds
            enable_encryption: false,
            queue_limits: QueueLimits {
                max_pending: 10000,
                max_processing_time: 10, // 10 seconds
                max_queue_per_priority: 2000,
            },
        }
    }

    /// Set the default TTL
    pub fn with_default_ttl(mut self, ttl: u64) -> Self {
        self.default_ttl = ttl;
        self
    }

    /// Set the maximum message size
    pub fn with_max_message_size(mut self, size: usize) -> Self {
        self.max_message_size = size;
        self
    }

    /// Set the default retry configuration
    pub fn with_default_retry(mut self, retry: RetryConfig) -> Self {
        self.default_retry = retry;
        self
    }

    /// Set the protocol timeout
    pub fn with_protocol_timeout(mut self, timeout: u64) -> Self {
        self.protocol_timeout = timeout;
        self
    }

    /// Enable or disable encryption
    pub fn with_encryption(mut self, enabled: bool) -> Self {
        self.enable_encryption = enabled;
        self
    }

    /// Set queue limits
    pub fn with_queue_limits(mut self, limits: QueueLimits) -> Self {
        self.queue_limits = limits;
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.default_ttl == 0 {
            return Err("Default TTL must be greater than 0".to_string());
        }

        if self.max_message_size == 0 {
            return Err("Maximum message size must be greater than 0".to_string());
        }

        if self.protocol_timeout == 0 {
            return Err("Protocol timeout must be greater than 0".to_string());
        }

        if self.default_retry.max_attempts == 0 {
            return Err("Retry max attempts must be greater than 0".to_string());
        }

        if self.queue_limits.max_pending == 0 {
            return Err("Max pending messages must be greater than 0".to_string());
        }

        if self.queue_limits.max_processing_time == 0 {
            return Err("Max processing time must be greater than 0".to_string());
        }

        if self.queue_limits.max_queue_per_priority == 0 {
            return Err("Max queue per priority must be greater than 0".to_string());
        }

        Ok(())
    }
}

impl QueueLimits {
    /// Create new queue limits
    pub fn new(max_pending: usize, max_processing_time: u64, max_queue_per_priority: usize) -> Self {
        Self {
            max_pending,
            max_processing_time,
            max_queue_per_priority,
        }
    }

    /// Create limits for low-resource environments
    pub fn low_resource() -> Self {
        Self {
            max_pending: 50,
            max_processing_time: 15, // 15 seconds
            max_queue_per_priority: 10,
        }
    }

    /// Create limits for high-resource environments
    pub fn high_resource() -> Self {
        Self {
            max_pending: 10000,
            max_processing_time: 300, // 5 minutes
            max_queue_per_priority: 2000,
        }
    }

    /// Set max pending messages
    pub fn with_max_pending(mut self, max_pending: usize) -> Self {
        self.max_pending = max_pending;
        self
    }

    /// Set max processing time
    pub fn with_max_processing_time(mut self, max_processing_time: u64) -> Self {
        self.max_processing_time = max_processing_time;
        self
    }

    /// Set max queue per priority
    pub fn with_max_queue_per_priority(mut self, max_queue_per_priority: usize) -> Self {
        self.max_queue_per_priority = max_queue_per_priority;
        self
    }
} 