// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! AI-First API type definitions
//!
//! Types for the AI-First Citizen API Standard: response format,
//! metadata, human-AI collaboration context, and suggested actions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Universal AI-first response format - ALL ENDPOINTS MUST USE THIS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIFirstResponse<T> {
    /// Operation success status (machine-readable)
    pub success: bool,

    /// Strongly-typed response data
    pub data: T,

    /// AI-optimized error information
    pub error: Option<biomeos_types::BiomeError>,

    /// Unique request identifier for tracing and correlation
    pub request_id: Uuid,

    /// Processing time in milliseconds for performance monitoring
    pub processing_time_ms: u64,

    /// AI-specific metadata for decision making
    pub ai_metadata: AIResponseMetadata,

    /// Human interaction context (when applicable)
    pub human_context: Option<HumanInteractionContext>,

    /// Confidence score for AI decision making (0.0 - 1.0)
    pub confidence_score: f64,

    /// Suggested next actions for AI agents
    pub suggested_actions: Vec<SuggestedAction>,
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

    /// Estimated success probability for retry
    pub success_probability: f64,
}

/// Backoff strategy types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffType {
    /// Linear backoff (constant delay)
    Linear,
    /// Exponential backoff with configurable base
    Exponential {
        /// Exponent base multiplier
        base: f64,
    },
    /// Fibonacci sequence backoff
    Fibonacci,
    /// User-defined formula
    Custom {
        /// Formula expression (e.g. `"2^n * 100"`)
        formula: String,
    },
}

/// Metadata specifically designed for AI decision making
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AIResponseMetadata {
    /// Performance characteristics
    pub performance: PerformanceMetrics,

    /// Resource utilization
    pub resource_usage: AIResourceUsage,

    /// Quality indicators
    pub quality_metrics: QualityMetrics,

    /// Caching information
    pub cache_info: CacheInfo,

    /// Rate limiting status
    pub rate_limit_status: RateLimitStatus,

    /// Related operations or dependencies
    pub dependencies: Vec<String>,
}

/// Performance metrics for AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// CPU usage as a percentage (0–100)
    pub cpu_usage_percent: f64,
    /// Memory consumption in MiB
    pub memory_usage_mb: f64,
    /// Number of I/O operations performed
    pub io_operations: u64,
    /// Network round-trip latency in milliseconds
    pub network_latency_ms: Option<f64>,
    /// Operations per second throughput
    pub throughput_ops_per_sec: Option<f64>,
}

/// Resource usage for AI monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResourceUsage {
    /// Abstract compute units consumed
    pub compute_units_used: f64,
    /// Storage bytes consumed
    pub storage_bytes_used: u64,
    /// Network bandwidth bytes consumed
    pub bandwidth_bytes_used: u64,
    /// Estimated monetary cost in USD
    pub cost_estimate_usd: Option<f64>,
}

/// Quality metrics for AI evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Accuracy score (0.0–1.0), if measurable
    pub accuracy_score: Option<f64>,
    /// Data completeness score (0.0–1.0)
    pub completeness_score: f64,
    /// Reliability / confidence score (0.0–1.0)
    pub reliability_score: f64,
    /// When the underlying data was last refreshed
    pub freshness_timestamp: chrono::DateTime<chrono::Utc>,
}

/// Cache information for AI optimization
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheInfo {
    /// Whether this response came from cache
    pub is_cached: bool,
    /// Cache hit ratio for this endpoint (0.0–1.0)
    pub cache_hit_ratio: Option<f64>,
    /// When the cached entry expires
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Cache key used for lookup
    pub cache_key: Option<String>,
}

/// Rate limiting status for AI awareness
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RateLimitStatus {
    /// Maximum requests allowed in the window
    pub limit: Option<u64>,
    /// Remaining requests in the current window
    pub remaining: Option<u64>,
    /// When the rate limit window resets
    pub reset_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Suggested delay before retrying (ms)
    pub retry_after_ms: Option<u64>,
}

/// Context for human-AI collaborative operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanInteractionContext {
    /// Human user identifier (when applicable)
    pub user_id: Option<String>,

    /// Current interaction mode
    pub interaction_mode: InteractionMode,

    /// User preferences for AI operations
    pub preferences: AIUserPreferences,

    /// Whether human approval is required for this operation
    pub approval_required: bool,

    /// Confidence threshold for auto-execution
    pub confidence_threshold: f64,

    /// Escalation configuration
    pub escalation_config: EscalationConfig,

    /// Session context for multi-step operations
    pub session_context: Option<SessionContext>,
}

/// Interaction modes for human-AI collaboration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InteractionMode {
    /// AI operates completely autonomously
    FullyAutonomous,

    /// AI suggests actions, human approves before execution
    HumanApproval,

    /// Real-time collaboration between human and AI
    Collaborative,

    /// Human directs strategy, AI executes tactics
    HumanDirected,

    /// AI monitors and alerts, human makes key decisions
    HumanSupervised,

    /// Emergency mode - AI acts immediately, notifies human
    Emergency,
}

/// User preferences for AI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIUserPreferences {
    /// Preferred AI models for different operation types
    pub model_preferences: HashMap<String, String>,

    /// Auto-approval thresholds by operation category
    pub auto_approval_thresholds: HashMap<String, f64>,

    /// Notification preferences
    pub notifications: NotificationPreferences,

    /// Resource usage limits and preferences
    pub resource_limits: AIResourceLimits,

    /// Risk tolerance levels
    pub risk_tolerance: RiskTolerance,

    /// Learning preferences
    pub learning_enabled: bool,
}

/// Notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    /// Whether email notifications are enabled
    pub email_enabled: bool,
    /// Whether push notifications are enabled
    pub push_enabled: bool,
    /// Minimum urgency level to trigger a notification
    pub urgency_threshold: NotificationUrgency,
    /// Optional quiet-hours window (no notifications)
    pub quiet_hours: Option<QuietHours>,
}

/// Notification urgency levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationUrgency {
    /// Low priority — batched delivery
    Low,
    /// Normal priority
    Medium,
    /// High priority — immediate delivery
    High,
    /// Critical — wake-up / pager
    Critical,
}

/// Quiet hours configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHours {
    /// Hour of day to start quiet period (0–23)
    pub start_hour: u8,
    /// Hour of day to end quiet period (0–23)
    pub end_hour: u8,
    /// IANA timezone name
    pub timezone: String,
}

/// AI resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResourceLimits {
    /// Maximum cost per operation in USD
    pub max_cost_per_operation_usd: f64,
    /// Maximum processing time in milliseconds
    pub max_processing_time_ms: u64,
    /// Maximum memory usage in MiB
    pub max_memory_usage_mb: u64,
    /// Priority level for resource scheduling
    pub priority_level: PriorityLevel,
}

/// Priority levels for resource allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriorityLevel {
    /// Background / best-effort
    Low,
    /// Default priority
    Normal,
    /// Elevated priority
    High,
    /// Highest priority — preempts other work
    Critical,
}

/// Risk tolerance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskTolerance {
    /// Acceptable financial risk (0.0 = none, 1.0 = full)
    pub financial_risk_tolerance: f64,
    /// Acceptable operational risk (0.0 = none, 1.0 = full)
    pub operational_risk_tolerance: f64,
    /// Data sharing comfort level
    pub data_sharing_tolerance: DataSharingLevel,
    /// Whether to enable experimental / beta features
    pub experimental_features_enabled: bool,
}

/// Data sharing tolerance levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSharingLevel {
    /// No data sharing permitted
    None,
    /// Anonymized data only
    Anonymous,
    /// Aggregated / statistical data
    Aggregated,
    /// Full data sharing (with consent)
    Full,
}

/// Escalation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationConfig {
    /// Ordered list of escalation contacts
    pub escalation_contacts: Vec<EscalationContact>,
    /// Thresholds by category that trigger escalation
    pub escalation_thresholds: HashMap<String, f64>,
    /// Whether to auto-escalate without human trigger
    pub auto_escalation_enabled: bool,
    /// Minutes to wait before auto-escalating
    pub escalation_delay_minutes: u32,
}

/// Escalation contact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationContact {
    /// Contact channel type
    pub contact_type: ContactType,
    /// Contact address (email, phone, URL, etc.)
    pub address: String,
    /// Minimum urgency for this contact to be notified
    pub urgency_level: NotificationUrgency,
}

/// Contact types for escalation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContactType {
    /// Email notification
    Email,
    /// SMS text message
    SMS,
    /// Slack message
    Slack,
    /// Microsoft Teams message
    Teams,
    /// Generic webhook POST
    Webhook,
}

/// Session context for multi-step operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    /// Unique session identifier
    pub session_id: Uuid,
    /// When the session began
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// History of actions taken in this session
    pub previous_actions: Vec<ActionHistory>,
    /// Current high-level goal description
    pub current_goal: String,
    /// Arbitrary context variables for session state
    pub context_variables: HashMap<String, serde_json::Value>,
}

/// Action history for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionHistory {
    /// Action that was performed
    pub action: String,
    /// When the action was performed
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Outcome of the action
    pub result: ActionResult,
    /// AI confidence in the action (0.0–1.0)
    pub confidence_score: f64,
}

/// Action result status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionResult {
    /// Action completed successfully
    Success,
    /// Action failed
    Failure,
    /// Action partially succeeded
    PartialSuccess,
    /// Action was skipped
    Skipped,
}

/// Suggested actions for AI agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedAction {
    /// Type / category of the action
    pub action_type: String,
    /// Human-readable description
    pub description: String,
    /// AI confidence in this suggestion (0.0–1.0)
    pub confidence: f64,
    /// Estimated execution time in milliseconds
    pub estimated_duration_ms: Option<u64>,
    /// Estimated monetary cost in USD
    pub estimated_cost_usd: Option<f64>,
    /// Prerequisites that must be met first
    pub prerequisites: Vec<String>,
    /// Risk assessment for this action
    pub risk_level: RiskLevel,
}

/// Risk levels for suggested actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Negligible risk
    VeryLow,
    /// Low risk
    Low,
    /// Moderate risk — review recommended
    Medium,
    /// High risk — approval required
    High,
    /// Very high risk — manual intervention required
    VeryHigh,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_usage_mb: 0.0,
            io_operations: 0,
            network_latency_ms: None,
            throughput_ops_per_sec: None,
        }
    }
}

impl Default for AIResourceUsage {
    fn default() -> Self {
        Self {
            compute_units_used: 0.0,
            storage_bytes_used: 0,
            bandwidth_bytes_used: 0,
            cost_estimate_usd: None,
        }
    }
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self {
            accuracy_score: None,
            completeness_score: 1.0,
            reliability_score: 1.0,
            freshness_timestamp: chrono::Utc::now(),
        }
    }
}
