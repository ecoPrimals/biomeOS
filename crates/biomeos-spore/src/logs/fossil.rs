// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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

    /// Encrypted with `BearDog`? (future)
    pub encrypted: bool,

    /// Parent seed fingerprint (for decryption)
    pub parent_seed_fingerprint: Option<String>,
}

impl FossilRecord {
    /// Create a fossil record from an active session
    #[must_use]
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
    #[must_use]
    pub fn duration(&self) -> chrono::Duration {
        self.session_ended - self.session_started
    }

    /// Count issues by severity
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn find_by_node(&self, node_id: &str) -> Vec<&FossilIndexEntry> {
        self.fossils
            .iter()
            .filter(|f| f.node_id == node_id)
            .collect()
    }

    /// Load index from file
    pub fn load(path: &PathBuf) -> SporeResult<Self> {
        let content = fs::read_to_string(path)?;
        let index: Self = toml::from_str(&content)?;
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
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
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

    #[test]
    fn test_fossil_index_add() {
        let mut index = FossilIndex::new();
        index.add(FossilIndexEntry {
            node_id: "n1".into(),
            session_started: Utc::now(),
            archival_reason: ArchivalReason::Manual,
            fossil_path: PathBuf::from("/tmp/f1"),
            issue_count: 0,
            encrypted: false,
        });
        assert_eq!(index.fossils.len(), 1);
    }

    #[test]
    fn test_fossil_index_load_save() {
        let mut index = FossilIndex::new();
        index.add(FossilIndexEntry {
            node_id: "node-save".into(),
            session_started: Utc::now(),
            archival_reason: ArchivalReason::GracefulShutdown,
            fossil_path: PathBuf::from("/tmp/fossil"),
            issue_count: 0,
            encrypted: false,
        });
        let temp = tempfile::tempdir().expect("temp dir");
        let path = temp.path().join("index.toml");
        index.save(&path).expect("save");
        let loaded = FossilIndex::load(&path).expect("load");
        assert_eq!(loaded.fossils.len(), 1);
        assert_eq!(loaded.fossils[0].node_id, "node-save");
    }

    #[test]
    fn test_archival_reason_variants() {
        let _ = format!("{:?}", ArchivalReason::GracefulShutdown);
        let _ = format!("{:?}", ArchivalReason::Crash { exit_code: 1 });
        let _ = format!("{:?}", ArchivalReason::Manual);
        let _ = format!("{:?}", ArchivalReason::AutomaticRotation);
        let _ = format!("{:?}", ArchivalReason::Redeployment);
        let _ = format!("{:?}", ArchivalReason::Reboot);
    }

    #[test]
    fn test_archival_reason_serde() {
        let reason = ArchivalReason::Crash { exit_code: 1 };
        let json = serde_json::to_string(&reason).expect("serialize");
        let back: ArchivalReason = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(reason, back);
    }

    #[test]
    fn test_fossil_record_from_active_session() {
        let session = ActiveLogSession::new("node-1".into(), "deploy-1".into());
        let record = FossilRecord::from_active_session(&session, ArchivalReason::Manual);
        assert_eq!(record.node_id, "node-1");
        assert_eq!(record.deployment_id, "deploy-1");
        assert_eq!(record.archival_reason, ArchivalReason::Manual);
        assert!(record.duration().num_seconds() >= 0);
    }

    #[test]
    fn test_fossil_record_issue_count() {
        let record = FossilRecord {
            node_id: "n".into(),
            session_started: Utc::now() - chrono::Duration::seconds(60),
            session_ended: Utc::now(),
            archival_reason: ArchivalReason::GracefulShutdown,
            deployment_id: "d".into(),
            issues: vec![
                LogIssue {
                    timestamp: Utc::now(),
                    severity: IssueSeverity::Error,
                    primal: "p".into(),
                    description: "err".into(),
                    log_line: None,
                },
                LogIssue {
                    timestamp: Utc::now(),
                    severity: IssueSeverity::Warning,
                    primal: "p".into(),
                    description: "warn".into(),
                    log_line: None,
                },
            ],
            metrics: None,
            encrypted: false,
            parent_seed_fingerprint: None,
        };
        assert_eq!(record.issue_count(None), 2);
        assert_eq!(record.issue_count(Some(IssueSeverity::Error)), 1);
        assert_eq!(record.issue_count(Some(IssueSeverity::Warning)), 1);
    }

    #[test]
    fn test_fossil_index_default() {
        let index = FossilIndex::default();
        assert!(index.fossils.is_empty());
    }
}
