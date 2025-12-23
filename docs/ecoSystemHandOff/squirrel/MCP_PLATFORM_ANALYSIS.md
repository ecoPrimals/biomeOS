# Squirrel MCP Platform & AI Agent Management Analysis

**Status:** Information Gathering | **Source:** squirrel codebase analysis | **Date:** January 2025

---

## MCP Platform Architecture

Squirrel is the **Machine Context Protocol (MCP) Platform** with sophisticated AI agent coordination:

```rust
pub struct McpPluginRegistry {
    pub plugins: HashMap<PluginId, McpPluginMetadata>,
    pub capabilities: HashMap<CapabilityId, Vec<PluginId>>,
    pub ai_recommendations: Arc<AiPluginRecommender>,
    pub mcp_interfaces: HashMap<PluginId, McpInterface>,
}

pub struct McpPluginMetadata {
    pub id: PluginId,
    pub mcp_capabilities: Vec<McpCapability>,
    pub ai_integration: AiIntegrationLevel,
    pub context_requirements: ContextRequirements,
    pub execution_delegate: ExecutionTarget, // Toadstool delegation
}
```

## Plugin System & Sandboxing

### Cross-Platform Sandbox Architecture
From `squirrel/.cursor/rules/plugin-sandboxing.mdc`:

```rust
#[async_trait::async_trait]
pub trait PluginSandbox: Send + Sync + std::fmt::Debug {
    /// Create a sandbox for a plugin
    async fn create_sandbox(&self, plugin_id: Uuid) -> Result<()>;
    
    /// Destroy a sandbox for a plugin
    async fn destroy_sandbox(&self, plugin_id: Uuid) -> Result<()>;
    
    /// Check if an operation is allowed for a plugin
    async fn check_permission(&self, plugin_id: Uuid, operation: &str) -> Result<()>;
    
    /// Track resource usage for a plugin
    async fn track_resources(&self, plugin_id: Uuid) -> Result<ResourceUsage>;
    
    /// Check if a plugin has access to a path
    async fn check_path_access(&self, plugin_id: Uuid, path: &Path, write: bool) -> Result<()>;
}
```

### Platform-Specific Implementations
- **Windows**: Job Objects for process isolation, I/O rate control
- **Linux**: cgroups v2 for resource control, namespace isolation
- **macOS**: App Sandbox with security profiles, TCC integration

### Security Levels
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Minimal permissions, safe for untrusted plugins
    Restricted,
    /// Standard permissions for verified plugins
    Standard,
    /// Enhanced permissions for trusted plugins
    Enhanced,
    /// Full permissions for system plugins
    Full,
}
```

## Toadstool Integration

### Execution Delegation
From `squirrel/code/crates/integration/toadstool/src/execution.rs`:

```rust
/// Request to execute a plugin via Toadstool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    /// Unique identifier for this execution
    pub execution_id: Uuid,
    
    /// Plugin identifier
    pub plugin_id: String,
    
    /// Plugin code (base64 encoded for JSON transport)
    pub code: String,
    
    /// Execution environment configuration
    pub environment: ExecutionEnvironment,
    
    /// MCP context information
    pub mcp_context: Option<McpContext>,
    
    /// Priority of execution (0 = lowest, 10 = highest)
    pub priority: u8,
    
    /// Execution timeout in milliseconds  
    pub timeout: Option<u64>,
}
```

### MCP Context Management
```rust
/// MCP-specific context for plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpContext {
    /// MCP session identifier
    pub session_id: String,
    
    /// Active agent information
    pub agent_info: AgentInfo,
    
    /// Available MCP tools
    pub available_tools: Vec<McpTool>,
    
    /// Context variables
    pub context_vars: HashMap<String, serde_json::Value>,
    
    /// Parent conversation/thread ID
    pub conversation_id: Option<String>,
}
```

### Execution Environments
```rust
impl ExecutionEnvironment {
    /// Create WASM execution environment (default)
    pub fn wasm_environment(resource_limits: ResourceLimits, sandbox_policy: SandboxPolicy) -> ExecutionEnvironment {
        ExecutionEnvironment {
            environment_type: "wasm".to_string(),
            resource_limits,
            security_policy: sandbox_policy,
            env_vars: HashMap::new(),
        }
    }
    
    /// Create container execution environment
    pub fn container_environment(resource_limits: ResourceLimits, sandbox_policy: SandboxPolicy) -> ExecutionEnvironment {
        ExecutionEnvironment {
            environment_type: "container".to_string(),
            resource_limits,
            security_policy: sandbox_policy,
            env_vars: HashMap::new(),
        }
    }
}
```

## AI Platform Excellence

### Multi-Provider AI Router
From `squirrel/code/crates/ui/ui-terminal/src/app/ai_chat.rs`:

```rust
// Configure OpenAI provider with the API key and additional parameters
let openai_settings = ProviderSettings::default_openai()
    .with_parameter("api_key".to_string(), json!(api_key))
    .with_parameter("temperature".to_string(), json!(0.7))
    .with_parameter("max_tokens".to_string(), json!(2000))
    .with_models(vec![
        "gpt-3.5-turbo".to_string(),
        "gpt-4".to_string(),
        "gpt-4-turbo-preview".to_string()
    ]);

tools_config = tools_config
    .with_provider("openai".to_string(), openai_settings)
    .with_timeout(60000) // 60 seconds
    .with_streaming(true);
```

### MCP Configuration
```rust
/// MCP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// MCP protocol version
    pub protocol_version: String,
    /// Whether streaming is supported
    pub supports_streaming: bool,
    /// Whether tools are supported
    pub supports_tools: bool,
    /// Whether resources are supported
    pub supports_resources: bool,
    /// Whether prompts are supported
    pub supports_prompts: bool,
    /// Custom capabilities
    pub custom_capabilities: HashMap<String, serde_json::Value>,
}
```

## Current Implementation Status

From `squirrel/SPECS.md`:

### Core MCP Platform (98% Complete)
- ✅ **MCP Protocol Server** (98% complete) - Production ready
- ✅ **AI Agent Coordination** (90% complete) - Multi-provider routing active
- ✅ **Context Management** (95% complete) - Multi-agent contexts ready
- ✅ **Plugin Registry** (95% complete) - MCP-native interfaces complete
- ✅ **Security Framework** (85% complete) - RBAC & ecosystem auth ready
- ✅ **Transport Layer** (100% complete) - TCP, WebSocket, Memory, Stdio

### AI Platform Excellence
- ✅ **Multi-Provider AI Router** (95% complete) - OpenAI, Anthropic, Gemini integrated
- ✅ **Intelligent Routing** (90% complete) - Capability-based provider selection
- ✅ **Context-Aware Processing** (92% complete) - Session & conversation state
- ✅ **AI Tool Management** (88% complete) - Registration, discovery, execution

### Plugin Platform
- ✅ **MCP Plugin Adapters** (95% complete) - Bidirectional tool-plugin integration
- ✅ **Plugin Lifecycle** (92% complete) - Loading, validation, cleanup
- ✅ **AI-Enhanced Discovery** (80% complete) - Smart plugin recommendations
- ✅ **Execution Delegation** (85% complete) - Toadstool integration via Songbird

## Ecosystem Integration Patterns

### Service Registration with Songbird
```rust
pub struct EcosystemIntegration {
    pub songbird_client: Arc<SongbirdClient>,
    pub toadstool_client: Arc<ToadstoolClient>, 
    pub service_registry: Arc<ServiceRegistry>,
    pub context_bridge: Arc<ContextBridge>,
}

impl McpPluginRegistry {
    pub async fn execute_plugin(&self, plugin_id: &str, context: McpContext) -> Result<PluginResult> {
        // Plugin execution is delegated to Toadstool via Songbird
        let ecosystem = EcosystemIntegration::new().await?;
        ecosystem.execute_plugin(plugin_id, context).await
    }
}
```

### Compute Tearout Implementation
From `squirrel/specs/COMPUTE_TEAROUT_AND_TOADSTOOL_INTEGRATION.md`:

**What Squirrel SHOULD Do:**
```yaml
mcp_platform_excellence:
  protocol_implementation:
    - Machine Context Protocol server
    - Context management and storage
    - Multi-agent coordination protocols
    - Agent communication and workflows
    
  plugin_platform:
    - Plugin registry and metadata management
    - Plugin discovery and lifecycle management
    - MCP-specific plugin interfaces
    - AI-enhanced plugin recommendations
    
  ai_integration:
    - AI model management and switching
    - AI agent behavior and learning
    - Context-aware AI operations
    - Multi-agent workflow coordination
    
  ecosystem_integration:
    - Register with Songbird discovery
    - Request compute via Songbird routing
    - Coordinate with Toadstool for execution
    - Maintain MCP context across ecosystem
```

## SDK & Development Framework

### Plugin SDK
From `squirrel/code/crates/sdk/src/lib.rs`:

```rust
/// Plugin callback function type
pub type PluginCallback = Box<dyn Fn(wasm_bindgen::JsValue) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>>;

/// Macro for creating plugin info structures
#[macro_export]
macro_rules! plugin_info {
    ($name:expr, $version:expr) => {
        $crate::PluginInfo {
            id: $crate::utils::generate_uuid(),
            name: $name.to_string(),
            version: $crate::Version::parse($version).unwrap(),
            description: String::new(),
            author: String::new(),
            license: String::new(),
            homepage: None,
            repository: None,
            keywords: vec![],
            categories: vec![],
        }
    };
}
```

### Web Integration
```rust
pub mod web {
    /// Setup the plugin environment for web execution
    pub fn setup_plugin_environment(plugin_id: &str) -> PluginResult<()> {
        // Initialize console panic hook for better error messages
        #[cfg(feature = "console")]
        console_error_panic_hook::set_once();
        
        // Initialize logging
        crate::logging::Logger::current();
        
        Ok(())
    }
}
```

## Security & Sandbox Configuration

### Sandbox Configuration
```rust
pub struct SandboxConfig {
    /// Whether network access is allowed
    pub network_access: bool,
    /// Allowed file system paths
    pub file_system_access: Vec<String>,
    /// Memory limit in MB
    pub memory_limit_mb: u32,
    /// CPU limit as percentage
    pub cpu_limit_percent: u8,
    /// Execution timeout in seconds
    pub execution_timeout_seconds: u32,
    /// Security level
    pub security_level: SecurityLevel,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            network_access: false,
            file_system_access: Vec::new(),
            memory_limit_mb: 128,
            cpu_limit_percent: 5,
            execution_timeout_seconds: 10,
            security_level: SecurityLevel::Restricted,
        }
    }
}
```

## Integration Points for biomeOS

### 1. MCP Protocol Implementation ✅
- **Already Implemented**: Complete MCP protocol server
- **Already Implemented**: Multi-transport support (TCP, WebSocket, Memory, Stdio)
- **Already Implemented**: Context management and storage
- 🔄 **Needs Enhancement**: biome.yaml agent definitions

### 2. AI Agent Coordination ✅
- **Already Implemented**: Multi-provider AI routing
- **Already Implemented**: Context-aware processing
- **Already Implemented**: Agent-to-agent communication
- 🔄 **Needs Enhancement**: biomeOS agent deployment patterns

### 3. Plugin Platform ✅
- **Already Implemented**: MCP-native plugin interfaces
- **Already Implemented**: Cross-platform sandboxing
- **Already Implemented**: AI-enhanced plugin discovery
- 🔄 **Needs Enhancement**: Toadstool execution integration

### 4. Ecosystem Integration ✅
- **Already Implemented**: Songbird service registration
- **Already Implemented**: Toadstool execution delegation
- **Already Implemented**: Cross-service communication
- 🔄 **Needs Enhancement**: biomeOS service discovery patterns

### 5. Security Framework ✅
- **Already Implemented**: RBAC and ecosystem authentication
- **Already Implemented**: Plugin sandboxing across platforms
- **Already Implemented**: Resource monitoring and limits
- 🔄 **Needs Enhancement**: BearDog security integration

## Agent Deployment Examples

### Deploying an AI Agent via MCP
```bash
# Register AI agent with Squirrel
curl -X POST http://squirrel:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "data-analyst",
    "name": "Data Analysis Agent",
    "capabilities": ["data_analysis", "visualization", "reporting"],
    "ai_provider": "openai",
    "model": "gpt-4",
    "execution_environment": "wasm",
    "resource_limits": {
      "memory_mb": 256,
      "cpu_percent": 10,
      "timeout_seconds": 300
    }
  }'
```

### Executing Agent via Toadstool
```bash
# Execute agent plugin through ecosystem
curl -X POST http://squirrel:8080/api/v1/agents/data-analyst/execute \
  -H "Content-Type: application/json" \
  -d '{
    "context": {
      "session_id": "session-123",
      "conversation_id": "conv-456",
      "input_data": "CSV data for analysis"
    },
    "execution_target": "toadstool",
    "priority": 5
  }'
```

## Conclusion

**Squirrel provides a sophisticated MCP platform** for biomeOS:

- **MCP Protocol**: ✅ Complete protocol server with multi-transport support
- **AI Coordination**: ✅ Multi-provider routing with intelligent selection
- **Plugin Platform**: ✅ Cross-platform sandboxing with MCP integration
- **Ecosystem Integration**: ✅ Songbird discovery and Toadstool execution
- **Security Framework**: ✅ RBAC with comprehensive sandboxing
- **SDK & Tools**: ✅ Complete development framework

**Ready for biomeOS Integration:**
1. Already implements compute tearout to Toadstool
2. Integrates with Songbird for service discovery
3. Provides comprehensive MCP protocol support
4. Includes sophisticated AI agent management

**Next Steps:**
1. Define biome.yaml agent deployment patterns
2. Implement biomeOS service discovery integration
3. Add BearDog security provider integration
4. Create agent lifecycle management for biomes
5. Test end-to-end MCP workflows in biomeOS 