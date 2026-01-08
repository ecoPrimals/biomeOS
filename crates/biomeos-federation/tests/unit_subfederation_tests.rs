//! Unit tests for sub-federation system

use biomeos_federation::capability::{Capability, CapabilitySet};
use biomeos_federation::subfederation::{SubFederation, IsolationLevel};
use chrono::Utc;

#[test]
fn test_subfederation_creation() {
    let members = vec!["node-alpha".to_string(), "node-beta".to_string()];
    let caps = CapabilitySet::from_vec(vec![Capability::Gaming, Capability::Voice]);
    
    let subfed = SubFederation::new(
        "gaming".to_string(),
        "nat0".to_string(),
        members.clone(),
        caps,
        IsolationLevel::Low,
    );
    
    assert_eq!(subfed.name, "gaming");
    assert_eq!(subfed.parent_family, "nat0");
    assert_eq!(subfed.members.len(), 2);
    assert_eq!(subfed.isolation_level, IsolationLevel::Low);
}

#[test]
fn test_subfederation_wildcard_membership() {
    let members = vec!["node-alpha-*".to_string(), "node-beta-laptop".to_string()];
    let caps = CapabilitySet::new();
    
    let subfed = SubFederation::new(
        "test".to_string(),
        "family".to_string(),
        members,
        caps,
        IsolationLevel::None,
    );
    
    assert!(subfed.is_member("node-alpha-laptop"), "Should match wildcard");
    assert!(subfed.is_member("node-alpha-desktop"), "Should match wildcard");
    assert!(subfed.is_member("node-beta-laptop"), "Should match exact");
    assert!(!subfed.is_member("node-gamma-laptop"), "Should not match");
}

#[test]
fn test_subfederation_capability_check() {
    let members = vec!["node-alpha-*".to_string()];
    let mut caps = CapabilitySet::new();
    caps.add(Capability::Gaming);
    caps.add(Capability::Voice);
    
    let subfed = SubFederation::new(
        "gaming".to_string(),
        "family".to_string(),
        members,
        caps,
        IsolationLevel::Low,
    );
    
    assert!(
        subfed.has_capability("node-alpha-laptop", &Capability::Gaming),
        "Member should have gaming capability"
    );
    assert!(
        subfed.has_capability("node-alpha-laptop", &Capability::Voice),
        "Member should have voice capability"
    );
    assert!(
        !subfed.has_capability("node-alpha-laptop", &Capability::Storage),
        "Member should not have storage capability"
    );
    assert!(
        !subfed.has_capability("node-beta-laptop", &Capability::Gaming),
        "Non-member should not have any capability"
    );
}

#[test]
fn test_subfederation_add_remove_member() {
    let members = vec!["node-alpha".to_string()];
    let caps = CapabilitySet::new();
    
    let mut subfed = SubFederation::new(
        "test".to_string(),
        "family".to_string(),
        members,
        caps,
        IsolationLevel::None,
    );
    
    assert!(subfed.is_member("node-alpha"));
    assert!(!subfed.is_member("node-beta"));
    
    subfed.add_member("node-beta".to_string());
    assert!(subfed.is_member("node-beta"));
    assert_eq!(subfed.members.len(), 2);
    
    subfed.remove_member("node-alpha");
    assert!(!subfed.is_member("node-alpha"));
    assert!(subfed.is_member("node-beta"));
    assert_eq!(subfed.members.len(), 1);
}

#[test]
fn test_subfederation_isolation_levels() {
    assert!(IsolationLevel::None.allows_auto_approval());
    assert!(IsolationLevel::Low.allows_auto_approval());
    assert!(IsolationLevel::Medium.allows_auto_approval());
    assert!(!IsolationLevel::High.allows_auto_approval());
    assert!(!IsolationLevel::Critical.allows_auto_approval());
}

#[test]
fn test_subfederation_critical_isolation_denies_access() {
    let members = vec!["node-alpha".to_string()];
    let mut caps = CapabilitySet::new();
    caps.add(Capability::Storage);
    
    let subfed = SubFederation::new(
        "critical".to_string(),
        "family".to_string(),
        members,
        caps,
        IsolationLevel::Critical,
    );
    
    // Even though node is a member and has capability,
    // Critical isolation requires manual approval
    assert!(!subfed.has_capability("node-alpha", &Capability::Storage));
}

#[test]
fn test_subfederation_encryption_key_ref() {
    let members = vec!["node-alpha".to_string()];
    let caps = CapabilitySet::new();
    
    let mut subfed = SubFederation::new(
        "secure".to_string(),
        "family".to_string(),
        members,
        caps,
        IsolationLevel::High,
    );
    
    assert!(subfed.encryption_key_ref.is_none());
    
    subfed.set_encryption_key_ref("beardog-key-12345".to_string());
    assert_eq!(subfed.encryption_key_ref.unwrap(), "beardog-key-12345");
}

#[test]
fn test_subfederation_multiple_wildcards() {
    let members = vec![
        "node-alpha-*".to_string(),
        "node-beta-*".to_string(),
        "node-gamma-laptop".to_string(),
    ];
    let caps = CapabilitySet::new();
    
    let subfed = SubFederation::new(
        "multi".to_string(),
        "family".to_string(),
        members,
        caps,
        IsolationLevel::None,
    );
    
    assert!(subfed.is_member("node-alpha-desktop"));
    assert!(subfed.is_member("node-beta-laptop"));
    assert!(subfed.is_member("node-gamma-laptop"));
    assert!(!subfed.is_member("node-delta-desktop"));
}

#[test]
fn test_subfederation_empty_members() {
    let members = vec![];
    let caps = CapabilitySet::new();
    
    let subfed = SubFederation::new(
        "empty".to_string(),
        "family".to_string(),
        members,
        caps,
        IsolationLevel::None,
    );
    
    assert_eq!(subfed.members.len(), 0);
    assert!(!subfed.is_member("node-alpha"));
}

