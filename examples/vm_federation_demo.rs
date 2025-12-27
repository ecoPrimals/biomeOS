//! BiomeOS VM Federation Example
//!
//! Demonstrates using benchScale's libvirt backend to manage a BiomeOS VM federation.

use anyhow::Result;
use biomeos_core::vm_federation::VmFederationManager;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("🚀 BiomeOS VM Federation Example");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("Using benchScale libvirt backend for native VM management");
    println!();

    let manager = VmFederationManager::new()?;
    let federation_name = "biomeos-demo-federation";

    println!("Federation: {}", federation_name);
    println!();

    // Check status first
    println!("📊 Checking current status...");
    match manager.status(federation_name).await {
        Ok(status) => {
            println!("{}", status);
            if status.contains("not found") || status.contains("does not exist") {
                println!();
                println!("Federation not found. Creating new one...");
                println!();
                
                // Create federation
                println!("Step 1: Creating federation...");
                manager.create(federation_name).await?;
                println!("✅ Created");
                println!();
            }
        }
        Err(e) => {
            println!("Status check failed (likely doesn't exist yet): {}", e);
            println!();
            println!("Creating new federation...");
            manager.create(federation_name).await?;
            println!("✅ Created");
            println!();
        }
    }

    // Start VMs
    println!("Step 2: Starting VMs...");
    manager.start(federation_name).await?;
    println!("✅ Started");
    println!();

    // Wait for boot
    println!("⏱️  Waiting 30s for VMs to boot...");
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    println!();

    // Run tests
    println!("Step 3: Running tests...");
    manager.test(federation_name).await?;
    println!("✅ Tests complete");
    println!();

    // Check final status
    println!("Step 4: Final status...");
    let final_status = manager.status(federation_name).await?;
    println!("{}", final_status);
    println!();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Federation Demo Complete!");
    println!();
    println!("To clean up:");
    println!("  cargo run --example vm_federation_demo -- cleanup");
    println!();
    println!("Or manually:");
    println!("  cd ../benchscale");
    println!("  cargo run -- stop {}", federation_name);
    println!("  cargo run -- destroy {}", federation_name);

    Ok(())
}

