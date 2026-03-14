// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Capability-based access control for sub-federations
//!
//! This module defines capabilities that can be granted to nodes within sub-federations.
//! Capabilities are discovered at runtime, not hardcoded.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

/// A capability that can be granted to nodes in a sub-federation
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// Storage and file system access
    Storage,

    /// Compute resources (CPU, memory)
    Compute,

    /// Gaming-related capabilities (coordination, lobbies)
    Gaming,

    /// Data synchronization
    Sync,

    /// Voice communication
    Voice,

    /// Video streaming/communication
    Video,

    /// Network discovery (can discover other nodes)
    Discovery,

    /// Read-only access
    ReadOnly,

    /// Write access
    Write,

    /// Admin/management capabilities
    Admin,

    /// Custom capability (user-defined)
    Custom(String),
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Capability::Storage => write!(f, "storage"),
            Capability::Compute => write!(f, "compute"),
            Capability::Gaming => write!(f, "gaming"),
            Capability::Sync => write!(f, "sync"),
            Capability::Voice => write!(f, "voice"),
            Capability::Video => write!(f, "video"),
            Capability::Discovery => write!(f, "discovery"),
            Capability::ReadOnly => write!(f, "read_only"),
            Capability::Write => write!(f, "write"),
            Capability::Admin => write!(f, "admin"),
            Capability::Custom(s) => write!(f, "custom:{s}"),
        }
    }
}

impl Capability {
    /// Parse capability from string (convenience method)
    ///
    /// Note: For idiomatic Rust, prefer `s.parse::<Capability>()` which uses the `FromStr` trait.
    /// This method is provided for backwards compatibility.
    #[allow(clippy::should_implement_trait)] // We do implement FromStr, this is a convenience wrapper
    pub fn from_str(s: &str) -> Self {
        // Use the FromStr trait implementation (Err = Infallible)
        s.parse().expect("Capability::FromStr is infallible")
    }
}

/// Implement standard FromStr trait for idiomatic parsing
impl std::str::FromStr for Capability {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "storage" => Capability::Storage,
            "compute" => Capability::Compute,
            "gaming" => Capability::Gaming,
            "sync" => Capability::Sync,
            "voice" => Capability::Voice,
            "video" => Capability::Video,
            "discovery" => Capability::Discovery,
            "read_only" => Capability::ReadOnly,
            "write" => Capability::Write,
            "admin" => Capability::Admin,
            // Additional mappings for common primal capability types
            "security" => Capability::Custom("security".to_string()),
            "encryption" => Capability::Custom("encryption".to_string()),
            "trust" => Capability::Custom("trust".to_string()),
            "mesh" => Capability::Custom("mesh".to_string()),
            "ai" => Capability::Custom("ai".to_string()),
            "ml" => Capability::Custom("ml".to_string()),
            "inference" => Capability::Custom("inference".to_string()),
            "crypto" => Capability::Custom("crypto".to_string()),
            _ => {
                if let Some(custom) = s.strip_prefix("custom:") {
                    Capability::Custom(custom.to_string())
                } else {
                    Capability::Custom(s.to_string())
                }
            }
        })
    }
}

/// Implement From for ergonomic conversion
///
/// Note: This is infallible because `FromStr` returns `Infallible` error type.
/// Unknown capabilities become `Capability::Custom(s)`.
impl From<&str> for Capability {
    fn from(s: &str) -> Self {
        // SAFETY: `FromStr` for `Capability` returns `Result<Self, Infallible>`
        // which means parsing can never fail - it always produces a valid Capability.
        // Unknown strings become `Capability::Custom(s)`.
        match s.parse::<Capability>() {
            Ok(cap) => cap,
            Err(infallible) => match infallible {},
        }
    }
}

/// A set of capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySet {
    capabilities: HashSet<Capability>,
}

impl CapabilitySet {
    /// Create an empty capability set
    pub fn new() -> Self {
        Self {
            capabilities: HashSet::new(),
        }
    }

    /// Create a capability set from a vector
    pub fn from_vec(caps: Vec<Capability>) -> Self {
        Self {
            capabilities: caps.into_iter().collect(),
        }
    }

    /// Create a capability set from tags (e.g., from Songbird discovery)
    ///
    /// Parses each tag as a capability using the FromStr trait.
    /// Unknown tags are treated as Custom capabilities.
    pub fn from_tags(tags: &[String]) -> Self {
        let caps: Vec<Capability> = tags
            .iter()
            .map(|tag| tag.parse().expect("Capability::FromStr is infallible"))
            .collect();
        Self::from_vec(caps)
    }

    /// Add a capability
    pub fn add(&mut self, cap: Capability) {
        self.capabilities.insert(cap);
    }

    /// Remove a capability
    pub fn remove(&mut self, cap: &Capability) {
        self.capabilities.remove(cap);
    }

    /// Check if a capability is present
    pub fn has(&self, cap: &Capability) -> bool {
        self.capabilities.contains(cap)
    }

    /// Check if all capabilities from another set are present
    pub fn has_all(&self, other: &CapabilitySet) -> bool {
        other.capabilities.iter().all(|cap| self.has(cap))
    }

    /// Get all capabilities
    pub fn all(&self) -> Vec<&Capability> {
        self.capabilities.iter().collect()
    }

    /// Check if set is empty
    pub fn is_empty(&self) -> bool {
        self.capabilities.is_empty()
    }

    /// Merge with another capability set
    pub fn merge(&mut self, other: &CapabilitySet) {
        for cap in &other.capabilities {
            self.capabilities.insert(cap.clone());
        }
    }

    /// Create a read-only capability set
    pub fn read_only() -> Self {
        Self::from_vec(vec![Capability::ReadOnly, Capability::Discovery])
    }

    /// Create a compute-only capability set
    pub fn compute_only() -> Self {
        Self::from_vec(vec![Capability::Compute, Capability::Discovery])
    }

    /// Create a full access capability set
    pub fn full_access() -> Self {
        Self::from_vec(vec![
            Capability::Storage,
            Capability::Compute,
            Capability::Gaming,
            Capability::Sync,
            Capability::Voice,
            Capability::Video,
            Capability::Discovery,
            Capability::ReadOnly,
            Capability::Write,
            Capability::Admin,
        ])
    }
}

impl Default for CapabilitySet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_from_str() {
        assert_eq!(Capability::from_str("storage"), Capability::Storage);
        assert_eq!(Capability::from_str("GAMING"), Capability::Gaming);

        match Capability::from_str("custom:my_capability") {
            Capability::Custom(s) => assert_eq!(s, "my_capability"),
            _ => panic!("Expected Custom capability"),
        }
    }

    #[test]
    fn test_capability_set() {
        let mut set = CapabilitySet::new();
        assert!(set.is_empty());

        set.add(Capability::Storage);
        assert!(set.has(&Capability::Storage));
        assert!(!set.has(&Capability::Compute));

        set.add(Capability::Compute);
        assert_eq!(set.all().len(), 2);
    }

    #[test]
    fn test_capability_set_presets() {
        let read_only = CapabilitySet::read_only();
        assert!(read_only.has(&Capability::ReadOnly));
        assert!(read_only.has(&Capability::Discovery));

        let compute = CapabilitySet::compute_only();
        assert!(compute.has(&Capability::Compute));

        let full = CapabilitySet::full_access();
        assert!(full.has(&Capability::Storage));
        assert!(full.has(&Capability::Admin));
    }

    #[test]
    fn test_capability_from_str_custom_mappings() {
        assert_eq!(
            Capability::from_str("security"),
            Capability::Custom("security".to_string())
        );
        assert_eq!(
            Capability::from_str("encryption"),
            Capability::Custom("encryption".to_string())
        );
        assert_eq!(
            Capability::from_str("trust"),
            Capability::Custom("trust".to_string())
        );
        assert_eq!(
            Capability::from_str("mesh"),
            Capability::Custom("mesh".to_string())
        );
        assert_eq!(
            Capability::from_str("ai"),
            Capability::Custom("ai".to_string())
        );
        assert_eq!(
            Capability::from_str("ml"),
            Capability::Custom("ml".to_string())
        );
        assert_eq!(
            Capability::from_str("inference"),
            Capability::Custom("inference".to_string())
        );
        assert_eq!(
            Capability::from_str("crypto"),
            Capability::Custom("crypto".to_string())
        );
    }

    #[test]
    fn test_capability_from_str_trait() {
        let cap: Capability = "storage".parse().expect("parse storage");
        assert_eq!(cap, Capability::Storage);
        let cap: Capability = "unknown".parse().expect("parse unknown");
        assert_eq!(cap, Capability::Custom("unknown".to_string()));
    }

    #[test]
    fn test_capability_from_ref_str() {
        let cap: Capability = "storage".into();
        assert_eq!(cap, Capability::Storage);
        let cap: Capability = "custom:foo".into();
        assert_eq!(cap, Capability::Custom("foo".to_string()));
    }

    #[test]
    fn test_capability_display() {
        assert_eq!(Capability::Storage.to_string(), "storage");
        assert_eq!(Capability::ReadOnly.to_string(), "read_only");
        assert_eq!(
            Capability::Custom("foo".to_string()).to_string(),
            "custom:foo"
        );
    }

    #[test]
    fn test_capability_serialization_roundtrip() {
        let variants = [
            Capability::Storage,
            Capability::Compute,
            Capability::Gaming,
            Capability::Sync,
            Capability::Voice,
            Capability::Video,
            Capability::Discovery,
            Capability::ReadOnly,
            Capability::Write,
            Capability::Admin,
            Capability::Custom("mycap".to_string()),
        ];
        for cap in &variants {
            let json = serde_json::to_string(cap).expect("serialize Capability");
            let restored: Capability = serde_json::from_str(&json).expect("deserialize Capability");
            assert_eq!(cap, &restored);
        }
    }

    #[test]
    fn test_capability_set_from_tags() {
        let tags = vec![
            "storage".to_string(),
            "gaming".to_string(),
            "custom:foo".to_string(),
        ];
        let set = CapabilitySet::from_tags(&tags);
        assert!(set.has(&Capability::Storage));
        assert!(set.has(&Capability::Gaming));
        assert!(set.has(&Capability::Custom("foo".to_string())));
        assert_eq!(set.all().len(), 3);
    }

    #[test]
    fn test_capability_set_default() {
        let set = CapabilitySet::default();
        assert!(set.is_empty());
    }

    #[test]
    fn test_capability_set_merge() {
        let mut set1 = CapabilitySet::from_vec(vec![Capability::Storage]);
        let set2 = CapabilitySet::from_vec(vec![Capability::Storage, Capability::Compute]);
        set1.merge(&set2);
        assert!(set1.has(&Capability::Storage));
        assert!(set1.has(&Capability::Compute));
        assert_eq!(set1.all().len(), 2);
    }

    #[test]
    fn test_capability_set_has_all() {
        let set1 = CapabilitySet::from_vec(vec![
            Capability::Storage,
            Capability::Compute,
            Capability::Gaming,
        ]);
        let set2 = CapabilitySet::from_vec(vec![Capability::Storage, Capability::Compute]);
        assert!(set1.has_all(&set2));
        let set3 = CapabilitySet::from_vec(vec![Capability::Storage, Capability::Voice]);
        assert!(!set1.has_all(&set3));
    }

    #[test]
    fn test_capability_set_serialization_roundtrip() {
        let set = CapabilitySet::from_vec(vec![
            Capability::Storage,
            Capability::Gaming,
            Capability::Custom("foo".to_string()),
        ]);
        let json = serde_json::to_string(&set).expect("serialize CapabilitySet");
        let restored: CapabilitySet =
            serde_json::from_str(&json).expect("deserialize CapabilitySet");
        assert!(restored.has(&Capability::Storage));
        assert!(restored.has(&Capability::Gaming));
        assert!(restored.has(&Capability::Custom("foo".to_string())));
    }
}
