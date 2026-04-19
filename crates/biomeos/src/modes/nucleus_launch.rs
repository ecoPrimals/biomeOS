// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Nucleus-mode launch profile types and loader.
//!
//! Loaded from `config/nucleus_launch_profiles.toml` at compile time via
//! `include_str!`. Each primal can declare its subcommand, flags, env vars,
//! capability-resolved sockets, JWT generation, and AI passthrough without
//! requiring code changes in `nucleus.rs`.

use std::collections::HashMap;
use tracing::warn;

/// Per-primal launch profile for nucleus mode.
#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct NucleusLaunchProfile {
    pub subcommand: Option<String>,
    pub pass_socket_flag: Option<bool>,
    pub pass_family_id_flag: Option<bool>,
    pub pass_ai_model: Option<bool>,
    pub pass_ai_providers: Option<bool>,
    pub generate_jwt_secret: Option<bool>,
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
    #[serde(default)]
    pub capability_sockets: HashMap<String, String>,
}

/// Top-level config: a `[default]` profile and per-primal `[profiles.*]`.
#[derive(Debug, serde::Deserialize)]
pub(super) struct NucleusLaunchConfig {
    pub default: NucleusLaunchProfile,
    #[serde(default)]
    pub profiles: HashMap<String, NucleusLaunchProfile>,
}

static NUCLEUS_PROFILES_TOML: &str =
    include_str!("../../../../config/nucleus_launch_profiles.toml");

/// Parse the embedded TOML, falling back to a safe default on error.
pub(super) fn load_nucleus_profiles() -> NucleusLaunchConfig {
    toml::from_str(NUCLEUS_PROFILES_TOML).unwrap_or_else(|e| {
        warn!("Failed to parse nucleus launch profiles: {}", e);
        NucleusLaunchConfig {
            default: NucleusLaunchProfile {
                subcommand: Some("server".to_string()),
                pass_socket_flag: Some(true),
                pass_family_id_flag: Some(false),
                pass_ai_model: Some(false),
                pass_ai_providers: Some(false),
                generate_jwt_secret: Some(false),
                env_vars: HashMap::new(),
                capability_sockets: HashMap::new(),
            },
            profiles: HashMap::new(),
        }
    })
}
