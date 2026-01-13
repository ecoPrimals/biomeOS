//! Unwrapped format adapter
//!
//! Handles responses with direct data, no wrapper.
//! HTTP status codes indicate success/failure.
//!
//! Example:
//! ```json
//! {
//!   "encryption_tag": "beardog:family:iidn:tower_abc",
//!   "family_id": "iidn"
//! }
//! ```

use reqwest::Response;
use serde::de::DeserializeOwned;

use crate::primal_client::error::{ApiError, Result};

/// Adapter for unwrapped responses
#[derive(Clone, Debug, Default)]
pub struct UnwrappedFormatAdapter;

impl UnwrappedFormatAdapter {
    pub fn new() -> Self {
        Self
    }

    /// Parse response into expected type
    pub async fn parse<T>(&self, response: Response) -> Result<T>
    where
        T: DeserializeOwned + Send,
    {
        let status = response.status();

        if status.is_success() {
            // Success - parse data directly
            response
                .json::<T>()
                .await
                .map_err(|e| ApiError::ParseError {
                    message: format!("Failed to parse unwrapped response: {}", e),
                    body: None,
                })
        } else {
            // Error - extract error info
            Err(self.extract_error(response).await)
        }
    }

    /// Extract error information from response
    async fn extract_error(&self, response: Response) -> ApiError {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| String::from("<no body>"));

        match status.as_u16() {
            401 => ApiError::Unauthorized { message: body },
            403 => ApiError::Forbidden { message: body },
            404 => ApiError::NotFound { resource: body },
            500..=599 => ApiError::ServerError {
                status: status.as_u16(),
                message: body,
            },
            _ => ApiError::Other {
                message: format!("HTTP {}: {}", status, body),
            },
        }
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_unwrapped_success() {
        // This is a placeholder for when we have mock responses
        // Real tests will be added when integration testing infrastructure is ready
    }
}
