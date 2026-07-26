// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unit tests for capability translation registry.
//!
//! Extracted from capability_translation module to keep main module under 1000 LOC.

pub(super) fn find_capability_registry_config() -> Option<std::path::PathBuf> {
    let mut dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        let candidate = dir.join("config/capability_registry.toml");
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

mod config_loading;
mod registry;
mod socket_and_call;
