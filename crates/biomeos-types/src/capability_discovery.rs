// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Capability-first socket discovery (5-tier standard).
//!
//! Implements the primalSpring 5-tier discovery protocol so any crate can
//! resolve a capability domain to a Unix socket path without importing
//! identity-specific helpers or hardcoding primal names.
//!
//! ## Tier Precedence
//!
//! 1. `{CAPABILITY}_PROVIDER_SOCKET` — explicit env override (e.g. `SECURITY_PROVIDER_SOCKET`)
//! 2. `{PRIMAL}_SOCKET` — identity env fallback via taxonomy resolution
//! 3. `$XDG_RUNTIME_DIR/biomeos/{capability}.sock` or `{primal}.sock`
//! 4. `/tmp/biomeos/{capability}.sock` or `{primal}.sock`
//! 5. Manifest / socket-registry (file-based fallback)

use std::path::{Path, PathBuf};

use crate::CapabilityTaxonomy;

/// Resolve a capability domain to a Unix socket path using the 5-tier
/// discovery protocol.
///
/// `env` is an injectable environment lookup (use `std_env` for production,
/// or a closure over a `HashMap` for tests).
///
/// Returns `None` if no socket can be found at any tier.
#[must_use]
pub fn discover_capability_socket(
    capability: &str,
    env: &dyn Fn(&str) -> Option<String>,
) -> Option<String> {
    // Tier 1: explicit capability env var (e.g. SECURITY_PROVIDER_SOCKET)
    let cap_env_key = format!(
        "{}_PROVIDER_SOCKET",
        capability.to_uppercase().replace('.', "_")
    );
    if let Some(socket) = env(&cap_env_key) {
        return Some(socket);
    }

    // Resolve primal name from taxonomy for identity-based tiers
    let primal = CapabilityTaxonomy::resolve_to_primal(capability);

    // Tier 2: primal-specific env var (e.g. BEARDOG_SOCKET)
    if let Some(p) = primal {
        let primal_env_key = format!("{}_SOCKET", p.to_uppercase());
        if let Some(socket) = env(&primal_env_key) {
            return Some(socket);
        }
    }

    // Determine runtime directory for filesystem probes
    let runtime_dir = env("XDG_RUNTIME_DIR").map(PathBuf::from);
    let family_id = env("BIOMEOS_FAMILY_ID")
        .or_else(|| env("FAMILY_ID"))
        .unwrap_or_default();

    // Tier 3: XDG runtime directory
    if let Some(ref base) = runtime_dir {
        let biomeos_dir = base.join("biomeos");
        if let Some(found) = probe_socket_dir(&biomeos_dir, capability, primal, &family_id) {
            return Some(found);
        }
    }

    // Tier 4: /tmp fallback (PRIMAL_IPC_PROTOCOL.md standard tier)
    let tmp_dir = crate::constants::runtime_paths::fallback_runtime_dir(&family_id);
    if let Some(found) = probe_socket_dir(&tmp_dir, capability, primal, &family_id) {
        return Some(found);
    }

    // Tier 5: socket-registry.json
    if let Some(ref base) = runtime_dir {
        let registry_path = base.join("biomeos/socket-registry.json");
        if let Some(found) = probe_socket_registry(&registry_path, capability) {
            return Some(found);
        }
    }

    None
}

/// Probe a directory for capability or primal-named sockets.
fn probe_socket_dir(
    dir: &Path,
    capability: &str,
    primal: Option<&str>,
    family_id: &str,
) -> Option<String> {
    let candidates = build_socket_candidates(dir, capability, primal, family_id);
    candidates.into_iter().find(|p| Path::new(p).exists())
}

/// Build candidate socket paths in priority order.
fn build_socket_candidates(
    dir: &Path,
    capability: &str,
    primal: Option<&str>,
    family_id: &str,
) -> Vec<String> {
    let mut candidates = Vec::with_capacity(6);

    // Capability-named sockets first (primalSpring standard)
    candidates.push(
        dir.join(format!("{capability}.sock"))
            .to_string_lossy()
            .to_string(),
    );
    if !family_id.is_empty() {
        candidates.push(
            dir.join(format!("{capability}-{family_id}.sock"))
                .to_string_lossy()
                .to_string(),
        );
    }

    // Identity-named sockets as fallback
    if let Some(p) = primal {
        candidates.push(dir.join(format!("{p}.sock")).to_string_lossy().to_string());
        if !family_id.is_empty() {
            candidates.push(
                dir.join(format!("{p}-{family_id}.sock"))
                    .to_string_lossy()
                    .to_string(),
            );
        }
    }

    candidates
}

/// Probe a socket-registry.json for a capability's socket path.
fn probe_socket_registry(registry_path: &Path, capability: &str) -> Option<String> {
    let content = std::fs::read_to_string(registry_path).ok()?;
    let registry: serde_json::Value = serde_json::from_str(&content).ok()?;
    registry
        .get(capability)
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Standard environment lookup that delegates to `std::env::var`.
#[must_use]
pub fn std_env(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn mock_env(vars: &HashMap<String, String>) -> impl Fn(&str) -> Option<String> + '_ {
        move |key: &str| vars.get(key).cloned()
    }

    #[test]
    fn tier1_capability_env_var() {
        let mut env = HashMap::new();
        env.insert(
            "SECURITY_PROVIDER_SOCKET".to_string(),
            "/run/custom/security.sock".to_string(),
        );
        let result = discover_capability_socket("security", &mock_env(&env));
        assert_eq!(result, Some("/run/custom/security.sock".to_string()));
    }

    #[test]
    fn tier2_primal_env_var() {
        let mut env = HashMap::new();
        env.insert(
            "BEARDOG_SOCKET".to_string(),
            "/run/beardog.sock".to_string(),
        );
        let result = discover_capability_socket("security", &mock_env(&env));
        assert_eq!(result, Some("/run/beardog.sock".to_string()));
    }

    #[test]
    fn tier3_xdg_capability_socket() {
        let dir = tempfile::tempdir().unwrap();
        let biomeos_dir = dir.path().join("biomeos");
        std::fs::create_dir_all(&biomeos_dir).unwrap();
        std::fs::write(biomeos_dir.join("security.sock"), "").unwrap();

        let mut env = HashMap::new();
        env.insert(
            "XDG_RUNTIME_DIR".to_string(),
            dir.path().to_string_lossy().to_string(),
        );
        let result = discover_capability_socket("security", &mock_env(&env));
        assert!(result.is_some());
        assert!(result.unwrap().contains("security.sock"));
    }

    #[test]
    fn tier3_xdg_primal_socket_fallback() {
        let dir = tempfile::tempdir().unwrap();
        let biomeos_dir = dir.path().join("biomeos");
        std::fs::create_dir_all(&biomeos_dir).unwrap();
        std::fs::write(biomeos_dir.join("beardog.sock"), "").unwrap();

        let mut env = HashMap::new();
        env.insert(
            "XDG_RUNTIME_DIR".to_string(),
            dir.path().to_string_lossy().to_string(),
        );
        let result = discover_capability_socket("security", &mock_env(&env));
        assert!(result.is_some());
        assert!(result.unwrap().contains("beardog.sock"));
    }

    #[test]
    fn tier3_xdg_family_suffixed_socket() {
        let dir = tempfile::tempdir().unwrap();
        let biomeos_dir = dir.path().join("biomeos");
        std::fs::create_dir_all(&biomeos_dir).unwrap();
        std::fs::write(biomeos_dir.join("security-nat0.sock"), "").unwrap();

        let mut env = HashMap::new();
        env.insert(
            "XDG_RUNTIME_DIR".to_string(),
            dir.path().to_string_lossy().to_string(),
        );
        env.insert("FAMILY_ID".to_string(), "nat0".to_string());
        let result = discover_capability_socket("security", &mock_env(&env));
        assert!(result.is_some());
        assert!(result.unwrap().contains("security-nat0.sock"));
    }

    #[test]
    fn returns_none_when_nothing_found() {
        let env: HashMap<String, String> = HashMap::new();
        let result = discover_capability_socket("nonexistent", &mock_env(&env));
        assert!(result.is_none());
    }

    #[test]
    fn genetic_resolves_to_beardog() {
        assert_eq!(
            CapabilityTaxonomy::resolve_to_primal("genetic"),
            Some(crate::primal_names::BEARDOG)
        );
    }

    #[test]
    fn lineage_resolves_to_beardog() {
        assert_eq!(
            CapabilityTaxonomy::resolve_to_primal("lineage"),
            Some(crate::primal_names::BEARDOG)
        );
    }

    #[test]
    fn security_resolves_to_beardog_via_env() {
        let mut env = HashMap::new();
        env.insert(
            "BEARDOG_SOCKET".to_string(),
            "/tmp/test-beardog.sock".to_string(),
        );
        let result = discover_capability_socket("encryption", &mock_env(&env));
        assert_eq!(result, Some("/tmp/test-beardog.sock".to_string()));
    }

    #[test]
    fn discovery_resolves_to_songbird_via_env() {
        let mut env = HashMap::new();
        env.insert(
            "SONGBIRD_SOCKET".to_string(),
            "/tmp/test-songbird.sock".to_string(),
        );
        let result = discover_capability_socket("discovery", &mock_env(&env));
        assert_eq!(result, Some("/tmp/test-songbird.sock".to_string()));
    }
}
