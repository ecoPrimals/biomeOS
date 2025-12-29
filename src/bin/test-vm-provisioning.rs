//! Phase 1: VM Provisioning Test
//! 
//! Simple test to validate benchScale + agentReagents substrate
//! Goal: Create 2 VMs, verify they boot and are SSH accessible

use anyhow::{Context, Result};
use std::path::Path;
use std::thread;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║  Phase 1: VM Provisioning Test                          ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
    println!("Goal: Validate benchScale + agentReagents substrate");
    println!();

    // Configuration
    println!("📋 Configuration:");
    println!("  • Backend: libvirt (KVM)");
    println!("  • Template: agentReagents/rustdesk-ubuntu-22.04");
    println!("  • RAM: 2GB per VM");
    println!("  • CPUs: 2 per VM");
    println!("  • Disk: 25GB per VM");
    println!();

    let template_path = Path::new("/var/lib/libvirt/images/rustdesk-ubuntu-22.04-template.qcow2");
    
    // Check if template exists
    if !template_path.exists() {
        println!("⚠️  Template not found at: {}", template_path.display());
        println!("    Expected: agentReagents template");
        println!();
        println!("📝 To create the template:");
        println!("   cd ../../primalTools/agentReagents");
        println!("   sudo ./scripts/build-rustdesk-template.sh");
        println!("   sudo cp images/templates/rustdesk-ubuntu-22.04-template.qcow2 /var/lib/libvirt/images/");
        return Ok(());
    }

    println!("✅ Template found: {}", template_path.display());
    println!();

    // Get SSH key (use SUDO_USER if running with sudo)
    let actual_user = std::env::var("SUDO_USER").unwrap_or_else(|_| "eastgate".to_string());
    let ssh_key_path = Path::new("/home").join(&actual_user).join(".ssh/id_rsa.pub");
    
    if !ssh_key_path.exists() {
        println!("⚠️  SSH key not found at: {}", ssh_key_path.display());
        println!("    Run: ssh-keygen -t rsa -b 4096");
        return Ok(());
    }

    let ssh_key = std::fs::read_to_string(&ssh_key_path)
        .context("Failed to read SSH public key")?
        .trim()
        .to_string();

    println!("✅ SSH key loaded: {}", ssh_key_path.display());
    println!("   (User: {})", actual_user);
    println!();

    // Create benchScale backend
    println!("🔧 Initializing benchScale LibvirtBackend...");
    let backend = benchscale::LibvirtBackend::new()
        .context("Failed to initialize LibvirtBackend")?;
    println!("✅ LibvirtBackend initialized");
    println!();

    // Create cloud-init configuration
    println!("☁️  Creating cloud-init configuration...");
    let cloud_init = benchscale::CloudInit::builder()
        .add_user("biomeos", &ssh_key)
        .package("avahi-daemon")
        .package("avahi-utils")
        .package_update(false)  // Skip update for speed
        .build();
    println!("✅ Cloud-init configured");
    println!();

    // Create VM 1
    println!("═══════════════════════════════════════════════════════════");
    println!("Creating VM 1...");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    
    let vm1 = backend.create_desktop_vm(
        "biomeos-test-vm1",
        template_path,
        &cloud_init,
        2048,  // 2GB RAM
        2,     // 2 CPUs
        25,    // 25GB disk (template is 2.9GB, needs to be larger)
    ).await.context("Failed to create VM1")?;

    println!("✅ VM1 created!");
    println!("   • Name: {}", vm1.name);
    println!("   • IP: {}", vm1.ip_address);
    println!("   • Status: Running");
    println!();

    // Wait a bit before creating VM2
    println!("⏳ Waiting 5 seconds before creating VM2...");
    thread::sleep(Duration::from_secs(5));
    println!();

    // Create VM 2
    println!("═══════════════════════════════════════════════════════════");
    println!("Creating VM 2...");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    
    let vm2 = backend.create_desktop_vm(
        "biomeos-test-vm2",
        template_path,
        &cloud_init,
        2048,  // 2GB RAM
        2,     // 2 CPUs
        25,    // 25GB disk (template is 2.9GB, needs to be larger)
    ).await.context("Failed to create VM2")?;

    println!("✅ VM2 created!");
    println!("   • Name: {}", vm2.name);
    println!("   • IP: {}", vm2.ip_address);
    println!("   • Status: Running");
    println!();

    // Summary
    println!("═══════════════════════════════════════════════════════════");
    println!("Phase 1: SUCCESS! ✅");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("VMs Created:");
    println!("  • VM1: {} ({})", vm1.name, vm1.ip_address);
    println!("  • VM2: {} ({})", vm2.name, vm2.ip_address);
    println!();
    println!("Success Criteria:");
    println!("  ✅ 2 VMs created");
    println!("  ✅ VMs booted");
    println!("  ✅ VMs got IPs");
    println!();
    println!("Next Steps:");
    println!("  1. Test SSH access:");
    println!("     ssh biomeos@{}", vm1.ip_address);
    println!("     ssh biomeos@{}", vm2.ip_address);
    println!();
    println!("  2. Test ping between VMs:");
    println!("     ssh biomeos@{} ping -c 3 {}", vm1.ip_address, vm2.ip_address);
    println!();
    println!("  3. Once validated → Phase 2 (biomeOS deployment)");
    println!();
    println!("To cleanup:");
    println!("  sudo virsh destroy biomeos-test-vm1 biomeos-test-vm2");
    println!("  sudo virsh undefine biomeos-test-vm1 biomeos-test-vm2 --remove-all-storage");
    println!();

    Ok(())
}

