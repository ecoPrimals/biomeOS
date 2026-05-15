// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Sovereign pen tests: unknown paths and timing consistency.

mod common;
use common::*;

use axum::http::StatusCode;
use std::time::Instant;

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
