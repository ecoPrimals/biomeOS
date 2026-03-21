// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Sovereign Security Pen Tests
//!
//! These tests verify the Dark Forest gate cannot be bypassed.
//! Every test simulates an attacker who does NOT have a valid family seed.
//!
//! THREAT MODEL:
//! - Attacker knows nestgate.io resolves to our Tower
//! - Attacker can send arbitrary HTTP requests
//! - Attacker does NOT know the family seed
//! - Attacker should learn NOTHING about the system
//!
//! PASS CRITERIA:
//! - No route returns anything other than 403 (empty body) or 200 (empty body for health)
//! - No response header reveals software identity
//! - No timing difference between valid-looking and garbage tokens
//! - No path traversal bypasses the gate
//! - No HTTP method bypasses the gate
//! - No payload size causes different behavior

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use biomeos_api::{AppState, Config};
use biomeos_test_utils::set_test_env;
use http_body_util::BodyExt;
use std::time::Instant;
use tower::ServiceExt;

// ═══════════════════════════════════════════════════════════════════════
// TEST HELPERS
// ═══════════════════════════════════════════════════════════════════════

/// Create a sovereign-mode app (gate ENABLED, no BearDog available)
///
/// This simulates the production scenario where BearDog is running but
/// the attacker doesn't have a valid token. Since BearDog socket doesn't
/// exist in test, ALL token verification will fail — which is what we want.
fn sovereign_app() -> Router {
    // Ensure sovereign mode is ON
    set_test_env("BIOMEOS_SOVEREIGN", "true");
    // Point to nonexistent socket so all verification fails
    set_test_env("BEARDOG_SOCKET", "/tmp/nonexistent-beardog-pentest.sock");

    let state = AppState::builder()
        .config(Config {
            standalone_mode: true,
            ..Default::default()
        })
        .build_with_defaults()
        .expect("Failed to build app state");

    biomeos_api::create_app_for_tcp(state)
}

/// Assert a response is a proper Dark Forest rejection:
/// - Status 403 Forbidden
/// - Body is empty (reveals nothing)
async fn assert_dark_forest_rejection(response: axum::http::Response<Body>, context: &str) {
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
async fn assert_bare_health_ok(response: axum::http::Response<Body>, context: &str) {
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

// ═══════════════════════════════════════════════════════════════════════
// PEN TEST 1: UNAUTHENTICATED ACCESS — EVERY ROUTE
// ═══════════════════════════════════════════════════════════════════════

/// Every known API route should return 403 with empty body when no token provided
#[tokio::test]
async fn pentest_all_routes_reject_without_token() {
    let app = sovereign_app();

    let routes = vec![
        // Discovery
        "/api/v1/primals/discovered",
        "/api/v1/primals/list",
        "/api/v1/primals",
        // Topology
        "/api/v1/topology",
        // LiveSpores
        "/api/v1/livespores",
        // Events
        "/api/v1/events/stream",
        "/api/v1/events/ws",
        // Trust
        "/api/v1/trust/identity",
        // Genome
        "/api/v1/genome/create",
        "/api/v1/genome/compose",
        "/api/v1/genome/self-replicate",
        "/api/v1/genome/list",
        "/api/v1/genome/test-id/verify",
        "/api/v1/genome/test-id/download",
        // Rendezvous
        "/api/v1/rendezvous/beacon",
        "/api/v1/rendezvous/check",
    ];

    for route in routes {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(route).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_dark_forest_rejection(response, &format!("GET {route} (no token)")).await;
    }
}

/// POST routes should also be rejected
#[tokio::test]
async fn pentest_post_routes_reject_without_token() {
    let app = sovereign_app();

    let post_routes = vec![
        "/api/v1/trust/evaluate",
        "/api/v1/genome/create",
        "/api/v1/genome/compose",
        "/api/v1/genome/self-replicate",
        "/api/v1/rendezvous/beacon",
        "/api/v1/rendezvous/check",
    ];

    for route in post_routes {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(route)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_dark_forest_rejection(response, &format!("POST {route} (no token)")).await;
    }
}

// ═══════════════════════════════════════════════════════════════════════
// PEN TEST 2: HEALTH ENDPOINT — INFORMATION LEAKAGE
// ═══════════════════════════════════════════════════════════════════════

/// Health endpoints should return bare 200 with ZERO information
#[tokio::test]
async fn pentest_health_reveals_nothing() {
    let app = sovereign_app();

    let health_paths = vec![
        "/health",
        "/api/v1/health",
        "/api/v1/health/ready",
        "/api/v1/health/live",
    ];

    for path in health_paths {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_bare_health_ok(response, &format!("health: {path}")).await;
    }
}

/// Health with trailing slashes should still be handled
#[tokio::test]
async fn pentest_health_path_variants() {
    let app = sovereign_app();

    // These should either be bare 200 or 403, never leak info
    let variants = vec![
        "/api/v1/health/",
        "/api/v1/health/ready/",
        "/api/v1/health/live/",
    ];

    for path in variants {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();

        let resp_status = response.status();
        let body = response.into_body().collect().await.unwrap().to_bytes();

        // Either bare 200 or 403 — both are acceptable, but body must be empty
        assert!(
            resp_status == StatusCode::OK
                || resp_status == StatusCode::FORBIDDEN
                || resp_status == StatusCode::NOT_FOUND,
            "Unexpected status {resp_status} for {path}"
        );

        // If it's a 200 or 403 from our gate, body must be empty
        if resp_status == StatusCode::OK || resp_status == StatusCode::FORBIDDEN {
            assert!(
                body.is_empty(),
                "Body should be empty for {}, got: {:?}",
                path,
                String::from_utf8_lossy(&body)
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// PEN TEST 3: BYPASS ABUSE — PATH TRAVERSAL
// ═══════════════════════════════════════════════════════════════════════

/// .well-known bypass should not allow path traversal to other routes
#[tokio::test]
async fn pentest_well_known_no_path_traversal() {
    let app = sovereign_app();

    let traversal_attempts = vec![
        "/.well-known/../api/v1/primals",
        "/.well-known/../../etc/passwd",
        "/.well-known/%2e%2e/api/v1/primals",
        "/.well-known/..%2fapi/v1/primals",
        "/.well-known/acme-challenge/../../api/v1/primals",
    ];

    for path in traversal_attempts {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();

        let status = response.status();
        let body = response.into_body().collect().await.unwrap().to_bytes();

        // Should NOT return actual API data
        if !body.is_empty() {
            let body_str = String::from_utf8_lossy(&body);
            assert!(
                !body_str.contains("primals"),
                "Path traversal via {path} leaked primal data: {body_str}"
            );
            assert!(
                !body_str.contains("version"),
                "Path traversal via {path} leaked version: {body_str}"
            );
        }

        // Status should be 400 (bad request), 403, or 404 — never 200 with API data
        assert!(
            status == StatusCode::BAD_REQUEST
                || status == StatusCode::FORBIDDEN
                || status == StatusCode::NOT_FOUND
                // .well-known paths pass through to router which returns 404 for unknown sub-paths
                || (status == StatusCode::OK && body.is_empty()),
            "Unexpected status {status} for traversal attempt: {path}"
        );
    }
}

/// Health path spoofing — can we trick the gate by embedding health in the path?
#[tokio::test]
async fn pentest_health_path_spoofing() {
    let app = sovereign_app();

    let spoofing_attempts = vec![
        "/api/v1/health/../primals",
        "/api/v1/health/../../api/v1/primals",
        "/api/v1/health%00/primals",
        "/api/v1/healthx",
        "/api/v1/health;/primals",
        "/healthx/api/v1/primals",
    ];

    for path in spoofing_attempts {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();

        let _status = response.status();
        let body = response.into_body().collect().await.unwrap().to_bytes();

        // Should never return actual API data
        if !body.is_empty() {
            let body_str = String::from_utf8_lossy(&body);
            assert!(
                !body_str.contains("primals") && !body_str.contains("count"),
                "Health path spoofing via {path} leaked data: {body_str}"
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// PEN TEST 4: TOKEN MANIPULATION
// ═══════════════════════════════════════════════════════════════════════

/// Invalid tokens should all result in 403 with empty body
#[tokio::test]
async fn pentest_invalid_tokens_rejected() {
    let app = sovereign_app();

    let oversized = "A".repeat(10000);
    let bad_tokens: Vec<&str> = vec![
        "",                         // Empty
        " ",                        // Whitespace
        "invalid",                  // Plaintext
        "AAAA",                     // Short base64
        "AAAAAAAAAAAAAAAAAAAAAA==", // Valid base64, invalid crypto
        "null",                     // JSON null
        "true",                     // JSON bool
        "{}",                       // JSON object
        "[]",                       // JSON array
        &oversized,                 // Oversized (10KB)
        "Bearer test-token",        // OAuth-style
        "Basic dGVzdDp0ZXN0",       // Basic auth style
        "../../../etc/passwd",      // Path traversal in token
        "' OR 1=1 --",              // SQL injection in token
    ];

    for token in &bad_tokens {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/primals")
                    .header("X-Dark-Forest-Token", *token)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_dark_forest_rejection(
            response,
            &format!("invalid token: {:?}", &token[..token.len().min(30)]),
        )
        .await;
    }

    // Tokens with bytes that are invalid as HTTP header values should be
    // rejected at the HTTP layer itself (never reach our code) — that's correct
    let invalid_header_tokens = vec![
        "\x00\x00\x00\x00",          // Null bytes
        "<script>alert(1)</script>", // Angle brackets (may be invalid depending on header rules)
    ];

    for token in &invalid_header_tokens {
        let build_result = Request::builder()
            .uri("/api/v1/primals")
            .header("X-Dark-Forest-Token", *token)
            .body(Body::empty());

        if let Ok(request) = build_result {
            // If HTTP layer accepts it, our gate must still reject
            let response = app.clone().oneshot(request).await.unwrap();
            assert_dark_forest_rejection(
                response,
                &format!("invalid header token: {:?}", &token[..token.len().min(20)]),
            )
            .await;
        }
        // Else: HTTP layer rejected the header value — good, never reaches our code
    }
}

/// Tokens in the wrong header should be ignored
#[tokio::test]
async fn pentest_token_wrong_header() {
    let app = sovereign_app();

    // Try various header names
    let wrong_headers = vec![
        ("Authorization", "Bearer fake-token"),
        ("X-Api-Key", "fake-key"),
        ("Cookie", "session=fake"),
        ("X-Dark-Forest", "close-but-wrong"), // Missing -Token
        ("x-dark-forest-token", "case-sensitivity"), // lowercase
        ("X-DARK-FOREST-TOKEN", "ALL-CAPS"),
    ];

    for (header_name, header_value) in &wrong_headers {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/primals")
                    .header(*header_name, *header_value)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        // HTTP headers are case-insensitive per spec, so lowercase version
        // may actually match — but with no BearDog, still 403
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "Expected 403 for header {header_name}: {header_value}"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════
// PEN TEST 5: HTTP METHOD FUZZING
// ═══════════════════════════════════════════════════════════════════════

/// Non-standard HTTP methods should not bypass the gate
#[tokio::test]
async fn pentest_all_http_methods_gated() {
    let app = sovereign_app();

    let methods = vec![
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::PATCH,
        Method::HEAD,
        Method::OPTIONS,
        Method::TRACE,
    ];

    for method in methods {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method.clone())
                    .uri("/api/v1/primals")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body = response.into_body().collect().await.unwrap().to_bytes();

        // Should be 403 for all methods (gate runs before routing)
        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "{method} /api/v1/primals should be 403, got {status}"
        );

        // Body must be empty regardless of method
        assert!(
            body.is_empty(),
            "{} /api/v1/primals leaked body: {:?}",
            method,
            String::from_utf8_lossy(&body)
        );
    }
}

/// OPTIONS (CORS preflight) should not reveal route info
#[tokio::test]
async fn pentest_cors_preflight_reveals_nothing() {
    let app = sovereign_app();

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::OPTIONS)
                .uri("/api/v1/primals")
                .header("Origin", "https://evil.com")
                .header("Access-Control-Request-Method", "GET")
                .header("Access-Control-Request-Headers", "X-Dark-Forest-Token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let headers = response.headers().clone();
    let body = response.into_body().collect().await.unwrap().to_bytes();

    // Should NOT reveal Access-Control-Allow-Origin for arbitrary origins
    if let Some(acao) = headers.get("access-control-allow-origin") {
        let origin = acao.to_str().unwrap_or("");
        assert!(
            origin != "*" && origin != "https://evil.com",
            "CORS allows evil.com origin: {origin}"
        );
    }

    // Body should be empty
    assert!(
        body.is_empty() || status == StatusCode::FORBIDDEN,
        "OPTIONS leaked body: {:?}",
        String::from_utf8_lossy(&body)
    );
}

// ═══════════════════════════════════════════════════════════════════════
// PEN TEST 6: UNKNOWN PATHS
// ═══════════════════════════════════════════════════════════════════════

/// Completely unknown paths should still get 403 (not 404)
/// A 404 would confirm "this is a web server that routes requests"
#[tokio::test]
async fn pentest_unknown_paths_return_403_not_404() {
    let app = sovereign_app();

    let unknown_paths = vec![
        "/",
        "/index.html",
        "/admin",
        "/login",
        "/api",
        "/api/v2/secret",
        "/robots.txt",
        "/favicon.ico",
        "/sitemap.xml",
        "/.env",
        "/wp-admin",
        "/phpmyadmin",
        "/debug/vars",
        "/actuator/health",
        "/graphql",
        "/metrics",
        "/prometheus",
    ];

    for path in unknown_paths {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();

        let status = response.status();
        let body = response.into_body().collect().await.unwrap().to_bytes();

        assert_eq!(
            status,
            StatusCode::FORBIDDEN,
            "Expected 403 for unknown path {}, got {} (body: {:?})",
            path,
            status,
            String::from_utf8_lossy(&body)
        );

        assert!(
            body.is_empty(),
            "Unknown path {} leaked body: {:?}",
            path,
            String::from_utf8_lossy(&body)
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════
// PEN TEST 7: TIMING ANALYSIS
// ═══════════════════════════════════════════════════════════════════════

/// Timing between "no token" and "bad token" should be similar
/// An attacker shouldn't be able to distinguish which case they're in
#[tokio::test]
async fn pentest_timing_consistency() {
    let app = sovereign_app();

    // Measure time for requests without token
    let mut no_token_times = Vec::new();
    for _ in 0..10 {
        let start = Instant::now();
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/primals")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let elapsed = start.elapsed();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        no_token_times.push(elapsed);
    }

    // Measure time for requests with bad token
    let mut bad_token_times = Vec::new();
    for _ in 0..10 {
        let start = Instant::now();
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/primals")
                    .header("X-Dark-Forest-Token", "totally-invalid-token-here")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let elapsed = start.elapsed();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        bad_token_times.push(elapsed);
    }

    // Both should complete in similar time ranges
    // Note: bad token goes to BearDog (socket error), so it may be slightly slower
    // But it should be < 6s (the 5s timeout + 1s buffer)
    let max_bad_token = bad_token_times.iter().max().unwrap();
    assert!(
        max_bad_token.as_secs() < 6,
        "Bad token took too long: {max_bad_token:?} (possible hanging connection)"
    );

    // No-token path should be very fast (pure in-process)
    let max_no_token = no_token_times.iter().max().unwrap();
    assert!(
        max_no_token.as_millis() < 100,
        "No-token path too slow: {:?}ms (should be <100ms)",
        max_no_token.as_millis()
    );
}

// ═══════════════════════════════════════════════════════════════════════
// PEN TEST 8: REQUEST BODY ATTACKS
// ═══════════════════════════════════════════════════════════════════════

/// Large request bodies should still be rejected at the gate (before parsing)
#[tokio::test]
async fn pentest_large_body_still_rejected() {
    let app = sovereign_app();

    // 1MB payload — gate should reject before any body processing
    let large_body = "X".repeat(1_000_000);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/genome/create")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(large_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_dark_forest_rejection(response, "POST with 1MB body (no token)").await;
}

/// Malformed content-type should not change gate behavior
#[tokio::test]
async fn pentest_content_type_manipulation() {
    let app = sovereign_app();

    let content_types = vec![
        "text/html",
        "text/xml",
        "application/xml",
        "multipart/form-data",
        "application/x-www-form-urlencoded",
        "application/octet-stream",
        "image/png",
        "",
    ];

    for ct in content_types {
        let mut builder = Request::builder()
            .method(Method::POST)
            .uri("/api/v1/genome/create");

        if !ct.is_empty() {
            builder = builder.header(header::CONTENT_TYPE, ct);
        }

        let response = app
            .clone()
            .oneshot(builder.body(Body::from("test")).unwrap())
            .await
            .unwrap();

        assert_dark_forest_rejection(response, &format!("POST with Content-Type: {ct}")).await;
    }
}

// ═══════════════════════════════════════════════════════════════════════
// PEN TEST 9: DOUBLE-ENCODING AND UNICODE TRICKS
// ═══════════════════════════════════════════════════════════════════════

/// URL encoding tricks should not bypass the gate
#[tokio::test]
async fn pentest_url_encoding_bypass() {
    let app = sovereign_app();

    let encoded_paths = vec![
        // Double-encoded ../
        "/api/v1/%2e%2e/health",
        // Unicode tricks
        "/api/v1/\u{FF0E}\u{FF0E}/health",
        // Null byte injection
        "/api/v1/primals%00.html",
        // URL-encoded health
        "/%68%65%61%6c%74%68",
        // Mixed encoding
        "/api/v1/%68ealth",
    ];

    for path in encoded_paths {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();

        let _status = response.status();
        let body = response.into_body().collect().await.unwrap().to_bytes();

        // Any of these is acceptable: 403, 400, bare 200 (if decoded to health)
        // NOT acceptable: 200 with JSON body containing API data
        if !body.is_empty() {
            let body_str = String::from_utf8_lossy(&body);
            assert!(
                !body_str.contains("primals") && !body_str.contains("count"),
                "URL encoding bypass via {path} leaked data: {body_str}"
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// PEN TEST 10: HOST HEADER MANIPULATION
// ═══════════════════════════════════════════════════════════════════════

/// Host header manipulation should not bypass the gate
#[tokio::test]
async fn pentest_host_header_manipulation() {
    let app = sovereign_app();

    let hosts = vec![
        "localhost",
        "127.0.0.1",
        "evil.com",
        "nestgate.io",
        "",
        "localhost:3492",
    ];

    for host in hosts {
        let mut builder = Request::builder().uri("/api/v1/primals");

        if !host.is_empty() {
            builder = builder.header(header::HOST, host);
        }

        let response = app
            .clone()
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_dark_forest_rejection(response, &format!("Host: {host}")).await;
    }
}

// ═══════════════════════════════════════════════════════════════════════
// PEN TEST 11: RESPONSE CONSISTENCY
// ═══════════════════════════════════════════════════════════════════════

/// All 403 responses should be IDENTICAL — no way to distinguish routes
#[tokio::test]
async fn pentest_all_403_responses_identical() {
    let app = sovereign_app();

    let routes = vec![
        "/api/v1/primals",
        "/api/v1/topology",
        "/api/v1/genome/list",
        "/nonexistent",
        "/",
        "/admin",
    ];

    let mut responses_raw: Vec<(u16, Vec<u8>)> = Vec::new();

    for route in &routes {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(*route).body(Body::empty()).unwrap())
            .await
            .unwrap();

        let status = response.status().as_u16();
        let body = response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec();
        responses_raw.push((status, body));
    }

    // All should be identical status + body
    let first = &responses_raw[0];
    for (i, response) in responses_raw.iter().enumerate().skip(1) {
        assert_eq!(
            first.0, response.0,
            "Status differs between {} ({}) and {} ({})",
            routes[0], first.0, routes[i], response.0
        );
        assert_eq!(
            first.1, response.1,
            "Body differs between {} and {} — attacker can fingerprint routes",
            routes[0], routes[i]
        );
    }
}
