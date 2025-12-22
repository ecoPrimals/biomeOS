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
        command: Vec<String>,
        environment: HashMap<String, String>,
        working_dir: Option<String>,
    },
    /// HTTP request
    Http {
        url: String,
        method: HttpMethod,
        headers: HashMap<String, String>,
        body: Option<String>,
        timeout: Option<Duration>,
    },
    /// TCP socket check
    TcpSocket {
        host: String,
        port: u16,
        timeout: Option<Duration>,
    },
    /// Send signal
    Signal {
        signal: Signal,
        target: SignalTarget,
    },
    /// Wait for condition
    Wait {
        condition: WaitCondition,
        timeout: Duration,
    },
    /// Custom action
    Custom {
        action_type: String,
        config: HashMap<String, serde_json::Value>,
    },
}

/// HTTP methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

/// System signals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Signal {
    Sigterm,
    Sigkill,
    Sigint,
    Sighup,
    Sigusr1,
    Sigusr2,
    Custom(String),
}

/// Signal targets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignalTarget {
    MainProcess,
    AllProcesses,
    ProcessGroup(String),
    ProcessById(u32),
}

/// Wait conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WaitCondition {
    /// Wait for port to be available
    PortAvailable(u16),
    /// Wait for HTTP endpoint to respond
    HttpResponse {
        url: String,
        expected_status: u16,
    },
    /// Wait for file to exist
    FileExists(String),
    /// Wait for process to exit
    ProcessExit(u32),
    /// Wait for custom condition
    Custom {
        condition_type: String,
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
    Equals,
    NotEquals,
    Contains,
    NotContains,
    GreaterThan,
    LessThan,
    Exists,
    NotExists,
} 