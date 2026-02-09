# BiomeOS Discovery & Health Monitoring API Documentation

## 🎯 **Universal, Capability-Based Discovery**

The BiomeOS Discovery System enables **universal service discovery** that is completely **agnostic** to specific service names or ecosystem architectures. Services are discovered by **capabilities**, ensuring your applications work with any BiomeOS-compatible ecosystem.

### **🌟 Core Principles**
- **🚫 Service Name Agnostic**: No hardcoded assumptions about "Songbird" or specific services
- **🎯 Capability-Driven**: Find services by what they can do, not what they're called
- **🌐 Architecture Flexible**: Works with monolithic, microservice, or hybrid ecosystems
- **🔧 Future-Proof**: Adapts to ecosystem evolution automatically

---

## 📋 **Core Types Overview**

| Type | Purpose | Usage |
|------|---------|-------|
| [`UniversalBiomeOSManager`](#universalbiomeosmanager) | Main discovery & health interface | Primary API for all operations |
| [`DiscoveryResult`](#discoveryresult) | Service discovery result | Describes discovered services |
| [`ProbeResult`](#proberesult) | Endpoint probe result | Details about specific endpoints |
| [`SystemHealth`](#systemhealth) | System health status | Overall system health information |
| [`HealthStatus`](#healthstatus) | Health enumeration | Service and system health levels |

---

## 🚀 **UniversalBiomeOSManager**

The main interface for capability-based discovery and health monitoring.

### **Core Discovery Methods**

#### **Universal Service Discovery**
```rust
impl UniversalBiomeOSManager {
    /// Discover services by capabilities (universal approach)
    pub async fn discover_by_capability(
        &self, 
        endpoint: &str, 
        required_capabilities: &[PrimalCapability]
    ) -> Result<Vec<DiscoveryResult>>
    
    /// Discover orchestration services (replaces hardcoded Songbird calls)
    pub async fn discover_orchestration_services(
        &self, 
        orchestration_endpoint: &str
    ) -> Result<Vec<DiscoveryResult>>
    
    /// Discover all service discovery providers
    pub async fn discover_service_discovery_providers(
        &self, 
        endpoints: &[String]
    ) -> Result<Vec<DiscoveryResult>>
    
    /// Registry-based discovery
    pub async fn discover_registry(
        &self, 
        registry_url: &str
    ) -> Result<Vec<DiscoveryResult>>
    
    /// Network scan discovery
    pub async fn discover_network_scan(&self) -> Result<Vec<DiscoveryResult>>
    
    /// Probe specific endpoint for capabilities
    pub async fn probe_endpoint(&self, endpoint: &str) -> Result<ProbeResult>
}
```

### **Usage Examples**

#### **Capability-Based Discovery**
```rust
use biomeos_core::{UniversalBiomeOSManager, BiomeOSConfig};
use biomeos_primal_sdk::PrimalCapability;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = BiomeOSConfig::default();
    let manager = UniversalBiomeOSManager::new(config);
    
    // Find services by specific capabilities
    let routing_services = manager.discover_by_capability(
        "http://discovery-service:8080",
        &[
            PrimalCapability::message_routing(),
            PrimalCapability::load_balancing(),
        ]
    ).await?;
    
    println!("Found {} routing services:", routing_services.len());
    for service in routing_services {
        println!("  📦 {}: {} ({})", 
            service.primal_id, 
            service.endpoint,
            service.primal_type.category
        );
    }
    
    Ok(())
}
```

#### **Universal Orchestration Discovery**
```rust
// This works whether you have:
// - One monolithic Songbird providing all orchestration
// - Multiple specialized services (routing, discovery, load-balancing)
// - Community-built alternatives with enhanced capabilities

let orchestration_services = manager
    .discover_orchestration_services("http://ecosystem-endpoint:8080")
    .await?;

// Automatically finds services providing:
// - service_discovery capability
// - message_routing capability  
// - load_balancing capability

for service in orchestration_services {
    println!("🎼 Orchestration service: {} with {} capabilities", 
        service.primal_id, 
        service.capabilities.len()
    );
}
```

#### **Multi-Provider Discovery**
```rust
// Find all service discovery providers across different endpoints
let discovery_providers = manager.discover_service_discovery_providers(&[
    "http://team-a-services:8080".to_string(),
    "http://team-b-registry:9000".to_string(),
    "http://community-discovery:7000".to_string(),
]).await?;

println!("🌐 Found {} discovery providers across ecosystem", discovery_providers.len());
```

#### **Network-Wide Discovery**
```rust
// Scan network for any BiomeOS-compatible services
let all_services = manager.discover_network_scan().await?;

// Group by category
use std::collections::HashMap;
let mut by_category: HashMap<String, Vec<_>> = HashMap::new();
for service in all_services {
    by_category
        .entry(service.primal_type.category.clone())
        .or_default()
        .push(service);
}

for (category, services) in by_category {
    println!("📂 {}: {} services", category, services.len());
}
```

#### **Service Probing**
```rust
// Probe specific endpoint for detailed information
let probe_result = manager.probe_endpoint("http://unknown-service:8080").await?;

println!("🔍 Service Details:");
println!("  Name: {}", probe_result.name);
println!("  Category: {}", probe_result.category);
println!("  Capabilities: {}", probe_result.capabilities.len());

for cap in &probe_result.capabilities {
    println!("    ⚡ {}/{} ({})", cap.domain, cap.name, cap.version);
}
```

---

## 📦 **DiscoveryResult**

Comprehensive result from service discovery operations.

### **Definition**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResult {
    pub primal_id: String,
    pub primal_type: PrimalType,
    pub endpoint: String,
    pub capabilities: Vec<PrimalCapability>,
    pub health: PrimalHealth,
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}
```

### **Usage Examples**
```rust
// Filter by capability
let gpu_services: Vec<_> = discovery_results.into_iter()
    .filter(|result| {
        result.capabilities.iter().any(|cap| 
            cap.name.contains("gpu") || cap.name.contains("cuda")
        )
    })
    .collect();

// Filter by health status  
let healthy_services: Vec<_> = discovery_results.into_iter()
    .filter(|result| result.health.is_healthy())
    .collect();

// Sort by discovery time (newest first)
let mut recent_services = discovery_results;
recent_services.sort_by(|a, b| b.discovered_at.cmp(&a.discovered_at));
```

---

## 🔍 **ProbeResult**

Detailed information from endpoint probing.

### **Definition**
```rust
#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub name: String,
    pub category: String,
    pub capabilities: Vec<biomeos_primal_sdk::PrimalCapability>,
}
```

### **Usage Examples**
```rust
// Capability checking
async fn check_service_compatibility(endpoint: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let probe = manager.probe_endpoint(endpoint).await?;
    
    let has_required_caps = probe.capabilities.iter().any(|cap| {
        cap.matches_domain("compute") && cap.matches_name("gpu_acceleration")
    });
    
    Ok(has_required_caps && probe.category == "ai")
}

// Service catalog building
async fn build_service_catalog() -> Result<ServiceCatalog, Box<dyn std::error::Error>> {
    let endpoints = discover_all_endpoints().await?;
    let mut catalog = ServiceCatalog::new();
    
    for endpoint in endpoints {
        if let Ok(probe) = manager.probe_endpoint(&endpoint).await {
            catalog.add_service(ServiceInfo {
                name: probe.name,
                category: probe.category,
                endpoint,
                capabilities: probe.capabilities,
                last_probed: chrono::Utc::now(),
            });
        }
    }
    
    Ok(catalog)
}
```

---

## 🏥 **Health Monitoring**

Comprehensive system and service health monitoring.

### **Core Health Methods**
```rust
impl UniversalBiomeOSManager {
    /// Get comprehensive system health
    pub async fn get_system_health(&self) -> SystemHealth
    
    /// Start health monitoring service
    pub async fn start_monitoring(&self) -> Result<()>
    
    /// Get uptime in seconds
    pub fn calculate_system_uptime(&self) -> Result<u64>
}
```

### **SystemHealth Structure**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub overall_status: HealthStatus,
    pub primal_health: HashMap<String, PrimalHealth>,
    pub resource_usage: SystemResourceUsage,
    pub uptime: chrono::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResourceUsage {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub disk_usage_percent: f64,
    pub network_usage_mbps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Warning,
    Critical,
    Unhealthy,
    Unknown,
}
```

### **Health Monitoring Examples**

#### **Basic Health Check**
```rust
let system_health = manager.get_system_health().await;

println!("🏥 System Health Report");
println!("  Overall Status: {:?}", system_health.overall_status);
println!("  Uptime: {} seconds", system_health.uptime.num_seconds());
println!("  CPU Usage: {:.1}%", system_health.resource_usage.cpu_usage_percent);
println!("  Memory Usage: {:.1}%", system_health.resource_usage.memory_usage_percent);
println!("  Disk Usage: {:.1}%", system_health.resource_usage.disk_usage_percent);

// Check individual primals
for (primal_id, health) in &system_health.primal_health {
    println!("  📦 {}: {:?}", primal_id, health);
}
```

#### **Health-Based Service Selection**
```rust
async fn find_healthy_services(
    manager: &UniversalBiomeOSManager,
    required_caps: &[PrimalCapability]
) -> Result<Vec<DiscoveryResult>, Box<dyn std::error::Error>> {
    // Discover services by capability
    let all_services = manager.discover_by_capability(
        "http://discovery-endpoint:8080", 
        required_caps
    ).await?;
    
    // Filter to only healthy services
    let healthy_services: Vec<_> = all_services.into_iter()
        .filter(|service| service.health.is_healthy())
        .collect();
    
    // Sort by capability count (more capable services first)
    let mut sorted_services = healthy_services;
    sorted_services.sort_by(|a, b| b.capabilities.len().cmp(&a.capabilities.len()));
    
    Ok(sorted_services)
}
```

#### **Continuous Health Monitoring**
```rust
use tokio::time::{interval, Duration};

async fn continuous_health_monitoring(
    manager: UniversalBiomeOSManager
) -> Result<(), Box<dyn std::error::Error>> {
    let mut interval = interval(Duration::from_secs(30));
    
    loop {
        interval.tick().await;
        
        let health = manager.get_system_health().await;
        
        // Log health status
        println!("[{}] System: {:?} | CPU: {:.1}% | Memory: {:.1}% | Disk: {:.1}%",
            chrono::Utc::now().format("%H:%M:%S"),
            health.overall_status,
            health.resource_usage.cpu_usage_percent,
            health.resource_usage.memory_usage_percent,
            health.resource_usage.disk_usage_percent,
        );
        
        // Alert on critical status
        if health.overall_status == HealthStatus::Critical {
            send_alert("System health is critical!").await?;
        }
        
        // Check individual primal health
        for (primal_id, primal_health) in &health.primal_health {
            if !primal_health.is_healthy() {
                println!("⚠️  Primal {} is unhealthy: {:?}", primal_id, primal_health);
            }
        }
    }
}
```

---

## 🌐 **Advanced Discovery Patterns**

### **Pattern 1: Capability Composition**
```rust
async fn find_full_stack_services() -> Result<Vec<DiscoveryResult>, Box<dyn std::error::Error>> {
    // Find services that provide multiple complementary capabilities
    let services = manager.discover_by_capability(
        "http://discovery:8080",
        &[
            PrimalCapability::code_execution(),
            PrimalCapability::service_discovery(),
            PrimalCapability::authentication(),
        ]
    ).await?;
    
    // Filter to services that have ALL required capabilities
    let full_stack: Vec<_> = services.into_iter()
        .filter(|service| {
            let has_compute = service.capabilities.iter().any(|c| c.matches_name("code_execution"));
            let has_discovery = service.capabilities.iter().any(|c| c.matches_name("service_discovery"));  
            let has_auth = service.capabilities.iter().any(|c| c.matches_name("authentication"));
            
            has_compute && has_discovery && has_auth
        })
        .collect();
    
    Ok(full_stack)
}
```

### **Pattern 2: Ecosystem Architecture Detection**
```rust
#[derive(Debug)]
enum EcosystemArchitecture {
    Monolithic { primary_service: String },
    Microservices { service_count: usize },
    Hybrid { core_services: Vec<String>, specialized_services: Vec<String> },
}

async fn detect_ecosystem_architecture() -> Result<EcosystemArchitecture, Box<dyn std::error::Error>> {
    // Scan for all services
    let all_services = manager.discover_network_scan().await?;
    
    // Analyze capability distribution
    let mut orchestration_services = Vec::new();
    let mut specialized_services = Vec::new();
    
    for service in all_services {
        let capability_count = service.capabilities.len();
        let has_orchestration = service.capabilities.iter().any(|c| 
            c.matches_domain("orchestration") || c.matches_domain("networking")
        );
        
        if has_orchestration && capability_count > 5 {
            orchestration_services.push(service.primal_id);
        } else if capability_count <= 3 {
            specialized_services.push(service.primal_id);
        }
    }
    
    let architecture = if orchestration_services.len() == 1 && specialized_services.len() <= 2 {
        EcosystemArchitecture::Monolithic {
            primary_service: orchestration_services.into_iter().next().unwrap(),
        }
    } else if specialized_services.len() > orchestration_services.len() * 2 {
        EcosystemArchitecture::Microservices {
            service_count: specialized_services.len(),
        }
    } else {
        EcosystemArchitecture::Hybrid {
            core_services: orchestration_services,
            specialized_services,
        }
    };
    
    Ok(architecture)
}
```

### **Pattern 3: Capability-Based Load Balancing**
```rust
use std::collections::HashMap;

async fn capability_aware_load_balancer(
    request: ServiceRequest
) -> Result<DiscoveryResult, Box<dyn std::error::Error>> {
    // Find all services that can handle this request
    let capable_services = manager.discover_by_capability(
        &request.discovery_endpoint,
        &request.required_capabilities
    ).await?;
    
    // Filter by health and availability
    let available_services: Vec<_> = capable_services.into_iter()
        .filter(|s| s.health.is_healthy())
        .collect();
    
    if available_services.is_empty() {
        return Err("No healthy services available for request".into());
    }
    
    // Load balance based on capability match quality
    let best_service = available_services.into_iter()
        .max_by_key(|service| {
            // Score based on capability overlap
            let overlap = service.capabilities.iter()
                .filter(|cap| request.required_capabilities.iter()
                    .any(|req| req.satisfies(&cap.domain, &cap.name)))
                .count();
            
            // Prefer services with exact capability matches
            overlap * 100 + service.capabilities.len()
        })
        .unwrap();
    
    Ok(best_service)
}

struct ServiceRequest {
    discovery_endpoint: String,
    required_capabilities: Vec<PrimalCapability>,
    payload: serde_json::Value,
}
```

---

## 🔧 **Integration Examples**

### **Example 1: Universal Service Client**
```rust
/// Universal client that works with any BiomeOS ecosystem
pub struct UniversalServiceClient {
    manager: UniversalBiomeOSManager,
    discovered_services: HashMap<String, DiscoveryResult>,
}

impl UniversalServiceClient {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = BiomeOSConfig::default();
        let manager = UniversalBiomeOSManager::new(config);
        
        Ok(Self {
            manager,
            discovered_services: HashMap::new(),
        })
    }
    
    /// Find and call any service by capability
    pub async fn call_by_capability(
        &mut self,
        required_caps: Vec<PrimalCapability>,
        method: &str,
        payload: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        // Try multiple discovery strategies
        let mut services = Vec::new();
        
        // Strategy 1: Network scan
        if let Ok(network_services) = self.manager.discover_network_scan().await {
            services.extend(network_services);
        }
        
        // Strategy 2: Try common discovery endpoints
        for endpoint in ["http://discovery:8080", "http://registry:9000", "http://songbird:8080"] {
            if let Ok(endpoint_services) = self.manager.discover_by_capability(endpoint, &required_caps).await {
                services.extend(endpoint_services);
            }
        }
        
        // Find best service for this request
        let service = services.into_iter()
            .filter(|s| s.health.is_healthy())
            .max_by_key(|s| s.capabilities.len())
            .ok_or("No suitable service found")?;
        
        // Make the call
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/api/v1/{}", service.endpoint, method))
            .json(&payload)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        
        Ok(response)
    }
}

// Usage
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = UniversalServiceClient::new().await?;
    
    // Call any service that can execute code
    let result = client.call_by_capability(
        vec![PrimalCapability::code_execution()],
        "execute",
        serde_json::json!({
            "code": "print('Hello from universal client!')",
            "language": "python"
        })
    ).await?;
    
    println!("Execution result: {}", result);
    Ok(())
}
```

### **Example 2: Ecosystem Health Dashboard**
```rust
use std::collections::HashMap;
use tokio::time::{interval, Duration};

pub struct EcosystemDashboard {
    manager: UniversalBiomeOSManager,
    service_history: HashMap<String, Vec<HealthSnapshot>>,
}

#[derive(Debug, Clone)]
struct HealthSnapshot {
    timestamp: chrono::DateTime<chrono::Utc>,
    health: PrimalHealth,
    response_time_ms: Option<u64>,
}

impl EcosystemDashboard {
    pub async fn new() -> Self {
        let config = BiomeOSConfig::default();
        let manager = UniversalBiomeOSManager::new(config);
        
        Self {
            manager,
            service_history: HashMap::new(),
        }
    }
    
    pub async fn run_dashboard(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut interval = interval(Duration::from_secs(30));
        
        loop {
            interval.tick().await;
            
            // Get overall system health
            let system_health = self.manager.get_system_health().await;
            
            // Discover all services
            let services = self.manager.discover_network_scan().await.unwrap_or_default();
            
            // Update service history
            for service in &services {
                let snapshot = HealthSnapshot {
                    timestamp: chrono::Utc::now(),
                    health: service.health.clone(),
                    response_time_ms: None, // Could add response time measurement
                };
                
                self.service_history
                    .entry(service.primal_id.clone())
                    .or_insert_with(Vec::new)
                    .push(snapshot);
            }
            
            // Print dashboard
            self.print_dashboard(&system_health, &services).await;
        }
    }
    
    async fn print_dashboard(&self, system_health: &SystemHealth, services: &[DiscoveryResult]) {
        println!("\n🌱 BiomeOS Ecosystem Dashboard - {}", chrono::Utc::now().format("%H:%M:%S"));
        println!("═".repeat(60));
        
        // System overview
        println!("🖥️  System: {:?} | Uptime: {}s", 
            system_health.overall_status,
            system_health.uptime.num_seconds()
        );
        println!("📊 Resources: CPU {:.1}% | Memory {:.1}% | Disk {:.1}%",
            system_health.resource_usage.cpu_usage_percent,
            system_health.resource_usage.memory_usage_percent,
            system_health.resource_usage.disk_usage_percent,
        );
        
        // Services by category
        let mut by_category: HashMap<String, Vec<_>> = HashMap::new();
        for service in services {
            by_category
                .entry(service.primal_type.category.clone())
                .or_default()
                .push(service);
        }
        
        println!("\n📦 Services by Category:");
        for (category, category_services) in by_category {
            println!("  {} ({}):", category, category_services.len());
            for service in category_services {
                let health_icon = match service.health {
                    PrimalHealth::Healthy => "✅",
                    PrimalHealth::Degraded => "⚠️",
                    PrimalHealth::Unhealthy => "❌", 
                    PrimalHealth::Unknown => "❓",
                };
                println!("    {} {} ({})", health_icon, service.primal_id, service.endpoint);
            }
        }
        
        // Capability summary
        let mut all_capabilities = HashMap::new();
        for service in services {
            for cap in &service.capabilities {
                *all_capabilities.entry(format!("{}/{}", cap.domain, cap.name)).or_insert(0) += 1;
            }
        }
        
        println!("\n⚡ Available Capabilities:");
        let mut sorted_caps: Vec<_> = all_capabilities.iter().collect();
        sorted_caps.sort_by(|a, b| b.1.cmp(a.1));
        for (cap_name, count) in sorted_caps.iter().take(10) {
            println!("  {} ({}x)", cap_name, count);
        }
    }
}
```

---

## 🌟 **Why This API Enables True Universality**

The BiomeOS Discovery & Health Monitoring API achieves **true universal compatibility** through:

### **🚫 Zero Hardcoded Dependencies**
- Services discovered by capability, never by name
- Works with any service providing required capabilities
- No assumptions about ecosystem structure

### **🎯 Capability-Driven Architecture** 
- `discover_by_capability()` is the core discovery method
- Flexible capability matching with aliases and equivalences
- Dynamic service composition based on available capabilities

### **🌐 Ecosystem Agnostic Design**
- Same API works with monolithic, microservice, or hybrid architectures
- Automatic adaptation to different discovery mechanisms
- Graceful degradation when services are unavailable

### **🔧 Future-Proof Interfaces**
- New capabilities discovered automatically
- Service splitting/merging transparent to clients
- Community extensions work seamlessly

### **📊 Comprehensive Observability**
- Real-time health monitoring for system and services
- Performance metrics and resource usage tracking
- Historical health data for trend analysis

**Your applications built with this API will work with ANY BiomeOS ecosystem - today, tomorrow, and as the ecosystem evolves!** 🎉 