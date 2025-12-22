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
pub mod startup_shutdown;
pub mod updates;
pub mod scaling;

// Re-export main types from each module
pub use hooks::{
    LifecycleHook, LifecyclePhase, LifecycleAction, LifecycleFailureAction,
    LifecycleCondition, LifecycleConditionType, ConditionOperator,
    HttpMethod, Signal, SignalTarget, WaitCondition,
};

pub use startup_shutdown::{
    StartupSpec, ShutdownSpec, StartupHealthCheck, HealthCheckType, HttpScheme,
};

pub use updates::{
    UpdateStrategySpec, UpdateType, RollingUpdateSpec, RollingUpdateValue,
    CanaryUpdateSpec, CanaryStep, BlueGreenUpdateSpec, UpdateHook, UpdatePhase,
    StepAnalysis, AnalysisCriteria, ThresholdOperator, CanaryAnalysis,
    AnalysisTemplate, MetricsProvider, TrafficSplittingSpec, TrafficRouter,
    HeaderRoutingSpec, CookieRoutingSpec, PromotionCriteria, PromotionCondition,
    PromotionConditionType, PreviewTest, PreviewTestType,
};

pub use scaling::{
    ScalingSpec, ScalingType, HorizontalScalingSpec, VerticalScalingSpec,
    CustomScalingSpec, ScalingMetric, ScalingMetricType, ResourceName,
    ObjectTarget, ScalingTargetValue, MetricSelector, ScalingPolicy,
    ScalingPolicyType, ScalingBehavior, ScalingDirectionBehavior,
    ScalingPolicySelect, VerticalScalingMode, VerticalResourcePolicy,
    ContainerResourcePolicy, ControlledValue, VerticalRecommender,
    CustomMetric,
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