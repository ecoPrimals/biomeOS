//! Lifecycle negotiation - request transitions, respect sovereignty

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Request for lifecycle transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleRequest {
    /// What transition we're requesting
    pub transition: LifecycleTransition,

    /// Why we're requesting it
    pub reason: TransitionReason,

    /// How urgent is this
    pub urgency: Urgency,

    /// Who's requesting
    pub requestor: String,
}

/// Lifecycle transition types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleTransition {
    /// Request primal to start
    Start,

    /// Request graceful shutdown
    GracefulStop,

    /// Request immediate stop (emergency)
    EmergencyStop,

    /// Request restart
    Restart,

    /// Request scale down
    ScaleDown,
}

/// Why are we requesting this transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionReason {
    /// Ecosystem health requires it
    EcosystemHealth,

    /// User explicitly requested
    UserRequest,

    /// Resource pressure
    ResourcePressure,

    /// Detected failure
    FailureDetected,

    /// Routine maintenance
    Maintenance,

    /// Other reason
    Other(String),
}

/// How urgent is the request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Urgency {
    /// Low priority, can wait
    Low,

    /// Normal priority
    Normal,

    /// High priority
    High,

    /// Critical, immediate action needed
    Critical,
}

/// Response from primal to lifecycle request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleResponse {
    /// Primal accepts the request
    Accepted,

    /// Primal defers (needs time)
    Deferred {
        /// How long to wait
        duration: Duration,
        /// Why deferred
        reason: String,
    },

    /// Primal refuses the request
    Refused {
        /// Why refused
        reason: String,
    },

    /// Primal doesn't support this transition
    NotSupported,
}

impl LifecycleRequest {
    /// Create new request
    pub fn new(transition: LifecycleTransition, reason: TransitionReason) -> Self {
        Self {
            transition,
            reason,
            urgency: Urgency::Normal,
            requestor: "BiomeOS".to_string(),
        }
    }

    /// Set urgency
    pub fn with_urgency(mut self, urgency: Urgency) -> Self {
        self.urgency = urgency;
        self
    }

    /// Set requestor
    pub fn with_requestor(mut self, requestor: String) -> Self {
        self.requestor = requestor;
        self
    }
}

impl LifecycleResponse {
    /// Check if request was successful
    pub fn is_success(&self) -> bool {
        matches!(self, LifecycleResponse::Accepted)
    }

    /// Check if we should retry
    pub fn should_retry(&self) -> bool {
        matches!(self, LifecycleResponse::Deferred { .. })
    }
}
