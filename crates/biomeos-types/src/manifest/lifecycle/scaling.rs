// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn scaling_spec_default() {
        let spec = ScalingSpec::default();
        assert!(matches!(spec.scaling_type, ScalingType::None));
        assert!(spec.horizontal.is_none());
        assert!(spec.vertical.is_none());
        assert!(spec.custom.is_none());
    }

    #[test]
    fn scaling_type_variants() {
        let _ = ScalingType::None;
        let _ = ScalingType::Horizontal;
        let _ = ScalingType::Vertical;
        let _ = ScalingType::Both;
        let _ = ScalingType::Custom;
    }

    #[test]
    fn scaling_spec_serialization_roundtrip() {
        let spec = ScalingSpec::default();
        let json = serde_json::to_string(&spec).expect("serialize");
        let parsed: ScalingSpec = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed.scaling_type, ScalingType::None));
    }

    #[test]
    fn scaling_type_serialization() {
        let t = ScalingType::Horizontal;
        let json = serde_json::to_string(&t).expect("serialize");
        assert!(json.contains("Horizontal"));
        let parsed: ScalingType = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed, ScalingType::Horizontal));
    }

    #[test]
    fn resource_name_variants() {
        let _ = ResourceName::Cpu;
        let _ = ResourceName::Memory;
        let _ = ResourceName::Storage;
        let _ = ResourceName::Custom("gpu".to_string());
    }

    #[test]
    fn resource_name_serialization() {
        let r = ResourceName::Cpu;
        let json = serde_json::to_string(&r).expect("serialize");
        let parsed: ResourceName = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed, ResourceName::Cpu));

        let r2 = ResourceName::Custom("nvidia.com/gpu".to_string());
        let json2 = serde_json::to_string(&r2).expect("serialize");
        let parsed2: ResourceName = serde_json::from_str(&json2).expect("deserialize");
        assert!(matches!(&parsed2, ResourceName::Custom(s) if s == "nvidia.com/gpu"));
    }

    #[test]
    fn scaling_target_value_variants() {
        let _ = ScalingTargetValue::Utilization(80);
        let _ = ScalingTargetValue::AverageValue("100m".to_string());
        let _ = ScalingTargetValue::Value("1".to_string());
    }

    #[test]
    fn scaling_target_value_serialization() {
        let v = ScalingTargetValue::Utilization(70);
        let json = serde_json::to_string(&v).expect("serialize");
        let parsed: ScalingTargetValue = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed, ScalingTargetValue::Utilization(70)));
    }

    #[test]
    fn scaling_policy_type_variants() {
        let _ = ScalingPolicyType::Pods;
        let _ = ScalingPolicyType::Percent;
    }

    #[test]
    fn scaling_policy_select_variants() {
        let _ = ScalingPolicySelect::Max;
        let _ = ScalingPolicySelect::Min;
        let _ = ScalingPolicySelect::Disabled;
    }

    #[test]
    fn vertical_scaling_mode_variants() {
        let _ = VerticalScalingMode::Off;
        let _ = VerticalScalingMode::Initial;
        let _ = VerticalScalingMode::Auto;
    }

    #[test]
    fn controlled_value_variants() {
        let _ = ControlledValue::RequestsCpuMillis;
        let _ = ControlledValue::RequestsMemoryBytes;
        let _ = ControlledValue::LimitsCpuMillis;
        let _ = ControlledValue::LimitsMemoryBytes;
    }

    #[test]
    fn horizontal_scaling_spec_serialization() {
        let spec = HorizontalScalingSpec {
            min_replicas: 1,
            max_replicas: 10,
            target_metrics: vec![],
            scale_up_policy: None,
            scale_down_policy: None,
            behavior: None,
        };
        let json = serde_json::to_string(&spec).expect("serialize");
        let parsed: HorizontalScalingSpec = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.min_replicas, 1);
        assert_eq!(parsed.max_replicas, 10);
    }

    #[test]
    fn scaling_metric_type_resource_serialization() {
        let m = ScalingMetricType::Resource {
            name: ResourceName::Cpu,
        };
        let json = serde_json::to_string(&m).expect("serialize");
        let parsed: ScalingMetricType = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed, ScalingMetricType::Resource { .. }));
    }

    #[test]
    fn scaling_metric_type_pods_serialization() {
        let m = ScalingMetricType::Pods {
            metric_name: "qps".to_string(),
        };
        let json = serde_json::to_string(&m).expect("serialize");
        let parsed: ScalingMetricType = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed, ScalingMetricType::Pods { metric_name } if metric_name == "qps"));
    }

    #[test]
    fn object_target_serialization() {
        let t = ObjectTarget {
            kind: "Service".to_string(),
            name: "my-svc".to_string(),
            api_version: "v1".to_string(),
        };
        let json = serde_json::to_string(&t).expect("serialize");
        let parsed: ObjectTarget = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.kind, "Service");
        assert_eq!(parsed.name, "my-svc");
    }

    #[test]
    fn custom_scaling_spec_serialization() {
        let mut config = HashMap::new();
        config.insert("key".to_string(), serde_json::json!("value"));
        let spec = CustomScalingSpec {
            provider: "prometheus".to_string(),
            config,
            metrics: vec![CustomMetric {
                name: "cpu_usage".to_string(),
                query: "rate(cpu[5m])".to_string(),
                target: 0.8,
            }],
        };
        let json = serde_json::to_string(&spec).expect("serialize");
        let parsed: CustomScalingSpec = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.provider, "prometheus");
        assert_eq!(parsed.metrics.len(), 1);
        assert!((parsed.metrics[0].target - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn vertical_scaling_spec_serialization() {
        let spec = VerticalScalingSpec {
            update_mode: VerticalScalingMode::Auto,
            resource_policy: None,
            recommenders: vec![VerticalRecommender {
                name: "vpa".to_string(),
            }],
        };
        let json = serde_json::to_string(&spec).expect("serialize");
        let parsed: VerticalScalingSpec = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(parsed.update_mode, VerticalScalingMode::Auto));
        assert_eq!(parsed.recommenders.len(), 1);
    }

    #[test]
    fn scaling_behavior_serialization() {
        let behavior = ScalingBehavior {
            scale_up: None,
            scale_down: None,
        };
        let json = serde_json::to_string(&behavior).expect("serialize");
        let _: ScalingBehavior = serde_json::from_str(&json).expect("deserialize");
    }

    #[test]
    fn metric_selector_serialization() {
        let mut match_labels = HashMap::new();
        match_labels.insert("app".to_string(), "api".to_string());
        let selector = MetricSelector { match_labels };
        let json = serde_json::to_string(&selector).expect("serialize");
        let parsed: MetricSelector = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.match_labels.get("app"), Some(&"api".to_string()));
    }

    #[test]
    fn scaling_spec_with_horizontal() {
        let spec = ScalingSpec {
            scaling_type: ScalingType::Horizontal,
            horizontal: Some(HorizontalScalingSpec {
                min_replicas: 2,
                max_replicas: 20,
                target_metrics: vec![],
                scale_up_policy: None,
                scale_down_policy: None,
                behavior: None,
            }),
            vertical: None,
            custom: None,
        };
        assert!(spec.horizontal.is_some());
        assert_eq!(spec.horizontal.as_ref().unwrap().min_replicas, 2);
    }

    #[test]
    fn scaling_spec_clone() {
        let spec = ScalingSpec::default();
        let cloned = spec;
        assert!(matches!(cloned.scaling_type, ScalingType::None));
    }

    #[test]
    fn scaling_spec_debug() {
        let spec = ScalingSpec::default();
        let debug_str = format!("{spec:?}");
        assert!(debug_str.contains("ScalingSpec"));
    }
}
