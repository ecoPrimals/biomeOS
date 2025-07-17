//! API request types and structures

use super::types::*;
use crate::PrimalContext;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Operation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRequest {
    /// Request ID
    pub request_id: String,
    /// Operation name
    pub operation: String,
    /// Operation parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Operation context
    pub context: PrimalContext,
    /// Operation options
    pub options: OperationOptions,
}

/// Operation options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationOptions {
    /// Operation timeout
    pub timeout: Option<u64>,
    /// Operation priority
    pub priority: OperationPriority,
    /// Retry policy
    pub retry_policy: Option<RetryPolicy>,
    /// Tracing options
    pub tracing: Option<TracingOptions>,
}

/// Inter-primal message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterPrimalMessage {
    /// Message ID
    pub message_id: String,
    /// Source primal
    pub source_primal: String,
    /// Target primal
    pub target_primal: String,
    /// Message type
    pub message_type: MessageType,
    /// Message payload
    pub payload: serde_json::Value,
    /// Message context
    pub context: PrimalContext,
    /// Message metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Configuration update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationUpdate {
    /// Configuration changes
    pub changes: HashMap<String, serde_json::Value>,
    /// Update metadata
    pub metadata: UpdateMetadata,
}

/// Log request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRequest {
    /// Log level filter
    pub level: Option<LogLevel>,
    /// Component filter
    pub component: Option<String>,
    /// Time range
    pub time_range: Option<TimeRange>,
    /// Log limit
    pub limit: Option<u64>,
    /// Log offset
    pub offset: Option<u64>,
    /// Search query
    pub query: Option<String>,
}

/// Metrics request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsRequest {
    /// Metric names
    pub metrics: Vec<String>,
    /// Time range
    pub time_range: Option<TimeRange>,
    /// Aggregation
    pub aggregation: Option<MetricsAggregation>,
    /// Filters
    pub filters: HashMap<String, String>,
}
