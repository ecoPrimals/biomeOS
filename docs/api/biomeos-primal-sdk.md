# BiomeOS Primal SDK API Documentation

## 🌟 **Universal, Capability-Based Development Kit**

The BiomeOS Primal SDK enables **universal, agnostic development** of services that participate in the BiomeOS ecosystem. The SDK is designed around **capabilities**, not service names, ensuring your primals work in any ecosystem architecture.

### **🎯 Key Principles**
- **🚫 Zero Hardcoding**: No assumptions about specific service names or ecosystem structure
- **🎯 Capability-Driven**: Services defined by what they can do, not what they're called
- **🌐 Universal Compatibility**: Works with any BiomeOS-compatible ecosystem
- **🔧 Future-Proof**: Adapts to ecosystem evolution without code changes

---

## 📋 **Core Types Overview**

| Type | Purpose | Usage |
|------|---------|-------|
| [`EcoPrimal`](#ecoprimal-trait) | Core primal interface | Implement this trait for your service |
| [`PrimalType`](#primaltype) | Service classification | Define your service category and identity |
| [`PrimalCapability`](#primalcapability) | Service capabilities | Declare what your service can do |
| [`PrimalRequest`](#primalrequest) | Request format | Handle incoming requests |
| [`PrimalResponse`](#primalresponse) | Response format | Send responses back |
| [`PrimalMetadata`](#primalmetadata) | Service metadata | Describe your service |
| [`PrimalHealth`](#primalhealth) | Health status | Report service health |

---

## 🚀 **EcoPrimal Trait**

The core interface that all BiomeOS primals must implement.

### **Definition**
```rust
#[async_trait::async_trait]
pub trait EcoPrimal: Send + Sync {
    /// Get primal metadata
    fn metadata(&self) -> &PrimalMetadata;
    
    /// Get primal capabilities
    fn capabilities(&self) -> &[PrimalCapability];
    
    /// Initialize the primal
    async fn initialize(&self, config: &PrimalConfig) -> PrimalResult<()>;
    
    /// Handle a primal request
    async fn handle_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse>;
    
    /// Get health status
    async fn health_check(&self) -> PrimalResult<PrimalHealth>;
    
    /// Shutdown the primal
    async fn shutdown(&self) -> PrimalResult<()>;
}
```

### **Implementation Example**
```rust
use biomeos_primal_sdk::*;

pub struct MyOrchestrationService {
    metadata: PrimalMetadata,
    capabilities: Vec<PrimalCapability>,
}

#[async_trait::async_trait]
impl EcoPrimal for MyOrchestrationService {
    fn metadata(&self) -> &PrimalMetadata {
        &self.metadata
    }
    
    fn capabilities(&self) -> &[PrimalCapability] {
        &self.capabilities
    }
    
    async fn initialize(&self, config: &PrimalConfig) -> PrimalResult<()> {
        // Initialize your service
        println!("Initializing orchestration service with config: {:?}", config);
        Ok(())
    }
    
    async fn handle_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        match request.method.as_str() {
            "route_message" => {
                // Handle message routing
                let response_payload = serde_json::json!({
                    "status": "routed",
                    "destination": "target-service"
                });
                Ok(PrimalResponse::success(request.request_id, response_payload))
            },
            "discover_services" => {
                // Handle service discovery
                let services = serde_json::json!([
                    {"name": "service-a", "endpoint": "http://svc-a:8080"},
                    {"name": "service-b", "endpoint": "http://svc-b:9000"}
                ]);
                Ok(PrimalResponse::success(request.request_id, services))
            },
            _ => {
                Err(PrimalError::not_found(format!("Unknown method: {}", request.method)))
            }
        }
    }
    
    async fn health_check(&self) -> PrimalResult<PrimalHealth> {
        // Check your service health
        Ok(PrimalHealth::Healthy)
    }
    
    async fn shutdown(&self) -> PrimalResult<()> {
        // Clean shutdown
        println!("Shutting down orchestration service");
        Ok(())
    }
}

impl MyOrchestrationService {
    pub fn new() -> Self {
        let capabilities = vec![
            PrimalCapability::service_discovery(),
            PrimalCapability::message_routing(),
            PrimalCapability::load_balancing(),
        ];
        
        let metadata = PrimalMetadata::new(
            "my-orchestration",
            "1.0.0",
            "Advanced orchestration service with routing and discovery",
            PrimalType::new("orchestration", "my-orchestration", "1.0.0"),
            capabilities.clone(),
        );
        
        Self { metadata, capabilities }
    }
}
```

---

## 🏷️ **PrimalType**

Universal service classification system that replaces hardcoded enums.

### **Definition**
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimalType {
    pub category: String,
    pub name: String,
    pub version: String,
    pub metadata: HashMap<String, String>,
}
```

### **Core Methods**
```rust
impl PrimalType {
    /// Create a new primal type
    pub fn new(category: &str, name: &str, version: &str) -> Self
    
    /// Create with metadata
    pub fn with_metadata(category: &str, name: &str, version: &str, metadata: HashMap<String, String>) -> Self
    
    /// Check if this primal provides a specific category
    pub fn is_category(&self, category: &str) -> bool
    
    /// Check if this primal has a specific name
    pub fn is_name(&self, name: &str) -> bool
}
```

### **Usage Examples**

#### **Custom Service Types**
```rust
// Universal approach - define any service type
let ai_service = PrimalType::new("ai", "language-model", "2.1.0");
let blockchain_service = PrimalType::new("blockchain", "ethereum-bridge", "1.0.0");
let gaming_engine = PrimalType::new("gaming", "physics-engine", "3.0.0");
```

#### **Community Services**
```rust
// Community-contributed services
let community_service = PrimalType::community("super-optimizer", "performance");
let team_service = PrimalType::new("custom", "team-alpha-processor", "1.0.0");
```

#### **Legacy Compatibility**
```rust
// Still supports known service types for compatibility
let toadstool = PrimalType::toadstool();
let songbird = PrimalType::songbird(); 
let nestgate = PrimalType::nestgate();
let beardog = PrimalType::beardog();
```

---

## ⚡ **PrimalCapability**

Flexible capability declaration system.

### **Definition**
```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimalCapability {
    pub domain: String,
    pub name: String,
    pub version: String,
    pub parameters: HashMap<String, serde_json::Value>,
}
```

### **Core Methods**
```rust
impl PrimalCapability {
    /// Create a new capability
    pub fn new(domain: &str, name: &str, version: &str) -> Self
    
    /// Create with parameters
    pub fn with_parameters(domain: &str, name: &str, version: &str, parameters: HashMap<String, serde_json::Value>) -> Self
    
    /// Check domain match
    pub fn matches_domain(&self, domain: &str) -> bool
    
    /// Check name match
    pub fn matches_name(&self, name: &str) -> bool
    
    /// Check if satisfies requirements
    pub fn satisfies(&self, required_domain: &str, required_name: &str) -> bool
}
```

### **Built-in Capabilities**
```rust
// Networking capabilities
PrimalCapability::service_discovery()  // networking/service_discovery:v1
PrimalCapability::message_routing()    // networking/message_routing:v1
PrimalCapability::load_balancing()     // orchestration/load_balancing:v1
PrimalCapability::service_mesh()       // networking/service_mesh:v1

// Security capabilities
PrimalCapability::authentication()     // security/authentication:v1
PrimalCapability::authorization()      // security/authorization:v1
PrimalCapability::key_management()     // security/key_management:v1
PrimalCapability::sandboxing()         // security/sandboxing:v1

// System capabilities
PrimalCapability::system_management()  // system/management:v1
PrimalCapability::plugin_management()  // system/plugin_management:v1
PrimalCapability::code_execution()     // compute/code_execution:v1
```

### **Custom Capabilities**
```rust
// Simple custom capabilities
let ai_capability = PrimalCapability::custom("natural-language", "Advanced NLP processing");
let blockchain_capability = PrimalCapability::custom("smart-contracts", "Ethereum contract execution");

// Structured custom capabilities
let mut params = HashMap::new();
params.insert("max_throughput".to_string(), serde_json::Value::Number(1000.into()));
params.insert("supported_protocols".to_string(), serde_json::Value::Array(vec![
    serde_json::Value::String("http".to_string()),
    serde_json::Value::String("grpc".to_string()),
]));

let high_perf_routing = PrimalCapability::with_parameters(
    "networking",
    "high_performance_routing", 
    "v2",
    params
);
```

---

## 📨 **PrimalRequest**

Standard request format for primal communication.

### **Definition**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRequest {
    pub request_id: uuid::Uuid,
    pub method: String,
    pub payload: serde_json::Value,
    pub metadata: HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub source: Option<String>,
    pub timeout_ms: Option<u64>,
    pub required_capabilities: Vec<PrimalCapability>,
}
```

### **Core Methods**
```rust
impl PrimalRequest {
    /// Create a new request
    pub fn new(method: impl Into<String>, payload: serde_json::Value) -> Self
    
    /// Create with source identification
    pub fn with_source(method: impl Into<String>, payload: serde_json::Value, source: impl Into<String>) -> Self
    
    /// Add required capability
    pub fn require_capability(mut self, capability: PrimalCapability) -> Self
}
```

### **Usage Examples**
```rust
// Simple request
let request = PrimalRequest::new(
    "process_data",
    serde_json::json!({"data": "some important data"})
);

// Request with source tracking
let request = PrimalRequest::with_source(
    "route_message",
    serde_json::json!({"destination": "target-service", "message": "hello"}),
    "client-service-v1.2.3"
);

// Request with capability requirements
let request = PrimalRequest::new(
    "high_performance_compute",
    serde_json::json!({"algorithm": "matrix_multiply", "size": 10000})
)
.require_capability(PrimalCapability::code_execution())
.require_capability(PrimalCapability::custom("gpu-acceleration", "CUDA support"));
```

---

## 📤 **PrimalResponse**

Standard response format for primal communication.

### **Definition**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResponse {
    pub request_id: uuid::Uuid,
    pub status: ResponseStatus,
    pub payload: serde_json::Value,
    pub metadata: HashMap<String, String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub processing_time_ms: Option<u64>,
    pub provided_capabilities: Vec<PrimalCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseStatus {
    Success,
    Error { code: String, message: String },
    PartialSuccess,
}
```

### **Core Methods**
```rust
impl PrimalResponse {
    /// Create a successful response
    pub fn success(request_id: uuid::Uuid, payload: serde_json::Value) -> Self
    
    /// Create an error response
    pub fn error(request_id: uuid::Uuid, code: impl Into<String>, message: impl Into<String>) -> Self
    
    /// Create with processing time
    pub fn success_with_timing(request_id: uuid::Uuid, payload: serde_json::Value, processing_time_ms: u64) -> Self
}
```

### **Usage Examples**
```rust
// Successful response
let response = PrimalResponse::success(
    request.request_id,
    serde_json::json!({
        "result": "operation completed",
        "processed_items": 1337,
        "status": "success"
    })
);

// Error response
let error_response = PrimalResponse::error(
    request.request_id,
    "INVALID_INPUT",
    "The provided data format is not supported"
);

// Response with timing info
let timed_response = PrimalResponse::success_with_timing(
    request.request_id,
    serde_json::json!({"computed_result": 42}),
    250 // 250ms processing time
);
```

---

## 🏥 **PrimalHealth**

Health status enumeration with convenience methods.

### **Definition**
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrimalHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}
```

### **Core Methods**
```rust
impl PrimalHealth {
    /// Convenience constructors
    pub fn healthy() -> Self
    pub fn degraded() -> Self
    pub fn unhealthy() -> Self
    pub fn unknown() -> Self
    
    /// Health check
    pub fn is_healthy(&self) -> bool
}
```

### **Usage Examples**
```rust
async fn health_check(&self) -> PrimalResult<PrimalHealth> {
    // Check database connection
    if !self.database.is_connected() {
        return Ok(PrimalHealth::unhealthy());
    }
    
    // Check response time
    if self.average_response_time_ms > 5000 {
        return Ok(PrimalHealth::degraded());
    }
    
    // All systems operational
    Ok(PrimalHealth::healthy())
}
```

---

## 📋 **PrimalMetadata**

Comprehensive service metadata for discovery and management.

### **Definition**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub primal_type: PrimalType,
    pub capabilities: Vec<PrimalCapability>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub homepage: Option<String>,
    pub tags: Vec<String>,
    pub keywords: Vec<String>,
    pub min_biomeos_version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
```

### **Usage Example**
```rust
let metadata = PrimalMetadata::new(
    "advanced-ai-processor",
    "2.1.0",
    "High-performance AI inference and training service with GPU acceleration",
    PrimalType::new("ai", "inference-engine", "2.1.0"),
    vec![
        PrimalCapability::code_execution(),
        PrimalCapability::custom("gpu-compute", "CUDA and OpenCL support"),
        PrimalCapability::custom("model-serving", "TensorFlow and PyTorch models"),
    ]
)
.with_author("AI Team <ai-team@company.com>")
.with_license("MIT")
.with_repository("https://github.com/company/ai-processor")
.with_documentation("https://docs.company.com/ai-processor")
.with_tags(vec!["ai", "gpu", "inference", "training"]);
```

---

## ❌ **Error Handling**

Comprehensive error system with standard error types.

### **PrimalError**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalError {
    pub code: String,
    pub message: String,
    pub details: Option<HashMap<String, serde_json::Value>>,
}
```

### **Common Error Constructors**
```rust
// Standard errors
let invalid_req = PrimalError::invalid_request("Missing required field 'data'");
let not_found = PrimalError::not_found("Service 'unknown-service' not found");
let internal = PrimalError::internal_error("Database connection failed");
let timeout = PrimalError::timeout("Request exceeded 30s timeout");
let unauthorized = PrimalError::unauthorized("API key is invalid or expired");

// Custom errors with details
let mut details = HashMap::new();
details.insert("retry_after".to_string(), serde_json::Value::Number(300.into()));
details.insert("error_id".to_string(), serde_json::Value::String("ERR_001".to_string()));

let rate_limit = PrimalError::with_details(
    "RATE_LIMIT_EXCEEDED",
    "Too many requests, please retry after 5 minutes",
    details
);
```

### **PrimalResult Type**
```rust
pub type PrimalResult<T> = Result<T, PrimalError>;

// Usage in trait methods
async fn handle_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
    // Your implementation
    if request.payload.is_null() {
        return Err(PrimalError::invalid_request("Payload cannot be null"));
    }
    
    // Process request...
    Ok(PrimalResponse::success(request.request_id, result))
}
```

---

## 🌟 **Complete Example: Universal Service**

Here's a complete example of a universal service that works in any BiomeOS ecosystem:

```rust
use biomeos_primal_sdk::*;
use std::collections::HashMap;
use serde_json;

pub struct UniversalProcessingService {
    metadata: PrimalMetadata,
    capabilities: Vec<PrimalCapability>,
    is_initialized: bool,
}

impl UniversalProcessingService {
    pub fn new() -> Self {
        let capabilities = vec![
            PrimalCapability::code_execution(),
            PrimalCapability::custom("data-processing", "High-throughput data processing"),
            PrimalCapability::custom("batch-processing", "Large dataset batch operations"),
        ];
        
        let primal_type = PrimalType::new("compute", "universal-processor", "1.0.0");
        
        let metadata = PrimalMetadata::new(
            "universal-processor",
            "1.0.0", 
            "Universal data processing service compatible with any BiomeOS ecosystem",
            primal_type,
            capabilities.clone(),
        );
        
        Self {
            metadata,
            capabilities,
            is_initialized: false,
        }
    }
}

#[async_trait::async_trait]
impl EcoPrimal for UniversalProcessingService {
    fn metadata(&self) -> &PrimalMetadata {
        &self.metadata
    }
    
    fn capabilities(&self) -> &[PrimalCapability] {
        &self.capabilities
    }
    
    async fn initialize(&self, config: &PrimalConfig) -> PrimalResult<()> {
        println!("🚀 Initializing Universal Processing Service");
        println!("📋 Config: {:?}", config.name);
        println!("⚡ Capabilities: {:?}", config.capabilities);
        
        // Initialize your service (database connections, etc.)
        Ok(())
    }
    
    async fn handle_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        let start_time = std::time::Instant::now();
        
        let result = match request.method.as_str() {
            "process_data" => {
                let data = request.payload.get("data")
                    .ok_or_else(|| PrimalError::invalid_request("Missing 'data' field"))?;
                
                // Process the data (your business logic here)
                let processed = serde_json::json!({
                    "original": data,
                    "processed": format!("PROCESSED: {}", data),
                    "timestamp": chrono::Utc::now(),
                    "processing_node": "universal-processor"
                });
                
                Ok(processed)
            },
            
            "batch_process" => {
                let items = request.payload.get("items")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| PrimalError::invalid_request("Missing or invalid 'items' array"))?;
                
                let processed_items: Vec<_> = items.iter()
                    .map(|item| format!("BATCH_PROCESSED: {}", item))
                    .collect();
                
                Ok(serde_json::json!({
                    "processed_count": processed_items.len(),
                    "results": processed_items,
                    "batch_id": uuid::Uuid::new_v4()
                }))
            },
            
            "get_capabilities" => {
                let caps: Vec<_> = self.capabilities.iter()
                    .map(|cap| serde_json::json!({
                        "domain": cap.domain,
                        "name": cap.name,
                        "version": cap.version
                    }))
                    .collect();
                
                Ok(serde_json::json!({
                    "capabilities": caps,
                    "service_type": self.metadata.primal_type,
                    "version": self.metadata.version
                }))
            },
            
            _ => {
                return Err(PrimalError::not_found(format!("Unknown method: {}", request.method)));
            }
        };
        
        match result {
            Ok(payload) => {
                let processing_time = start_time.elapsed().as_millis() as u64;
                Ok(PrimalResponse::success_with_timing(request.request_id, payload, processing_time))
            },
            Err(error) => Err(error)
        }
    }
    
    async fn health_check(&self) -> PrimalResult<PrimalHealth> {
        if !self.is_initialized {
            return Ok(PrimalHealth::degraded());
        }
        
        // Add your health checks here
        // - Database connectivity
        // - External service availability  
        // - Resource usage
        // - Performance metrics
        
        Ok(PrimalHealth::healthy())
    }
    
    async fn shutdown(&self) -> PrimalResult<()> {
        println!("🛑 Shutting down Universal Processing Service");
        // Clean shutdown logic
        Ok(())
    }
}

// Usage example
#[tokio::main]
async fn main() -> PrimalResult<()> {
    let service = UniversalProcessingService::new();
    
    // Initialize
    let config = PrimalConfig {
        name: "my-processor".to_string(),
        primal_type: service.metadata().primal_type.clone(),
        capabilities: service.capabilities().to_vec(),
        configuration: HashMap::new(),
    };
    
    service.initialize(&config).await?;
    
    // Handle a request
    let request = PrimalRequest::new(
        "process_data",
        serde_json::json!({"data": "Hello, BiomeOS!"})
    );
    
    let response = service.handle_request(request).await?;
    println!("✅ Response: {:?}", response);
    
    // Check health
    let health = service.health_check().await?;
    println!("🏥 Health: {:?}", health);
    
    Ok(())
}
```

---

## 🌐 **Ecosystem Integration Patterns**

### **Pattern 1: Capability Registration**
```rust
// Register your service with discovery systems
let registration = ServiceRegistration {
    name: service.metadata().name.clone(),
    primal_type: service.metadata().primal_type.clone(),
    capabilities: service.capabilities().to_vec(),
    endpoint: "http://my-service:8080".to_string(),
    health_check_endpoint: "http://my-service:8080/health".to_string(),
};

// Works with any discovery system (Songbird, registry, or custom)
discovery_client.register_service(registration).await?;
```

### **Pattern 2: Dynamic Capability Matching**
```rust
// Find services by capability, not by name
let required_caps = vec![
    PrimalCapability::code_execution(),
    PrimalCapability::custom("gpu-compute", "CUDA support"),
];

let compatible_services = discovery_client
    .find_by_capabilities(required_caps)
    .await?;
```

### **Pattern 3: Multi-Provider Compatibility**
```rust
// Your service works with any ecosystem architecture
match ecosystem_type {
    EcosystemType::Monolithic => {
        // Works with single Songbird providing all orchestration
    },
    EcosystemType::Microservices => {
        // Works with specialized routing, discovery, load-balancing services
    },
    EcosystemType::Community => {
        // Works with community-built alternatives and extensions
    },
}
```

---

## 🎯 **Best Practices**

### **✅ Universal Design**
- Use `PrimalCapability` for service discovery, not service names
- Design for capability composition, not monolithic services
- Support dynamic capability negotiation
- Implement graceful degradation when capabilities are unavailable

### **✅ Error Handling**
- Always use `PrimalResult<T>` for operation results
- Provide detailed error messages with actionable information
- Include error codes for programmatic handling
- Add context with error details when helpful

### **✅ Performance**
- Include processing time in responses for monitoring
- Implement health checks that reflect actual service capability
- Use appropriate timeouts for external dependencies
- Monitor and report resource usage

### **✅ Observability**
- Log all requests and responses (excluding sensitive data)
- Provide detailed metadata for service discovery
- Implement comprehensive health checks
- Support graceful shutdown procedures

---

## 🌟 **Why This SDK Enables True Universality**

The BiomeOS Primal SDK achieves **true universal compatibility** by:

1. **🚫 Eliminating Hardcoded Dependencies**: Services are discovered by capability, not by name
2. **🎯 Capability-Driven Architecture**: What matters is what a service *can do*, not what it's called
3. **🌐 Ecosystem Agnostic Design**: Same code works with monolithic, microservice, or hybrid architectures  
4. **🔧 Future-Proof Interfaces**: New capabilities can be added without breaking existing code
5. **🚀 Community Extensibility**: Anyone can contribute new service types and capabilities

**Your primal built with this SDK will work in ANY BiomeOS ecosystem - today and in the future!** 🎉 