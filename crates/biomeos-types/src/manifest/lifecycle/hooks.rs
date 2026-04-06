// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Lifecycle Hooks and Actions
//!
//! This module contains lifecycle hook definitions, phases, actions,
//! and conditions for managing service lifecycle events.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Lifecycle hooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleHook {
    /// Hook name
    pub name: String,

    /// Hook phase
    pub phase: LifecyclePhase,

    /// Hook action
    pub action: LifecycleAction,

    /// Hook timeout
    pub timeout: Option<Duration>,

    /// Hook retries
    pub retries: Option<u32>,

    /// Failure policy
    pub on_failure: LifecycleFailureAction,

    /// Hook conditions
    pub conditions: Vec<LifecycleCondition>,
}

/// Lifecycle phases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecyclePhase {
    /// Pre-start phase
    PreStart,
    /// Post-start phase
    PostStart,
    /// Pre-stop phase
    PreStop,
    /// Post-stop phase
    PostStop,
    /// Pre-update phase
    PreUpdate,
    /// Post-update phase
    PostUpdate,
    /// Pre-scaling phase
    PreScaling,
    /// Post-scaling phase
    PostScaling,
    /// Custom phase
    Custom(String),
}

/// Lifecycle actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleAction {
    /// Execute command
    Exec {
        /// Command and arguments
        command: Vec<String>,
        /// Environment variables
        environment: HashMap<String, String>,
        /// Working directory
        working_dir: Option<String>,
    },
    /// HTTP request
    Http {
        /// Request URL
        url: String,
        /// HTTP method
        method: HttpMethod,
        /// Request headers
        headers: HashMap<String, String>,
        /// Request body
        body: Option<String>,
        /// Request timeout
        timeout: Option<Duration>,
    },
    /// TCP socket check
    TcpSocket {
        /// Target host
        host: String,
        /// Target port
        port: u16,
        /// Connection timeout
        timeout: Option<Duration>,
    },
    /// Send signal
    Signal {
        /// Signal to send
        signal: Signal,
        /// Signal target
        target: SignalTarget,
    },
    /// Wait for condition
    Wait {
        /// Condition to wait for
        condition: WaitCondition,
        /// Maximum wait time
        timeout: Duration,
    },
    /// Custom action
    Custom {
        /// Action type identifier
        action_type: String,
        /// Action configuration
        config: HashMap<String, serde_json::Value>,
    },
}

/// HTTP methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    /// GET request
    Get,
    /// POST request
    Post,
    /// PUT request
    Put,
    /// DELETE request
    Delete,
    /// PATCH request
    Patch,
    /// HEAD request
    Head,
    /// OPTIONS request
    Options,
}

/// System signals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Signal {
    /// Graceful termination
    Sigterm,
    /// Forced kill
    Sigkill,
    /// Interrupt (Ctrl-C)
    Sigint,
    /// Hang-up / reload config
    Sighup,
    /// User-defined signal 1
    Sigusr1,
    /// User-defined signal 2
    Sigusr2,
    /// Custom signal
    Custom(String),
}

/// Signal targets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalTarget {
    /// Main process only
    MainProcess,
    /// All child processes
    AllProcesses,
    /// Named process group
    ProcessGroup(String),
    /// Specific process by PID
    ProcessById(u32),
}

/// Wait conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WaitCondition {
    /// Wait for port to be available
    PortAvailable(u16),
    /// Wait for HTTP endpoint to respond
    HttpResponse {
        /// URL to poll
        url: String,
        /// Expected HTTP status code
        expected_status: u16,
    },
    /// Wait for file to exist
    FileExists(String),
    /// Wait for process to exit
    ProcessExit(u32),
    /// Wait for custom condition
    Custom {
        /// Condition type identifier
        condition_type: String,
        /// Condition configuration
        config: HashMap<String, String>,
    },
}

/// Lifecycle failure actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleFailureAction {
    /// Continue execution
    Continue,
    /// Fail the operation
    Fail,
    /// Retry the hook
    Retry,
    /// Abort the entire lifecycle
    Abort,
    /// Custom action
    Custom(String),
}

/// Lifecycle conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleCondition {
    /// Condition type
    pub condition_type: LifecycleConditionType,

    /// Condition operator
    pub operator: ConditionOperator,

    /// Expected value
    pub value: String,
}

/// Lifecycle condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleConditionType {
    /// Environment variable
    EnvironmentVariable(String),
    /// Service status
    ServiceStatus(String),
    /// File content
    FileContent(String),
    /// HTTP response
    HttpResponse(String),
    /// Custom condition
    Custom(String),
}

/// Condition operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    /// Exact equality
    Equals,
    /// Not equal
    NotEquals,
    /// Contains substring
    Contains,
    /// Does not contain substring
    NotContains,
    /// Greater than (numeric)
    GreaterThan,
    /// Less than (numeric)
    LessThan,
    /// Value exists / is defined
    Exists,
    /// Value does not exist
    NotExists,
}
