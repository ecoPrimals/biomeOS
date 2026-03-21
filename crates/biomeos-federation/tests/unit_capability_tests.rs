// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Unit tests for capability system

use biomeos_federation::capability::{Capability, CapabilitySet};

#[test]
fn test_capability_from_str() {
    assert_eq!(Capability::from_str("storage"), Capability::Storage);
    assert_eq!(Capability::from_str("GAMING"), Capability::Gaming);
    assert_eq!(Capability::from_str("compute"), Capability::Compute);
    assert_eq!(Capability::from_str("sync"), Capability::Sync);
    assert_eq!(Capability::from_str("voice"), Capability::Voice);
    assert_eq!(Capability::from_str("video"), Capability::Video);
    assert_eq!(Capability::from_str("discovery"), Capability::Discovery);

    match Capability::from_str("custom:my_capability") {
        Capability::Custom(s) => assert_eq!(s, "my_capability"),
        _ => panic!("Expected Custom capability"),
    }

    match Capability::from_str("unknown") {
        Capability::Custom(s) => assert_eq!(s, "unknown"),
        _ => panic!("Expected Custom capability for unknown string"),
    }
}

#[test]
fn test_capability_display() {
    assert_eq!(Capability::Storage.to_string(), "storage");
    assert_eq!(Capability::Gaming.to_string(), "gaming");
    assert_eq!(
        Capability::Custom("test".to_string()).to_string(),
        "custom:test"
    );
}

#[test]
fn test_capability_set_new() {
    let set = CapabilitySet::new();
    assert!(set.is_empty());
    assert_eq!(set.all().len(), 0);
}

#[test]
fn test_capability_set_add_remove() {
    let mut set = CapabilitySet::new();

    set.add(Capability::Storage);
    assert!(set.has(&Capability::Storage));
    assert!(!set.has(&Capability::Compute));
    assert_eq!(set.all().len(), 1);

    set.add(Capability::Compute);
    assert_eq!(set.all().len(), 2);

    set.remove(&Capability::Storage);
    assert!(!set.has(&Capability::Storage));
    assert!(set.has(&Capability::Compute));
    assert_eq!(set.all().len(), 1);
}

#[test]
fn test_capability_set_from_vec() {
    let caps = vec![Capability::Storage, Capability::Compute, Capability::Gaming];
    let set = CapabilitySet::from_vec(caps);

    assert!(set.has(&Capability::Storage));
    assert!(set.has(&Capability::Compute));
    assert!(set.has(&Capability::Gaming));
    assert!(!set.has(&Capability::Voice));
    assert_eq!(set.all().len(), 3);
}

#[test]
fn test_capability_set_has_all() {
    let set1 = CapabilitySet::from_vec(vec![
        Capability::Storage,
        Capability::Compute,
        Capability::Gaming,
    ]);

    let set2 = CapabilitySet::from_vec(vec![Capability::Storage, Capability::Compute]);

    let set3 = CapabilitySet::from_vec(vec![Capability::Storage, Capability::Voice]);

    assert!(
        set1.has_all(&set2),
        "set1 should have all capabilities from set2"
    );
    assert!(!set1.has_all(&set3), "set1 should not have Voice from set3");
}

#[test]
fn test_capability_set_merge() {
    let mut set1 = CapabilitySet::from_vec(vec![Capability::Storage, Capability::Compute]);
    let set2 = CapabilitySet::from_vec(vec![Capability::Gaming, Capability::Voice]);

    set1.merge(&set2);

    assert!(set1.has(&Capability::Storage));
    assert!(set1.has(&Capability::Compute));
    assert!(set1.has(&Capability::Gaming));
    assert!(set1.has(&Capability::Voice));
    assert_eq!(set1.all().len(), 4);
}

#[test]
fn test_capability_set_presets() {
    let read_only = CapabilitySet::read_only();
    assert!(read_only.has(&Capability::ReadOnly));
    assert!(read_only.has(&Capability::Discovery));
    assert!(!read_only.has(&Capability::Write));

    let compute = CapabilitySet::compute_only();
    assert!(compute.has(&Capability::Compute));
    assert!(compute.has(&Capability::Discovery));
    assert!(!compute.has(&Capability::Storage));

    let full = CapabilitySet::full_access();
    assert!(full.has(&Capability::Storage));
    assert!(full.has(&Capability::Compute));
    assert!(full.has(&Capability::Admin));
}

#[test]
fn test_capability_set_duplicates() {
    let mut set = CapabilitySet::new();

    set.add(Capability::Storage);
    set.add(Capability::Storage);
    set.add(Capability::Storage);

    assert_eq!(set.all().len(), 1, "Duplicates should not be added");
}

#[test]
fn test_capability_custom_variants() {
    let cap1 = Capability::Custom("encryption".to_string());
    let cap2 = Capability::Custom("authentication".to_string());
    let cap3 = Capability::Custom("encryption".to_string());

    let mut set = CapabilitySet::new();
    set.add(cap1.clone());
    set.add(cap2.clone());
    set.add(cap3);

    assert!(set.has(&cap1));
    assert!(set.has(&cap2));
    assert_eq!(
        set.all().len(),
        2,
        "Same custom capability should not duplicate"
    );
}
