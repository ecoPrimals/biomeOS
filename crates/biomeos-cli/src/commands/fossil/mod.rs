// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Fossil log management CLI commands
//!
//! This handles the fossil record system for archived logs,
//! separate from the service log streaming (`Commands::Logs`)

mod format;
mod handlers;
#[cfg(test)]
mod tests;

use clap::{Args, Subcommand};
use std::path::{Path, PathBuf};

pub use format::CleanupPlan;

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
pub async fn run(args: FossilArgs) -> anyhow::Result<()> {
    handlers::dispatch(args).await
}

/// Execute a fossil command with an explicit log root (tests; avoids `BIOMEOS_CLI_LOG_ROOT`).
pub async fn run_at(args: FossilArgs, log_root: &Path) -> anyhow::Result<()> {
    handlers::dispatch_at(args, Some(log_root)).await
}
