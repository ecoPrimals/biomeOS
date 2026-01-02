//! Protocol adapters for different communication protocols

use reqwest::{Method, Response};

use crate::primal_client::error::Result;

pub mod http;

pub use http::HttpProtocolAdapter;

/// Protocol adapter enum (concrete types, not trait object)
/// 
/// Supports generic methods without dyn compatibility issues.
#[derive(Clone, Debug)]
pub enum ProtocolAdapter {
    /// HTTP/HTTPS protocol
    Http(HttpProtocolAdapter),
    // Future: Add more protocols as needed
    // WebSocket(WebSocketProtocolAdapter),
    // Grpc(GrpcProtocolAdapter),
}

impl ProtocolAdapter {
    /// Make a request to endpoint with given method and body
    pub async fn request(
        &self,
        endpoint: &str,
        method: Method,
        body: Option<Vec<u8>>,
    ) -> Result<Response> {
        match self {
            Self::Http(adapter) => adapter.request(endpoint, method, body).await,
        }
    }

    /// Get protocol identifier
    pub fn protocol(&self) -> &str {
        match self {
            Self::Http(adapter) => adapter.protocol(),
        }
    }

    /// Check if endpoint is supported by this adapter
    pub fn supports(&self, endpoint: &str) -> bool {
        match self {
            Self::Http(adapter) => adapter.supports(endpoint),
        }
    }
}

impl Default for ProtocolAdapter {
    fn default() -> Self {
        Self::Http(HttpProtocolAdapter::default())
    }
}

