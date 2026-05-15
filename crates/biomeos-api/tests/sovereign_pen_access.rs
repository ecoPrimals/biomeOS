// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Sovereign pen tests: unauthenticated route access (GET/POST).

mod common;
use common::*;

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
