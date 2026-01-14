//! TRUE PRIMAL Atomic Orchestration Example
//!
//! Shows how atomic-deploy works as an orchestrator (not launcher!)
//! 
//! This demonstrates:
//! 1. Discovering running primals (not launching them)
//! 2. Verifying atomic requirements
//! 3. Providing deployment guidance
//! 4. Coordinating primal interactions

use anyhow::Result;
use biomeos_atomic_deploy::{
    CoordinationStatus, DeploymentGuide, PrimalCoordinator, PrimalDiscovery,
};
use std::path::PathBuf;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("🌳 TRUE PRIMAL Atomic Orchestration Example");
    info!("");

    // Step 1: Setup discovery (scans Unix sockets, no launching!)
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp/biomeos-runtime"));

    let discovery = PrimalDiscovery::new(runtime_dir)?;
    let coordinator = PrimalCoordinator::new(discovery);

    info!("📡 Step 1: Discovering running primals...");
    
    // Discover all running primals
    let discovered = coordinator.discovery().discover_all().await?;
    
    info!("Found {} primal(s):", discovered.len());
    for primal in &discovered {
        info!(
            "  • {} (family: {}, socket: {})",
            primal.name,
            primal.family_id.as_deref().unwrap_or("unknown"),
            primal.socket_path.display()
        );
    }
    info!("");

    // Step 2: Verify Tower atomic requirements
    info!("🏗️  Step 2: Verifying Tower atomic requirements...");
    
    let tower_primals = ["beardog", "songbird"];
    let status = coordinator.verify_primals(&tower_primals).await?;

    match status {
        CoordinationStatus::Ready => {
            info!("✅ Tower atomic is READY!");
            info!("   All required primals are running and responsive.");
            
            // Step 3: Coordinate interactions
            info!("");
            info!("🤝 Step 3: Coordinating primal introductions...");
            
            let tower_instances: Vec<_> = discovered
                .iter()
                .filter(|p| tower_primals.contains(&p.name.as_str()))
                .cloned()
                .collect();
            
            coordinator
                .coordinate_introductions(&tower_instances)
                .await?;
            
            info!("✅ Tower atomic coordination complete!");
            info!("");
            info!("🌸 You can now visualize with PetalTongue:");
            info!("   BIOMEOS_URL=http://localhost:3000 ./plasmidBin/petal-tongue");
        }
        
        CoordinationStatus::MissingPrimals(missing) => {
            info!("⚠️  Tower atomic NOT ready - missing primals:");
            for primal in &missing {
                info!("   • {}", primal);
            }
            info!("");
            
            // Provide deployment guidance
            info!("📖 Step 3: Generating deployment guide...");
            let guide = coordinator.generate_guide("tower", &tower_primals, "nat0");
            
            print_deployment_guide(&guide);
        }
        
        CoordinationStatus::Unresponsive(unresponsive) => {
            info!("⚠️  Tower atomic NOT ready - unresponsive primals:");
            for primal in &unresponsive {
                info!("   • {}", primal);
            }
            info!("");
            info!("💡 Try restarting the unresponsive primals.");
        }
    }

    Ok(())
}

/// Print a deployment guide
fn print_deployment_guide(guide: &DeploymentGuide) {
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("📖 Deployment Guide for {}", guide.atomic_name);
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");
    
    info!("Required Primals:");
    for primal in &guide.required_primals {
        info!("  • {}", primal);
    }
    info!("");
    
    info!("Start Commands:");
    for (i, cmd) in guide.start_commands.iter().enumerate() {
        info!("  {}. {}", i + 1, cmd);
    }
    info!("");
    
    info!("Verification:");
    info!("  {}", guide.verification);
    info!("");
    
    info!("Expected Sockets:");
    for socket in &guide.expected_sockets {
        info!("  • {}", socket);
    }
    info!("");
    
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("🌳 TRUE PRIMAL: Primals self-start, we discover!");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

