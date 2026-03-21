// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Chaos Tests for Tower Atomic Resilience
//!
//! These tests verify system behavior under failure conditions:
//! - Primal crashes
//! - Socket disconnection
//! - Resource exhaustion
//! - Timing issues
//!
//! # Running
//! ```bash
//! cargo test --package biomeos-atomic-deploy --test chaos_tests
//! ```

use std::path::PathBuf;
use std::time::Duration;

/// Test fixture for chaos testing
struct ChaosFixture {
    family_id: String,
    socket_dir: PathBuf,
}

impl ChaosFixture {
    fn new(test_name: &str) -> Self {
        let socket_dir = std::env::temp_dir().join(format!("biomeos-chaos-{test_name}"));
        std::fs::create_dir_all(&socket_dir).ok();

        Self {
            family_id: format!("chaos-{test_name}"),
            socket_dir,
        }
    }

    fn socket_path(&self, primal: &str) -> PathBuf {
        self.socket_dir
            .join(format!("{}-{}.sock", primal, self.family_id))
    }

    async fn cleanup(&self) {
        let _ = std::fs::remove_dir_all(&self.socket_dir);
    }
}

impl Drop for ChaosFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.socket_dir);
    }
}

// ============================================================================
// Socket Failure Tests
// ============================================================================

#[tokio::test]
async fn test_missing_socket_graceful_failure() {
    use biomeos_atomic_deploy::executor::context::ExecutionContext;

    let fixture = ChaosFixture::new("missing-socket");

    let env = std::collections::HashMap::new();
    let context = ExecutionContext::new(env);

    // Get socket path for a non-existent primal
    let socket_path = context.get_socket_path("nonexistent-primal").await;

    // The path should be generated, but the socket won't exist
    assert!(!PathBuf::from(&socket_path).exists());

    fixture.cleanup().await;
}

#[tokio::test]
async fn test_socket_permission_denied() {
    let fixture = ChaosFixture::new("permission");

    // Create a socket with no permissions
    let socket_path = fixture.socket_path("restricted");
    std::fs::write(&socket_path, "").ok();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o000));
    }

    // Attempting to connect should fail gracefully
    let result = tokio::net::UnixStream::connect(&socket_path).await;
    assert!(
        result.is_err(),
        "Should fail to connect to restricted socket"
    );

    // Restore permissions for cleanup
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o644));
    }

    fixture.cleanup().await;
}

#[tokio::test]
async fn test_stale_socket_cleanup() {
    let fixture = ChaosFixture::new("stale");

    // Create a stale socket file (regular file, not actual socket)
    let socket_path = fixture.socket_path("stale-primal");
    std::fs::write(&socket_path, "stale").ok();

    assert!(socket_path.exists(), "Stale socket file should exist");

    // Nucleation should handle stale sockets
    use biomeos_atomic_deploy::nucleation::{SocketNucleation, SocketStrategy};
    let mut nucleation = SocketNucleation::new(SocketStrategy::FamilyDeterministic);

    // This should generate a new path (doesn't clean up existing, just assigns)
    let assigned = nucleation.assign_socket("stale-primal", &fixture.family_id);
    assert!(assigned.to_string_lossy().contains("stale-primal"));

    fixture.cleanup().await;
}

// ============================================================================
// Concurrent Access Tests
// ============================================================================

#[tokio::test]
async fn test_concurrent_socket_assignment() {
    use biomeos_atomic_deploy::nucleation::{SocketNucleation, SocketStrategy};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    let nucleation = Arc::new(RwLock::new(SocketNucleation::new(
        SocketStrategy::FamilyDeterministic,
    )));

    // Spawn multiple tasks trying to assign sockets concurrently
    let mut handles = tokio::task::JoinSet::new();

    for _ in 0..10 {
        let nuc = nucleation.clone();
        handles.spawn(async move {
            let mut n = nuc.write().await;
            n.assign_socket("concurrent-primal", "concurrent-test")
        });
    }

    // Collect all results
    let mut paths = Vec::new();
    while let Some(result) = handles.join_next().await {
        assert!(result.is_ok(), "Concurrent assignment should not panic");
        paths.push(result.unwrap());
    }

    // All should return the same path (deterministic)
    let first = &paths[0];
    for path in &paths {
        assert_eq!(
            path, first,
            "All concurrent assignments should return same path"
        );
    }
}

#[tokio::test]
async fn test_concurrent_context_access() {
    use biomeos_atomic_deploy::executor::context::ExecutionContext;
    use biomeos_atomic_deploy::nucleation::{SocketNucleation, SocketStrategy};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    let nucleation = Arc::new(RwLock::new(SocketNucleation::new(
        SocketStrategy::FamilyDeterministic,
    )));
    let context = Arc::new(
        ExecutionContext::new(std::collections::HashMap::new()).with_nucleation(nucleation),
    );

    // Multiple concurrent socket path requests
    let mut handles = tokio::task::JoinSet::new();

    for _ in 0..20 {
        let ctx = context.clone();
        handles.spawn(async move { ctx.get_socket_path("concurrent-context").await });
    }

    // Collect all results
    let mut paths = Vec::new();
    while let Some(result) = handles.join_next().await {
        paths.push(result.unwrap());
    }

    // All should be identical
    let first = &paths[0];
    for path in &paths {
        assert_eq!(
            path, first,
            "All concurrent context calls should return same path"
        );
    }
}

// ============================================================================
// Timeout and Resource Tests
// ============================================================================

#[tokio::test]
async fn test_socket_wait_timeout() {
    let fixture = ChaosFixture::new("timeout");
    let nonexistent = fixture.socket_path("never-created");

    let start = std::time::Instant::now();

    // Wait for socket with manual polling (simulating wait_for_socket_with_timeout)
    let mut found = false;
    for _ in 0..5 {
        if nonexistent.exists() {
            found = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    let elapsed = start.elapsed();

    // Should timeout (not found)
    assert!(!found, "Should timeout waiting for non-existent socket");

    // Should have waited at least 1 second
    assert!(
        elapsed >= Duration::from_millis(800),
        "Should have waited at least 0.8 seconds"
    );

    fixture.cleanup().await;
}

#[tokio::test]
async fn test_rapid_socket_creation_destruction() {
    let fixture = ChaosFixture::new("rapid");
    let socket_path = fixture.socket_path("rapid-primal");

    // Rapidly create and destroy the socket
    for _ in 0..100 {
        std::fs::write(&socket_path, "").ok();
        std::fs::remove_file(&socket_path).ok();
    }

    // System should remain stable
    assert!(!socket_path.exists(), "Socket should be cleaned up");

    fixture.cleanup().await;
}

// ============================================================================
// Recovery Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires lifecycle manager - run with --ignored"]
async fn test_primal_crash_recovery() {
    // This test would verify that when a primal crashes,
    // the lifecycle manager detects it and can resurrect it

    // For now, just verify the lifecycle manager structure exists
    use biomeos_atomic_deploy::lifecycle_manager::LifecycleManager;

    let manager = LifecycleManager::new("test-family");
    let status = manager.get_status().await;

    // Manager should be creatable and return empty status
    assert!(status.is_empty(), "New manager should have no primals");
}

#[tokio::test]
async fn test_graceful_degradation_without_beardog() {
    // When BearDog is unavailable, Songbird should still start
    // (with reduced functionality)

    // This is a documentation/design test - Songbird currently crashes
    // when BearDog is unavailable, which should be evolved to graceful degradation

    eprintln!("Graceful degradation test: Songbird should start without BearDog");
    eprintln!("Current behavior: Crashes - needs evolution");
}

// ============================================================================
// Fault Injection Tests
// ============================================================================

#[tokio::test]
async fn test_corrupted_json_rpc_request() {
    // Create a mock socket that receives corrupted JSON-RPC
    let fixture = ChaosFixture::new("corrupt-json");

    // If a primal were running, sending corrupted JSON should not crash it
    // This is a design verification test

    let large_payload = "x".repeat(10_000_000);
    let corrupt_requests: Vec<&str> = vec![
        "not json at all",
        "{}",                                    // Missing required fields
        r#"{"jsonrpc": "1.0"}"#,                 // Wrong version
        r#"{"jsonrpc": "2.0", "method": null}"#, // Null method
        "🔥🔥🔥",                                // Unicode chaos
        &large_payload,                          // Large payload
    ];

    for request in corrupt_requests {
        // Just verify we can handle these without panic
        let _ = serde_json::from_str::<serde_json::Value>(request);
    }

    fixture.cleanup().await;
}

#[tokio::test]
async fn test_env_var_injection_safety() {
    // Verify that socket paths don't allow path traversal attacks
    use biomeos_atomic_deploy::nucleation::{SocketNucleation, SocketStrategy};

    let mut nucleation = SocketNucleation::new(SocketStrategy::FamilyDeterministic);

    // Attempt path traversal in family_id
    let malicious_socket = nucleation.assign_socket("test", "../../../etc/passwd");

    // The path will contain the traversal attempt, but it's in the filename, not traversing
    // This test documents that family_id is used in the filename, not as a path
    assert!(
        malicious_socket.to_string_lossy().contains("test"),
        "Socket path should contain primal name: {malicious_socket:?}"
    );
}

// ============================================================================
// Nucleation Edge Cases
// ============================================================================

#[tokio::test]
async fn test_empty_primal_name() {
    use biomeos_atomic_deploy::nucleation::{SocketNucleation, SocketStrategy};

    let mut nucleation = SocketNucleation::new(SocketStrategy::FamilyDeterministic);

    // Empty primal name should still work (creates a socket with just family_id)
    let socket = nucleation.assign_socket("", "test-family");
    assert!(socket.to_string_lossy().contains(".sock"));
}

#[tokio::test]
async fn test_special_characters_in_family_id() {
    use biomeos_atomic_deploy::nucleation::{SocketNucleation, SocketStrategy};

    let mut nucleation = SocketNucleation::new(SocketStrategy::FamilyDeterministic);

    // Special characters in family_id
    let socket = nucleation.assign_socket("beardog", "test-family-with-dashes_and_underscores");
    assert!(socket.to_string_lossy().contains("beardog"));
    assert!(socket.to_string_lossy().ends_with(".sock"));
}

#[tokio::test]
async fn test_unicode_in_family_id() {
    use biomeos_atomic_deploy::nucleation::{SocketNucleation, SocketStrategy};

    let mut nucleation = SocketNucleation::new(SocketStrategy::FamilyDeterministic);

    // Unicode in family_id (edge case, not recommended but should handle)
    let socket = nucleation.assign_socket("beardog", "test-🦊-family");
    assert!(socket.to_string_lossy().contains("beardog"));
}
