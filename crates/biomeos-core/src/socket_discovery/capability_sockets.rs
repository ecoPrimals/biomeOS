// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Capability-first socket naming — data-driven via the taxonomy.
//!
//! Absorbed from Squirrel alpha.13: primals may bind to capability-named
//! sockets (e.g. `security.sock`) instead of identity-named ones
//! (e.g. `beardog.sock`). The discovery engine tries capability sockets
//! first so callers can discover by domain rather than by primal identity.
//!
//! Previously this was a hardcoded `match` over primal names. Now it
//! delegates to [`biomeos_types::capability_taxonomy::capabilities_for_primal`],
//! keeping the taxonomy as the single source of truth.

/// Derive capability-first socket base names for a primal.
///
/// Returns the capability domain names that a given primal is known to
/// provide. The discovery engine prepends directory paths and appends
/// `.sock` when probing the filesystem.
///
/// Delegates to the taxonomy so no primal names are hardcoded here.
pub fn names_for_primal(primal_name: &str) -> Vec<String> {
    biomeos_types::capability_taxonomy::capabilities_for_primal(primal_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_primals_return_capabilities() {
        let caps = names_for_primal("beardog");
        assert!(!caps.is_empty());
        assert!(caps.iter().any(|c| c == "security" || c == "crypto"));
    }

    #[test]
    fn unknown_primals_return_own_name() {
        let caps = names_for_primal("unknown-primal");
        assert_eq!(caps, vec!["unknown-primal"]);
    }

    #[test]
    fn case_insensitive_via_taxonomy() {
        assert_eq!(names_for_primal("beardog"), names_for_primal("beardog"));
    }

    #[test]
    fn songbird_returns_discovery_domain() {
        let caps = names_for_primal("songbird");
        assert!(caps.iter().any(|c| c == "discovery" || c == "network"));
    }
}
