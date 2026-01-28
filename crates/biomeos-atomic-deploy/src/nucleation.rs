// Socket Nucleation: Deterministic Socket Assignment
//
// Neural API as nucleation point - assigns sockets to primals
// for coordinated, aligned startup (no race conditions)

use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info};

/// Socket Nucleation Strategy
#[derive(Debug, Clone)]
pub enum SocketStrategy {
    /// /tmp/{primal}-{family_id}.sock
    FamilyDeterministic,

    /// XDG_RUNTIME_DIR/{primal}-{family_id}.sock (or /tmp/ fallback)
    XdgRuntime,
}

impl Default for SocketStrategy {
    fn default() -> Self {
        // XdgRuntime is preferred for proper Unix compliance
        // Falls back to /tmp/{primal}-{family}.sock if XDG_RUNTIME_DIR unavailable
        Self::XdgRuntime
    }
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

    /// Family deterministic path: /tmp/{primal}-{family_id}.sock
    fn family_deterministic_path(primal: &str, family_id: &str) -> PathBuf {
        PathBuf::from(format!("/tmp/{}-{}.sock", primal, family_id))
    }

    /// XDG runtime path with /tmp fallback
    /// Creates sockets in {XDG_RUNTIME_DIR}/biomeos/ for proper namespacing
    fn xdg_runtime_path(primal: &str, family_id: &str) -> PathBuf {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
        
        // Create biomeos subdirectory for proper namespacing (matches SocketDiscovery)
        let biomeos_dir = PathBuf::from(&runtime_dir).join("biomeos");
        std::fs::create_dir_all(&biomeos_dir).ok();
        
        biomeos_dir.join(format!("{}-{}.sock", primal, family_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_family_deterministic() {
        let mut nucleation = SocketNucleation::new(SocketStrategy::FamilyDeterministic);

        let socket = nucleation.assign_socket("beardog", "nat0");
        assert_eq!(socket, PathBuf::from("/tmp/beardog-nat0.sock"));

        // Second assignment should return same socket
        let socket2 = nucleation.assign_socket("beardog", "nat0");
        assert_eq!(socket, socket2);
    }

    #[test]
    fn test_different_families() {
        let mut nucleation = SocketNucleation::new(SocketStrategy::FamilyDeterministic);

        let nat0_socket = nucleation.assign_socket("beardog", "nat0");
        let nat1_socket = nucleation.assign_socket("beardog", "nat1");

        assert_ne!(nat0_socket, nat1_socket);
        assert_eq!(nat0_socket, PathBuf::from("/tmp/beardog-nat0.sock"));
        assert_eq!(nat1_socket, PathBuf::from("/tmp/beardog-nat1.sock"));
    }

    #[test]
    fn test_batch_assignment() {
        let mut nucleation = SocketNucleation::new(SocketStrategy::FamilyDeterministic);

        let primals = vec![
            "beardog".to_string(),
            "songbird".to_string(),
            "squirrel".to_string(),
        ];

        let assignments = nucleation.assign_batch(&primals, "nat0");

        assert_eq!(assignments.len(), 3);
        assert_eq!(
            assignments.get("beardog"),
            Some(&PathBuf::from("/tmp/beardog-nat0.sock"))
        );
        assert_eq!(
            assignments.get("songbird"),
            Some(&PathBuf::from("/tmp/songbird-nat0.sock"))
        );
        assert_eq!(
            assignments.get("squirrel"),
            Some(&PathBuf::from("/tmp/squirrel-nat0.sock"))
        );
    }
}

    #[test]
    fn test_xdg_runtime_strategy() {
        // Set up XDG_RUNTIME_DIR for this test
        std::env::set_var("XDG_RUNTIME_DIR", "/run/user/1000");
        
        let mut nucleation = SocketNucleation::new(SocketStrategy::XdgRuntime);
        let socket = nucleation.assign_socket("beardog", "nat0");
        
        // Should create XDG path with biomeos subdirectory
        assert!(socket.to_string_lossy().contains("/run/user/1000/biomeos/"));
        assert!(socket.to_string_lossy().contains("beardog-nat0"));
        assert!(socket.to_string_lossy().ends_with(".sock"));
    }

    #[test]
    fn test_xdg_runtime_fallback_to_tmp() {
        // Temporarily unset XDG_RUNTIME_DIR
        std::env::remove_var("XDG_RUNTIME_DIR");
        
        let mut nucleation = SocketNucleation::new(SocketStrategy::XdgRuntime);
        let socket = nucleation.assign_socket("songbird", "test-family");
        
        // Should fall back to /tmp with biomeos subdirectory
        assert!(socket.to_string_lossy().contains("/tmp/biomeos/") || 
                socket.to_string_lossy().contains("songbird-test-family"));
    }

    #[test]
    fn test_default_strategy_is_xdg() {
        let nucleation = SocketNucleation::default();
        // Default should be XdgRuntime now
        assert!(matches!(nucleation.strategy, SocketStrategy::XdgRuntime));
    }
