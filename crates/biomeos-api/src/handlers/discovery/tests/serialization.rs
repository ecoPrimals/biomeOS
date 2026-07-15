// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[test]
fn test_discovered_primal_serialization() {
    let primal = DiscoveredPrimal {
        id: "beardog-local".to_string(),
        name: "BearDog".to_string(),
        primal_type: "security".to_string(),
        version: "0.11.0".to_string(),
        health: "healthy".to_string(),
        capabilities: vec!["security".to_string(), "crypto".to_string()],
        endpoint: "unix:///tmp/beardog.sock".to_string(),
        last_seen: 1_234_567_890,
        trust_level: Some(3),
        family_id: Some("test-family".to_string()),
        allowed_capabilities: Some(vec!["*".to_string()]),
        denied_capabilities: Some(vec![]),
        error: None,
    };

    let json = serde_json::to_string(&primal).expect("serialize");
    assert!(json.contains("beardog-local"));
    assert!(json.contains("BearDog"));
    assert!(json.contains("security"));

    let deserialized: DiscoveredPrimal = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, "beardog-local");
    assert_eq!(deserialized.trust_level, Some(3));
}

#[test]
fn test_discovered_primal_optional_fields_skip_none() {
    let primal = DiscoveredPrimal {
        id: "test".to_string(),
        name: "Test".to_string(),
        primal_type: "test".to_string(),
        version: "1.0.0".to_string(),
        health: "healthy".to_string(),
        capabilities: vec![],
        endpoint: "unix:///tmp/test.sock".to_string(),
        last_seen: 0,
        trust_level: None, // Should skip
        family_id: None,   // Should skip
        allowed_capabilities: None,
        denied_capabilities: None,
        error: None,
    };

    let json = serde_json::to_string(&primal).expect("serialize");
    // Optional None fields should not appear in JSON (skip_serializing_if)
    assert!(!json.contains("trust_level"));
    assert!(!json.contains("family_id"));
    assert!(!json.contains("error"));
}

#[test]
fn test_discovered_primals_response_serialization() {
    let response = DiscoveredPrimalsResponse {
        primals: vec![],
        count: 0,
        mode: "standalone".to_string(),
    };

    let json = serde_json::to_string(&response).expect("serialize");
    assert!(json.contains("\"count\":0"));
    assert!(json.contains("\"mode\":\"standalone\""));
}

#[test]
fn test_discovered_primal_deserialization() {
    let json = r#"{
        "id": "test-primal",
        "name": "Test",
        "primal_type": "security",
        "version": "1.0.0",
        "health": "healthy",
        "capabilities": ["security", "crypto"],
        "endpoint": "unix:///tmp/test.sock",
        "last_seen": 1234567890,
        "trust_level": 2,
        "family_id": "test-family"
    }"#;

    let primal: DiscoveredPrimal = serde_json::from_str(json).expect("should deserialize");
    assert_eq!(primal.id, "test-primal");
    assert_eq!(primal.name, "Test");
    assert_eq!(primal.trust_level, Some(2));
    assert_eq!(primal.family_id, Some("test-family".to_string()));
}

#[test]
fn test_discovered_primal_all_fields() {
    let primal = DiscoveredPrimal {
        id: "full-primal".to_string(),
        name: "Full".to_string(),
        primal_type: "compute".to_string(),
        version: "2.0.0".to_string(),
        health: "degraded".to_string(),
        capabilities: vec!["compute".to_string(), "execution".to_string()],
        endpoint: "unix:///tmp/full.sock".to_string(),
        last_seen: 9_999_999_999,
        trust_level: Some(2),
        family_id: Some("family-1".to_string()),
        allowed_capabilities: Some(vec!["compute/*".to_string()]),
        denied_capabilities: Some(vec!["compute/admin".to_string()]),
        error: Some("health probe failed: connection refused".to_string()),
    };

    let json = serde_json::to_string(&primal).expect("should serialize");
    let deserialized: DiscoveredPrimal = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(deserialized.id, primal.id);
    assert_eq!(
        deserialized.allowed_capabilities,
        primal.allowed_capabilities
    );
    assert_eq!(deserialized.denied_capabilities, primal.denied_capabilities);
}

#[test]
fn test_discovered_primals_response_with_primals() {
    let primals = vec![
        DiscoveredPrimal {
            id: "primal-1".to_string(),
            name: "Primal1".to_string(),
            primal_type: "security".to_string(),
            version: "1.0.0".to_string(),
            health: "healthy".to_string(),
            capabilities: vec!["security".to_string()],
            endpoint: "unix:///tmp/p1.sock".to_string(),
            last_seen: 1_234_567_890,
            trust_level: Some(3),
            family_id: Some("family-1".to_string()),
            allowed_capabilities: None,
            denied_capabilities: None,
            error: None,
        },
        DiscoveredPrimal {
            id: "primal-2".to_string(),
            name: "Primal2".to_string(),
            primal_type: "orchestration".to_string(),
            version: "2.0.0".to_string(),
            health: "healthy".to_string(),
            capabilities: vec!["orchestration".to_string()],
            endpoint: "unix:///tmp/p2.sock".to_string(),
            last_seen: 1_234_567_891,
            trust_level: Some(2),
            family_id: None,
            allowed_capabilities: None,
            denied_capabilities: None,
            error: None,
        },
    ];

    let response = DiscoveredPrimalsResponse {
        primals: primals.clone(),
        count: primals.len(),
        mode: "live".to_string(),
    };

    let json = serde_json::to_string(&response).expect("should serialize");
    assert!(json.contains("\"count\":2"));
    assert!(json.contains("\"mode\":\"live\""));
    assert!(json.contains("primal-1"));
    assert!(json.contains("primal-2"));
}

#[test]
fn test_discovered_primal_empty_capabilities() {
    let primal = DiscoveredPrimal {
        id: "empty-caps".to_string(),
        name: "Empty".to_string(),
        primal_type: "security".to_string(),
        version: "1.0.0".to_string(),
        health: "healthy".to_string(),
        capabilities: vec![],
        endpoint: "unix:///tmp/empty.sock".to_string(),
        last_seen: 0,
        trust_level: None,
        family_id: None,
        allowed_capabilities: None,
        denied_capabilities: None,
        error: None,
    };
    let json = serde_json::to_string(&primal).expect("serialize");
    assert!(json.contains("empty-caps"));
    assert!(json.contains("\"capabilities\":[]"));
}
