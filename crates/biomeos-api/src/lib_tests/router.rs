// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::gate_disabled;
use crate::*;
use axum::body::Body;
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn router_health_returns_json_when_gate_disabled() {
    let state = AppState::builder().build_with_defaults().expect("state");
    let app = create_app_with_gate(state, gate_disabled());
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&body).expect("json");
    assert_eq!(v["status"], "healthy");
}

#[tokio::test]
async fn router_readiness_and_liveness_when_gate_disabled() {
    let state = AppState::builder().build_with_defaults().expect("state");
    for path in ["/api/v1/health/ready", "/api/v1/health/live"] {
        let app = create_app_with_gate(state.clone(), gate_disabled());
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri(path)
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK, "path {path}");
    }
}

#[tokio::test]
async fn router_topology_forbidden_without_token_when_sovereign() {
    let state = AppState::builder().build_with_defaults().expect("state");
    let app = create_app_for_tcp(state);
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/v1/topology")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn router_well_known_bypasses_gate_when_sovereign() {
    let state = AppState::builder().build_with_defaults().expect("state");
    let app = create_app_for_tcp(state);
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/.well-known/acme-challenge/token")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_ne!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn router_health_bare_ok_when_sovereign_no_body() {
    let state = AppState::builder().build_with_defaults().expect("state");
    let app = create_app_for_tcp(state);
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    assert!(body.is_empty());
}

#[tokio::test]
async fn router_unknown_route_returns_404() {
    let state = AppState::builder().build_with_defaults().expect("state");
    let app = create_app_with_gate(state, gate_disabled());
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/v1/route-that-does-not-exist")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn router_health_includes_security_headers() {
    let state = AppState::builder().build_with_defaults().expect("state");
    let app = create_app_with_gate(state, gate_disabled());
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/v1/health")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response
            .headers()
            .get("strict-transport-security")
            .is_some()
    );
    assert!(response.headers().get("x-content-type-options").is_some());
    assert!(response.headers().get("content-security-policy").is_some());
    assert!(response.headers().get("x-frame-options").is_some());
    assert!(response.headers().get("referrer-policy").is_some());
    assert!(response.headers().get("cache-control").is_some());
}

#[tokio::test]
async fn router_cors_permissive_reflects_origin_when_gate_disabled() {
    let state = AppState::builder().build_with_defaults().expect("state");
    let app = create_app_with_gate(state, gate_disabled());
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/api/v1/health")
                .header("origin", "http://localhost:3000")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response
            .headers()
            .get("access-control-allow-origin")
            .is_some()
    );
}

#[tokio::test]
async fn router_post_body_over_limit_returns_413() {
    let state = AppState::builder().build_with_defaults().expect("state");
    let app = create_app_with_gate(state, gate_disabled());
    let oversized = vec![b'x'; 1024 * 1024 + 1];
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method(axum::http::Method::POST)
                .uri("/api/v1/capabilities/discover")
                .header(axum::http::header::CONTENT_TYPE, "application/json")
                .body(Body::from(oversized))
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}
