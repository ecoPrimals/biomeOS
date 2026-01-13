//! Auto-detect format adapter
//!
//! Automatically detects response format by trying multiple adapters.
//! Uses HTTP status codes as the primary indicator, then tries:
//! 1. Format hint (if provided)
//! 2. Unwrapped format
//! 3. Wrapped format
//!
//! Caches successful format for future requests.

use reqwest::Response;
use serde::de::DeserializeOwned;
use tracing::{debug, warn};

use super::{UnwrappedFormatAdapter, WrappedFormatAdapter};
use crate::primal_client::error::{ApiError, Result};
use crate::primal_client::handle::FormatHint;

/// Auto-detecting format adapter
#[derive(Clone, Debug)]
pub struct AutoFormatAdapter {
    unwrapped: UnwrappedFormatAdapter,
    wrapped: WrappedFormatAdapter,
}

impl Default for AutoFormatAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoFormatAdapter {
    pub fn new() -> Self {
        Self {
            unwrapped: UnwrappedFormatAdapter::new(),
            wrapped: WrappedFormatAdapter::new(),
        }
    }

    /// Parse response with automatic format detection
    pub async fn parse<T>(&self, response: Response) -> Result<T>
    where
        T: DeserializeOwned + Send,
    {
        self.try_all_formats(response).await
    }

    /// Try parsing with a hint
    async fn try_with_hint<T>(&self, response: Response, hint: FormatHint) -> Result<T>
    where
        T: DeserializeOwned + Send,
    {
        debug!("Trying format hint: {:?}", hint);

        match hint {
            FormatHint::Unwrapped | FormatHint::StatusCodeBased => {
                self.unwrapped.parse(response).await
            }
            FormatHint::Wrapped => self.wrapped.parse(response).await,
            FormatHint::Unknown => {
                // No hint, fall through to auto-detection
                self.try_all_formats(response).await
            }
        }
    }

    /// Try all formats in order
    async fn try_all_formats<T>(&self, response: Response) -> Result<T>
    where
        T: DeserializeOwned + Send,
    {
        let status = response.status();
        let url = response.url().clone();

        // Clone body for multiple attempts
        let body = response
            .bytes()
            .await
            .map_err(|e| ApiError::RequestFailed {
                message: format!("Failed to read response body: {}", e),
                source: Some(Box::new(e)),
            })?;

        // Try unwrapped first (modern REST pattern)
        debug!("Attempting unwrapped format for {}", url);
        match serde_json::from_slice::<T>(&body) {
            Ok(data) if status.is_success() => {
                debug!("✅ Successfully parsed as unwrapped format");
                return Ok(data);
            }
            Ok(_) => {
                debug!("⚠️  Parsed as unwrapped but HTTP status indicates error");
            }
            Err(e) => {
                debug!("⚠️  Unwrapped parsing failed: {}", e);
            }
        }

        // Try wrapped format
        debug!("Attempting wrapped format for {}", url);
        if let Ok(wrapped_str) = std::str::from_utf8(&body) {
            match serde_json::from_str::<serde_json::Value>(wrapped_str) {
                Ok(json) if json.get("success").is_some() => {
                    debug!("✅ Detected wrapped format (has 'success' field)");
                    // Re-parse as wrapped
                    return serde_json::from_value::<T>(
                        json.get("data").cloned().unwrap_or(serde_json::Value::Null),
                    )
                    .map_err(|e| ApiError::ParseError {
                        message: format!("Failed to extract data from wrapped response: {}", e),
                        body: Some(wrapped_str.to_string()),
                    });
                }
                _ => {
                    debug!("⚠️  Not a wrapped format");
                }
            }
        }

        // All formats failed
        warn!("❌ Failed to parse response with any format");
        Err(ApiError::ParseError {
            message: format!(
                "Could not parse response as unwrapped or wrapped format (status: {})",
                status
            ),
            body: Some(String::from_utf8_lossy(&body).to_string()),
        })
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_auto_detect() {
        // Placeholder for tests
    }
}
