//! Integration tests for discovery API handler
//!
//! Tests the /api/v1/primals/discovered endpoint with various scenarios

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use biomeos_api::{AppState, Config};
use biomeos_core::CompositeDiscovery;
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt; // Required for .oneshot() method on Router

/// Helper to create test app with standalone discovery
async fn test_app_with_standalone_discovery() -> Router {
    let discovery = CompositeDiscovery::new();

    let mut config = Config::default();
    config.standalone_mode = true;

    let state = AppState::builder()
        .discovery(discovery)
        .config(config)
        .build()
        .expect("Failed to build test app state");

    biomeos_api::create_app(state)
}

/// Helper to create test app in standalone mode
async fn test_app_standalone() -> Router {
    std::env::set_var("BIOMEOS_STANDALONE_MODE", "true");

    let state = AppState::builder()
        .config_from_env()
        .build_with_defaults()
        .expect("Failed to build standalone app state");

    biomeos_api::create_app(state)
}

#[tokio::test]
async fn test_get_discovered_primals_standalone_mode() {
    let app = test_app_standalone().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/primals/discovered")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify structure
    assert!(json.get("primals").is_some());
    assert!(json.get("count").is_some());
    assert_eq!(json["mode"], "standalone");

    // Verify we have demo primals in standalone mode
    let primals = json["primals"].as_array().unwrap();
    assert!(
        !primals.is_empty(),
        "Standalone mode should return demo primals"
    );

    // Verify primal structure
    let first_primal = &primals[0];
    assert!(first_primal.get("id").is_some());
    assert!(first_primal.get("name").is_some());
    assert!(first_primal.get("primal_type").is_some());
    assert!(first_primal.get("version").is_some());
    assert!(first_primal.get("health").is_some());
    assert!(first_primal.get("capabilities").is_some());
    assert!(first_primal.get("endpoint").is_some());
    assert!(first_primal.get("last_seen").is_some());
}

#[tokio::test]
async fn test_get_discovered_primals_alternative_routes() {
    let app = test_app_standalone().await;

    // Test /api/v1/primals/list
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/primals/list")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Test /api/v1/primals
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/primals")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_discovered_primals_response_structure() {
    let app = test_app_standalone().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/primals/discovered")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify response structure
    assert!(json.is_object());
    assert_eq!(
        json["primals"].as_array().unwrap().len(),
        json["count"].as_u64().unwrap() as usize
    );

    // Verify each primal has required fields
    for primal in json["primals"].as_array().unwrap() {
        assert!(primal["id"].is_string());
        assert!(primal["name"].is_string());
        assert!(primal["primal_type"].is_string());
        assert!(primal["version"].is_string());
        assert!(primal["health"].is_string());
        assert!(primal["capabilities"].is_array());
        assert!(primal["endpoint"].is_string());
        assert!(primal["last_seen"].is_number());

        // Health should be valid value
        let health = primal["health"].as_str().unwrap();
        assert!(
            ["healthy", "degraded", "unhealthy", "unknown"].contains(&health),
            "Invalid health status: {}",
            health
        );

        // Capabilities should be non-empty array
        assert!(!primal["capabilities"].as_array().unwrap().is_empty());
    }
}

#[tokio::test]
async fn test_discovered_primals_trust_fields() {
    let app = test_app_standalone().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/primals/discovered")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Check for optional trust fields
    let primals = json["primals"].as_array().unwrap();

    for primal in primals {
        // Trust fields are optional
        if let Some(trust_level) = primal.get("trust_level") {
            assert!(trust_level.is_number());
            let level = trust_level.as_u64().unwrap();
            assert!(level <= 100, "Trust level should be 0-100");
        }

        if let Some(family_id) = primal.get("family_id") {
            assert!(family_id.is_string());
        }

        if let Some(allowed_caps) = primal.get("allowed_capabilities") {
            assert!(allowed_caps.is_array());
        }

        if let Some(denied_caps) = primal.get("denied_capabilities") {
            assert!(denied_caps.is_array());
        }
    }
}

#[tokio::test]
async fn test_discovered_primals_standalone_mode_via_config() {
    let app = test_app_with_standalone_discovery().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/primals/discovered")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should succeed even with standalone discovery
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify basic structure
    assert!(json.get("primals").is_some());
    assert!(json.get("count").is_some());
    assert!(json.get("mode").is_some());
}

#[tokio::test]
async fn test_discovered_primals_content_type() {
    let app = test_app_standalone().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/primals/discovered")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Verify JSON content type
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));
}

#[tokio::test]
async fn test_discovered_primals_timestamp_validity() {
    let app = test_app_standalone().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/primals/discovered")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Verify timestamps are reasonable (not too old, not in future)
    for primal in json["primals"].as_array().unwrap() {
        let last_seen = primal["last_seen"].as_u64().unwrap();

        // Should be within last hour and not in future
        assert!(
            last_seen > now - 3600 && last_seen <= now,
            "Invalid timestamp: {} (now: {})",
            last_seen,
            now
        );
    }
}

#[tokio::test]
async fn test_discovered_primals_capabilities_format() {
    let app = test_app_standalone().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/primals/discovered")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify capabilities are properly formatted
    for primal in json["primals"].as_array().unwrap() {
        let capabilities = primal["capabilities"].as_array().unwrap();

        for cap in capabilities {
            let cap_str = cap.as_str().unwrap();
            // Capabilities should not be empty strings
            assert!(!cap_str.is_empty());
            // Should be lowercase
            assert_eq!(cap_str, cap_str.to_lowercase());
        }
    }
}

#[tokio::test]
async fn test_discovered_primals_endpoint_format() {
    let app = test_app_standalone().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/primals/discovered")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify endpoint format
    for primal in json["primals"].as_array().unwrap() {
        let endpoint = primal["endpoint"].as_str().unwrap();

        // Endpoint should not be empty
        assert!(!endpoint.is_empty());

        // Should start with valid protocol or path
        assert!(
            endpoint.starts_with("http://")
                || endpoint.starts_with("https://")
                || endpoint.starts_with("unix://")
                || endpoint.starts_with('/'),
            "Invalid endpoint format: {}",
            endpoint
        );
    }
}

#[tokio::test]
async fn test_discovered_primals_health_endpoint() {
    let app = test_app_standalone().await;

    // Also test the /api/v1/health endpoint
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}
