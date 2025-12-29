//! VM Provisioning using benchScale v2.0.0
//!
//! This binary provisions VMs for biomeOS validation using the proper
//! benchScale API with cloud-init validation built-in.

use anyhow::{Context, Result};
use benchscale::{CloudInit, LibvirtBackend};
use biomeos_validation::{get_template_path, load_ssh_public_key, print_header, print_section};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    print_header("🦀 biomeOS VM Provisioning (benchScale v2.0.0) 🦀");

    println!("Using proper tool integration:");
    println!("  • benchScale v2.0.0 API");
    println!("  • create_desktop_vm_ready() - guaranteed SSH access");
    println!("  • No timing assumptions");
    println!("  • Framework validates everything");
    println!();

    // Get template path
    let template_path = get_template_path();
    if !template_path.exists() {
        anyhow::bail!(
            "Template not found at: {}. Please ensure agentReagents is cloned and template is built.",
            template_path.display()
        );
    }
    println!("✅ Template found: {}", template_path.display());

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

    print_section("Creating VM 1 (with SSH validation)");
    println!("Using: create_desktop_vm_ready()");
    println!("  • Provisions VM");
    println!("  • Waits for cloud-init");
    println!("  • Validates SSH access");
    println!("  • Returns when READY ✅");
    println!();

    let vm1 = backend
        .create_desktop_vm_ready(
            "biomeos-test-vm1",
            &template_path,
            &cloud_init,
            2048, // 2GB RAM
            2,    // 2 CPUs
            25,   // 25GB disk
        )
        .await
        .context("Failed to create VM1")?;

    println!("✅ VM1 ready!");
    println!("   • Name: {}", vm1.name);
    println!("   • IP: {}", vm1.ip_address);
    println!("   • SSH: VALIDATED ✅");
    println!();

    print_section("Creating VM 2 (with SSH validation)");

    let vm2 = backend
        .create_desktop_vm_ready(
            "biomeos-test-vm2",
            &template_path,
            &cloud_init,
            2048, // 2GB RAM
            2,    // 2 CPUs
            25,   // 25GB disk
        )
        .await
        .context("Failed to create VM2")?;

    println!("✅ VM2 ready!");
    println!("   • Name: {}", vm2.name);
    println!("   • IP: {}", vm2.ip_address);
    println!("   • SSH: VALIDATED ✅");
    println!();

    print_section("Phase 1: SUCCESS! ✅");
    println!("VMs Created and Validated:");
    println!("  • VM1: {} ({})", vm1.name, vm1.ip_address);
    println!("  • VM2: {} ({})", vm2.name, vm2.ip_address);
    println!();
    println!("Success Criteria:");
    println!("  ✅ 2 VMs created");
    println!("  ✅ VMs booted");
    println!("  ✅ VMs got IPs");
    println!("  ✅ Cloud-init completed");
    println!("  ✅ SSH access validated");
    println!();
    println!("Validation Quality:");
    println!("  ✅ No timing assumptions");
    println!("  ✅ Framework-level validation");
    println!("  ✅ Clear error messages");
    println!("  ✅ Proper tool usage");
    println!();
    println!("Next Steps:");
    println!("  1. SSH to VMs:");
    println!("     ssh biomeos@{}", vm1.ip_address);
    println!("     ssh biomeos@{}", vm2.ip_address);
    println!();
    println!("  2. Test connectivity:");
    println!(
        "     ssh biomeos@{} ping -c 3 {}",
        vm1.ip_address, vm2.ip_address
    );
    println!();
    println!("  3. Phase 2: Deploy biomeOS");
    println!("     cargo run --bin validate-federation");
    println!();
    println!("To cleanup:");
    println!("  sudo virsh destroy {} {}", vm1.name, vm2.name);
    println!(
        "  sudo virsh undefine {} {} --remove-all-storage",
        vm1.name, vm2.name
    );
    println!();

    print_header("✅ Validation Complete! ✅");
    println!("This demonstrates PROPER tool usage:");
    println!("  ✅ benchScale v2.0.0 API");
    println!("  ✅ Cloud-init validation built-in");
    println!("  ✅ No workarounds needed");
    println!("  ✅ Type-safe, testable");
    println!("  ✅ Clear separation of concerns");
    println!();
    println!("\"A carpenter uses a hammer, but doesn't become the hammer\"");
    println!();

    Ok(())
}

