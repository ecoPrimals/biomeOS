// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Neural API socket resolution - extracted from engine.rs to keep files under 1000 lines.

use std::env;
use std::path::{Path, PathBuf};

/// Resolve Neural API socket path for registry queries.
///
/// Checks (in order): explicit override, instance socket, env var, standard locations.
pub fn resolve_neural_api_socket(
    family_id: &str,
    instance_socket: Option<&PathBuf>,
    env_override: Option<&Path>,
) -> Option<PathBuf> {
    if let Some(socket) = instance_socket {
        if socket.exists() {
            return Some(socket.clone());
        }
    }

    let env_path = env_override
        .map(PathBuf::from)
        .or_else(|| env::var("NEURAL_API_SOCKET").ok().map(PathBuf::from));
    if let Some(path) = env_path {
        if path.exists() {
            return Some(path);
        }
    }

    let temp_dir = std::env::temp_dir();
    let standard_locations = vec![
        temp_dir.join(format!("neural-api-{}.sock", family_id)),
        temp_dir.join("neural-api.sock"),
    ];
    standard_locations.into_iter().find(|path| path.exists())
}
