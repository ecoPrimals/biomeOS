# ADR-001: Universal Capability-Based Architecture

## Status
**Accepted** - Implemented across all BiomeOS components

## Context

Traditional distributed systems suffer from hardcoded service dependencies that create brittle architectures:

```rust
// ❌ Hardcoded approach
let response = reqwest::get("http://songbird:8080/api/route").await?;
let toadstool_result = reqwest::post("http://toadstool:8080/execute", &payload).await?;
```

This approach has several problems:
1. **Service Name Coupling**: Code breaks when service names change
2. **Architecture Rigidity**: Assumes specific services always exist in specific configurations
3. **Ecosystem Lock-in**: Services only work with predetermined ecosystem layouts
4. **Evolution Brittleness**: System breaks when services are split, merged, or replaced

### The Universal Requirement

BiomeOS needs to support multiple ecosystem architectures:
- **Monolithic**: Single services providing multiple capabilities (e.g., one Songbird doing everything)
- **Microservices**: Specialized services with focused capabilities (e.g., separate routing, discovery, and load-balancing services)
- **Hybrid**: Mix of core services and specialized extensions
- **Community-Driven**: Services provided by different teams and organizations
- **Future Architectures**: Yet-to-be-invented service patterns

## Decision

**We adopt a Universal Capability-Based Architecture where services are discovered and used based on their capabilities, not their names.**

### Core Principles

1. **Capability-Driven Discovery**: Services are found by what they can do, not what they're called
2. **Zero Hardcoded Dependencies**: No service names or endpoints hardcoded in application logic
3. **Universal Compatibility**: Same code works with any ecosystem architecture
4. **Future-Proof Design**: System adapts automatically as ecosystem evolves

### Implementation

#### Capability Definition
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimalCapability {
    pub domain: String,      // e.g., "networking", "ai", "storage"
    pub name: String,        // e.g., "routing", "inference", "persistence"
    pub version: String,     // e.g., "v1.0", "v2.1"
    pub parameters: HashMap<String, serde_json::Value>,
}
```

#### Universal Discovery
```rust
// ✅ Capability-based approach
let routing_services = manager.discover_by_capability(
    "http://discovery-endpoint:8080",
    &[PrimalCapability::message_routing()]
).await?;

let compute_services = manager.discover_by_capability(
    "http://discovery-endpoint:8080", 
    &[PrimalCapability::code_execution()]
).await?;
```

#### Service Implementation
```rust
#[async_trait::async_trait]
pub trait EcoPrimal: Send + Sync {
    /// Declare what this service can do
    fn capabilities(&self) -> &[PrimalCapability];
    
    /// Handle requests based on capability
    async fn handle_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse>;
}
```

## Rationale

### Why Capability-Based Discovery?

1. **Architecture Agnostic**: Same discovery mechanism works whether capabilities are provided by one service or many
2. **Future-Proof**: New services providing existing capabilities are discovered automatically  
3. **Flexible Composition**: Applications can compose services based on needed capabilities
4. **Ecosystem Evolution**: Services can be split, merged, or replaced without breaking clients

### Real-World Scenarios

#### Scenario 1: Songbird Evolution
```rust
// Day 1: Monolithic Songbird
let orchestration_services = discover_by_capability(&[
    PrimalCapability::service_discovery(),
    PrimalCapability::message_routing(),
    PrimalCapability::load_balancing(),
]).await?;
// Result: [songbird-monolith]

// Day 365: Split Architecture 
let orchestration_services = discover_by_capability(&[
    PrimalCapability::service_discovery(),
    PrimalCapability::message_routing(), 
    PrimalCapability::load_balancing(),
]).await?;
// Result: [discovery-service, routing-service, load-balancer-service]

// Application code: UNCHANGED! ✅
```

#### Scenario 2: Community Innovation
```rust
// Community builds enhanced AI orchestration
let ai_orchestration = discover_by_capability(&[
    PrimalCapability::custom("ai-orchestration", "ML-driven service orchestration"),
    PrimalCapability::custom("predictive-scaling", "AI-based auto-scaling"),
]).await?;

// BiomeOS applications automatically discover and can use these new capabilities
```

## Consequences

### Positive Consequences

1. **✅ True Universality**: Code works with any ecosystem architecture
2. **✅ Future-Proof**: Adapts automatically to ecosystem evolution
3. **✅ Innovation Enablement**: New service types integrate seamlessly
4. **✅ Reduced Maintenance**: No hardcoded dependencies to update
5. **✅ Community Empowerment**: Anyone can contribute compatible services
6. **✅ Graceful Degradation**: Systems continue working when services change

### Negative Consequences

1. **❌ Additional Complexity**: Discovery layer adds system complexity
2. **❌ Capability Standardization**: Requires community coordination on capability definitions
3. **❌ Performance Overhead**: Discovery adds latency compared to hardcoded endpoints
4. **❌ Debugging Challenges**: Dynamic discovery can make debugging more complex

### Mitigation Strategies

1. **Complexity**: Provide clear SDK abstractions and documentation
2. **Standardization**: Establish working groups for capability standardization
3. **Performance**: Implement capability caching and discovery optimization
4. **Debugging**: Build comprehensive monitoring and tracing tools

## Examples

### Before: Hardcoded Approach
```rust
// ❌ Brittle, architecture-specific code
pub struct EcosystemClient {
    songbird_url: String,
    toadstool_url: String,
    nestgate_url: String,
}

impl EcosystemClient {
    pub async fn route_message(&self, message: &str) -> Result<()> {
        // Assumes Songbird exists and provides routing
        let response = reqwest::post(
            &format!("{}/route", self.songbird_url),
            message
        ).await?;
        
        if !response.status().is_success() {
            return Err("Songbird routing failed".into());
        }
        
        Ok(())
    }
    
    pub async fn execute_code(&self, code: &str) -> Result<String> {
        // Assumes Toadstool exists and provides execution
        let response = reqwest::post(
            &format!("{}/execute", self.toadstool_url),
            code
        ).await?;
        
        Ok(response.text().await?)
    }
}
```

### After: Capability-Based Approach
```rust
// ✅ Universal, architecture-agnostic code
pub struct UniversalEcosystemClient {
    discovery_manager: UniversalBiomeOSManager,
    service_cache: HashMap<String, DiscoveryResult>,
}

impl UniversalEcosystemClient {
    pub async fn route_message(&self, message: &str) -> Result<()> {
        // Find any service that can route messages
        let routing_services = self.discovery_manager.discover_by_capability(
            "http://discovery:8080",
            &[PrimalCapability::message_routing()]
        ).await?;
        
        let service = routing_services.into_iter()
            .filter(|s| s.health.is_healthy())
            .next()
            .ok_or("No healthy routing services available")?;
        
        // Use the discovered service (could be Songbird, or a specialized router, or anything else)
        let request = PrimalRequest::new("route", serde_json::json!({ "message": message }));
        let response = self.call_service(&service.endpoint, request).await?;
        
        Ok(())
    }
    
    pub async fn execute_code(&self, code: &str) -> Result<String> {
        // Find any service that can execute code
        let compute_services = self.discovery_manager.discover_by_capability(
            "http://discovery:8080",
            &[PrimalCapability::code_execution()]
        ).await?;
        
        let service = compute_services.into_iter()
            .filter(|s| s.health.is_healthy())
            .max_by_key(|s| s.capabilities.len()) // Prefer more capable services
            .ok_or("No healthy compute services available")?;
        
        // Use the discovered service (could be Toadstool, or a new execution engine, or anything else)
        let request = PrimalRequest::new("execute", serde_json::json!({ "code": code }));
        let response = self.call_service(&service.endpoint, request).await?;
        
        Ok(response.payload.get("result")
            .and_then(|r| r.as_str())
            .unwrap_or("No result")
            .to_string())
    }
}
```

## Implementation Status

### ✅ Completed
- [x] PrimalCapability type system
- [x] EcoPrimal trait with capability declaration
- [x] UniversalBiomeOSManager with capability-based discovery
- [x] CLI tools with capability-based commands
- [x] Integration tests validating capability discovery
- [x] Documentation and examples

### 🚧 In Progress
- [ ] Standard capability registry
- [ ] Performance optimization (capability caching)
- [ ] Enhanced debugging tools

### 📋 Planned
- [ ] Cross-ecosystem capability bridging
- [ ] AI-powered capability matching
- [ ] Capability marketplace and ratings

## References

- [Primal Integration Guide](../guides/primal-integration-guide.md)
- [Ecosystem Participation Guide](../guides/ecosystem-participation-guide.md) 
- [BiomeOS Primal SDK API](../api/biomeos-primal-sdk.md)
- [Discovery & Health Monitoring API](../api/discovery-and-health-monitoring.md)
- [CLI Tools Documentation](../../CLI_TOOLS_README.md)

## Decision Record

**Date**: 2025-07-22  
**Decided by**: BiomeOS Architecture Team  
**Stakeholders**: Core team, community contributors, ecosystem participants  
**Review Date**: 2025-12-22 (6 months) 