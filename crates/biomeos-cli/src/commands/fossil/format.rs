// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Formatting, cleanup planning, and log scanning for fossil commands.

use anyhow::Result;
use biomeos_spore::logs::{ActiveLogSession, FossilIndexEntry};
use std::path::{Path, PathBuf};

/// Result of computing a cleanup plan for old fossils.
#[derive(Debug, Clone, Default)]
pub struct CleanupPlan {
    /// Paths that would be or were removed
    pub to_remove: Vec<PathBuf>,
    /// Total bytes that would be or were freed
    pub freed_bytes: u64,
    /// Number of fossils in the plan
    pub count: usize,
}

/// Filters sessions by optional node ID. Returns sessions matching the filter.
pub fn filter_sessions<'a>(
    sessions: &'a [ActiveLogSession],
    node_filter: Option<&str>,
) -> Vec<&'a ActiveLogSession> {
    sessions
        .iter()
        .filter(|s| node_filter.is_none_or(|n| s.node_id.contains(n)))
        .collect()
}

/// Builds display lines for a single session.
pub fn format_session_display(session: &ActiveLogSession) -> Vec<String> {
    let mut lines = Vec::new();
    let duration = session.duration();
    let hours = duration.num_hours();
    let mins = duration.num_minutes() % 60;

    lines.push(format!("Node: {}", session.node_id));
    lines.push(format!(
        "  Started: {} ({}h {}m ago)",
        session.started_at.format("%Y-%m-%d %H:%M:%S"),
        hours,
        mins
    ));

    if !session.process_pids.is_empty() {
        let pids: Vec<String> = session
            .process_pids
            .iter()
            .map(std::string::ToString::to_string)
            .collect();
        lines.push(format!("  PIDs: {}", pids.join(" ")));
    }

    if !session.log_files.is_empty() {
        lines.push("  Logs:".to_string());
        for log_file in &session.log_files {
            let size_kb = log_file.size_bytes / 1024;
            let status = if log_file.pid.is_some() {
                "active"
            } else {
                "closed"
            };
            lines.push(format!(
                "    • {:<15} ({} KB, {})",
                format!("{}.log", log_file.primal),
                size_kb,
                status
            ));
        }
    }

    lines
}

/// Builds display lines for fossil detail view.
pub fn format_fossil_detail(fossil: &FossilIndexEntry) -> Vec<String> {
    vec![
        format!("Node: {}", fossil.node_id),
        format!(
            "Session: {}",
            fossil.session_started.format("%Y-%m-%d %H:%M:%S")
        ),
        format!("Reason: {:?}", fossil.archival_reason),
        format!("Path: {}", fossil.fossil_path.display()),
        format!("Issues: {}", fossil.issue_count),
        format!("Encrypted: {}", if fossil.encrypted { "Yes" } else { "No" }),
    ]
}

/// Computes which fossils to remove based on cutoff. Does not perform IO.
pub fn compute_cleanup_plan(
    fossils: &[FossilIndexEntry],
    cutoff: chrono::DateTime<chrono::Utc>,
) -> CleanupPlan {
    let mut to_remove = Vec::new();
    let mut freed_bytes: u64 = 0;

    for fossil in fossils {
        if fossil.session_started < cutoff && fossil.fossil_path.exists() {
            if let Ok(metadata) = std::fs::metadata(&fossil.fossil_path) {
                freed_bytes += metadata.len();
            }
            to_remove.push(fossil.fossil_path.clone());
        }
    }

    CleanupPlan {
        count: to_remove.len(),
        to_remove,
        freed_bytes,
    }
}

/// Scans a directory for .log files. Returns paths to log files found.
pub fn scan_old_logs(from: &Path) -> Result<Vec<PathBuf>> {
    let mut old_logs = Vec::new();

    if !from.exists() {
        return Ok(old_logs);
    }

    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().is_some_and(|ext| ext == "log") {
            old_logs.push(path);
        }
    }

    Ok(old_logs)
}
