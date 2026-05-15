// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Sovereign pen tests: Host header manipulation and 403 response consistency.

mod common;
use common::*;

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
