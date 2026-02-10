//! AI suggestion types — pure data structures for suggestion engine.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AI suggestion from Squirrel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISuggestion {
    /// Unique suggestion ID
    pub id: String,

    /// Suggestion type
    pub suggestion_type: SuggestionType,

    /// Confidence score (0.0-1.0)
    pub confidence: f32,

    /// Human-readable explanation
    pub explanation: String,

    /// Suggested action
    pub action: SuggestedAction,

    /// Expected impact
    pub impact: Impact,
}

/// Type of suggestion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionType {
    /// Device assignment recommendation
    DeviceAssignment,

    /// Topology optimization
    TopologyOptimization,

    /// Bottleneck prediction
    BottleneckPrediction,

    /// Resource reallocation
    ResourceReallocation,

    /// Performance improvement
    PerformanceImprovement,
}

/// Suggested action to take
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SuggestedAction {
    /// Assign a device to a primal
    AssignDevice {
        /// Target device ID
        device_id: String,
        /// Target primal ID
        primal_id: String,
        /// Reason for the suggestion
        reason: String,
    },

    /// Remove a device assignment
    RemoveAssignment {
        /// Device ID to unassign
        device_id: String,
        /// Primal ID currently assigned
        primal_id: String,
        /// Reason for the suggestion
        reason: String,
    },

    /// Reallocate resources
    ReallocateResources {
        /// Source primal ID
        from_primal: String,
        /// Destination primal ID
        to_primal: String,
        /// Type of resource (cpu, memory, gpu, etc.)
        resource_type: String,
        /// Amount to reallocate
        amount: String,
    },

    /// Add more capacity
    AddCapacity {
        /// Type of primal to add
        primal_type: String,
        /// Estimated capacity need
        estimated_need: String,
    },

    /// Optimize configuration
    OptimizeConfig {
        /// Target primal ID
        primal_id: String,
        /// Configuration key to change
        config_key: String,
        /// Suggested new value
        suggested_value: serde_json::Value,
    },
}

/// Expected impact of taking the suggested action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Impact {
    /// Performance improvement percentage
    pub performance_improvement: Option<f32>,

    /// Cost implications
    pub cost_change: Option<String>,

    /// Affected primals
    pub affected_primals: Vec<String>,

    /// Risk level (low, medium, high)
    pub risk_level: String,
}

/// Context for generating suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionContext {
    /// Current device assignments (device_id -> primal_id)
    pub assignments: HashMap<String, String>,

    /// Available devices
    pub available_devices: Vec<DeviceInfo>,

    /// Running primals
    pub running_primals: Vec<PrimalInfo>,

    /// Recent events (optional)
    pub recent_events: Option<Vec<String>>,

    /// User preferences (optional)
    pub preferences: Option<HashMap<String, String>>,
}

/// Device information for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Unique device identifier
    pub id: String,
    /// Type of device (gpu, cpu, storage, etc.)
    pub device_type: String,
    /// List of device capabilities
    pub capabilities: Vec<String>,
    /// Currently assigned primal ID, if any
    pub current_assignment: Option<String>,
}

/// Primal information for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Unique primal identifier
    pub id: String,
    /// Primal name
    pub name: String,
    /// Type of primal (security, discovery, compute, etc.)
    pub primal_type: String,
    /// List of primal capabilities
    pub capabilities: Vec<String>,
    /// Health status (healthy, degraded, unhealthy)
    pub health: String,
    /// Current load factor (0.0 - 1.0), if known
    pub load: Option<f32>,
}

/// User feedback on a suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionFeedback {
    /// User accepted and applied the suggestion
    Accepted,

    /// User rejected the suggestion with reason
    Rejected {
        /// Reason for rejection
        reason: String,
    },

    /// User dismissed without action
    Dismissed,

    /// User modified the suggestion
    Modified {
        /// Description of modifications made
        changes: String,
    },
}
