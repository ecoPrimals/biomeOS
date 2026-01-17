//! Neural API LiveSpore Evolution
//!
//! Extends LiveSpore with Neural API graph-based deployment
//! Adds metrics collection and rollback capabilities

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Metrics collected during deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentMetrics {
    pub total_duration_ms: u64,
    pub primals_deployed: usize,
    pub primals_failed: usize,
    pub phase_metrics: Vec<PhaseMetrics>,
    pub timestamp: String,
}

/// Metrics for individual phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseMetrics {
    pub phase_id: usize,
    pub node_count: usize,
    pub duration_ms: u64,
    pub success: bool,
    pub failures: Vec<String>,
}

/// Rollback state tracker
#[derive(Debug, Clone)]
pub struct RollbackState {
    pub spawned_pids: Vec<u32>,
    pub created_sockets: Vec<PathBuf>,
    pub started_at: Instant,
}

impl RollbackState {
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
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;
                if let Err(e) = kill(Pid::from_raw(*pid as i32), Signal::SIGTERM) {
                    warn!("   Failed to kill {}: {}", pid, e);
                } else {
                    // Modern async: Wait for process to actually exit (with timeout)
                    let pid_to_check = *pid as i32;
                    let wait_for_exit = async {
                        let mut interval = tokio::time::interval(Duration::from_millis(10));
                        for _ in 0..100 {  // Check for up to 1 second
                            interval.tick().await;
                            // Check if process still exists
                            if kill(Pid::from_raw(pid_to_check), None).is_err() {
                                // Process is gone
                                return;
                            }
                        }
                        // Force kill if still alive
                        let _ = kill(Pid::from_raw(pid_to_check), Signal::SIGKILL);
                    };
                    wait_for_exit.await;
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

/// LiveSpore with Neural API support
pub struct NeuralSpore {
    pub root_path: PathBuf,
    pub graphs_dir: PathBuf,
    pub binaries_dir: PathBuf,
    pub metrics: Option<DeploymentMetrics>,
}

impl NeuralSpore {
    /// Create new Neural Spore structure
    pub fn new(usb_mount: PathBuf) -> Result<Self> {
        let root_path = usb_mount.join("biomeOS");
        let graphs_dir = root_path.join("graphs");
        let binaries_dir = root_path.join("primals");

        Ok(Self {
            root_path,
            graphs_dir,
            binaries_dir,
            metrics: None,
        })
    }

    /// Prepare LiveSpore structure
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

    /// Copy neural graphs to LiveSpore
    pub async fn install_graphs(&self, source_graphs_dir: &Path) -> Result<()> {
        info!("📊 Installing Neural API graphs...");

        // Copy all .toml graphs
        let mut entries = tokio::fs::read_dir(source_graphs_dir).await?;
        let mut copied = 0;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                let filename = path.file_name().unwrap();
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

    /// Copy primal binaries to LiveSpore
    pub async fn install_binaries(&self, source_bin_dir: &Path) -> Result<()> {
        info!("🔧 Installing primal binaries...");

        let mut entries = tokio::fs::read_dir(source_bin_dir).await?;
        let mut copied = 0;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() {
                let filename = path.file_name().unwrap();
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
            r#"# 🌱 Neural LiveSpore - Portable NUCLEUS

**Version**: 2.0.0 (Neural API)
**Created**: {}
**Type**: LiveSpore with Neural API graph deployment

## 🚀 Quick Start

### Deploy NUCLEUS Ecosystem

```bash
cd biomeOS
./primals/nucleus deploy --family nat0 --graph graphs/nucleus_simple.toml
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
"#,
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
