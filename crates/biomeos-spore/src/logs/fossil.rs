//! Fossil record archival system.
//!
//! Provides immutable log archives with:
//! - Archival reason tracking
//! - Issue and metrics preservation
//! - Searchable index

use super::metrics::{IssueSeverity, LogIssue, LogMetrics};
use super::session::ActiveLogSession;
use crate::error::SporeResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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

/// Reason for log archival.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArchivalReason {
    /// Normal shutdown
    GracefulShutdown,

    /// Process crashed
    Crash {
        /// Process exit code
        exit_code: i32,
    },

    /// Manual archival by user
    Manual,

    /// Automatic archival (age threshold)
    AutomaticRotation,

    /// New deployment replacing old
    Redeployment,

    /// System reboot
    Reboot,
}

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

/// Entry in the fossil index.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fossil_index_new() {
        let index = FossilIndex::new();
        assert!(index.fossils.is_empty());
    }

    #[test]
    fn test_fossil_index_find_by_node() {
        let mut index = FossilIndex::new();
        index.add(FossilIndexEntry {
            node_id: "node-1".into(),
            session_started: Utc::now(),
            archival_reason: ArchivalReason::GracefulShutdown,
            fossil_path: PathBuf::from("/tmp/fossil1"),
            issue_count: 0,
            encrypted: false,
        });

        let found = index.find_by_node("node-1");
        assert_eq!(found.len(), 1);

        let not_found = index.find_by_node("node-2");
        assert!(not_found.is_empty());
    }
}
