//! Universal API Client for biomeOS Ecosystem
//!
//! This module provides a unified API client that can communicate with any Primal
//! in the biomeOS ecosystem using the universal adapter pattern.

use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{info, warn, error};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Universal API client for the biomeOS ecosystem
#[derive(Debug, Clone)]
pub struct UniversalApiClient {
    /// HTTP client for making requests
    client: Client,
    
    /// Known Primal endpoints
    primal_endpoints: HashMap<String, String>,
    
    /// Connected Primal adapters
    primal_adapters: HashMap<String, PrimalAdapter>,
    
    /// API timeout configuration
    timeout: Duration,
}

impl UniversalApiClient {
    /// Create a new universal API client
    pub async fn new(endpoints: &HashMap<String, String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
            
        let mut api_client = Self {
            client,
            primal_endpoints: endpoints.clone(),
            primal_adapters: HashMap::new(),
            timeout: Duration::from_secs(30),
        };
        
        // Discover and validate endpoints
        api_client.validate_endpoints().await;
        
        Ok(api_client)
    }
    
    /// Discover all available Primals in the ecosystem
    pub async fn discover_primals(&self) -> Result<Vec<PrimalInfo>> {
        let mut discovered_primals = Vec::new();
        
        for (name, endpoint) in &self.primal_endpoints {
            match self.probe_primal_endpoint(name, endpoint).await {
                Ok(primal_info) => {
                    info!("✅ Discovered Primal: {} at {}", name, endpoint);
                    discovered_primals.push(primal_info);
                }
                Err(e) => {
                    warn!("⚠️  Primal {} at {} unavailable: {}", name, endpoint, e);
                    // Continue discovering other Primals - graceful degradation
                }
            }
        }
        
        Ok(discovered_primals)
    }
    
    /// Connect to a specific Primal and create an adapter
    pub async fn connect_to_primal(&mut self, primal_info: &PrimalInfo) -> Result<PrimalAdapter> {
        let adapter = PrimalAdapter::new(
            primal_info.clone(),
            self.client.clone(),
        ).await?;
        
        // Test the connection
        adapter.health_check().await?;
        
        self.primal_adapters.insert(primal_info.name.clone(), adapter.clone());
        
        Ok(adapter)
    }
    
    /// Get ecosystem-wide status
    pub async fn get_ecosystem_status(&self) -> Result<EcosystemStatus> {
        let mut primal_statuses = HashMap::new();
        let mut total_services = 0;
        let mut healthy_services = 0;
        
        for (name, adapter) in &self.primal_adapters {
            match adapter.get_status().await {
                Ok(status) => {
                    total_services += status.service_count;
                    if status.health == "healthy" {
                        healthy_services += status.service_count;
                    }
                    primal_statuses.insert(name.clone(), status);
                }
                Err(e) => {
                    warn!("Failed to get status from {}: {}", name, e);
                    primal_statuses.insert(name.clone(), PrimalStatus {
                        name: name.clone(),
                        health: "unavailable".to_string(),
                        service_count: 0,
                        last_seen: Utc::now(),
                        capabilities: vec![],
                        version: "unknown".to_string(),
                    });
                }
            }
        }
        
        let overall_health = if healthy_services == total_services && total_services > 0 {
            "healthy"
        } else if healthy_services > 0 {
            "degraded"
        } else {
            "unhealthy"
        };
        
        Ok(EcosystemStatus {
            overall_health: overall_health.to_string(),
            total_primals: self.primal_adapters.len(),
            healthy_primals: primal_statuses.values()
                .filter(|s| s.health == "healthy")
                .count(),
            primal_statuses,
            last_updated: Utc::now(),
        })
    }
    
    /// Deploy a biome across the ecosystem
    pub async fn deploy_biome(&self, manifest: BiomeManifest) -> Result<BiomeDeployment> {
        info!("Deploying biome: {}", manifest.metadata.name);
        
        let deployment_id = Uuid::new_v4().to_string();
        let mut deployment_results = HashMap::new();
        
        // Coordinate deployment across relevant Primals
        for (primal_name, primal_config) in &manifest.primals {
            if !primal_config.enabled {
                continue;
            }
            
            if let Some(adapter) = self.primal_adapters.get(primal_name) {
                info!("Coordinating deployment with {}", primal_name);
                
                let coordination_request = DeploymentCoordinationRequest {
                    deployment_id: deployment_id.clone(),
                    manifest: manifest.clone(),
                    primal_specific_config: primal_config.clone(),
                    timestamp: Utc::now(),
                };
                
                match adapter.coordinate_deployment(coordination_request).await {
                    Ok(result) => {
                        info!("✅ {} deployment coordination successful", primal_name);
                        deployment_results.insert(primal_name.clone(), result);
                    }
                    Err(e) => {
                        warn!("⚠️  {} deployment coordination failed: {}", primal_name, e);
                        deployment_results.insert(primal_name.clone(), DeploymentResult {
                            primal_name: primal_name.clone(),
                            status: "failed".to_string(),
                            message: Some(e.to_string()),
                            endpoints: HashMap::new(),
                            timestamp: Utc::now(),
                        });
                    }
                }
            } else {
                warn!("Primal {} not connected - skipping deployment coordination", primal_name);
            }
        }
        
        Ok(BiomeDeployment {
            deployment_id,
            manifest,
            status: determine_deployment_status(&deployment_results),
            results: deployment_results,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        })
    }
    
    /// Get real-time events from all Primals
    pub async fn get_real_time_events(&self) -> Result<Vec<EcosystemEvent>> {
        let mut all_events = Vec::new();
        
        for (name, adapter) in &self.primal_adapters {
            match adapter.get_recent_events().await {
                Ok(events) => {
                    for event in events {
                        all_events.push(EcosystemEvent {
                            source_primal: name.clone(),
                            event_type: event.event_type,
                            data: event.data,
                            timestamp: event.timestamp,
                        });
                    }
                }
                Err(e) => {
                    warn!("Failed to get events from {}: {}", name, e);
                }
            }
        }
        
        // Sort events by timestamp (most recent first)
        all_events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(all_events)
    }
    
    /// Execute an AI-assisted command across the ecosystem
    pub async fn execute_ai_command(&self, command: AICommand) -> Result<AICommandResult> {
        info!("Executing AI command: {}", command.command);
        
        // Determine which Primals need to be involved
        let relevant_primals = self.determine_relevant_primals(&command).await?;
        
        let mut results = HashMap::new();
        
        for primal_name in relevant_primals {
            if let Some(adapter) = self.primal_adapters.get(&primal_name) {
                match adapter.execute_ai_command(&command).await {
                    Ok(result) => {
                        results.insert(primal_name.clone(), result);
                    }
                    Err(e) => {
                        warn!("AI command failed on {}: {}", primal_name, e);
                    }
                }
            }
        }
        
        Ok(AICommandResult {
            command: command.command.clone(),
            results,
            timestamp: Utc::now(),
        })
    }
    
    /// Validate all configured endpoints
    async fn validate_endpoints(&mut self) {
        let mut valid_endpoints = HashMap::new();
        
        for (name, endpoint) in &self.primal_endpoints {
            if self.test_endpoint_connectivity(endpoint).await {
                valid_endpoints.insert(name.clone(), endpoint.clone());
            } else {
                warn!("Endpoint {} at {} is not reachable", name, endpoint);
            }
        }
        
        self.primal_endpoints = valid_endpoints;
    }
    
    /// Probe a Primal endpoint to get its information
    async fn probe_primal_endpoint(&self, name: &str, endpoint: &str) -> Result<PrimalInfo> {
        let health_url = format!("{}/api/v1/health", endpoint);
        
        let response = self.client
            .get(&health_url)
            .timeout(self.timeout)
            .send()
            .await?;
            
        if !response.status().is_success() {
            return Err(anyhow!("Health check failed with status: {}", response.status()));
        }
        
        let health_data: serde_json::Value = response.json().await?;
        
        Ok(PrimalInfo {
            name: name.to_string(),
            endpoint: endpoint.to_string(),
            capabilities: health_data.get("capabilities")
                .and_then(|c| c.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect())
                .unwrap_or_default(),
            api_version: health_data.get("api_version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            health: health_data.get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
        })
    }
    
    /// Test endpoint connectivity
    async fn test_endpoint_connectivity(&self, endpoint: &str) -> bool {
        let test_url = format!("{}/api/v1/health", endpoint);
        
        match self.client
            .get(&test_url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
    
    /// Determine which Primals are relevant for an AI command
    async fn determine_relevant_primals(&self, command: &AICommand) -> Result<Vec<String>> {
        let mut relevant = Vec::new();
        
        // Analyze command to determine which Primals should handle it
        let command_lower = command.command.to_lowercase();
        
        for (name, adapter) in &self.primal_adapters {
            let capabilities = &adapter.primal_info.capabilities;
            
            let is_relevant = if command_lower.contains("deploy") || command_lower.contains("orchestrate") {
                capabilities.contains(&"orchestration".to_string())
            } else if command_lower.contains("storage") || command_lower.contains("data") {
                capabilities.contains(&"storage".to_string())
            } else if command_lower.contains("compute") || command_lower.contains("run") {
                capabilities.contains(&"compute".to_string())
            } else if command_lower.contains("security") || command_lower.contains("auth") {
                capabilities.contains(&"security".to_string())
            } else if command_lower.contains("ai") || command_lower.contains("ml") {
                capabilities.contains(&"ai".to_string())
            } else {
                // For general commands, include orchestration Primals
                capabilities.contains(&"orchestration".to_string())
            };
            
            if is_relevant {
                relevant.push(name.clone());
            }
        }
        
        // If no specific Primals found, include all connected ones
        if relevant.is_empty() {
            relevant = self.primal_adapters.keys().cloned().collect();
        }
        
        Ok(relevant)
    }
}

/// Individual Primal adapter for API communication
#[derive(Debug, Clone)]
pub struct PrimalAdapter {
    /// Information about this Primal
    pub primal_info: PrimalInfo,
    
    /// HTTP client
    client: Client,
}

impl PrimalAdapter {
    /// Create a new Primal adapter
    pub async fn new(primal_info: PrimalInfo, client: Client) -> Result<Self> {
        Ok(Self {
            primal_info,
            client,
        })
    }
    
    /// Perform health check on this Primal
    pub async fn health_check(&self) -> Result<()> {
        let url = format!("{}/api/v1/health", self.primal_info.endpoint);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
            
        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow!("Health check failed: {}", response.status()))
        }
    }
    
    /// Get status from this Primal
    pub async fn get_status(&self) -> Result<PrimalStatus> {
        let url = format!("{}/api/v1/status", self.primal_info.endpoint);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
            
        if response.status().is_success() {
            let status_data: serde_json::Value = response.json().await?;
            
            Ok(PrimalStatus {
                name: self.primal_info.name.clone(),
                health: status_data.get("health")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                service_count: status_data.get("service_count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize,
                last_seen: Utc::now(),
                capabilities: self.primal_info.capabilities.clone(),
                version: status_data.get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
            })
        } else {
            Err(anyhow!("Status request failed: {}", response.status()))
        }
    }
    
    /// Coordinate deployment with this Primal
    pub async fn coordinate_deployment(&self, request: DeploymentCoordinationRequest) -> Result<DeploymentResult> {
        let url = format!("{}/api/v1/coordinate", self.primal_info.endpoint);
        
        let coordination_payload = serde_json::json!({
            "coordination_request": {
                "from": "biomeos-ui",
                "to": self.primal_info.name,
                "api_version": "universal/v1",
                "request_type": "BiomeDeployment",
                "request_data": {
                    "deployment_id": request.deployment_id,
                    "manifest": request.manifest,
                    "primal_config": request.primal_specific_config
                },
                "timestamp": request.timestamp
            }
        });
        
        let response = self.client
            .post(&url)
            .json(&coordination_payload)
            .send()
            .await?;
            
        if response.status().is_success() {
            let response_data: serde_json::Value = response.json().await?;
            
            Ok(DeploymentResult {
                primal_name: self.primal_info.name.clone(),
                status: "success".to_string(),
                message: Some("Deployment coordination successful".to_string()),
                endpoints: extract_endpoints_from_response(&response_data),
                timestamp: Utc::now(),
            })
        } else {
            Err(anyhow!("Deployment coordination failed: {}", response.status()))
        }
    }
    
    /// Get recent events from this Primal
    pub async fn get_recent_events(&self) -> Result<Vec<PrimalEvent>> {
        let url = format!("{}/api/v1/events", self.primal_info.endpoint);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;
            
        if response.status().is_success() {
            let events_data: serde_json::Value = response.json().await?;
            
            if let Some(events_array) = events_data.get("events").and_then(|e| e.as_array()) {
                let mut events = Vec::new();
                
                for event_data in events_array {
                    if let Ok(event) = serde_json::from_value::<PrimalEvent>(event_data.clone()) {
                        events.push(event);
                    }
                }
                
                Ok(events)
            } else {
                Ok(vec![])
            }
        } else {
            Err(anyhow!("Events request failed: {}", response.status()))
        }
    }
    
    /// Execute AI command on this Primal
    pub async fn execute_ai_command(&self, command: &AICommand) -> Result<serde_json::Value> {
        let url = format!("{}/api/v1/ai-command", self.primal_info.endpoint);
        
        let response = self.client
            .post(&url)
            .json(command)
            .send()
            .await?;
            
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow!("AI command failed: {}", response.status()))
        }
    }
}

// Helper functions

fn determine_deployment_status(results: &HashMap<String, DeploymentResult>) -> String {
    let total = results.len();
    let successful = results.values()
        .filter(|r| r.status == "success")
        .count();
    
    if successful == total && total > 0 {
        "success".to_string()
    } else if successful > 0 {
        "partial".to_string()
    } else {
        "failed".to_string()
    }
}

fn extract_endpoints_from_response(response: &serde_json::Value) -> HashMap<String, String> {
    let mut endpoints = HashMap::new();
    
    if let Some(response_data) = response.get("response_data") {
        if let Some(endpoints_data) = response_data.get("endpoints") {
            if let Some(endpoints_obj) = endpoints_data.as_object() {
                for (key, value) in endpoints_obj {
                    if let Some(endpoint_str) = value.as_str() {
                        endpoints.insert(key.clone(), endpoint_str.to_string());
                    }
                }
            }
        }
    }
    
    endpoints
}

// Data structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    pub name: String,
    pub endpoint: String,
    pub capabilities: Vec<String>,
    pub api_version: String,
    pub health: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalStatus {
    pub name: String,
    pub health: String,
    pub service_count: usize,
    pub last_seen: DateTime<Utc>,
    pub capabilities: Vec<String>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemStatus {
    pub overall_health: String,
    pub total_primals: usize,
    pub healthy_primals: usize,
    pub primal_statuses: HashMap<String, PrimalStatus>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeManifest {
    pub metadata: BiomeMetadata,
    pub primals: HashMap<String, PrimalConfig>,
    pub services: HashMap<String, ServiceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalConfig {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub capabilities: Vec<String>,
    pub config: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub primal: String,
    pub runtime: String,
    pub image: Option<String>,
    pub resources: ResourceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub cpu: f64,
    pub memory: String,
    pub storage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentCoordinationRequest {
    pub deployment_id: String,
    pub manifest: BiomeManifest,
    pub primal_specific_config: PrimalConfig,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    pub primal_name: String,
    pub status: String,
    pub message: Option<String>,
    pub endpoints: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeDeployment {
    pub deployment_id: String,
    pub manifest: BiomeManifest,
    pub status: String,
    pub results: HashMap<String, DeploymentResult>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AICommand {
    pub command: String,
    pub context: Option<String>,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AICommandResult {
    pub command: String,
    pub results: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEvent {
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemEvent {
    pub source_primal: String,
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

// Re-export for convenience
pub use self::{UniversalApiClient as BiomeOSApiClient}; 