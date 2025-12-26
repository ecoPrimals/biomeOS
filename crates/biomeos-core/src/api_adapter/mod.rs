//! API Adapter System
//!
//! Discovers and adapts to primal API interfaces without requiring standardization.
//! Each primal can have its own API structure - we learn and adapt to it.

pub mod adapters;
pub mod cache;
pub mod cli_adapter;
pub mod discovery;

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// HTTP Method wrapper that can be serialized
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

impl From<HttpMethod> for reqwest::Method {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::GET => reqwest::Method::GET,
            HttpMethod::POST => reqwest::Method::POST,
            HttpMethod::PUT => reqwest::Method::PUT,
            HttpMethod::DELETE => reqwest::Method::DELETE,
            HttpMethod::PATCH => reqwest::Method::PATCH,
        }
    }
}

impl From<reqwest::Method> for HttpMethod {
    fn from(method: reqwest::Method) -> Self {
        if method == reqwest::Method::GET {
            HttpMethod::GET
        } else if method == reqwest::Method::POST {
            HttpMethod::POST
        } else if method == reqwest::Method::PUT {
            HttpMethod::PUT
        } else if method == reqwest::Method::DELETE {
            HttpMethod::DELETE
        } else if method == reqwest::Method::PATCH {
            HttpMethod::PATCH
        } else {
            HttpMethod::GET // Default
        }
    }
}

/// Discovered API adapter for a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiAdapter {
    /// Base URL for the API
    pub base_url: String,

    /// Discovered health check endpoint
    pub health_endpoint: Option<String>,

    /// Discovered service registration endpoint
    pub register_endpoint: Option<RegisterEndpoint>,

    /// Discovered service discovery/list endpoint
    pub discovery_endpoint: Option<String>,

    /// Authentication method (if discovered)
    pub auth_method: Option<AuthMethod>,

    /// Response format
    pub response_format: ResponseFormat,

    /// Additional discovered endpoints
    pub discovered_endpoints: HashMap<String, EndpointInfo>,

    /// Primal name (for caching)
    pub primal_name: String,
}

/// Information about a discovered endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointInfo {
    pub path: String,
    pub method: HttpMethod,
    pub requires_auth: bool,
    pub response_type: ResponseType,
}

/// Registration endpoint details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterEndpoint {
    pub path: String,
    pub method: HttpMethod,
}

/// Authentication method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    None,
    Bearer,
    ApiKey { header: String },
    Basic,
}

/// Response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseFormat {
    Json,
    Text,
    Binary,
}

/// Response type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseType {
    Json,
    Text,
    Html,
    Binary,
}

impl ApiAdapter {
    /// Create a new API adapter for a base URL
    pub fn new(base_url: impl Into<String>, primal_name: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            health_endpoint: None,
            register_endpoint: None,
            discovery_endpoint: None,
            auth_method: Some(AuthMethod::None),
            response_format: ResponseFormat::Json,
            discovered_endpoints: HashMap::new(),
            primal_name: primal_name.into(),
        }
    }

    /// Check if the primal is healthy using discovered health endpoint
    pub async fn check_health(&self) -> Result<bool> {
        if let Some(endpoint) = &self.health_endpoint {
            let url = format!("{}{}", self.base_url, endpoint);
            let client = Client::new();

            match client.get(&url).send().await {
                Ok(response) => Ok(response.status().is_success()),
                Err(_) => Ok(false),
            }
        } else {
            // No health endpoint discovered yet
            Ok(false)
        }
    }

    /// Try an endpoint to see if it exists
    pub async fn try_endpoint(&self, path: &str) -> Result<bool> {
        let url = format!("{}{}", self.base_url, path);
        let client = Client::new();

        match client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Try an endpoint with a specific method
    pub async fn try_endpoint_with_method(&self, path: &str, method: HttpMethod) -> Result<bool> {
        let url = format!("{}{}", self.base_url, path);
        let client = Client::new();

        let reqwest_method: reqwest::Method = method.into();
        let request = match reqwest_method {
            reqwest::Method::GET => client.get(&url),
            reqwest::Method::POST => client.post(&url),
            reqwest::Method::PUT => client.put(&url),
            reqwest::Method::DELETE => client.delete(&url),
            reqwest::Method::PATCH => client.patch(&url),
            _ => return Ok(false),
        };

        match request.send().await {
            Ok(response) => Ok(response.status().is_success() || response.status().as_u16() == 400),
            Err(_) => Ok(false),
        }
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the primal name
    pub fn primal_name(&self) -> &str {
        &self.primal_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_adapter() {
        let adapter = ApiAdapter::new("http://localhost:8080", "test-primal");
        assert_eq!(adapter.base_url(), "http://localhost:8080");
        assert_eq!(adapter.primal_name(), "test-primal");
    }
}
