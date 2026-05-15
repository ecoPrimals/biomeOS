// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Shared helpers for sovereign security pen tests (`sovereign_pen_*.rs`).

#![allow(
    dead_code,
    unused_imports,
    reason = "each integration test binary uses a subset of helpers"
)]

use axum::{Router, http::StatusCode};
use biomeos_api::{AppState, Config};

pub use axum::body::Body;
pub use axum::http::{Method, Request, header};
pub use http_body_util::BodyExt;
pub use tower::ServiceExt;

/// Create a sovereign-mode app (gate ENABLED, no BearDog available)
///
/// This simulates the production scenario where BearDog is running but
/// the attacker doesn't have a valid token. Since BearDog socket doesn't
/// exist in test, ALL token verification will fail — which is what we want.
pub fn sovereign_app() -> Router {
    let family_id = biomeos_core::family_discovery::get_family_id();
    let gate = biomeos_api::dark_forest_gate::DarkForestGateConfig {
        enabled: true,
        neural_api_socket: Some("/tmp/nonexistent-beardog-pentest.sock".to_string()),
        family_id,
    };

    let state = AppState::builder()
        .config(Config {
            standalone_mode: true,
            ..Default::default()
        })
        .build_with_defaults()
        .expect("Failed to build app state");

    biomeos_api::create_app_for_tcp_with_gate(state, gate)
}

/// Assert a response is a proper Dark Forest rejection:
/// - Status 403 Forbidden
/// - Body is empty (reveals nothing)
pub async fn assert_dark_forest_rejection(response: axum::http::Response<Body>, context: &str) {
    let status = response.status();

    // Collect response headers for inspection
    let headers = response.headers().clone();
    let body = response.into_body().collect().await.unwrap().to_bytes();

    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "Expected 403 for {context}, got {status}"
    );

    assert!(
        body.is_empty(),
        "Expected empty body for {}, got {} bytes: {:?}",
        context,
        body.len(),
        String::from_utf8_lossy(&body)
    );

    // Verify no information leaks in headers
    assert_no_info_leak(&headers, context);
}

/// Assert a response is a bare health OK:
/// - Status 200 OK
/// - Body is empty (reveals nothing)
pub async fn assert_bare_health_ok(response: axum::http::Response<Body>, context: &str) {
    let status = response.status();
    let headers = response.headers().clone();
    let body = response.into_body().collect().await.unwrap().to_bytes();

    assert_eq!(
        status,
        StatusCode::OK,
        "Expected 200 for {context}, got {status}"
    );

    assert!(
        body.is_empty(),
        "Health response for {} should have empty body, got {} bytes: {:?}",
        context,
        body.len(),
        String::from_utf8_lossy(&body)
    );

    // Verify no information leaks in headers
    assert_no_info_leak(&headers, context);
}

/// Check that response headers don't leak information
fn assert_no_info_leak(headers: &axum::http::HeaderMap, context: &str) {
    // Should not have a Server header revealing software
    if let Some(server) = headers.get("server") {
        let server_str = server.to_str().unwrap_or("");
        assert!(
            !server_str.to_lowercase().contains("axum"),
            "Server header leaks 'axum' for {context}"
        );
        assert!(
            !server_str.to_lowercase().contains("hyper"),
            "Server header leaks 'hyper' for {context}"
        );
        assert!(
            !server_str.to_lowercase().contains("biomeos"),
            "Server header leaks 'biomeos' for {context}"
        );
        assert!(
            !server_str.to_lowercase().contains("tower"),
            "Server header leaks 'tower' for {context}"
        );
    }

    // Should not have X-Powered-By or similar
    assert!(
        headers.get("x-powered-by").is_none(),
        "X-Powered-By header present for {context}"
    );

    // Content-Type on empty 403 should not be application/json
    // (that would hint at a JSON API backend)
    if let Some(ct) = headers.get("content-type") {
        let ct_str = ct.to_str().unwrap_or("");
        // For empty bodies, there should be no content-type, or it should be generic
        assert!(
            !ct_str.contains("application/json"),
            "Content-Type 'application/json' on rejection leaks API info for {context}"
        );
    }
}
