//! Full Federation Validation (Phase 2)
//!
//! This binary performs complete federation validation:
//! 1. Provision VMs
//! 2. Deploy biomeOS
//! 3. Start Songbird P2P
//! 4. Validate mDNS/UDP discovery
//! 5. Confirm federation coordination

use anyhow::{Context, Result};
use benchscale::{CloudInit, LibvirtBackend};
use biomeos_validation::{
    load_ssh_public_key, print_header, print_section, BiomeOsDeployment, DeployedVm, Topology,
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    print_header("🚀 Full Federation Validation 🚀");

    println!("Complete validation pipeline:");
    println!("  1. Provision VMs");
    println!("  2. Deploy biomeOS");
    println!("  3. Start Songbird P2P");
    println!("  4. Validate mDNS discovery");
    println!("  5. Confirm federation");
    println!();

    // Get topology (default to federation-2node)
    let topology_name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "federation-2node".to_string());

    let topology = match topology_name.as_str() {
        "federation-2node" => Topology::federation_2_node(),
        "federation-3node" => Topology::federation_3_node(),
        _ => {
            eprintln!("Unknown topology: {}", topology_name);
            eprintln!("Available: federation-2node, federation-3node");
            std::process::exit(1);
        }
    };

    print_section(&format!("Phase 1: Provision {} Topology", topology.name));

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

    // Provision VMs
    let mut deployed_vms = Vec::new();
    for (i, vm_config) in topology.vms.iter().enumerate() {
        println!("Creating VM {} of {}: {}", i + 1, topology.vms.len(), vm_config.name);

        let template_path = vm_config.template_path()?;
        let vm = backend
            .create_desktop_vm(
                &vm_config.name,
                &template_path,
                &cloud_init,
                vm_config.memory_mb,
                vm_config.vcpus,
                vm_config.disk_size_gb,
            )
            .await
            .with_context(|| format!("Failed to create VM: {}", vm_config.name))?;

        println!("✅ {} created ({})", vm.name, vm.ip_address);
        deployed_vms.push(DeployedVm::new(vm.name, vm.ip_address));

        // Small delay between VMs
        if i < topology.vms.len() - 1 {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }

    println!();
    print_section("Phase 2: Deploy biomeOS");

    let deployment = BiomeOsDeployment::default();

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
    print_section("Phase 3: Start Songbird P2P (TODO)");
    println!("TODO: Implement Songbird startup");
    println!("  • SSH to VMs");
    println!("  • Start Songbird orchestrate");
    println!("  • Wait for initialization");
    println!();

    println!();
    print_section("Phase 4: Validate mDNS Discovery (TODO)");
    println!("TODO: Implement mDNS validation");
    println!("  • Query avahi-browse");
    println!("  • Verify peer discovery");
    println!("  • Confirm service announcements");
    println!();

    println!();
    print_section("Phase 5: Confirm Federation (TODO)");
    println!("TODO: Implement federation validation");
    println!("  • Test P2P communication");
    println!("  • Verify coordination");
    println!("  • Validate data replication");
    println!();

    print_section("✅ Validation Status");
    println!("Phase 1: Provision VMs ✅ COMPLETE");
    println!("Phase 2: Deploy biomeOS ✅ COMPLETE");
    println!("Phase 3: Start Songbird 🚧 TODO");
    println!("Phase 4: Validate mDNS 🚧 TODO");
    println!("Phase 5: Confirm Federation 🚧 TODO");
    println!();

    println!("VMs provisioned and deployed:");
    for vm in &deployed_vms {
        println!("  • {} ({})", vm.name, vm.ip_address);
    }
    println!();

    println!("Cleanup:");
    let names: Vec<_> = deployed_vms.iter().map(|vm| vm.name.as_str()).collect();
    println!("  sudo virsh destroy {}", names.join(" "));
    println!("  sudo virsh undefine {} --remove-all-storage", names.join(" "));
    println!();

    print_header("✅ Phases 1-2 Complete! ✅");
    println!("Next: Implement phases 3-5 for full validation");
    println!();

    Ok(())
}

