// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

// biomeos-genomebin-v3/src/manifest.rs
// genomeBin manifest - metadata and capabilities
//
// Deep Debt Principles:
// - Self-describing (complete metadata)
// - Capability-based (not hardcoded)
// - Modern idiomatic Rust (builder pattern)

use crate::Arch;
use serde::{Deserialize, Serialize};

/// genomeBin manifest with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenomeManifest {
    /// Primal name (e.g., "beardog")
    pub name: String,

    /// Version (e.g., "0.9.0")
    pub version: String,

    /// Human-readable description
    pub description: String,

    /// Supported architectures
    pub architectures: Vec<Arch>,

    /// NUCLEUS atomic type (if part of atomic)
    /// "TOWER", "NODE", "NEST", "NUCLEUS", or None for standalone
    pub nucleus_atomic: Option<String>,

    /// Primal capabilities (for discovery)
    pub capabilities: Vec<String>,

    /// Creation timestamp (ISO 8601)
    pub created_at: String,

    /// Optional: genomeBin format version
    #[serde(default = "default_format_version")]
    pub format_version: String,
}

fn default_format_version() -> String {
    "3.0.0".to_string()
}

impl GenomeManifest {
    /// Create new manifest
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: "0.1.0".to_string(),
            description: String::new(),
            architectures: Vec::new(),
            nucleus_atomic: None,
            capabilities: Vec::new(),
            created_at: chrono::Utc::now().to_rfc3339(),
            format_version: default_format_version(),
        }
    }

    /// Builder: Set version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Builder: Set description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Builder: Set NUCLEUS atomic type
    pub fn nucleus_atomic(mut self, atomic_type: impl Into<String>) -> Self {
        self.nucleus_atomic = Some(atomic_type.into());
        self
    }

    /// Builder: Add capability
    pub fn add_capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// Check if has specific capability
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }

    /// Check if supports architecture
    pub fn supports_arch(&self, arch: Arch) -> bool {
        self.architectures.contains(&arch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let manifest = GenomeManifest::new("test-primal");
        assert_eq!(manifest.name, "test-primal");
        assert_eq!(manifest.version, "0.1.0");
        assert_eq!(manifest.format_version, "3.0.0");
        assert!(manifest.architectures.is_empty());
        assert!(manifest.capabilities.is_empty());
        assert!(manifest.nucleus_atomic.is_none());
    }

    #[test]
    fn test_manifest_builder() {
        let manifest = GenomeManifest::new("beardog")
            .version("0.9.0")
            .description("BearDog Security Primal")
            .nucleus_atomic("TOWER-component")
            .add_capability("encryption")
            .add_capability("identity");

        assert_eq!(manifest.version, "0.9.0");
        assert_eq!(manifest.description, "BearDog Security Primal");
        assert_eq!(manifest.nucleus_atomic, Some("TOWER-component".to_string()));
        assert_eq!(manifest.capabilities.len(), 2);
        assert!(manifest.has_capability("encryption"));
        assert!(manifest.has_capability("identity"));
        assert!(!manifest.has_capability("compute"));
    }

    #[test]
    fn test_arch_support() {
        let mut manifest = GenomeManifest::new("test");
        assert!(!manifest.supports_arch(Arch::X86_64));

        manifest.architectures.push(Arch::X86_64);
        assert!(manifest.supports_arch(Arch::X86_64));
        assert!(!manifest.supports_arch(Arch::Aarch64));
    }
}
