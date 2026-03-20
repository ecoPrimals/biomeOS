// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Neural API LiveSpore Evolution
//!
//! Extends LiveSpore with Neural API graph-based deployment
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

/// LiveSpore with Neural API support
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

    /// Copy primal binaries to LiveSpore
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ========== DeploymentMetrics Tests ==========

    #[test]
    fn test_deployment_metrics_creation() {
        let metrics = DeploymentMetrics {
            total_duration_ms: 1500,
            primals_deployed: 5,
            primals_failed: 1,
            phase_metrics: vec![],
            timestamp: "2026-01-24T19:00:00Z".to_string(),
        };

        assert_eq!(metrics.total_duration_ms, 1500);
        assert_eq!(metrics.primals_deployed, 5);
        assert_eq!(metrics.primals_failed, 1);
        assert!(metrics.phase_metrics.is_empty());
    }

    #[test]
    fn test_phase_metrics() {
        let phase = PhaseMetrics {
            phase_id: 1,
            node_count: 3,
            duration_ms: 500,
            success: true,
            failures: vec![],
        };

        assert_eq!(phase.phase_id, 1);
        assert_eq!(phase.node_count, 3);
        assert!(phase.success);
        assert!(phase.failures.is_empty());
    }

    #[test]
    fn test_phase_metrics_with_failures() {
        let phase = PhaseMetrics {
            phase_id: 2,
            node_count: 4,
            duration_ms: 250,
            success: false,
            failures: vec!["tower".to_string(), "beardog".to_string()],
        };

        assert!(!phase.success);
        assert_eq!(phase.failures.len(), 2);
        assert!(phase.failures.contains(&"tower".to_string()));
    }

    // ========== RollbackState Tests ==========

    #[test]
    fn test_rollback_state_new() {
        let state = RollbackState::new();

        assert!(state.spawned_pids.is_empty());
        assert!(state.created_sockets.is_empty());
        // started_at should be recent
        assert!(state.started_at.elapsed().as_secs() < 1);
    }

    #[test]
    fn test_rollback_state_track_process() {
        let mut state = RollbackState::new();

        state.track_process(1234);
        state.track_process(5678);

        assert_eq!(state.spawned_pids.len(), 2);
        assert!(state.spawned_pids.contains(&1234));
        assert!(state.spawned_pids.contains(&5678));
    }

    #[test]
    fn test_rollback_state_track_socket() {
        let mut state = RollbackState::new();

        state.track_socket(PathBuf::from("/tmp/socket1.sock"));
        state.track_socket(PathBuf::from("/tmp/socket2.sock"));

        assert_eq!(state.created_sockets.len(), 2);
        assert!(
            state
                .created_sockets
                .contains(&PathBuf::from("/tmp/socket1.sock"))
        );
    }

    #[tokio::test]
    async fn test_rollback_state_empty_rollback() {
        let state = RollbackState::new();

        // Empty rollback should succeed
        let result = state.rollback().await;
        assert!(result.is_ok());
    }

    // ========== NeuralSpore Tests ==========

    #[test]
    fn test_neural_spore_new() {
        let temp_dir = TempDir::new().unwrap();
        let spore = NeuralSpore::new(temp_dir.path()).unwrap();

        assert_eq!(spore.root_path, temp_dir.path().join("biomeOS"));
        assert_eq!(spore.graphs_dir, temp_dir.path().join("biomeOS/graphs"));
        assert_eq!(spore.binaries_dir, temp_dir.path().join("biomeOS/primals"));
        assert!(spore.metrics.is_none());
    }

    #[tokio::test]
    async fn test_neural_spore_prepare() {
        let temp_dir = TempDir::new().unwrap();
        let spore = NeuralSpore::new(temp_dir.path()).unwrap();

        spore.prepare().await.unwrap();

        // Verify directory structure created
        assert!(spore.root_path.exists());
        assert!(spore.graphs_dir.exists());
        assert!(spore.binaries_dir.exists());
        assert!(spore.root_path.join("logs").exists());
        assert!(spore.root_path.join("metrics").exists());
    }

    #[tokio::test]
    async fn test_neural_spore_install_graphs() {
        let temp_dir = TempDir::new().unwrap();
        let spore = NeuralSpore::new(temp_dir.path()).unwrap();
        spore.prepare().await.unwrap();

        // Create source graphs directory with test graphs
        let source_dir = temp_dir.path().join("source_graphs");
        tokio::fs::create_dir_all(&source_dir).await.unwrap();

        // Create test graph files
        tokio::fs::write(source_dir.join("test1.toml"), b"[graph]\nname = 'test1'")
            .await
            .unwrap();
        tokio::fs::write(source_dir.join("test2.toml"), b"[graph]\nname = 'test2'")
            .await
            .unwrap();

        // Install graphs
        spore.install_graphs(&source_dir).await.unwrap();

        // Verify graphs were copied
        assert!(spore.graphs_dir.join("test1.toml").exists());
        assert!(spore.graphs_dir.join("test2.toml").exists());
    }

    #[tokio::test]
    async fn test_neural_spore_install_binaries() {
        let temp_dir = TempDir::new().unwrap();
        let spore = NeuralSpore::new(temp_dir.path()).unwrap();
        spore.prepare().await.unwrap();

        // Create source binaries directory with test files
        let source_dir = temp_dir.path().join("source_bins");
        tokio::fs::create_dir_all(&source_dir).await.unwrap();

        // Create test binary files
        tokio::fs::write(source_dir.join("tower"), b"#!/bin/sh\necho tower")
            .await
            .unwrap();
        tokio::fs::write(source_dir.join("beardog"), b"#!/bin/sh\necho beardog")
            .await
            .unwrap();

        // Install binaries
        spore.install_binaries(&source_dir).await.unwrap();

        // Verify binaries were copied
        assert!(spore.binaries_dir.join("tower").exists());
        assert!(spore.binaries_dir.join("beardog").exists());

        // Verify they are executable (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = tokio::fs::metadata(spore.binaries_dir.join("tower"))
                .await
                .unwrap()
                .permissions();
            assert_eq!(perms.mode() & 0o111, 0o111); // Executable bits set
        }
    }

    #[tokio::test]
    async fn test_neural_spore_install_nucleus() {
        let temp_dir = TempDir::new().unwrap();
        let spore = NeuralSpore::new(temp_dir.path()).unwrap();
        spore.prepare().await.unwrap();

        // Create test nucleus binary
        let nucleus_path = temp_dir.path().join("nucleus_src");
        tokio::fs::write(&nucleus_path, b"#!/bin/sh\necho nucleus")
            .await
            .unwrap();

        // Install nucleus
        spore.install_nucleus(&nucleus_path).await.unwrap();

        // Verify nucleus was installed
        let installed_nucleus = spore.binaries_dir.join("nucleus");
        assert!(installed_nucleus.exists());

        // Verify executable permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = tokio::fs::metadata(&installed_nucleus)
                .await
                .unwrap()
                .permissions();
            assert_eq!(perms.mode() & 0o111, 0o111);
        }
    }

    #[tokio::test]
    async fn test_neural_spore_create_readme() {
        let temp_dir = TempDir::new().unwrap();
        let spore = NeuralSpore::new(temp_dir.path()).unwrap();
        spore.prepare().await.unwrap();

        spore.create_readme().await.unwrap();

        // Verify README was created
        let readme_path = spore.root_path.join("README.md");
        assert!(readme_path.exists());

        // Verify content
        let content = tokio::fs::read_to_string(&readme_path).await.unwrap();
        assert!(content.contains("Neural LiveSpore"));
        assert!(content.contains("Quick Start"));
        assert!(content.contains("Deploy NUCLEUS Ecosystem"));
    }

    #[tokio::test]
    async fn test_neural_spore_save_metrics() {
        let temp_dir = TempDir::new().unwrap();
        let spore = NeuralSpore::new(temp_dir.path()).unwrap();
        spore.prepare().await.unwrap();

        let metrics = DeploymentMetrics {
            total_duration_ms: 2500,
            primals_deployed: 6,
            primals_failed: 0,
            phase_metrics: vec![
                PhaseMetrics {
                    phase_id: 1,
                    node_count: 3,
                    duration_ms: 1200,
                    success: true,
                    failures: vec![],
                },
                PhaseMetrics {
                    phase_id: 2,
                    node_count: 3,
                    duration_ms: 1300,
                    success: true,
                    failures: vec![],
                },
            ],
            timestamp: "2026-01-24T20:00:00Z".to_string(),
        };

        spore.save_metrics(&metrics).await.unwrap();

        // Verify metrics file was created
        let metrics_path = spore
            .root_path
            .join(format!("metrics/deployment-{}.json", metrics.timestamp));
        assert!(metrics_path.exists());

        // Verify content
        let content = tokio::fs::read_to_string(&metrics_path).await.unwrap();
        let loaded: DeploymentMetrics = serde_json::from_str(&content).unwrap();

        assert_eq!(loaded.total_duration_ms, 2500);
        assert_eq!(loaded.primals_deployed, 6);
        assert_eq!(loaded.phase_metrics.len(), 2);
    }

    // ========== Integration Tests ==========

    #[tokio::test]
    async fn test_neural_spore_full_setup() {
        let temp_dir = TempDir::new().unwrap();
        let spore = NeuralSpore::new(temp_dir.path()).unwrap();

        // Step 1: Prepare structure
        spore.prepare().await.unwrap();

        // Step 2: Install graphs
        let graphs_dir = temp_dir.path().join("graphs_source");
        tokio::fs::create_dir_all(&graphs_dir).await.unwrap();
        tokio::fs::write(graphs_dir.join("ecosystem.toml"), b"[graph]")
            .await
            .unwrap();
        spore.install_graphs(&graphs_dir).await.unwrap();

        // Step 3: Install binaries
        let bins_dir = temp_dir.path().join("bins_source");
        tokio::fs::create_dir_all(&bins_dir).await.unwrap();
        tokio::fs::write(bins_dir.join("tower"), b"#!/bin/sh")
            .await
            .unwrap();
        spore.install_binaries(&bins_dir).await.unwrap();

        // Step 4: Install nucleus
        let nucleus_path = temp_dir.path().join("nucleus_bin");
        tokio::fs::write(&nucleus_path, b"#!/bin/sh").await.unwrap();
        spore.install_nucleus(&nucleus_path).await.unwrap();

        // Step 5: Create README
        spore.create_readme().await.unwrap();

        // Step 6: Save metrics
        let metrics = DeploymentMetrics {
            total_duration_ms: 5000,
            primals_deployed: 10,
            primals_failed: 0,
            phase_metrics: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        spore.save_metrics(&metrics).await.unwrap();

        // Verify complete structure
        assert!(spore.root_path.join("graphs/ecosystem.toml").exists());
        assert!(spore.binaries_dir.join("tower").exists());
        assert!(spore.binaries_dir.join("nucleus").exists());
        assert!(spore.root_path.join("README.md").exists());
        // Metrics filename now includes timestamp
        assert!(spore.root_path.join("metrics").exists());
    }

    #[test]
    fn test_deployment_metrics_serialization() {
        let metrics = DeploymentMetrics {
            total_duration_ms: 3000,
            primals_deployed: 4,
            primals_failed: 1,
            phase_metrics: vec![PhaseMetrics {
                phase_id: 1,
                node_count: 4,
                duration_ms: 3000,
                success: false,
                failures: vec!["songbird".to_string()],
            }],
            timestamp: "2026-01-24T20:30:00Z".to_string(),
        };

        // Test JSON serialization
        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: DeploymentMetrics = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.total_duration_ms, 3000);
        assert_eq!(deserialized.primals_failed, 1);
        assert_eq!(deserialized.phase_metrics[0].failures.len(), 1);
    }

    #[test]
    fn test_multiple_phase_metrics() {
        let mut metrics = DeploymentMetrics {
            total_duration_ms: 0,
            primals_deployed: 0,
            primals_failed: 0,
            phase_metrics: vec![],
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Add multiple phases
        for i in 0..5 {
            metrics.phase_metrics.push(PhaseMetrics {
                phase_id: i,
                node_count: 2 + i,
                duration_ms: (500 + i * 100) as u64,
                success: i % 2 == 0,
                failures: if i % 2 != 0 {
                    vec![format!("primal-{}", i)]
                } else {
                    vec![]
                },
            });
        }

        assert_eq!(metrics.phase_metrics.len(), 5);

        // Verify success/failure pattern
        assert!(metrics.phase_metrics[0].success);
        assert!(!metrics.phase_metrics[1].success);
        assert!(metrics.phase_metrics[2].success);
    }

    #[test]
    fn test_rollback_state_elapsed_time() {
        let state = RollbackState::new();
        // Verify started_at exists and elapsed() is non-negative (no wall-clock sleep needed)
        let elapsed = state.started_at.elapsed();
        assert!(elapsed.as_secs() < 60, "started_at should be recent");
    }
}
