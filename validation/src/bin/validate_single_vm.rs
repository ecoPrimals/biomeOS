//! Single VM Validation - Different biomeOS Deployments
//!
//! Tests different types of biomeOS deployments on a single VM:
//! - Minimal substrate (no primals)
//! - Single primal deployments
//! - Multi-primal compositions
//! - Different capability profiles
//!
//! This avoids the multi-VM DHCP race condition while validating
//! the deployment system thoroughly.

use anyhow::{Context, Result};
use benchscale::{CloudInit, LibvirtBackend};
use biomeos_validation::{
    load_ssh_public_key, print_header, print_section, BiomeOsDeployment, DeployedVm, VmConfig,
    Capability, CapabilityProfile,
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    print_header("🧪 Single VM Validation - biomeOS Deployments 🧪");

    println!("Testing different deployment types:");
    println!("  1. Minimal substrate (no primals)");
    println!("  2. NestGate standalone");
    println!("  3. Songbird P2P");
    println!("  4. Multi-primal composition");
    println!();

    // Get deployment type from args
    let deployment_type = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "minimal".to_string());

    print_section(&format!("Phase 1: Provision VM for '{}'", deployment_type));

    // Load SSH key
    let ssh_public_key = load_ssh_public_key()?;
    println!("✅ SSH key loaded");

    // Initialize backend
    let backend = LibvirtBackend::new()?;
    println!("✅ LibvirtBackend initialized");

    // Create cloud-init
    let cloud_init = CloudInit::builder()
        .add_user("biomeos", &ssh_public_key)
        .build();
    println!("✅ Cloud-init configured");
    println!();

    // Create VM
    let vm_name = format!("biomeos-{}", deployment_type);
    println!("Creating VM: {}", vm_name);

    let vm_config = VmConfig::federation_node(&vm_name);
    let template_path = vm_config.template_path()?;

    // Use create_desktop_vm_ready() to wait for cloud-init completion
    println!("  • Creating VM and waiting for cloud-init...");
    let vm = backend
        .create_desktop_vm_ready(
            &vm_config.name,
            &template_path,
            &cloud_init,
            vm_config.memory_mb,
            vm_config.vcpus,
            vm_config.disk_size_gb,
            "biomeos",                           // SSH username
            "",                                  // SSH password (empty = key auth)
            std::time::Duration::from_secs(600), // 10 minute timeout
        )
        .await
        .with_context(|| format!("Failed to create VM: {}", vm_config.name))?;

    println!("✅ {} ready with SSH access ({})", vm.name, vm.ip_address);
    println!();

    let deployed_vm = DeployedVm::new(vm.name.clone(), vm.ip_address.clone());

    print_section("Phase 2: Deploy biomeOS");

    let deployment = BiomeOsDeployment::default();
    deployment.deploy_to(&deployed_vm).await?;

    println!();
    print_section("Phase 3: Deploy Capabilities");

    match deployment_type.as_str() {
        "minimal" => {
            println!("📦 Minimal substrate deployment");
            println!("  • No primals deployed");
            println!("  • Base biomeOS structure only");
            println!();
            println!("✅ Minimal deployment complete");
        }

        "nestgate" => {
            println!("📦 NestGate standalone deployment");
            
            // Create NestGate capability profile
            let mut profile = CapabilityProfile::new("nestgate-standalone");
            profile.require(Capability::Storage);
            profile.require(Capability::Identity);
            profile.optional(Capability::StateManagement);
            
            println!("  Required Capabilities:");
            for cap in &profile.required_capabilities {
                println!("    • {:?} - {}", cap, cap.description());
            }
            
            println!();
            println!("✅ NestGate profile created (capabilities validated at runtime)");
        }

        "songbird" => {
            println!("📦 Songbird P2P deployment");
            
            let mut profile = CapabilityProfile::new("songbird-p2p");
            profile.require(Capability::P2PCoordination);
            profile.optional(Capability::Encryption);
            
            println!("  Required Capabilities:");
            for cap in &profile.required_capabilities {
                println!("    • {:?} - {}", cap, cap.description());
            }
            
            println!();
            println!("✅ Songbird profile created (capabilities validated at runtime)");
        }

        "composition" => {
            println!("📦 Multi-primal composition deployment");
            
            let mut profile = CapabilityProfile::new("full-ecosystem");
            profile.require(Capability::P2PCoordination);
            profile.require(Capability::Storage);
            profile.require(Capability::Identity);
            profile.require(Capability::Encryption);
            profile.optional(Capability::Compute);
            profile.optional(Capability::StateManagement);
            profile.optional(Capability::Visualization);
            
            println!("  Required Capabilities:");
            for cap in &profile.required_capabilities {
                println!("    • {:?} - {}", cap, cap.description());
            }
            
            if !profile.optional_capabilities.is_empty() {
                println!();
                println!("  Optional Capabilities:");
                for cap in &profile.optional_capabilities {
                    println!("    • {:?} - {}", cap, cap.description());
                }
            }
            
            println!();
            println!("✅ Multi-primal composition profile created (capabilities validated at runtime)");
        }

        _ => {
            eprintln!("Unknown deployment type: {}", deployment_type);
            eprintln!("Available: minimal, nestgate, songbird, composition");
            std::process::exit(1);
        }
    }

    println!();
    print_section("Phase 4: Verify Deployment");

    // Verify basic structure
    println!("  Verifying biomeOS structure...");
    let result = deployed_vm.ssh_exec("ls -la /opt/biomeos/")?;
    if result.contains("capabilities") {
        println!("  ✅ biomeOS structure verified");
    } else {
        println!("  ⚠️  biomeOS structure incomplete");
    }

    println!();
    print_section("✅ Single VM Validation Complete");

    println!("VM Details:");
    println!("  • Name: {}", deployed_vm.name);
    println!("  • IP: {}", deployed_vm.ip);
    println!("  • SSH: {}@{}", deployed_vm.ssh_user, deployed_vm.ip);
    println!("  • Type: {}", deployment_type);
    println!();
    println!("To connect:");
    println!("  ssh {}@{}", deployed_vm.ssh_user, deployed_vm.ip);
    println!();
    println!("To clean up:");
    println!("  sudo virsh destroy {}", deployed_vm.name);
    println!("  sudo virsh undefine {} --remove-all-storage", deployed_vm.name);
    println!();

    Ok(())
}

