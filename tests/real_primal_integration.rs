//! Integration tests with real primal binaries
//!
//! These tests start actual primal services from ../phase1bins/ and test
//! BiomeOS integration with real services, not mocks.
//!
//! # Philosophy
//!
//! - **Real Services**: Use actual primal binaries
//! - **No Mocks**: Test production behavior
//! - **Capability-Based**: Discover services, don't hardcode
//! - **Graceful Degradation**: Tests skip if binaries unavailable

use anyhow::Result;
use biomeos_core::discovery_bootstrap::DiscoveryBootstrap;
use std::path::Path;
use std::process::{Child, Command};
use std::time::Duration;
use tokio::time::sleep;

/// Helper to check if a primal binary exists
fn primal_binary_exists(name: &str) -> bool {
    let path = format!("../phase1bins/{}", name);
    Path::new(&path).exists()
}

/// Helper to start a primal binary
///
/// Returns None if binary doesn't exist (test will be skipped)
fn start_primal(binary: &str, port: u16) -> Option<Child> {
    let path = format!("../phase1bins/{}", binary);

    if !Path::new(&path).exists() {
        eprintln!("⚠️  Binary not found: {}", path);
        return None;
    }

    match Command::new(&path)
        .arg("--port")
        .arg(port.to_string())
        .spawn()
    {
        Ok(child) => {
            eprintln!("✅ Started {} on port {}", binary, port);
            Some(child)
        }
        Err(e) => {
            eprintln!("❌ Failed to start {}: {}", binary, e);
            None
        }
    }
}

/// Helper to wait for service to be ready
async fn wait_for_service(endpoint: &str, max_attempts: u32) -> bool {
    for attempt in 1..=max_attempts {
        if let Ok(response) = reqwest::get(format!("{}/health", endpoint)).await {
            if response.status().is_success() {
                eprintln!("✅ Service ready at {} (attempt {})", endpoint, attempt);
                return true;
            }
        }
        sleep(Duration::from_millis(500)).await;
    }
    eprintln!(
        "❌ Service not ready at {} after {} attempts",
        endpoint, max_attempts
    );
    false
}

#[tokio::test]
#[ignore] // Run with: cargo test --test real_primal_integration -- --ignored
async fn test_songbird_discovery_real() -> Result<()> {
    // Check if Songbird binary exists
    if !primal_binary_exists("songbird-bin") {
        eprintln!("⚠️  Skipping test: songbird-bin not found in ../phase1bins/");
        return Ok(());
    }

    // Start Songbird
    let mut songbird = match start_primal("songbird-bin", 3000) {
        Some(child) => child,
        None => {
            eprintln!("⚠️  Skipping test: Could not start Songbird");
            return Ok(());
        }
    };

    // Wait for Songbird to be ready
    if !wait_for_service("http://localhost:3000", 20).await {
        songbird.kill()?;
        anyhow::bail!("Songbird failed to start");
    }

    // Test discovery bootstrap
    std::env::set_var("DISCOVERY_ENDPOINT", "http://localhost:3000");
    let bootstrap = DiscoveryBootstrap::new("universal-adapter");
    let endpoint = bootstrap.find_universal_adapter().await?;

    assert_eq!(endpoint, "http://localhost:3000");
    eprintln!("✅ Discovery bootstrap found Songbird");

    // Test health check
    let response = reqwest::get("http://localhost:3000/health").await?;
    assert!(response.status().is_success());
    eprintln!("✅ Songbird health check passed");

    // Cleanup
    songbird.kill()?;
    std::env::remove_var("DISCOVERY_ENDPOINT");

    Ok(())
}

#[tokio::test]
#[ignore] // Run with: cargo test --test real_primal_integration -- --ignored
async fn test_toadstool_compute_real() -> Result<()> {
    // Check if ToadStool binary exists
    if !primal_binary_exists("toadstool-bin") {
        eprintln!("⚠️  Skipping test: toadstool-bin not found in ../phase1bins/");
        return Ok(());
    }

    // Start ToadStool
    let mut toadstool = match start_primal("toadstool-bin", 8080) {
        Some(child) => child,
        None => {
            eprintln!("⚠️  Skipping test: Could not start ToadStool");
            return Ok(());
        }
    };

    // Wait for ToadStool to be ready
    if !wait_for_service("http://localhost:8080", 20).await {
        toadstool.kill()?;
        anyhow::bail!("ToadStool failed to start");
    }

    // Test health check
    let response = reqwest::get("http://localhost:8080/health").await?;
    assert!(response.status().is_success());
    eprintln!("✅ ToadStool health check passed");

    // Cleanup
    toadstool.kill()?;

    Ok(())
}

#[tokio::test]
#[ignore] // Run with: cargo test --test real_primal_integration -- --ignored
async fn test_multi_primal_ecosystem() -> Result<()> {
    // Check if binaries exist
    let has_songbird = primal_binary_exists("songbird-bin");
    let has_toadstool = primal_binary_exists("toadstool-bin");

    if !has_songbird || !has_toadstool {
        eprintln!("⚠️  Skipping test: Need both songbird-bin and toadstool-bin");
        return Ok(());
    }

    // Start Songbird (discovery)
    let mut songbird = match start_primal("songbird-bin", 3000) {
        Some(child) => child,
        None => {
            eprintln!("⚠️  Skipping test: Could not start Songbird");
            return Ok(());
        }
    };

    // Wait for Songbird
    if !wait_for_service("http://localhost:3000", 20).await {
        songbird.kill()?;
        anyhow::bail!("Songbird failed to start");
    }

    // Start ToadStool (compute)
    let mut toadstool = match start_primal("toadstool-bin", 8080) {
        Some(child) => child,
        None => {
            songbird.kill()?;
            eprintln!("⚠️  Skipping test: Could not start ToadStool");
            return Ok(());
        }
    };

    // Wait for ToadStool
    if !wait_for_service("http://localhost:8080", 20).await {
        songbird.kill()?;
        toadstool.kill()?;
        anyhow::bail!("ToadStool failed to start");
    }

    // Test ecosystem discovery
    std::env::set_var("DISCOVERY_ENDPOINT", "http://localhost:3000");
    let bootstrap = DiscoveryBootstrap::new("universal-adapter");
    let discovery_endpoint = bootstrap.find_universal_adapter().await?;

    eprintln!("✅ Multi-primal ecosystem running:");
    eprintln!("   - Songbird (discovery): {}", discovery_endpoint);
    eprintln!("   - ToadStool (compute): http://localhost:8080");

    // Verify both services are healthy
    let songbird_health = reqwest::get("http://localhost:3000/health").await?;
    let toadstool_health = reqwest::get("http://localhost:8080/health").await?;

    assert!(songbird_health.status().is_success());
    assert!(toadstool_health.status().is_success());

    eprintln!("✅ Multi-primal ecosystem health checks passed");

    // Cleanup
    songbird.kill()?;
    toadstool.kill()?;
    std::env::remove_var("DISCOVERY_ENDPOINT");

    Ok(())
}

#[tokio::test]
#[ignore] // Run with: cargo test --test real_primal_integration -- --ignored
async fn test_capability_based_discovery() -> Result<()> {
    // This test demonstrates capability-based discovery without hardcoding

    // Check if Songbird exists
    if !primal_binary_exists("songbird-bin") {
        eprintln!("⚠️  Skipping test: songbird-bin not found");
        return Ok(());
    }

    // Start Songbird
    let mut songbird = match start_primal("songbird-bin", 3000) {
        Some(child) => child,
        None => return Ok(()),
    };

    if !wait_for_service("http://localhost:3000", 20).await {
        songbird.kill()?;
        anyhow::bail!("Songbird failed to start");
    }

    // Test capability-based discovery (no hardcoded primal names)
    use biomeos_core::clients::songbird::SongbirdClient;

    std::env::set_var("DISCOVERY_ENDPOINT", "http://localhost:3000");
    let client = SongbirdClient::new("http://localhost:3000");

    // Query by capability, not by primal name
    let compute_services = client.discover_by_capability("compute").await;
    eprintln!(
        "✅ Capability query executed (compute services: {:?})",
        compute_services.as_ref().map(|s| s.len())
    );

    // Cleanup
    songbird.kill()?;
    std::env::remove_var("DISCOVERY_ENDPOINT");

    Ok(())
}

#[test]
fn test_phase1bins_available() {
    // Document which binaries are available for testing
    let binaries = vec![
        "beardog-v0.9.3-senderfixed-dec24",
        "toadstool-bin",
        "squirrel-bin",
        "nestgate-bin",
        "songbird-bin",
    ];

    eprintln!("\n📦 Phase 1 Binary Availability:");
    for binary in binaries {
        let exists = primal_binary_exists(binary);
        let status = if exists { "✅" } else { "❌" };
        eprintln!("   {} {}", status, binary);
    }
    eprintln!();
}
