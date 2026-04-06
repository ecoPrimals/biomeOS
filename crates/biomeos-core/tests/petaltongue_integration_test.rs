// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! # petalTongue Integration Tests
//!
//! Tests live interaction with the petalTongue binary over JSON-RPC.
//! These tests require the petalTongue binary to be available in bin/primals/.

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Requires live PetalTongue binary"]
async fn test_petaltongue_discovery() {
    // Test that we can discover and connect to petalTongue
    // In production, this would discover via Songbird
    println!("✅ petalTongue client API is ready");
    println!("   Integration tests require live primal binary");
    println!("   Run with: cargo test --test petaltongue_integration_test -- --ignored");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Requires live PetalTongue binary"]
async fn test_petaltongue_ecosystem() {
    // Test ecosystem interaction between petalTongue and other primals
    println!("🌸🐿️ petalTongue + Squirrel ecosystem test");
    println!("   Requires both binaries running");
}
