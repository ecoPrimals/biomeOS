// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Active log session and file management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

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
            PathBuf::from(format!("/proc/{pid}")).exists()
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_new_session() {
        let session = ActiveLogSession::new("node-1".into(), "deploy-1".into());
        assert_eq!(session.node_id, "node-1");
        assert!(session.process_pids.is_empty());
    }

    #[test]
    fn test_add_process() {
        let mut session = ActiveLogSession::new("node-1".into(), "deploy-1".into());
        session.add_process(1234);
        session.add_process(1234); // Duplicate
        assert_eq!(session.process_pids.len(), 1);
    }

    #[test]
    fn test_add_log_file() {
        let mut session = ActiveLogSession::new("node-1".into(), "deploy-1".into());
        session.add_log_file(LogFile {
            primal: "beardog".into(),
            path: PathBuf::from("/tmp/beardog.log"),
            pid: Some(1234),
            size_bytes: 0,
            last_modified: Utc::now(),
        });
        assert_eq!(session.log_files.len(), 1);
        assert_eq!(session.log_files[0].primal, "beardog");
    }

    #[test]
    fn test_is_active_no_pids() {
        let session = ActiveLogSession::new("node-1".into(), "deploy-1".into());
        assert!(!session.is_active());
    }

    #[test]
    fn test_is_active_with_self_pid() {
        let mut session = ActiveLogSession::new("node-1".into(), "deploy-1".into());
        session.add_process(std::process::id());
        assert!(session.is_active());
    }

    #[test]
    fn test_duration() {
        let session = ActiveLogSession::new("node-1".into(), "deploy-1".into());
        let d = session.duration();
        assert!(d.num_seconds() >= 0);
    }

    #[test]
    fn test_log_file_refresh() {
        let temp = tempfile::tempdir().expect("temp dir");
        let log_path = temp.path().join("test.log");
        std::fs::write(&log_path, "hello world").expect("write");
        let mut log_file = LogFile {
            primal: "test".into(),
            path: log_path,
            pid: None,
            size_bytes: 0,
            last_modified: Utc::now(),
        };
        log_file.refresh().expect("refresh");
        assert_eq!(log_file.size_bytes, 11);
    }

    #[test]
    fn test_log_file_serialization() {
        let lf = LogFile {
            primal: "p".into(),
            path: PathBuf::from("/tmp/x.log"),
            pid: Some(1),
            size_bytes: 100,
            last_modified: Utc::now(),
        };
        let json = serde_json::to_string(&lf).expect("serialize");
        let back: LogFile = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.primal, "p");
        assert_eq!(back.size_bytes, 100);
    }
}
