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
        Self::FamilyDeterministic
    }
}

/// Socket Nucleation - Assigns sockets deterministically
pub struct SocketNucleation {
    strategy: SocketStrategy,
    assignments: HashMap<String, PathBuf>,
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
            SocketStrategy::XdgRuntime => {
                Self::xdg_runtime_path(primal, family_id)
            }
        };
        
        info!("📍 Socket assigned: {} → {:?}", key, socket);
        self.assignments.insert(key, socket.clone());
        socket
    }
    
    /// Assign sockets for all primals in a list
    pub fn assign_batch(&mut self, primals: &[String], family_id: &str) 
        -> HashMap<String, PathBuf> 
    {
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
    fn xdg_runtime_path(primal: &str, family_id: &str) -> PathBuf {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| "/tmp".to_string());
        
        PathBuf::from(format!("{}/{}-{}.sock", runtime_dir, primal, family_id))
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
        assert_eq!(assignments.get("beardog"), Some(&PathBuf::from("/tmp/beardog-nat0.sock")));
        assert_eq!(assignments.get("songbird"), Some(&PathBuf::from("/tmp/songbird-nat0.sock")));
        assert_eq!(assignments.get("squirrel"), Some(&PathBuf::from("/tmp/squirrel-nat0.sock")));
    }
}

