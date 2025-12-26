//! Lifecycle Specifications
//!
//! This module provides comprehensive lifecycle management for services,
//! organized into logical sub-modules for better maintainability.
//!
//! The module is split into:
//! - `hooks`: Lifecycle hooks, phases, actions, and conditions
//! - `startup_shutdown`: Startup and shutdown specifications
//! - `updates`: Update strategies and deployment types
//! - `scaling`: Horizontal, vertical, and custom scaling

use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod hooks;
pub mod scaling;
pub mod startup_shutdown;
pub mod updates;

// Re-export main types from each module
pub use hooks::{
    ConditionOperator, HttpMethod, LifecycleAction, LifecycleCondition, LifecycleConditionType,
    LifecycleFailureAction, LifecycleHook, LifecyclePhase, Signal, SignalTarget, WaitCondition,
};

pub use startup_shutdown::{
    HealthCheckType, HttpScheme, ShutdownSpec, StartupHealthCheck, StartupSpec,
};

pub use updates::{
    AnalysisCriteria, AnalysisTemplate, BlueGreenUpdateSpec, CanaryAnalysis, CanaryStep,
    CanaryUpdateSpec, CookieRoutingSpec, HeaderRoutingSpec, MetricsProvider, PreviewTest,
    PreviewTestType, PromotionCondition, PromotionConditionType, PromotionCriteria,
    RollingUpdateSpec, RollingUpdateValue, StepAnalysis, ThresholdOperator, TrafficRouter,
    TrafficSplittingSpec, UpdateHook, UpdatePhase, UpdateStrategySpec, UpdateType,
};

pub use scaling::{
    ContainerResourcePolicy, ControlledValue, CustomMetric, CustomScalingSpec,
    HorizontalScalingSpec, MetricSelector, ObjectTarget, ResourceName, ScalingBehavior,
    ScalingDirectionBehavior, ScalingMetric, ScalingMetricType, ScalingPolicy, ScalingPolicySelect,
    ScalingPolicyType, ScalingSpec, ScalingTargetValue, ScalingType, VerticalRecommender,
    VerticalResourcePolicy, VerticalScalingMode, VerticalScalingSpec,
};

/// Main Lifecycle specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleSpec {
    /// Lifecycle hooks
    pub hooks: Vec<LifecycleHook>,

    /// Termination grace period
    pub termination_grace_period: Option<Duration>,

    /// Shutdown configuration
    pub shutdown: Option<ShutdownSpec>,

    /// Startup configuration
    pub startup: Option<StartupSpec>,

    /// Update strategy
    pub update_strategy: Option<UpdateStrategySpec>,
}

impl Default for LifecycleSpec {
    fn default() -> Self {
        Self {
            hooks: Vec::new(),
            termination_grace_period: Some(Duration::from_secs(30)),
            shutdown: None,
            startup: None,
            update_strategy: None,
        }
    }
}
