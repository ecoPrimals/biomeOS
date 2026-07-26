// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test")]

use super::*;

mod metadata;
mod parsing;
mod primal_resolution;

pub(super) fn resolve(cap: &str) -> Option<&'static str> {
    CapabilityTaxonomy::from_str_flexible(cap).and_then(|c| c.default_primal_with(false))
}
