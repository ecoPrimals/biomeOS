// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Capability taxonomy helper functions.
//!
//! These derive capability hints from the [`CapabilityTaxonomy`] rather than
//! maintaining a parallel hardcoded mapping. The taxonomy's `category()` and
//! `default_primal_with()` methods are the single source of truth.

use super::category::CapabilityCategory;
use super::definition::CapabilityTaxonomy;

/// Well-known capabilities indexed by category for reverse lookups.
const CATEGORY_HINTS: &[(CapabilityCategory, &[&str])] = &[
    (CapabilityCategory::Security, &["crypto", "security"]),
    (CapabilityCategory::Communication, &["discovery", "network"]),
    (CapabilityCategory::Compute, &["compute"]),
    (CapabilityCategory::Storage, &["storage"]),
    (CapabilityCategory::AI, &["ai"]),
    (CapabilityCategory::Orchestration, &["orchestration"]),
    (
        CapabilityCategory::UserInterface,
        &["rendering", "input", "ui"],
    ),
    (CapabilityCategory::Specialized, &["specialized", "domain"]),
];

/// Get capability category names for a primal based on the taxonomy.
///
/// This resolves through [`CapabilityTaxonomy::default_primal_with`] so that
/// the mapping stays consistent with the taxonomy definition. No primal names
/// are matched directly — we iterate taxonomy variants and collect categories
/// whose `default_primal` points at the given name.
///
/// **Bootstrap-time hint only.** In production, primals self-report capabilities
/// via `discover_capabilities` / `capability.list`.
#[must_use]
pub fn capabilities_for_primal(primal_name: &str) -> Vec<String> {
    let mut categories = Vec::new();

    for &(category, labels) in CATEGORY_HINTS {
        if category_maps_to_primal(category, primal_name) {
            categories.extend(labels.iter().map(|s| (*s).to_string()));
            break;
        }
    }

    if categories.is_empty() {
        categories.push(primal_name.to_string());
    }

    categories
}

/// Check whether any taxonomy variant in `category` has `primal_name` as its
/// default primal (non-strict mode).
fn category_maps_to_primal(category: CapabilityCategory, primal_name: &str) -> bool {
    CapabilityTaxonomy::representative_for_category(category)
        .and_then(|cap| cap.default_primal_with(false))
        .is_some_and(|default| default == primal_name)
}
