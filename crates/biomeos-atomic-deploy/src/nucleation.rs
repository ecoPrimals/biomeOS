// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Socket nucleation: deterministic socket assignment for primals.
//!
//! Neural API assigns sockets for coordinated startup (no race conditions).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Socket Nucleation Strategy
#[derive(Debug, Clone, Default)]
pub enum SocketStrategy {
    /// /tmp/{primal}-{family_id}.sock
    FamilyDeterministic,

    /// XDG_RUNTIME_DIR/{primal}-{family_id}.sock (or /tmp/ fallback)
    #[default]
    XdgRuntime,
}

/// Socket Nucleation - Assigns sockets deterministically
pub struct SocketNucleation {
    strategy: SocketStrategy,
    assignments: HashMap<String, PathBuf>,
}

impl Default for SocketNucleation {
    fn default() -> Self {
        Self::new(SocketStrategy::default())
    }
}

impl SocketNucleation {
    /// Create new socket nucleation coordinator
    #[must_use]
    pub fn new(strategy: SocketStrategy) -> Self {
        Self {
            strategy,
            assignments: HashMap::new(),
        }
    }

    /// Assign socket path for a primal
    ///
    /// This is the "nucleation point" - provides deterministic socket
    /// assignment that prevents race conditions and enables coordinated startup
    pub fn assign_socket(&mut self, primal: &str, family_id: &str) -> PathBuf {
        let xdg_parent = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from);
        let runtime_dir = xdg_parent.as_deref();
        self.assign_socket_with_runtime_dir(primal, family_id, runtime_dir)
    }

    /// Like [`Self::assign_socket`], but supplies the XDG runtime parent directory explicitly
    /// (the directory that would be `$XDG_RUNTIME_DIR`), without reading environment variables.
    ///
    /// Pass `None` to use the same `/tmp/biomeos-$USER` fallback as when `XDG_RUNTIME_DIR` is unset.
    pub fn assign_socket_with_runtime_dir(
        &mut self,
        primal: &str,
        family_id: &str,
        runtime_dir: Option<&Path>,
    ) -> PathBuf {
        // Check if already assigned
        let key = format!("{primal}-{family_id}");
        if let Some(existing) = self.assignments.get(&key) {
            debug!("Socket already assigned for {}: {:?}", key, existing);
            return existing.clone();
        }

        // Assign new socket
        let socket = match self.strategy {
            SocketStrategy::FamilyDeterministic => {
                Self::family_deterministic_path(primal, family_id)
            }
            SocketStrategy::XdgRuntime => {
                Self::xdg_runtime_path_with(primal, family_id, runtime_dir)
            }
        };

        if let Some(parent) = socket.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                tracing::warn!("Failed to create socket dir {}: {e}", parent.display());
            }
        }

        info!("📍 Socket assigned: {} → {:?}", key, socket);
        self.assignments.insert(key, socket.clone());
        socket
    }

    /// Assign sockets for all primals in a list
    pub fn assign_batch(
        &mut self,
        primals: &[String],
        family_id: &str,
    ) -> HashMap<String, PathBuf> {
        let mut batch = HashMap::new();

        for primal in primals {
            let socket = self.assign_socket(primal, family_id);
            batch.insert(primal.clone(), socket);
        }

        batch
    }

    /// Get all assigned sockets
    #[must_use]
    pub const fn assignments(&self) -> &HashMap<String, PathBuf> {
        &self.assignments
    }

    /// Multi-family assignment for a shared primal
    ///
    /// When a primal can serve multiple families (e.g., Songbird, Toadstool),
    /// assign a socket for each family to the same logical primal.
    ///
    /// Returns the list of socket paths created for the primal.
    pub fn assign_multi_family(&mut self, primal: &str, family_ids: &[String]) -> Vec<PathBuf> {
        family_ids
            .iter()
            .map(|fid| self.assign_socket(primal, fid))
            .collect()
    }

    /// Check if a primal can be shared across families
    ///
    /// Primals that provide cryptographic key management MUST have separate
    /// instances per family (key material is family-specific).
    /// Other primals CAN share instances with family-suffixed sockets.
    ///
    /// **Capability-based**: Uses the taxonomy to determine if a primal holds
    /// family-specific state (e.g., key material) rather than hardcoding names.
    #[must_use]
    pub fn can_share(primal: &str) -> bool {
        use biomeos_types::capability_taxonomy::CapabilityTaxonomy;

        // Check if this primal provides security/key-management capabilities
        // via the taxonomy (reverse lookup: name → capability → category)
        let is_key_holder = CapabilityTaxonomy::resolve_to_primal("key_management")
            .is_some_and(|name| name == primal)
            || CapabilityTaxonomy::resolve_to_primal("encryption")
                .is_some_and(|name| name == primal);

        // Primals holding per-family cryptographic state cannot share
        !is_key_holder
    }

    /// Plan a multi-family deployment
    ///
    /// Returns (`dedicated_instances`, `shared_instances)`:
    /// - `dedicated_instances`: primals that need one instance per family
    /// - `shared_instances`: primals that can share across families
    pub fn plan_multi_family(
        &mut self,
        primals: &[String],
        family_ids: &[String],
    ) -> (HashMap<String, Vec<PathBuf>>, HashMap<String, Vec<PathBuf>>) {
        let mut dedicated = HashMap::new();
        let mut shared = HashMap::new();

        for primal in primals {
            if Self::can_share(primal) {
                // Shared: one instance, multiple family sockets
                let sockets = self.assign_multi_family(primal, family_ids);
                shared.insert(primal.clone(), sockets);
            } else {
                // Dedicated: separate instance per family
                let sockets: Vec<PathBuf> = family_ids
                    .iter()
                    .map(|fid| self.assign_socket(primal, fid))
                    .collect();
                dedicated.insert(primal.clone(), sockets);
            }
        }

        (dedicated, shared)
    }

    /// Family deterministic path using `SystemPaths`
    ///
    /// Uses XDG-compliant paths via `SystemPaths` instead of hardcoded `/tmp/`.
    /// Falls back to temp dir if XDG not available.
    fn family_deterministic_path(primal: &str, family_id: &str) -> PathBuf {
        use biomeos_types::paths::SystemPaths;

        // Use SystemPaths for XDG-compliant paths
        let paths = SystemPaths::new_lazy();
        paths.primal_socket(&format!("{primal}-{family_id}"))
    }

    /// XDG runtime path using explicit runtime parent or tmp fallback (no `XDG_RUNTIME_DIR` read).
    ///
    /// Creates sockets in `runtime_parent/biomeos/` for proper namespacing.
    fn xdg_runtime_path_with(primal: &str, family_id: &str, runtime_dir: Option<&Path>) -> PathBuf {
        use biomeos_types::paths::SystemPaths;

        let resolved = SystemPaths::runtime_dir_from_xdg_parent(runtime_dir);

        if let Err(e) = std::fs::create_dir_all(&resolved) {
            tracing::warn!("Failed to create runtime dir: {}", e);
        }

        resolved.join(format!("{}-{}.sock", primal, family_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_family_deterministic() {
        let mut nucleation = SocketNucleation::new(SocketStrategy::FamilyDeterministic);

        let socket = nucleation.assign_socket("beardog", "test_family");

        // Verify socket follows expected pattern (XDG-compliant)
        let socket_str = socket.to_string_lossy();
        assert!(
            socket_str.contains("beardog-test_family"),
            "Socket should contain primal-family: {socket_str}"
        );
        assert!(
            socket_str.ends_with(".sock"),
            "Socket should end with .sock: {socket_str}"
        );

        // Second assignment should return same socket (deterministic)
        let socket2 = nucleation.assign_socket("beardog", "test_family");
        assert_eq!(socket, socket2);
    }

    #[test]
    fn test_different_families() {
        let mut nucleation = SocketNucleation::new(SocketStrategy::FamilyDeterministic);

        let family_a_socket = nucleation.assign_socket("beardog", "test_family");
        let family_b_socket = nucleation.assign_socket("beardog", "nat1");

        // Different families should get different sockets
        assert_ne!(family_a_socket, family_b_socket);

        // Verify both follow expected patterns
        assert!(
            family_a_socket
                .to_string_lossy()
                .contains("beardog-test_family")
        );
        assert!(family_b_socket.to_string_lossy().contains("beardog-nat1"));
    }

    #[test]
    fn test_batch_assignment() {
        let mut nucleation = SocketNucleation::new(SocketStrategy::FamilyDeterministic);

        let primals = vec![
            "beardog".to_string(),
            "songbird".to_string(),
            "squirrel".to_string(),
        ];

        let assignments = nucleation.assign_batch(&primals, "test_family");

        assert_eq!(assignments.len(), 3);

        // Verify each primal got a unique, properly-named socket
        let beardog = assignments.get("beardog").unwrap();
        let songbird = assignments.get("songbird").unwrap();
        let squirrel = assignments.get("squirrel").unwrap();

        assert!(beardog.to_string_lossy().contains("beardog-test_family"));
        assert!(songbird.to_string_lossy().contains("songbird-test_family"));
        assert!(squirrel.to_string_lossy().contains("squirrel-test_family"));

        // All should be .sock files
        assert!(beardog.to_string_lossy().ends_with(".sock"));
        assert!(songbird.to_string_lossy().ends_with(".sock"));
        assert!(squirrel.to_string_lossy().ends_with(".sock"));
    }

    #[test]
    fn test_xdg_runtime_strategy() {
        let mut nucleation = SocketNucleation::new(SocketStrategy::XdgRuntime);
        let socket = nucleation.assign_socket_with_runtime_dir(
            "beardog",
            "test_family",
            Some(std::path::Path::new("/run/user/1000")),
        );

        assert!(socket.to_string_lossy().contains("/run/user/1000/biomeos/"));
        assert!(socket.to_string_lossy().contains("beardog-test_family"));
        assert!(socket.to_string_lossy().ends_with(".sock"));
    }

    #[test]
    fn test_xdg_runtime_fallback_to_tmp() {
        let mut nucleation = SocketNucleation::new(SocketStrategy::XdgRuntime);
        let socket = nucleation.assign_socket_with_runtime_dir("songbird", "test-family", None);

        assert!(
            socket.to_string_lossy().contains("/tmp/biomeos/")
                || socket.to_string_lossy().contains("songbird-test-family")
        );
    }
}

#[test]
fn test_default_strategy_is_xdg() {
    let nucleation = SocketNucleation::default();
    // Default should be XdgRuntime now
    assert!(matches!(nucleation.strategy, SocketStrategy::XdgRuntime));
}

#[test]
fn test_can_share() {
    // BearDog cannot share (crypto keys are per-family)
    assert!(!SocketNucleation::can_share("beardog"));
    // Others can share
    assert!(SocketNucleation::can_share("songbird"));
    assert!(SocketNucleation::can_share("nestgate"));
    assert!(SocketNucleation::can_share("toadstool"));
    assert!(SocketNucleation::can_share("squirrel"));
}

#[test]
fn test_multi_family_assignment() {
    let mut nucleation = SocketNucleation::new(SocketStrategy::FamilyDeterministic);
    let families = vec!["alpha".to_string(), "beta".to_string()];

    let sockets = nucleation.assign_multi_family("songbird", &families);
    assert_eq!(sockets.len(), 2);

    // Each should have the correct family suffix
    assert!(sockets[0].to_string_lossy().contains("songbird-alpha"));
    assert!(sockets[1].to_string_lossy().contains("songbird-beta"));
}

#[test]
fn test_plan_multi_family() {
    let mut nucleation = SocketNucleation::new(SocketStrategy::FamilyDeterministic);
    let primals = vec![
        "beardog".to_string(),
        "songbird".to_string(),
        "nestgate".to_string(),
    ];
    let families = vec!["alpha".to_string(), "beta".to_string()];

    let (dedicated, shared) = nucleation.plan_multi_family(&primals, &families);

    // BearDog should be dedicated (separate instance per family)
    assert!(dedicated.contains_key("beardog"));
    assert_eq!(dedicated["beardog"].len(), 2);

    // Songbird and NestGate should be shared
    assert!(shared.contains_key("songbird"));
    assert!(shared.contains_key("nestgate"));
    assert_eq!(shared["songbird"].len(), 2);
}
