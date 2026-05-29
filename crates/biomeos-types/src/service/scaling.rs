// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Service scaling configuration, policies, and metrics.

use serde::{Deserialize, Serialize};

/// Service scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceScaling {
    /// Scaling type
    pub scaling_type: ScalingType,

    /// Minimum replicas
    pub min_replicas: u32,

    /// Maximum replicas
    pub max_replicas: u32,

    /// Scaling policies
    pub policies: Vec<ScalingPolicy>,

    /// Scaling metrics
    pub metrics: Vec<ScalingMetric>,
}

/// Scaling types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingType {
    /// Manual scaling
    Manual,

    /// Horizontal pod autoscaling
    Hpa,

    /// Vertical pod autoscaling
    Vpa,

    /// Custom scaling
    Custom(String),
}

/// Scaling policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingPolicy {
    /// Policy name
    pub name: String,

    /// Scaling direction
    pub direction: ScalingDirection,

    /// Scaling amount
    pub amount: ScalingAmount,

    /// Policy cooldown
    pub cooldown: u32,
}

/// Scaling directions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingDirection {
    /// Scale up (add replicas)
    Up,
    /// Scale down (remove replicas)
    Down,
    /// Scale in either direction
    Both,
}

/// Scaling amount
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingAmount {
    /// Fixed number of replicas
    Fixed(u32),

    /// Percentage of current replicas
    Percent(u32),
}

/// Scaling metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingMetric {
    /// Metric name
    pub name: String,

    /// Metric type
    pub metric_type: ScalingMetricType,

    /// Target value
    pub target_value: f64,

    /// Current value
    pub current_value: Option<f64>,
}

/// Scaling metric types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingMetricType {
    /// CPU utilization
    CpuUtilization,

    /// Memory utilization
    MemoryUtilization,

    /// Network utilization
    NetworkUtilization,

    /// Request rate
    RequestRate,

    /// Response time
    ResponseTime,

    /// Queue length
    QueueLength,

    /// Custom metric
    Custom {
        /// Metric source
        source: String,
        /// Query/selector
        query: String,
    },
}

impl Default for ServiceScaling {
    fn default() -> Self {
        Self {
            scaling_type: ScalingType::Manual,
            min_replicas: 1,
            max_replicas: 1,
            policies: vec![],
            metrics: vec![],
        }
    }
}
