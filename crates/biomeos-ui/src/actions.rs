//! User actions
//!
//! Actions represent user interactions that need to be handled by the orchestrator.

use serde::{Deserialize, Serialize};

/// User action - represents a user interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserAction {
    /// Assign a device to a primal
    AssignDevice {
        /// Device ID
        device_id: String,

        /// Primal ID
        primal_id: String,
    },

    /// Unassign a device from a primal
    UnassignDevice {
        /// Device ID
        device_id: String,
    },

    /// Start a primal
    StartPrimal {
        /// Primal name
        primal_name: String,
    },

    /// Stop a primal
    StopPrimal {
        /// Primal ID
        primal_id: String,
    },

    /// Restart a primal
    RestartPrimal {
        /// Primal ID
        primal_id: String,
    },

    /// Accept an AI suggestion
    AcceptSuggestion {
        /// Suggestion ID
        suggestion_id: String,
    },

    /// Dismiss an AI suggestion
    DismissSuggestion {
        /// Suggestion ID
        suggestion_id: String,
    },

    /// Refresh the UI state
    Refresh,
}

/// Action result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionResult {
    /// Action succeeded
    Success {
        /// Success message
        message: String,
    },

    /// Action failed
    Error {
        /// Error message
        message: String,
    },
}

impl ActionResult {
    /// Create a success result
    pub fn success(message: impl Into<String>) -> Self {
        Self::Success {
            message: message.into(),
        }
    }

    /// Create an error result
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
        }
    }

    /// Check if the result is a success
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Check if the result is an error
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }
}
