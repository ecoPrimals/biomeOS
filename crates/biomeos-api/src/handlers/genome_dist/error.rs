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
