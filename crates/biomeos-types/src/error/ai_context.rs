//! AI Error Context and Automation Features
//!
//! This module contains AI-specific error context, retry strategies,
//! suggested actions, and automation features for intelligent error handling.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// AI-specific error context for automation and decision making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIErrorContext {
    /// Unique error identifier for tracking
    pub error_id: Uuid,

    /// When the error occurred
    pub timestamp: DateTime<Utc>,

    /// Error category for AI classification
    pub category: AIErrorCategory,

    /// Automated retry strategy
    pub retry_strategy: RetryStrategy,

    /// Actionable hints for AI automation
    pub automation_hints: Vec<String>,

    /// Severity level for prioritization
    pub severity: ErrorSeverity,

    /// Whether human intervention is required
    pub requires_human_intervention: bool,

    /// Related error context for debugging
    pub context: HashMap<String, serde_json::Value>,

    /// Confidence score for automated handling (0.0-1.0)
    pub automation_confidence: f64,

    /// Suggested next actions for AI agents
    pub suggested_actions: Vec<SuggestedAction>,

    /// Error correlation ID for tracking related errors
    pub correlation_id: Option<Uuid>,

    /// Component or service that generated the error
    pub source_component: Option<String>,
}

/// Error categories for AI classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AIErrorCategory {
    /// Insufficient computational resources
    ResourceLimitation,

    /// Configuration or parameter issues
    ConfigurationIssue,

    /// Authentication or authorization failures
    SecurityViolation,

    /// Network connectivity problems
    NetworkFailure,

    /// Runtime execution errors
    RuntimeError,

    /// Requires human decision or input
    HumanInterventionRequired,

    /// External dependency failures
    DependencyFailure,

    /// Rate limiting or throttling
    RateLimiting,

    /// Data validation or integrity issues
    DataIssue,

    /// Service unavailable or degraded
    ServiceUnavailable,

    /// Internal system errors
    SystemError,

    /// User input errors
    UserError,
}

/// Automated retry strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryStrategy {
    /// Whether automatic retry is recommended
    pub should_retry: bool,

    /// Initial delay in milliseconds
    pub delay_ms: u64,

    /// Maximum retry attempts
    pub max_attempts: u32,

    /// Backoff strategy type
    pub backoff_strategy: BackoffType,

    /// Conditions that must be met for retry
    pub retry_conditions: Vec<String>,

    /// Estimated success probability for retry (0.0-1.0)
    pub success_probability: f64,

    /// Maximum total retry time in milliseconds
    pub max_retry_time_ms: Option<u64>,
}

/// Backoff strategy types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffType {
    /// Linear backoff with fixed increment
    Linear {
        /// Increment in milliseconds
        increment_ms: u64,
    },

    /// Exponential backoff
    Exponential {
        /// Exponential base multiplier
        base: f64,
        /// Maximum delay in milliseconds
        max_delay_ms: u64,
    },

    /// Fibonacci backoff
    Fibonacci {
        /// Maximum delay in milliseconds
        max_delay_ms: u64,
    },

    /// Custom backoff with predefined delays
    Custom {
        /// Sequence of delay values in milliseconds
        delays_ms: Vec<u64>,
    },
}

/// Error severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Informational - no action required
    Info,

    /// Warning - action may be required
    Warning,

    /// Error - action required
    Error,

    /// Critical - immediate action required
    Critical,

    /// Emergency - system failure imminent
    Emergency,
}

/// Suggested actions for AI automation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedAction {
    /// Action identifier
    pub id: String,

    /// Action type
    pub action_type: ActionType,

    /// Human-readable description
    pub description: String,

    /// Whether this action can be automated
    pub automated: bool,

    /// Confidence in action success (0.0-1.0)
    pub confidence: f64,

    /// Estimated time to complete in seconds
    pub estimated_duration_secs: Option<u32>,

    /// Prerequisites for this action
    pub prerequisites: Vec<String>,

    /// Expected outcome
    pub expected_outcome: String,

    /// Risk level of this action
    pub risk_level: ActionRiskLevel,
}

/// Types of suggested actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    /// Retry the failed operation
    Retry,

    /// Restart a service or component
    Restart,

    /// Update configuration
    UpdateConfig,

    /// Scale resources
    Scale,

    /// Fallback to alternative method
    Fallback,

    /// Request human intervention
    RequestHuman,

    /// Log additional information
    LogMore,

    /// Check system health
    HealthCheck,

    /// Custom action
    Custom {
        /// Custom action type identifier
        action_type: String,
    },
}

/// Risk levels for actions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionRiskLevel {
    /// No risk - safe to execute automatically
    None,

    /// Low risk - minimal impact if fails
    Low,

    /// Medium risk - some impact if fails
    Medium,

    /// High risk - significant impact if fails
    High,

    /// Critical risk - requires human approval
    Critical,
}

impl Default for AIErrorContext {
    fn default() -> Self {
        Self {
            error_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            category: AIErrorCategory::UserError,
            retry_strategy: RetryStrategy::default(),
            automation_hints: Vec::new(),
            severity: ErrorSeverity::Error,
            requires_human_intervention: false,
            context: HashMap::new(),
            automation_confidence: 0.5,
            suggested_actions: Vec::new(),
            correlation_id: None,
            source_component: None,
        }
    }
}

impl AIErrorContext {
    /// Create new AI error context
    pub fn new(category: AIErrorCategory) -> Self {
        Self {
            error_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            category,
            retry_strategy: RetryStrategy::default(),
            automation_hints: vec![],
            severity: ErrorSeverity::Error,
            requires_human_intervention: false,
            context: HashMap::new(),
            automation_confidence: 0.5,
            suggested_actions: vec![],
            correlation_id: None,
            source_component: None,
        }
    }

    /// Create with retry strategy
    pub fn with_retry(category: AIErrorCategory, retry_strategy: RetryStrategy) -> Self {
        Self {
            retry_strategy,
            ..Self::new(category)
        }
    }

    /// Add automation hint
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.automation_hints.push(hint.into());
        self
    }

    /// Set severity
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set human intervention requirement
    pub fn requires_human(mut self) -> Self {
        self.requires_human_intervention = true;
        self.automation_confidence = 0.0;
        self
    }
}

impl RetryStrategy {
    /// Create default retry strategy (no retry)
    pub fn no_retry() -> Self {
        Self {
            should_retry: false,
            delay_ms: 0,
            max_attempts: 0,
            backoff_strategy: BackoffType::Linear { increment_ms: 1000 },
            retry_conditions: vec![],
            success_probability: 0.0,
            max_retry_time_ms: None,
        }
    }

    /// Create exponential backoff retry strategy
    pub fn exponential_backoff(
        max_attempts: u32,
        initial_delay_ms: u64,
        max_delay_ms: u64,
    ) -> Self {
        Self {
            should_retry: true,
            delay_ms: initial_delay_ms,
            max_attempts,
            backoff_strategy: BackoffType::Exponential {
                base: 2.0,
                max_delay_ms,
            },
            retry_conditions: vec![],
            success_probability: 0.7,
            max_retry_time_ms: Some(max_delay_ms * max_attempts as u64),
        }
    }

    /// Create linear backoff retry strategy
    pub fn linear_backoff(max_attempts: u32, delay_ms: u64, increment_ms: u64) -> Self {
        Self {
            should_retry: true,
            delay_ms,
            max_attempts,
            backoff_strategy: BackoffType::Linear { increment_ms },
            retry_conditions: vec![],
            success_probability: 0.6,
            max_retry_time_ms: Some(
                (delay_ms + increment_ms * max_attempts as u64) * max_attempts as u64,
            ),
        }
    }
}

impl Default for RetryStrategy {
    fn default() -> Self {
        Self::no_retry()
    }
}
