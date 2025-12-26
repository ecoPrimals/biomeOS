//! API Discovery
//!
//! Discovers API endpoints by trying common patterns.
//! No assumptions, no hardcoding - just intelligent probing.

use super::{ApiAdapter, EndpointInfo, HttpMethod, RegisterEndpoint, ResponseType};
use anyhow::Result;
use reqwest;

/// Common health check endpoint patterns
const HEALTH_PATTERNS: &[&str] = &[
    "/health",
    "/api/health",
    "/api/v1/health",
    "/healthz",
    "/_health",
    "/status",
    "/ping",
];

/// Common service registration patterns  
const REGISTER_PATTERNS: &[(&str, HttpMethod)] = &[
    ("/api/v1/services/register", HttpMethod::POST),
    ("/api/v1/services", HttpMethod::POST),
    ("/api/services/register", HttpMethod::POST),
    ("/api/services", HttpMethod::POST),
    ("/register", HttpMethod::POST),
    ("/services", HttpMethod::POST),
];

/// Common service discovery patterns
const DISCOVERY_PATTERNS: &[&str] = &[
    "/api/v1/services",
    "/api/services",
    "/services",
    "/discover",
    "/api/discover",
];

/// Discover API interface for a primal
pub async fn discover_api_interface(
    base_url: impl Into<String>,
    primal_name: impl Into<String>,
) -> Result<ApiAdapter> {
    let base_url = base_url.into();
    let primal_name = primal_name.into();

    let mut adapter = ApiAdapter::new(&base_url, &primal_name);

    println!("🔍 Discovering API for {} at {}", primal_name, base_url);

    // Discover health endpoint
    adapter.health_endpoint = discover_health_endpoint(&adapter).await;
    if let Some(ref endpoint) = adapter.health_endpoint {
        println!("  ✓ Health endpoint: {}", endpoint);
    } else {
        println!("  ⚠ No health endpoint found");
    }

    // Discover registration endpoint
    adapter.register_endpoint = discover_register_endpoint(&adapter).await;
    if let Some(ref endpoint) = adapter.register_endpoint {
        println!(
            "  ✓ Register endpoint: {} {:?}",
            endpoint.path, endpoint.method
        );
    } else {
        println!("  ⚠ No register endpoint found");
    }

    // Discover service listing endpoint
    adapter.discovery_endpoint = discover_discovery_endpoint(&adapter).await;
    if let Some(ref endpoint) = adapter.discovery_endpoint {
        println!("  ✓ Discovery endpoint: {}", endpoint);
    } else {
        println!("  ⚠ No discovery endpoint found");
    }

    Ok(adapter)
}

/// Discover health check endpoint
async fn discover_health_endpoint(adapter: &ApiAdapter) -> Option<String> {
    for pattern in HEALTH_PATTERNS {
        if adapter.try_endpoint(pattern).await.unwrap_or(false) {
            return Some(pattern.to_string());
        }
    }
    None
}

/// Discover service registration endpoint
async fn discover_register_endpoint(adapter: &ApiAdapter) -> Option<RegisterEndpoint> {
    for (pattern, method) in REGISTER_PATTERNS {
        if adapter
            .try_endpoint_with_method(pattern, method.clone())
            .await
            .unwrap_or(false)
        {
            return Some(RegisterEndpoint {
                path: pattern.to_string(),
                method: method.clone(),
            });
        }
    }
    None
}

/// Discover service discovery/listing endpoint
async fn discover_discovery_endpoint(adapter: &ApiAdapter) -> Option<String> {
    for pattern in DISCOVERY_PATTERNS {
        if adapter.try_endpoint(pattern).await.unwrap_or(false) {
            return Some(pattern.to_string());
        }
    }
    None
}

/// Probe a URL to detect its purpose
pub async fn probe_endpoint(url: &str) -> Result<EndpointInfo> {
    let client = reqwest::Client::new();

    // Try GET first
    let response = client.get(url).send().await?;
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let response_type = if content_type.contains("json") {
        ResponseType::Json
    } else if content_type.contains("html") {
        ResponseType::Html
    } else if content_type.contains("text") {
        ResponseType::Text
    } else {
        ResponseType::Binary
    };

    Ok(EndpointInfo {
        path: url.to_string(),
        method: HttpMethod::GET,
        requires_auth: response.status().as_u16() == 401,
        response_type,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_patterns() {
        // Verify patterns exist and contain expected values
        assert!(HEALTH_PATTERNS.len() >= 3);
        assert!(HEALTH_PATTERNS.contains(&"/health"));
    }

    #[test]
    fn test_register_patterns() {
        // Verify patterns exist and contain expected values
        assert!(REGISTER_PATTERNS.len() >= 3);
        assert!(REGISTER_PATTERNS
            .iter()
            .any(|(path, _)| path.contains("register")));
    }
}
