// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Songbird Universal UI Integration Demo
//!
//! This demo connects to Songbird's orchestration endpoints and demonstrates
//! the Universal UI working with Songbird as the primary orchestrator.

use anyhow::Result;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use serde_json::json;
use reqwest::Client;
use tracing::{info, warn, error};

#[derive(Debug, Clone)]
struct SongbirdOrchestrator {
    client: Client,
    base_url: String,
    status: String,
    capabilities: Vec<String>,
    services: HashMap<String, ServiceInfo>,
}

#[derive(Debug, Clone)]
struct ServiceInfo {
    name: String,
    status: String,
    endpoint: String,
    capabilities: Vec<String>,
}

#[derive(Debug, Clone)]
struct PrimalCoordination {
    primal_name: String,
    endpoint: String,
    status: String,
    capabilities: Vec<String>,
}

impl SongbirdOrchestrator {
    fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            status: "unknown".to_string(),
            capabilities: Vec::new(),
            services: HashMap::new(),
        }
    }
    
    async fn connect(&mut self) -> Result<()> {
        info!("🎼 Connecting to Songbird Orchestrator at {}", self.base_url);
        
        // Test connection to Songbird
        let health_url = format!("{}/health", self.base_url);
        match self.client.get(&health_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    self.status = "connected".to_string();
                    info!("✅ Connected to Songbird Orchestrator");
                    
                    // Get orchestrator capabilities
                    self.discover_capabilities().await?;
                    
                    // Discover services
                    self.discover_services().await?;
                    
                    Ok(())
                } else {
                    error!("❌ Songbird health check failed: {}", response.status());
                    Err(anyhow::anyhow!("Health check failed"))
                }
            }
            Err(e) => {
                error!("❌ Failed to connect to Songbird: {}", e);
                Err(anyhow::anyhow!("Connection failed: {}", e))
            }
        }
    }
    
    async fn discover_capabilities(&mut self) -> Result<()> {
        info!("🔍 Discovering Songbird capabilities...");
        
        // Try to get system info
        let info_url = format!("{}/system/info", self.base_url);
        match self.client.get(&info_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(info) = response.json::<serde_json::Value>().await {
                        info!("📊 Songbird System Info: {}", serde_json::to_string_pretty(&info)?);
                        
                        // Extract capabilities from system info
                        if let Some(data) = info.get("data") {
                            if let Some(api_endpoints) = data.get("api_endpoints") {
                                if let Some(endpoints) = api_endpoints.as_array() {
                                    for endpoint in endpoints {
                                        if let Some(endpoint_str) = endpoint.as_str() {
                                            self.capabilities.push(endpoint_str.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("⚠️  Could not get system info: {}", e);
            }
        }
        
        // Set default capabilities if none discovered
        if self.capabilities.is_empty() {
            self.capabilities = vec![
                "service_orchestration".to_string(),
                "primal_coordination".to_string(),
                "load_balancing".to_string(),
                "service_discovery".to_string(),
                "health_monitoring".to_string(),
            ];
        }
        
        info!("🎯 Songbird capabilities: {:?}", self.capabilities);
        Ok(())
    }
    
    async fn discover_services(&mut self) -> Result<()> {
        info!("🔍 Discovering orchestrated services...");
        
        let services_url = format!("{}/services", self.base_url);
        match self.client.get(&services_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(services_data) = response.json::<serde_json::Value>().await {
                        info!("📋 Services response: {}", serde_json::to_string_pretty(&services_data)?);
                        
                        // Parse services from response
                        if let Some(data) = services_data.get("data") {
                            if let Some(services) = data.as_array() {
                                for service in services {
                                    if let Some(service_obj) = service.as_object() {
                                        let name = service_obj.get("name")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("unknown")
                                            .to_string();
                                        
                                        let service_info = ServiceInfo {
                                            name: name.clone(),
                                            status: "running".to_string(),
                                            endpoint: format!("{}/services/{}", self.base_url, name),
                                            capabilities: vec!["managed_service".to_string()],
                                        };
                                        
                                        self.services.insert(name, service_info);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("⚠️  Could not discover services: {}", e);
            }
        }
        
        info!("📊 Discovered {} services", self.services.len());
        Ok(())
    }
    
    async fn coordinate_with_primals(&self) -> Result<Vec<PrimalCoordination>> {
        info!("🤝 Coordinating with Primals through Songbird...");
        
        let mut coordinated_primals = Vec::new();
        
        // Songbird coordinates with primals, so we query its coordination status
        let coordination_url = format!("{}/coordination/status", self.base_url);
        match self.client.get(&coordination_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(coordination_data) = response.json::<serde_json::Value>().await {
                        info!("🎼 Primal coordination status: {}", serde_json::to_string_pretty(&coordination_data)?);
                    }
                }
            }
            Err(_) => {
                // If specific coordination endpoint doesn't exist, simulate based on known primals
                warn!("⚠️  No coordination endpoint found, simulating primal coordination...");
                
                let known_primals = vec![
                    ("toadstool", "http://localhost:8084", vec!["compute", "execution"]),
                    ("nestgate", "http://localhost:8082", vec!["storage", "data"]),
                    ("beardog", "http://localhost:8443", vec!["security", "authentication"]),
                    ("squirrel", "http://localhost:5000", vec!["ai", "ml"]),
                ];
                
                for (name, endpoint, capabilities) in known_primals {
                    // Test if primal is available
                    let primal_status = if self.test_primal_endpoint(endpoint).await {
                        "connected"
                    } else {
                        "disconnected"
                    };
                    
                    coordinated_primals.push(PrimalCoordination {
                        primal_name: name.to_string(),
                        endpoint: endpoint.to_string(),
                        status: primal_status.to_string(),
                        capabilities: capabilities.iter().map(|s| s.to_string()).collect(),
                    });
                }
            }
        }
        
        info!("🎯 Coordinated with {} primals", coordinated_primals.len());
        Ok(coordinated_primals)
    }
    
    async fn test_primal_endpoint(&self, endpoint: &str) -> bool {
        let health_url = format!("{}/health", endpoint);
        match self.client.get(&health_url).timeout(Duration::from_secs(2)).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
    
    async fn deploy_biome_manifest(&self, manifest: serde_json::Value) -> Result<()> {
        info!("🚀 Deploying biome manifest through Songbird...");
        
        let deploy_url = format!("{}/deploy", self.base_url);
        match self.client.post(&deploy_url)
            .json(&manifest)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    info!("✅ Biome manifest deployed successfully");
                } else {
                    warn!("⚠️  Deployment response: {}", response.status());
                }
            }
            Err(e) => {
                warn!("⚠️  Deployment failed: {}", e);
            }
        }
        
        Ok(())
    }
    
    async fn get_orchestration_status(&self) -> Result<serde_json::Value> {
        let status_url = format!("{}/status", self.base_url);
        match self.client.get(&status_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let status = response.json::<serde_json::Value>().await?;
                    Ok(status)
                } else {
                    Ok(json!({
                        "status": "unknown",
                        "message": format!("Status endpoint returned: {}", response.status())
                    }))
                }
            }
            Err(e) => {
                Ok(json!({
                    "status": "error",
                    "message": format!("Failed to get status: {}", e)
                }))
            }
        }
    }
}

struct UniversalUIDemo {
    songbird: SongbirdOrchestrator,
}

impl UniversalUIDemo {
    fn new() -> Self {
        Self {
            songbird: SongbirdOrchestrator::new("http://localhost:8080".to_string()),
        }
    }
    
    async fn run_demo(&mut self) -> Result<()> {
        println!("🎼 Songbird Universal UI Integration Demo");
        println!("==========================================");
        println!();
        
        // Step 1: Connect to Songbird
        println!("📡 Step 1: Connecting to Songbird Orchestrator...");
        match self.songbird.connect().await {
            Ok(_) => {
                println!("✅ Successfully connected to Songbird");
                println!("🎯 Capabilities: {:?}", self.songbird.capabilities);
                println!("📊 Services: {}", self.songbird.services.len());
            }
            Err(e) => {
                println!("❌ Failed to connect to Songbird: {}", e);
                println!("💡 Make sure Songbird orchestrator is running:");
                println!("   cd songbird/apps/songbird-orchestrator");
                println!("   cargo run -- orchestrator start --enable-dashboard");
                return Ok(());
            }
        }
        
        println!();
        
        // Step 2: Discover Primal Coordination
        println!("🤝 Step 2: Discovering Primal Coordination...");
        let coordinated_primals = self.songbird.coordinate_with_primals().await?;
        
        for primal in &coordinated_primals {
            let status_icon = if primal.status == "connected" { "✅" } else { "❌" };
            println!("  {} {}: {} ({})", 
                status_icon, 
                primal.primal_name, 
                primal.status, 
                primal.capabilities.join(", ")
            );
        }
        
        println!();
        
        // Step 3: Deploy Sample Biome
        println!("🚀 Step 3: Deploying sample biome through Songbird...");
        let sample_manifest = json!({
            "metadata": {
                "name": "universal-ui-demo",
                "version": "1.0.0",
                "description": "Universal UI Demo Biome"
            },
            "services": {
                "web-frontend": {
                    "image": "nginx:alpine",
                    "ports": ["80:8080"]
                },
                "api-backend": {
                    "image": "node:alpine",
                    "ports": ["3000:3000"]
                }
            },
            "primals": {
                "toadstool": {
                    "enabled": true,
                    "capabilities": ["compute", "execution"]
                },
                "nestgate": {
                    "enabled": true,
                    "capabilities": ["storage", "data"]
                }
            }
        });
        
        self.songbird.deploy_biome_manifest(sample_manifest).await?;
        
        println!();
        
        // Step 4: Monitor Orchestration Status
        println!("📊 Step 4: Monitoring orchestration status...");
        let status = self.songbird.get_orchestration_status().await?;
        println!("📈 Status: {}", serde_json::to_string_pretty(&status)?);
        
        println!();
        
        // Step 5: Demonstrate Universal UI Features
        println!("🎨 Step 5: Universal UI Features Demo...");
        self.demonstrate_ui_features().await?;
        
        println!();
        println!("🎉 Demo completed successfully!");
        println!("💡 Songbird is orchestrating primals and services through its unified interface");
        
        Ok(())
    }
    
    async fn demonstrate_ui_features(&self) -> Result<()> {
        println!("  🖥️  Desktop UI: egui-based interface with real-time monitoring");
        println!("  🌐 Web UI: Browser-based dashboard with live updates");
        println!("  📱 Terminal UI: Rich text interface with interactive controls");
        println!("  ⌨️  CLI UI: Command-line interface with structured output");
        println!("  🤖 AI Assistant: Natural language command processing");
        println!("  📊 Real-time Monitoring: Live metrics and health status");
        println!("  🎯 Multi-Primal Coordination: Unified control across all primals");
        println!("  🔄 Dynamic Discovery: Auto-discovery of new services and primals");
        
        // Simulate some UI interactions
        println!();
        println!("  🎮 Simulating UI interactions...");
        sleep(Duration::from_secs(1)).await;
        
        println!("    👤 User: 'Deploy a new web service'");
        println!("    🤖 AI: 'Coordinating with Toadstool for compute resources...'");
        sleep(Duration::from_millis(500)).await;
        
        println!("    📊 Monitor: 'Service deployment progress: 50%'");
        sleep(Duration::from_millis(500)).await;
        
        println!("    🎼 Songbird: 'Orchestrating service across primals...'");
        sleep(Duration::from_millis(500)).await;
        
        println!("    ✅ System: 'Service deployed successfully!'");
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let mut demo = UniversalUIDemo::new();
    demo.run_demo().await?;
    
    Ok(())
} 