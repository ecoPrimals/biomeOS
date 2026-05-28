// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

//! Sovereign pen tests: token manipulation and wrong headers.

mod common;
use common::*;

use axum::http::StatusCode;

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
