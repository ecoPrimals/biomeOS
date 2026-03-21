// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Command handlers for fossil subcommands.

use anyhow::Result;
use biomeos_spore::logs::{ArchivalReason, FossilIndex, LogConfig, LogManager};
use std::path::PathBuf;
use tracing::info;

use super::format::{
    compute_cleanup_plan, filter_sessions, format_fossil_detail, format_session_display,
    scan_old_logs,
};
use super::{FossilAction, FossilArgs};

/// Log layout for CLI handlers. When `BIOMEOS_CLI_LOG_ROOT` is set (tests), active and fossil
/// directories are rooted there instead of `/var/biomeos/logs/...`.
fn cli_log_config() -> LogConfig {
    let mut config = LogConfig::default();
    if let Ok(root) = std::env::var("BIOMEOS_CLI_LOG_ROOT") {
        let root = PathBuf::from(root);
        config.active_dir = root.join("active");
        config.fossil_dir = root.join("fossil");
    }
    config
}

/// Dispatch fossil subcommand
pub(super) async fn dispatch(args: FossilArgs) -> Result<()> {
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
    let config = cli_log_config();
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
    let config = cli_log_config();
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

    let config = cli_log_config();
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

    let config = cli_log_config();
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

    let config = cli_log_config();
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

    let config = cli_log_config();
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
