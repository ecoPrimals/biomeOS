// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Capability category for grouping

use serde::{Deserialize, Serialize};
use std::fmt;

/// Capability category for grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityCategory {
    /// Security and cryptography capabilities
    Security,
    /// Communication and networking capabilities
    Communication,
    /// Compute and execution capabilities
    Compute,
    /// Storage and data capabilities
    Storage,
    /// User interface and rendering capabilities
    UserInterface,
    /// Orchestration and management capabilities
    Orchestration,
    /// AI and intelligence capabilities
    AI,
    /// Specialized and custom capabilities
    Specialized,
}

impl fmt::Display for CapabilityCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = format!("{self:?}");
        write!(f, "{s}")
    }
}

#[allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_capability_category_variants() {
        let _ = CapabilityCategory::Security;
        let _ = CapabilityCategory::Communication;
        let _ = CapabilityCategory::Compute;
        let _ = CapabilityCategory::Storage;
        let _ = CapabilityCategory::UserInterface;
        let _ = CapabilityCategory::Orchestration;
        let _ = CapabilityCategory::AI;
        let _ = CapabilityCategory::Specialized;
    }

    #[test]
    fn test_capability_category_display() {
        assert_eq!(CapabilityCategory::Security.to_string(), "Security");
        assert_eq!(CapabilityCategory::AI.to_string(), "AI");
        assert_eq!(CapabilityCategory::Compute.to_string(), "Compute");
    }

    #[test]
    fn test_capability_category_serde() {
        for category in [
            CapabilityCategory::Security,
            CapabilityCategory::Communication,
            CapabilityCategory::Compute,
            CapabilityCategory::Storage,
            CapabilityCategory::UserInterface,
            CapabilityCategory::Orchestration,
            CapabilityCategory::AI,
            CapabilityCategory::Specialized,
        ] {
            let json = serde_json::to_string(&category).expect("serialize");
            let parsed: CapabilityCategory = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(parsed, category);
        }
    }

    #[test]
    fn test_capability_category_clone_copy() {
        let cat = CapabilityCategory::Storage;
        let cloned = cat;
        assert_eq!(cat, cloned);
    }

    #[test]
    fn test_capability_category_hash_eq() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(CapabilityCategory::Security);
        set.insert(CapabilityCategory::AI);
        assert_eq!(set.len(), 2);
        assert!(set.contains(&CapabilityCategory::Security));
    }
}
