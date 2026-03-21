// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Distribution error types.

use serde::Serialize;

/// Distribution error response
#[derive(Debug, Serialize)]
pub struct DistError {
    /// Error message
    pub error: String,
    /// Error code
    pub code: String,
}
