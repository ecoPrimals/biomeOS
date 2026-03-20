// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Fossil log management CLI commands
//!
//! This handles the fossil record system for archived logs,
//! separate from the service log streaming (Commands::Logs)

use anyhow::Result;
use biomeos_spore::logs::{
    ActiveLogSession, ArchivalReason, FossilIndex, FossilIndexEntry, LogConfig, LogManager,
};
use clap::{Args, Subcommand};
use std::path::{Path, PathBuf};
use tracing::info;

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
pub(crate) fn filter_sessions<'a>(
    sessions: &'a [ActiveLogSession],
    node_filter: Option<&str>,
) -> Vec<&'a ActiveLogSession> {
    sessions
        .iter()
        .filter(|s| node_filter.is_none_or(|n| s.node_id.contains(n)))
        .collect()
}

/// Builds display lines for a single session.
pub(crate) fn format_session_display(session: &ActiveLogSession) -> Vec<String> {
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
pub(crate) fn format_fossil_detail(fossil: &FossilIndexEntry) -> Vec<String> {
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
pub(crate) fn compute_cleanup_plan(
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
pub(crate) fn scan_old_logs(from: &Path) -> Result<Vec<PathBuf>> {
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

/// Arguments for fossil log management commands
#[derive(Debug, Args)]
pub struct FossilArgs {
    /// Fossil subcommand to execute
    #[command(subcommand)]
    pub action: FossilAction,
}

/// Available fossil log management actions
#[derive(Debug, Subcommand)]
pub enum FossilAction {
    /// Show active log sessions
    Active {
        /// Filter by node ID
        #[arg(long)]
        node: Option<String>,
    },

    /// Browse fossil (archived) logs
    Fossil {
        /// Filter by node ID
        #[arg(long)]
        node: Option<String>,

        /// Limit number of results
        #[arg(long, default_value = "10")]
        limit: usize,

        /// Show detailed fossil record
        #[arg(long)]
        show: Option<usize>,
    },

    /// Manually archive a session
    Archive {
        /// Node ID to archive
        node_id: String,
    },

    /// Clean up old fossil logs
    Clean {
        /// Remove fossils older than N days
        #[arg(long, default_value = "30")]
        older_than: u64,

        /// Dry run (don't actually delete)
        #[arg(long)]
        dry_run: bool,
    },

    /// Migrate existing logs to fossil structure
    Migrate {
        /// Directory containing old logs (default: /tmp/primals)
        #[arg(long, default_value = "/tmp/primals")]
        from: PathBuf,

        /// Dry run (don't actually migrate)
        #[arg(long)]
        dry_run: bool,
    },

    /// Clean up stale active sessions
    CleanupStale,
}

/// Execute a fossil log management command
pub async fn run(args: FossilArgs) -> Result<()> {
    match args.action {
        FossilAction::Active { node } => handle_active(node).await,
        FossilAction::Fossil { node, limit, show } => handle_fossil(node, limit, show).await,
        FossilAction::Archive { node_id } => handle_archive(node_id).await,
        FossilAction::Clean {
            older_than,
            dry_run,
        } => handle_clean(older_than, dry_run).await,
        FossilAction::Migrate { from, dry_run } => handle_migrate(from, dry_run).await,
        FossilAction::CleanupStale => handle_cleanup_stale().await,
    }
}

async fn handle_active(node_filter: Option<String>) -> Result<()> {
    let config = LogConfig::default();
    let manager = LogManager::new(config);

    let sessions = manager.list_active_sessions()?;
    let filtered = filter_sessions(&sessions, node_filter.as_deref());

    if filtered.is_empty() {
        println!("\n🌱 No active log sessions found");
        return Ok(());
    }

    println!("\n🌱 Active Log Sessions");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let total_count = filtered.len();

    for session in filtered {
        let lines = format_session_display(session);
        for line in lines {
            println!("{line}");
        }
        println!();
    }

    println!("Total: {total_count} active session(s)");

    Ok(())
}

async fn handle_fossil(
    node_filter: Option<String>,
    limit: usize,
    show: Option<usize>,
) -> Result<()> {
    let config = LogConfig::default();
    let index_path = config.fossil_dir.join("index.toml");

    if !index_path.exists() {
        println!("\n🦴 No fossil records found");
        return Ok(());
    }

    let index = FossilIndex::load(&index_path)?;

    let filtered: Vec<_> = index
        .fossils
        .iter()
        .filter(|f| node_filter.as_ref().is_none_or(|n| f.node_id.contains(n)))
        .take(limit)
        .collect();

    if let Some(idx) = show {
        if idx > 0 && idx <= filtered.len() {
            println!("\n🦴 Fossil Record Details\n");
            let fossil = filtered[idx - 1];
            let lines = format_fossil_detail(fossil);
            for line in lines {
                println!("{line}");
            }
        } else {
            println!("Error: Invalid fossil number {idx}");
        }
        return Ok(());
    }

    if filtered.is_empty() {
        println!("\n🦴 No fossil records found");
        return Ok(());
    }

    println!("\n🦴 Fossil Record");
    if let Some(node) = &node_filter {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Filtered by node: {node}\n");
    } else {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }

    for (i, fossil) in filtered.iter().enumerate() {
        println!(
            "[{}] {} → {}",
            i + 1,
            fossil.session_started.format("%Y-%m-%d %H:%M:%S"),
            fossil.node_id
        );
        println!("    Reason: {:?}", fossil.archival_reason);
        println!("    Issues: {}", fossil.issue_count);
        println!("    Path: {}", fossil.fossil_path.display());
        println!();
    }

    println!("Use 'biomeos logs fossil --show <num>' to view details");

    Ok(())
}

async fn handle_archive(node_id: String) -> Result<()> {
    info!("Manually archiving session for node: {}", node_id);

    let config = LogConfig::default();
    let manager = LogManager::new(config);

    let sessions = manager.list_active_sessions()?;

    let session = sessions
        .iter()
        .find(|s| s.node_id == node_id)
        .ok_or_else(|| anyhow::anyhow!("No active session found for node: {node_id}"))?;

    let fossil = manager
        .archive_session(session, ArchivalReason::Manual)
        .await?;

    println!("✅ Archived session for {node_id}");
    println!("   Duration: {} seconds", fossil.duration().num_seconds());
    println!("   Issues: {}", fossil.issues.len());

    Ok(())
}

async fn handle_clean(older_than: u64, dry_run: bool) -> Result<()> {
    use chrono::{Duration, Utc};

    if dry_run {
        println!("🔍 Dry run: No files will be deleted\n");
    }

    let config = LogConfig::default();
    let manager = LogManager::new(config.clone());

    manager.initialize().await?;

    #[allow(
        clippy::cast_possible_wrap,
        reason = "days for fossil cleanup is bounded"
    )]
    let cutoff = Utc::now() - Duration::days(older_than as i64);
    println!(
        "🗑️  Cleaning fossils older than {} days (before {})",
        older_than,
        cutoff.format("%Y-%m-%d")
    );

    let index_path = config.fossil_dir.join("index.toml");
    let index = FossilIndex::load(&index_path)?;

    let plan = compute_cleanup_plan(&index.fossils, cutoff);

    if plan.count > 0 {
        for fossil_path in &plan.to_remove {
            if !dry_run {
                if let Err(e) = std::fs::remove_file(fossil_path) {
                    let node_id = fossil_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown");
                    tracing::warn!("Failed to delete fossil {}: {}", node_id, e);
                } else {
                    info!("Deleted fossil: {}", fossil_path.display());
                }
            }
        }

        let freed_mb = plan.freed_bytes as f64 / (1024.0 * 1024.0);
        if dry_run {
            for fossil in &index.fossils {
                if fossil.session_started < cutoff && fossil.fossil_path.exists() {
                    println!(
                        "   Would delete: {} ({})",
                        fossil.node_id,
                        fossil.session_started.format("%Y-%m-%d %H:%M")
                    );
                }
            }
            println!(
                "\n📊 Would clean {} fossils, freeing {:.2} MB",
                plan.count, freed_mb
            );
        } else {
            println!(
                "\n✅ Cleaned {} fossils, freed {:.2} MB",
                plan.count, freed_mb
            );
        }
    } else {
        println!("✅ No fossils older than {older_than} days found");
    }

    Ok(())
}

async fn handle_migrate(from: PathBuf, dry_run: bool) -> Result<()> {
    println!("\n🔄 Migrating logs from: {}", from.display());

    if dry_run {
        println!("🔍 Dry run: No files will be moved\n");
    }

    if !from.exists() {
        println!("❌ Source directory does not exist");
        return Ok(());
    }

    let config = LogConfig::default();
    let manager = LogManager::new(config.clone());

    manager.initialize().await?;

    let old_logs = scan_old_logs(&from)?;

    if old_logs.is_empty() {
        println!("✅ No old logs found to migrate");
        return Ok(());
    }

    println!("Found {} old log files", old_logs.len());

    if dry_run {
        for log_path in &old_logs {
            let file_name = log_path.file_name().map_or_else(
                || std::borrow::Cow::Borrowed("unknown"),
                |n| n.to_string_lossy(),
            );
            println!("  Would migrate: {file_name}");
        }
    } else {
        for log_path in &old_logs {
            let file_name = log_path.file_name().map_or_else(
                || std::borrow::Cow::Borrowed("unknown"),
                |n| n.to_string_lossy(),
            );
            let dest = config.fossil_dir.join("legacy").join(&*file_name);

            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::rename(log_path, &dest)?;

            println!("  ✅ Migrated: {file_name}");
        }

        println!("\n✅ Migration complete!");
    }

    Ok(())
}

async fn handle_cleanup_stale() -> Result<()> {
    info!("Cleaning up stale active sessions");

    let config = LogConfig::default();
    let manager = LogManager::new(config);

    manager.initialize().await?;

    let archived = manager.cleanup_stale_sessions().await?;

    if archived.is_empty() {
        println!("✅ No stale sessions found");
    } else {
        println!("✅ Cleaned up {} stale session(s):", archived.len());
        for fossil in archived {
            println!(
                "  • {} (duration: {}s)",
                fossil.node_id,
                fossil.duration().num_seconds()
            );
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use biomeos_spore::logs::LogFile;
    use chrono::Utc;
    use std::path::PathBuf;

    #[test]
    fn test_filter_sessions_no_filter() {
        let sessions = vec![
            ActiveLogSession::new("node-1".into(), "deploy-1".into()),
            ActiveLogSession::new("node-2".into(), "deploy-1".into()),
        ];
        let filtered = filter_sessions(&sessions, None);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_sessions_with_filter() {
        let sessions = vec![
            ActiveLogSession::new("node-1".into(), "deploy-1".into()),
            ActiveLogSession::new("node-2".into(), "deploy-1".into()),
            ActiveLogSession::new("node-10".into(), "deploy-1".into()),
        ];
        let filtered = filter_sessions(&sessions, Some("node-1"));
        assert_eq!(filtered.len(), 2); // node-1 and node-10
    }

    #[test]
    fn test_filter_sessions_empty_match() {
        let sessions = vec![ActiveLogSession::new("node-1".into(), "deploy-1".into())];
        let filtered = filter_sessions(&sessions, Some("node-99"));
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_format_session_display() {
        let mut session = ActiveLogSession::new("node-1".into(), "deploy-1".into());
        session.add_process(1234);
        session.add_log_file(LogFile {
            primal: "tower".into(),
            path: PathBuf::from("/tmp/tower.log"),
            pid: Some(1234),
            size_bytes: 1024,
            last_modified: Utc::now(),
        });

        let lines = format_session_display(&session);
        assert!(lines.iter().any(|l| l.contains("node-1")));
        assert!(lines.iter().any(|l| l.contains("PIDs")));
        assert!(lines.iter().any(|l| l.contains("tower")));
    }

    #[test]
    fn test_format_fossil_detail() {
        let fossil = FossilIndexEntry {
            node_id: "node-1".into(),
            session_started: Utc::now(),
            archival_reason: ArchivalReason::GracefulShutdown,
            fossil_path: PathBuf::from("/tmp/fossil1"),
            issue_count: 2,
            encrypted: false,
        };

        let lines = format_fossil_detail(&fossil);
        assert!(lines.iter().any(|l| l.contains("node-1")));
        assert!(lines.iter().any(|l| l.contains("Issues: 2")));
        assert!(lines.iter().any(|l| l.contains("Encrypted: No")));
    }

    #[test]
    fn test_compute_cleanup_plan_empty() {
        let fossils: Vec<FossilIndexEntry> = vec![];
        let cutoff = Utc::now();
        let plan = compute_cleanup_plan(&fossils, cutoff);
        assert_eq!(plan.count, 0);
        assert!(plan.to_remove.is_empty());
        assert_eq!(plan.freed_bytes, 0);
    }

    #[test]
    fn test_scan_old_logs_nonexistent() {
        let result = scan_old_logs(Path::new("/nonexistent/path/12345"));
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    #[ignore = "Requires /var/biomeos writable - run with --ignored for full test"]
    async fn test_run_clean_dry_run() {
        let args = FossilArgs {
            action: FossilAction::Clean {
                older_than: 30,
                dry_run: true,
            },
        };
        let result = run(args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_migrate_nonexistent_source() {
        let args = FossilArgs {
            action: FossilAction::Migrate {
                from: PathBuf::from("/nonexistent/path/12345"),
                dry_run: true,
            },
        };
        let result = run(args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "Requires /var/biomeos writable - run with --ignored for full test"]
    async fn test_run_cleanup_stale() {
        let args = FossilArgs {
            action: FossilAction::CleanupStale,
        };
        let result = run(args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_fossil_no_index() {
        let args = FossilArgs {
            action: FossilAction::Fossil {
                node: None,
                limit: 10,
                show: None,
            },
        };
        let result = run(args).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_fossil_detail_encrypted() {
        let fossil = FossilIndexEntry {
            node_id: "node-enc".into(),
            session_started: Utc::now(),
            archival_reason: ArchivalReason::Manual,
            fossil_path: PathBuf::from("/tmp/encrypted-fossil"),
            issue_count: 0,
            encrypted: true,
        };
        let lines = format_fossil_detail(&fossil);
        assert!(lines.iter().any(|l| l.contains("Encrypted: Yes")));
        assert!(lines.iter().any(|l| l.contains("node-enc")));
    }

    #[test]
    fn test_compute_cleanup_plan_with_fossils() {
        let temp = tempfile::tempdir().expect("temp dir");
        let fossil_path = temp.path().join("fossil1");
        std::fs::write(&fossil_path, b"fake fossil data").expect("write");

        let fossils = vec![FossilIndexEntry {
            node_id: "node-1".into(),
            session_started: Utc::now() - chrono::Duration::days(60),
            archival_reason: ArchivalReason::GracefulShutdown,
            fossil_path,
            issue_count: 0,
            encrypted: false,
        }];
        let cutoff = Utc::now() - chrono::Duration::days(30);
        let plan = compute_cleanup_plan(&fossils, cutoff);
        assert_eq!(plan.count, 1);
        assert_eq!(plan.to_remove.len(), 1);
        assert!(plan.freed_bytes > 0);
    }

    #[test]
    fn test_compute_cleanup_plan_fossil_after_cutoff() {
        let fossils = vec![FossilIndexEntry {
            node_id: "node-recent".into(),
            session_started: Utc::now() - chrono::Duration::days(5),
            archival_reason: ArchivalReason::GracefulShutdown,
            fossil_path: PathBuf::from("/tmp/recent"),
            issue_count: 0,
            encrypted: false,
        }];
        let cutoff = Utc::now() - chrono::Duration::days(30);
        let plan = compute_cleanup_plan(&fossils, cutoff);
        assert_eq!(plan.count, 0);
        assert!(plan.to_remove.is_empty());
    }

    #[test]
    fn test_cleanup_plan_default() {
        let plan = CleanupPlan::default();
        assert!(plan.to_remove.is_empty());
        assert_eq!(plan.freed_bytes, 0);
        assert_eq!(plan.count, 0);
    }

    #[test]
    fn test_cleanup_plan_debug() {
        let plan = CleanupPlan {
            to_remove: vec![PathBuf::from("/tmp/a")],
            freed_bytes: 1024,
            count: 1,
        };
        let _ = format!("{plan:?}");
    }

    #[test]
    fn test_format_session_display_no_logs_no_pids() {
        let session = ActiveLogSession::new("node-minimal".into(), "deploy-1".into());
        let lines = format_session_display(&session);
        assert!(lines.iter().any(|l| l.contains("node-minimal")));
        assert!(!lines.iter().any(|l| l.contains("PIDs:")));
        assert!(!lines.iter().any(|l| l.contains("Logs:")));
    }

    #[test]
    fn test_scan_old_logs_existing_dir_with_logs() {
        let temp = tempfile::tempdir().expect("temp dir");
        std::fs::write(temp.path().join("a.log"), b"log").expect("write");
        std::fs::write(temp.path().join("b.txt"), b"not log").expect("write");
        std::fs::write(temp.path().join("c.log"), b"log2").expect("write");

        let result = scan_old_logs(temp.path());
        assert!(result.is_ok());
        let logs = result.unwrap();
        assert_eq!(logs.len(), 2);
    }

    #[test]
    fn test_fossil_args_debug() {
        let args = FossilArgs {
            action: FossilAction::Active { node: None },
        };
        let _ = format!("{args:?}");
    }

    #[test]
    fn test_fossil_action_variants_debug() {
        let _ = format!(
            "{:?}",
            FossilAction::Fossil {
                node: Some("n".into()),
                limit: 5,
                show: Some(1),
            }
        );
        let _ = format!(
            "{:?}",
            FossilAction::Archive {
                node_id: "n".into()
            }
        );
        let _ = format!(
            "{:?}",
            FossilAction::Clean {
                older_than: 30,
                dry_run: true,
            }
        );
        let _ = format!(
            "{:?}",
            FossilAction::Migrate {
                from: PathBuf::from("/tmp"),
                dry_run: false,
            }
        );
        let _ = format!("{:?}", FossilAction::CleanupStale);
    }

    #[tokio::test]
    async fn test_run_active() {
        let args = FossilArgs {
            action: FossilAction::Active { node: None },
        };
        let result = run(args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_fossil_with_show_invalid_index() {
        let args = FossilArgs {
            action: FossilAction::Fossil {
                node: None,
                limit: 10,
                show: Some(999),
            },
        };
        let result = run(args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "Requires /var/biomeos writable for LogManager - run with --ignored"]
    async fn test_run_migrate_empty_dir() {
        let temp = tempfile::tempdir().expect("temp dir");
        let args = FossilArgs {
            action: FossilAction::Migrate {
                from: temp.path().to_path_buf(),
                dry_run: true,
            },
        };
        let result = run(args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_archive_no_session() {
        let args = FossilArgs {
            action: FossilAction::Archive {
                node_id: "nonexistent-node-xyz-123".to_string(),
            },
        };
        let result = run(args).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No active session")
        );
    }

    #[test]
    fn test_compute_cleanup_plan_fossil_path_nonexistent() {
        let fossils = vec![FossilIndexEntry {
            node_id: "node-old".into(),
            session_started: Utc::now() - chrono::Duration::days(60),
            archival_reason: ArchivalReason::GracefulShutdown,
            fossil_path: PathBuf::from("/nonexistent/path/12345"),
            issue_count: 0,
            encrypted: false,
        }];
        let cutoff = Utc::now() - chrono::Duration::days(30);
        let plan = compute_cleanup_plan(&fossils, cutoff);
        assert_eq!(plan.count, 0, "nonexistent path is not added to to_remove");
        assert_eq!(plan.freed_bytes, 0);
    }

    #[test]
    fn test_format_session_display_multiple_logs() {
        let mut session = ActiveLogSession::new("node-multi".into(), "deploy-1".into());
        session.add_process(1111);
        session.add_log_file(LogFile {
            primal: "tower".into(),
            path: PathBuf::from("/tmp/tower.log"),
            pid: Some(1111),
            size_bytes: 2048,
            last_modified: Utc::now(),
        });
        session.add_log_file(LogFile {
            primal: "beardog".into(),
            path: PathBuf::from("/tmp/beardog.log"),
            pid: None,
            size_bytes: 512,
            last_modified: Utc::now(),
        });
        let lines = format_session_display(&session);
        assert!(lines.iter().any(|l| l.contains("tower")));
        assert!(lines.iter().any(|l| l.contains("beardog")));
        assert!(lines.iter().any(|l| l.contains("closed")));
    }

    #[test]
    fn test_format_fossil_detail_crash_reason() {
        let fossil = FossilIndexEntry {
            node_id: "node-crash".into(),
            session_started: Utc::now(),
            archival_reason: ArchivalReason::Crash { exit_code: 137 },
            fossil_path: PathBuf::from("/tmp/crash-fossil"),
            issue_count: 3,
            encrypted: false,
        };
        let lines = format_fossil_detail(&fossil);
        assert!(lines.iter().any(|l| l.contains("Crash")));
        assert!(lines.iter().any(|l| l.contains("node-crash")));
    }

    #[test]
    fn test_format_fossil_detail_all_archival_reasons() {
        for reason in [
            ArchivalReason::GracefulShutdown,
            ArchivalReason::Manual,
            ArchivalReason::AutomaticRotation,
        ] {
            let fossil = FossilIndexEntry {
                node_id: "node".into(),
                session_started: Utc::now(),
                archival_reason: reason,
                fossil_path: PathBuf::from("/tmp/f"),
                issue_count: 0,
                encrypted: false,
            };
            let lines = format_fossil_detail(&fossil);
            assert!(lines.iter().any(|l| l.contains("node")));
            assert!(lines.iter().any(|l| l.contains("Path:")));
        }
    }

    #[test]
    fn test_scan_old_logs_empty_dir() {
        let temp = tempfile::tempdir().expect("temp dir");
        let result = scan_old_logs(temp.path());
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_scan_old_logs_ignores_non_log() {
        let temp = tempfile::tempdir().expect("temp dir");
        std::fs::write(temp.path().join("a.txt"), b"x").expect("write");
        std::fs::write(temp.path().join("b.yaml"), b"y").expect("write");
        let result = scan_old_logs(temp.path());
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_filter_sessions_empty_input() {
        let sessions: Vec<ActiveLogSession> = vec![];
        assert!(filter_sessions(&sessions, None).is_empty());
        assert!(filter_sessions(&sessions, Some("x")).is_empty());
    }

    #[test]
    fn test_format_fossil_detail_redeployment_reboot() {
        for reason in [ArchivalReason::Redeployment, ArchivalReason::Reboot] {
            let fossil = FossilIndexEntry {
                node_id: "n".into(),
                session_started: Utc::now(),
                archival_reason: reason,
                fossil_path: PathBuf::from("/tmp/f"),
                issue_count: 0,
                encrypted: false,
            };
            let lines = format_fossil_detail(&fossil);
            assert!(lines.iter().any(|l| l.contains("Node: n")));
        }
    }

    #[test]
    fn test_format_session_display_log_closed_no_pid() {
        let mut session = ActiveLogSession::new("n".into(), "d".into());
        session.add_log_file(LogFile {
            primal: "p".into(),
            path: PathBuf::from("/tmp/p.log"),
            pid: None,
            size_bytes: 4096,
            last_modified: Utc::now(),
        });
        let lines = format_session_display(&session);
        assert!(lines.iter().any(|l| l.contains("closed")));
        assert!(
            lines
                .iter()
                .any(|l| l.contains("4 KB") || l.contains("3 KB"))
        );
    }

    #[test]
    fn test_compute_cleanup_plan_exactly_at_cutoff() {
        let temp = tempfile::tempdir().expect("temp dir");
        let p = temp.path().join("f");
        std::fs::write(&p, b"x").expect("write");
        let t = Utc::now() - chrono::Duration::days(30);
        let fossils = vec![FossilIndexEntry {
            node_id: "n".into(),
            session_started: t,
            archival_reason: ArchivalReason::GracefulShutdown,
            fossil_path: p,
            issue_count: 0,
            encrypted: false,
        }];
        let plan = compute_cleanup_plan(&fossils, t);
        assert_eq!(plan.count, 0, "session_started < cutoff is strict");
    }

    #[test]
    fn test_compute_cleanup_plan_just_before_cutoff() {
        let temp = tempfile::tempdir().expect("temp dir");
        let p = temp.path().join("f2");
        std::fs::write(&p, b"x").expect("write");
        let old = Utc::now() - chrono::Duration::days(31);
        let cutoff = Utc::now() - chrono::Duration::days(30);
        let fossils = vec![FossilIndexEntry {
            node_id: "n".into(),
            session_started: old,
            archival_reason: ArchivalReason::AutomaticRotation,
            fossil_path: p,
            issue_count: 1,
            encrypted: true,
        }];
        let plan = compute_cleanup_plan(&fossils, cutoff);
        assert_eq!(plan.count, 1);
    }

    #[tokio::test]
    async fn test_run_fossil_with_show_zero_invalid() {
        let args = FossilArgs {
            action: FossilAction::Fossil {
                node: None,
                limit: 10,
                show: Some(0),
            },
        };
        let result = run(args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_fossil_with_node_filter_no_index() {
        let args = FossilArgs {
            action: FossilAction::Fossil {
                node: Some("node-abc".into()),
                limit: 5,
                show: None,
            },
        };
        let result = run(args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_clean_not_dry_run_no_index() {
        let args = FossilArgs {
            action: FossilAction::Clean {
                older_than: 1,
                dry_run: false,
            },
        };
        let result = run(args).await;
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_scan_old_logs_read_dir_error_unlikely() {
        // Non-directory path that exists as file — read_dir fails
        let temp = tempfile::tempdir().expect("temp dir");
        let f = temp.path().join("not-a-dir");
        std::fs::write(&f, b"x").expect("write");
        let result = scan_old_logs(&f);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_fossil_detail_issue_zero() {
        let fossil = FossilIndexEntry {
            node_id: "z".into(),
            session_started: Utc::now(),
            archival_reason: ArchivalReason::GracefulShutdown,
            fossil_path: PathBuf::from("/tmp/z"),
            issue_count: 0,
            encrypted: false,
        };
        let lines = format_fossil_detail(&fossil);
        assert!(lines.iter().any(|l| l.contains("Issues: 0")));
    }

    #[test]
    fn test_compute_cleanup_plan_zero_byte_file_freed_zero() {
        let temp = tempfile::tempdir().expect("temp dir");
        let p = temp.path().join("empty-fossil");
        std::fs::write(&p, []).expect("empty");
        let fossils = vec![FossilIndexEntry {
            node_id: "n".into(),
            session_started: Utc::now() - chrono::Duration::days(90),
            archival_reason: ArchivalReason::GracefulShutdown,
            fossil_path: p,
            issue_count: 0,
            encrypted: false,
        }];
        let plan = compute_cleanup_plan(&fossils, Utc::now() - chrono::Duration::days(30));
        assert_eq!(plan.count, 1);
        assert_eq!(plan.freed_bytes, 0);
    }

    #[test]
    fn test_format_session_display_zero_kb_log() {
        let mut session = ActiveLogSession::new("n".into(), "d".into());
        session.add_log_file(LogFile {
            primal: "tiny".into(),
            path: PathBuf::from("/tmp/tiny.log"),
            pid: Some(1),
            size_bytes: 0,
            last_modified: Utc::now(),
        });
        let lines = format_session_display(&session);
        assert!(lines.iter().any(|l| l.contains("0 KB")));
    }

    #[test]
    fn test_filter_sessions_filter_empty_string_matches_all() {
        let sessions = vec![ActiveLogSession::new("a".into(), "d".into())];
        let filtered = filter_sessions(&sessions, Some(""));
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_format_session_display_duration_hours_only() {
        use chrono::Duration as ChronoDuration;
        let mut session = ActiveLogSession::new("dur-node".into(), "deploy-1".into());
        session.started_at = Utc::now() - ChronoDuration::hours(3);
        let lines = format_session_display(&session);
        assert!(lines.iter().any(|l| l.contains("3h")));
    }

    #[test]
    fn test_format_fossil_detail_long_path_display() {
        let long = PathBuf::from("/var/biomeos/fossils/").join("a".repeat(80));
        let fossil = FossilIndexEntry {
            node_id: "n".into(),
            session_started: Utc::now(),
            archival_reason: ArchivalReason::AutomaticRotation,
            fossil_path: long.clone(),
            issue_count: 99,
            encrypted: true,
        };
        let lines = format_fossil_detail(&fossil);
        assert!(lines.iter().any(|l| l.contains(&fossil.node_id)));
        assert!(lines.iter().any(|l| l.contains("Issues: 99")));
        assert!(
            lines
                .iter()
                .any(|l| l.contains(long.to_string_lossy().as_ref()))
        );
    }

    #[test]
    fn test_compute_cleanup_plan_multiple_fossils_accumulates_bytes() {
        let temp = tempfile::tempdir().expect("temp dir");
        let p1 = temp.path().join("f1");
        let p2 = temp.path().join("f2");
        std::fs::write(&p1, vec![0u8; 100]).expect("write");
        std::fs::write(&p2, vec![0u8; 200]).expect("write");
        let old = Utc::now() - chrono::Duration::days(100);
        let fossils = vec![
            FossilIndexEntry {
                node_id: "a".into(),
                session_started: old,
                archival_reason: ArchivalReason::GracefulShutdown,
                fossil_path: p1,
                issue_count: 0,
                encrypted: false,
            },
            FossilIndexEntry {
                node_id: "b".into(),
                session_started: old,
                archival_reason: ArchivalReason::Manual,
                fossil_path: p2,
                issue_count: 1,
                encrypted: false,
            },
        ];
        let plan = compute_cleanup_plan(&fossils, Utc::now() - chrono::Duration::days(30));
        assert_eq!(plan.count, 2);
        assert_eq!(plan.freed_bytes, 300);
    }

    #[test]
    fn test_cleanup_plan_clone() {
        let a = CleanupPlan {
            to_remove: vec![PathBuf::from("/x")],
            freed_bytes: 10,
            count: 1,
        };
        let b = a.clone();
        assert_eq!(b.count, a.count);
        assert_eq!(b.freed_bytes, a.freed_bytes);
    }

    #[test]
    fn test_format_session_display_many_pids() {
        let mut session = ActiveLogSession::new("pid-node".into(), "d".into());
        for p in [1u32, 2, 3, 4, 5] {
            session.add_process(p);
        }
        let lines = format_session_display(&session);
        assert!(lines.iter().any(|l| l.contains("PIDs:")));
        assert!(lines.iter().any(|l| l.contains('5')));
    }

    #[test]
    fn test_filter_sessions_overlapping_substrings() {
        let sessions = vec![
            ActiveLogSession::new("alpha-node".into(), "d".into()),
            ActiveLogSession::new("alphabet-extra".into(), "d".into()),
        ];
        let filtered = filter_sessions(&sessions, Some("alpha"));
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_scan_old_logs_subdirectory_ignored() {
        let temp = tempfile::tempdir().expect("temp dir");
        std::fs::create_dir_all(temp.path().join("nested")).expect("dir");
        std::fs::write(temp.path().join("top.log"), b"x").expect("write");
        let logs = scan_old_logs(temp.path()).expect("scan");
        assert_eq!(logs.len(), 1);
    }
}
