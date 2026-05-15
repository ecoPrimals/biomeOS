// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Sovereign pen tests: request bodies, content-type, URL encoding.

mod common;
use common::*;

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
