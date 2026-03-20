// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! API monitoring and log streaming types for the TUI dashboard.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// API endpoint status for monitoring ingestion
#[derive(Debug, Clone)]
pub struct ApiEndpointStatus {
    /// Endpoint URL
    pub endpoint: String,
    /// Current connection status
    pub status: ApiStatus,
    /// Timestamp of last successful API call
    pub last_successful_call: Option<Instant>,
    /// Number of consecutive errors
    pub error_count: u32,
    /// Average response time across recent calls
    pub average_response_time: Duration,
}

/// API connection status
#[derive(Debug, Clone)]
pub enum ApiStatus {
    /// Successfully connected
    Connected,
    /// Not connected
    Disconnected,
    /// Connection error
    Error {
        /// Error description
        message: String,
    },
    /// Connection timed out
    Timeout,
}

/// API errors for monitoring
#[derive(Debug, Clone)]
pub struct ApiError {
    /// When the error occurred
    pub timestamp: Instant,
    /// Endpoint that produced the error
    pub endpoint: String,
    /// Error description
    pub error: String,
    /// Number of retry attempts made
    pub retry_count: u32,
}

/// Log entry from streaming
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// When the log entry was created
    pub timestamp: Instant,
    /// Source component that produced the log
    pub source: String,
    /// Log severity level
    pub level: LogLevel,
    /// Log message content
    pub message: String,
    /// Additional structured metadata
    pub metadata: HashMap<String, String>,
}

/// Log levels
#[derive(Debug, Clone)]
pub enum LogLevel {
    /// Trace-level detail
    Trace,
    /// Debug information
    Debug,
    /// Informational message
    Info,
    /// Warning message
    Warn,
    /// Error message
    Error,
}

/// Log filters for streaming
#[derive(Debug, Clone)]
pub struct LogFilter {
    /// Filter by source pattern (glob)
    pub source_pattern: Option<String>,
    /// Filter by minimum log level
    pub level_filter: Option<LogLevel>,
    /// Filter by message content pattern
    pub message_pattern: Option<String>,
    /// Filter by time range
    pub time_range: Option<(Instant, Instant)>,
}
