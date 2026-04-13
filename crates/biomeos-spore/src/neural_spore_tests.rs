// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

// Sibling tests for neural_spore.rs
#![expect(clippy::unwrap_used, reason = "test")]

use super::neural_spore::*;
use std::path::PathBuf;
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
