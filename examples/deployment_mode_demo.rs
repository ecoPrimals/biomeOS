//! Deployment Mode Detection Demo
//!
//! Demonstrates how biomeOS detects its deployment mode and adapts socket paths accordingly.
//!
//! # Usage
//!
//! ```bash
//! # Default (Sibling Spore)
//! cargo run --example deployment_mode_demo
//!
//! # Cold Spore (USB)
//! BIOMEOS_DEPLOYMENT_MODE=cold BIOMEOS_MEDIA_PATH=/media/usb0 \
//!     cargo run --example deployment_mode_demo
//!
//! # Live Spore (Installed)
//! BIOMEOS_DEPLOYMENT_MODE=live \
//!     cargo run --example deployment_mode_demo
//! ```

use biomeos_core::deployment_mode::DeploymentMode;

fn main() {
    println!("🌱 LiveSpore Deployment Mode Detection Demo\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Detect the current deployment mode
    match DeploymentMode::detect() {
        Ok(mode) => {
            println!("✅ Deployment Mode Detected:");
            println!("   {}\n", mode.description());

            // Show socket configuration
            let socket_prefix = mode.socket_prefix();
            println!("🔌 Socket Configuration:");
            println!("   Base Path: {}", socket_prefix.display());
            println!("   Example Sockets:");
            println!(
                "     - beardog:   {}/beardog-nat0.sock",
                socket_prefix.display()
            );
            println!(
                "     - songbird:  {}/songbird-nat0.sock",
                socket_prefix.display()
            );
            println!(
                "     - toadstool: {}/toadstool-nat0.sock",
                socket_prefix.display()
            );
            println!(
                "     - nestgate:  {}/nestgate-nat0.sock\n",
                socket_prefix.display()
            );

            // Show mode-specific information
            match mode {
                DeploymentMode::ColdSpore {
                    media_path,
                    persistence,
                    host_os,
                } => {
                    println!("📦 Cold Spore Details:");
                    println!("   Media Path: {}", media_path.display());
                    println!(
                        "   Persistence: {}",
                        if persistence { "Enabled" } else { "Ephemeral" }
                    );
                    println!("   Host OS: {}\n", host_os.name());

                    println!("💡 Cold Spore Mode:");
                    println!("   - Running from removable media (USB/SD)");
                    println!("   - No installation required");
                    println!(
                        "   - {} data storage",
                        if persistence {
                            "Persistent"
                        } else {
                            "Ephemeral"
                        }
                    );
                    println!("   - Portable across machines\n");
                }
                DeploymentMode::LiveSpore {
                    root_partition,
                    boot_partition,
                    installed_version,
                } => {
                    println!("💿 Live Spore Details:");
                    println!("   Root: {}", root_partition.display());
                    println!("   Boot: {}", boot_partition.display());
                    println!("   Version: {}\n", installed_version);

                    println!("💡 Live Spore Mode:");
                    println!("   - Installed to bare metal");
                    println!("   - Full hardware access");
                    println!("   - Maximum performance");
                    println!("   - Persistent storage on disk\n");
                }
                DeploymentMode::SiblingSpore {
                    host_os,
                    install_dir,
                    isolation,
                } => {
                    println!("🤝 Sibling Spore Details:");
                    println!("   Host OS: {}", host_os.name());
                    println!("   Install Dir: {}", install_dir.display());
                    println!("   Isolation: {:?}\n", isolation);

                    println!("💡 Sibling Spore Mode:");
                    println!("   - Running on top of existing OS");
                    println!("   - No repartitioning required");
                    println!("   - Coexists with host OS");
                    println!("   - Can discover bare metal biomeOS nodes\n");
                }
            }
        }
        Err(e) => {
            println!("❌ Error detecting deployment mode: {}\n", e);
            println!("This might happen if:");
            println!("  - Running in an unsupported environment");
            println!("  - Missing required permissions");
            println!("  - Environment variables misconfigured\n");
        }
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("💡 Environment Variable Overrides:");
    println!("   BIOMEOS_DEPLOYMENT_MODE=cold|live|sibling");
    println!("   BIOMEOS_MEDIA_PATH=/media/usb0  (for cold)");
    println!("   BIOMEOS_PERSISTENCE=true        (for cold)");
    println!("   BIOMEOS_INSTALL_DIR=~/biomeos   (for sibling)");
    println!("   BIOMEOS_ISOLATION=sandboxed|shared|full\n");

    println!("Different orders of the same architecture. 🍄🐸🌱\n");
}
