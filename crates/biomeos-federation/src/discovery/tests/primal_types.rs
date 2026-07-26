// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Discovery tests - extracted to keep discovery/mod.rs under 1000 lines

#![expect(clippy::expect_used, reason = "test assertions")]

use std::path::PathBuf;

use super::super::*;

#[test]
fn test_discovered_primal_serde_roundtrip() {
    let dp = DiscoveredPrimal {
        name: "beardog".into(),
        primal_type: "security".into(),
        capabilities: CapabilitySet::from_vec(vec![Capability::Storage]),
        endpoints: vec![PrimalEndpoint::UnixSocket {
            path: PathBuf::from("/tmp/beardog.sock"),
        }],
        metadata: HashMap::from([("key".into(), "val".into())]),
        error: None,
    };
    let json = serde_json::to_string(&dp).expect("serialize");
    let restored: DiscoveredPrimal = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.name, "beardog");
    assert_eq!(restored.primal_type, "security");
    assert!(restored.capabilities.has(&Capability::Storage));
    assert_eq!(restored.endpoints.len(), 1);
    assert_eq!(restored.metadata["key"], "val");
}

#[test]
fn test_discovered_primal_clone() {
    let dp = DiscoveredPrimal {
        name: "x".into(),
        primal_type: "y".into(),
        capabilities: CapabilitySet::new(),
        endpoints: vec![],
        metadata: HashMap::new(),
        error: None,
    };
    let cloned = dp;
    assert_eq!(cloned.name, "x");
}

#[test]
fn test_discovered_primal_debug() {
    let dp = DiscoveredPrimal {
        name: "test".into(),
        primal_type: "t".into(),
        capabilities: CapabilitySet::new(),
        endpoints: vec![],
        metadata: HashMap::new(),
        error: None,
    };
    let dbg = format!("{dp:?}");
    assert!(dbg.contains("test"));
    assert!(dbg.contains("DiscoveredPrimal"));
}

#[test]
fn test_primal_discovery_new() {
    let pd = PrimalDiscovery::new();
    assert!(pd.all().is_empty());
}

#[test]
fn test_primal_discovery_default() {
    let pd = PrimalDiscovery::default();
    assert!(pd.all().is_empty());
}

#[test]
fn test_primal_discovery_get_none() {
    let pd = PrimalDiscovery::new();
    assert!(pd.get("unknown").is_none());
}

#[test]
fn test_primal_discovery_with_registered() {
    let mut pd = PrimalDiscovery::new();
    pd.discovered_primals.insert(
        "beardog".into(),
        DiscoveredPrimal {
            name: "beardog".into(),
            primal_type: "security".into(),
            capabilities: CapabilitySet::from_vec(vec![Capability::Storage]),
            endpoints: vec![],
            metadata: HashMap::new(),
            error: None,
        },
    );

    assert!(pd.get("beardog").is_some());
    assert_eq!(pd.get("beardog").expect("should exist").name, "beardog");
    assert_eq!(pd.all().len(), 1);
}

#[test]
fn test_primal_discovery_with_capability() {
    let mut pd = PrimalDiscovery::new();
    pd.discovered_primals.insert(
        "store".into(),
        DiscoveredPrimal {
            name: "store".into(),
            primal_type: "storage".into(),
            capabilities: CapabilitySet::from_vec(vec![Capability::Storage]),
            endpoints: vec![],
            metadata: HashMap::new(),
            error: None,
        },
    );
    pd.discovered_primals.insert(
        "compute".into(),
        DiscoveredPrimal {
            name: "compute".into(),
            primal_type: "compute".into(),
            capabilities: CapabilitySet::from_vec(vec![Capability::Compute]),
            endpoints: vec![],
            metadata: HashMap::new(),
            error: None,
        },
    );

    assert_eq!(pd.with_capability(&Capability::Storage).len(), 1);
    assert_eq!(pd.with_capability(&Capability::Compute).len(), 1);
    assert_eq!(pd.with_capability(&Capability::Voice).len(), 0);
}
