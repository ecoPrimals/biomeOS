//! Wrapped format adapter
//!
//! Handles responses wrapped in ApiResponse { success, data, error }.
//! Explicit success/failure indication in response body.
//!
//! Success Example:
//! ```json
//! {
//!   "success": true,
//!   "data": {
//!     "encryption_tag": "beardog:family:iidn:tower_abc",
//!     "family_id": "iidn"
//!   }
//! }
//! ```
//!
//! Error Example:
//! ```json
//! {
//!   "success": false,
//!   "error": {
//!     "code": "unauthorized",
//!     "message": "Insufficient permissions"
//!   }
//! }
//! ```

use reqwest::Response;
use serde::de::DeserializeOwned;

use crate::primal_client::error::{ApiError, Result};

/// Adapter for wrapped responses
#[derive(Clone, Debug, Default)]
pub struct WrappedFormatAdapter;

impl WrappedFormatAdapter {
    pub fn new() -> Self {
        Self
    }
    
    /// Parse response into expected type
    pub async fn parse<T>(&self, response: Response) -> Result<T>
    where
        T: DeserializeOwned + Send,
    {
        // Manual deserialization to avoid Default bound on T
        let text = response.text().await.map_err(|e| ApiError::ParseError {
            message: format!("Failed to read response body: {}", e),
            body: None,
        })?;
        
        // Parse as generic JSON first
        let json: serde_json::Value = serde_json::from_str(&text).map_err(|e| ApiError::ParseError {
            message: format!("Failed to parse wrapped response: {}", e),
            body: Some(text.clone()),
        })?;
        
        // Extract fields manually
        let success = json.get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        if success {
            // Success - extract and parse data
            let data_value = json.get("data").ok_or_else(|| ApiError::ParseError {
                message: "Response marked as success but no data field present".to_string(),
                body: Some(text.clone()),
            })?;
            
            serde_json::from_value::<T>(data_value.clone()).map_err(|e| ApiError::ParseError {
                message: format!("Failed to parse data field: {}", e),
                body: Some(text),
            })
        } else {
            // Error - extract error info
            let error = json.get("error");
            let code = error
                .and_then(|e| e.get("code"))
                .and_then(|c| c.as_str())
                .unwrap_or("unknown");
            let message = error
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            match code {
                "unauthorized" => Err(ApiError::Unauthorized { message }),
                "forbidden" => Err(ApiError::Forbidden { message }),
                "not_found" => Err(ApiError::NotFound { resource: message }),
                _ => Err(ApiError::Other { message }),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wrapped_success() {
        // Placeholder for tests
    }

    #[tokio::test]
    async fn test_wrapped_error() {
        // Placeholder for tests
    }
}

