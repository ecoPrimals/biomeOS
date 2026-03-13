// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! E2E test for VM Federation with mandatory validation
//!
//! This test validates that our VM federation manager properly waits for
//! cloud-init completion and SSH access before declaring success.

use biomeos_core::vm_federation::{ValidationConfig, VmFederationManager};
use std::time::Duration;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_vm_federation_with_validation() -> anyhow::Result<()> {
    // Only run if libvirt testing is explicitly enabled
    if std::env::var("BENCHSCALE_TEST_LIBVIRT").is_err() {
        println!("Skipping VM federation test (set BENCHSCALE_TEST_LIBVIRT=1 to run)");
        return Ok(());
    }

    // Create manager with shorter timeouts for testing
    let config = ValidationConfig {
        cloud_init_timeout: Duration::from_secs(600), // 10 minutes
        ssh_timeout: Duration::from_secs(300),
        ssh_retry_interval: Duration::from_secs(30),
        ssh_max_retries: 20,
    };

    let manager = VmFederationManager::with_validation_config(config)?;
    let federation_name = "test-validation-federation";

    // Create federation (this will validate VMs are ready)
    println!("Creating federation with validation...");
    manager.create(federation_name).await?;

    // If we got here, VMs are guaranteed to be SSH-accessible
    println!("✅ Federation created and validated!");

    // Verify status
    let status = manager.status(federation_name).await?;
    println!("Federation status:\n{}", status);

    // Cleanup
    manager.stop(federation_name).await?;
    manager.destroy(federation_name).await?;

    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_validation_timeout() -> anyhow::Result<()> {
    // Only run if libvirt testing is explicitly enabled
    if std::env::var("BENCHSCALE_TEST_LIBVIRT").is_err() {
        return Ok(());
    }

    // Create manager with very short timeout to test failure path
    let config = ValidationConfig {
        cloud_init_timeout: Duration::from_secs(10), // Intentionally short
        ssh_timeout: Duration::from_secs(5),
        ssh_retry_interval: Duration::from_secs(2),
        ssh_max_retries: 3,
    };

    let manager = VmFederationManager::with_validation_config(config)?;

    // This should timeout (cloud-init takes longer than 10s)
    let result = manager.create("test-timeout-federation").await;

    // We expect this to fail with a timeout
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Timeout"));

    Ok(())
}
