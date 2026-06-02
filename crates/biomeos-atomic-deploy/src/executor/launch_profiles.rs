// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Data-driven primal launch profiles
//!
//! Loaded from `config/primal_launch_profiles.toml`. Primals not listed in the
//! config inherit the `[default]` profile. New primals can be onboarded by
//! adding a TOML entry — no code changes needed.

use std::collections::HashMap;

use tokio::process::Command;
use tracing::{info, warn};

use super::context::ExecutionContext;

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct LaunchProfile {
    socket_flag: Option<String>,
    pass_family_id: Option<bool>,
    env_socket: Option<String>,
    /// CLI flag for TCP listen address (TCP-only mode). When set, the primal
    /// receives `--<flag> host:port` instead of the UDS socket path.
    tcp_listen_flag: Option<String>,
    #[serde(default)]
    extra_env: HashMap<String, String>,
    #[serde(default)]
    env_sockets: HashMap<String, String>,
    #[serde(default)]
    cli_sockets: HashMap<String, String>,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct LaunchProfilesConfig {
    default: LaunchProfile,
    #[serde(default)]
    profiles: HashMap<String, LaunchProfile>,
}

static LAUNCH_PROFILES_TOML: &str = include_str!("../../../../config/primal_launch_profiles.toml");

pub(crate) fn load_launch_profiles() -> LaunchProfilesConfig {
    toml::from_str(LAUNCH_PROFILES_TOML).unwrap_or_else(|e| {
        warn!("Failed to parse primal launch profiles: {}", e);
        LaunchProfilesConfig {
            default: LaunchProfile {
                socket_flag: Some("--socket".to_string()),
                pass_family_id: Some(true),
                env_socket: Some("PRIMAL_SOCKET".to_string()),
                tcp_listen_flag: None,
                extra_env: HashMap::new(),
                env_sockets: HashMap::new(),
                cli_sockets: HashMap::new(),
            },
            profiles: HashMap::new(),
        }
    })
}

/// Configure primal-specific socket paths and arguments
///
/// Uses data-driven launch profiles from `config/primal_launch_profiles.toml`.
/// Primals not listed in the config inherit the `[default]` profile.
/// New primals can be onboarded by adding a TOML entry — no code changes needed.
///
/// Pub(crate) for reuse by `capability_handlers::primal_start` (capability-based, no hardcoded names).
pub(crate) async fn configure_primal_sockets(
    cmd: &mut Command,
    primal_name: &str,
    socket_path: &str,
    family_id: &str,
    context: &ExecutionContext,
) {
    let config = load_launch_profiles();
    let profile = config.profiles.get(primal_name);
    let defaults = &config.default;

    let socket_flag = profile
        .and_then(|p| p.socket_flag.as_deref())
        .or(defaults.socket_flag.as_deref())
        .unwrap_or("--socket");

    let pass_family_id = profile
        .and_then(|p| p.pass_family_id)
        .or(defaults.pass_family_id)
        .unwrap_or(true);

    let env_socket = profile
        .and_then(|p| p.env_socket.as_deref())
        .or(defaults.env_socket.as_deref());

    // Primary socket CLI flag — in TCP-only mode, use the profile's
    // tcp_listen_flag if the primal declares one.
    let tcp_listen_flag = profile.and_then(|p| p.tcp_listen_flag.as_deref());
    if context.tcp_only {
        if let Some(listen_flag) = tcp_listen_flag {
            if let Some(port) = context.get_tcp_port(primal_name).await {
                let host = biomeos_types::constants::DEFAULT_LOCALHOST;
                cmd.arg(listen_flag).arg(format!("{host}:{port}"));
                info!("   TCP-only: {} {} {host}:{port}", primal_name, listen_flag);
            }
        } else {
            cmd.arg(socket_flag).arg(socket_path);
        }
    } else {
        cmd.arg(socket_flag).arg(socket_path);
    }

    if pass_family_id {
        cmd.arg("--family-id").arg(family_id);
    }

    // Env var fallback for socket path (only for unknown primals without a profile)
    if profile.is_none() {
        if let Some(env_name) = env_socket {
            cmd.env(env_name, socket_path);
        }
        warn!("   No launch profile for '{}', using defaults", primal_name);
    }

    // Static extra env vars from the profile
    if let Some(p) = profile {
        for (key, value) in &p.extra_env {
            cmd.env(key, value);
        }

        // Env vars whose values are resolved socket paths of other primals.
        // In TCP-only mode, resolve to TCP addresses instead of UDS paths.
        for (env_name, socket_ref) in &p.env_sockets {
            if socket_ref == "$family_id" {
                cmd.env(env_name, family_id);
            } else if context.tcp_only {
                if let Some(port) = context.get_tcp_port(socket_ref).await {
                    let host = biomeos_types::constants::DEFAULT_LOCALHOST;
                    let tcp_addr = format!("tcp://{host}:{port}");
                    cmd.env(env_name, &tcp_addr);
                    info!("   TCP-only env: {env_name}={tcp_addr}");
                } else {
                    let resolved = context.get_socket_path(socket_ref).await;
                    cmd.env(env_name, &resolved);
                }
            } else {
                let resolved = context.get_socket_path(socket_ref).await;
                cmd.env(env_name, &resolved);
            }
        }

        // Extra CLI flags whose values are resolved socket paths.
        // In TCP-only mode, resolve to TCP addresses.
        for (flag, socket_ref) in &p.cli_sockets {
            if context.tcp_only {
                if let Some(port) = context.get_tcp_port(socket_ref).await {
                    let host = biomeos_types::constants::DEFAULT_LOCALHOST;
                    let tcp_addr = format!("tcp://{host}:{port}");
                    cmd.arg(flag).arg(&tcp_addr);
                    info!(
                        "   TCP-only: {} → {} @ {}",
                        primal_name, socket_ref, tcp_addr
                    );
                } else {
                    let resolved = context.get_socket_path(socket_ref).await;
                    cmd.arg(flag).arg(&resolved);
                    info!("   Bonding {} → {}: {}", primal_name, socket_ref, resolved);
                }
            } else {
                let resolved = context.get_socket_path(socket_ref).await;
                cmd.arg(flag).arg(&resolved);
                info!("   Bonding {} → {}: {}", primal_name, socket_ref, resolved);
            }
        }
    }
}
