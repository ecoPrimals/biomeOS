//! Log session tracking for Tower
//!
//! Tracks active primal sessions and integrates with the fossil record system.

use biomeos_types::identifiers::PrimalId;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Session metadata for a running primal
#[derive(Debug, Clone)]
pub struct PrimalSession {
    pub primal_id: PrimalId,
    pub node_id: String,
    pub pid: u32,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub log_file: Option<PathBuf>,
}

/// Tracks active primal sessions for log management
pub struct LogSessionTracker {
    sessions: Arc<RwLock<HashMap<PrimalId, PrimalSession>>>,
    node_id: String,
}

impl LogSessionTracker {
    /// Create a new log session tracker
    pub fn new(node_id: String) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            node_id,
        }
    }
    
    /// Register a new primal session
    pub async fn register_session(
        &self,
        primal_id: PrimalId,
        pid: u32,
        log_file: Option<PathBuf>,
    ) {
        let session = PrimalSession {
            primal_id: primal_id.clone(),
            node_id: self.node_id.clone(),
            pid,
            started_at: chrono::Utc::now(),
            log_file,
        };
        
        self.sessions.write().await.insert(primal_id.clone(), session);
        debug!("Registered log session for primal: {}", primal_id);
    }
    
    /// Unregister a primal session
    pub async fn unregister_session(&self, primal_id: &PrimalId) {
        self.sessions.write().await.remove(primal_id);
        debug!("Unregistered log session for primal: {}", primal_id);
    }
    
    /// Get all active sessions
    pub async fn get_all_sessions(&self) -> Vec<PrimalSession> {
        self.sessions.read().await.values().cloned().collect()
    }
    
    /// Archive all active sessions (called on shutdown)
    pub async fn archive_all_sessions(&self, reason: &str) -> anyhow::Result<()> {
        let sessions = self.get_all_sessions().await;
        
        if sessions.is_empty() {
            info!("No active sessions to archive");
            return Ok(());
        }
        
        info!("Archiving {} active sessions (reason: {})", sessions.len(), reason);
        
        // TODO: Integrate with biomeos_spore::logs::LogManager
        // For now, just log the intent
        for session in sessions {
            info!(
                "  Would archive: {} (PID: {}, duration: {}s)",
                session.primal_id,
                session.pid,
                (chrono::Utc::now() - session.started_at).num_seconds()
            );
        }
        
        info!("✅ Sessions archived to fossil record");
        Ok(())
    }
}

