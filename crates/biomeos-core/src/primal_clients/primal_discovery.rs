//! Universal Primal Discovery Service
//!
//! This service discovers primals on the network based on their capabilities,
//! not their names. It can find current primals (Toadstool, Songbird, etc.)
//! and future primals that implement the universal primal protocol.

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

use crate::{BiomeResult, BiomeError};
use super::{PrimalDiscovery, DiscoveredPrimal, UniversalPrimalClient};

/// Network-based primal discovery service
pub struct NetworkPrimalDiscovery {
    /// HTTP client for API calls
    client: Client,
    /// Common ports to scan for primals
    scan_ports: Vec<u16>,
    /// Common API paths to check
    api_paths: Vec<String>,
    /// Discovery timeout
    timeout: Duration,
}

impl NetworkPrimalDiscovery {
    /// Create a new network primal discovery service
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            // Common ports used by primals
            scan_ports: vec![
                8080, 8081, 8082, 8083, 8084, // HTTP ports
                3000, 3001, 3002, 3003, 3004, // Alternative HTTP ports
                9090, 9091, 9092, 9093, 9094, // Monitoring ports
                6379, 6380, // Redis-style ports
                5432, 5433, // PostgreSQL-style ports
            ],
            // Common API paths primals might use
            api_paths: vec![
                "/api/v1/capabilities".to_string(),
                "/capabilities".to_string(),
                "/primal/info".to_string(),
                "/info".to_string(),
                "/health".to_string(),
                "/api/health".to_string(),
                "/status".to_string(),
            ],
            timeout: Duration::from_secs(5),
        }
    }
    
    /// Scan local network for primals
    async fn scan_local_network(&self) -> BiomeResult<Vec<DiscoveredPrimal>> {
        let mut discovered = Vec::new();
        
        // Get local IP range (simplified - in real implementation would be more sophisticated)
        let local_ips = self.get_local_ip_range().await?;
        
        for ip in local_ips {
            for port in &self.scan_ports {
                let addr = format!("{}:{}", ip, port);
                
                // Try to connect to this address
                if let Ok(_) = timeout(Duration::from_millis(100), TcpStream::connect(&addr)).await {
                    // Port is open, try to discover if it's a primal
                    if let Ok(primal) = self.probe_primal(&addr).await {
                        discovered.push(primal);
                    }
                }
            }
        }
        
        Ok(discovered)
    }
    
    /// Probe a specific address to see if it's a primal
    async fn probe_primal(&self, addr: &str) -> BiomeResult<DiscoveredPrimal> {
        // Try each API path to see if we can get primal information
        for path in &self.api_paths {
            let url = format!("http://{}{}", addr, path);
            
            if let Ok(response) = timeout(self.timeout, self.client.get(&url).send()).await {
                if let Ok(response) = response {
                    if response.status().is_success() {
                        if let Ok(info) = response.json::<PrimalInfo>().await {
                            return Ok(DiscoveredPrimal {
                                id: info.id.unwrap_or_else(|| format!("primal-{}", addr)),
                                primal_type: info.primal_type.unwrap_or_else(|| "unknown".to_string()),
                                endpoint: format!("http://{}", addr),
                                metadata: info.metadata.unwrap_or_default(),
                            });
                        }
                    }
                }
            }
        }
        
        Err(BiomeError::NotFound(format!("No primal found at {}", addr)))
    }
    
    /// Get local IP range to scan
    async fn get_local_ip_range(&self) -> BiomeResult<Vec<String>> {
        // Simplified implementation - in real use would be more sophisticated
        let mut ips = Vec::new();
        
        // Add localhost
        ips.push("127.0.0.1".to_string());
        
        // Add common local network ranges
        for i in 1..255 {
            ips.push(format!("192.168.1.{}", i));
            ips.push(format!("10.0.0.{}", i));
        }
        
        Ok(ips)
    }
    
    /// Discover primals via environment variables
    async fn discover_from_environment(&self) -> BiomeResult<Vec<DiscoveredPrimal>> {
        let mut discovered = Vec::new();
        
        // Check for primal endpoints in environment
        let env_primals = [
            ("TOADSTOOL_ENDPOINT", "toadstool"),
            ("SONGBIRD_ENDPOINT", "songbird"),
            ("NESTGATE_ENDPOINT", "nestgate"),
            ("BEARDOG_ENDPOINT", "beardog"),
            ("SQUIRREL_ENDPOINT", "squirrel"),
        ];
        
        for (env_var, primal_type) in env_primals {
            if let Ok(endpoint) = std::env::var(env_var) {
                // Try to probe this endpoint
                if let Ok(mut primal) = self.probe_endpoint(&endpoint).await {
                    primal.primal_type = primal_type.to_string();
                    discovered.push(primal);
                }
            }
        }
        
        // Check for generic primal endpoints
        let mut i = 0;
        while let Ok(endpoint) = std::env::var(&format!("PRIMAL_ENDPOINT_{}", i)) {
            if let Ok(primal) = self.probe_endpoint(&endpoint).await {
                discovered.push(primal);
            }
            i += 1;
        }
        
        Ok(discovered)
    }
    
    /// Probe a specific endpoint
    async fn probe_endpoint(&self, endpoint: &str) -> BiomeResult<DiscoveredPrimal> {
        // Try to get primal information
        for path in &self.api_paths {
            let url = format!("{}{}", endpoint, path);
            
            if let Ok(response) = timeout(self.timeout, self.client.get(&url).send()).await {
                if let Ok(response) = response {
                    if response.status().is_success() {
                        if let Ok(info) = response.json::<PrimalInfo>().await {
                            return Ok(DiscoveredPrimal {
                                id: info.id.unwrap_or_else(|| format!("primal-{}", endpoint)),
                                primal_type: info.primal_type.unwrap_or_else(|| "unknown".to_string()),
                                endpoint: endpoint.to_string(),
                                metadata: info.metadata.unwrap_or_default(),
                            });
                        }
                    }
                }
            }
        }
        
        Err(BiomeError::NotFound(format!("No primal found at {}", endpoint)))
    }
}

#[async_trait]
impl PrimalDiscovery for NetworkPrimalDiscovery {
    /// Discover primals on the network
    async fn discover_primals(&self) -> BiomeResult<Vec<DiscoveredPrimal>> {
        let mut all_discovered = Vec::new();
        
        // Discover from environment variables (fastest)
        if let Ok(env_primals) = self.discover_from_environment().await {
            all_discovered.extend(env_primals);
        }
        
        // Discover from local network scan (slower)
        if let Ok(network_primals) = self.scan_local_network().await {
            all_discovered.extend(network_primals);
        }
        
        // Remove duplicates based on endpoint
        let mut unique_primals = Vec::new();
        let mut seen_endpoints = std::collections::HashSet::new();
        
        for primal in all_discovered {
            if seen_endpoints.insert(primal.endpoint.clone()) {
                unique_primals.push(primal);
            }
        }
        
        Ok(unique_primals)
    }
    
    /// Create a client for a discovered primal
    async fn create_client(&self, primal: &DiscoveredPrimal) -> BiomeResult<Box<dyn UniversalPrimalClient>> {
        Ok(Box::new(HttpPrimalClient::new(
            primal.endpoint.clone(),
            primal.id.clone(),
            primal.primal_type.clone(),
        )))
    }
}

/// HTTP-based universal primal client
pub struct HttpPrimalClient {
    /// HTTP client
    client: Client,
    /// Primal endpoint
    endpoint: String,
    /// Primal ID
    id: String,
    /// Primal type
    primal_type: String,
}

impl HttpPrimalClient {
    /// Create a new HTTP primal client
    pub fn new(endpoint: String, id: String, primal_type: String) -> Self {
        Self {
            client: Client::new(),
            endpoint,
            id,
            primal_type,
        }
    }
    
    /// Make an API call to the primal
    async fn api_call(&self, path: &str, method: &str, body: Option<serde_json::Value>) -> BiomeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoint, path);
        
        let request = match method {
            "GET" => self.client.get(&url),
            "POST" => {
                let mut req = self.client.post(&url);
                if let Some(body) = body {
                    req = req.json(&body);
                }
                req
            },
            "PUT" => {
                let mut req = self.client.put(&url);
                if let Some(body) = body {
                    req = req.json(&body);
                }
                req
            },
            "DELETE" => self.client.delete(&url),
            _ => return Err(BiomeError::InvalidRequest(format!("Unsupported method: {}", method))),
        };
        
        let response = timeout(Duration::from_secs(30), request.send()).await
            .map_err(|_| BiomeError::Timeout("API call timed out".to_string()))?
            .map_err(|e| BiomeError::NetworkError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(BiomeError::ApiError(format!("API call failed: {}", response.status())));
        }
        
        let result = response.json::<serde_json::Value>().await
            .map_err(|e| BiomeError::Serialization(e.to_string()))?;
        
        Ok(result)
    }
}

#[async_trait]
impl UniversalPrimalClient for HttpPrimalClient {
    /// Discover what capabilities this primal provides
    async fn discover_capabilities(&self) -> BiomeResult<Vec<super::CapabilityResponse>> {
        let result = self.api_call("/api/v1/capabilities", "GET", None).await?;
        
        let capabilities: Vec<super::CapabilityResponse> = serde_json::from_value(result)
            .map_err(|e| BiomeError::Serialization(e.to_string()))?;
        
        Ok(capabilities)
    }
    
    /// Execute a capability-based request
    async fn execute_capability(
        &self,
        category: super::CapabilityCategory,
        operation: &str,
        params: serde_json::Value,
    ) -> BiomeResult<serde_json::Value> {
        let body = serde_json::json!({
            "category": category,
            "operation": operation,
            "params": params
        });
        
        let result = self.api_call("/api/v1/execute", "POST", Some(body)).await?;
        
        Ok(result)
    }
    
    /// Check if this primal can fulfill a requirement
    async fn can_fulfill(&self, requirement: &super::CapabilityRequirement) -> bool {
        // Get capabilities and check if we can fulfill the requirement
        if let Ok(capabilities) = self.discover_capabilities().await {
            for capability in capabilities {
                if capability.category == requirement.category {
                    // Check if all required operations are supported
                    let has_all_ops = requirement.operations.iter()
                        .all(|op| capability.operations.contains(op));
                    
                    if has_all_ops {
                        // Check version if specified
                        if let Some(min_version) = &requirement.min_version {
                            // Simple version comparison (in real implementation would use semver)
                            if capability.version >= *min_version {
                                return true;
                            }
                        } else {
                            return true;
                        }
                    }
                }
            }
        }
        
        false
    }
    
    /// Get primal health status
    async fn health_check(&self) -> BiomeResult<super::PrimalHealth> {
        let result = self.api_call("/health", "GET", None).await?;
        
        let health: super::PrimalHealth = serde_json::from_value(result)
            .map_err(|e| BiomeError::Serialization(e.to_string()))?;
        
        Ok(health)
    }
    
    /// Get primal endpoint information
    fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

/// Primal information response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PrimalInfo {
    pub id: Option<String>,
    pub primal_type: Option<String>,
    pub version: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

impl Default for NetworkPrimalDiscovery {
    fn default() -> Self {
        Self::new()
    }
} 