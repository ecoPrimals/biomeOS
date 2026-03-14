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
}
