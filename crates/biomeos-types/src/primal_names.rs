// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Canonical primal name constants.
//!
//! These constants are the **single source of truth** for primal identifiers used
//! during bootstrap, test fixtures, and capability-to-provider fallback resolution.
//!
//! **In production**, primals are discovered at runtime via Songbird capability
//! discovery and `CapabilityTaxonomy::resolve_to_primal()`. These constants are
//! only used as bootstrap hints before discovery is available.
//!
//! Set `BIOMEOS_STRICT_DISCOVERY=1` to disable all bootstrap-time name usage.

/// BearDog — Tower atomic: cryptography and identity.
pub const BEARDOG: &str = "beardog";

/// Songbird — Tower atomic: service mesh, HTTP, discovery.
pub const SONGBIRD: &str = "songbird";

/// ToadStool — Node atomic: compute and GPU dispatch.
pub const TOADSTOOL: &str = "toadstool";

/// NestGate — Nest atomic: storage and persistence.
pub const NESTGATE: &str = "nestgate";

/// Squirrel — AI bridge and MCP platform.
pub const SQUIRREL: &str = "squirrel";

/// LoamSpine — Provenance: permanent content-addressed storage.
pub const LOAMSPINE: &str = "loamspine";

/// rhizoCrypt — Provenance: ephemeral DAG workspace.
pub const RHIZOCRYPT: &str = "rhizocrypt";

/// sweetGrass — Provenance: attribution and rewards.
pub const SWEETGRASS: &str = "sweetgrass";

/// All bootstrap-time primal names (Tower + Node + Nest core).
pub const CORE_PRIMALS: &[&str] = &[BEARDOG, SONGBIRD, TOADSTOOL, NESTGATE, SQUIRREL];

/// Provenance trio primals.
pub const PROVENANCE_PRIMALS: &[&str] = &[LOAMSPINE, RHIZOCRYPT, SWEETGRASS];

/// Check whether a string matches a known primal name (case-insensitive).
#[must_use]
pub fn is_known_primal(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    CORE_PRIMALS.contains(&lower.as_str()) || PROVENANCE_PRIMALS.contains(&lower.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_are_lowercase() {
        for name in CORE_PRIMALS.iter().chain(PROVENANCE_PRIMALS.iter()) {
            assert_eq!(*name, name.to_ascii_lowercase(), "{name} must be lowercase");
        }
    }

    #[test]
    fn is_known_primal_core() {
        assert!(is_known_primal("beardog"));
        assert!(is_known_primal("songbird"));
        assert!(is_known_primal("toadstool"));
        assert!(is_known_primal("nestgate"));
        assert!(is_known_primal("squirrel"));
    }

    #[test]
    fn is_known_primal_provenance() {
        assert!(is_known_primal("loamspine"));
        assert!(is_known_primal("rhizocrypt"));
        assert!(is_known_primal("sweetgrass"));
    }

    #[test]
    fn is_known_primal_case_insensitive() {
        assert!(is_known_primal("BearDog"));
        assert!(is_known_primal("SONGBIRD"));
        assert!(is_known_primal("LoamSpine"));
    }

    #[test]
    fn is_known_primal_unknown() {
        assert!(!is_known_primal(""));
        assert!(!is_known_primal("unknown"));
        assert!(!is_known_primal("barracuda"));
    }
}
