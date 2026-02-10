//! Scaling Specifications
//!
//! This module contains scaling specifications including horizontal
//! scaling (HPA), vertical scaling (VPA), and custom scaling.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Scaling specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingSpec {
    /// Scaling type
    pub scaling_type: ScalingType,

    /// Horizontal scaling
    pub horizontal: Option<HorizontalScalingSpec>,

    /// Vertical scaling
    pub vertical: Option<VerticalScalingSpec>,

    /// Custom scaling
    pub custom: Option<CustomScalingSpec>,
}

/// Scaling types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingType {
    /// No scaling
    None,
    /// Horizontal scaling only
    Horizontal,
    /// Vertical scaling only
    Vertical,
    /// Both horizontal and vertical scaling
    Both,
    /// Custom scaling strategy
    Custom,
}

/// Horizontal scaling specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HorizontalScalingSpec {
    /// Minimum replicas
    pub min_replicas: u32,

    /// Maximum replicas
    pub max_replicas: u32,

    /// Target metrics
    pub target_metrics: Vec<ScalingMetric>,

    /// Scale-up policy
    pub scale_up_policy: Option<ScalingPolicy>,

    /// Scale-down policy
    pub scale_down_policy: Option<ScalingPolicy>,

    /// Behavior configuration
    pub behavior: Option<ScalingBehavior>,
}

/// Scaling metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingMetric {
    /// Metric type
    pub metric_type: ScalingMetricType,

    /// Target value
    pub target_value: ScalingTargetValue,

    /// Metric selector
    pub selector: Option<MetricSelector>,
}

/// Scaling metric types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingMetricType {
    /// Resource metrics (CPU, Memory)
    Resource {
        /// Resource name to monitor
        name: ResourceName,
    },
    /// Pod metrics
    Pods {
        /// Metric name to query
        metric_name: String,
    },
    /// Object metrics
    Object {
        /// Metric name to query
        metric_name: String,
        /// Target object reference
        target: ObjectTarget,
    },
    /// External metrics
    External {
        /// External metric name
        metric_name: String,
    },
}

/// Resource names
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceName {
    /// CPU resource
    Cpu,
    /// Memory resource
    Memory,
    /// Storage resource
    Storage,
    /// Custom resource
    Custom(String),
}

/// Object target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectTarget {
    /// Target kind
    pub kind: String,

    /// Target name
    pub name: String,

    /// Target API version
    pub api_version: String,
}

/// Scaling target value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingTargetValue {
    /// Utilization percentage
    Utilization(u32),
    /// Average value
    AverageValue(String),
    /// Value
    Value(String),
}

/// Metric selector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSelector {
    /// Match labels
    pub match_labels: HashMap<String, String>,
}

/// Scaling policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingPolicy {
    /// Policy type
    pub policy_type: ScalingPolicyType,

    /// Value
    pub value: u32,

    /// Period seconds
    pub period_seconds: u32,
}

/// Scaling policy types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingPolicyType {
    /// Scale by pod count
    Pods,
    /// Scale by percentage
    Percent,
}

/// Scaling behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingBehavior {
    /// Scale-up behavior
    pub scale_up: Option<ScalingDirectionBehavior>,

    /// Scale-down behavior
    pub scale_down: Option<ScalingDirectionBehavior>,
}

/// Scaling direction behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingDirectionBehavior {
    /// Stabilization window seconds
    pub stabilization_window_seconds: Option<u32>,

    /// Select policy
    pub select_policy: Option<ScalingPolicySelect>,

    /// Policies
    pub policies: Vec<ScalingPolicy>,
}

/// Scaling policy select
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingPolicySelect {
    /// Select the policy with the maximum change
    Max,
    /// Select the policy with the minimum change
    Min,
    /// Scaling is disabled
    Disabled,
}

/// Vertical scaling specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerticalScalingSpec {
    /// Update mode
    pub update_mode: VerticalScalingMode,

    /// Resource policy
    pub resource_policy: Option<VerticalResourcePolicy>,

    /// Recommenders
    pub recommenders: Vec<VerticalRecommender>,
}

/// Vertical scaling modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerticalScalingMode {
    /// Vertical scaling is off
    Off,
    /// Apply recommendations only at creation
    Initial,
    /// Automatically apply recommendations
    Auto,
}

/// Vertical resource policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerticalResourcePolicy {
    /// Container policies
    pub container_policies: Vec<ContainerResourcePolicy>,
}

/// Container resource policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerResourcePolicy {
    /// Container name
    pub container_name: String,

    /// Controlled resources
    pub controlled_resources: Vec<ResourceName>,

    /// Controlled values
    pub controlled_values: Vec<ControlledValue>,

    /// Min allowed
    pub min_allowed: HashMap<String, String>,

    /// Max allowed
    pub max_allowed: HashMap<String, String>,
}

/// Controlled values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlledValue {
    /// CPU requests in millicores
    RequestsCpuMillis,
    /// Memory requests in bytes
    RequestsMemoryBytes,
    /// CPU limits in millicores
    LimitsCpuMillis,
    /// Memory limits in bytes
    LimitsMemoryBytes,
}

/// Vertical recommender
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerticalRecommender {
    /// Name
    pub name: String,
}

/// Custom scaling specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomScalingSpec {
    /// Scaling provider
    pub provider: String,

    /// Configuration
    pub config: HashMap<String, serde_json::Value>,

    /// Metrics
    pub metrics: Vec<CustomMetric>,
}

/// Custom metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMetric {
    /// Metric name
    pub name: String,

    /// Metric query
    pub query: String,

    /// Target value
    pub target: f64,
}

impl Default for ScalingSpec {
    fn default() -> Self {
        Self {
            scaling_type: ScalingType::None,
            horizontal: None,
            vertical: None,
            custom: None,
        }
    }
}
