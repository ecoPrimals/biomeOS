//! Universal biomeOS Installation Example
//!
//! This example demonstrates the "grandma-safe", AI-first installation process
//! that works across any platform where Toadstool can provide universal compute.

use biomeos_core::{UniversalInstaller, BiomeResult};

#[tokio::main]
async fn main() -> BiomeResult<()> {
    println!("🌱 biomeOS Universal Installer Demo");
    println!("===================================");
    println!();
    println!("This demonstrates biomeOS as a universal platform for the next era:");
    println!("• Safe by default with AI-first configuration");
    println!("• OS agnostic: bare metal, Windows, Linux, any platform");
    println!("• Primal agnostic: mix and match any Primals, new or old");
    println!("• Each layer adds more universality");
    println!();
    
    // Create a new universal installer with grandma-safe defaults
    let mut installer = UniversalInstaller::new();
    
    // Run the AI-guided installation process
    installer.install_with_ai_guidance().await?;
    
    println!();
    println!("🎯 Key Universal Platform Features Demonstrated:");
    println!();
    println!("1. 🧠 AI-First Experience:");
    println!("   • Automatic platform detection and optimization");
    println!("   • Personalized recommendations based on hardware");
    println!("   • Grandma-safe explanations and guidance");
    println!("   • Zero-configuration security (MYCORRHIZA closed system)");
    println!();
    println!("2. 🌍 OS Agnosticism:");
    println!("   • Works on bare metal, Windows, Linux, containers, cloud");
    println!("   • Toadstool provides universal compute abstraction");
    println!("   • Platform-specific optimizations automatically applied");
    println!("   • Each layer adds more universality");
    println!();
    println!("3. 🔧 Primal Agnosticism:");
    println!("   • Universal Primal trait - any system can participate");
    println!("   • Mix and match current, future, or community Primals");
    println!("   • Dynamic discovery and installation of new Primals");
    println!("   • Capability-based orchestration");
    println!();
    println!("4. 🔒 MYCORRHIZA Energy Flow Management:");
    println!("   • Closed system (default): Complete sovereignty");
    println!("   • Private open: Trust-based external access");
    println!("   • Commercial open: Pay-to-play cloud integration");
    println!("   • Deep packet inspection and threat response");
    println!();
    println!("5. 🚀 Next Era Computing:");
    println!("   • Biological computing metaphors and patterns");
    println!("   • Self-healing and adaptive behaviors");
    println!("   • Personal digital sovereignty");
    println!("   • AI-native from the ground up");
    println!();
    
    // Demonstrate the layered universality concept
    demonstrate_layered_universality().await?;
    
    Ok(())
}

/// Demonstrate how each layer adds more universality
async fn demonstrate_layered_universality() -> BiomeResult<()> {
    println!("🏗️ Layered Universality Architecture:");
    println!();
    
    println!("Layer 0: Hardware (Any Hardware)");
    println!("├─ 💻 Works on any CPU architecture");
    println!("├─ 🗄️ Supports any storage technology");
    println!("├─ 🌐 Uses any network interface");
    println!("└─ ⚡ Adapts to any power constraints");
    println!();
    
    println!("Layer 1: Toadstool Universal Compute");
    println!("├─ 🐧 Linux: Native containers, systemd integration");
    println!("├─ 🪟 Windows: Process isolation, Service Manager");
    println!("├─ 🍎 macOS: App Sandbox, launchd services");
    println!("├─ 🚢 Container: Docker, Podman, any runtime");
    println!("├─ ☁️ Cloud: AWS, GCP, Azure, any provider");
    println!("└─ 🔩 Bare Metal: Direct hardware management");
    println!();
    
    println!("Layer 2: Primal Ecosystem (Universal Services)");
    println!("├─ 🎼 Songbird: Universal service mesh");
    println!("├─ 🏰 NestGate: Universal storage management");
    println!("├─ 🐕 BearDog: Universal security framework");
    println!("├─ 🐿️ Squirrel: Universal AI platform");
    println!("└─ 🔌 Any Primal: Community or enterprise extensions");
    println!();
    
    println!("Layer 3: Application Universe");
    println!("├─ 🧬 Biological computing applications");
    println!("├─ 🤖 AI/ML workloads and agents");
    println!("├─ 🔬 Scientific computing and simulations");
    println!("├─ 🏢 Enterprise applications and services");
    println!("├─ 🎮 Interactive and media applications");
    println!("└─ 🌟 Future applications we can't imagine yet");
    println!();
    
    println!("🎯 The Result: True Universal Computing");
    println!("• Write once, run anywhere (any OS, any hardware)");
    println!("• Compose freely (any Primals, any services)");
    println!("• Scale infinitely (edge to cloud to quantum)");
    println!("• Secure by default (MYCORRHIZA protection)");
    println!("• AI-assisted everything (grandma to expert)");
    
    Ok(())
} 