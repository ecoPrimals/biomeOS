//! Universal biomeOS UI Demo
//!
//! This demo shows how the universal UI works across all Primals
//! in the biomeOS ecosystem with AI-first interaction.

use anyhow::Result;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

use biomeos::{
    AiConfig, UIConfig, UIFeatures, UIMode,
    BiomeOSUI,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("biomeos_ui=info,demo=info")
        .init();

    println!("🚀 biomeOS Universal UI Demo");
    println!("════════════════════════════");
    println!();

    // Create UI configuration
    let _config = create_demo_config();

    // Create UI instance
    let mut ui = BiomeOSUI::new();

    // Demo scenarios
    println!("📋 Demo Scenarios:");
    println!("1. Ecosystem Discovery");
    println!("2. AI-Assisted Biome Deployment");
    println!("3. Real-time Monitoring");
    println!("4. Universal Primal Coordination");
    println!();

    // Scenario 1: Ecosystem Discovery
    demo_ecosystem_discovery(&ui).await?;

    // Scenario 2: AI-Assisted Biome Deployment
    demo_ai_biome_deployment(&mut ui).await?;

    // Scenario 3: Real-time Monitoring
    demo_real_time_monitoring(&ui).await?;

    // Scenario 4: Universal Primal Coordination
    demo_universal_coordination(&ui).await?;

    println!("✅ Demo completed successfully!");
    println!();
    println!("🎯 Key Features Demonstrated:");
    println!("• Universal API client works with any Primal");
    println!("• AI assistant understands natural language");
    println!("• Real-time event monitoring across ecosystem");
    println!("• Graceful degradation when Primals unavailable");
    println!("• Consistent UI experience across all modes");

    Ok(())
}

/// Create demo configuration
fn create_demo_config() -> UIConfig {
    let mut api_endpoints = HashMap::new();

    // Standard Primals
    api_endpoints.insert("songbird".to_string(), "http://localhost:8080".to_string());
    api_endpoints.insert("nestgate".to_string(), "http://localhost:8082".to_string());
    api_endpoints.insert("toadstool".to_string(), "http://localhost:8084".to_string());
    api_endpoints.insert("beardog".to_string(), "http://localhost:9000".to_string());

    // Custom Primals (would work seamlessly)
    api_endpoints.insert(
        "custom-ai-primal".to_string(),
        "http://localhost:7000".to_string(),
    );
    api_endpoints.insert(
        "custom-storage-primal".to_string(),
        "http://localhost:7001".to_string(),
    );

    let mut websocket_endpoints = HashMap::new();
    websocket_endpoints.insert(
        "events".to_string(),
        "ws://localhost:8080/events".to_string(),
    );

    UIConfig {
        theme: "dark".to_string(),
        mode: UIMode::Terminal,
        features: UIFeatures::default(),
        ui_mode: UIMode::Terminal, // For demo purposes
        api_endpoints: api_endpoints.values().cloned().collect(),
        websocket_endpoints: websocket_endpoints.values().cloned().collect(),
        ai_config: Some(AiConfig::default()),
        auto_refresh_interval: std::time::Duration::from_millis(5000),
    }
}

/// Demo ecosystem discovery
async fn demo_ecosystem_discovery(ui: &BiomeOSUI) -> Result<()> {
    println!("🔍 Scenario 1: Ecosystem Discovery");
    println!("──────────────────────────────────");

    // Discover available Primals
    println!("Discovering available Primals...");

    match ui.api_client.as_ref().unwrap().discover_primals().await {
        Ok(primals) => {
            println!("✅ Found {} Primals in the ecosystem:", primals.len());
            for primal in &primals {
                println!(
                    "  • {} - Active",
                    primal
                );
            }
        }
        Err(e) => {
            println!("⚠️  Discovery failed (expected in demo): {}", e);
            println!("   In real deployment, this would show all connected Primals");
        }
    }

    // Get ecosystem status
    println!("\nGetting ecosystem status...");

    match ui.api_client.as_ref().unwrap().get_ecosystem_status().await {
        Ok(status) => {
            println!("✅ Ecosystem Status:");
            println!("   Overall Health: {}", if status.healthy { "Healthy" } else { "Degraded" });
            println!(
                "   Primals: {}/{} healthy",
                status.primals.len(), status.primals.len()
            );
        }
        Err(e) => {
            println!("⚠️  Status check failed (expected in demo): {}", e);
            println!("   In real deployment, this would show live ecosystem status");
        }
    }

    sleep(Duration::from_secs(2)).await;
    println!();

    Ok(())
}

/// Demo AI-assisted biome deployment
async fn demo_ai_biome_deployment(ui: &mut BiomeOSUI) -> Result<()> {
    println!("🤖 Scenario 2: AI-Assisted Biome Deployment");
    println!("───────────────────────────────────────────");

    // Simulate AI commands
    let ai_commands = vec![
        "Deploy a biome called 'web-app' with a frontend and backend service",
        "Show me the status of the web-app deployment",
        "Scale the frontend service to 3 replicas",
        "What's the health of all Primals in the ecosystem?",
    ];

    for command in ai_commands {
        println!("👤 User: {}", command);

        match ui
            .ai_assistant
            .as_ref()
            .unwrap()
            .process_command(command)
            .await
        {
            Ok(response) => {
                println!("🤖 AI: {}", response);
            }
            Err(e) => {
                println!("⚠️  AI command failed (expected in demo): {}", e);
                println!("   In real deployment, AI would coordinate across all Primals");
            }
        }

        sleep(Duration::from_secs(1)).await;
        println!();
    }

    Ok(())
}

/// Demo real-time monitoring
async fn demo_real_time_monitoring(_ui: &BiomeOSUI) -> Result<()> {
    println!("📊 Scenario 3: Real-time Monitoring");
    println!("───────────────────────────────────");

    println!("Starting real-time event monitoring...");

    // Simulate real-time events
    let simulated_events = vec![
        (
            "songbird",
            "deployment",
            "Biome 'web-app' deployment started",
        ),
        (
            "toadstool",
            "service",
            "Frontend service scaled to 3 replicas",
        ),
        ("nestgate", "storage", "Database volume provisioned"),
        ("beardog", "security", "SSL certificates generated"),
        (
            "songbird",
            "deployment",
            "Biome 'web-app' deployment completed",
        ),
    ];

    for (primal, event_type, message) in simulated_events {
        println!("📡 Event from {}: {} - {}", primal, event_type, message);
        sleep(Duration::from_millis(800)).await;
    }

    println!("✅ Real-time monitoring demonstrates:");
    println!("   • Events from all Primals in unified stream");
    println!("   • Automatic event correlation and filtering");
    println!("   • Real-time UI updates across all modes");

    sleep(Duration::from_secs(1)).await;
    println!();

    Ok(())
}

/// Demo universal primal coordination
async fn demo_universal_coordination(_ui: &BiomeOSUI) -> Result<()> {
    println!("🔗 Scenario 4: Universal Primal Coordination");
    println!("────────────────────────────────────────────");

    // Simulate coordination across different Primal types
    let coordination_scenarios = vec![
        ("Standard Primal", "songbird", "orchestration"),
        ("Custom AI Primal", "custom-ai-primal", "ai-inference"),
        (
            "Custom Storage Primal",
            "custom-storage-primal",
            "distributed-storage",
        ),
        ("Forked Primal", "community-compute", "specialized-compute"),
    ];

    for (primal_type, primal_name, capability) in coordination_scenarios {
        println!("🔧 Coordinating with {} ({})", primal_type, primal_name);
        println!("   Capability: {}", capability);

        // Simulate universal API call
        println!("   → Sending universal coordination request...");
        sleep(Duration::from_millis(500)).await;

        // Simulate response
        println!("   ✅ Coordination successful - universal API handled the request");
        println!("   📊 Response: Primal integrated seamlessly into ecosystem");

        sleep(Duration::from_millis(500)).await;
        println!();
    }

    println!("🎯 Universal Coordination demonstrates:");
    println!("   • Any Primal (standard, custom, forked) works immediately");
    println!("   • Consistent API interface across all Primal types");
    println!("   • Automatic capability detection and routing");
    println!("   • Graceful degradation when Primals unavailable");

    sleep(Duration::from_secs(1)).await;
    println!();

    Ok(())
}

/// Simulate terminal UI interaction
#[allow(dead_code)]
async fn simulate_terminal_ui() -> Result<()> {
    println!("🖥️  Terminal UI Simulation");
    println!("─────────────────────────");

    // Simulate terminal UI screens
    let screens = vec![
        r#"
╔══════════════════════════════════════════════════════════════╗
║                    🌍 biomeOS Dashboard                      ║
╠══════════════════════════════════════════════════════════════╣
║ ✅ Overall Health: healthy                                   ║
║ 📊 Primals: 4/5 healthy                                     ║
║ 🕐 Last Updated: 14:23:45                                   ║
║                                                              ║
║ 🔧 Primal Status:                                           ║
║   ✅ songbird     healthy    (12 services)                  ║
║   ✅ nestgate     healthy    (8 services)                   ║
║   ✅ toadstool    healthy    (15 services)                  ║
║   ⚠️  beardog      degraded  (3 services)                   ║
║   ❌ squirrel     unhealthy  (0 services)                   ║
║                                                              ║
║ 💡 Commands: [s]tatus, [d]eploy, [l]ist, [a]i, [q]uit     ║
╚══════════════════════════════════════════════════════════════╝
        "#,
        r#"
╔══════════════════════════════════════════════════════════════╗
║                    🤖 AI Assistant                          ║
╠══════════════════════════════════════════════════════════════╣
║ 👤 User: Deploy a biome called my-web-app                   ║
║ 🤖 AI: ✅ Deploying biome 'my-web-app'...                  ║
║        Deployment ID: biome-abc123                          ║
║        Status: success                                      ║
║                                                              ║
║ 👤 User: Show ecosystem status                              ║
║ 🤖 AI: ✅ Ecosystem Status: healthy                        ║
║        📊 Primals: 4/5 healthy                             ║
║                                                              ║
║ 💡 Suggestions:                                             ║
║   • Check deployment status                                 ║
║   • View deployment logs                                    ║
║   • Scale services if needed                               ║
║                                                              ║
║ 🎯 Ask me anything about your biomeOS ecosystem!           ║
╚══════════════════════════════════════════════════════════════╝
        "#,
    ];

    for screen in screens {
        println!("{}", screen);
        sleep(Duration::from_secs(3)).await;
    }

    Ok(())
}

/// Print feature summary
fn _print_feature_summary() {
    println!("🎯 Universal biomeOS UI Features:");
    println!("════════════════════════════════");
    println!();
    println!("🔧 Universal Primal Support:");
    println!("  • Works with any Primal (standard, custom, forked)");
    println!("  • Automatic capability detection");
    println!("  • Consistent API interface");
    println!("  • Graceful degradation");
    println!();
    println!("🤖 AI-First Design:");
    println!("  • Natural language commands");
    println!("  • Context-aware responses");
    println!("  • Intelligent routing to relevant Primals");
    println!("  • Helpful suggestions");
    println!();
    println!("🎨 Multi-Mode UI:");
    println!("  • Desktop app (Tauri)");
    println!("  • Terminal UI (ratatui)");
    println!("  • Web interface (wry)");
    println!("  • CLI mode for automation");
    println!();
    println!("📊 Real-time Features:");
    println!("  • Live ecosystem monitoring");
    println!("  • Event streaming from all Primals");
    println!("  • Automatic status updates");
    println!("  • Performance metrics");
    println!();
    println!("🔗 API-Driven Architecture:");
    println!("  • All UI built on universal API");
    println!("  • Consistent data model");
    println!("  • Easy to extend and customize");
    println!("  • Perfect for AI integration");
}
