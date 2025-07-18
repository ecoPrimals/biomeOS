//! Biome Requirements Module
//!
//! This module defines requirement specifications for biomes including
//! capability requirements, performance requirements, and scaling requirements.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::universal_primal::{CapabilityRequirement, Constraint, ResourceRequirements};

/// Biome requirements specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeRequirements {
    /// Required capabilities
    pub capabilities: Vec<CapabilityRequirement>,
    /// Resource requirements
    pub resources: ResourceRequirements,
    /// Performance requirements
    pub performance: PerformanceRequirements,
    /// Availability requirements
    pub availability: AvailabilityRequirements,
    /// Backup requirements
    pub backup: BackupRequirements,
    /// Scaling requirements
    pub scaling: ScalingRequirements,
    /// Compliance requirements
    pub compliance: Vec<String>,
    /// Geographic constraints
    pub geographic: Vec<String>,
    /// Time constraints
    pub time_constraints: Vec<String>,
}

/// Performance requirements for the biome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRequirements {
    /// Maximum acceptable latency in milliseconds
    pub max_latency_ms: u64,
    /// Minimum throughput requirements
    pub min_throughput: HashMap<String, u64>,
    /// Maximum response time in milliseconds
    pub max_response_time_ms: u64,
    /// Minimum uptime percentage
    pub min_uptime_percent: f64,
}

/// Availability requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityRequirements {
    /// Fault tolerance level
    pub fault_tolerance: FaultToleranceLevel,
    /// Disaster recovery requirements
    pub disaster_recovery: bool,
    /// Multi-region support requirement
    pub multi_region: bool,
    /// Maintenance window tolerance
    pub maintenance_window_minutes: u64,
}

/// Fault tolerance levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FaultToleranceLevel {
    /// No fault tolerance
    None,
    /// Basic fault tolerance
    Basic,
    /// High fault tolerance
    High,
    /// Maximum fault tolerance
    Maximum,
}

/// Backup requirements specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRequirements {
    /// Backup frequency in hours
    pub frequency_hours: u64,
    /// Retention period in days
    pub retention_days: u64,
    /// Backup encryption required
    pub encryption_required: bool,
    /// Cross-region backup required
    pub cross_region: bool,
}

/// Scaling requirements for the biome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingRequirements {
    /// Horizontal scaling enabled
    pub horizontal_scaling: bool,
    /// Vertical scaling enabled
    pub vertical_scaling: bool,
    /// Minimum instances
    pub min_instances: u32,
    /// Maximum instances
    pub max_instances: u32,
    /// Scaling triggers
    pub triggers: Vec<ScalingTrigger>,
    /// Scaling policies
    pub policies: Vec<ScalingPolicy>,
}

/// Scaling trigger specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingTrigger {
    /// Trigger name
    pub name: String,
    /// Metric to monitor
    pub metric: String,
    /// Threshold value
    pub threshold: f64,
    /// Comparison operator
    pub operator: String,
    /// Duration in seconds
    pub duration_seconds: u64,
}

/// Scaling policy specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingPolicy {
    /// Policy name
    pub name: String,
    /// Scaling direction
    pub direction: ScalingDirection,
    /// Scaling amount
    pub amount: ScalingAmount,
    /// Cooldown period in seconds
    pub cooldown_seconds: u64,
}

/// Scaling direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingDirection {
    /// Scale up
    Up,
    /// Scale down
    Down,
}

/// Scaling amount specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingAmount {
    /// Fixed number of instances
    Fixed(u32),
    /// Percentage of current instances
    Percentage(f64),
    /// Dynamic based on load
    Dynamic,
} 