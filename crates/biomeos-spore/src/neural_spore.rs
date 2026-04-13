// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Neural API `LiveSpore` Evolution
//!
//! Extends `LiveSpore` with Neural API graph-based deployment
//! Adds metrics collection and rollback capabilities

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Metrics collected during deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentMetrics {
    /// Total deployment wall-clock time in milliseconds
    pub total_duration_ms: u64,
    /// Number of primals successfully deployed
    pub primals_deployed: usize,
    /// Number of primals that failed deployment
    pub primals_failed: usize,
    /// Per-phase timing and status
    pub phase_metrics: Vec<PhaseMetrics>,
    /// ISO-8601 timestamp of when metrics were captured
    pub timestamp: String,
}

/// Metrics for individual phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseMetrics {
    /// Phase ordinal (0-based)
    pub phase_id: usize,
    /// Number of nodes in this phase
    pub node_count: usize,
    /// Phase duration in milliseconds
    pub duration_ms: u64,
    /// Whether this phase completed successfully
    pub success: bool,
    /// Error messages for nodes that failed
    pub failures: Vec<String>,
}

/// Rollback state tracker
#[derive(Debug, Clone)]
pub struct RollbackState {
    /// PIDs of spawned primal processes to kill on rollback
    pub spawned_pids: Vec<u32>,
    /// Socket file paths to clean up on rollback
    pub created_sockets: Vec<PathBuf>,
    /// When the deployment started (for timeout tracking)
    pub started_at: Instant,
}

impl RollbackState {
    /// Create a new empty rollback state
    #[must_use]
    pub fn new() -> Self {
        Self {
            spawned_pids: Vec::new(),
            created_sockets: Vec::new(),
            started_at: Instant::now(),
        }
    }

    /// Add spawned process
    pub fn track_process(&mut self, pid: u32) {
        self.spawned_pids.push(pid);
    }

    /// Add created socket
    pub fn track_socket(&mut self, socket_path: PathBuf) {
        self.created_sockets.push(socket_path);
    }

    /// Execute rollback
    pub async fn rollback(&self) -> Result<()> {
        info!(
            "🔄 Rolling back deployment ({} processes, {} sockets)",
            self.spawned_pids.len(),
            self.created_sockets.len()
        );

        // Kill spawned processes
        for pid in &self.spawned_pids {
            info!("   Killing process {}", pid);
            #[cfg(unix)]
            {
                use rustix::process::{Pid, Signal, kill_process, test_kill_process};
                let pid_i32 = i32::try_from(*pid).unwrap_or(-1);
                if let Some(rustix_pid) = Pid::from_raw(pid_i32) {
                    if let Err(e) = kill_process(rustix_pid, Signal::Term) {
                        warn!("   Failed to kill {}: {}", pid, e);
                    } else {
                        // Modern async: Wait for process to actually exit (with timeout)
                        let pid_to_check = pid_i32;
                        let wait_for_exit = async {
                            let mut interval = tokio::time::interval(Duration::from_millis(10));
                            for _ in 0..100 {
                                // Check for up to 1 second
                                interval.tick().await;
                                // Check if process still exists (test_kill_process returns Err when gone)
                                if let Some(p) = Pid::from_raw(pid_to_check)
                                    && test_kill_process(p).is_err()
                                {
                                    // Process is gone
                                    return;
                                }
                            }
                            // Force kill if still alive
                            if let Some(p) = Pid::from_raw(pid_to_check) {
                                let _ = kill_process(p, Signal::Kill);
                            }
                        };
                        wait_for_exit.await;
                    }
                }
            }
        }

        // Clean up sockets
        for socket in &self.created_sockets {
            if socket.exists() {
                info!("   Removing socket: {}", socket.display());
                if let Err(e) = tokio::fs::remove_file(socket).await {
                    warn!("   Failed to remove socket: {}", e);
                }
            }
        }

        info!(
            "✅ Rollback complete ({}ms elapsed)",
            self.started_at.elapsed().as_millis()
        );

        Ok(())
    }
}

/// `LiveSpore` with Neural API support
pub struct NeuralSpore {
    /// Root path of the spore (e.g. USB mount / biomeOS)
    pub root_path: PathBuf,
    /// Directory containing graph TOML files
    pub graphs_dir: PathBuf,
    /// Directory containing primal binaries
    pub binaries_dir: PathBuf,
    /// Metrics collected during the last deployment (if any)
    pub metrics: Option<DeploymentMetrics>,
}

impl NeuralSpore {
    /// Create new Neural Spore structure
    pub fn new(usb_mount: impl AsRef<std::path::Path>) -> Result<Self> {
        let root_path = usb_mount.as_ref().join("biomeOS");
        let graphs_dir = root_path.join("graphs");
        let binaries_dir = root_path.join("primals");

        Ok(Self {
            root_path,
            graphs_dir,
            binaries_dir,
            metrics: None,
        })
    }

    /// Prepare `LiveSpore` structure
    pub async fn prepare(&self) -> Result<()> {
        info!(
            "🌱 Preparing Neural LiveSpore at: {}",
            self.root_path.display()
        );

        // Create directory structure
        tokio::fs::create_dir_all(&self.graphs_dir)
            .await
            .context("Failed to create graphs directory")?;
        tokio::fs::create_dir_all(&self.binaries_dir)
            .await
            .context("Failed to create binaries directory")?;
        tokio::fs::create_dir_all(self.root_path.join("logs")).await?;
        tokio::fs::create_dir_all(self.root_path.join("metrics")).await?;

        info!("✅ Directory structure created");
        Ok(())
    }

    /// Copy neural graphs to `LiveSpore`
    pub async fn install_graphs(&self, source_graphs_dir: &Path) -> Result<()> {
        info!("📊 Installing Neural API graphs...");

        // Copy all .toml graphs
        let mut entries = tokio::fs::read_dir(source_graphs_dir).await?;
        let mut copied = 0;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                let filename = path
                    .file_name()
                    .context("Graph file path has no filename")?;
                let dest = self.graphs_dir.join(filename);

                tokio::fs::copy(&path, &dest)
                    .await
                    .context(format!("Failed to copy {}", filename.to_string_lossy()))?;

                info!("   Copied: {}", filename.to_string_lossy());
                copied += 1;
            }
        }

        info!("✅ Installed {} graph(s)", copied);
        Ok(())
    }

    /// Copy primal binaries to `LiveSpore`
    pub async fn install_binaries(&self, source_bin_dir: &Path) -> Result<()> {
        info!("🔧 Installing primal binaries...");

        let mut entries = tokio::fs::read_dir(source_bin_dir).await?;
        let mut copied = 0;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                let filename = path
                    .file_name()
                    .context("Binary file path has no filename")?;
                let dest = self.binaries_dir.join(filename);

                tokio::fs::copy(&path, &dest)
                    .await
                    .context(format!("Failed to copy {}", filename.to_string_lossy()))?;

                // Make executable
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = tokio::fs::metadata(&dest).await?.permissions();
                    perms.set_mode(0o755);
                    tokio::fs::set_permissions(&dest, perms).await?;
                }

                info!("   Installed: {}", filename.to_string_lossy());
                copied += 1;
            }
        }

        info!("✅ Installed {} binary(ies)", copied);
        Ok(())
    }

    /// Copy nucleus binary
    pub async fn install_nucleus(&self, nucleus_binary: &Path) -> Result<()> {
        info!("🧬 Installing nucleus orchestrator...");

        let dest = self.binaries_dir.join("nucleus");
        tokio::fs::copy(nucleus_binary, &dest)
            .await
            .context("Failed to copy nucleus binary")?;

        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = tokio::fs::metadata(&dest).await?.permissions();
            perms.set_mode(0o755);
            tokio::fs::set_permissions(&dest, perms).await?;
        }

        info!("✅ Nucleus orchestrator installed");
        Ok(())
    }

    /// Create README
    pub async fn create_readme(&self) -> Result<()> {
        let readme = format!(
            r"# 🌱 Neural LiveSpore - Portable NUCLEUS

**Version**: 2.0.0 (Neural API)
**Created**: {}
**Type**: LiveSpore with Neural API graph deployment

## 🚀 Quick Start

### Deploy NUCLEUS Ecosystem

```bash
cd biomeOS
./primals/nucleus deploy --family 1894e909e454 --graph graphs/nucleus_simple.toml
```

This will deploy:
- ✅ BearDog (Security & Encryption)
- ✅ Songbird (Discovery & P2P)
- ✅ Toadstool (Compute & GPU)
- ✅ NestGate (Storage)

### Check Status

```bash
./primals/nucleus status
```

### Verify Health

```bash
./primals/nucleus verify
```

## 📊 What's Inside

### Primal Binaries (`primals/`)
- `beardog-server` - Security & encryption
- `songbird-orchestrator` - Discovery & P2P
- `toadstool` - Compute orchestration
- `nestgate` - Storage & persistence
- `nucleus` - Neural API orchestrator

### Neural Graphs (`graphs/`)
- `nucleus_simple.toml` - 4-primal NUCLEUS deployment
- `nucleus_ecosystem.toml` - Full 6-primal ecosystem
- Additional atomic graphs (tower, node, nest)

### Logs & Metrics
- `logs/` - Primal log files
- `metrics/` - Deployment metrics (JSON)

## 🧬 Architecture

This LiveSpore uses the **Neural API** for deployment:
- **Graph-based**: Declarative TOML definitions
- **DAG resolution**: Automatic dependency ordering
- **Parallel execution**: Optimal performance
- **Metrics**: Full deployment tracking
- **Rollback**: Automatic on failure

## 🌟 Features

✅ Single command deployment
✅ Automatic dependency resolution
✅ Parallel primal execution
✅ Inter-primal discovery
✅ Socket-based IPC
✅ Metrics collection
✅ Rollback on failure

## 📚 Documentation

For more information, see:
- Neural API docs in workspace
- Individual primal READMEs
- biomeOS documentation

---

**biomeOS**: Deploy and assume ecosystems, not isolated primals. ✨
",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        tokio::fs::write(self.root_path.join("README.md"), readme).await?;
        info!("✅ README created");
        Ok(())
    }

    /// Save deployment metrics
    pub async fn save_metrics(&self, metrics: &DeploymentMetrics) -> Result<()> {
        let metrics_file = self
            .root_path
            .join("metrics")
            .join(format!("deployment-{}.json", metrics.timestamp));

        let json = serde_json::to_string_pretty(metrics)?;
        tokio::fs::write(&metrics_file, json).await?;

        info!("📊 Metrics saved: {}", metrics_file.display());
        Ok(())
    }
}

impl Default for RollbackState {
    fn default() -> Self {
        Self::new()
    }
}
