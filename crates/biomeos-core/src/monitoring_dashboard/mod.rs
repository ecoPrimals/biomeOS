//! # Real-Time Monitoring Dashboard for biomeOS
//!
//! This module provides a comprehensive monitoring dashboard that aggregates
//! metrics from all primals in the federation, provides real-time alerting,
//! and delivers actionable insights for ecosystem health management.

pub mod alerts;
pub mod collectors;
pub mod config;
pub mod dashboard;
pub mod metrics;
pub mod notifications;
pub mod types;
pub mod visualization;

// Re-export main types
pub use alerts::*;
pub use collectors::*;
pub use config::{
    AggregationInterval, AlertCondition, AlertConfig, AlertDestination, AlertDestinationType,
    AlertFrequency, AlertSeverity, ChartType, ColorScheme, ComparisonOperator, DashboardConfig,
    TimeRange, VisualizationConfig,
};
pub use dashboard::*;
pub use metrics::{
    AggregatedMetric, AggregationFunction, GroupKey, MetricsAggregator, MetricsProcessor,
    MetricsQuery, MetricsStorage, QueryResult, TimeRange as MetricsTimeRange, TrendAnalysis,
};
pub use notifications::*;
pub use types::*;
pub use visualization::*;
