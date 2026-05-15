// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Sovereign pen tests: health endpoints and path bypass attempts.

mod common;
use common::*;

use axum::http::StatusCode;

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
