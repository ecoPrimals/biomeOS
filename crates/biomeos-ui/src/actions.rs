// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_assign_device() {
        let action = UserAction::AssignDevice {
            device_id: "gpu0".to_string(),
            primal_id: "toadstool-1".to_string(),
        };

        match action {
            UserAction::AssignDevice {
                device_id,
                primal_id,
            } => {
                assert_eq!(device_id, "gpu0");
                assert_eq!(primal_id, "toadstool-1");
            }
            _ => panic!("Wrong action type"),
        }
    }

    #[test]
    fn test_action_unassign_device() {
        let action = UserAction::UnassignDevice {
            device_id: "gpu0".to_string(),
        };

        match action {
            UserAction::UnassignDevice { device_id } => {
                assert_eq!(device_id, "gpu0");
            }
            _ => panic!("Wrong action type"),
        }
    }

    #[test]
    fn test_action_start_primal() {
        let action = UserAction::StartPrimal {
            primal_name: "beardog".to_string(),
        };

        match action {
            UserAction::StartPrimal { primal_name } => {
                assert_eq!(primal_name, "beardog");
            }
            _ => panic!("Wrong action type"),
        }
    }

    #[test]
    fn test_action_stop_primal() {
        let action = UserAction::StopPrimal {
            primal_id: "beardog-1".to_string(),
        };

        match action {
            UserAction::StopPrimal { primal_id } => {
                assert_eq!(primal_id, "beardog-1");
            }
            _ => panic!("Wrong action type"),
        }
    }

    #[test]
    fn test_action_restart_primal() {
        let action = UserAction::RestartPrimal {
            primal_id: "beardog-1".to_string(),
        };

        match action {
            UserAction::RestartPrimal { primal_id } => {
                assert_eq!(primal_id, "beardog-1");
            }
            _ => panic!("Wrong action type"),
        }
    }

    #[test]
    fn test_action_accept_suggestion() {
        let action = UserAction::AcceptSuggestion {
            suggestion_id: "sug-123".to_string(),
        };

        match action {
            UserAction::AcceptSuggestion { suggestion_id } => {
                assert_eq!(suggestion_id, "sug-123");
            }
            _ => panic!("Wrong action type"),
        }
    }

    #[test]
    fn test_action_dismiss_suggestion() {
        let action = UserAction::DismissSuggestion {
            suggestion_id: "sug-456".to_string(),
        };

        match action {
            UserAction::DismissSuggestion { suggestion_id } => {
                assert_eq!(suggestion_id, "sug-456");
            }
            _ => panic!("Wrong action type"),
        }
    }

    #[test]
    fn test_action_refresh() {
        let action = UserAction::Refresh;
        assert!(matches!(action, UserAction::Refresh));
    }

    #[test]
    fn test_action_result_success() {
        let result = ActionResult::success("Operation completed");
        assert!(result.is_success());
        assert!(!result.is_error());

        match result {
            ActionResult::Success { message } => {
                assert_eq!(message, "Operation completed");
            }
            ActionResult::Error { .. } => panic!("Wrong result type"),
        }
    }

    #[test]
    fn test_action_result_error() {
        let result = ActionResult::error("Operation failed");
        assert!(!result.is_success());
        assert!(result.is_error());

        match result {
            ActionResult::Error { message } => {
                assert_eq!(message, "Operation failed");
            }
            ActionResult::Success { .. } => panic!("Wrong result type"),
        }
    }

    #[test]
    fn test_action_serialization() {
        let action = UserAction::AssignDevice {
            device_id: "test-device".to_string(),
            primal_id: "test-primal".to_string(),
        };

        let json = serde_json::to_string(&action).expect("Should serialize");
        assert!(json.contains("test-device"));
        assert!(json.contains("test-primal"));

        let deserialized: UserAction = serde_json::from_str(&json).expect("Should deserialize");
        match deserialized {
            UserAction::AssignDevice {
                device_id,
                primal_id,
            } => {
                assert_eq!(device_id, "test-device");
                assert_eq!(primal_id, "test-primal");
            }
            _ => panic!("Wrong action type after deserialization"),
        }
    }

    #[test]
    fn test_action_result_serialization() {
        let result = ActionResult::success("test message");

        let json = serde_json::to_string(&result).expect("Should serialize");
        assert!(json.contains("test message"));

        let deserialized: ActionResult = serde_json::from_str(&json).expect("Should deserialize");
        assert!(deserialized.is_success());
    }
}
