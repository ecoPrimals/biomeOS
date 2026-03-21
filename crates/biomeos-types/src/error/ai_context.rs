// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
    #[must_use]
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.automation_hints.push(hint.into());
        self
    }

    /// Set severity
    #[must_use]
    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set human intervention requirement
    #[must_use]
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
    #[expect(
        clippy::cast_lossless,
        reason = "u32 max_attempts widened to u64 for max_retry_time_ms arithmetic"
    )]
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
    #[expect(
        clippy::cast_lossless,
        reason = "u32 max_attempts widened to u64 for max_retry_time_ms arithmetic"
    )]
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

#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_error_context_default() {
        let ctx = AIErrorContext::default();
        assert_eq!(ctx.category, AIErrorCategory::UserError);
        assert_eq!(ctx.severity, ErrorSeverity::Error);
        assert!(!ctx.requires_human_intervention);
        assert!((ctx.automation_confidence - 0.5).abs() < f64::EPSILON);
        assert!(ctx.automation_hints.is_empty());
        assert!(ctx.suggested_actions.is_empty());
    }

    #[test]
    fn test_ai_error_context_new() {
        let ctx = AIErrorContext::new(AIErrorCategory::NetworkFailure);
        assert_eq!(ctx.category, AIErrorCategory::NetworkFailure);
        assert_eq!(ctx.severity, ErrorSeverity::Error);
    }

    #[test]
    fn test_ai_error_context_with_retry() {
        let strategy = RetryStrategy::exponential_backoff(3, 100, 5000);
        let ctx = AIErrorContext::with_retry(AIErrorCategory::RateLimiting, strategy);
        assert_eq!(ctx.category, AIErrorCategory::RateLimiting);
        assert!(ctx.retry_strategy.should_retry);
        assert_eq!(ctx.retry_strategy.max_attempts, 3);
    }

    #[test]
    fn test_ai_error_context_with_hint() {
        let ctx = AIErrorContext::new(AIErrorCategory::ConfigurationIssue)
            .with_hint("Check BIND_ADDRESS env var");
        assert_eq!(ctx.automation_hints.len(), 1);
        assert_eq!(ctx.automation_hints[0], "Check BIND_ADDRESS env var");
    }

    #[test]
    fn test_ai_error_context_with_severity() {
        let ctx = AIErrorContext::new(AIErrorCategory::SystemError)
            .with_severity(ErrorSeverity::Critical);
        assert_eq!(ctx.severity, ErrorSeverity::Critical);
    }

    #[test]
    fn test_ai_error_context_requires_human() {
        let ctx = AIErrorContext::new(AIErrorCategory::HumanInterventionRequired).requires_human();
        assert!(ctx.requires_human_intervention);
        assert!((ctx.automation_confidence - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_retry_strategy_no_retry() {
        let s = RetryStrategy::no_retry();
        assert!(!s.should_retry);
        assert_eq!(s.max_attempts, 0);
        assert_eq!(s.delay_ms, 0);
    }

    #[test]
    fn test_retry_strategy_exponential_backoff() {
        let s = RetryStrategy::exponential_backoff(5, 200, 10000);
        assert!(s.should_retry);
        assert_eq!(s.max_attempts, 5);
        assert_eq!(s.delay_ms, 200);
        assert!((s.success_probability - 0.7).abs() < f64::EPSILON);
        assert!(s.max_retry_time_ms.is_some());
        if let BackoffType::Exponential { base, max_delay_ms } = s.backoff_strategy {
            assert!((base - 2.0).abs() < f64::EPSILON);
            assert_eq!(max_delay_ms, 10000);
        } else {
            panic!("Expected Exponential backoff");
        }
    }

    #[test]
    fn test_retry_strategy_linear_backoff() {
        let s = RetryStrategy::linear_backoff(4, 100, 50);
        assert!(s.should_retry);
        assert_eq!(s.max_attempts, 4);
        assert_eq!(s.delay_ms, 100);
        assert!((s.success_probability - 0.6).abs() < f64::EPSILON);
        if let BackoffType::Linear { increment_ms } = s.backoff_strategy {
            assert_eq!(increment_ms, 50);
        } else {
            panic!("Expected Linear backoff");
        }
    }

    #[test]
    fn test_error_severity_ordering() {
        assert!(ErrorSeverity::Info < ErrorSeverity::Warning);
        assert!(ErrorSeverity::Warning < ErrorSeverity::Error);
        assert!(ErrorSeverity::Error < ErrorSeverity::Critical);
        assert!(ErrorSeverity::Critical < ErrorSeverity::Emergency);
    }

    #[test]
    fn test_action_risk_level_ordering() {
        assert!(ActionRiskLevel::None < ActionRiskLevel::Low);
        assert!(ActionRiskLevel::Low < ActionRiskLevel::Medium);
        assert!(ActionRiskLevel::Medium < ActionRiskLevel::High);
        assert!(ActionRiskLevel::High < ActionRiskLevel::Critical);
    }

    #[test]
    fn test_ai_error_category_serialization() {
        let categories = [
            AIErrorCategory::ResourceLimitation,
            AIErrorCategory::ConfigurationIssue,
            AIErrorCategory::NetworkFailure,
            AIErrorCategory::RateLimiting,
        ];
        for cat in categories {
            let json = serde_json::to_string(&cat).expect("serialize");
            let parsed: AIErrorCategory = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(cat, parsed);
        }
    }

    #[test]
    fn test_retry_strategy_serialization() {
        let s = RetryStrategy::exponential_backoff(3, 100, 5000);
        let json = serde_json::to_string(&s).expect("serialize");
        let parsed: RetryStrategy = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(s.should_retry, parsed.should_retry);
        assert_eq!(s.max_attempts, parsed.max_attempts);
    }

    #[test]
    fn test_backoff_type_serialization() {
        let linear = BackoffType::Linear { increment_ms: 500 };
        let json = serde_json::to_string(&linear).expect("serialize");
        let parsed: BackoffType = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed, BackoffType::Linear { increment_ms: 500 }));

        let exp = BackoffType::Exponential {
            base: 2.0,
            max_delay_ms: 10000,
        };
        let json = serde_json::to_string(&exp).expect("serialize");
        let parsed: BackoffType = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(
            parsed,
            BackoffType::Exponential {
                base: 2.0,
                max_delay_ms: 10000
            }
        ));
    }

    #[test]
    fn test_action_type_custom_serialization() {
        let custom = ActionType::Custom {
            action_type: "custom.restart".to_string(),
        };
        let json = serde_json::to_string(&custom).expect("serialize");
        let parsed: ActionType = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(
            parsed,
            ActionType::Custom { action_type } if action_type == "custom.restart"
        ));
    }

    #[test]
    fn test_all_ai_error_category_variants_serialization() {
        let categories = [
            AIErrorCategory::ResourceLimitation,
            AIErrorCategory::ConfigurationIssue,
            AIErrorCategory::SecurityViolation,
            AIErrorCategory::NetworkFailure,
            AIErrorCategory::RuntimeError,
            AIErrorCategory::HumanInterventionRequired,
            AIErrorCategory::DependencyFailure,
            AIErrorCategory::RateLimiting,
            AIErrorCategory::DataIssue,
            AIErrorCategory::ServiceUnavailable,
            AIErrorCategory::SystemError,
            AIErrorCategory::UserError,
        ];
        for cat in categories {
            let json = serde_json::to_string(&cat).expect("serialize");
            let parsed: AIErrorCategory = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(cat, parsed);
        }
    }

    #[test]
    fn test_backoff_type_fibonacci_serialization() {
        let fib = BackoffType::Fibonacci { max_delay_ms: 5000 };
        let json = serde_json::to_string(&fib).expect("serialize");
        let parsed: BackoffType = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(
            parsed,
            BackoffType::Fibonacci { max_delay_ms: 5000 }
        ));
    }

    #[test]
    fn test_backoff_type_custom_serialization() {
        let custom = BackoffType::Custom {
            delays_ms: vec![100, 200, 400, 800],
        };
        let json = serde_json::to_string(&custom).expect("serialize");
        let parsed: BackoffType = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(
            parsed,
            BackoffType::Custom { delays_ms } if delays_ms == vec![100, 200, 400, 800]
        ));
    }

    #[test]
    fn test_action_type_display_debug() {
        let variants = [
            ActionType::Retry,
            ActionType::Restart,
            ActionType::UpdateConfig,
            ActionType::Scale,
            ActionType::Fallback,
            ActionType::RequestHuman,
            ActionType::LogMore,
            ActionType::HealthCheck,
            ActionType::Custom {
                action_type: "custom.test".to_string(),
            },
        ];
        for v in variants {
            let debug_str = format!("{v:?}");
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_action_type_all_variants_serialization() {
        let variants = [
            ActionType::Retry,
            ActionType::Restart,
            ActionType::UpdateConfig,
            ActionType::Scale,
            ActionType::Fallback,
            ActionType::RequestHuman,
            ActionType::LogMore,
            ActionType::HealthCheck,
        ];
        for v in variants {
            let json = serde_json::to_string(&v).expect("serialize");
            let parsed: ActionType = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(v, parsed, "round-trip failed for {v:?}");
        }
    }

    #[test]
    fn test_error_severity_comparisons() {
        assert!(ErrorSeverity::Info < ErrorSeverity::Warning);
        assert!(ErrorSeverity::Warning < ErrorSeverity::Error);
        assert!(ErrorSeverity::Error < ErrorSeverity::Critical);
        assert!(ErrorSeverity::Critical < ErrorSeverity::Emergency);
        assert!(ErrorSeverity::Emergency > ErrorSeverity::Info);
        assert_eq!(
            ErrorSeverity::Error.cmp(&ErrorSeverity::Error),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn test_error_severity_serialization() {
        let severities = [
            ErrorSeverity::Info,
            ErrorSeverity::Warning,
            ErrorSeverity::Error,
            ErrorSeverity::Critical,
            ErrorSeverity::Emergency,
        ];
        for sev in severities {
            let json = serde_json::to_string(&sev).expect("serialize");
            let parsed: ErrorSeverity = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(sev, parsed);
        }
    }

    #[test]
    fn test_suggested_action_serialization() {
        let action = SuggestedAction {
            id: "retry-1".to_string(),
            action_type: ActionType::Retry,
            description: "Retry the operation".to_string(),
            automated: true,
            confidence: 0.8,
            estimated_duration_secs: Some(5),
            prerequisites: vec!["network".to_string()],
            expected_outcome: "Success".to_string(),
            risk_level: ActionRiskLevel::Low,
        };
        let json = serde_json::to_string(&action).expect("serialize");
        let parsed: SuggestedAction = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.id, action.id);
        assert!((parsed.confidence - action.confidence).abs() < f64::EPSILON);
    }
}
