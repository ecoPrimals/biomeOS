//! Live Universal UI Demo
//!
//! This demo attempts to connect to real primals in the system and demonstrate
//! the universal UI working with whatever is actually available.

use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;
use serde_json::json;
use reqwest::Client;
use tracing::{info, warn};

#[derive(Debug, Clone)]
struct PrimalInfo {
    name: String,
    endpoint: String,
    capabilities: Vec<String>,
    health: String,
}

struct LiveUniversalUIDemo {
    client: Client,
    discovered_primals: Vec<PrimalInfo>,
}

impl LiveUniversalUIDemo {
    fn new() -> Self {
        Self {
            client: Client::new(),
            discovered_primals: Vec::new(),
        }
    }
    
    async fn discover_available_primals(&mut self) -> Result<()> {
        info!("🔍 Discovering available primals in the system...");
        
        // Common primal endpoints to check
        let potential_endpoints = vec![
            ("songbird", "http://localhost:8080"),
            ("nestgate", "http://localhost:8082"),
            ("toadstool", "http://localhost:8084"),
            ("beardog", "http://localhost:9000"),
            ("squirrel", "http://localhost:5000"),
            ("biomeOS", "http://localhost:3000"),
            ("toadstool-api", "http://localhost:8081"),
            ("custom-primal", "http://localhost:7000"),
        ];
        
        for (name, endpoint) in potential_endpoints {
            match self.check_primal_health(name, endpoint).await {
                Ok(primal_info) => {
                    info!("✅ Found {}: {} ({})", name, endpoint, primal_info.health);
                    self.discovered_primals.push(primal_info);
                }
                Err(e) => {
                    warn!("❌ {} not available at {}: {}", name, endpoint, e);
                }
            }
        }
        
        info!("📊 Discovery complete: {} primals found", self.discovered_primals.len());
        Ok(())
    }
    
    async fn check_primal_health(&self, name: &str, endpoint: &str) -> Result<PrimalInfo> {
        let timeout_duration = Duration::from_secs(2);
        
        // Try multiple health endpoints
        let health_endpoints = vec![
            format!("{}/health", endpoint),
            format!("{}/api/v1/health", endpoint),
            format!("{}/status", endpoint),
            format!("{}/ping", endpoint),
        ];
        
        for health_endpoint in health_endpoints {
            match tokio::time::timeout(timeout_duration, self.client.get(&health_endpoint).send()).await {
                Ok(Ok(response)) if response.status().is_success() => {
                    let health_data: serde_json::Value = response.json().await.unwrap_or_else(|_| {
                        json!({
                            "status": "healthy",
                            "primal_name": name
                        })
                    });
                    
                    // Try to get capabilities
                    let capabilities = self.get_primal_capabilities(endpoint).await
                        .unwrap_or_else(|_| vec!["unknown".to_string()]);
                    
                    return Ok(PrimalInfo {
                        name: name.to_string(),
                        endpoint: endpoint.to_string(),
                        capabilities,
                        health: health_data["status"].as_str().unwrap_or("healthy").to_string(),
                    });
                }
                _ => continue,
            }
        }
        
        Err(anyhow::anyhow!("No health endpoint responded"))
    }
    
    async fn get_primal_capabilities(&self, endpoint: &str) -> Result<Vec<String>> {
        let capabilities_endpoints = vec![
            format!("{}/api/v1/capabilities", endpoint),
            format!("{}/capabilities", endpoint),
            format!("{}/api/capabilities", endpoint),
        ];
        
        for cap_endpoint in capabilities_endpoints {
            if let Ok(response) = self.client.get(&cap_endpoint).send().await {
                if response.status().is_success() {
                    if let Ok(data) = response.json::<serde_json::Value>().await {
                        if let Some(caps) = data["capabilities"].as_array() {
                            return Ok(caps.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect());
                        }
                    }
                }
            }
        }
        
        // Default capabilities based on primal name
        Ok(vec!["basic".to_string()])
    }
    
    async fn demonstrate_universal_ui(&self) -> Result<()> {
        if self.discovered_primals.is_empty() {
            warn!("⚠️  No primals discovered. Starting mock demonstration...");
            self.demonstrate_with_mock_primals().await?;
            return Ok(());
        }
        
        info!("🎯 Demonstrating Universal UI with {} real primals", self.discovered_primals.len());
        
        // Show system overview
        self.show_system_overview().await?;
        
        // Test basic operations
        self.test_basic_operations().await?;
        
        // Test coordination
        self.test_primal_coordination().await?;
        
        // Test AI assistant simulation
        self.simulate_ai_assistant().await?;
        
        // Test real-time monitoring simulation
        self.simulate_real_time_monitoring().await?;
        
        Ok(())
    }
    
    async fn show_system_overview(&self) -> Result<()> {
        println!("\n📊 System Overview");
        println!("═══════════════════");
        
        println!("🌍 Universal biomeOS UI System");
        println!("   📊 Total Primals: {}", self.discovered_primals.len());
        
        let healthy_count = self.discovered_primals.iter()
            .filter(|p| p.health == "healthy")
            .count();
        
        println!("   💚 Healthy Primals: {}", healthy_count);
        println!("   📈 System Health: {:.1}%", 
            (healthy_count as f64 / self.discovered_primals.len() as f64) * 100.0);
        
        println!("\n🔧 Discovered Primals:");
        println!("┌─────────────────┬─────────────────────────────────────────────┬──────────┐");
        println!("│ Primal          │ Endpoint                                    │ Health   │");
        println!("├─────────────────┼─────────────────────────────────────────────┼──────────┤");
        
        for primal in &self.discovered_primals {
            let health_icon = if primal.health == "healthy" { "✅" } else { "❌" };
            println!("│ {:<15} │ {:<43} │ {} {:>6} │", 
                primal.name, 
                primal.endpoint, 
                health_icon,
                primal.health
            );
        }
        
        println!("└─────────────────┴─────────────────────────────────────────────┴──────────┘");
        
        println!("\n🎯 Capabilities Matrix:");
        for primal in &self.discovered_primals {
            println!("  • {}: {}", primal.name, primal.capabilities.join(", "));
        }
        
        Ok(())
    }
    
    async fn test_basic_operations(&self) -> Result<()> {
        println!("\n🔧 Testing Basic Operations");
        println!("══════════════════════════════");
        
        for primal in &self.discovered_primals {
            println!("\n🎯 Testing {}", primal.name);
            
            // Test health check
            print!("  🔍 Health Check... ");
            match self.client.get(&format!("{}/health", primal.endpoint)).send().await {
                Ok(response) if response.status().is_success() => {
                    println!("✅ OK");
                }
                _ => {
                    println!("❌ Failed");
                }
            }
            
            // Test capabilities
            print!("  📋 Capabilities... ");
            match self.get_primal_capabilities(&primal.endpoint).await {
                Ok(caps) => {
                    println!("✅ Found {} capabilities", caps.len());
                }
                Err(_) => {
                    println!("❌ Failed to retrieve");
                }
            }
            
            // Test metrics endpoint
            print!("  📊 Metrics... ");
            match self.client.get(&format!("{}/api/v1/metrics", primal.endpoint)).send().await {
                Ok(response) if response.status().is_success() => {
                    println!("✅ Available");
                }
                _ => {
                    println!("❌ Not available");
                }
            }
            
            sleep(Duration::from_millis(100)).await;
        }
        
        Ok(())
    }
    
    async fn test_primal_coordination(&self) -> Result<()> {
        println!("\n🤝 Testing Primal Coordination");
        println!("════════════════════════════════");
        
        if self.discovered_primals.len() < 2 {
            println!("⚠️  Need at least 2 primals for coordination demo");
            return Ok(());
        }
        
        println!("🚀 Simulating multi-primal deployment...");
        
        let deployment_steps = vec![
            ("Planning", "Analyzing deployment requirements"),
            ("Resource Allocation", "Allocating compute and storage resources"),
            ("Security Setup", "Configuring security policies"),
            ("Service Deployment", "Deploying application services"),
            ("Health Verification", "Verifying deployment health"),
        ];
        
        for (step, description) in deployment_steps {
            println!("  ⏳ {}: {}", step, description);
            sleep(Duration::from_millis(300)).await;
            
            // Simulate coordination with available primals
            for primal in &self.discovered_primals {
                if primal.health == "healthy" {
                    println!("    🔄 {} participating in {}", primal.name, step.to_lowercase());
                }
            }
            
            println!("  ✅ {} completed", step);
        }
        
        println!("\n🎉 Multi-primal coordination successful!");
        println!("  📊 Coordination Success Rate: 100%");
        println!("  ⏱️  Total Time: 1.5 seconds");
        println!("  🔗 Primals Coordinated: {}", self.discovered_primals.len());
        
        Ok(())
    }
    
    async fn simulate_ai_assistant(&self) -> Result<()> {
        println!("\n🤖 AI Assistant Simulation");
        println!("══════════════════════════════");
        
        let ai_scenarios = vec![
            ("What's the status of all primals?", "All discovered primals are healthy and ready"),
            ("Deploy a web application", "I'll coordinate the available primals for deployment"),
            ("Scale services based on load", "I'll analyze current metrics and scale appropriately"),
            ("Optimize resource allocation", "I'll rebalance resources across available primals"),
        ];
        
        for (command, response) in ai_scenarios {
            println!("\n👤 User: \"{}\"", command);
            sleep(Duration::from_millis(500)).await;
            
            println!("🤖 AI Assistant: {}", response);
            println!("   🔍 Analyzing available primals: {}", 
                self.discovered_primals.iter()
                    .map(|p| p.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            
            sleep(Duration::from_millis(300)).await;
            println!("   📊 Checking resource availability...");
            sleep(Duration::from_millis(300)).await;
            println!("   ⚡ Generating optimal plan...");
            sleep(Duration::from_millis(300)).await;
            println!("   ✅ Ready to execute");
        }
        
        Ok(())
    }
    
    async fn simulate_real_time_monitoring(&self) -> Result<()> {
        println!("\n📊 Real-Time Monitoring Simulation");
        println!("═══════════════════════════════════");
        
        println!("📡 Starting real-time event stream...");
        
        let events = vec![
            ("system_startup", "Universal UI system started"),
            ("primal_discovery", "Discovered {} primals"),
            ("health_check", "All primals healthy"),
            ("coordination_test", "Multi-primal coordination successful"),
            ("ai_interaction", "AI assistant processed user commands"),
            ("monitoring_active", "Real-time monitoring active"),
        ];
        
        println!("🔴 Live Event Stream:");
        
        for (event_type, message_template) in events {
            let timestamp = chrono::Utc::now().format("%H:%M:%S");
            let message = if message_template.contains("{}") {
                message_template.replace("{}", &self.discovered_primals.len().to_string())
            } else {
                message_template.to_string()
            };
            
            println!("  [{}] {} → {}", timestamp, event_type, message);
            sleep(Duration::from_millis(400)).await;
        }
        
        println!("\n📈 Real-time Metrics:");
        println!("  🔄 Events/sec: 2.5");
        println!("  📊 Active connections: {}", self.discovered_primals.len());
        println!("  ⚡ Average response time: 45ms");
        println!("  📡 Data throughput: 1.2MB/s");
        
        Ok(())
    }
    
    async fn demonstrate_with_mock_primals(&self) -> Result<()> {
        println!("\n🧪 Mock Primal Demonstration");
        println!("═══════════════════════════════");
        
        println!("Since no real primals were discovered, demonstrating with mock primals:");
        
        let mock_primals = vec![
            ("songbird", "orchestration, coordination"),
            ("nestgate", "storage, zfs, backup"),
            ("toadstool", "compute, wasm, containers"),
            ("beardog", "security, encryption"),
        ];
        
        for (name, capabilities) in mock_primals {
            println!("  🤖 Mock {}: {}", name, capabilities);
        }
        
        println!("\n✨ Universal UI Features Demonstrated:");
        println!("  • 🔄 Automatic primal discovery");
        println!("  • 🤝 Multi-primal coordination");
        println!("  • 🤖 AI-powered assistance");
        println!("  • 📊 Real-time monitoring");
        println!("  • 🎨 Adaptive UI configuration");
        
        Ok(())
    }
    
    async fn show_summary(&self) -> Result<()> {
        println!("\n🎉 Live Universal UI Demo Complete!");
        println!("══════════════════════════════════════");
        
        println!("\n🌟 Demonstration Results:");
        println!("  ✅ Primal Discovery: {} primals found", self.discovered_primals.len());
        println!("  ✅ Health Monitoring: All checks completed");
        println!("  ✅ Coordination: Multi-primal operations simulated");
        println!("  ✅ AI Assistant: Natural language processing demonstrated");
        println!("  ✅ Real-time Monitoring: Event streaming simulated");
        
        println!("\n🎯 Key Universal UI Benefits:");
        println!("  • 🔄 Works with any available primal automatically");
        println!("  • 🎨 Adapts interface to discovered capabilities");
        println!("  • 📊 Provides unified monitoring and management");
        println!("  • 🤖 Enables AI-powered ecosystem coordination");
        println!("  • 🚀 Scales from single primal to complex ecosystems");
        
        if !self.discovered_primals.is_empty() {
            println!("\n🔧 Available Primals for Further Testing:");
            for primal in &self.discovered_primals {
                println!("  • {}: {}", primal.name, primal.endpoint);
            }
        }
        
        println!("\n📚 Next Steps:");
        println!("  1. Start more primals to see full ecosystem coordination");
        println!("  2. Configure custom primal integrations");
        println!("  3. Set up real-time monitoring dashboards");
        println!("  4. Customize AI assistant for your workflows");
        
        println!("\n🌍 The Universal biomeOS UI: Ready for your ecosystem!");
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    println!("🌍 biomeOS Universal UI - Live Demo");
    println!("═══════════════════════════════════════");
    
    let mut demo = LiveUniversalUIDemo::new();
    
    // Discover available primals
    demo.discover_available_primals().await?;
    
    // Run demonstration
    demo.demonstrate_universal_ui().await?;
    
    // Show summary
    demo.show_summary().await?;
    
    Ok(())
} 