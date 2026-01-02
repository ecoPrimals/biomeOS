# Universal Primal Client Specification

**Version**: 1.0.0  
**Date**: January 3, 2026  
**Owner**: biomeOS Core Team  
**Status**: Specification & Scaffolding

---

## 🎯 Purpose

The **Universal Primal Client** is biomeOS's agnostic API negotiation layer that enables communication with any primal, regardless of:
- Response format (wrapped vs. unwrapped)
- Protocol (HTTP, gRPC, tarpc, etc.)
- API version
- Schema format (OpenAPI, JSON Schema, etc.)

**Core Principle**: biomeOS adapts to primals, primals don't adapt to biomeOS.

---

## 🏗️ Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────────┐
│                        biomeOS                              │
│                                                             │
│  ┌───────────────────────────────────────────────────┐    │
│  │       Universal Primal Client                     │    │
│  │                                                    │    │
│  │  ┌──────────────┐  ┌──────────────┐              │    │
│  │  │   Format     │  │   Protocol   │              │    │
│  │  │   Adapter    │  │   Adapter    │              │    │
│  │  └──────────────┘  └──────────────┘              │    │
│  │                                                    │    │
│  │  ┌──────────────┐  ┌──────────────┐              │    │
│  │  │   Schema     │  │   Discovery  │              │    │
│  │  │   Parser     │  │   Client     │              │    │
│  │  └──────────────┘  └──────────────┘              │    │
│  └───────────────────────────────────────────────────┘    │
│                           │                                │
└───────────────────────────┼────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
   ┌────▼────┐        ┌────▼────┐        ┌────▼────┐
   │ BearDog │        │Songbird │        │ Future  │
   │         │        │         │        │ Primal  │
   │ Wrapped │        │HTTP+RPC │        │ Unknown │
   │ HTTP    │        │Multicast│        │ Format  │
   └─────────┘        └─────────┘        └─────────┘
```

---

## 📋 Core Components

### 1. Format Adapter

**Purpose**: Handles different response formats transparently

**Supported Formats**:
- **Unwrapped**: `{"encryption_tag": "...", "family_id": "..."}`
- **Wrapped**: `{"success": true, "data": {...}}`
- **Error Objects**: `{"error": {...}}`
- **RFC 7807**: Problem Details for HTTP APIs

**Interface**:
```rust
pub trait FormatAdapter: Send + Sync {
    /// Parse response into expected type, handling any format
    async fn parse<T>(&self, response: Response) -> Result<T>
    where
        T: DeserializeOwned;
    
    /// Check if response indicates success
    fn is_success(&self, response: &Response) -> bool;
    
    /// Extract error information
    fn extract_error(&self, response: Response) -> Result<ApiError>;
}
```

---

### 2. Protocol Adapter

**Purpose**: Abstract over different communication protocols

**Supported Protocols**:
- **HTTP/HTTPS**: Standard REST APIs
- **tarpc**: Rust RPC (Songbird)
- **gRPC**: Future support
- **WebSocket**: Future support

**Interface**:
```rust
pub trait ProtocolAdapter: Send + Sync {
    /// Make a request to endpoint with given method and body
    async fn request(
        &self,
        endpoint: &str,
        method: Method,
        body: Option<Vec<u8>>,
    ) -> Result<Response>;
    
    /// Get protocol identifier
    fn protocol(&self) -> &str;
}
```

---

### 3. Schema Parser

**Purpose**: Understand primal API structure from schema

**Supported Schemas**:
- **OpenAPI 3.x**: Industry standard
- **JSON Schema**: Lightweight alternative
- **GraphQL Schema**: Future support
- **Primal Manifest**: Custom discovery format

**Interface**:
```rust
pub trait SchemaParser: Send + Sync {
    /// Parse schema from bytes
    fn parse(&self, schema_bytes: &[u8]) -> Result<ApiSchema>;
    
    /// Get endpoint information
    fn get_endpoint(&self, schema: &ApiSchema, operation: &str) 
        -> Result<EndpointInfo>;
    
    /// Validate request against schema
    fn validate_request(
        &self,
        schema: &ApiSchema,
        endpoint: &str,
        request: &serde_json::Value,
    ) -> Result<()>;
}
```

---

### 4. Discovery Client

**Purpose**: Find primals and their capabilities

**Discovery Methods**:
- **mDNS**: Local network discovery
- **UDP Multicast**: Songbird-style
- **Consul**: Service mesh
- **Environment Variables**: Explicit endpoints
- **DHT**: Distributed hash table (future)

**Interface**:
```rust
pub trait DiscoveryClient: Send + Sync {
    /// Discover primals with given capability
    async fn discover(&self, capability: &str) -> Result<Vec<PrimalInfo>>;
    
    /// Get primal schema
    async fn get_schema(&self, primal: &PrimalInfo) -> Result<Vec<u8>>;
    
    /// Subscribe to primal lifecycle events
    async fn subscribe(&self) -> Result<Receiver<PrimalEvent>>;
}
```

---

## 🔧 Universal Primal Client API

### Main Client

```rust
pub struct UniversalPrimalClient {
    format_adapters: Vec<Box<dyn FormatAdapter>>,
    protocol_adapters: HashMap<String, Box<dyn ProtocolAdapter>>,
    schema_parsers: HashMap<String, Box<dyn SchemaParser>>,
    discovery: Box<dyn DiscoveryClient>,
    cache: Arc<RwLock<ClientCache>>,
}

impl UniversalPrimalClient {
    /// Create a new universal client
    pub fn new(config: ClientConfig) -> Self;
    
    /// Discover primal by capability
    pub async fn discover_primal(&self, capability: &str) 
        -> Result<PrimalHandle>;
    
    /// Call primal endpoint with automatic format negotiation
    pub async fn call<Req, Res>(
        &self,
        primal: &PrimalHandle,
        operation: &str,
        request: Req,
    ) -> Result<Res>
    where
        Req: Serialize,
        Res: DeserializeOwned;
    
    /// Get primal metadata
    pub async fn get_metadata(&self, primal: &PrimalHandle) 
        -> Result<PrimalMetadata>;
    
    /// Check if primal supports capability
    pub fn has_capability(&self, primal: &PrimalHandle, cap: &str) 
        -> bool;
}
```

---

## 📊 Data Structures

### Primal Handle

```rust
pub struct PrimalHandle {
    pub id: PrimalId,
    pub name: String,
    pub endpoints: Vec<Endpoint>,
    pub capabilities: Vec<String>,
    pub schema: Option<ApiSchema>,
    pub protocol: String,
    pub format_hint: Option<FormatHint>,
}

pub struct Endpoint {
    pub url: String,
    pub protocol: String,
    pub priority: u8,
}

pub enum FormatHint {
    Wrapped,
    Unwrapped,
    StatusCodeBased,
    Unknown,
}
```

### API Schema

```rust
pub struct ApiSchema {
    pub version: String,
    pub format: SchemaFormat,
    pub operations: HashMap<String, Operation>,
    pub types: HashMap<String, TypeDefinition>,
}

pub struct Operation {
    pub method: Method,
    pub path: String,
    pub request_type: Option<String>,
    pub response_type: String,
    pub error_type: Option<String>,
}

pub enum SchemaFormat {
    OpenApi3,
    JsonSchema,
    Custom(String),
}
```

### Primal Metadata

```rust
pub struct PrimalMetadata {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<Capability>,
    pub api_version: String,
    pub schema_url: Option<String>,
    pub health_endpoint: Option<String>,
}

pub struct Capability {
    pub name: String,
    pub version: String,
    pub operations: Vec<String>,
}
```

---

## 🔄 Request Flow

### Automatic Format Negotiation

```
┌─────────────────────────────────────────────────────────────┐
│ 1. biomeOS calls: client.call(beardog, "get_identity", ())  │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. Universal Client:                                        │
│    - Check cache for BearDog schema                         │
│    - If not cached, discover BearDog                        │
│    - Fetch schema from BearDog                              │
│    - Parse schema (OpenAPI, JSON Schema, etc.)              │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. Schema Parser:                                           │
│    - Find "get_identity" operation in schema                │
│    - Get endpoint: GET /api/v1/trust/identity               │
│    - Get response type: IdentityResponse                    │
│    - Get format hint: wrapped/unwrapped/unknown             │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. Protocol Adapter (HTTP):                                 │
│    - Make GET request to http://localhost:9000/...          │
│    - Get response                                            │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│ 5. Format Adapter:                                          │
│    - Check HTTP status code                                 │
│    - If 2xx: Try format hint first                          │
│    - If hint fails, try all adapters:                       │
│      a. Unwrapped                                           │
│      b. Wrapped (ApiResponse<T>)                            │
│      c. Custom formats                                      │
│    - If 4xx/5xx: Parse error                                │
│    - Return Result<IdentityResponse>                        │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│ 6. Cache:                                                    │
│    - Cache successful format for future requests            │
│    - Cache schema for primal                                │
└───────────────────────┬─────────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────────┐
│ 7. Return to biomeOS: Ok(IdentityResponse {...})           │
└─────────────────────────────────────────────────────────────┘
```

---

## 🎯 Usage Examples

### Example 1: Basic Call

```rust
use biomeos::primal_client::UniversalPrimalClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Create client
    let client = UniversalPrimalClient::new(ClientConfig::default());
    
    // Discover BearDog by capability
    let beardog = client.discover_primal("security").await?;
    
    // Call endpoint - format is handled automatically
    let identity: IdentityResponse = client
        .call(&beardog, "get_identity", ())
        .await?;
    
    println!("BearDog identity: {:?}", identity);
    
    Ok(())
}
```

### Example 2: With Error Handling

```rust
match client.call(&beardog, "get_identity", ()).await {
    Ok(identity) => {
        println!("Got identity: {}", identity.encryption_tag);
    }
    Err(ApiError::Unauthorized { .. }) => {
        println!("Not authorized to get identity");
    }
    Err(ApiError::NotFound { .. }) => {
        println!("Endpoint not found");
    }
    Err(e) => {
        println!("Request failed: {}", e);
    }
}
```

### Example 3: Multiple Primals

```rust
// Discover all primals with "orchestration" capability
let orchestrators = client.discover_primal("orchestration").await?;

for orchestrator in orchestrators {
    println!("Found: {} ({})", orchestrator.name, orchestrator.id);
    
    // Get metadata
    let metadata = client.get_metadata(&orchestrator).await?;
    println!("  Capabilities: {:?}", metadata.capabilities);
    
    // Check specific capability
    if client.has_capability(&orchestrator, "federation") {
        println!("  ✅ Supports federation");
    }
}
```

### Example 4: Trust Evaluation

```rust
// Discover security provider
let security = client.discover_primal("security").await?;

// Evaluate trust for peer
let request = TrustEvaluationRequest {
    peer_id: "tower-2".to_string(),
    peer_tags: vec!["beardog:family:iidn:tower2".to_string()],
    connection_info: Default::default(),
};

let response: TrustEvaluationResponse = client
    .call(&security, "evaluate_trust", request)
    .await?;

match response.decision.as_str() {
    "auto_accept" => println!("✅ Auto-accept ({})", response.reason),
    "prompt_user" => println!("⚠️  Prompt user ({})", response.reason),
    "reject" => println!("❌ Reject ({})", response.reason),
    _ => println!("❓ Unknown decision: {}", response.decision),
}
```

---

## 🔒 Security Considerations

### Authentication

```rust
pub struct ClientConfig {
    // Authentication method
    pub auth: Option<AuthMethod>,
    
    // TLS configuration
    pub tls: Option<TlsConfig>,
    
    // Timeout settings
    pub timeouts: TimeoutConfig,
}

pub enum AuthMethod {
    None,
    BearerToken(String),
    ApiKey { key: String, header: String },
    MutualTls { cert: Vec<u8>, key: Vec<u8> },
    GeneticLineage { tag: String, proof: Vec<u8> },
}
```

### Trust Verification

```rust
impl UniversalPrimalClient {
    /// Verify primal identity before making calls
    pub async fn verify_primal(&self, primal: &PrimalHandle) 
        -> Result<VerificationResult>;
    
    /// Set trust policy
    pub fn set_trust_policy(&mut self, policy: TrustPolicy);
}

pub enum TrustPolicy {
    /// Trust all primals (development only)
    TrustAll,
    
    /// Trust primals with genetic lineage from same family
    GeneticLineage { family_id: String },
    
    /// Trust primals with specific tags
    Tags { required_tags: Vec<String> },
    
    /// Custom trust evaluation
    Custom(Box<dyn Fn(&PrimalHandle) -> bool + Send + Sync>),
}
```

---

## 📁 File Structure

```
crates/biomeos-core/src/
├── primal_client/
│   ├── mod.rs                    # Main client
│   ├── client.rs                 # UniversalPrimalClient impl
│   ├── handle.rs                 # PrimalHandle types
│   ├── adapters/
│   │   ├── mod.rs
│   │   ├── format/
│   │   │   ├── mod.rs
│   │   │   ├── unwrapped.rs     # Unwrapped format
│   │   │   ├── wrapped.rs       # Wrapped ApiResponse<T>
│   │   │   ├── rfc7807.rs       # RFC 7807 Problem Details
│   │   │   └── auto.rs          # Auto-detect format
│   │   └── protocol/
│   │       ├── mod.rs
│   │       ├── http.rs          # HTTP/HTTPS adapter
│   │       ├── tarpc.rs         # tarpc adapter
│   │       └── grpc.rs          # gRPC adapter (future)
│   ├── schema/
│   │   ├── mod.rs
│   │   ├── openapi.rs           # OpenAPI parser
│   │   ├── json_schema.rs       # JSON Schema parser
│   │   └── manifest.rs          # Primal manifest parser
│   ├── discovery/
│   │   ├── mod.rs
│   │   ├── mdns.rs              # mDNS discovery
│   │   ├── multicast.rs         # UDP multicast
│   │   ├── consul.rs            # Consul discovery
│   │   └── env.rs               # Environment variable config
│   ├── cache.rs                 # Response/schema cache
│   ├── error.rs                 # Error types
│   └── config.rs                # Configuration types
│
└── types/
    └── primal.rs                # Shared primal types
```

---

## 🧪 Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_unwrapped_format() {
        let adapter = UnwrappedFormatAdapter::new();
        let response = mock_response(200, r#"{"key": "value"}"#);
        
        let data: HashMap<String, String> = adapter
            .parse(response)
            .await
            .unwrap();
        
        assert_eq!(data.get("key"), Some(&"value".to_string()));
    }
    
    #[tokio::test]
    async fn test_wrapped_format() {
        let adapter = WrappedFormatAdapter::new();
        let response = mock_response(
            200,
            r#"{"success": true, "data": {"key": "value"}}"#
        );
        
        let data: HashMap<String, String> = adapter
            .parse(response)
            .await
            .unwrap();
        
        assert_eq!(data.get("key"), Some(&"value".to_string()));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_beardog_integration() {
    // Start mock BearDog server
    let mock_server = MockServer::start().await;
    
    // Configure client
    let client = UniversalPrimalClient::builder()
        .discovery_endpoint(&mock_server.uri())
        .build();
    
    // Discover BearDog
    let beardog = client.discover_primal("security").await.unwrap();
    
    // Call identity endpoint
    let identity: IdentityResponse = client
        .call(&beardog, "get_identity", ())
        .await
        .unwrap();
    
    assert_eq!(identity.family_id, "test-family");
}
```

---

## 📊 Performance Considerations

### Caching Strategy

```rust
pub struct ClientCache {
    // Cache primal schemas (TTL: 1 hour)
    schemas: LruCache<PrimalId, (ApiSchema, Instant)>,
    
    // Cache successful format adapters (TTL: 5 minutes)
    formats: LruCache<PrimalId, FormatHint>,
    
    // Cache discovered primals (TTL: 30 seconds)
    primals: LruCache<String, Vec<PrimalHandle>>,
}
```

### Connection Pooling

```rust
impl HttpProtocolAdapter {
    pub fn new(config: HttpConfig) -> Self {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();
        
        Self { client, config }
    }
}
```

---

## 🚀 Implementation Phases

### Phase 1: Foundation (This Week)
- [ ] Create module structure
- [ ] Implement basic UniversalPrimalClient
- [ ] HTTP protocol adapter
- [ ] Unwrapped format adapter
- [ ] Wrapped format adapter
- [ ] Environment variable discovery
- [ ] Basic error types

### Phase 2: Schema Support (Next Week)
- [ ] OpenAPI 3.x parser
- [ ] JSON Schema parser
- [ ] Schema-based validation
- [ ] Schema caching
- [ ] Auto-format detection

### Phase 3: Advanced Discovery (Week 3)
- [ ] mDNS discovery client
- [ ] UDP multicast discovery
- [ ] Consul integration
- [ ] Discovery caching
- [ ] Primal lifecycle events

### Phase 4: Additional Protocols (Week 4)
- [ ] tarpc protocol adapter
- [ ] gRPC protocol adapter
- [ ] WebSocket protocol adapter
- [ ] Protocol negotiation

### Phase 5: Production Hardening (Week 5+)
- [ ] Comprehensive error handling
- [ ] Retry logic with exponential backoff
- [ ] Circuit breaker pattern
- [ ] Metrics and observability
- [ ] Performance benchmarks

---

## 📝 Configuration Example

```toml
# biomeos.toml

[primal_client]
# Cache TTLs
schema_cache_ttl = "1h"
format_cache_ttl = "5m"
discovery_cache_ttl = "30s"

# Timeouts
request_timeout = "30s"
connect_timeout = "10s"

# Connection pooling
max_idle_per_host = 10
pool_idle_timeout = "90s"

# Discovery methods (tried in order)
discovery_methods = ["env", "mdns", "multicast"]

# Trust policy
trust_policy = "genetic_lineage"
family_id = "iidn"

# Retry configuration
max_retries = 3
retry_backoff = "exponential"

[primal_client.endpoints]
# Explicit endpoint configuration (overrides discovery)
beardog = "http://localhost:9000"
songbird = "http://localhost:8080"
```

---

## 🎯 Success Criteria

### Functional Requirements
- ✅ Communicate with primals using any response format
- ✅ Automatically detect and adapt to primal APIs
- ✅ Support multiple protocols (HTTP, tarpc, gRPC)
- ✅ Parse OpenAPI and JSON Schema specifications
- ✅ Discover primals via mDNS, multicast, or explicit config
- ✅ Cache schemas and format hints for performance
- ✅ Handle errors gracefully with retry logic

### Non-Functional Requirements
- **Performance**: < 10ms overhead for cached calls
- **Reliability**: 99.9% success rate for valid requests
- **Compatibility**: Works with any primal following basic conventions
- **Testability**: > 90% code coverage
- **Documentation**: All public APIs documented with examples

---

## 🔗 Related Documents

- [API_EVOLUTION_AGNOSTIC_RESPONSE_HANDLING.md](../API_EVOLUTION_AGNOSTIC_RESPONSE_HANDLING.md)
- [HANDOFF_GENERIC_TRUST_DISCOVERY_INTEGRATION.md](../HANDOFF_GENERIC_TRUST_DISCOVERY_INTEGRATION.md)
- [DYNAMIC_API_SCHEMA_DISCOVERY.md](../docs/api/DYNAMIC_API_SCHEMA_DISCOVERY.md)

---

**Version**: 1.0.0  
**Status**: Specification Complete, Ready for Implementation  
**Owner**: biomeOS Core Team  
**Timeline**: Phase 1 starts immediately, full implementation in 5 weeks

🎯 **biomeOS will be the universal adapter for all primal interactions!** 🎯

