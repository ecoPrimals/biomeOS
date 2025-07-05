# Songbird Service Discovery & API Gateway Architecture

**Status:** Information Gathering | **Source:** songbird codebase analysis | **Date:** January 2025

---

## Core Architecture

Songbird implements a sophisticated orchestrator with pluggable service discovery backends:

```rust
pub struct Orchestrator {
    config: Arc<OrchestratorConfig>,
    registry: Arc<ServiceRegistry>,
    discovery: Arc<dyn ServiceDiscovery>,
    services: Arc<DashMap<String, ServiceInstance>>,
    request_router: Arc<RequestRouter>,
    load_balancer: Arc<dyn LoadBalancer>,
    communication: Arc<dyn CommunicationLayer>,
    protocol_router: Arc<ProtocolRouter>,
    observability: Arc<ObservabilityEngine>,
}
```

## Service Discovery Backends

### Discovery Backend Types
```rust
pub enum DiscoveryBackend {
    Static,
    Songbird(SongbirdDiscoveryConfig),
    Etcd { endpoints: Vec<String>, username: Option<String>, password: Option<String> },
    Kubernetes { namespace: Option<String>, in_cluster: bool, kubeconfig_path: Option<PathBuf> },
}
```

### Songbird Native Discovery
From `songbird/src/discovery/songbird_discovery.rs`:

```rust
pub struct SongbirdDiscovery {
    /// Local node information
    local_node: LocalNode,
    /// Known nodes in the federation
    known_nodes: Arc<RwLock<HashMap<NodeId, NodeInfo>>>,
    /// Services registered on this node
    local_services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
    /// Event broadcasting for service changes
    event_sender: broadcast::Sender<ServiceEvent>,
    /// Configuration
    config: SongbirdDiscoveryConfig,
}
```

## Communication Layer

### Protocol Router
From `songbird/src/communication/protocol_router.rs`:

```rust
pub struct ProtocolRouter {
    http_layer: Arc<HttpCommunication>,
    websocket_layer: Arc<WebSocketCommunication>,
    in_memory_layer: Arc<InMemoryCommunication>,
    protocol_preferences: Arc<parking_lot::RwLock<HashMap<String, CommunicationProtocol>>>,
}
```

### Auto-Protocol Detection
```rust
fn detect_protocol_from_service(&self, service_info: &ServiceInfo) -> CommunicationProtocol {
    // Check service type
    if service_info.service_type == "test" || service_info.service_type == "mock" {
        return CommunicationProtocol::InMemory;
    }
    
    // Check capabilities
    if service_info.capabilities.contains(&"websocket".to_string()) {
        return CommunicationProtocol::WebSocket;
    }
    
    // Check endpoints for protocol hints
    // Check tags for protocol preferences
    // Default to HTTP
}
```

## Load Balancing

### Load Balancer Interface
```rust
#[async_trait]
pub trait LoadBalancer: Send + Sync {
    async fn select_service(&self) -> Option<String>;
    async fn update_service_health(&self, service_id: &str, is_healthy: bool) -> Result<()>;
    async fn get_stats(&self) -> Result<LoadBalancerStats>;
}
```

### Load Balancing Strategies
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
    Random,
    HealthBased,
    Custom(String),
}
```

## Service Registration

### Service Information Structure
```rust
pub struct ServiceInfo {
    pub id: String,
    pub name: String,
    pub service_type: String,
    pub version: String,
    pub endpoints: Vec<ServiceEndpoint>,
    pub capabilities: Vec<String>,
    pub health_status: ServiceHealthStatus,
    pub metadata: HashMap<String, String>,
    pub tags: HashMap<String, String>,
    pub last_heartbeat: DateTime<Utc>,
}
```

### Registration Process
```rust
impl Orchestrator {
    pub async fn register_service(&self, registration: ServiceRegistration) -> Result<ServiceInfo> {
        // 1. Validate service registration
        // 2. Register with discovery backend
        // 3. Add to local registry
        // 4. Configure protocol routing
        // 5. Start health monitoring
        // 6. Broadcast service event
    }
}
```

## API Gateway Features

### HTTP API Endpoints
From `songbird/docs/user/API_REFERENCE.md`:

- **Service Management**:
  - `POST /services/register` - Register service
  - `GET /services` - List services
  - `GET /services/:id` - Get service details
  - `DELETE /services/:id` - Unregister service

- **Communication**:
  - `POST /communication/send` - Send message to service
  - `POST /communication/broadcast` - Broadcast message
  - `GET /communication/stats` - Communication statistics

- **Metrics & Monitoring**:
  - `GET /metrics` - Orchestrator metrics
  - `GET /metrics/prometheus` - Prometheus format
  - `GET /stream/events` - Real-time events (SSE)

### Request Routing
```rust
pub struct RequestRouter {
    load_balancer: Arc<dyn LoadBalancer>,
    communication: Arc<dyn CommunicationLayer>,
}

impl RequestRouter {
    pub async fn route_request(&self, request: ServiceRequest) -> Result<ServiceResponse> {
        // 1. Select target service using load balancer
        // 2. Determine communication protocol
        // 3. Route request through appropriate channel
        // 4. Handle response and errors
    }
}
```

## Federation Support

### Federation Manager
```rust
pub struct FederationManager {
    config: FederationConfig,
    mode: FederationMode,
    status: Arc<RwLock<FederationStatus>>,
}

pub enum FederationMode {
    Standalone,
    Peer,
    Leader,
    Follower,
}
```

### Cross-Cluster Communication
Songbird supports federation between multiple instances for distributed service discovery and load balancing.

## Security Integration

### Authentication Provider
```rust
#[async_trait]
pub trait AuthenticationProvider: Send + Sync {
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken>;
    async fn validate_token(&self, token: &str) -> Result<UserInfo>;
    async fn refresh_token(&self, refresh_token: &str) -> Result<AuthToken>;
}
```

### Security Configuration
```rust
pub struct SecurityConfig {
    pub authentication_required: bool,
    pub authorization_enabled: bool,
    pub rate_limiting: Option<RateLimitConfig>,
    pub cors_config: Option<CorsConfig>,
}
```

## Integration with Other Primals

### NestGate Integration
From `nestgate/code/crates/nestgate-network/src/songbird.rs`:

```rust
pub struct SongbirdIntegration {
    config: SongbirdConfig,
    client: SongbirdClient,
    registration: ServiceRegistration,
    active_connections: Arc<RwLock<HashMap<String, ActiveConnection>>>,
}
```

### Squirrel Integration
From `squirrel/code/crates/integration/web/src/songbird.rs`:

```rust
#[async_trait]
pub trait SongbirdService: Send + Sync {
    async fn register_service(&self, registration: ServiceRegistration) -> Result<ServiceEndpoint>;
    async fn discover_services(&self, service_name: &str) -> Result<Vec<ServiceEndpoint>>;
    async fn update_health(&self, health: HealthStatus) -> Result<()>;
    async fn start_health_reporting(&self, health_provider: Box<dyn HealthProvider>) -> Result<()>;
}
```

## Observability & Monitoring

### Observability Engine
```rust
pub struct ObservabilityEngine {
    metrics_collector: Arc<MetricsCollector>,
    event_processor: Arc<EventProcessor>,
    health_monitor: Arc<HealthMonitor>,
}
```

### Real-time Monitoring
- **Server-Sent Events**: Real-time service events and metrics
- **Prometheus Integration**: Metrics export for monitoring systems
- **Health Checking**: Continuous service health monitoring
- **Performance Metrics**: Request latency, throughput, error rates

## Configuration Management

### Environment-based Configuration
```rust
impl SongbirdConfig {
    fn default() -> Self {
        Self {
            orchestrator_url: std::env::var("SONGBIRD_URL")
                .unwrap_or_else(|_| format!("http://{}:{}", 
                    std::env::var("NESTGATE_SONGBIRD_HOST").unwrap_or_else(|_| "songbird-orchestrator".to_string()),
                    std::env::var("NESTGATE_SONGBIRD_PORT").unwrap_or_else(|_| "8000".to_string())
                )),
        }
    }
}
```

## Integration Points for biomeOS

### 1. Service Registration Standards ✅
- **Already Implemented**: Comprehensive service registration
- **Already Implemented**: Multi-protocol support
- **Already Implemented**: Health monitoring
- 🔄 **Needs Enhancement**: Primal-specific registration patterns

### 2. API Gateway Functionality ✅
- **Already Implemented**: HTTP API gateway
- **Already Implemented**: Request routing and load balancing
- **Already Implemented**: Real-time monitoring
- 🔄 **Needs Enhancement**: biomeOS-specific endpoints

### 3. Service Discovery ✅
- **Already Implemented**: Multiple discovery backends
- **Already Implemented**: Native Songbird discovery
- **Already Implemented**: Federation support
- 🔄 **Needs Enhancement**: biome.yaml service definitions

### 4. Communication Protocols ✅
- **Already Implemented**: HTTP, WebSocket, In-Memory
- **Already Implemented**: Auto-protocol detection
- **Already Implemented**: Protocol routing
- 🔄 **Needs Enhancement**: Primal-specific communication patterns

### 5. Security & Authentication ✅
- **Already Implemented**: Authentication providers
- **Already Implemented**: Security configuration
- **Already Implemented**: Rate limiting and CORS
- 🔄 **Needs Enhancement**: BearDog integration

## Conclusion

**Songbird is exceptionally well-architected** for the biomeOS ecosystem:

- **Service Discovery**: ✅ Multiple backends with native Songbird discovery
- **API Gateway**: ✅ Full HTTP API with real-time monitoring
- **Load Balancing**: ✅ Multiple strategies with health-based routing
- **Communication**: ✅ Multi-protocol support with auto-detection
- **Federation**: ✅ Multi-cluster support for distributed biomes
- **Security**: ✅ Authentication and authorization framework
- **Observability**: ✅ Comprehensive monitoring and metrics

**Ready for biomeOS Integration:**
1. Already integrates with NestGate and Squirrel
2. Supports the "Songbird Pattern" architecture
3. Provides all necessary APIs for Primal coordination
4. Includes federation for multi-biome networks

**Next Steps:**
1. Define biomeOS-specific service registration patterns
2. Implement biome.yaml service discovery integration
3. Add BearDog authentication provider
4. Create biomeOS dashboard endpoints 