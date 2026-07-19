// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::capability_registry::CapabilityRegistry;

/// All socket tests use `CapabilityRegistry::with_socket_path` to inject an explicit
/// temp-dir socket, avoiding env-var races with `XDG_RUNTIME_DIR`.
pub(super) fn make_registry(
    name: &str,
) -> (tempfile::TempDir, std::path::PathBuf, CapabilityRegistry) {
    let temp = tempfile::tempdir().expect("temp dir");
    let runtime_dir = temp.path().join("biomeos");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime dir");
    let socket_path = runtime_dir.join(format!("biomeos-registry-{name}.sock"));
    let registry = CapabilityRegistry::with_socket_path(name.to_string(), socket_path.clone());
    (temp, socket_path, registry)
}
