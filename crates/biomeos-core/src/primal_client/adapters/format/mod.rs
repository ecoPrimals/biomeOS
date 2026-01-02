//! Format adapters for different response formats

use reqwest::Response;
use serde::de::DeserializeOwned;

use crate::primal_client::error::{ApiError, Result};

pub mod auto;
pub mod unwrapped;
pub mod wrapped;

pub use auto::AutoFormatAdapter;
pub use unwrapped::UnwrappedFormatAdapter;
pub use wrapped::WrappedFormatAdapter;

/// Format adapter enum (replaces trait object for dyn compatibility)
#[derive(Clone, Debug)]
pub enum FormatAdapter {
    /// Auto-detect format from response
    Auto(AutoFormatAdapter),
    
    /// Expect unwrapped responses (HTTP status-based)
    Unwrapped(UnwrappedFormatAdapter),
    
    /// Expect wrapped responses (success/data/error envelope)
    Wrapped(WrappedFormatAdapter),
}

impl FormatAdapter {
    /// Parse response into expected type
    pub async fn parse<T>(&self, response: Response) -> Result<T>
    where
        T: DeserializeOwned + Send,
    {
        match self {
            FormatAdapter::Auto(adapter) => adapter.parse(response).await,
            FormatAdapter::Unwrapped(adapter) => adapter.parse(response).await,
            FormatAdapter::Wrapped(adapter) => adapter.parse(response).await,
        }
    }
    
    /// Check if response indicates success
    pub fn is_success(&self, response: &Response) -> bool {
        response.status().is_success()
    }
    
    /// Extract error information from response
    pub async fn extract_error(&self, response: Response) -> ApiError {
        let status = response.status();
        let body = response.text().await.unwrap_or_else(|_| String::from("<no body>"));

        match status.as_u16() {
            401 => ApiError::Unauthorized {
                message: body,
            },
            403 => ApiError::Forbidden {
                message: body,
            },
            404 => ApiError::NotFound {
                resource: body,
            },
            500..=599 => ApiError::ServerError {
                status: status.as_u16(),
                message: body,
            },
            _ => ApiError::Other {
                message: format!("HTTP {}: {}", status, body),
            },
        }
    }
    
    /// Get adapter name for logging
    pub fn name(&self) -> &'static str {
        match self {
            FormatAdapter::Auto(_) => "auto",
            FormatAdapter::Unwrapped(_) => "unwrapped",
            FormatAdapter::Wrapped(_) => "wrapped",
        }
    }
    
    /// Create auto-detecting adapter (default)
    pub fn auto() -> Self {
        FormatAdapter::Auto(AutoFormatAdapter::new())
    }
    
    /// Create unwrapped adapter (HTTP status-based)
    pub fn unwrapped() -> Self {
        FormatAdapter::Unwrapped(UnwrappedFormatAdapter::new())
    }
    
    /// Create wrapped adapter (envelope-based)
    pub fn wrapped() -> Self {
        FormatAdapter::Wrapped(WrappedFormatAdapter::new())
    }
}

impl Default for FormatAdapter {
    fn default() -> Self {
        Self::auto()
    }
}

