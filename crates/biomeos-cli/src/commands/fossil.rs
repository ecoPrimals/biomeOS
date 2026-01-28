//! Fossil log management CLI commands
//!
//! This handles the fossil record system for archived logs,
//! separate from the service log streaming (Commands::Logs)

use anyhow::Result;
use biomeos_spore::logs::{FossilIndex, LogConfig, LogManager};
use clap::{Args, Subcommand};
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Args)]
pub struct FossilArgs {
    #[command(subcommand)]
    pub action: FossilAction,
}

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

    let filtered: Vec<_> = sessions
        .iter()
        .filter(|s| node_filter.as_ref().is_none_or(|n| s.node_id.contains(n)))
        .collect();

    if filtered.is_empty() {
        println!("\n🌱 No active log sessions found");
        return Ok(());
    }

    println!("\n🌱 Active Log Sessions");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let total_count = filtered.len();

    for session in filtered {
        let duration = session.duration();
        let hours = duration.num_hours();
        let mins = duration.num_minutes() % 60;

        println!("Node: {}", session.node_id);
        println!(
            "  Started: {} ({}h {}m ago)",
            session.started_at.format("%Y-%m-%d %H:%M:%S"),
            hours,
            mins
        );

        if !session.process_pids.is_empty() {
            print!("  PIDs:");
            for pid in &session.process_pids {
                print!(" {}", pid);
            }
            println!();
        }

        if !session.log_files.is_empty() {
            println!("  Logs:");
            for log_file in &session.log_files {
                let size_kb = log_file.size_bytes / 1024;
                let status = if log_file.pid.is_some() {
                    "active"
                } else {
                    "closed"
                };
                println!(
                    "    • {:<15} ({} KB, {})",
                    format!("{}.log", log_file.primal),
                    size_kb,
                    status
                );
            }
        }

        println!();
    }

    println!("Total: {} active session(s)", total_count);

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
            println!("Node: {}", fossil.node_id);
            println!(
                "Session: {}",
                fossil.session_started.format("%Y-%m-%d %H:%M:%S")
            );
            println!("Reason: {:?}", fossil.archival_reason);
            println!("Path: {}", fossil.fossil_path.display());
            println!("Issues: {}", fossil.issue_count);
            println!("Encrypted: {}", if fossil.encrypted { "Yes" } else { "No" });
        } else {
            println!("Error: Invalid fossil number {}", idx);
        }
        return Ok(());
    }

    if filtered.is_empty() {
        println!("\n🦴 No fossil records found");
        return Ok(());
    }

    println!("\n🦴 Fossil Record");
    if let Some(node) = node_filter {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Filtered by node: {}\n", node);
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
        .ok_or_else(|| anyhow::anyhow!("No active session found for node: {}", node_id))?;

    let fossil = manager
        .archive_session(session, biomeos_spore::logs::ArchivalReason::Manual)
        .await?;

    println!("✅ Archived session for {}", node_id);
    println!("   Duration: {} seconds", fossil.duration().num_seconds());
    println!("   Issues: {}", fossil.issues.len());

    Ok(())
}

/// EVOLVED (Jan 27, 2026): Complete fossil cleanup implementation
async fn handle_clean(older_than: u64, dry_run: bool) -> Result<()> {
    use chrono::{Duration, Utc};

    if dry_run {
        println!("🔍 Dry run: No files will be deleted\n");
    }

    let config = LogConfig::default();
    let manager = LogManager::new(config.clone());

    // Initialize and get fossil index
    manager.initialize().await?;

    let cutoff = Utc::now() - Duration::days(older_than as i64);
    println!(
        "🗑️  Cleaning fossils older than {} days (before {})",
        older_than,
        cutoff.format("%Y-%m-%d")
    );

    // Get all fossils from index
    let index_path = config.fossil_dir.join("index.toml");
    let index = FossilIndex::load(&index_path)?;
    let mut cleaned_count = 0;
    let mut freed_bytes: u64 = 0;

    for fossil in &index.fossils {
        // Check if fossil is older than cutoff (using session_started as reference)
        if fossil.session_started < cutoff {
            // Use fossil_path from the index entry
            let fossil_path = &fossil.fossil_path;

            if fossil_path.exists() {
                if let Ok(metadata) = std::fs::metadata(fossil_path) {
                    freed_bytes += metadata.len();
                }

                if dry_run {
                    println!(
                        "   Would delete: {} ({})",
                        fossil.node_id,
                        fossil.session_started.format("%Y-%m-%d %H:%M")
                    );
                } else {
                    if let Err(e) = std::fs::remove_file(fossil_path) {
                        tracing::warn!("Failed to delete fossil {}: {}", fossil.node_id, e);
                        continue;
                    }
                    info!("Deleted fossil: {}", fossil.node_id);
                }
                cleaned_count += 1;
            }
        }
    }

    if cleaned_count > 0 {
        let freed_mb = freed_bytes as f64 / (1024.0 * 1024.0);
        if dry_run {
            println!(
                "\n📊 Would clean {} fossils, freeing {:.2} MB",
                cleaned_count, freed_mb
            );
        } else {
            println!(
                "\n✅ Cleaned {} fossils, freed {:.2} MB",
                cleaned_count, freed_mb
            );
        }
    } else {
        println!("✅ No fossils older than {} days found", older_than);
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

    // Initialize log directories
    manager.initialize().await?;

    // Scan for old log files
    let mut old_logs = Vec::new();
    for entry in std::fs::read_dir(&from)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().is_some_and(|ext| ext == "log") {
            old_logs.push(path);
        }
    }

    if old_logs.is_empty() {
        println!("✅ No old logs found to migrate");
        return Ok(());
    }

    println!("Found {} old log files", old_logs.len());

    if !dry_run {
        // Archive to fossil directory
        for log_path in old_logs {
            let file_name = log_path.file_name().unwrap().to_string_lossy();
            let dest = config.fossil_dir.join("legacy").join(&*file_name);

            std::fs::create_dir_all(dest.parent().unwrap())?;
            std::fs::rename(&log_path, &dest)?;

            println!("  ✅ Migrated: {}", file_name);
        }

        println!("\n✅ Migration complete!");
    } else {
        for log_path in old_logs {
            let file_name = log_path.file_name().unwrap().to_string_lossy();
            println!("  Would migrate: {}", file_name);
        }
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
