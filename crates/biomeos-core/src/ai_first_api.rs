//! AI-First Citizen API Standard Implementation
//!
//! ✅ MIGRATION NOTICE: Core AI types have been moved to biomeos-types
//! This module now re-exports the unified AI-first system and provides
//! backward compatibility for the biomeOS ecosystem.
//!
//! Implements the AI-First response format and human-AI collaboration context
//! as defined in handOff/AI_FIRST_CITIZEN_API_STANDARD.md

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Re-export unified AI-first types from biomeos-types
pub use biomeos_types::{
    error::AIErrorCategory, // From unified error system
    error::ErrorSeverity,   // From unified error system
    BiomeError,             // From unified error system
    BiomeOSConfig,          // From unified config system
    BiomeResult,            // From unified error system
    Environment,            // From unified config system
};

/// Universal AI-first response format - ALL ENDPOINTS MUST USE THIS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIFirstResponse<T> {
    /// Operation success status (machine-readable)
    pub success: bool,

    /// Strongly-typed response data
    pub data: T,

    /// AI-optimized error information
    pub error: Option<BiomeError>,

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

// ✅ MIGRATED: AIFirstError is now BiomeError from biomeos-types
// ✅ MIGRATED: AIErrorCategory is now unified in biomeos-types
// The unified system provides comprehensive error context, retry strategies,
// and AI-first features while maintaining backward compatibility.

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

// ✅ MIGRATED: ErrorSeverity is now unified in biomeos-types

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
        error: BiomeError,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_first_response_success() {
        let request_id = Uuid::new_v4();
        let response: AIFirstResponse<String> =
            AIFirstResponse::success(request_id, "test data".to_string(), 100, 0.95);

        assert!(response.success);
        assert!(response.error.is_none());
        assert_eq!(response.data, "test data");
        assert_eq!(response.processing_time_ms, 100);
        assert_eq!(response.confidence_score, 0.95);
    }

    #[test]
    fn test_ai_first_response_error() {
        let request_id = Uuid::new_v4();
        let error = BiomeError::internal_error("Test error", Some("ERR001"));
        let response: AIFirstResponse<String> =
            AIFirstResponse::error(request_id, error, 50, "default".to_string());

        assert!(!response.success);
        assert!(response.error.is_some());
        assert_eq!(response.confidence_score, 0.0);
    }

    #[test]
    fn test_retry_strategy() {
        let strategy = RetryStrategy {
            should_retry: true,
            delay_ms: 1000,
            max_attempts: 3,
            backoff_strategy: BackoffType::Exponential { base: 2.0 },
            retry_conditions: vec!["network_error".to_string()],
            success_probability: 0.8,
        };

        assert!(strategy.should_retry);
        assert_eq!(strategy.max_attempts, 3);
        assert_eq!(strategy.success_probability, 0.8);
    }

    #[test]
    fn test_backoff_types() {
        let types = [
            BackoffType::Linear,
            BackoffType::Exponential { base: 2.0 },
            BackoffType::Fibonacci,
            BackoffType::Custom {
                formula: "x^2".to_string(),
            },
        ];

        for backoff in types {
            let json = serde_json::to_string(&backoff).unwrap();
            let _: BackoffType = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_performance_metrics_default() {
        let metrics = PerformanceMetrics::default();
        assert_eq!(metrics.cpu_usage_percent, 0.0);
        assert_eq!(metrics.memory_usage_mb, 0.0);
        assert_eq!(metrics.io_operations, 0);
    }

    #[test]
    fn test_ai_resource_usage_default() {
        let usage = AIResourceUsage::default();
        assert_eq!(usage.compute_units_used, 0.0);
        assert_eq!(usage.storage_bytes_used, 0);
    }

    #[test]
    fn test_quality_metrics_default() {
        let metrics = QualityMetrics::default();
        assert!(metrics.accuracy_score.is_none());
        assert_eq!(metrics.completeness_score, 1.0);
        assert_eq!(metrics.reliability_score, 1.0);
    }

    #[test]
    fn test_cache_info_default() {
        let cache = CacheInfo::default();
        assert!(!cache.is_cached);
        assert!(cache.cache_hit_ratio.is_none());
    }

    #[test]
    fn test_rate_limit_status_default() {
        let status = RateLimitStatus::default();
        assert!(status.limit.is_none());
        assert!(status.remaining.is_none());
    }

    #[test]
    fn test_interaction_modes() {
        let modes = [
            InteractionMode::FullyAutonomous,
            InteractionMode::HumanApproval,
            InteractionMode::Collaborative,
            InteractionMode::HumanDirected,
            InteractionMode::HumanSupervised,
            InteractionMode::Emergency,
        ];

        for mode in modes {
            let json = serde_json::to_string(&mode).unwrap();
            let _: InteractionMode = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_notification_urgency_levels() {
        let levels = [
            NotificationUrgency::Low,
            NotificationUrgency::Medium,
            NotificationUrgency::High,
            NotificationUrgency::Critical,
        ];

        for level in levels {
            let json = serde_json::to_string(&level).unwrap();
            let _: NotificationUrgency = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_priority_levels() {
        let levels = [
            PriorityLevel::Low,
            PriorityLevel::Normal,
            PriorityLevel::High,
            PriorityLevel::Critical,
        ];

        for level in levels {
            let json = serde_json::to_string(&level).unwrap();
            let _: PriorityLevel = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_data_sharing_levels() {
        let levels = [
            DataSharingLevel::None,
            DataSharingLevel::Anonymous,
            DataSharingLevel::Aggregated,
            DataSharingLevel::Full,
        ];

        for level in levels {
            let json = serde_json::to_string(&level).unwrap();
            let _: DataSharingLevel = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_contact_types() {
        let types = [
            ContactType::Email,
            ContactType::SMS,
            ContactType::Slack,
            ContactType::Teams,
            ContactType::Webhook,
        ];

        for contact_type in types {
            let json = serde_json::to_string(&contact_type).unwrap();
            let _: ContactType = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_action_results() {
        let results = [
            ActionResult::Success,
            ActionResult::Failure,
            ActionResult::PartialSuccess,
            ActionResult::Skipped,
        ];

        for result in results {
            let json = serde_json::to_string(&result).unwrap();
            let _: ActionResult = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_risk_levels() {
        let levels = [
            RiskLevel::VeryLow,
            RiskLevel::Low,
            RiskLevel::Medium,
            RiskLevel::High,
            RiskLevel::VeryHigh,
        ];

        for level in levels {
            let json = serde_json::to_string(&level).unwrap();
            let _: RiskLevel = serde_json::from_str(&json).unwrap();
        }
    }

    #[test]
    fn test_suggested_action() {
        let action = SuggestedAction {
            action_type: "restart".to_string(),
            description: "Restart the service".to_string(),
            confidence: 0.9,
            estimated_duration_ms: Some(5000),
            estimated_cost_usd: Some(0.01),
            prerequisites: vec!["backup".to_string()],
            risk_level: RiskLevel::Low,
        };

        assert_eq!(action.confidence, 0.9);
        assert!(action.estimated_duration_ms.is_some());
    }

    #[test]
    fn test_quiet_hours() {
        let hours = QuietHours {
            start_hour: 22,
            end_hour: 7,
            timezone: "America/New_York".to_string(),
        };

        assert_eq!(hours.start_hour, 22);
        assert_eq!(hours.end_hour, 7);
    }

    #[test]
    fn test_ai_resource_limits() {
        let limits = AIResourceLimits {
            max_cost_per_operation_usd: 1.0,
            max_processing_time_ms: 30000,
            max_memory_usage_mb: 1024,
            priority_level: PriorityLevel::High,
        };

        assert_eq!(limits.max_cost_per_operation_usd, 1.0);
    }

    #[test]
    fn test_risk_tolerance() {
        let tolerance = RiskTolerance {
            financial_risk_tolerance: 0.5,
            operational_risk_tolerance: 0.3,
            data_sharing_tolerance: DataSharingLevel::Aggregated,
            experimental_features_enabled: false,
        };

        assert!(!tolerance.experimental_features_enabled);
    }

    #[test]
    fn test_escalation_contact() {
        let contact = EscalationContact {
            contact_type: ContactType::Slack,
            address: "#alerts".to_string(),
            urgency_level: NotificationUrgency::High,
        };

        assert_eq!(contact.address, "#alerts");
    }

    #[test]
    fn test_session_context() {
        let context = SessionContext {
            session_id: Uuid::new_v4(),
            started_at: chrono::Utc::now(),
            previous_actions: vec![],
            current_goal: "Deploy application".to_string(),
            context_variables: HashMap::new(),
        };

        assert_eq!(context.current_goal, "Deploy application");
    }

    #[test]
    fn test_action_history() {
        let history = ActionHistory {
            action: "create_container".to_string(),
            timestamp: chrono::Utc::now(),
            result: ActionResult::Success,
            confidence_score: 0.95,
        };

        assert_eq!(history.action, "create_container");
    }

    #[test]
    fn test_ai_response_metadata_default() {
        let metadata = AIResponseMetadata::default();
        assert!(metadata.dependencies.is_empty());
    }
}
