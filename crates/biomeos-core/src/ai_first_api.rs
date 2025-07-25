//! AI-First Citizen API Standard Implementation
//!
//! Implements the AI-First response format and human-AI collaboration context
//! as defined in handOff/AI_FIRST_CITIZEN_API_STANDARD.md

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
    pub error: Option<AIFirstError>,

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

/// AI-optimized error structure with automation hints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIFirstError {
    /// Machine-readable error code (UPPER_SNAKE_CASE)
    pub code: String,

    /// Human-readable message (for logging/debugging)
    pub message: String,

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
}

/// Error categories for AI classification
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    Linear,
    Exponential { base: f64 },
    Fibonacci,
    Custom { formula: String },
}

/// Error severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
    Fatal,
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
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: f64,
    pub io_operations: u64,
    pub network_latency_ms: Option<f64>,
    pub throughput_ops_per_sec: Option<f64>,
}

/// Resource usage for AI monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResourceUsage {
    pub compute_units_used: f64,
    pub storage_bytes_used: u64,
    pub bandwidth_bytes_used: u64,
    pub cost_estimate_usd: Option<f64>,
}

/// Quality metrics for AI evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub accuracy_score: Option<f64>,
    pub completeness_score: f64,
    pub reliability_score: f64,
    pub freshness_timestamp: chrono::DateTime<chrono::Utc>,
}

/// Cache information for AI optimization
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheInfo {
    pub is_cached: bool,
    pub cache_hit_ratio: Option<f64>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub cache_key: Option<String>,
}

/// Rate limiting status for AI awareness
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RateLimitStatus {
    pub limit: Option<u64>,
    pub remaining: Option<u64>,
    pub reset_at: Option<chrono::DateTime<chrono::Utc>>,
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
    pub email_enabled: bool,
    pub push_enabled: bool,
    pub urgency_threshold: NotificationUrgency,
    pub quiet_hours: Option<QuietHours>,
}

/// Notification urgency levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationUrgency {
    Low,
    Medium,
    High,
    Critical,
}

/// Quiet hours configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHours {
    pub start_hour: u8,
    pub end_hour: u8,
    pub timezone: String,
}

/// AI resource limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResourceLimits {
    pub max_cost_per_operation_usd: f64,
    pub max_processing_time_ms: u64,
    pub max_memory_usage_mb: u64,
    pub priority_level: PriorityLevel,
}

/// Priority levels for resource allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PriorityLevel {
    Low,
    Normal,
    High,
    Critical,
}

/// Risk tolerance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskTolerance {
    pub financial_risk_tolerance: f64,
    pub operational_risk_tolerance: f64,
    pub data_sharing_tolerance: DataSharingLevel,
    pub experimental_features_enabled: bool,
}

/// Data sharing tolerance levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSharingLevel {
    None,
    Anonymous,
    Aggregated,
    Full,
}

/// Escalation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationConfig {
    pub escalation_contacts: Vec<EscalationContact>,
    pub escalation_thresholds: HashMap<String, f64>,
    pub auto_escalation_enabled: bool,
    pub escalation_delay_minutes: u32,
}

/// Escalation contact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationContact {
    pub contact_type: ContactType,
    pub address: String,
    pub urgency_level: NotificationUrgency,
}

/// Contact types for escalation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContactType {
    Email,
    SMS,
    Slack,
    Teams,
    Webhook,
}

/// Session context for multi-step operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub session_id: Uuid,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub previous_actions: Vec<ActionHistory>,
    pub current_goal: String,
    pub context_variables: HashMap<String, serde_json::Value>,
}

/// Action history for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionHistory {
    pub action: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub result: ActionResult,
    pub confidence_score: f64,
}

/// Action result status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionResult {
    Success,
    Failure,
    PartialSuccess,
    Skipped,
}

/// Suggested actions for AI agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedAction {
    pub action_type: String,
    pub description: String,
    pub confidence: f64,
    pub estimated_duration_ms: Option<u64>,
    pub estimated_cost_usd: Option<f64>,
    pub prerequisites: Vec<String>,
    pub risk_level: RiskLevel,
}

/// Risk levels for suggested actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Helper functions for creating AI-first responses
impl<T> AIFirstResponse<T> {
    /// Create a successful AI-first response
    pub fn success(
        request_id: Uuid,
        data: T,
        processing_time_ms: u64,
        confidence_score: f64,
    ) -> Self {
        Self {
            success: true,
            data,
            error: None,
            request_id,
            processing_time_ms,
            ai_metadata: AIResponseMetadata::default(),
            human_context: None,
            confidence_score,
            suggested_actions: Vec::new(),
        }
    }

    /// Create a failed AI-first response
    pub fn error(
        request_id: Uuid,
        error: AIFirstError,
        processing_time_ms: u64,
        default_data: T,
    ) -> Self {
        Self {
            success: false,
            data: default_data,
            error: Some(error),
            request_id,
            processing_time_ms,
            ai_metadata: AIResponseMetadata::default(),
            human_context: None,
            confidence_score: 0.0,
            suggested_actions: Vec::new(),
        }
    }
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
