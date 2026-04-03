// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Traffic management: splitting, circuit breaking, rate limits, retries, and backoff.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Traffic management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficManagement {
    /// Traffic splitting
    pub traffic_splitting: Option<TrafficSplitting>,

    /// Circuit breaker
    pub circuit_breaker: Option<CircuitBreaker>,

    /// Rate limiting
    pub rate_limiting: Option<RateLimiting>,

    /// Timeout configuration
    pub timeouts: Option<TrafficTimeouts>,

    /// Retry configuration
    pub retries: Option<RetryConfig>,
}

/// Traffic splitting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficSplitting {
    /// Traffic routes
    pub routes: Vec<TrafficRoute>,

    /// Default route
    pub default_route: Option<String>,

    /// Splitting strategy
    pub strategy: SplittingStrategy,
}

/// Traffic route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficRoute {
    /// Route name
    pub name: String,

    /// Target service
    pub target: String,

    /// Traffic weight (percentage)
    pub weight: u32,

    /// Route conditions
    pub conditions: Vec<RouteCondition>,
}

/// Route condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RouteCondition {
    /// Header condition
    Header {
        /// Header name
        name: String,
        /// Expected value
        value: String,
        /// Comparison operator
        operator: ConditionOperator,
    },

    /// Path condition
    Path {
        /// Path pattern
        pattern: String,
        /// Comparison operator
        operator: ConditionOperator,
    },

    /// Query parameter condition
    QueryParam {
        /// Parameter name
        name: String,
        /// Expected value
        value: String,
        /// Comparison operator
        operator: ConditionOperator,
    },

    /// Custom condition
    Custom {
        /// Condition type identifier
        condition_type: String,
        /// Additional configuration
        config: HashMap<String, String>,
    },
}

/// Condition operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    /// Exact equality
    Equals,
    /// Not equal
    NotEquals,
    /// Contains substring
    Contains,
    /// Does not contain substring
    NotContains,
    /// Starts with prefix
    StartsWith,
    /// Ends with suffix
    EndsWith,
    /// Matches regex pattern
    Matches,
}

/// Traffic splitting strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SplittingStrategy {
    /// Weighted round-robin
    WeightedRoundRobin,
    /// Canary deployment
    Canary,
    /// Blue-green deployment
    BlueGreen,
    /// A/B testing
    AbTesting,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreaker {
    /// Failure threshold
    pub failure_threshold: u32,

    /// Recovery timeout (seconds)
    pub recovery_timeout: u32,

    /// Request volume threshold
    pub request_volume_threshold: u32,

    /// Error rate threshold (percentage)
    pub error_rate_threshold: f64,

    /// Sleep window (seconds)
    pub sleep_window: u32,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiting {
    /// Rate limit rules
    pub rules: Vec<RateLimitRule>,

    /// Default rate limit
    pub default_limit: Option<RateLimit>,

    /// Rate limiting strategy
    pub strategy: RateLimitStrategy,
}

/// Rate limit rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitRule {
    /// Rule name
    pub name: String,

    /// Rule conditions
    pub conditions: Vec<RateLimitCondition>,

    /// Rate limit
    pub limit: RateLimit,

    /// Rule priority
    pub priority: u32,
}

/// Rate limit condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitCondition {
    /// Client IP condition
    ClientIp(String),

    /// User ID condition
    UserId(String),

    /// API key condition
    ApiKey(String),

    /// Custom condition
    Custom {
        /// Condition type identifier
        condition_type: String,
        /// Condition value
        value: String,
    },
}

/// Rate limit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Requests per time window
    pub requests: u32,

    /// Time window (seconds)
    pub window: u32,

    /// Burst size
    pub burst: Option<u32>,
}

/// Rate limiting strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitStrategy {
    /// Token bucket
    TokenBucket,
    /// Leaky bucket
    LeakyBucket,
    /// Fixed window
    FixedWindow,
    /// Sliding window
    SlidingWindow,
}

/// Traffic timeouts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficTimeouts {
    /// Request timeout (seconds)
    pub request: u32,

    /// Response timeout (seconds)
    pub response: u32,

    /// Connection timeout (seconds)
    pub connection: u32,

    /// Idle timeout (seconds)
    pub idle: u32,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,

    /// Retry timeout (seconds)
    pub timeout: u32,

    /// Retry conditions
    pub conditions: Vec<RetryCondition>,

    /// Backoff strategy
    pub backoff: BackoffStrategy,
}

/// Retry conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryCondition {
    /// HTTP status code
    HttpStatus(u16),

    /// Connection error
    ConnectionError,

    /// Timeout error
    TimeoutError,

    /// Custom condition
    Custom(String),
}

/// Backoff strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    /// Fixed backoff
    Fixed {
        /// Delay in seconds
        delay: u32,
    },

    /// Exponential backoff
    Exponential {
        /// Base delay in seconds
        base_delay: u32,
        /// Maximum delay cap in seconds
        max_delay: u32,
        /// Multiplier per attempt
        multiplier: f64,
    },

    /// Linear backoff
    Linear {
        /// Base delay in seconds
        base_delay: u32,
        /// Linear increment per attempt
        increment: u32,
    },

    /// Random backoff
    Random {
        /// Minimum delay in seconds
        min_delay: u32,
        /// Maximum delay in seconds
        max_delay: u32,
    },
}
