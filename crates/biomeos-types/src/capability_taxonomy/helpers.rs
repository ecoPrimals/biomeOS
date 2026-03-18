// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Capability taxonomy helper functions

/// Get capability category names for a primal based on the taxonomy
///
/// This is the reverse of `CapabilityTaxonomy::default_primal()`.
/// Returns the high-level capability categories that a primal provides,
/// derived from the taxonomy rather than hardcoded per-callsite.
///
/// **DEEP DEBT NOTE**: This is a bootstrap-time hint. In production,
/// primals should self-report capabilities via `discover_capabilities`.
#[must_use]
pub fn capabilities_for_primal(primal_name: &str) -> Vec<String> {
    match primal_name {
        "beardog" => vec!["crypto".to_string(), "security".to_string()],
        "songbird" => vec!["discovery".to_string(), "network".to_string()],
        "toadstool" => vec!["compute".to_string()],
        "nestgate" => vec!["storage".to_string()],
        "squirrel" => vec!["ai".to_string()],
        "biomeos" => vec!["orchestration".to_string()],
        other => vec![other.to_string()],
    }
}
