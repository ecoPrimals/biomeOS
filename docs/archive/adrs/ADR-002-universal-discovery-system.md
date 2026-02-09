# ADR-002: Universal Discovery System Design

## Status
**Accepted** - Implemented in biomeos-core

## Context

BiomeOS requires a discovery system that works universally across different ecosystem architectures without hardcoding assumptions about specific services or topologies.

### The Challenge

Different ecosystems may use different discovery mechanisms:
- **Monolithic Ecosystems**: Single service providing discovery functionality
- **Registry-Based**: Centralized service registry (like Consul, etcd)
- **Service Mesh**: Discovery via service mesh (like Istio, Linkerd)
- **Network Scanning**: Auto-discovery via network protocols
- **DNS-Based**: Service discovery via DNS (like SRV records)
- **Hybrid**: Combination of multiple mechanisms

### Requirements

1. **Architecture Agnostic**: Work with any discovery mechanism
2. **Multiple Strategy Support**: Try multiple discovery methods
3. **Graceful Degradation**: Continue working when some discovery methods fail
4. **Capability-Focused**: Discover services by capabilities, not names
5. **Performance**: Efficient discovery with caching
6. **Extensible**: Easy to add new discovery mechanisms

## Decision

**We implement a Universal Discovery System that tries multiple discovery strategies and aggregates results based on capabilities.**

### Architecture

```rust
pub struct UniversalBiomeOSManager {
    discovery_service: Arc<PrimalDiscoveryService>,
    // ... other components
}

impl UniversalBiomeOSManager {
    /// Primary discovery method - capability-based
    pub async fn discover_by_capability(
        &self, 
        endpoint: &str, 
        required_capabilities: &[PrimalCapability]
    ) -> Result<Vec<DiscoveryResult>>
    
    /// Registry-based discovery
    pub async fn discover_registry(&self, registry_url: &str) -> Result<Vec<DiscoveryResult>>
    
    /// Network scanning discovery
    pub async fn discover_network_scan(&self) -> Result<Vec<DiscoveryResult>>
    
    /// Service-discovery provider discovery (meta-discovery)
    pub async fn discover_service_discovery_providers(&self, endpoints: &[String]) -> Result<Vec<DiscoveryResult>>
}
```

### Discovery Strategy Hierarchy

1. **Capability-Based Discovery** (Primary)
   - Standard API: `GET /api/v1/discovery/services`
   - Capability filtering via headers
   - Works with any service implementing the standard

2. **Registry Discovery** (Fallback)
   - Query centralized service registries
   - Transform registry data to DiscoveryResult format
   - Support multiple registry types

3. **Network Scanning** (Fallback)
   - Scan network ranges for BiomeOS-compatible services
   - Probe standard ports and endpoints
   - Auto-discover based on service signatures

4. **Meta-Discovery** (Orchestration)
   - Find services that provide discovery capabilities
   - Use discovered discovery services for further discovery
   - Build discovery service federation

## Implementation Details

### Universal Discovery API

```rust
/// Standard discovery endpoint that any service can implement
/// GET /api/v1/discovery/services
/// Headers:
///   X-BiomeOS-Discovery: true
///   X-BiomeOS-Required-Capabilities: domain/name:version,domain2/name2:version2

pub async fn discover_by_capability(
    &self, 
    endpoint: &str, 
    required_capabilities: &[PrimalCapability]
) -> Result<Vec<DiscoveryResult>> {
    let discovery_url = format!("{}/api/v1/discovery/services", endpoint);
    
    let response = reqwest::Client::new()
        .get(&discovery_url)
        .header("User-Agent", "BiomeOS-Universal-Manager/1.0")
        .header("X-BiomeOS-Discovery", "true")
        .header("X-BiomeOS-Required-Capabilities", self.capabilities_to_header(required_capabilities))
        .timeout(Duration::from_secs(15))
        .send()
        .await?;
    
    // Parse response and filter by capabilities
    let services = self.parse_discovery_response(response).await?;
    Ok(self.filter_by_capabilities(services, required_capabilities))
}
```

### Multi-Strategy Discovery

```rust
/// Try multiple discovery strategies and aggregate results
pub async fn comprehensive_discovery(&self, required_capabilities: &[PrimalCapability]) -> Result<Vec<DiscoveryResult>> {
    let mut all_results = Vec::new();
    
    // Strategy 1: Try common discovery endpoints
    let common_endpoints = [
        "http://discovery:8080",
        "http://registry:9000", 
        "http://songbird:8080",  // Legacy compatibility
        "http://service-mesh:7000",
    ];
    
    for endpoint in &common_endpoints {
        match self.discover_by_capability(endpoint, required_capabilities).await {
            Ok(mut results) => {
                results.iter_mut().for_each(|r| r.discovered_via = Some(endpoint.to_string()));
                all_results.extend(results);
            },
            Err(e) => tracing::debug!("Discovery failed for {}: {}", endpoint, e),
        }
    }
    
    // Strategy 2: Network scanning
    match self.discover_network_scan().await {
        Ok(mut scan_results) => {
            let filtered: Vec<_> = scan_results.into_iter()
                .filter(|service| self.service_matches_capabilities(&service.capabilities, required_capabilities))
                .collect();
            all_results.extend(filtered);
        },
        Err(e) => tracing::debug!("Network scan failed: {}", e),
    }
    
    // Strategy 3: DNS-based discovery
    match self.discover_dns_services().await {
        Ok(dns_results) => all_results.extend(dns_results),
        Err(e) => tracing::debug!("DNS discovery failed: {}", e),
    }
    
    // Deduplicate and rank results
    Ok(self.deduplicate_and_rank(all_results))
}
```

### Capability Matching Logic

```rust
/// Flexible capability matching with aliases and equivalences
fn service_matches_capabilities(
    &self, 
    service_caps: &[PrimalCapability], 
    required_caps: &[PrimalCapability]
) -> bool {
    if required_caps.is_empty() {
        return true;
    }
    
    required_caps.iter().any(|required| {
        service_caps.iter().any(|service_cap| {
            // Direct match
            service_cap.domain == required.domain && service_cap.name == required.name ||
            // Substring matches
            service_cap.name.contains(&required.name) ||
            required.name.contains(&service_cap.name) ||
            // Known equivalences
            self.capabilities_are_equivalent(&service_cap.name, &required.name)
        })
    })
}

/// Handle common capability aliases and equivalences
fn capabilities_are_equivalent(&self, cap1: &str, cap2: &str) -> bool {
    let normalize = |s: &str| s.to_lowercase().replace('-', "_").replace(' ', "_");
    let norm1 = normalize(cap1);
    let norm2 = normalize(cap2);
    
    if norm1 == norm2 {
        return true;
    }
    
    // Common equivalences
    matches!(
        (norm1.as_str(), norm2.as_str()),
        ("routing", "message_routing") | ("message_routing", "routing") |
        ("load_balancing", "load_balancing") | 
        ("service_mesh", "service_discovery") | ("service_discovery", "service_mesh") |
        ("orchestration", "service_discovery") | ("service_discovery", "orchestration")
    )
}
```

## Rationale

### Why Multiple Discovery Strategies?

1. **Ecosystem Diversity**: Different ecosystems use different discovery mechanisms
2. **Reliability**: Fallback options when primary discovery fails
3. **Migration Support**: Support systems transitioning between discovery methods
4. **Community Flexibility**: Allow community innovations in discovery

### Why Standard API Design?

1. **Interoperability**: Any service can implement the standard discovery API
2. **Future-Proof**: Standard evolves with ecosystem needs
3. **Tool Compatibility**: CLI tools and dashboards work with any compliant service
4. **Testing**: Easier to test and validate discovery implementations

## Examples

### Universal Client Usage

```rust
// Works with any ecosystem architecture
pub struct UniversalClient {
    manager: UniversalBiomeOSManager,
}

impl UniversalClient {
    pub async fn find_ai_services(&self) -> Result<Vec<DiscoveryResult>> {
        // This works whether the ecosystem has:
        // - One AI service providing all capabilities
        // - Multiple specialized AI services
        // - Community-contributed AI extensions
        // - Future AI service architectures
        
        self.manager.discover_by_capability(
            "http://discovery:8080",  // Could be any discovery endpoint
            &[
                PrimalCapability::new("ai", "inference", "v1"),
                PrimalCapability::new("ai", "training", "v1"),
            ]
        ).await
    }
    
    pub async fn adaptive_discovery(&self) -> Result<Vec<DiscoveryResult>> {
        // Try multiple discovery mechanisms automatically
        self.manager.comprehensive_discovery(&[
            PrimalCapability::code_execution(),
            PrimalCapability::message_routing(),
        ]).await
    }
}
```

### Service Discovery Provider Implementation

```rust
// Any service can provide discovery capabilities
#[async_trait::async_trait]
impl EcoPrimal for MyDiscoveryService {
    fn capabilities(&self) -> &[PrimalCapability] {
        &[
            PrimalCapability::service_discovery(),
            PrimalCapability::custom("advanced-filtering", "Advanced service filtering"),
        ]
    }
    
    async fn handle_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        match request.method.as_str() {
            "discover_services" => {
                // Implement your discovery logic
                let services = self.discover_services_internal(&request).await?;
                Ok(PrimalResponse::success(request.request_id, serde_json::to_value(services)?))
            },
            _ => Err(PrimalError::not_found("Unknown method"))
        }
    }
}
```

## Consequences

### Positive Consequences

1. **✅ Universal Compatibility**: Works with any ecosystem architecture
2. **✅ High Availability**: Multiple discovery strategies provide redundancy  
3. **✅ Graceful Degradation**: System continues working when some discovery fails
4. **✅ Innovation Enablement**: Easy to add new discovery mechanisms
5. **✅ Migration Support**: Supports ecosystem transitions smoothly
6. **✅ Community Extensibility**: Anyone can implement discovery services

### Negative Consequences

1. **❌ Complexity**: Multiple strategies increase system complexity
2. **❌ Performance**: Multiple discovery attempts can add latency
3. **❌ Resource Usage**: Network scanning and multiple queries use more resources
4. **❌ Configuration**: More configuration options for users to understand

### Mitigation Strategies

1. **Complexity**: Provide clear abstractions and hide complexity in SDK
2. **Performance**: Implement smart caching and parallel discovery
3. **Resource Usage**: Make discovery strategies configurable and optional
4. **Configuration**: Provide sensible defaults and auto-configuration

## Configuration

### Discovery Configuration
```toml
[discovery]
# Primary discovery strategy
primary_strategy = "capability_based"

# Discovery endpoints to try
endpoints = [
    "http://discovery:8080",
    "http://registry:9000",
    "http://songbird:8080"
]

# Enable network scanning
network_scan_enabled = true
network_scan_ranges = ["192.168.0.0/16", "10.0.0.0/8"]
network_scan_ports = "8000-9000"

# Caching configuration
cache_ttl_seconds = 300
max_cache_entries = 1000

# Timeouts
discovery_timeout_seconds = 15
probe_timeout_seconds = 3
```

### Environment Variables
```bash
# Override discovery endpoints
BIOMEOS_DISCOVERY_ENDPOINTS="http://custom-discovery:8080,http://backup-registry:9001"

# Disable specific strategies
BIOMEOS_NETWORK_SCAN_ENABLED=false

# Performance tuning
BIOMEOS_DISCOVERY_TIMEOUT=30
BIOMEOS_CACHE_TTL=600
```

## Testing Strategy

### Unit Tests
```rust
#[tokio::test]
async fn test_capability_based_discovery() {
    let manager = create_test_manager();
    
    // Mock discovery endpoint
    let mock_server = MockServer::start().await;
    mock_discovery_response(&mock_server, &test_services()).await;
    
    let results = manager.discover_by_capability(
        &mock_server.uri(),
        &[PrimalCapability::new("ai", "inference", "v1")]
    ).await.unwrap();
    
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|r| r.primal_id == "ai-service-1"));
}

#[tokio::test]
async fn test_multi_strategy_discovery() {
    let manager = create_test_manager();
    
    // Test that multiple strategies are tried and results aggregated
    let results = manager.comprehensive_discovery(&[
        PrimalCapability::code_execution()
    ]).await.unwrap();
    
    // Should find services from multiple discovery methods
    assert!(!results.is_empty());
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_real_ecosystem_discovery() {
    // Test against real ecosystem instances
    let manager = UniversalBiomeOSManager::new(BiomeOSConfig::default());
    
    // Should work with different ecosystem configurations
    for ecosystem_type in [EcosystemType::Monolithic, EcosystemType::Microservices] {
        let results = test_discovery_in_ecosystem(&manager, ecosystem_type).await;
        assert!(!results.is_empty(), "Discovery failed for {:?}", ecosystem_type);
    }
}
```

## Future Enhancements

### Planned Features
- **AI-Powered Discovery**: Use ML to improve capability matching
- **Discovery Federation**: Chain discovery services for multi-region support
- **Performance Analytics**: Track discovery performance and optimize
- **Capability Marketplace**: Rate and review service capabilities

### Potential Extensions
- **Blockchain Discovery**: Decentralized service registry on blockchain
- **IoT Discovery**: Specialized discovery for IoT and edge devices
- **Cross-Cloud Discovery**: Discovery across multiple cloud providers
- **Semantic Discovery**: Natural language capability queries

## References

- [ADR-001: Universal Capability-Based Architecture](ADR-001-universal-capability-based-architecture.md)
- [Discovery & Health Monitoring API](../api/discovery-and-health-monitoring.md)
- [CLI Tools Documentation](../../CLI_TOOLS_README.md)

## Decision Record

**Date**: 2025-07-22  
**Decided by**: BiomeOS Architecture Team  
**Stakeholders**: Core team, ecosystem participants, tool developers  
**Review Date**: 2025-12-22 (6 months) 