// Socket Nucleation: Deterministic Socket Assignment
//
// Neural API as nucleation point - assigns sockets to primals
// for coordinated, aligned startup (no race conditions)

use std::collections::HashMap;
use std::path::PathBuf;
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
        // Check if already assigned
        let key = format!("{}-{}", primal, family_id);
        if let Some(existing) = self.assignments.get(&key) {
            debug!("Socket already assigned for {}: {:?}", key, existing);
            return existing.clone();
        }

        // Assign new socket
        let socket = match self.strategy {
            SocketStrategy::FamilyDeterministic => {
                Self::family_deterministic_path(primal, family_id)
            }
            SocketStrategy::XdgRuntime => Self::xdg_runtime_path(primal, family_id),
        };

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
    pub fn assignments(&self) -> &HashMap<String, PathBuf> {
        &self.assignments
    }

    /// Family deterministic path using SystemPaths
    ///
    /// Uses XDG-compliant paths via `SystemPaths` instead of hardcoded `/tmp/`.
    /// Falls back to temp dir if XDG not available.
    fn family_deterministic_path(primal: &str, family_id: &str) -> PathBuf {
        use biomeos_types::paths::SystemPaths;

        // Use SystemPaths for XDG-compliant paths
        let paths = SystemPaths::new_lazy();
        paths.primal_socket(&format!("{}-{}", primal, family_id))
    }

    /// XDG runtime path using SystemPaths
    ///
    /// Creates sockets in XDG_RUNTIME_DIR/biomeos/ for proper namespacing
    fn xdg_runtime_path(primal: &str, family_id: &str) -> PathBuf {
        use biomeos_types::paths::SystemPaths;

        // Use SystemPaths for XDG-compliant paths with automatic directory creation
        let paths = SystemPaths::new_lazy();

        // Ensure runtime directory exists
        if let Err(e) = std::fs::create_dir_all(paths.runtime_dir()) {
            tracing::warn!("Failed to create runtime dir: {}", e);
        }

        paths.primal_socket(&format!("{}-{}", primal, family_id))
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
            "Socket should contain primal-family: {}",
            socket_str
        );
        assert!(
            socket_str.ends_with(".sock"),
            "Socket should end with .sock: {}",
            socket_str
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
        assert!(family_a_socket.to_string_lossy().contains("beardog-test_family"));
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
}

#[test]
fn test_xdg_runtime_strategy() {
    // Set up XDG_RUNTIME_DIR for this test
    std::env::set_var("XDG_RUNTIME_DIR", "/run/user/1000");

    let mut nucleation = SocketNucleation::new(SocketStrategy::XdgRuntime);
    let socket = nucleation.assign_socket("beardog", "test_family");

    // Should create XDG path with biomeos subdirectory
    assert!(socket.to_string_lossy().contains("/run/user/1000/biomeos/"));
    assert!(socket.to_string_lossy().contains("beardog-test_family"));
    assert!(socket.to_string_lossy().ends_with(".sock"));
}

#[test]
fn test_xdg_runtime_fallback_to_tmp() {
    // Temporarily unset XDG_RUNTIME_DIR
    std::env::remove_var("XDG_RUNTIME_DIR");

    let mut nucleation = SocketNucleation::new(SocketStrategy::XdgRuntime);
    let socket = nucleation.assign_socket("songbird", "test-family");

    // Should fall back to /tmp with biomeos subdirectory
    assert!(
        socket.to_string_lossy().contains("/tmp/biomeos/")
            || socket.to_string_lossy().contains("songbird-test-family")
    );
}

#[test]
fn test_default_strategy_is_xdg() {
    let nucleation = SocketNucleation::default();
    // Default should be XdgRuntime now
    assert!(matches!(nucleation.strategy, SocketStrategy::XdgRuntime));
}
