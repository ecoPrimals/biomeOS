// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Health monitoring metrics types
//!
//! Extracted from `health.rs` for cohesion. Re-exported via `health` module.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Aggregated health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    /// Response time metrics (milliseconds)
    pub response_time: Option<ResponseTimeMetrics>,

    /// Resource utilization metrics
    pub resources: Option<ResourceMetrics>,

    /// Error rate metrics
    pub errors: Option<ErrorMetrics>,

    /// Availability metrics
    pub availability: Option<AvailabilityMetrics>,

    /// Custom metrics
    pub custom: HashMap<String, serde_json::Value>,
}

/// Response time metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeMetrics {
    /// Average response time in milliseconds
    pub average_ms: f64,
    /// 50th percentile (median) response time in milliseconds
    pub p50_ms: f64,
    /// 95th percentile response time in milliseconds
    pub p95_ms: f64,
    /// 99th percentile response time in milliseconds
    pub p99_ms: f64,
    /// Maximum response time in milliseconds
    pub max_ms: f64,
}

/// Resource utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// CPU utilization (0.0-1.0)
    pub cpu_usage: Option<f64>,

    /// Memory utilization (0.0-1.0)
    pub memory_usage: Option<f64>,

    /// Disk utilization (0.0-1.0)
    pub disk_usage: Option<f64>,

    /// Network utilization (bytes/sec)
    pub network_io: Option<NetworkIoMetrics>,
}

/// Network I/O metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIoMetrics {
    /// Inbound bytes per second
    pub bytes_in_per_sec: f64,
    /// Outbound bytes per second
    pub bytes_out_per_sec: f64,
    /// Inbound packets per second
    pub packets_in_per_sec: f64,
    /// Outbound packets per second
    pub packets_out_per_sec: f64,
}

/// Error rate metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    /// Total error rate (errors/second)
    pub error_rate: f64,

    /// Error rate by category
    pub errors_by_category: HashMap<String, f64>,

    /// Recent error count
    pub recent_errors: u64,
}

/// Availability metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityMetrics {
    /// Uptime percentage (0.0-1.0)
    pub uptime_percentage: f64,

    /// Total uptime in seconds
    pub uptime_seconds: u64,

    /// Total downtime in seconds
    pub downtime_seconds: u64,

    /// Number of outages
    pub outage_count: u64,

    /// Mean time to recovery (MTTR) in seconds
    pub mttr_seconds: Option<f64>,
}
