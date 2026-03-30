// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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

/// `BearDog` — Tower atomic: cryptography and identity.
pub const BEARDOG: &str = "beardog";

/// Songbird — Tower atomic: service mesh, HTTP, discovery.
pub const SONGBIRD: &str = "songbird";

/// `ToadStool` — Node atomic: compute and GPU dispatch.
pub const TOADSTOOL: &str = "toadstool";

/// `NestGate` — Nest atomic: storage and persistence.
pub const NESTGATE: &str = "nestgate";

/// Squirrel — AI bridge and MCP platform.
pub const SQUIRREL: &str = "squirrel";

/// `LoamSpine` — Provenance: permanent content-addressed storage.
pub const LOAMSPINE: &str = "loamspine";

/// rhizoCrypt — Provenance: ephemeral DAG workspace.
pub const RHIZOCRYPT: &str = "rhizocrypt";

/// sweetGrass — Provenance: attribution and rewards.
pub const SWEETGRASS: &str = "sweetgrass";

// =========================================================================
// Spring primals — science/domain providers discovered at runtime.
// Springs register via `capability.register` with the Neural API.
// =========================================================================

/// airSpring — ecology, agriculture, ET0, water balance.
pub const AIRSPRING: &str = "airspring";

/// wetSpring — life science, microbial ecology, analytical chemistry.
pub const WETSPRING: &str = "wetspring";

/// neuralSpring — ML surrogates, learning, isomorphic patterns.
pub const NEURALSPRING: &str = "neuralspring";

/// groundSpring — measurement, signal vs noise, inverse problems.
pub const GROUNDSPRING: &str = "groundspring";

/// hotSpring — computational physics, nuclear EOS, lattice QCD.
pub const HOTSPRING: &str = "hotspring";

/// healthSpring — PK/PD, microbiome, biosignal, clinical diagnostics.
pub const HEALTHSPRING: &str = "healthspring";

/// ludoSpring — game science, HCI, procedural content, interaction design.
pub const LUDOSPRING: &str = "ludospring";

/// petalTongue — UI rendering, input handling, topology visualization.
pub const PETALTONGUE: &str = "petaltongue";

/// skunkBat — ephemeral compute, sandbox, experiments.
pub const SKUNKBAT: &str = "skunkbat";

/// sourDough — primal compliance, lifecycle standards, ecosystem starter.
pub const SOURDOUGH: &str = "sourdough";

/// biomeOS itself — appears in socket discovery as a pseudo-primal.
pub const BIOMEOS: &str = "biomeos";

/// biomeOS device management variant.
pub const BIOMEOS_DEVICE_MANAGEMENT: &str = "biomeos-device-management";

/// primalSpring — ecosystem integration experiments and IPC resilience.
pub const PRIMALSPRING: &str = "primalspring";

// =========================================================================
// Display names — mixed-case for UI/logs.
// Absorbed from neuralSpring `primal_names::display`.
// =========================================================================

/// Human-readable display names keyed by lowercase identifier.
pub mod display {
    /// `BearDog` display name.
    pub const BEARDOG: &str = "BearDog";
    /// `SongBird` display name.
    pub const SONGBIRD: &str = "SongBird";
    /// `ToadStool` display name.
    pub const TOADSTOOL: &str = "ToadStool";
    /// `NestGate` display name.
    pub const NESTGATE: &str = "NestGate";
    /// Squirrel display name.
    pub const SQUIRREL: &str = "Squirrel";
    /// `LoamSpine` display name.
    pub const LOAMSPINE: &str = "LoamSpine";
    /// rhizoCrypt display name.
    pub const RHIZOCRYPT: &str = "rhizoCrypt";
    /// sweetGrass display name.
    pub const SWEETGRASS: &str = "sweetGrass";
    /// airSpring display name.
    pub const AIRSPRING: &str = "airSpring";
    /// wetSpring display name.
    pub const WETSPRING: &str = "wetSpring";
    /// neuralSpring display name.
    pub const NEURALSPRING: &str = "neuralSpring";
    /// groundSpring display name.
    pub const GROUNDSPRING: &str = "groundSpring";
    /// hotSpring display name.
    pub const HOTSPRING: &str = "hotSpring";
    /// healthSpring display name.
    pub const HEALTHSPRING: &str = "healthSpring";
    /// ludoSpring display name.
    pub const LUDOSPRING: &str = "ludoSpring";
    /// biomeOS display name.
    pub const BIOMEOS: &str = "biomeOS";
    /// primalSpring display name.
    pub const PRIMALSPRING: &str = "primalSpring";
    /// petalTongue display name.
    pub const PETALTONGUE: &str = "petalTongue";
    /// skunkBat display name.
    pub const SKUNKBAT: &str = "skunkBat";
    /// sourDough display name.
    pub const SOURDOUGH: &str = "sourDough";

    /// Look up the display name for a lowercase primal identifier.
    #[must_use]
    pub fn for_id(id: &str) -> Option<&'static str> {
        match id {
            super::BEARDOG => Some(BEARDOG),
            super::SONGBIRD => Some(SONGBIRD),
            super::TOADSTOOL => Some(TOADSTOOL),
            super::NESTGATE => Some(NESTGATE),
            super::SQUIRREL => Some(SQUIRREL),
            super::LOAMSPINE => Some(LOAMSPINE),
            super::RHIZOCRYPT => Some(RHIZOCRYPT),
            super::SWEETGRASS => Some(SWEETGRASS),
            super::AIRSPRING => Some(AIRSPRING),
            super::WETSPRING => Some(WETSPRING),
            super::NEURALSPRING => Some(NEURALSPRING),
            super::GROUNDSPRING => Some(GROUNDSPRING),
            super::HOTSPRING => Some(HOTSPRING),
            super::HEALTHSPRING => Some(HEALTHSPRING),
            super::LUDOSPRING => Some(LUDOSPRING),
            super::BIOMEOS => Some(BIOMEOS),
            super::PRIMALSPRING => Some(PRIMALSPRING),
            super::PETALTONGUE => Some(PETALTONGUE),
            super::SKUNKBAT => Some(SKUNKBAT),
            super::SOURDOUGH => Some(SOURDOUGH),
            _ => None,
        }
    }
}

/// All bootstrap-time primal names (Tower + Node + Nest core).
pub const CORE_PRIMALS: &[&str] = &[BEARDOG, SONGBIRD, TOADSTOOL, NESTGATE, SQUIRREL];

/// Provenance trio primals.
pub const PROVENANCE_PRIMALS: &[&str] = &[LOAMSPINE, RHIZOCRYPT, SWEETGRASS];

/// Spring primals — science/domain providers.
pub const SPRING_PRIMALS: &[&str] = &[
    AIRSPRING,
    WETSPRING,
    NEURALSPRING,
    GROUNDSPRING,
    HOTSPRING,
    HEALTHSPRING,
    LUDOSPRING,
];

/// Additional primals — UI, sandbox, compliance.
pub const AUXILIARY_PRIMALS: &[&str] = &[PETALTONGUE, SKUNKBAT, SOURDOUGH, PRIMALSPRING];

// =========================================================================
// Niche self-knowledge — capabilities that biomeOS itself provides.
//
// Inspired by primalSpring's niche.rs pattern: each primal declares what it
// provides so self-registration is data-driven, not hardcoded inline.
// =========================================================================

/// Capabilities that biomeOS provides to the ecosystem.
///
/// Used by `register_self_in_registry` to register biomeOS in the Neural API
/// capability router. Adding a new capability here automatically propagates
/// to every bootstrap and self-registration callsite.
pub const BIOMEOS_SELF_CAPABILITIES: &[&str] = &[
    "primal.germination",
    "primal.terraria",
    "ecosystem.coordination",
    "ecosystem.nucleation",
    "graph.execution",
];

/// Check whether a string matches a known primal name (case-insensitive).
#[must_use]
pub fn is_known_primal(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    CORE_PRIMALS.contains(&lower.as_str())
        || PROVENANCE_PRIMALS.contains(&lower.as_str())
        || SPRING_PRIMALS.contains(&lower.as_str())
        || AUXILIARY_PRIMALS.contains(&lower.as_str())
        || lower == BIOMEOS
        || lower == BIOMEOS_DEVICE_MANAGEMENT
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
    fn is_known_primal_springs() {
        assert!(is_known_primal("airspring"));
        assert!(is_known_primal("wetspring"));
        assert!(is_known_primal("neuralspring"));
        assert!(is_known_primal("groundspring"));
        assert!(is_known_primal("hotspring"));
        assert!(is_known_primal("healthspring"));
        assert!(is_known_primal("ludospring"));
        assert!(is_known_primal("AirSpring"));
    }

    #[test]
    fn is_known_primal_unknown() {
        assert!(!is_known_primal(""));
        assert!(!is_known_primal("unknown"));
        assert!(!is_known_primal("barracuda"));
    }

    #[test]
    fn spring_primals_are_lowercase() {
        for name in SPRING_PRIMALS {
            assert_eq!(*name, name.to_ascii_lowercase(), "{name} must be lowercase");
        }
    }

    #[test]
    fn is_known_primal_biomeos() {
        assert!(is_known_primal("biomeos"));
        assert!(is_known_primal("BIOMEOS"));
    }

    #[test]
    fn core_primals_count() {
        assert_eq!(CORE_PRIMALS.len(), 5);
    }

    #[test]
    fn provenance_primals_count() {
        assert_eq!(PROVENANCE_PRIMALS.len(), 3);
    }

    #[test]
    fn spring_primals_count() {
        assert_eq!(SPRING_PRIMALS.len(), 7);
    }

    #[test]
    fn all_primal_names_are_unique() {
        let mut all: Vec<&str> = Vec::new();
        all.extend_from_slice(CORE_PRIMALS);
        all.extend_from_slice(PROVENANCE_PRIMALS);
        all.extend_from_slice(SPRING_PRIMALS);
        all.extend_from_slice(AUXILIARY_PRIMALS);
        all.push(BIOMEOS);
        all.push(BIOMEOS_DEVICE_MANAGEMENT);
        let unique: std::collections::HashSet<&&str> = all.iter().collect();
        assert_eq!(all.len(), unique.len(), "duplicate primal name found");
    }

    #[test]
    fn is_known_primal_auxiliary() {
        assert!(is_known_primal("primalspring"));
        assert!(is_known_primal("PrimalSpring"));
        assert!(is_known_primal("petaltongue"));
        assert!(is_known_primal("PetalTongue"));
        assert!(is_known_primal("skunkbat"));
        assert!(is_known_primal("sourdough"));
    }

    #[test]
    fn display_for_id_core() {
        assert_eq!(display::for_id("beardog"), Some("BearDog"));
        assert_eq!(display::for_id("songbird"), Some("SongBird"));
        assert_eq!(display::for_id("toadstool"), Some("ToadStool"));
        assert_eq!(display::for_id("nestgate"), Some("NestGate"));
        assert_eq!(display::for_id("squirrel"), Some("Squirrel"));
    }

    #[test]
    fn display_for_id_provenance() {
        assert_eq!(display::for_id("loamspine"), Some("LoamSpine"));
        assert_eq!(display::for_id("rhizocrypt"), Some("rhizoCrypt"));
        assert_eq!(display::for_id("sweetgrass"), Some("sweetGrass"));
    }

    #[test]
    fn display_for_id_springs() {
        assert_eq!(display::for_id("airspring"), Some("airSpring"));
        assert_eq!(display::for_id("neuralspring"), Some("neuralSpring"));
        assert_eq!(display::for_id("healthspring"), Some("healthSpring"));
    }

    #[test]
    fn display_for_id_unknown() {
        assert_eq!(display::for_id("unknown"), None);
        assert_eq!(display::for_id(""), None);
    }

    #[test]
    fn display_for_id_biomeos() {
        assert_eq!(display::for_id("biomeos"), Some("biomeOS"));
    }
}
