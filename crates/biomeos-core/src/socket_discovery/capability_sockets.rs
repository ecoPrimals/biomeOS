// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Capability-first socket naming.
//!
//! Absorbed from Squirrel alpha.13: primals may bind to capability-named
//! sockets (e.g. `security.sock`) instead of identity-named ones
//! (e.g. `beardog.sock`). The discovery engine tries capability sockets
//! first so callers can discover by domain rather than by primal identity.

use biomeos_types::primal_names;

/// Derive capability-first socket base names for a primal.
///
/// Returns the capability domain names that a given primal is known to
/// provide. The discovery engine prepends directory paths and appends
/// `.sock` when probing the filesystem.
pub(crate) fn names_for_primal(primal_name: &str) -> Vec<&'static str> {
    match primal_name {
        p if p.eq_ignore_ascii_case(primal_names::BEARDOG) => {
            vec!["security", "crypto"]
        }
        p if p.eq_ignore_ascii_case(primal_names::SONGBIRD) => {
            vec!["discovery", "relay", "mesh"]
        }
        p if p.eq_ignore_ascii_case(primal_names::TOADSTOOL) => {
            vec!["compute", "gpu"]
        }
        p if p.eq_ignore_ascii_case(primal_names::NESTGATE) => {
            vec!["storage", "provenance"]
        }
        p if p.eq_ignore_ascii_case(primal_names::SQUIRREL) => {
            vec!["ai", "mcp"]
        }
        p if p.eq_ignore_ascii_case(primal_names::LOAMSPINE) => {
            vec!["orchestration", "lifecycle"]
        }
        p if p.eq_ignore_ascii_case(primal_names::PETALTONGUE) => {
            vec!["ui", "visualization"]
        }
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_primals_return_capabilities() {
        assert!(!names_for_primal("beardog").is_empty());
        assert!(names_for_primal("beardog").contains(&"security"));
        assert!(names_for_primal("beardog").contains(&"crypto"));
    }

    #[test]
    fn unknown_primals_return_empty() {
        assert!(names_for_primal("unknown-primal").is_empty());
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(names_for_primal("BearDog"), names_for_primal("beardog"));
        assert_eq!(names_for_primal("SONGBIRD"), names_for_primal("songbird"));
    }
}
