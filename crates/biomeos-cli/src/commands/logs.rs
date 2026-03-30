// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Fossil log management CLI commands
//! 
//! This handles the fossil record system for archived logs,
//! separate from the service log streaming (Commands::Logs)

use anyhow::Result;
use biomeos_spore::logs::{LogConfig, LogManager, FossilIndex};
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
        FossilAction::Clean { older_than, dry_run } => handle_clean(older_than, dry_run).await,
        FossilAction::Migrate { from, dry_run } => handle_migrate(from, dry_run).await,
        FossilAction::CleanupStale => handle_cleanup_stale().await,
    }
}

async fn handle_active(node_filter: Option<String>) -> Result<()> {
    let config = LogConfig::default();
    let manager = LogManager::new(config);
    
    let sessions = manager.list_active_sessions()?;
    
    let filtered: Vec<_> = sessions.iter()
        .filter(|s| node_filter.as_ref().map_or(true, |n| s.node_id.contains(n)))
        .collect();
    
    if filtered.is_empty() {
        println!("\n🌱 No active log sessions found");
        return Ok(());
    }
    
    println!("\n🌱 Active Log Sessions");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    for session in filtered {
        let duration = session.duration();
        let hours = duration.num_hours();
        let mins = duration.num_minutes() % 60;
        
        println!("Node: {}", session.node_id);
        println!("  Started: {} ({}h {}m ago)", 
            session.started_at.format("%Y-%m-%d %H:%M:%S"),
            hours, mins);
        
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
                let status = if log_file.pid.is_some() { "active" } else { "closed" };
                println!("    • {:<15} ({} KB, {})", 
                    format!("{}.log", log_file.primal), size_kb, status);
            }
        }
        
        println!();
    }
    
    println!("Total: {} active session(s)", filtered.len());
    
    Ok(())
}

async fn handle_fossil(node_filter: Option<String>, limit: usize, show: Option<usize>) -> Result<()> {
    let config = LogConfig::default();
    let index_path = config.fossil_dir.join("index.toml");
    
    if !index_path.exists() {
        println!("\n🦴 No fossil records found");
        return Ok(());
    }
    
    let index = FossilIndex::load(&index_path)?;
    
    let filtered: Vec<_> = index.fossils.iter()
        .filter(|f| node_filter.as_ref().map_or(true, |n| f.node_id.contains(n)))
        .take(limit)
        .collect();
    
    if let Some(idx) = show {
        if idx > 0 && idx <= filtered.len() {
            println!("\n🦴 Fossil Record Details\n");
            let fossil = filtered[idx - 1];
            println!("Node: {}", fossil.node_id);
            println!("Session: {}", fossil.session_started.format("%Y-%m-%d %H:%M:%S"));
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
        println!("[{}] {} → {}", 
            i + 1,
            fossil.session_started.format("%Y-%m-%d %H:%M:%S"),
            fossil.node_id);
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
    
    let session = sessions.iter()
        .find(|s| s.node_id == node_id)
        .ok_or_else(|| anyhow::anyhow!("No active session found for node: {}", node_id))?;
    
    let fossil = manager.archive_session(
        session,
        biomeos_spore::logs::ArchivalReason::Manual,
    ).await?;
    
    println!("✅ Archived session for {}", node_id);
    println!("   Duration: {} seconds", fossil.duration().num_seconds());
    println!("   Issues: {}", fossil.issues.len());
    
    Ok(())
}

/// Handle fossil cleanup
///
/// EVOLVED (Jan 27, 2026): Complete implementation
async fn handle_clean(older_than: u64, dry_run: bool) -> Result<()> {
    use chrono::{Duration, Utc};

    if dry_run {
        println!("🔍 Dry run: No files will be deleted\n");
    }
    
    let config = LogConfig::default();
    let index_path = config.fossil_dir.join("index.toml");

    // Check if fossil index exists
    if !index_path.exists() {
        println!("📁 No fossil index found at: {}", index_path.display());
        println!("   No fossils to clean.");
        return Ok(());
    }

    // Load fossil index
    let index = FossilIndex::load(&index_path)?;
    let cutoff = Utc::now() - Duration::days(older_than as i64);

    println!("🧹 Cleaning fossils older than {} days", older_than);
    println!("   Cutoff date: {}", cutoff.format("%Y-%m-%d %H:%M:%S"));
    println!();

    let mut cleaned_count = 0;
    let mut freed_bytes: u64 = 0;
    let mut kept_count = 0;

    for entry in &index.fossils {
        let is_old = entry.session_started < cutoff;

        if is_old {
            // Calculate size of fossil directory
            let dir_size = calculate_dir_size(&entry.fossil_path);

            if dry_run {
                println!(
                    "   [DRY RUN] Would delete: {} ({} bytes)",
                    entry.fossil_path.display(),
                    dir_size
                );
            } else {
                // Delete the fossil directory
                if entry.fossil_path.exists() {
                    match std::fs::remove_dir_all(&entry.fossil_path) {
                        Ok(()) => {
                            println!(
                                "   ✅ Deleted: {} ({} bytes)",
                                entry.fossil_path.display(),
                                dir_size
                            );
                            freed_bytes += dir_size;
                        }
                        Err(e) => {
                            println!(
                                "   ⚠️ Failed to delete {}: {}",
                                entry.fossil_path.display(),
                                e
                            );
                            continue;
                        }
                    }
                }
            }
            cleaned_count += 1;
        } else {
            kept_count += 1;
        }
    }

    println!();
    if dry_run {
        println!("📊 Summary (dry run):");
        println!("   Would clean: {} fossils", cleaned_count);
        println!("   Would keep:  {} fossils", kept_count);
    } else {
        // Update index to remove cleaned entries
        if cleaned_count > 0 {
            let mut new_index = FossilIndex::new();
            for entry in &index.fossils {
                if entry.session_started >= cutoff {
                    new_index.add(entry.clone());
                }
            }
            new_index.save(&index_path)?;
        }

        println!("📊 Cleanup complete:");
        println!("   Cleaned: {} fossils", cleaned_count);
        println!("   Freed:   {} bytes ({:.2} MB)", freed_bytes, freed_bytes as f64 / 1_048_576.0);
        println!("   Kept:    {} fossils", kept_count);
    }
    
    Ok(())
}

/// Calculate total size of a directory
fn calculate_dir_size(path: &PathBuf) -> u64 {
    if !path.exists() {
        return 0;
    }

    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                total += path.metadata().map(|m| m.len()).unwrap_or(0);
            } else if path.is_dir() {
                total += calculate_dir_size(&path);
            }
        }
    }
    total
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
    manager.initialize()?;
    
    // Scan for old log files
    let mut old_logs = Vec::new();
    for entry in std::fs::read_dir(&from)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "log") {
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
        for log_path in &old_logs {
            let file_name = log_path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "unknown.log".to_string());
            let dest = config.fossil_dir.join("legacy").join(&file_name);

            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::rename(log_path, &dest)?;

            println!("  ✅ Migrated: {}", file_name);
        }

        println!("\n✅ Migration complete!");
    } else {
        for log_path in &old_logs {
            let file_name = log_path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "unknown.log".to_string());
            println!("  Would migrate: {}", file_name);
        }
    }
    
    Ok(())
}

async fn handle_cleanup_stale() -> Result<()> {
    info!("Cleaning up stale active sessions");
    
    let config = LogConfig::default();
    let manager = LogManager::new(config);
    
    manager.initialize()?;
    
    let archived = manager.cleanup_stale_sessions().await?;
    
    if archived.is_empty() {
        println!("✅ No stale sessions found");
    } else {
        println!("✅ Cleaned up {} stale session(s):", archived.len());
        for fossil in archived {
            println!("  • {} (duration: {}s)", 
                fossil.node_id,
                fossil.duration().num_seconds());
        }
    }
    
    Ok(())
}

