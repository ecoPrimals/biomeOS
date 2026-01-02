//! Topology Provisioning
//!
//! Provision multiple VMs of different types using topology configurations.

use anyhow::{Context, Result};
use benchscale::{CloudInit, LibvirtBackend};
use biomeos_validation::{load_ssh_public_key, print_header, print_section, Topology};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    print_header("🦀 biomeOS Topology Provisioning 🦀");

    println!("Validation Substrate: Reliable VM provisioning");
    println!("  • Multiple VM types");
    println!("  • Different configurations");
    println!("  • Topologies for different scenarios");
    println!();

    // Parse command line args
    let topology_name = std::env::args().nth(1).unwrap_or_else(|| "simple-test".to_string());

    let topology = match topology_name.as_str() {
        "simple-test" => Topology::simple_test(),
        "federation-2node" => Topology::federation_2_node(),
        "federation-3node" => Topology::federation_3_node(),
        "mixed-ecosystem" => Topology::mixed_ecosystem(),
        _ => {
            eprintln!("Unknown topology: {}", topology_name);
            eprintln!();
            eprintln!("Available topologies:");
            eprintln!("  • simple-test        - 2 desktop VMs");
            eprintln!("  • federation-2node   - 2 federation nodes");
            eprintln!("  • federation-3node   - 3 federation nodes");
            eprintln!("  • mixed-ecosystem    - 2 federation + 1 compute");
            eprintln!();
            eprintln!("Usage: cargo run --bin provision-topology [TOPOLOGY]");
            std::process::exit(1);
        }
    };

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  Topology: {:<48}║", topology.name);
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
    println!("VMs to provision: {}", topology.vms.len());
    for (i, vm) in topology.vms.iter().enumerate() {
        println!("  {}. {} - {}", i + 1, vm.name, vm.description());
    }
    println!();

    // Load SSH key
    let ssh_public_key = load_ssh_public_key()?;
    println!("✅ SSH key loaded: ~/.ssh/id_rsa.pub");
    println!("   (User: {})", whoami::username());
    println!();

    print_section("🔧 Initializing benchScale LibvirtBackend");
    let backend = LibvirtBackend::new()?;
    println!("✅ LibvirtBackend initialized");
    println!();

    print_section("☁️  Creating cloud-init configuration");
    let cloud_init = CloudInit::builder()
        .add_user("biomeos", &ssh_public_key)
        .build();
    println!("✅ Cloud-init configured");
    println!();

    // Provision each VM
    let mut created_vms = Vec::new();

    for (i, vm_config) in topology.vms.iter().enumerate() {
        print_section(&format!("Creating VM {} of {}: {}", i + 1, topology.vms.len(), vm_config.name));
        println!("Type: {}", vm_config.description());

        let template_path = vm_config.template_path()?;
        println!("Template: {}", template_path.display());
        println!("Waiting for cloud-init and SSH...");
        println!();

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

        println!("✅ {} ready with SSH access!", vm_config.name);
        println!("   • Name: {}", vm.name);
        println!("   • IP: {}", vm.ip_address);
        println!("   • Resources: {}MB RAM, {} CPUs, {}GB disk", 
                 vm_config.memory_mb, vm_config.vcpus, vm_config.disk_size_gb);
        println!();

        created_vms.push(vm);

        // Small delay between VMs
        if i < topology.vms.len() - 1 {
            sleep(Duration::from_secs(2)).await;
        }
    }

    print_section("✅ Topology Provisioned Successfully!");
    println!("Topology: {}", topology.name);
    println!("VMs Created: {}", created_vms.len());
    println!();

    for (i, vm) in created_vms.iter().enumerate() {
        let config = &topology.vms[i];
        println!("{}. {} ({})", i + 1, vm.name, vm.ip_address);
        println!("   Type: {}", config.description());
    }

    println!();
    print_section("Validation Commands");
    println!("Test SSH access:");
    for vm in &created_vms {
        println!("  ssh biomeos@{}", vm.ip_address);
    }
    println!();

    println!("Test connectivity:");
    if created_vms.len() >= 2 {
        println!("  ssh biomeos@{} ping -c 3 {}", 
                 created_vms[0].ip_address, created_vms[1].ip_address);
    }
    println!();

    println!("Cleanup:");
    let names: Vec<_> = created_vms.iter().map(|vm| vm.name.as_str()).collect();
    println!("  sudo virsh destroy {}", names.join(" "));
    println!("  sudo virsh undefine {} --remove-all-storage", names.join(" "));
    println!();

    print_header("✅ Validation Substrate Ready! ✅");
    println!("Control substrate capabilities:");
    println!("  ✅ Multiple VM types");
    println!("  ✅ Different configurations");
    println!("  ✅ Topology-based provisioning");
    println!("  ✅ Reliable, repeatable");
    println!();
    println!("Next: Use this substrate for biomeOS validation");
    println!();

    Ok(())
}

