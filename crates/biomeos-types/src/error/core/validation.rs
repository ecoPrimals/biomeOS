// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use serde::{Deserialize, Serialize};

/// Individual validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Field that failed validation
    pub field: String,

    /// Error message
    pub message: String,

    /// Validation code
    pub code: String,

    /// Rejected value
    pub rejected_value: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_error_can_be_constructed() {
        let ve = ValidationError {
            field: "email".to_string(),
            message: "invalid format".to_string(),
            code: "format".to_string(),
            rejected_value: Some(serde_json::json!("bad@")),
        };
        assert_eq!(ve.field, "email");
        assert_eq!(ve.message, "invalid format");
        assert_eq!(ve.code, "format");
        assert!(ve.rejected_value.is_some());
    }
}
