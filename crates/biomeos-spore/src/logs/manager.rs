// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Log manager implementations.
//!
//! Provides:
//! - `LogManager` - Main log management interface
//! - `SporeLogManager` - Spore-specific log integration

use super::config::LogConfig;
use super::fossil::{ArchivalReason, FossilIndex, FossilIndexEntry, FossilRecord};
use super::session::ActiveLogSession;
use crate::error::SporeResult;
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;
use tracing::{debug, info, warn};

/// Main log management interface
pub struct LogManager {
    config: LogConfig,
}

impl LogManager {
    /// Create a new log manager
    #[must_use]
    pub const fn new(config: LogConfig) -> Self {
        Self { config }
    }

    /// Initialize log directories
    pub async fn initialize(&self) -> SporeResult<()> {
        info!("Initializing log management system");

        async_fs::create_dir_all(&self.config.active_dir).await?;
        async_fs::create_dir_all(&self.config.fossil_dir).await?;

        info!("✅ Log directories initialized");
        Ok(())
    }

    /// List all active log sessions
    pub fn list_active_sessions(&self) -> SporeResult<Vec<ActiveLogSession>> {
        let mut sessions = Vec::new();

        if !self.config.active_dir.exists() {
            return Ok(sessions);
        }

        for entry in fs::read_dir(&self.config.active_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let metadata_path = path.join(".metadata.toml");
                if metadata_path.exists() {
                    match Self::load_active_session(&metadata_path) {
                        Ok(session) => sessions.push(session),
                        Err(e) => {
                            warn!(
                                "Failed to load session from {}: {}",
                                metadata_path.display(),
                                e
                            );
                        }
                    }
                }
            }
        }

        Ok(sessions)
    }

    /// Load an active session from metadata file
    fn load_active_session(path: &PathBuf) -> SporeResult<ActiveLogSession> {
        let content = fs::read_to_string(path)?;
        let session: ActiveLogSession = toml::from_str(&content)?;
        Ok(session)
    }

    /// Archive a session to fossil record
    pub async fn archive_session(
        &self,
        session: &ActiveLogSession,
        reason: ArchivalReason,
    ) -> SporeResult<FossilRecord> {
        info!("Archiving session for node: {}", session.node_id);

        let fossil = FossilRecord::from_active_session(session, reason);

        // Create fossil directory
        let fossil_dir_name = format!(
            "{}_{}",
            fossil.session_started.format("%Y-%m-%d_%H-%M-%S"),
            fossil.node_id
        );
        let fossil_dir = self.config.fossil_dir.join(fossil_dir_name);
        async_fs::create_dir_all(&fossil_dir).await?;

        // Copy log files to fossil directory
        for log_file in &session.log_files {
            if log_file.path.exists() {
                #[expect(clippy::expect_used, reason = "log file path must have filename")]
                let dest = fossil_dir.join(
                    log_file
                        .path
                        .file_name()
                        .expect("log file path must have filename"),
                );
                async_fs::copy(&log_file.path, &dest).await?;
                debug!(
                    "Archived log: {} → {}",
                    log_file.path.display(),
                    dest.display()
                );
            }
        }

        // Save fossil metadata
        let metadata_path = fossil_dir.join(".fossil.toml");
        let metadata_content = toml::to_string_pretty(&fossil)?;
        async_fs::write(&metadata_path, metadata_content).await?;

        info!("✅ Session archived to: {}", fossil_dir.display());

        // Update fossil index
        self.update_fossil_index(&fossil, &fossil_dir)?;

        Ok(fossil)
    }

    /// Update the fossil index with a new entry
    fn update_fossil_index(&self, fossil: &FossilRecord, fossil_path: &Path) -> SporeResult<()> {
        let index_path = self.config.fossil_dir.join("index.toml");

        let mut index = if index_path.exists() {
            FossilIndex::load(&index_path)?
        } else {
            FossilIndex::new()
        };

        let entry = FossilIndexEntry {
            node_id: fossil.node_id.clone(),
            session_started: fossil.session_started,
            archival_reason: fossil.archival_reason.clone(),
            fossil_path: fossil_path.to_path_buf(),
            issue_count: fossil.issues.len(),
            encrypted: fossil.encrypted,
        };

        index.add(entry);
        index.save(&index_path)?;

        debug!("Updated fossil index");
        Ok(())
    }

    /// Clean up stale active sessions (processes no longer running)
    pub async fn cleanup_stale_sessions(&self) -> SporeResult<Vec<FossilRecord>> {
        info!("Cleaning up stale active sessions");

        let active_sessions = self.list_active_sessions()?;
        let mut archived = Vec::new();

        for session in active_sessions {
            if !session.is_active() {
                info!("Found stale session: {}", session.node_id);
                let fossil = self
                    .archive_session(&session, ArchivalReason::AutomaticRotation)
                    .await?;
                archived.push(fossil);

                // Remove active session directory
                let session_dir = self.config.active_dir.join(&session.node_id);
                if session_dir.exists() {
                    async_fs::remove_dir_all(&session_dir).await?;
                }
            }
        }

        info!("✅ Cleaned up {} stale sessions", archived.len());
        Ok(archived)
    }
}

/// Spore-specific log management
pub struct SporeLogManager {
    spore_root: PathBuf,
}

impl SporeLogManager {
    /// Create a new spore log manager
    #[must_use]
    pub const fn new(spore_root: PathBuf) -> Self {
        Self { spore_root }
    }

    /// Initialize spore log directory
    pub async fn initialize(&self) -> SporeResult<()> {
        let log_dir = self.spore_root.join(".spore.logs");
        async_fs::create_dir_all(&log_dir).await?;

        let deployments_dir = log_dir.join("deployments");
        async_fs::create_dir_all(&deployments_dir).await?;

        let fossil_dir = log_dir.join("fossil");
        async_fs::create_dir_all(&fossil_dir).await?;

        info!(
            "✅ Spore log directories initialized: {}",
            log_dir.display()
        );
        Ok(())
    }

    /// Record a new deployment
    pub async fn record_deployment(&self, deployment_id: &str) -> SporeResult<()> {
        let log_dir = self.spore_root.join(".spore.logs/deployments");
        let log_file = log_dir.join(format!("{deployment_id}.log"));

        let entry = format!(
            "[{}] Deployment: {}\n",
            Utc::now().to_rfc3339(),
            deployment_id
        );

        async_fs::write(&log_file, entry).await?;
        info!("Recorded deployment: {}", deployment_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logs::session::{ActiveLogSession, LogFile};
    use chrono::Utc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_log_manager_initialize() {
        let temp = TempDir::new().unwrap();
        let config = LogConfig {
            active_dir: temp.path().join("active"),
            fossil_dir: temp.path().join("fossil"),
            ..Default::default()
        };

        let manager = LogManager::new(config.clone());
        manager.initialize().await.unwrap();

        assert!(config.active_dir.exists());
        assert!(config.fossil_dir.exists());
    }

    #[tokio::test]
    async fn test_spore_log_manager_initialize() {
        let temp = TempDir::new().unwrap();
        let manager = SporeLogManager::new(temp.path().to_path_buf());
        manager.initialize().await.unwrap();

        assert!(temp.path().join(".spore.logs").exists());
        assert!(temp.path().join(".spore.logs/deployments").exists());
        assert!(temp.path().join(".spore.logs/fossil").exists());
    }

    #[test]
    fn test_list_active_sessions_empty() {
        let temp = TempDir::new().unwrap();
        let config = LogConfig {
            active_dir: temp.path().join("active"),
            fossil_dir: temp.path().join("fossil"),
            ..Default::default()
        };
        let manager = LogManager::new(config);
        let sessions = manager.list_active_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_list_active_sessions_nonexistent_dir() {
        let temp = TempDir::new().unwrap();
        let config = LogConfig {
            active_dir: temp.path().join("nonexistent"),
            fossil_dir: temp.path().join("fossil"),
            ..Default::default()
        };
        let manager = LogManager::new(config);
        let sessions = manager.list_active_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn test_spore_log_manager_record_deployment() {
        let temp = TempDir::new().unwrap();
        let manager = SporeLogManager::new(temp.path().to_path_buf());
        manager.initialize().await.unwrap();

        manager.record_deployment("deploy-123").await.unwrap();

        let log_path = temp.path().join(".spore.logs/deployments/deploy-123.log");
        assert!(log_path.exists());
        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(content.contains("deploy-123"));
    }

    #[tokio::test]
    async fn test_archive_session() {
        let temp = TempDir::new().unwrap();
        let config = LogConfig {
            active_dir: temp.path().join("active"),
            fossil_dir: temp.path().join("fossil"),
            ..Default::default()
        };
        let manager = LogManager::new(config.clone());
        manager.initialize().await.unwrap();

        let mut session = ActiveLogSession::new("node-1".to_string(), "deploy-1".to_string());
        let log_path = temp.path().join("active/node-1/test.log");
        std::fs::create_dir_all(log_path.parent().unwrap()).unwrap();
        std::fs::write(&log_path, "log content").unwrap();

        session.add_log_file(LogFile {
            primal: "tower".to_string(),
            path: log_path,
            pid: None,
            size_bytes: 10,
            last_modified: Utc::now(),
        });

        let fossil = manager
            .archive_session(&session, crate::logs::ArchivalReason::GracefulShutdown)
            .await
            .unwrap();

        assert_eq!(fossil.node_id, "node-1");
        assert_eq!(fossil.deployment_id, "deploy-1");
    }

    #[tokio::test]
    async fn test_cleanup_stale_sessions_empty() {
        let temp = TempDir::new().unwrap();
        let config = LogConfig {
            active_dir: temp.path().join("active"),
            fossil_dir: temp.path().join("fossil"),
            ..Default::default()
        };
        let manager = LogManager::new(config);
        manager.initialize().await.unwrap();

        let archived = manager.cleanup_stale_sessions().await.unwrap();
        assert!(archived.is_empty());
    }
}
