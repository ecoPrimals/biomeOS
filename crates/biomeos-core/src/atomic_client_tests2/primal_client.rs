// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::atomic_primal_client::AtomicPrimalClient;

#[tokio::test]
async fn test_atomic_primal_client_discover_failure() {
    let result = AtomicPrimalClient::discover("nonexistent_primal_xyz_789").await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not found") || err.contains("Primal"));
}

#[tokio::test]
async fn test_atomic_primal_client_health_check_connection_refused() {
    let client =
        AtomicPrimalClient::unix("test-primal", "/nonexistent/socket/health_check_test.sock");
    let result = client.health_check().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_atomic_primal_client_execute_command_connection_refused() {
    let client = AtomicPrimalClient::unix("test-primal", "/nonexistent/socket/execute_test.sock");
    let result = client.execute_command("echo hello").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_atomic_primal_client_get_identity_connection_refused() {
    let client = AtomicPrimalClient::unix("test-primal", "/nonexistent/socket/identity_test.sock");
    let result = client.get_identity().await;
    assert!(result.is_err());
}
