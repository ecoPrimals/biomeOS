// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::super::capability_domains::*;

#[test]
fn test_capability_to_provider_security_domain() {
    // Security capabilities should map to beardog
    assert_eq!(capability_to_provider_fallback("security"), Some("beardog"));
    assert_eq!(capability_to_provider_fallback("crypto"), Some("beardog"));
    assert_eq!(
        capability_to_provider_fallback("encryption"),
        Some("beardog")
    );
    assert_eq!(capability_to_provider_fallback("genetic"), Some("beardog"));
    assert_eq!(capability_to_provider_fallback("beacon"), Some("beardog"));
    assert_eq!(capability_to_provider_fallback("tls"), Some("beardog"));
    assert_eq!(capability_to_provider_fallback("jwt"), Some("beardog"));
}

#[test]
fn test_capability_to_provider_network_domain() {
    // Network capabilities should map to songbird
    assert_eq!(
        capability_to_provider_fallback("discovery"),
        Some("songbird")
    );
    assert_eq!(capability_to_provider_fallback("http"), Some("songbird"));
    assert_eq!(
        capability_to_provider_fallback("orchestration"),
        Some("songbird")
    );
    assert_eq!(
        capability_to_provider_fallback("federation"),
        Some("songbird")
    );
    assert_eq!(capability_to_provider_fallback("network"), Some("songbird"));
}

#[test]
fn test_capability_to_provider_storage_domain() {
    // Storage capabilities should map to nestgate
    assert_eq!(capability_to_provider_fallback("storage"), Some("nestgate"));
    assert_eq!(
        capability_to_provider_fallback("versioning"),
        Some("nestgate")
    );
    assert_eq!(
        capability_to_provider_fallback("persistence"),
        Some("nestgate")
    );
}

#[test]
fn test_capability_to_provider_content_domain() {
    assert_eq!(capability_to_provider_fallback("content"), Some("nestgate"));
    assert_eq!(
        capability_to_provider_fallback("content_addressed"),
        Some("nestgate")
    );
    assert_eq!(
        capability_to_provider_fallback("publishing"),
        Some("nestgate")
    );
}

#[test]
fn test_capability_to_provider_compute_domain() {
    assert_eq!(
        capability_to_provider_fallback("compute"),
        Some("toadstool")
    );
    assert_eq!(
        capability_to_provider_fallback("execution"),
        Some("toadstool")
    );
    assert_eq!(
        capability_to_provider_fallback("parsing"),
        Some("toadstool")
    );
    assert_eq!(
        capability_to_provider_fallback("hardware_learning"),
        Some("toadstool")
    );
    assert_eq!(
        capability_to_provider_fallback("compute.hardware.observe"),
        Some("toadstool")
    );
    assert_eq!(
        capability_to_provider_fallback("compute.hardware.apply"),
        Some("toadstool")
    );
}

#[test]
fn test_capability_to_provider_ai_domain() {
    // AI capabilities should map to squirrel
    assert_eq!(capability_to_provider_fallback("ai"), Some("squirrel"));
    assert_eq!(capability_to_provider_fallback("mcp"), Some("squirrel"));
    assert_eq!(
        capability_to_provider_fallback("assistance"),
        Some("squirrel")
    );
    assert_eq!(capability_to_provider_fallback("ml"), Some("squirrel"));
}

#[test]
fn test_capability_to_provider_prefix_matching() {
    // Prefix matching: "crypto.encrypt" should match "crypto" domain
    assert_eq!(
        capability_to_provider_fallback("crypto.encrypt"),
        Some("beardog")
    );
    assert_eq!(
        capability_to_provider_fallback("storage.put"),
        Some("nestgate")
    );
    assert_eq!(
        capability_to_provider_fallback("network.beacon_exchange"),
        Some("songbird")
    );
    assert_eq!(
        capability_to_provider_fallback("ai.query"),
        Some("squirrel")
    );
}
