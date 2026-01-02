//! Provision VMs with capability-based profiles
//!
//! Deploy based on required capabilities, not hardcoded primal names.

use anyhow::{Context, Result};
use benchscale::{CloudInit, LibvirtBackend};
use biomeos_validation::{
    load_ssh_public_key, print_header, print_section, BiomeOsDeployment, Capability,
    CapabilityProfile, DeployedVm, Topology,
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    print_header("🦀 Capability-Based Deployment 🦀");

    println!("Principle: Agnostic capability discovery");
    println!("  • No hardcoded primal names");
    println!("  • Primals discovered at runtime");
    println!("  • Capability-based profiles");
    println!();

    // Parse command line args
    let profile_name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "minimal-federation".to_string());

    let profile = match profile_name.as_str() {
        "minimal-federation" => CapabilityProfile::minimal_federation(),
        "full-federation" => CapabilityProfile::full_federation(),
        "compute-node" => CapabilityProfile::compute_node(),
        "storage-node" => CapabilityProfile::storage_node(),
        "full-ecosystem" => CapabilityProfile::full_ecosystem(),
        _ => {
            eprintln!("Unknown profile: {}", profile_name);
            eprintln!();
            eprintln!("Available profiles:");
            eprintln!("  • minimal-federation - P2P coordination only");
            eprintln!("  • full-federation    - P2P + Identity + Storage");
            eprintln!("  • compute-node       - P2P + Compute");
            eprintln!("  • storage-node       - P2P + Storage + Encryption");
            eprintln!("  • full-ecosystem     - All capabilities");
            eprintln!();
            eprintln!("Usage: cargo run --bin provision-with-capabilities [PROFILE]");
            std::process::exit(1);
        }
    };

    print_section(&format!("Capability Profile: {}", profile.name));
    println!("{}", profile.description());
    println!();

    println!("Required Capabilities:");
    for cap in &profile.required_capabilities {
        println!("  • {:?}: {}", cap, cap.description());
        println!("    Example providers: {}", cap.example_providers().join(", "));
    }
    println!();

    if !profile.optional_capabilities.is_empty() {
        println!("Optional Capabilities:");
        for cap in &profile.optional_capabilities {
            println!("  • {:?}: {}", cap, cap.description());
            println!("    Example providers: {}", cap.example_providers().join(", "));
        }
        println!();
    }

    println!("Note: Actual primals are discovered at runtime from primalBins/");
    println!("      biomeOS will find primals that provide these capabilities");
    println!();

    // Use 2-node topology for demonstration
    let topology = Topology::federation_2_node();

    print_section("Phase 1: Provision VMs");

    let ssh_public_key = load_ssh_public_key()?;
    println!("✅ SSH key loaded");

    let backend = LibvirtBackend::new()?;
    println!("✅ LibvirtBackend initialized");

    let cloud_init = CloudInit::builder()
        .add_user("biomeos", &ssh_public_key)
        .build();
    println!("✅ Cloud-init configured");
    println!();

    let mut deployed_vms = Vec::new();
    for (i, vm_config) in topology.vms.iter().enumerate() {
        println!("Creating VM {} of {}: {}", i + 1, topology.vms.len(), vm_config.name);
        println!("  • Waiting for cloud-init and SSH...");

        let template_path = vm_config.template_path()?;
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
        deployed_vms.push(DeployedVm::new(vm.name, vm.ip_address));

        if i < topology.vms.len() - 1 {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }

    println!();
    print_section("Phase 2: Deploy biomeOS with Capability Profile");

    let deployment = BiomeOsDeployment::with_profile(profile);

    for vm in &deployed_vms {
        deployment.deploy_to(vm).await?;
    }

    println!();
    print_section("Phase 2b: Verify Deployment");

    for vm in &deployed_vms {
        if !deployment.verify(vm).await? {
            anyhow::bail!("Deployment verification failed for {}", vm.name);
        }
    }

    println!();
    print_section("✅ Capability-Based Deployment Complete!");

    println!("VMs provisioned with capability profile:");
    for vm in &deployed_vms {
        println!("  • {} ({})", vm.name, vm.ip_address);
    }
    println!();

    println!("Runtime Discovery:");
    println!("  • biomeOS will scan primalBins/ at startup");
    println!("  • Primals self-report their capabilities");
    println!("  • Required capabilities are started automatically");
    println!("  • Optional capabilities are started if available");
    println!();

    println!("Cleanup:");
    let names: Vec<_> = deployed_vms.iter().map(|vm| vm.name.as_str()).collect();
    println!("  sudo virsh destroy {}", names.join(" "));
    println!("  sudo virsh undefine {} --remove-all-storage", names.join(" "));
    println!();

    print_header("✅ Agnostic Deployment Complete! ✅");
    println!("Principles upheld:");
    println!("  ✅ No hardcoded primal names");
    println!("  ✅ Capability-based profiles");
    println!("  ✅ Runtime discovery");
    println!("  ✅ Agnostic orchestration");
    println!();

    Ok(())
}

