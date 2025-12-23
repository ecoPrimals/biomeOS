# Toadstool Manifest Parsing & Configuration Capabilities

**Status:** Information Gathering | **Source:** toadstool codebase analysis | **Date:** January 2025

---

## Current Manifest Structure

Toadstool already has a sophisticated `BiomeManifest` structure defined in `toadstool/crates/cli/src/lib.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeManifest {
    /// Biome metadata
    pub metadata: BiomeMetadata,
    /// Primal configurations
    pub primals: HashMap<String, PrimalConfig>,
    /// Service definitions
    pub services: HashMap<String, ServiceConfig>,
    /// Resource requirements
    pub resources: BiomeResources,
    /// Security policies
    pub security: BiomeSecurity,
    /// Network configuration
    pub networking: BiomeNetworking,
    /// Storage configuration
    pub storage: BiomeStorage,
}
```

## Workload Source Support

Toadstool supports multiple workload sources:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WorkloadSource {
    /// OCI container registry
    Container { registry: String, image: String, tag: String, digest: Option<String> },
    /// WebAssembly module
    Wasm { source: String, checksum: String, wasi_config: Option<HashMap<String, serde_yaml::Value>> },
    /// Git repository
    Git { repository: String, branch: Option<String>, commit: Option<String>, path: Option<String> },
    /// IPFS content
    Ipfs { hash: String, gateway: Option<String> },
    /// Local file path
    Local { path: PathBuf },
}
```

## Runtime Types Supported

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RuntimeType {
    /// Container runtime (Docker, Containerd, Podman)
    Container,
    /// WebAssembly runtime (Wasmtime, Wasmer)
    Wasm,
    /// Native process runtime
    Native,
    /// GPU compute runtime
    Gpu,
    /// Custom runtime extension
    Custom(String),
}
```

## Configuration Management

### Hierarchical Configuration Loading
From `toadstool/crates/core/config/src/lib.rs`:

```rust
impl ConfigBuilder {
    pub fn build(self) -> ConfigResult<ToadStoolConfig> {
        // Loads configuration with precedence:
        // 1. Default values
        // 2. /etc/toadstool/config.toml
        // 3. ~/.config/toadstool/config.toml
        // 4. ./toadstool.toml
        // 5. Environment variables (TOADSTOOL_*)
        // 6. Command line arguments
    }
}
```

### Auto-Configuration
From `toadstool/crates/auto_config/src/lib.rs`:

```rust
pub struct IntelligentAutoConfig {
    /// Hardware detection and optimization
    pub hardware_detector: hardware::HardwareDetector,
    /// Ecosystem service discovery
    pub ecosystem_discoverer: ecosystem::EcosystemDiscoverer,
    /// Natural language configuration
    pub nlp_processor: natural_language::NaturalLanguageProcessor,
}
```

## Execution Engine

### Biome Executor
From `toadstool/crates/cli/src/executor.rs`:

```rust
pub struct BiomeExecutor {
    /// ToadStool runtime orchestrator
    runtime: Arc<RuntimeOrchestrator>,
    /// Distributed coordinator for ecosystem integration
    distributed: Arc<DistributedCoordinator>,
    /// Running biomes registry
    biomes: Arc<tokio::sync::RwLock<HashMap<String, RunningBiome>>>,
    /// Configuration
    config: ToadStoolConfig,
}
```

### Execution Flow
1. **BearDog Security First**: If `manifest.security.beardog_required`, BearDog starts first
2. **Primal Orchestration**: Other Primals started in dependency order
3. **Service Management**: Services started after Primals are ready
4. **Health Monitoring**: Continuous health checking and process management

## Ecosystem Integration

### Songbird Integration
From `toadstool/crates/distributed/src/songbird_integration.rs`:

```rust
pub struct NodeRegistration {
    pub node_id: NodeId,
    pub node_type: NodeType,
    pub capabilities: NodeCapabilities,
    pub endpoints: Vec<String>,
    pub protocols: Vec<String>,
    pub metadata: NodeMetadata,
}
```

### Service Discovery
Toadstool includes ecosystem auto-discovery:

```rust
async fn test_songbird_endpoint(&self, endpoint: &str) -> ToadStoolResult<ServiceInfo> {
    // Tests Songbird endpoints for service discovery
    // Supports both /api/v1/info and /health endpoints
}
```

## Universal Scheduler

### Job Types
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UniversalJobType {
    /// Local execution
    Local,
    /// Remote ToadStool execution
    RemoteToadStool { endpoint: String },
    /// Ecosystem tool execution
    EcosystemTool { tool_name: String, endpoint: String },
    /// Recursive ToadStool hosting
    RecursiveHosting { toadstool_config: ToadStoolHostingConfig },
    /// CPU-intensive computational work
    ComputeIntensive,
    /// Data processing and analytics
    DataProcessing,
    /// Machine learning and AI workloads
    MachineLearning,
    /// Scientific simulations
    Simulation,
    /// Container-based execution
    Container,
    /// WebAssembly execution
    WASM,
    /// GPU-accelerated execution
    GPU,
}
```

## Template System

Toadstool includes sophisticated biome templates:

```rust
pub enum BiomeTemplate {
    Basic,
    Science,
    AiResearch,
    Quantum,
    Genomics,
    Vision,
    Distributed,
    Sovereign,
    Development,
    Custom(CustomTemplateSpec),
}
```

## Validation & Health Checking

### Manifest Validation
```rust
pub fn validate_manifest(manifest: &BiomeManifest) -> Result<Vec<String>> {
    // Validates:
    // - Required Primals (BearDog if security.beardog_required)
    // - Service dependencies
    // - Resource limits
    // - Returns warnings for optimization
}
```

## Integration Points Needed for biomeOS

### 1. Complete biome.yaml Schema
- ✅ **Already Implemented**: Sophisticated manifest structure
- ✅ **Already Implemented**: Multiple workload source types
- ✅ **Already Implemented**: Primal configuration support
- 🔄 **Needs Enhancement**: Align with biomeOS API inoculum

### 2. Inter-Primal Communication
- ✅ **Already Implemented**: Songbird integration framework
- ✅ **Already Implemented**: Service discovery mechanisms
- 🔄 **Needs Enhancement**: Implement "Songbird Pattern" consistently

### 3. Security Integration
- ✅ **Already Implemented**: BearDog-first startup sequence
- ✅ **Already Implemented**: Security context enforcement
- 🔄 **Needs Enhancement**: Cross-Primal authentication

### 4. Storage Integration
- ✅ **Already Implemented**: Volume mounting support
- ✅ **Already Implemented**: Storage configuration in manifest
- 🔄 **Needs Enhancement**: NestGate-specific volume provisioning

### 5. AI Agent Support
- ✅ **Already Implemented**: WASM runtime support
- ✅ **Already Implemented**: Plugin extension system
- 🔄 **Needs Enhancement**: Squirrel MCP integration

## Conclusion

**Toadstool is remarkably advanced** and already implements most of the core functionality needed for biomeOS:

- **Manifest parsing**: ✅ Complete and sophisticated
- **Multi-runtime support**: ✅ Container, WASM, Native, GPU
- **Ecosystem integration**: ✅ Songbird discovery framework
- **Security-first approach**: ✅ BearDog integration built-in
- **Auto-configuration**: ✅ Intelligent hardware detection
- **Template system**: ✅ Pre-built biome templates

**Next Steps:**
1. Align manifest schema with biomeOS API inoculum
2. Implement consistent Songbird Pattern communication
3. Add NestGate-specific storage provisioning
4. Integrate Squirrel MCP protocol support
5. Test end-to-end biome bootstrapping 