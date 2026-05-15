// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Sovereign pen tests: HTTP method fuzzing and CORS preflight.

mod common;
use common::*;

use axum::http::StatusCode;

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
