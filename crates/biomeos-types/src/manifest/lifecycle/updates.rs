// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Update Strategies and Deployment Types
//!
//! This module contains update strategy specifications including
//! rolling updates, canary deployments, and blue-green deployments.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use super::hooks::{HttpMethod, LifecycleAction};

/// Update strategy specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStrategySpec {
    /// Update type
    pub update_type: UpdateType,

    /// Rolling update configuration
    pub rolling_update: Option<RollingUpdateSpec>,

    /// Canary update configuration
    pub canary: Option<CanaryUpdateSpec>,

    /// Blue-green update configuration
    pub blue_green: Option<BlueGreenUpdateSpec>,

    /// Update hooks
    pub hooks: Vec<UpdateHook>,
}

/// Update types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateType {
    /// Recreate all instances
    Recreate,
    /// Rolling update
    RollingUpdate,
    /// Canary deployment
    Canary,
    /// Blue-green deployment
    BlueGreen,
}

/// Rolling update specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollingUpdateSpec {
    /// Maximum unavailable instances
    pub max_unavailable: RollingUpdateValue,

    /// Maximum surge instances
    pub max_surge: RollingUpdateValue,

    /// Update batch size
    pub batch_size: Option<u32>,

    /// Batch delay
    pub batch_delay: Option<Duration>,
}

/// Rolling update values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollingUpdateValue {
    /// Percentage
    Percent(u32),
    /// Absolute count
    Count(u32),
}

/// Canary update specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryUpdateSpec {
    /// Canary steps
    pub steps: Vec<CanaryStep>,

    /// Analysis configuration
    pub analysis: Option<CanaryAnalysis>,

    /// Traffic splitting
    pub traffic_splitting: Option<TrafficSplittingSpec>,

    /// Promotion criteria
    pub promotion: Option<PromotionCriteria>,
}

/// Canary step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryStep {
    /// Step name
    pub name: String,

    /// Traffic weight percentage
    pub weight: u32,

    /// Step duration
    pub duration: Option<Duration>,

    /// Pause before step
    pub pause: Option<bool>,

    /// Step analysis
    pub analysis: Option<StepAnalysis>,
}

/// Step analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepAnalysis {
    /// Success criteria
    pub success_criteria: Vec<AnalysisCriteria>,

    /// Failure criteria
    pub failure_criteria: Vec<AnalysisCriteria>,

    /// Analysis duration
    pub duration: Duration,
}

/// Analysis criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisCriteria {
    /// Metric name
    pub metric: String,

    /// Threshold operator
    pub operator: ThresholdOperator,

    /// Threshold value
    pub threshold: f64,

    /// Evaluation window
    pub window: Duration,
}

/// Threshold operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThresholdOperator {
    /// Value is greater than threshold
    GreaterThan,
    /// Value is less than threshold
    LessThan,
    /// Value is greater than or equal to threshold
    GreaterThanOrEqual,
    /// Value is less than or equal to threshold
    LessThanOrEqual,
    /// Value equals threshold
    Equal,
    /// Value does not equal threshold
    NotEqual,
}

/// Canary analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryAnalysis {
    /// Analysis templates
    pub templates: Vec<AnalysisTemplate>,

    /// Analysis interval
    pub interval: Duration,

    /// Analysis timeout
    pub timeout: Duration,
}

/// Analysis template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisTemplate {
    /// Template name
    pub name: String,

    /// Metrics provider
    pub provider: MetricsProvider,

    /// Query
    pub query: String,

    /// Success condition
    pub success_condition: String,

    /// Failure condition
    pub failure_condition: Option<String>,
}

/// Metrics providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricsProvider {
    /// Prometheus metrics system
    Prometheus,
    /// `DataDog` monitoring platform
    DataDog,
    /// New Relic observability platform
    NewRelic,
    /// Grafana monitoring stack
    Grafana,
    /// AWS `CloudWatch`
    CloudWatch,
    /// Custom metrics provider
    Custom(String),
}

/// Traffic splitting specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficSplittingSpec {
    /// Traffic router
    pub router: TrafficRouter,

    /// Header-based routing
    pub header_routing: Option<HeaderRoutingSpec>,

    /// Cookie-based routing
    pub cookie_routing: Option<CookieRoutingSpec>,
}

/// Traffic routers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrafficRouter {
    /// Istio service mesh
    Istio,
    /// Nginx ingress controller
    Nginx,
    /// Traefik proxy
    Traefik,
    /// Ambassador API gateway
    Ambassador,
    /// Custom traffic router
    Custom(String),
}

/// Header routing specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderRoutingSpec {
    /// Header name
    pub header_name: String,

    /// Header value
    pub header_value: String,

    /// Routing percentage
    pub percentage: Option<u32>,
}

/// Cookie routing specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieRoutingSpec {
    /// Cookie name
    pub cookie_name: String,

    /// Cookie value
    pub cookie_value: String,

    /// Routing percentage
    pub percentage: Option<u32>,
}

/// Promotion criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionCriteria {
    /// Auto-promotion enabled
    pub auto_promotion: bool,

    /// Promotion conditions
    pub conditions: Vec<PromotionCondition>,

    /// Manual approval required
    pub manual_approval: bool,

    /// Promotion timeout
    pub timeout: Option<Duration>,
}

/// Promotion condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionCondition {
    /// Condition type
    pub condition_type: PromotionConditionType,

    /// Threshold
    pub threshold: f64,

    /// Duration
    pub duration: Duration,
}

/// Promotion condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromotionConditionType {
    /// Success rate percentage
    SuccessRate,
    /// Error rate percentage
    ErrorRate,
    /// Response time in milliseconds
    ResponseTime,
    /// Total request count
    RequestCount,
    /// Custom promotion condition
    Custom(String),
}

/// Blue-green update specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueGreenUpdateSpec {
    /// Preview service
    pub preview_service: String,

    /// Active service
    pub active_service: String,

    /// Auto-promotion enabled
    pub auto_promotion: bool,

    /// Preview tests
    pub preview_tests: Vec<PreviewTest>,

    /// Promotion delay
    pub promotion_delay: Option<Duration>,

    /// Rollback on failure
    pub auto_rollback: bool,
}

/// Preview test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewTest {
    /// Test name
    pub name: String,

    /// Test type
    pub test_type: PreviewTestType,

    /// Test timeout
    pub timeout: Duration,

    /// Required for promotion
    pub required: bool,
}

/// Preview test types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreviewTestType {
    /// HTTP test
    Http {
        /// URL to test
        url: String,
        /// HTTP method to use
        method: HttpMethod,
        /// Expected HTTP status code
        expected_status: u16,
    },
    /// Load test
    LoadTest {
        /// Test duration
        duration: Duration,
        /// Number of concurrent users
        concurrent_users: u32,
        /// Target URL for the load test
        target_url: String,
    },
    /// Custom test
    Custom {
        /// Test runner executable
        test_runner: String,
        /// Test configuration
        config: HashMap<String, String>,
    },
}

/// Update hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateHook {
    /// Hook name
    pub name: String,

    /// Hook phase
    pub phase: UpdatePhase,

    /// Hook action
    pub action: LifecycleAction,

    /// Hook timeout
    pub timeout: Option<Duration>,
}

/// Update phases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdatePhase {
    /// Before the update begins
    PreUpdate,
    /// After the update completes
    PostUpdate,
    /// Before a rollback begins
    PreRollback,
    /// After a rollback completes
    PostRollback,
}
