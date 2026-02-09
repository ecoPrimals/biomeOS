# BiomeOS Primal Integration Guide

## 🎯 **Build Universal Primals for Any Ecosystem**

This guide walks you through creating **universal, capability-based primals** that work in any BiomeOS ecosystem - whether it's monolithic, microservices, community-driven, or hybrid architectures.

### **🌟 What You'll Learn**
- ✅ Create truly **universal primals** with zero hardcoded dependencies
- ✅ Implement **capability-based discovery** and service interaction
- ✅ Build **future-proof services** that adapt to ecosystem evolution
- ✅ Integrate with **any BiomeOS architecture** seamlessly

---

## 📋 **Prerequisites**

### **Development Environment**
- **Rust 1.70+** with async support
- **Cargo** for dependency management
- **Basic understanding** of async/await in Rust

### **BiomeOS Dependencies**
```toml
[dependencies]
biomeos-primal-sdk = { path = "../crates/biomeos-primal-sdk" }
biomeos-core = { path = "../crates/biomeos-core" }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
reqwest = { version = "0.11", features = ["json"] }
tracing = "0.1"
```

---

## 🚀 **Step 1: Define Your Primal's Purpose**

### **Capability-First Design**
Instead of thinking "I'm building a new Songbird" or "I'm replacing Toadstool", think in terms of **capabilities**:

```rust
// ❌ DON'T: Hardcoded service assumptions
"I'm building a Songbird replacement"

// ✅ DO: Capability-driven design  
"I'm building a service that provides:
- message routing capabilities
- service discovery capabilities  
- load balancing capabilities"
```

### **Example: AI Processing Primal**
Let's build an AI processing primal as our example:

**Capabilities we'll provide:**
- `ai/inference` - Machine learning inference
- `ai/training` - Model training capabilities
- `compute/gpu` - GPU acceleration
- `data/preprocessing` - Data preprocessing

---

## 🏗️ **Step 2: Project Structure**

Create your primal project:
```bash
cargo new my-ai-primal --lib
cd my-ai-primal
```

**Recommended structure:**
```
my-ai-primal/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Main primal implementation
│   ├── capabilities/   # Capability implementations
│   │   ├── mod.rs
│   │   ├── inference.rs
│   │   ├── training.rs
│   │   └── preprocessing.rs
│   ├── health.rs       # Health monitoring
│   ├── config.rs       # Configuration
│   └── main.rs         # Binary entry point
├── examples/
│   └── usage_demo.rs
└── README.md
```

---

## 🎯 **Step 3: Implement the EcoPrimal Trait**

### **Core Implementation (src/lib.rs)**
```rust
use biomeos_primal_sdk::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod capabilities;
pub mod health;
pub mod config;

use capabilities::*;

pub struct AIProcessingPrimal {
    metadata: PrimalMetadata,
    capabilities: Vec<PrimalCapability>,
    config: Arc<RwLock<AIPrimalConfig>>,
    
    // Capability handlers
    inference_handler: Arc<InferenceHandler>,
    training_handler: Arc<TrainingHandler>,
    preprocessing_handler: Arc<PreprocessingHandler>,
    
    // State
    is_initialized: Arc<RwLock<bool>>,
    request_count: Arc<RwLock<u64>>,
}

impl AIProcessingPrimal {
    pub fn new() -> Self {
        // Define our capabilities (what we can do)
        let capabilities = vec![
            PrimalCapability::new("ai", "inference", "v2.1"),
            PrimalCapability::new("ai", "training", "v1.0"), 
            PrimalCapability::new("compute", "gpu", "v1.0"),
            PrimalCapability::new("data", "preprocessing", "v1.0"),
            
            // Custom capabilities
            PrimalCapability::custom("tensorflow", "TensorFlow model support"),
            PrimalCapability::custom("pytorch", "PyTorch model support"),
            PrimalCapability::custom("huggingface", "HuggingFace transformers"),
        ];
        
        // Define our primal type
        let primal_type = PrimalType::new("ai", "inference-engine", "2.1.0");
        
        // Create metadata
        let metadata = PrimalMetadata::new(
            "ai-processing-primal",
            "2.1.0",
            "Universal AI processing primal with inference, training, and preprocessing capabilities",
            primal_type,
            capabilities.clone(),
        );
        
        Self {
            metadata,
            capabilities,
            config: Arc::new(RwLock::new(AIPrimalConfig::default())),
            inference_handler: Arc::new(InferenceHandler::new()),
            training_handler: Arc::new(TrainingHandler::new()),
            preprocessing_handler: Arc::new(PreprocessingHandler::new()),
            is_initialized: Arc::new(RwLock::new(false)),
            request_count: Arc::new(RwLock::new(0)),
        }
    }
}

#[async_trait::async_trait]
impl EcoPrimal for AIProcessingPrimal {
    fn metadata(&self) -> &PrimalMetadata {
        &self.metadata
    }
    
    fn capabilities(&self) -> &[PrimalCapability] {
        &self.capabilities
    }
    
    async fn initialize(&self, config: &PrimalConfig) -> PrimalResult<()> {
        println!("🤖 Initializing AI Processing Primal");
        
        // Parse our configuration
        let ai_config = AIPrimalConfig::from_primal_config(config)?;
        *self.config.write().await = ai_config;
        
        // Initialize capability handlers
        self.inference_handler.initialize().await?;
        self.training_handler.initialize().await?;
        self.preprocessing_handler.initialize().await?;
        
        *self.is_initialized.write().await = true;
        println!("✅ AI Processing Primal initialized successfully");
        
        Ok(())
    }
    
    async fn handle_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        // Increment request counter
        *self.request_count.write().await += 1;
        
        let start_time = std::time::Instant::now();
        
        // Route to appropriate capability handler
        let result = match request.method.as_str() {
            // AI Inference
            "predict" | "inference" => {
                self.inference_handler.handle_inference(&request).await
            },
            
            // AI Training
            "train" | "fit_model" => {
                self.training_handler.handle_training(&request).await
            },
            
            // Data Preprocessing
            "preprocess" | "transform_data" => {
                self.preprocessing_handler.handle_preprocessing(&request).await
            },
            
            // Capability discovery
            "get_capabilities" => {
                let caps: Vec<_> = self.capabilities.iter()
                    .map(|cap| serde_json::json!({
                        "domain": cap.domain,
                        "name": cap.name,
                        "version": cap.version,
                        "parameters": cap.parameters
                    }))
                    .collect();
                
                Ok(serde_json::json!({
                    "capabilities": caps,
                    "primal_type": self.metadata.primal_type,
                    "version": self.metadata.version,
                    "status": "ready"
                }))
            },
            
            // Health and status
            "ping" => {
                Ok(serde_json::json!({
                    "status": "healthy",
                    "uptime": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    "requests_handled": *self.request_count.read().await
                }))
            },
            
            _ => {
                Err(PrimalError::not_found(format!("Unknown method: {}", request.method)))
            }
        };
        
        // Create response with timing
        match result {
            Ok(payload) => {
                let processing_time = start_time.elapsed().as_millis() as u64;
                Ok(PrimalResponse::success_with_timing(request.request_id, payload, processing_time))
            },
            Err(error) => Err(error)
        }
    }
    
    async fn health_check(&self) -> PrimalResult<PrimalHealth> {
        if !*self.is_initialized.read().await {
            return Ok(PrimalHealth::degraded());
        }
        
        // Check each capability handler
        let inference_healthy = self.inference_handler.is_healthy().await;
        let training_healthy = self.training_handler.is_healthy().await;
        let preprocessing_healthy = self.preprocessing_handler.is_healthy().await;
        
        if !inference_healthy || !training_healthy || !preprocessing_healthy {
            return Ok(PrimalHealth::degraded());
        }
        
        // Check system resources
        let resource_check = health::check_system_resources().await;
        
        match resource_check {
            health::ResourceStatus::Healthy => Ok(PrimalHealth::healthy()),
            health::ResourceStatus::Degraded => Ok(PrimalHealth::degraded()),
            health::ResourceStatus::Critical => Ok(PrimalHealth::unhealthy()),
        }
    }
    
    async fn shutdown(&self) -> PrimalResult<()> {
        println!("🛑 Shutting down AI Processing Primal");
        
        // Shutdown capability handlers
        self.inference_handler.shutdown().await?;
        self.training_handler.shutdown().await?;
        self.preprocessing_handler.shutdown().await?;
        
        *self.is_initialized.write().await = false;
        
        println!("✅ AI Processing Primal shutdown complete");
        Ok(())
    }
}
```

---

## ⚡ **Step 4: Implement Capability Handlers**

### **Inference Handler (src/capabilities/inference.rs)**
```rust
use biomeos_primal_sdk::*;
use serde_json;

pub struct InferenceHandler {
    // Your ML framework integration here
    // e.g., TensorFlow, PyTorch, ONNX runtime
}

impl InferenceHandler {
    pub fn new() -> Self {
        Self {
            // Initialize your ML backends
        }
    }
    
    pub async fn initialize(&self) -> PrimalResult<()> {
        // Load models, initialize GPU, etc.
        println!("🧠 Initializing inference engine");
        Ok(())
    }
    
    pub async fn handle_inference(&self, request: &PrimalRequest) -> PrimalResult<serde_json::Value> {
        // Extract input data
        let input_data = request.payload.get("input")
            .ok_or_else(|| PrimalError::invalid_request("Missing 'input' field"))?;
        
        let model_name = request.payload.get("model")
            .and_then(|m| m.as_str())
            .unwrap_or("default");
        
        // Perform inference (your implementation here)
        let prediction = self.run_inference(model_name, input_data).await?;
        
        Ok(serde_json::json!({
            "prediction": prediction,
            "model": model_name,
            "confidence": 0.95,
            "processing_time_ms": 123,
            "capabilities_used": ["ai/inference", "compute/gpu"]
        }))
    }
    
    async fn run_inference(&self, model: &str, input: &serde_json::Value) -> PrimalResult<serde_json::Value> {
        // Your ML inference implementation
        // This could integrate with:
        // - TensorFlow Serving
        // - PyTorch
        // - ONNX Runtime
        // - Custom inference engines
        
        // For demo purposes:
        Ok(serde_json::json!({
            "result": format!("Inference result for model {} with input {:?}", model, input),
            "class": "example_class",
            "scores": [0.8, 0.15, 0.05]
        }))
    }
    
    pub async fn is_healthy(&self) -> bool {
        // Check if inference engine is working
        // - GPU availability
        // - Model loading status
        // - Memory usage
        true
    }
    
    pub async fn shutdown(&self) -> PrimalResult<()> {
        println!("🔌 Shutting down inference engine");
        Ok(())
    }
}
```

### **Training Handler (src/capabilities/training.rs)**
```rust
use biomeos_primal_sdk::*;
use serde_json;

pub struct TrainingHandler {
    // Training job management
    active_jobs: std::collections::HashMap<String, TrainingJob>,
}

#[derive(Debug)]
struct TrainingJob {
    id: String,
    model_type: String,
    status: TrainingStatus,
    progress: f64,
    started_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
enum TrainingStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

impl TrainingHandler {
    pub fn new() -> Self {
        Self {
            active_jobs: std::collections::HashMap::new(),
        }
    }
    
    pub async fn initialize(&self) -> PrimalResult<()> {
        println!("📚 Initializing training engine");
        Ok(())
    }
    
    pub async fn handle_training(&self, request: &PrimalRequest) -> PrimalResult<serde_json::Value> {
        match request.method.as_str() {
            "train" => self.start_training(request).await,
            "get_training_status" => self.get_training_status(request).await,
            "stop_training" => self.stop_training(request).await,
            _ => Err(PrimalError::not_found("Unknown training method"))
        }
    }
    
    async fn start_training(&self, request: &PrimalRequest) -> PrimalResult<serde_json::Value> {
        let training_config = request.payload.get("config")
            .ok_or_else(|| PrimalError::invalid_request("Missing training config"))?;
        
        let job_id = uuid::Uuid::new_v4().to_string();
        
        // Start training job (async)
        // Your training implementation here
        
        Ok(serde_json::json!({
            "job_id": job_id,
            "status": "started",
            "estimated_duration_hours": 2.5,
            "capabilities_used": ["ai/training", "compute/gpu"]
        }))
    }
    
    async fn get_training_status(&self, request: &PrimalRequest) -> PrimalResult<serde_json::Value> {
        let job_id = request.payload.get("job_id")
            .and_then(|j| j.as_str())
            .ok_or_else(|| PrimalError::invalid_request("Missing job_id"))?;
        
        // Return job status (your implementation)
        Ok(serde_json::json!({
            "job_id": job_id,
            "status": "running",
            "progress": 0.45,
            "eta_minutes": 67
        }))
    }
    
    async fn stop_training(&self, request: &PrimalRequest) -> PrimalResult<serde_json::Value> {
        let job_id = request.payload.get("job_id")
            .and_then(|j| j.as_str())
            .ok_or_else(|| PrimalError::invalid_request("Missing job_id"))?;
        
        // Stop training job
        Ok(serde_json::json!({
            "job_id": job_id,
            "status": "stopped",
            "final_progress": 0.67
        }))
    }
    
    pub async fn is_healthy(&self) -> bool {
        true // Check training resources
    }
    
    pub async fn shutdown(&self) -> PrimalResult<()> {
        println!("⏹️  Shutting down training engine");
        Ok(())
    }
}
```

---

## 🏥 **Step 5: Implement Health Monitoring**

### **Health Module (src/health.rs)**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum ResourceStatus {
    Healthy,
    Degraded,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemResources {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub gpu_usage: Option<f64>,
    pub disk_usage: f64,
}

pub async fn check_system_resources() -> ResourceStatus {
    let resources = get_system_resources().await;
    
    // Define thresholds
    let cpu_critical = 95.0;
    let memory_critical = 90.0;
    let disk_critical = 95.0;
    
    if resources.cpu_usage > cpu_critical 
        || resources.memory_usage > memory_critical 
        || resources.disk_usage > disk_critical {
        return ResourceStatus::Critical;
    }
    
    let cpu_warning = 80.0;
    let memory_warning = 75.0;
    let disk_warning = 80.0;
    
    if resources.cpu_usage > cpu_warning 
        || resources.memory_usage > memory_warning 
        || resources.disk_usage > disk_warning {
        return ResourceStatus::Degraded;
    }
    
    ResourceStatus::Healthy
}

async fn get_system_resources() -> SystemResources {
    // Use sysinfo or similar crate for real implementation
    SystemResources {
        cpu_usage: 45.2,
        memory_usage: 67.8,
        gpu_usage: Some(23.1),
        disk_usage: 34.5,
    }
}
```

---

## ⚙️ **Step 6: Configuration Management**

### **Configuration Module (src/config.rs)**
```rust
use biomeos_primal_sdk::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIPrimalConfig {
    pub inference_config: InferenceConfig,
    pub training_config: TrainingConfig,
    pub resource_limits: ResourceLimits,
    pub endpoints: EndpointConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    pub default_model: String,
    pub max_batch_size: usize,
    pub timeout_ms: u64,
    pub gpu_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub max_concurrent_jobs: usize,
    pub default_epochs: u32,
    pub checkpoint_interval: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u64,
    pub max_cpu_cores: u32,
    pub max_gpu_memory_mb: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub listen_port: u16,
    pub health_check_path: String,
    pub metrics_path: String,
}

impl Default for AIPrimalConfig {
    fn default() -> Self {
        Self {
            inference_config: InferenceConfig {
                default_model: "base-model".to_string(),
                max_batch_size: 32,
                timeout_ms: 30000,
                gpu_enabled: true,
            },
            training_config: TrainingConfig {
                max_concurrent_jobs: 2,
                default_epochs: 10,
                checkpoint_interval: 100,
            },
            resource_limits: ResourceLimits {
                max_memory_mb: 8192,
                max_cpu_cores: 4,
                max_gpu_memory_mb: Some(4096),
            },
            endpoints: EndpointConfig {
                listen_port: 8080,
                health_check_path: "/health".to_string(),
                metrics_path: "/metrics".to_string(),
            },
        }
    }
}

impl AIPrimalConfig {
    pub fn from_primal_config(config: &PrimalConfig) -> PrimalResult<Self> {
        // Extract AI-specific config from the general primal config
        let ai_config_value = config.configuration.get("ai_processing")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        
        let ai_config: AIPrimalConfig = serde_json::from_value(ai_config_value)
            .unwrap_or_default();
        
        Ok(ai_config)
    }
}
```

---

## 🎮 **Step 7: Create a Binary Entry Point**

### **Main Binary (src/main.rs)**
```rust
use my_ai_primal::AIProcessingPrimal;
use biomeos_primal_sdk::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::init();
    
    println!("🚀 Starting AI Processing Primal");
    
    // Create our primal
    let primal = AIProcessingPrimal::new();
    
    // Create configuration
    let mut ai_config = HashMap::new();
    ai_config.insert("inference".to_string(), serde_json::json!({
        "default_model": "gpt-3.5-turbo",
        "gpu_enabled": true,
        "max_batch_size": 16
    }));
    
    let config = PrimalConfig {
        name: "ai-processing-demo".to_string(),
        primal_type: primal.metadata().primal_type.clone(),
        capabilities: primal.capabilities().to_vec(),
        configuration: ai_config,
    };
    
    // Initialize
    primal.initialize(&config).await?;
    
    // Demo: Handle some requests
    demo_requests(&primal).await?;
    
    // Keep running (in real implementation, you'd start an HTTP server)
    println!("✅ AI Processing Primal running. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;
    
    // Graceful shutdown
    primal.shutdown().await?;
    
    Ok(())
}

async fn demo_requests(primal: &AIProcessingPrimal) -> PrimalResult<()> {
    // Demo 1: Inference request
    let inference_request = PrimalRequest::new(
        "inference",
        serde_json::json!({
            "model": "sentiment-analysis",
            "input": "This is a great day for AI!"
        })
    );
    
    let response = primal.handle_request(inference_request).await?;
    println!("🧠 Inference Response: {:?}", response.payload);
    
    // Demo 2: Training request
    let training_request = PrimalRequest::new(
        "train",
        serde_json::json!({
            "config": {
                "model_type": "transformer",
                "dataset": "custom-dataset",
                "epochs": 5
            }
        })
    );
    
    let training_response = primal.handle_request(training_request).await?;
    println!("📚 Training Response: {:?}", training_response.payload);
    
    // Demo 3: Health check
    let health = primal.health_check().await?;
    println!("🏥 Health Status: {:?}", health);
    
    Ok(())
}
```

---

## 🌐 **Step 8: Universal Discovery Integration**

### **Make Your Primal Discoverable**

Your primal needs to be discoverable by any BiomeOS discovery system:

#### **HTTP Server Integration**
```rust
use warp::Filter;

pub async fn start_discovery_server(primal: Arc<AIProcessingPrimal>) -> Result<(), Box<dyn std::error::Error>> {
    // Capability discovery endpoint
    let capabilities = warp::path!("api" / "v1" / "capabilities")
        .and(warp::get())
        .and(with_primal(primal.clone()))
        .and_then(handle_capabilities);
    
    // Health check endpoint
    let health = warp::path!("api" / "v1" / "health") 
        .and(warp::get())
        .and(with_primal(primal.clone()))
        .and_then(handle_health);
    
    // Service information endpoint (for discovery systems)
    let info = warp::path!("api" / "v1" / "info")
        .and(warp::get())
        .and(with_primal(primal.clone()))
        .and_then(handle_service_info);
    
    // Request handling endpoint
    let requests = warp::path!("api" / "v1" / "request")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_primal(primal.clone()))
        .and_then(handle_primal_request);
    
    let routes = capabilities.or(health).or(info).or(requests);
    
    println!("🌐 Starting discovery server on port 8080");
    warp::serve(routes)
        .run(([0, 0, 0, 0], 8080))
        .await;
    
    Ok(())
}

fn with_primal(primal: Arc<AIProcessingPrimal>) -> impl Filter<Extract = (Arc<AIProcessingPrimal>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || primal.clone())
}

async fn handle_capabilities(primal: Arc<AIProcessingPrimal>) -> Result<impl warp::Reply, warp::Rejection> {
    let caps: Vec<_> = primal.capabilities().iter()
        .map(|cap| serde_json::json!({
            "domain": cap.domain,
            "name": cap.name,
            "version": cap.version,
            "parameters": cap.parameters
        }))
        .collect();
    
    Ok(warp::reply::json(&serde_json::json!({
        "capabilities": caps,
        "primal_type": primal.metadata().primal_type,
        "version": primal.metadata().version
    })))
}

async fn handle_health(primal: Arc<AIProcessingPrimal>) -> Result<impl warp::Reply, warp::Rejection> {
    match primal.health_check().await {
        Ok(health) => Ok(warp::reply::json(&serde_json::json!({
            "status": health,
            "timestamp": chrono::Utc::now()
        }))),
        Err(_) => Ok(warp::reply::json(&serde_json::json!({
            "status": "unhealthy",
            "timestamp": chrono::Utc::now()
        })))
    }
}

async fn handle_service_info(primal: Arc<AIProcessingPrimal>) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&serde_json::json!({
        "name": primal.metadata().name,
        "primal_type": primal.metadata().primal_type,
        "capabilities": primal.capabilities(),
        "version": primal.metadata().version,
        "description": primal.metadata().description,
        "endpoints": {
            "capabilities": "/api/v1/capabilities",
            "health": "/api/v1/health",
            "request": "/api/v1/request"
        }
    })))
}

async fn handle_primal_request(
    request: PrimalRequest,
    primal: Arc<AIProcessingPrimal>
) -> Result<impl warp::Reply, warp::Rejection> {
    match primal.handle_request(request).await {
        Ok(response) => Ok(warp::reply::json(&response)),
        Err(error) => {
            eprintln!("Request handling error: {:?}", error);
            Ok(warp::reply::json(&serde_json::json!({
                "error": {
                    "code": error.code,
                    "message": error.message
                }
            })))
        }
    }
}
```

---

## 🧪 **Step 9: Testing Your Primal**

### **Unit Tests**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_primal_initialization() {
        let primal = AIProcessingPrimal::new();
        let config = create_test_config();
        
        let result = primal.initialize(&config).await;
        assert!(result.is_ok());
        
        let health = primal.health_check().await.unwrap();
        assert!(health.is_healthy());
    }
    
    #[tokio::test]
    async fn test_inference_capability() {
        let primal = AIProcessingPrimal::new();
        primal.initialize(&create_test_config()).await.unwrap();
        
        let request = PrimalRequest::new(
            "inference",
            serde_json::json!({
                "model": "test-model",
                "input": "test input"
            })
        );
        
        let response = primal.handle_request(request).await.unwrap();
        assert_eq!(response.status, ResponseStatus::Success);
    }
    
    #[tokio::test]
    async fn test_capability_discovery() {
        let primal = AIProcessingPrimal::new();
        
        let capabilities = primal.capabilities();
        assert!(capabilities.len() > 0);
        
        // Check for expected capabilities
        assert!(capabilities.iter().any(|c| c.matches_name("inference")));
        assert!(capabilities.iter().any(|c| c.matches_name("training")));
    }
    
    fn create_test_config() -> PrimalConfig {
        PrimalConfig {
            name: "test-ai-primal".to_string(),
            primal_type: PrimalType::new("ai", "test-engine", "1.0.0"),
            capabilities: vec![],
            configuration: std::collections::HashMap::new(),
        }
    }
}
```

### **Integration Tests**
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use reqwest;
    
    #[tokio::test]
    async fn test_discovery_compatibility() {
        // Start your primal server
        let primal = Arc::new(AIProcessingPrimal::new());
        tokio::spawn(start_discovery_server(primal.clone()));
        
        // Wait for server to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Test discovery endpoint
        let client = reqwest::Client::new();
        let response = client
            .get("http://localhost:8080/api/v1/capabilities")
            .send()
            .await
            .unwrap();
        
        assert!(response.status().is_success());
        
        let capabilities: serde_json::Value = response.json().await.unwrap();
        assert!(capabilities.get("capabilities").is_some());
    }
    
    #[tokio::test] 
    async fn test_universal_discovery() {
        // Test that your primal can be discovered by BiomeOS discovery systems
        use biomeos_core::{UniversalBiomeOSManager, BiomeOSConfig};
        
        let config = BiomeOSConfig::default();
        let manager = UniversalBiomeOSManager::new(config);
        
        // Test capability-based discovery
        let ai_services = manager.discover_by_capability(
            "http://localhost:8080",
            &[PrimalCapability::new("ai", "inference", "v2")]
        ).await.unwrap();
        
        assert!(ai_services.len() > 0);
        assert_eq!(ai_services[0].primal_id, "ai-processing-primal");
    }
}
```

---

## 🚀 **Step 10: Deployment & Registration**

### **Docker Deployment**
```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/my-ai-primal /usr/local/bin/

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health || exit 1

EXPOSE 8080

CMD ["my-ai-primal"]
```

### **Kubernetes Deployment**
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ai-processing-primal
  labels:
    app: ai-processing-primal
    biome.os/primal: "true"
    biome.os/category: "ai"
spec:
  replicas: 2
  selector:
    matchLabels:
      app: ai-processing-primal
  template:
    metadata:
      labels:
        app: ai-processing-primal
        biome.os/primal: "true"
        biome.os/category: "ai"
      annotations:
        biome.os/capabilities: "ai/inference,ai/training,compute/gpu"
    spec:
      containers:
      - name: ai-primal
        image: my-ai-primal:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "4Gi"
            cpu: "2"
        livenessProbe:
          httpGet:
            path: /api/v1/health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
---
apiVersion: v1
kind: Service
metadata:
  name: ai-processing-primal-service
  labels:
    biome.os/discovery: "true"
spec:
  selector:
    app: ai-processing-primal
  ports:
  - port: 8080
    targetPort: 8080
  type: ClusterIP
```

### **Service Registration**
```rust
// Automatic registration with any discovery system
pub async fn register_with_ecosystem() -> Result<(), Box<dyn std::error::Error>> {
    let primal = AIProcessingPrimal::new();
    
    // Try multiple discovery mechanisms
    let discovery_endpoints = vec![
        "http://songbird:8080",     // Traditional Songbird
        "http://discovery:9000",    // Registry-based discovery
        "http://service-mesh:7000", // Service mesh discovery
    ];
    
    for endpoint in discovery_endpoints {
        match register_with_discovery(&primal, endpoint).await {
            Ok(_) => println!("✅ Registered with discovery at {}", endpoint),
            Err(e) => println!("⚠️  Failed to register with {}: {}", endpoint, e),
        }
    }
    
    Ok(())
}

async fn register_with_discovery(
    primal: &AIProcessingPrimal, 
    discovery_endpoint: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let registration = serde_json::json!({
        "name": primal.metadata().name,
        "primal_type": primal.metadata().primal_type,
        "capabilities": primal.capabilities(),
        "endpoint": "http://ai-processing-primal:8080",
        "health_endpoint": "http://ai-processing-primal:8080/api/v1/health",
        "version": primal.metadata().version
    });
    
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/api/v1/register", discovery_endpoint))
        .json(&registration)
        .send()
        .await?;
    
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("Registration failed with status: {}", response.status()).into())
    }
}
```

---

## 🌟 **Step 11: Best Practices & Patterns**

### **✅ Universal Design Principles**

#### **1. Capability-First Architecture**
```rust
// ✅ GOOD: Design around capabilities
impl MyPrimal {
    fn supports_capability(&self, capability: &PrimalCapability) -> bool {
        self.capabilities().iter().any(|c| 
            c.domain == capability.domain && c.name == capability.name
        )
    }
    
    async fn handle_by_capability(&self, request: &PrimalRequest) -> PrimalResult<PrimalResponse> {
        // Route based on required capabilities, not hardcoded methods
        for required_cap in &request.required_capabilities {
            if self.supports_capability(required_cap) {
                return self.execute_capability(required_cap, request).await;
            }
        }
        Err(PrimalError::not_found("No matching capabilities"))
    }
}

// ❌ BAD: Hardcoded service assumptions
impl MyPrimal {
    async fn call_songbird(&self) -> Result<()> {
        // Don't assume Songbird exists!
    }
}
```

#### **2. Dynamic Service Discovery**
```rust
// ✅ GOOD: Find services by capability
async fn find_compute_service() -> Result<String, Box<dyn std::error::Error>> {
    let manager = UniversalBiomeOSManager::new(BiomeOSConfig::default());
    
    let compute_services = manager.discover_by_capability(
        "http://discovery:8080",
        &[PrimalCapability::code_execution()]
    ).await?;
    
    let healthy_service = compute_services.into_iter()
        .filter(|s| s.health.is_healthy())
        .next()
        .ok_or("No healthy compute services available")?;
    
    Ok(healthy_service.endpoint)
}

// ❌ BAD: Hardcoded service endpoints
async fn call_toadstool() -> Result<()> {
    // Don't hardcode "toadstool" endpoints!
    let response = reqwest::get("http://toadstool:8080/api/execute").await?;
}
```

#### **3. Graceful Capability Degradation**
```rust
impl MyPrimal {
    async fn process_with_fallback(&self, request: &PrimalRequest) -> PrimalResult<PrimalResponse> {
        // Try GPU acceleration first
        if let Some(gpu_service) = self.find_gpu_service().await {
            match gpu_service.process(request).await {
                Ok(result) => return Ok(result),
                Err(_) => println!("GPU processing failed, falling back to CPU"),
            }
        }
        
        // Fallback to CPU processing
        self.process_cpu_only(request).await
    }
    
    async fn find_gpu_service(&self) -> Option<ServiceClient> {
        // Look for services with GPU capabilities
        let services = self.discover_by_capability(&[
            PrimalCapability::custom("gpu", "CUDA support")
        ]).await.ok()?;
        
        services.into_iter().next().map(ServiceClient::from)
    }
}
```

### **✅ Performance & Reliability**

#### **1. Request Batching**
```rust
impl InferenceHandler {
    async fn handle_batch_inference(&self, requests: Vec<PrimalRequest>) -> Vec<PrimalResult<PrimalResponse>> {
        // Group requests by model
        let mut by_model: HashMap<String, Vec<_>> = HashMap::new();
        for req in requests {
            let model = req.payload.get("model")
                .and_then(|m| m.as_str())
                .unwrap_or("default");
            by_model.entry(model.to_string()).or_default().push(req);
        }
        
        // Process each model's batch
        let mut results = Vec::new();
        for (model, model_requests) in by_model {
            let batch_result = self.process_model_batch(&model, model_requests).await;
            results.extend(batch_result);
        }
        
        results
    }
}
```

#### **2. Resource Management**
```rust
impl MyPrimal {
    async fn check_resource_limits(&self, request: &PrimalRequest) -> PrimalResult<()> {
        let current_memory = self.get_memory_usage().await;
        let current_cpu = self.get_cpu_usage().await;
        
        let estimated_memory = self.estimate_request_memory(request);
        let estimated_cpu = self.estimate_request_cpu(request);
        
        if current_memory + estimated_memory > self.max_memory {
            return Err(PrimalError::new("RESOURCE_LIMIT", "Memory limit would be exceeded"));
        }
        
        if current_cpu + estimated_cpu > self.max_cpu {
            return Err(PrimalError::new("RESOURCE_LIMIT", "CPU limit would be exceeded"));
        }
        
        Ok(())
    }
}
```

#### **3. Health Monitoring**
```rust
impl MyPrimal {
    async fn comprehensive_health_check(&self) -> PrimalResult<PrimalHealth> {
        let mut health_score = 100.0;
        
        // Check system resources
        let resources = self.get_system_resources().await;
        if resources.memory_usage > 90.0 { health_score -= 30.0; }
        if resources.cpu_usage > 95.0 { health_score -= 40.0; }
        
        // Check capability handlers
        for handler in &self.capability_handlers {
            if !handler.is_healthy().await {
                health_score -= 20.0;
            }
        }
        
        // Check external dependencies
        let dependency_health = self.check_dependencies().await;
        health_score -= (100.0 - dependency_health) * 0.3;
        
        match health_score {
            90.0..=100.0 => Ok(PrimalHealth::healthy()),
            60.0..90.0 => Ok(PrimalHealth::degraded()),
            _ => Ok(PrimalHealth::unhealthy()),
        }
    }
}
```

---

## 🎉 **Congratulations!**

You've successfully created a **universal, capability-based primal** that works with any BiomeOS ecosystem!

### **🌟 What You've Achieved**

✅ **True Universality**: Your primal works with monolithic, microservice, or hybrid ecosystems  
✅ **Future-Proof Design**: Adapts to ecosystem evolution without code changes  
✅ **Capability-Based Discovery**: Found by what it can do, not what it's called  
✅ **Production-Ready**: Health monitoring, resource management, and graceful error handling  
✅ **Community Compatible**: Works with community extensions and custom architectures  

### **🚀 Next Steps**

1. **Deploy Your Primal**: Use the provided Docker and Kubernetes configurations
2. **Register with Discovery**: Use the registration patterns to join any ecosystem
3. **Monitor & Optimize**: Use the health monitoring to optimize performance  
4. **Contribute Back**: Share your primal patterns with the BiomeOS community
5. **Scale Up**: Add more capabilities and expand your primal's functionality

### **🌐 Your Impact**

By following this guide, you've created a primal that embodies the **core BiomeOS philosophy**:

> **"Build universal, capability-driven services that work in any ecosystem architecture"**

Your primal will work whether the ecosystem has one monolithic Songbird, ten specialized microservices, or completely new community-built alternatives. **That's the power of universal, capability-based design!** 🌟

---

## 📚 **Additional Resources**

- [BiomeOS Primal SDK API Documentation](../api/biomeos-primal-sdk.md)
- [Discovery & Health Monitoring API](../api/discovery-and-health-monitoring.md)  
- [CLI Tools Documentation](../../CLI_TOOLS_README.md)
- [Ecosystem Participation Guide](ecosystem-participation-guide.md)
- [Architecture Decision Records](../adrs/)

**Ready to build the future of universal, capability-based systems? Let's go!** 🚀 