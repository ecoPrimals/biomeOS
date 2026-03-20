// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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

#[allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_request_new() {
        let req = LifecycleRequest::new(
            LifecycleTransition::GracefulStop,
            TransitionReason::ResourcePressure,
        );
        assert!(matches!(req.transition, LifecycleTransition::GracefulStop));
        assert!(matches!(req.reason, TransitionReason::ResourcePressure));
        assert!(matches!(req.urgency, Urgency::Normal));
        assert_eq!(req.requestor, "BiomeOS");
    }

    #[test]
    fn test_lifecycle_request_with_urgency() {
        let req = LifecycleRequest::new(LifecycleTransition::Start, TransitionReason::UserRequest)
            .with_urgency(Urgency::Critical);
        assert!(matches!(req.urgency, Urgency::Critical));
    }

    #[test]
    fn test_lifecycle_request_with_requestor() {
        let req =
            LifecycleRequest::new(LifecycleTransition::Restart, TransitionReason::Maintenance)
                .with_requestor("admin".to_string());
        assert_eq!(req.requestor, "admin");
    }

    #[test]
    fn test_lifecycle_response_accepted() {
        let resp = LifecycleResponse::Accepted;
        assert!(resp.is_success());
        assert!(!resp.should_retry());
    }

    #[test]
    fn test_lifecycle_response_deferred() {
        let resp = LifecycleResponse::Deferred {
            duration: Duration::from_secs(10),
            reason: "busy".to_string(),
        };
        assert!(!resp.is_success());
        assert!(resp.should_retry());
    }

    #[test]
    fn test_lifecycle_response_refused() {
        let resp = LifecycleResponse::Refused {
            reason: "not now".to_string(),
        };
        assert!(!resp.is_success());
        assert!(!resp.should_retry());
    }

    #[test]
    fn test_lifecycle_response_not_supported() {
        let resp = LifecycleResponse::NotSupported;
        assert!(!resp.is_success());
        assert!(!resp.should_retry());
    }

    #[test]
    fn test_lifecycle_transition_serialization() {
        for transition in [
            LifecycleTransition::Start,
            LifecycleTransition::GracefulStop,
            LifecycleTransition::EmergencyStop,
            LifecycleTransition::Restart,
            LifecycleTransition::ScaleDown,
        ] {
            let json = serde_json::to_string(&transition).expect("serialize");
            let restored: LifecycleTransition = serde_json::from_str(&json).expect("deserialize");
            assert!(matches!(
                (transition, restored),
                (LifecycleTransition::Start, LifecycleTransition::Start)
                    | (
                        LifecycleTransition::GracefulStop,
                        LifecycleTransition::GracefulStop
                    )
                    | (
                        LifecycleTransition::EmergencyStop,
                        LifecycleTransition::EmergencyStop
                    )
                    | (LifecycleTransition::Restart, LifecycleTransition::Restart)
                    | (
                        LifecycleTransition::ScaleDown,
                        LifecycleTransition::ScaleDown
                    )
            ));
        }
    }

    #[test]
    fn test_transition_reason_serialization() {
        let reason = TransitionReason::Other("custom".to_string());
        let json = serde_json::to_string(&reason).expect("serialize");
        let restored: TransitionReason = serde_json::from_str(&json).expect("deserialize");
        match (&reason, &restored) {
            (TransitionReason::Other(a), TransitionReason::Other(b)) => assert_eq!(a, b),
            _ => panic!("Expected Other variant"),
        }
    }

    #[test]
    fn test_urgency_serialization() {
        let urgency = Urgency::High;
        let json = serde_json::to_string(&urgency).expect("serialize");
        let restored: Urgency = serde_json::from_str(&json).expect("deserialize");
        assert!(matches!(restored, Urgency::High));
    }
}
