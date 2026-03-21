// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Spore type definitions
//!
//! ColdSpore: Storage/archive format (genetic material only)
//! LiveSpore: Deployment-ready format (with execution environment)

use serde::{Deserialize, Serialize};

/// Type of spore
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SporeType {
    /// Cold spore - genetic material only, for storage/archival
    ///
    /// Like seeds in storage - dormant, preserved, can be stored indefinitely.
    /// No execution environment, just the genetic material and configuration.
    /// Ideal for backup, distribution, long-term storage.
    Cold,

    /// Live spore - ready for immediate deployment
    ///
    /// Like seeds ready to germinate - active, executable, ready to grow.
    /// Includes execution environment, handles filesystem limitations.
    /// Ideal for USB deployment, rapid activation.
    #[default]
    Live,
}

impl SporeType {
    /// Check if this spore type requires execution environment
    pub fn requires_execution_env(&self) -> bool {
        matches!(self, SporeType::Live)
    }

    /// Check if this spore type is for storage/archival
    pub fn is_archival(&self) -> bool {
        matches!(self, SporeType::Cold)
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            SporeType::Cold => "Genetic material only (storage/archive)",
            SporeType::Live => "Deployment-ready (executable)",
        }
    }

    /// Get emoji representation
    pub fn emoji(&self) -> &'static str {
        match self {
            SporeType::Cold => "❄️",
            SporeType::Live => "🌱",
        }
    }
}

impl std::fmt::Display for SporeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SporeType::Cold => write!(f, "ColdSpore"),
            SporeType::Live => write!(f, "LiveSpore"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spore_type_properties() {
        assert!(SporeType::Live.requires_execution_env());
        assert!(!SporeType::Cold.requires_execution_env());

        assert!(SporeType::Cold.is_archival());
        assert!(!SporeType::Live.is_archival());
    }

    #[test]
    fn test_spore_type_display() {
        assert_eq!(SporeType::Cold.to_string(), "ColdSpore");
        assert_eq!(SporeType::Live.to_string(), "LiveSpore");
    }

    #[test]
    fn test_default() {
        assert_eq!(SporeType::default(), SporeType::Live);
    }

    #[test]
    fn test_description() {
        assert!(SporeType::Cold.description().contains("storage"));
        assert!(SporeType::Live.description().contains("Deployment"));
    }

    #[test]
    fn test_emoji() {
        assert_eq!(SporeType::Cold.emoji(), "❄️");
        assert_eq!(SporeType::Live.emoji(), "🌱");
    }

    #[test]
    fn test_serialization_roundtrip_json() {
        let live = SporeType::Live;
        let json = serde_json::to_string(&live).expect("serialize");
        let deserialized: SporeType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized, live);

        let cold = SporeType::Cold;
        let json = serde_json::to_string(&cold).expect("serialize");
        let deserialized: SporeType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized, cold);
    }

    #[test]
    fn test_equality() {
        assert_eq!(SporeType::Live, SporeType::Live);
        assert_eq!(SporeType::Cold, SporeType::Cold);
        assert_ne!(SporeType::Live, SporeType::Cold);
    }

    #[test]
    fn test_clone_and_copy() {
        let original = SporeType::Cold;
        let copied = original; // Copy trait
        let also_copied = original; // Still valid because Copy

        assert_eq!(original, copied);
        assert_eq!(original, also_copied);
    }

    #[test]
    fn test_debug_format() {
        let debug = format!("{:?}", SporeType::Live);
        assert_eq!(debug, "Live");

        let debug = format!("{:?}", SporeType::Cold);
        assert_eq!(debug, "Cold");
    }
}
