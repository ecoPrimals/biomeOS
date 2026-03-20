// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Structured provenance metadata.
//!
//! Absorbed from primalSpring v0.3.0. Tracks where patterns, types, and
//! implementations were absorbed from, enabling traceability across the
//! ecoPrimals ecosystem.
//!
//! Every absorbed pattern should carry a `Provenance` so engineers can
//! trace it back to its origin spring/primal and understand the context
//! in which it was designed.

use serde::{Deserialize, Serialize};

/// Provenance metadata for an absorbed pattern or type.
///
/// Records where a piece of code or a design pattern originated,
/// when it was baselined, and why it was absorbed.
///
/// # Example
///
/// ```
/// use biomeos_types::Provenance;
///
/// let p = Provenance::new("airSpring", "2026-03-15")
///     .with_description("Type-safe numeric cast helpers")
///     .with_version("0.10.0");
///
/// assert_eq!(p.source, "airSpring");
/// assert_eq!(p.version.as_deref(), Some("0.10.0"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Provenance {
    /// Origin spring or primal identifier (e.g. "airSpring", "primalSpring").
    pub source: String,
    /// ISO-8601 date when this pattern was baselined into biomeOS.
    pub baseline_date: String,
    /// Human-readable description of what was absorbed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Version of the source when the pattern was absorbed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl Provenance {
    /// Create a new provenance record.
    #[must_use]
    pub fn new(source: impl Into<String>, baseline_date: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            baseline_date: baseline_date.into(),
            description: None,
            version: None,
        }
    }

    /// Set the description.
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the source version.
    #[must_use]
    pub fn with_version(mut self, ver: impl Into<String>) -> Self {
        self.version = Some(ver.into());
        self
    }
}

impl std::fmt::Display for Provenance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (baselined {})", self.source, self.baseline_date)?;
        if let Some(ref ver) = self.version {
            write!(f, " v{ver}")?;
        }
        if let Some(ref desc) = self.description {
            write!(f, ": {desc}")?;
        }
        Ok(())
    }
}

/// Collection of provenance records for a module or crate.
///
/// Useful for `biomeos validate` to report all absorbed patterns
/// and their origins.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProvenanceManifest {
    /// Module or crate these records belong to.
    pub module: String,
    /// Individual provenance records.
    pub records: Vec<Provenance>,
}

impl ProvenanceManifest {
    /// Create a new manifest for a module.
    #[must_use]
    pub fn new(module: impl Into<String>) -> Self {
        Self {
            module: module.into(),
            records: Vec::new(),
        }
    }

    /// Add a provenance record.
    pub fn add(&mut self, record: Provenance) -> &mut Self {
        self.records.push(record);
        self
    }

    /// Number of recorded absorptions.
    #[must_use]
    pub fn count(&self) -> usize {
        self.records.len()
    }

    /// All unique source springs/primals.
    #[must_use]
    pub fn sources(&self) -> Vec<&str> {
        let mut srcs: Vec<&str> = self.records.iter().map(|r| r.source.as_str()).collect();
        srcs.sort_unstable();
        srcs.dedup();
        srcs
    }
}

#[allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provenance_builder() {
        let p = Provenance::new("airSpring", "2026-03-15")
            .with_description("cast helpers")
            .with_version("0.10.0");
        assert_eq!(p.source, "airSpring");
        assert_eq!(p.baseline_date, "2026-03-15");
        assert_eq!(p.description.as_deref(), Some("cast helpers"));
        assert_eq!(p.version.as_deref(), Some("0.10.0"));
    }

    #[test]
    fn provenance_display() {
        let p = Provenance::new("primalSpring", "2026-03-18")
            .with_version("0.3.0")
            .with_description("structured provenance");
        let s = p.to_string();
        assert!(s.contains("primalSpring"));
        assert!(s.contains("2026-03-18"));
        assert!(s.contains("v0.3.0"));
        assert!(s.contains("structured provenance"));
    }

    #[test]
    fn provenance_serde_roundtrip() {
        let p = Provenance::new("wetSpring", "2026-02-01").with_version("0.5.0");
        let json = serde_json::to_string(&p).expect("serialize");
        let parsed: Provenance = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(p, parsed);
    }

    #[test]
    fn manifest_sources() {
        let mut m = ProvenanceManifest::new("biomeos-types");
        m.add(Provenance::new("airSpring", "2026-03-15"));
        m.add(Provenance::new("wetSpring", "2026-03-15"));
        m.add(Provenance::new("airSpring", "2026-03-16"));
        assert_eq!(m.count(), 3);
        assert_eq!(m.sources(), vec!["airSpring", "wetSpring"]);
    }

    #[test]
    fn provenance_without_optional_fields_serializes_clean() {
        let p = Provenance::new("groundSpring", "2026-01-01");
        let json = serde_json::to_string(&p).expect("serialize");
        assert!(!json.contains("description"));
        assert!(!json.contains("version"));
    }
}
