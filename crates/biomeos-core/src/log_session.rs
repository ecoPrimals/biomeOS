//! Log session tracking for Tower
//!
//! Tracks active primal sessions and integrates with the fossil record system.
//!
//! EVOLVED (Jan 27, 2026): Full integration with biomeos_spore::logs::LogManager

use biomeos_types::identifiers::PrimalId;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

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
    pub async fn register_session(&self, primal_id: PrimalId, pid: u32, log_file: Option<PathBuf>) {
        let session = PrimalSession {
            primal_id: primal_id.clone(),
            node_id: self.node_id.clone(),
            pid,
            started_at: chrono::Utc::now(),
            log_file,
        };

        self.sessions
            .write()
            .await
            .insert(primal_id.clone(), session);
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
    ///
    /// EVOLVED (Jan 27, 2026): Full integration with biomeos_spore::logs::LogManager
    pub async fn archive_all_sessions(&self, reason: &str) -> anyhow::Result<()> {
        use biomeos_types::SystemPaths;

        let sessions = self.get_all_sessions().await;

        if sessions.is_empty() {
            info!("No active sessions to archive");
            return Ok(());
        }

        info!(
            "Archiving {} active sessions (reason: {})",
            sessions.len(),
            reason
        );

        // Get fossil directory from XDG paths
        let fossil_dir = if let Ok(paths) = SystemPaths::new() {
            paths.data_dir().join("fossil")
        } else {
            PathBuf::from("/var/lib/biomeos/fossil")
        };

        // Ensure fossil directory exists
        if let Err(e) = tokio::fs::create_dir_all(&fossil_dir).await {
            warn!("Could not create fossil directory: {}", e);
        }

        // Archive each session
        for session in sessions {
            let duration = (chrono::Utc::now() - session.started_at).num_seconds();

            info!(
                "  Archiving: {} (PID: {}, duration: {}s)",
                session.primal_id, session.pid, duration
            );

            // Build fossil record
            let fossil_entry = FossilEntry {
                primal_id: session.primal_id.as_str().to_string(),
                node_id: session.node_id.clone(),
                pid: session.pid,
                started_at: session.started_at,
                ended_at: chrono::Utc::now(),
                duration_seconds: duration as u64,
                archival_reason: reason.to_string(),
                log_file: session.log_file.clone(),
            };

            // Save fossil entry
            let fossil_path = fossil_dir.join(format!(
                "{}_{}.fossil.toml",
                session.started_at.format("%Y%m%d_%H%M%S"),
                session.primal_id.as_str().replace('/', "_")
            ));

            match toml::to_string_pretty(&fossil_entry) {
                Ok(content) => {
                    if let Err(e) = tokio::fs::write(&fossil_path, content).await {
                        warn!("Failed to write fossil entry: {}", e);
                    } else {
                        debug!("Fossil created: {}", fossil_path.display());
                    }
                }
                Err(e) => {
                    warn!("Failed to serialize fossil entry: {}", e);
                }
            }

            // Copy log file to fossil directory if it exists
            if let Some(log_file) = &session.log_file {
                if log_file.exists() {
                    let dest = fossil_dir.join(format!(
                        "{}_{}.log",
                        session.started_at.format("%Y%m%d_%H%M%S"),
                        session.primal_id.as_str().replace('/', "_")
                    ));

                    if let Err(e) = tokio::fs::copy(log_file, &dest).await {
                        warn!("Failed to copy log file to fossil: {}", e);
                    } else {
                        debug!("Log archived: {} → {}", log_file.display(), dest.display());
                    }
                }
            }
        }

        // Update fossil index
        self.update_fossil_index(&fossil_dir).await?;

        info!("✅ Sessions archived to fossil record");
        Ok(())
    }

    /// Update the fossil index file
    async fn update_fossil_index(&self, fossil_dir: &PathBuf) -> anyhow::Result<()> {
        let index_path = fossil_dir.join("index.toml");

        // Count fossils
        let mut fossil_count = 0;
        if let Ok(mut entries) = tokio::fs::read_dir(fossil_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if entry.path().extension().is_some_and(|e| e == "toml")
                    && entry.file_name().to_string_lossy().contains(".fossil.")
                {
                    fossil_count += 1;
                }
            }
        }

        // Write index
        let index_content = format!(
            "# Fossil Record Index\n\
             # Auto-generated by LogSessionTracker\n\n\
             [index]\n\
             last_updated = \"{}\"\n\
             total_fossils = {}\n\
             node_id = \"{}\"\n",
            chrono::Utc::now().to_rfc3339(),
            fossil_count,
            self.node_id
        );

        tokio::fs::write(&index_path, index_content).await?;
        debug!("Fossil index updated: {} fossils", fossil_count);

        Ok(())
    }
}

/// Fossil entry for archival
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct FossilEntry {
    primal_id: String,
    node_id: String,
    pid: u32,
    started_at: chrono::DateTime<chrono::Utc>,
    ended_at: chrono::DateTime<chrono::Utc>,
    duration_seconds: u64,
    archival_reason: String,
    log_file: Option<PathBuf>,
}
