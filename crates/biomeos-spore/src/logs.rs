//! Log Management and Fossil Record System
//!
//! This module provides comprehensive log management for biomeOS deployments:
//! - Active log tracking by node ID
//! - Automatic archival to fossil records
//! - Issue detection and forensic analysis
//! - Future: BearDog encryption for secure audit trails

use crate::error::SporeResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;
use tokio::fs as async_fs;
use tracing::{debug, info, warn};

// ============================================================================
// Configuration
// ============================================================================

/// Configuration for log management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// Where to store active logs
    pub active_dir: PathBuf,

    /// Where to archive fossil logs
    pub fossil_dir: PathBuf,

    /// Maximum age before auto-archival (seconds)
    pub max_active_age_secs: u64,

    /// Whether to enable BearDog encryption (future)
    pub enable_encryption: bool,

    /// Compression for fossil logs
    pub compress_fossils: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            active_dir: PathBuf::from("/var/biomeos/logs/active"),
            fossil_dir: PathBuf::from("/var/biomeos/logs/fossil"),
            max_active_age_secs: 86400, // 24 hours
            enable_encryption: false,   // Future feature
            compress_fossils: true,
        }
    }
}

// ============================================================================
// Active Log Session
// ============================================================================

/// Metadata for an active log session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveLogSession {
    /// Node ID
    pub node_id: String,

    /// When this session started
    pub started_at: DateTime<Utc>,

    /// PIDs of running processes
    pub process_pids: Vec<u32>,

    /// Log file paths
    pub log_files: Vec<LogFile>,

    /// Deployment this session is from
    pub deployment_id: String,
}

impl ActiveLogSession {
    /// Create a new active log session
    pub fn new(node_id: String, deployment_id: String) -> Self {
        Self {
            node_id,
            started_at: Utc::now(),
            process_pids: Vec::new(),
            log_files: Vec::new(),
            deployment_id,
        }
    }

    /// Add a process PID to this session
    pub fn add_process(&mut self, pid: u32) {
        if !self.process_pids.contains(&pid) {
            self.process_pids.push(pid);
        }
    }

    /// Add a log file to this session
    pub fn add_log_file(&mut self, log_file: LogFile) {
        self.log_files.push(log_file);
    }

    /// Check if all processes are still running
    pub fn is_active(&self) -> bool {
        self.process_pids.iter().any(|&pid| {
            // Check if process exists via /proc
            PathBuf::from(format!("/proc/{}", pid)).exists()
        })
    }

    /// Get session duration
    pub fn duration(&self) -> chrono::Duration {
        Utc::now() - self.started_at
    }
}

/// Individual log file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFile {
    /// Primal name (tower, beardog, songbird)
    pub primal: String,

    /// File path
    pub path: PathBuf,

    /// Process PID
    pub pid: Option<u32>,

    /// Size in bytes
    pub size_bytes: u64,

    /// Last modified
    pub last_modified: DateTime<Utc>,
}

impl LogFile {
    /// Update size and last modified from filesystem
    pub fn refresh(&mut self) -> io::Result<()> {
        let metadata = fs::metadata(&self.path)?;
        self.size_bytes = metadata.len();
        self.last_modified = metadata.modified()?.into();
        Ok(())
    }
}

// ============================================================================
// Fossil Record
// ============================================================================

/// Fossil record metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FossilRecord {
    /// Original node ID
    pub node_id: String,

    /// When session started
    pub session_started: DateTime<Utc>,

    /// When session ended (archived)
    pub session_ended: DateTime<Utc>,

    /// Why was this archived?
    pub archival_reason: ArchivalReason,

    /// Deployment ID
    pub deployment_id: String,

    /// Issues detected (errors, warnings)
    pub issues: Vec<LogIssue>,

    /// Metrics summary
    pub metrics: Option<LogMetrics>,

    /// Encrypted with BearDog? (future)
    pub encrypted: bool,

    /// Parent seed fingerprint (for decryption)
    pub parent_seed_fingerprint: Option<String>,
}

impl FossilRecord {
    /// Create a fossil record from an active session
    pub fn from_active_session(session: &ActiveLogSession, reason: ArchivalReason) -> Self {
        Self {
            node_id: session.node_id.clone(),
            session_started: session.started_at,
            session_ended: Utc::now(),
            archival_reason: reason,
            deployment_id: session.deployment_id.clone(),
            issues: Vec::new(), // Will be populated by log analysis
            metrics: None,      // Will be calculated
            encrypted: false,   // Future feature
            parent_seed_fingerprint: None,
        }
    }

    /// Get session duration
    pub fn duration(&self) -> chrono::Duration {
        self.session_ended - self.session_started
    }

    /// Count issues by severity
    pub fn issue_count(&self, severity: Option<IssueSeverity>) -> usize {
        match severity {
            Some(sev) => self.issues.iter().filter(|i| i.severity == sev).count(),
            None => self.issues.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArchivalReason {
    /// Normal shutdown
    GracefulShutdown,

    /// Process crashed
    Crash { exit_code: i32 },

    /// Manual archival by user
    Manual,

    /// Automatic archival (age threshold)
    AutomaticRotation,

    /// New deployment replacing old
    Redeployment,

    /// System reboot
    Reboot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogIssue {
    /// Timestamp of issue
    pub timestamp: DateTime<Utc>,

    /// Severity (error, warning, info)
    pub severity: IssueSeverity,

    /// Primal where issue occurred
    pub primal: String,

    /// Issue description
    pub description: String,

    /// Log line where it occurred
    pub log_line: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IssueSeverity {
    Critical,
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMetrics {
    /// Total lines logged
    pub total_lines: u64,

    /// Lines by severity
    pub errors: u64,
    pub warnings: u64,
    pub info: u64,

    /// Total size
    pub total_size_bytes: u64,

    /// Session duration
    pub duration_secs: u64,
}

// ============================================================================
// Fossil Index
// ============================================================================

/// Searchable index of all fossil records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FossilIndex {
    /// All fossils
    pub fossils: Vec<FossilIndexEntry>,

    /// Last updated
    pub last_updated: DateTime<Utc>,
}

impl FossilIndex {
    /// Create a new empty index
    pub fn new() -> Self {
        Self {
            fossils: Vec::new(),
            last_updated: Utc::now(),
        }
    }

    /// Add a fossil to the index
    pub fn add(&mut self, entry: FossilIndexEntry) {
        self.fossils.push(entry);
        self.last_updated = Utc::now();
    }

    /// Find fossils by node ID
    pub fn find_by_node(&self, node_id: &str) -> Vec<&FossilIndexEntry> {
        self.fossils
            .iter()
            .filter(|f| f.node_id == node_id)
            .collect()
    }

    /// Load index from file
    pub fn load(path: &PathBuf) -> SporeResult<Self> {
        let content = fs::read_to_string(path)?;
        let index: FossilIndex = toml::from_str(&content)?;
        Ok(index)
    }

    /// Save index to file
    pub fn save(&self, path: &PathBuf) -> SporeResult<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

impl Default for FossilIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FossilIndexEntry {
    /// Node ID
    pub node_id: String,

    /// Session timestamp
    pub session_started: DateTime<Utc>,

    /// Archival reason
    pub archival_reason: ArchivalReason,

    /// Path to fossil directory
    pub fossil_path: PathBuf,

    /// Number of issues
    pub issue_count: usize,

    /// Encrypted?
    pub encrypted: bool,
}

// ============================================================================
// Log Manager
// ============================================================================

/// Main log management interface
pub struct LogManager {
    config: LogConfig,
}

impl LogManager {
    /// Create a new log manager
    pub fn new(config: LogConfig) -> Self {
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
                    match self.load_active_session(&metadata_path) {
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
    fn load_active_session(&self, path: &PathBuf) -> SporeResult<ActiveLogSession> {
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
                let dest = fossil_dir.join(log_file.path.file_name().unwrap());
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
        self.update_fossil_index(&fossil, &fossil_dir).await?;

        Ok(fossil)
    }

    /// Update the fossil index with a new entry
    async fn update_fossil_index(
        &self,
        fossil: &FossilRecord,
        fossil_path: &PathBuf,
    ) -> SporeResult<()> {
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
            fossil_path: fossil_path.clone(),
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

// ============================================================================
// Spore Log Integration
// ============================================================================

/// Spore-specific log management
pub struct SporeLogManager {
    spore_root: PathBuf,
}

impl SporeLogManager {
    /// Create a new spore log manager
    pub fn new(spore_root: PathBuf) -> Self {
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
        let log_file = log_dir.join(format!("{}.log", deployment_id));

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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ========== Configuration Tests ==========

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.active_dir, PathBuf::from("/var/biomeos/logs/active"));
        assert_eq!(config.fossil_dir, PathBuf::from("/var/biomeos/logs/fossil"));
        assert_eq!(config.max_active_age_secs, 86400);
        assert!(!config.enable_encryption); // Future feature
        assert!(config.compress_fossils);
    }

    #[test]
    fn test_log_config_custom() {
        let config = LogConfig {
            active_dir: PathBuf::from("/custom/active"),
            fossil_dir: PathBuf::from("/custom/fossil"),
            max_active_age_secs: 3600,
            enable_encryption: true,
            compress_fossils: false,
        };

        assert_eq!(config.active_dir, PathBuf::from("/custom/active"));
        assert_eq!(config.max_active_age_secs, 3600);
        assert!(config.enable_encryption);
        assert!(!config.compress_fossils);
    }

    // ========== Active Log Session Tests ==========

    #[test]
    fn test_active_log_session_creation() {
        let session = ActiveLogSession::new("test-node-1".to_string(), "deploy-123".to_string());

        assert_eq!(session.node_id, "test-node-1");
        assert_eq!(session.deployment_id, "deploy-123");
        assert!(session.process_pids.is_empty());
        assert!(session.log_files.is_empty());
    }

    #[test]
    fn test_active_log_session_add_process() {
        let mut session = ActiveLogSession::new("test-node".to_string(), "deploy-1".to_string());

        session.add_process(1234);
        assert_eq!(session.process_pids.len(), 1);
        assert!(session.process_pids.contains(&1234));

        // Adding same PID again should not duplicate
        session.add_process(1234);
        assert_eq!(session.process_pids.len(), 1);

        // Adding different PID should work
        session.add_process(5678);
        assert_eq!(session.process_pids.len(), 2);
    }

    #[test]
    fn test_active_log_session_duration() {
        let session = ActiveLogSession::new("test-node".to_string(), "deploy-1".to_string());

        std::thread::sleep(std::time::Duration::from_millis(100));
        let duration = session.duration();
        assert!(duration.num_milliseconds() >= 100);
    }

    #[test]
    fn test_active_log_session_add_log_file() {
        let mut session = ActiveLogSession::new("test-node".to_string(), "deploy-1".to_string());

        let log_file = LogFile {
            primal: "tower".to_string(),
            path: PathBuf::from("/tmp/test.log"),
            pid: Some(1234),
            size_bytes: 1024,
            last_modified: Utc::now(),
        };

        session.add_log_file(log_file.clone());
        assert_eq!(session.log_files.len(), 1);
        assert_eq!(session.log_files[0].primal, "tower");
        assert_eq!(session.log_files[0].size_bytes, 1024);
    }

    #[test]
    fn test_multiple_processes() {
        let mut session = ActiveLogSession::new("multi-node".to_string(), "deploy-xyz".to_string());

        // Add multiple processes
        for pid in 1000..1005 {
            session.add_process(pid);
        }
        assert_eq!(session.process_pids.len(), 5);

        // Verify all PIDs are present
        for pid in 1000..1005 {
            assert!(session.process_pids.contains(&pid));
        }
    }

    // ========== Log File Tests ==========

    #[test]
    fn test_log_file_refresh() {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test.log");

        // Create a test file
        std::fs::write(&log_path, b"test log content").unwrap();

        let mut log_file = LogFile {
            primal: "tower".to_string(),
            path: log_path.clone(),
            pid: Some(1234),
            size_bytes: 0, // Will be updated
            last_modified: Utc::now(),
        };

        // Refresh should update size
        log_file.refresh().unwrap();
        assert_eq!(log_file.size_bytes, 16); // "test log content".len()
    }

    #[test]
    fn test_log_file_refresh_missing_file() {
        let mut log_file = LogFile {
            primal: "tower".to_string(),
            path: PathBuf::from("/nonexistent/file.log"),
            pid: Some(1234),
            size_bytes: 0,
            last_modified: Utc::now(),
        };

        // Should return error for missing file
        assert!(log_file.refresh().is_err());
    }

    // ========== Fossil Record Tests ==========

    #[test]
    fn test_fossil_record_from_active_session() {
        let session = ActiveLogSession::new("node-1".to_string(), "deploy-abc".to_string());

        let fossil = FossilRecord::from_active_session(&session, ArchivalReason::GracefulShutdown);

        assert_eq!(fossil.node_id, "node-1");
        assert_eq!(fossil.deployment_id, "deploy-abc");
        assert_eq!(fossil.archival_reason, ArchivalReason::GracefulShutdown);
        assert!(!fossil.encrypted); // Future feature
        assert!(fossil.issues.is_empty());
        assert!(fossil.metrics.is_none());
    }

    #[test]
    fn test_fossil_record_duration() {
        let session = ActiveLogSession::new("node-1".to_string(), "deploy-1".to_string());

        std::thread::sleep(std::time::Duration::from_millis(50));

        let fossil = FossilRecord::from_active_session(&session, ArchivalReason::GracefulShutdown);

        let duration = fossil.duration();
        assert!(duration.num_milliseconds() >= 50);
    }

    #[test]
    fn test_fossil_record_issue_count() {
        let mut fossil = FossilRecord::from_active_session(
            &ActiveLogSession::new("node-1".to_string(), "deploy-1".to_string()),
            ArchivalReason::GracefulShutdown,
        );

        // Add some test issues
        fossil.issues.push(LogIssue {
            severity: IssueSeverity::Error,
            primal: "tower".to_string(),
            description: "Test error".to_string(),
            log_line: Some("ERROR: test error occurred".to_string()),
            timestamp: Utc::now(),
        });

        fossil.issues.push(LogIssue {
            severity: IssueSeverity::Warning,
            primal: "beardog".to_string(),
            description: "Test warning".to_string(),
            log_line: None,
            timestamp: Utc::now(),
        });

        fossil.issues.push(LogIssue {
            severity: IssueSeverity::Critical,
            primal: "songbird".to_string(),
            description: "Critical issue".to_string(),
            log_line: Some("CRITICAL: system failure".to_string()),
            timestamp: Utc::now(),
        });

        assert_eq!(fossil.issue_count(None), 3);
        assert_eq!(fossil.issue_count(Some(IssueSeverity::Error)), 1);
        assert_eq!(fossil.issue_count(Some(IssueSeverity::Warning)), 1);
        assert_eq!(fossil.issue_count(Some(IssueSeverity::Critical)), 1);
        assert_eq!(fossil.issue_count(Some(IssueSeverity::Info)), 0);
    }

    #[test]
    fn test_archival_reason_crash() {
        let reason = ArchivalReason::Crash { exit_code: 137 };

        match reason {
            ArchivalReason::Crash { exit_code } => assert_eq!(exit_code, 137),
            _ => panic!("Expected Crash variant"),
        }
    }

    #[test]
    fn test_archival_reason_equality() {
        assert_eq!(
            ArchivalReason::GracefulShutdown,
            ArchivalReason::GracefulShutdown
        );
        assert_eq!(
            ArchivalReason::Crash { exit_code: 1 },
            ArchivalReason::Crash { exit_code: 1 }
        );
        assert_ne!(
            ArchivalReason::Crash { exit_code: 1 },
            ArchivalReason::Crash { exit_code: 2 }
        );
    }

    // ========== Log Manager Tests ==========

    #[tokio::test]
    async fn test_log_manager_initialization() {
        let temp_dir = TempDir::new().unwrap();

        let config = LogConfig {
            active_dir: temp_dir.path().join("active"),
            fossil_dir: temp_dir.path().join("fossil"),
            max_active_age_secs: 3600,
            enable_encryption: false,
            compress_fossils: true,
        };

        let manager = LogManager::new(config.clone());
        manager.initialize().await.unwrap();

        // Verify directories were created
        assert!(config.active_dir.exists());
        assert!(config.fossil_dir.exists());
    }

    #[test]
    fn test_log_manager_list_active_sessions_empty() {
        let temp_dir = TempDir::new().unwrap();

        let config = LogConfig {
            active_dir: temp_dir.path().join("active"),
            fossil_dir: temp_dir.path().join("fossil"),
            ..Default::default()
        };

        // Active dir doesn't exist yet
        let manager = LogManager::new(config);
        let sessions = manager.list_active_sessions().unwrap();

        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn test_log_manager_archive_session() {
        let temp_dir = TempDir::new().unwrap();

        let config = LogConfig {
            active_dir: temp_dir.path().join("active"),
            fossil_dir: temp_dir.path().join("fossil"),
            ..Default::default()
        };

        let manager = LogManager::new(config.clone());
        manager.initialize().await.unwrap();

        // Create a test session with a log file
        let log_path = temp_dir.path().join("test.log");
        tokio::fs::write(&log_path, b"test log data").await.unwrap();

        let mut session = ActiveLogSession::new("test-node".to_string(), "deploy-456".to_string());
        session.add_process(std::process::id());

        let log_file = LogFile {
            primal: "tower".to_string(),
            path: log_path.clone(),
            pid: Some(std::process::id()),
            size_bytes: 13,
            last_modified: Utc::now(),
        };
        session.add_log_file(log_file);

        // Archive the session
        let fossil = manager
            .archive_session(&session, ArchivalReason::Manual)
            .await
            .unwrap();

        assert_eq!(fossil.node_id, "test-node");
        assert_eq!(fossil.deployment_id, "deploy-456");
        assert_eq!(fossil.archival_reason, ArchivalReason::Manual);
    }

    // ========== Spore Log Manager Tests ==========

    #[tokio::test]
    async fn test_spore_log_manager_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let spore_root = temp_dir.path().to_path_buf();

        let manager = SporeLogManager::new(spore_root.clone());
        manager.initialize().await.unwrap();

        // Verify spore log structure created
        assert!(spore_root.join(".spore.logs").exists());
        assert!(spore_root.join(".spore.logs/deployments").exists());
        assert!(spore_root.join(".spore.logs/fossil").exists());
    }

    #[tokio::test]
    async fn test_spore_log_manager_record_deployment() {
        let temp_dir = TempDir::new().unwrap();
        let spore_root = temp_dir.path().to_path_buf();

        let manager = SporeLogManager::new(spore_root.clone());
        manager.initialize().await.unwrap();

        // Record a deployment
        manager.record_deployment("deploy-789").await.unwrap();

        // Verify log file was created
        let log_file = spore_root.join(".spore.logs/deployments/deploy-789.log");
        assert!(log_file.exists());

        // Verify content includes deployment ID
        let content = tokio::fs::read_to_string(&log_file).await.unwrap();
        assert!(content.contains("deploy-789"));
    }

    // ========== Integration Tests ==========

    #[tokio::test]
    async fn test_full_log_lifecycle() {
        let temp_dir = TempDir::new().unwrap();

        let config = LogConfig {
            active_dir: temp_dir.path().join("active"),
            fossil_dir: temp_dir.path().join("fossil"),
            max_active_age_secs: 1, // 1 second for quick test
            enable_encryption: false,
            compress_fossils: true,
        };

        let manager = LogManager::new(config.clone());
        manager.initialize().await.unwrap();

        // 1. Create active session
        let mut session =
            ActiveLogSession::new("lifecycle-node".to_string(), "deploy-999".to_string());
        session.add_process(std::process::id());

        // 2. Add log file
        let log_path = temp_dir.path().join("lifecycle.log");
        tokio::fs::write(&log_path, b"lifecycle test data")
            .await
            .unwrap();

        let log_file = LogFile {
            primal: "beardog".to_string(),
            path: log_path,
            pid: Some(std::process::id()),
            size_bytes: 19,
            last_modified: Utc::now(),
        };
        session.add_log_file(log_file);

        // 3. Archive session
        let fossil = manager
            .archive_session(&session, ArchivalReason::Redeployment)
            .await
            .unwrap();

        // 4. Verify fossil record
        assert_eq!(fossil.archival_reason, ArchivalReason::Redeployment);
        assert!(fossil.duration().num_milliseconds() >= 0);
    }

    // ========== Issue Severity Tests ==========

    #[test]
    fn test_issue_severity_ordering() {
        // Ensure proper severity levels
        let critical = IssueSeverity::Critical;
        let error = IssueSeverity::Error;
        let warning = IssueSeverity::Warning;
        let info = IssueSeverity::Info;

        // Just verify they can be created and compared
        assert_eq!(critical, IssueSeverity::Critical);
        assert_ne!(error, warning);
        assert_ne!(info, critical);
    }

    // ========== Fossil Index Tests ==========

    #[test]
    fn test_fossil_index_creation() {
        let index = FossilIndex::new();
        assert!(index.fossils.is_empty());
    }

    #[test]
    fn test_fossil_index_add_entry() {
        let mut index = FossilIndex::new();

        let entry = FossilIndexEntry {
            node_id: "test-node".to_string(),
            session_started: Utc::now(),
            archival_reason: ArchivalReason::Manual,
            fossil_path: PathBuf::from("/tmp/fossil/test"),
            issue_count: 3,
            encrypted: false,
        };

        index.add(entry.clone());
        assert_eq!(index.fossils.len(), 1);
        assert_eq!(index.fossils[0].node_id, "test-node");
        assert_eq!(index.fossils[0].issue_count, 3);
    }

    #[test]
    fn test_fossil_index_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("index.toml");

        // Create and save index
        let mut index = FossilIndex::new();
        index.add(FossilIndexEntry {
            node_id: "node-save-load".to_string(),
            session_started: Utc::now(),
            archival_reason: ArchivalReason::GracefulShutdown,
            fossil_path: PathBuf::from("/tmp/fossil/save-load"),
            issue_count: 0,
            encrypted: false,
        });

        index.save(&index_path).unwrap();
        assert!(index_path.exists());

        // Load index
        let loaded = FossilIndex::load(&index_path).unwrap();
        assert_eq!(loaded.fossils.len(), 1);
        assert_eq!(loaded.fossils[0].node_id, "node-save-load");
    }
}
