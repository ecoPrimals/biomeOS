// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Startup and Shutdown Specifications
//!
//! This module contains startup and shutdown specifications,
//! including health checks and lifecycle hooks.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use super::hooks::LifecycleHook;

/// Shutdown specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownSpec {
    /// Graceful shutdown timeout
    pub graceful_timeout: Duration,

    /// Force shutdown timeout
    pub force_timeout: Duration,

    /// Shutdown order
    pub order: Option<i32>,

    /// Pre-shutdown hooks
    pub pre_hooks: Vec<LifecycleHook>,

    /// Post-shutdown hooks
    pub post_hooks: Vec<LifecycleHook>,
}

/// Startup specification  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupSpec {
    /// Startup timeout
    pub timeout: Duration,

    /// Initial delay
    pub initial_delay: Option<Duration>,

    /// Startup order
    pub order: Option<i32>,

    /// Pre-startup hooks
    pub pre_hooks: Vec<LifecycleHook>,

    /// Post-startup hooks
    pub post_hooks: Vec<LifecycleHook>,

    /// Health check after startup
    pub health_check: Option<StartupHealthCheck>,
}

/// Startup health check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupHealthCheck {
    /// Check type
    pub check_type: HealthCheckType,

    /// Check interval
    pub interval: Duration,

    /// Check timeout
    pub timeout: Duration,

    /// Success threshold
    pub success_threshold: u32,

    /// Failure threshold
    pub failure_threshold: u32,
}

/// Health check types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthCheckType {
    /// HTTP health check
    Http {
        /// HTTP path to check
        path: String,
        /// Port number
        port: u16,
        /// HTTP scheme
        scheme: HttpScheme,
        /// HTTP headers
        headers: HashMap<String, String>,
    },
    /// TCP health check
    Tcp {
        /// Port number
        port: u16,
    },
    /// Command health check
    Exec {
        /// Command to execute
        command: Vec<String>,
    },
    /// gRPC health check
    Grpc {
        /// Port number
        port: u16,
        /// Service name
        service: Option<String>,
    },
}

/// HTTP schemes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpScheme {
    /// Plain HTTP
    Http,
    /// HTTPS (TLS)
    Https,
}
