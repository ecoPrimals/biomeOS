# 🍄 ToadStool Daemon Mode - Architecture Proposal

**Date**: January 4, 2026  
**Context**: Multi-primal interaction testing insights  
**Principle**: Like the fungus, adapt form to environment

---

## 🎯 Core Insight

**ToadStool should have two modes**:

1. **CLI Mode** (Current): Direct invocation for specific projects
   ```bash
   cd my-ml-project && toadstool run biome.yaml
   ```

2. **Daemon Mode** (Proposed): Workload Execution Service for ecosystem
   ```bash
   toadstool daemon --register-with-biomeos
   ```

**Like the fungus it's named after**: Same organism, different fruiting body based on environment!

---

## 🚀 What Would ToadStool Daemon DO?

### 1. Workload Execution Service

**Purpose**: Accept and execute workload requests from other primals or remote nodes.

**API Endpoints**:
```rust
// Submit workload for execution
POST /api/v1/workload/submit
{
  "biome_yaml": "...",        // Or inline workload definition
  "context": {                // Execution context
    "priority": "normal",
    "resources": {"cpu": "2", "mem": "4Gi"},
    "requester": "beardog-abc123"
  }
}
→ { "workload_id": "wl-123", "status": "queued", "estimated_start": "2s" }

// Query workload status
GET /api/v1/workload/{id}
→ {
    "id": "wl-123",
    "status": "running",  // queued, running, completed, failed
    "progress": 0.75,
    "logs": "...",
    "outputs": {...}
  }

// Stop workload
DELETE /api/v1/workload/{id}
→ { "status": "stopped", "cleanup": "complete" }

// List active workloads
GET /api/v1/workloads
→ { "active": [...], "queued": [...], "capacity": {...} }
```

---

### 2. Capability Registration

**Register with biomeOS**:
```toml
# ToadStool reports to capability registry:
provides = [
  "Compute",        # Can execute workloads
  "Storage",        # Can manage data
  "Orchestration"   # Can coordinate multi-container workloads
]

requires = [
  "Security",       # For crypto/auth (BearDog)
  "Discovery"       # For finding other services (Songbird)
]

# Detailed capabilities:
compute_types = ["wasm", "container", "python", "native", "gpu"]
storage_types = ["local", "distributed", "encrypted"]
max_concurrent_workloads = 10
available_resources = { "cpu": 8, "mem": "32Gi", "gpu": 1 }
```

---

### 3. Long-Running Service Management

**Daemon manages persistent workloads**:
```bash
# User submits:
POST /api/v1/workload/submit
{
  "type": "persistent",
  "biome_yaml": "database-service.yaml",
  "restart_policy": "always"
}

# ToadStool daemon:
# - Starts the database
# - Monitors health
# - Restarts on failure
# - Reports status to biomeOS
# - Persists across ToadStool restarts
```

---

### 4. Resource Coordination

**Report capacity to biomeOS**:
```rust
// ToadStool daemon sends heartbeats with resource status
{
  "primal_id": "toadstool-tower1",
  "timestamp": "2026-01-04T...",
  "resources": {
    "cpu": { "total": 8, "available": 6, "usage": 25% },
    "mem": { "total": "32Gi", "available": "24Gi", "usage": 25% },
    "gpu": { "total": 1, "available": 1, "usage": 0% },
    "storage": { "total": "500Gi", "available": "350Gi" }
  },
  "workloads": {
    "active": 2,
    "queued": 0,
    "max_capacity": 10
  }
}
```

**biomeOS can route workloads based on capacity**:
```rust
// BearDog needs to run ML inference
let toadstool = registry.get_provider_with_capacity(
    Capability::Compute,
    ResourceRequirements { gpu: 1, mem: "8Gi" }
).await?;

// Submit to ToadStool with available resources
toadstool.submit_workload(inference_job).await?;
```

---

## 🌐 Use Cases

### Use Case 1: BearDog Requests Compute

**Scenario**: BearDog needs to run ML inference for trust evaluation.

```rust
// BearDog discovers ToadStool via capability registry
let compute = registry.get_provider(Capability::Compute).await?;

// Submit workload
let response = compute.post("/api/v1/workload/submit", json!({
    "type": "ml_inference",
    "model": "trust_evaluator_v2",
    "input": input_data,
    "requester": "beardog-trust-eval"
})).await?;

// Wait for result (or async callback)
let result = compute.get(&format!("/api/v1/workload/{}", response.workload_id))
    .await?;
```

**ToadStool daemon**:
- Receives request
- Validates with BearDog (security check)
- Allocates resources
- Executes ML model
- Returns results
- Logs for audit

---

### Use Case 2: Remote Workload Submission

**Scenario**: Tower 2 needs to offload compute to Tower 1.

```bash
# Tower 2 (overloaded)
Tower2-ToadStool discovers Tower1-ToadStool via Songbird
Tower2-ToadStool submits workload to Tower1-ToadStool
Tower1-ToadStool executes and returns results
Tower2-ToadStool returns results to requester
```

**This enables**:
- Load balancing across towers
- Distributed compute
- Failover for compute tasks

---

### Use Case 3: Persistent Services

**Scenario**: Run a database service managed by ToadStool.

```yaml
# biome.yaml
services:
  - name: postgres-db
    type: container
    image: postgres:16
    persistent: true
    restart_policy: always
    volumes:
      - /data/postgres:/var/lib/postgresql/data
```

```bash
# Submit to ToadStool daemon
POST /api/v1/workload/submit
{ "biome_yaml": "...", "type": "persistent" }

# ToadStool daemon:
# - Starts postgres container
# - Monitors health
# - Restarts on failure
# - Persists state across ToadStool restarts
# - Reports to biomeOS that "postgres-db" is available
```

**Other primals can now discover and use the database**:
```rust
let db = registry.get_service("postgres-db").await?;
db.connect("postgresql://...").await?;
```

---

## 🔧 Implementation Design

### Dual-Mode Architecture

```rust
// Main CLI
enum ToadStoolMode {
    Cli(CliArgs),      // Traditional CLI commands
    Daemon(DaemonConfig),  // Daemon mode
}

fn main() {
    let args = parse_args();
    
    match args.mode {
        ToadStoolMode::Cli(cli) => {
            // Run traditional CLI commands
            execute_cli_command(cli).await
        }
        ToadStoolMode::Daemon(config) => {
            // Start daemon mode
            start_daemon_server(config).await
        }
    }
}
```

### Daemon Server

```rust
struct ToadStoolDaemon {
    // API server for workload submission
    api_server: HttpServer,
    
    // Workload manager
    workload_manager: WorkloadManager,
    
    // biomeOS registry client
    registry_client: BiomeOSRegistryClient,
    
    // Resource monitor
    resource_monitor: ResourceMonitor,
    
    // Health monitoring
    health_monitor: HealthMonitor,
}

impl ToadStoolDaemon {
    async fn start(config: DaemonConfig) -> Result<Self> {
        // 1. Connect to biomeOS capability registry
        let registry_client = BiomeOSRegistryClient::connect(&config.biomeos_socket).await?;
        
        // 2. Register capabilities
        registry_client.register(PrimalRegistration {
            id: PrimalId::new(&format!("toadstool-{}", hostname())),
            provides: vec![
                Capability::Compute { compute_types: vec!["wasm", "container", "python"] },
                Capability::Storage { storage_types: vec!["local", "distributed"] },
                Capability::Orchestration,
            ],
            requires: vec![
                Capability::Security,
                Capability::Discovery,
            ],
            unix_socket_path: Some(config.unix_socket.clone()),
            http_endpoint: Some(format!("http://localhost:{}", config.http_port)),
        }).await?;
        
        // 3. Start API server
        let api_server = HttpServer::new(config.http_port, workload_routes());
        
        // 4. Start resource monitoring
        let resource_monitor = ResourceMonitor::start(registry_client.clone());
        
        // 5. Start health monitoring
        let health_monitor = HealthMonitor::start(config.health_check_interval);
        
        // 6. Send periodic heartbeats
        tokio::spawn(send_heartbeats(registry_client.clone()));
        
        Ok(Self {
            api_server,
            workload_manager: WorkloadManager::new(),
            registry_client,
            resource_monitor,
            health_monitor,
        })
    }
    
    async fn run(&self) -> Result<()> {
        // Run until shutdown signal
        tokio::select! {
            _ = self.api_server.serve() => {},
            _ = signal::ctrl_c() => {
                info!("Shutting down ToadStool daemon...");
            }
        }
        
        // Cleanup
        self.workload_manager.stop_all().await?;
        self.registry_client.unregister().await?;
        
        Ok(())
    }
}
```

---

## 🎯 Benefits

### 1. API-Driven Workload Execution
- Other primals can request compute without CLI
- Remote workload submission
- Programmatic control

### 2. Resource Pooling
- Multiple towers share compute capacity
- Load balancing across ToadStool instances
- Efficient resource utilization

### 3. Persistent Service Management
- Long-running services (databases, web servers)
- Automatic restart on failure
- State persistence

### 4. Ecosystem Integration
- Capability-based discovery
- Security via BearDog
- Discovery via Songbird
- Orchestration by biomeOS

### 5. True Fungal Behavior
- **CLI mode**: Specialized fruiting (specific project)
- **Daemon mode**: Mycelium network (shared compute)
- Same organism, different forms!

---

## 🔄 CLI vs Daemon Comparison

| Feature | CLI Mode | Daemon Mode |
|---------|----------|-------------|
| **Invocation** | `toadstool run biome.yaml` | `toadstool daemon` |
| **Use Case** | Direct project execution | Ecosystem compute service |
| **Lifecycle** | Exits after task | Runs continuously |
| **Discovery** | None (local only) | Registers with biomeOS |
| **API** | CLI only | HTTP + Unix socket |
| **Workloads** | Single, synchronous | Multiple, async/queued |
| **Resource Mgmt** | Project-local | System-wide, coordinated |
| **Integration** | Standalone | Full ecosystem integration |

---

## 📋 Implementation Effort

### Phase 1: Daemon Server (4-6 hours)
- Dual-mode CLI parsing
- HTTP API server
- Basic workload submission/query
- Health endpoint

### Phase 2: biomeOS Integration (3-4 hours)
- Capability registry client
- Registration on startup
- Heartbeat mechanism
- Resource reporting

### Phase 3: Workload Manager (4-6 hours)
- Queue management
- Lifecycle (start/stop/restart)
- Resource allocation
- Persistent workloads

### Phase 4: Advanced Features (6-8 hours)
- Multi-tower coordination
- Load balancing
- Advanced scheduling
- Metrics and monitoring

**Total**: 17-24 hours (can be parallelized with other primal work)

---

## 🎊 Conclusion

**ToadStool daemon mode transforms it from**:
- A CLI tool for specific projects →
- A workload execution service for the ecosystem

**Like its namesake fungus**:
- **Fruiting body** (CLI): Specialized, project-specific
- **Mycelium** (Daemon): Network-wide, resource-sharing

**This completes the three-primal architecture**:
- **BearDog**: Security & trust (daemon)
- **Songbird**: Discovery & routing (daemon)
- **ToadStool**: Compute & orchestration (CLI + daemon)

All primals can operate standalone (CLI) or as ecosystem services (daemon)!

🦀 **Adaptive Architecture • Fungal Intelligence • Production Ready** 🍄

