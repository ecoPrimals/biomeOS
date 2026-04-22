# biomeOS Bootstrap Mode: Ecosystem Genesis

**Date**: January 21, 2026  
**Type**: Lifecycle Specification  
**Status**: 🌱 **IMPLEMENTATION GROUNDWORK**

---

## 🎯 CORE CONCEPT

**biomeOS has two modes**:

1. **Bootstrap Mode** (Standalone) - When biomeOS is alone, it acts as the genesis primal
2. **Coordinated Mode** (Inherited) - Once Tower Atomic exists, biomeOS becomes a regular primal

**Like liveSpore as OS substrate**: biomeOS provides the substrate for the ecosystem, then becomes part of it.

---

## 🌱 BOOTSTRAP MODE (Genesis)

### When biomeOS Boots Alone

```
System starts
    ↓
No Tower Atomic exists
    ↓
biomeOS enters BOOTSTRAP MODE
    ↓
Responsibilities:
  - Generate initial family ID
  - Create capability registry
  - Assign own socket
  - Germinate Tower Atomic
  - Wait for Tower Atomic health
  - TRANSITION to Coordinated Mode
```

### What biomeOS Does in Bootstrap Mode

```rust
// Pseudo-code for bootstrap mode

async fn bootstrap_mode() -> Result<()> {
    info!("🌱 biomeOS starting in BOOTSTRAP MODE (no ecosystem)");
    
    // 1. Self-initialization
    let family_id = generate_or_load_family_id()?; // "${FAMILY_ID}"
    let biomeos_socket = assign_biomeos_socket(&family_id); // "/tmp/biomeos-${FAMILY_ID}.sock"
    
    // 2. Create capability registry (biomeOS provides this initially)
    let registry = CapabilityRegistry::new()?;
    
    // 3. Register self (bootstrapping)
    registry.register(DiscoveredPrimal {
        id: format!("biomeos-{}", family_id),
        capabilities: vec![
            "primal.germination".to_string(),
            "ecosystem.coordination".to_string(),
        ],
        primary_socket: biomeos_socket.clone(),
        health: PrimalHealth::Bootstrapping,
    })?;
    
    // 4. Germinate Tower Atomic (genesis)
    info!("🏰 Germinating Tower Atomic (ecosystem foundation)");
    germinate_tower_atomic(&family_id, &registry).await?;
    
    // 5. Wait for Tower Atomic to be healthy
    wait_for_tower_atomic_health().await?;
    
    // 6. TRANSITION to Coordinated Mode
    info!("🔄 Tower Atomic healthy - transitioning to COORDINATED MODE");
    transition_to_coordinated_mode(&family_id).await?;
    
    Ok(())
}
```

---

## 🔄 MODE TRANSITION (Genesis → Coordinated)

### The Critical Transition

**Bootstrap Mode** (biomeOS is the substrate):
```
biomeOS (standalone)
  - Provides capability registry
  - No security inheritance
  - Acts as genesis primal
```

**Coordinated Mode** (biomeOS inherits from Tower):
```
Tower Atomic (gen 0 - security foundation)
    ↓
biomeOS (gen 1 - inherits security)
  - Uses Tower Atomic for security
  - Registers in Tower's ecosystem
  - Acts as ecosystem manager primal
```

### Transition Steps

```rust
async fn transition_to_coordinated_mode(family_id: &str) -> Result<()> {
    info!("🔄 Transitioning biomeOS from Bootstrap to Coordinated Mode");
    
    // 1. Discover Tower Atomic
    let tower = discover_primal_by_capability("btsp.internal").await?;
    info!("✅ Discovered Tower Atomic: {}", tower.id);
    
    // 2. Establish secure connection with Tower
    let tunnel = establish_btsp_tunnel(&tower).await?;
    info!("✅ Secure tunnel to Tower Atomic established");
    
    // 3. Inherit security context from Tower
    let security_context = inherit_security_from_tower(&tunnel).await?;
    info!("✅ Security context inherited (generation: 1)");
    
    // 4. Re-register in Tower's ecosystem (now with security)
    re_register_with_security(&security_context).await?;
    info!("✅ Re-registered in Tower Atomic ecosystem");
    
    // 5. Update internal state
    self.mode = BiomeOsMode::Coordinated;
    self.security_parent = Some("tower_atomic".to_string());
    self.generation = 1;
    
    info!("🎊 biomeOS now operating in COORDINATED MODE (gen 1)");
    Ok(())
}
```

---

## 🏗️ IMPLEMENTATION: Bootstrap Sequence

### Phase 1: Initial Startup

**File**: `crates/biomeos-atomic-deploy/src/bootstrap.rs` (NEW)

```rust
/// biomeOS Bootstrap Mode
/// 
/// When biomeOS starts without an existing ecosystem,
/// it enters bootstrap mode to create the foundation.

use std::path::PathBuf;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

pub struct BootstrapConfig {
    pub family_id: String,
    pub biomeos_socket: PathBuf,
    pub tower_atomic_graph: PathBuf,
}

impl BootstrapConfig {
    /// Detect if we're in bootstrap mode
    pub async fn detect_mode() -> BiomeOsMode {
        // Check if Tower Atomic exists
        if Self::tower_atomic_exists().await {
            BiomeOsMode::Coordinated
        } else {
            BiomeOsMode::Bootstrap
        }
    }
    
    async fn tower_atomic_exists() -> bool {
        // Try to discover Tower Atomic
        // If found → Coordinated Mode
        // If not found → Bootstrap Mode
        
        let sockets = vec![
            "/tmp/songbird-${FAMILY_ID}.sock",
            "/tmp/beardog-${FAMILY_ID}.sock",
        ];
        
        for socket in sockets {
            if PathBuf::from(socket).exists() {
                // Try to connect
                if Self::try_connect(socket).await.is_ok() {
                    return true;
                }
            }
        }
        
        false
    }
    
    async fn try_connect(socket: &str) -> Result<()> {
        // Quick health check
        use tokio::net::UnixStream;
        let _stream = UnixStream::connect(socket).await?;
        Ok(())
    }
}

pub async fn bootstrap_ecosystem(config: BootstrapConfig) -> Result<()> {
    info!("🌱 === BIOMEOS BOOTSTRAP MODE ===");
    info!("🌍 No existing ecosystem detected");
    info!("🏗️  Creating ecosystem foundation...");
    
    // 1. Generate family ID if needed
    let family_id = config.family_id;
    info!("🧬 Family ID: {}", family_id);
    
    // 2. Assign biomeOS socket
    let biomeos_socket = PathBuf::from(format!("/tmp/biomeos-{}.sock", family_id));
    info!("📍 biomeOS socket: {:?}", biomeos_socket);
    
    // 3. Create capability registry (biomeOS hosts it initially)
    info!("📋 Creating capability registry...");
    let registry = create_initial_registry(&family_id).await?;
    
    // 4. Register biomeOS itself
    info!("🌍 Registering biomeOS (bootstrap)...");
    register_biomeos_bootstrap(&registry, &family_id, &biomeos_socket).await?;
    
    // 5. Germinate Tower Atomic
    info!("🏰 Germinating Tower Atomic (ecosystem foundation)...");
    let tower_graph = config.tower_atomic_graph;
    germinate_tower_atomic(&family_id, &tower_graph, &registry).await?;
    
    // 6. Wait for Tower Atomic health
    info!("⏳ Waiting for Tower Atomic to become healthy...");
    wait_for_tower_health(&family_id).await?;
    info!("✅ Tower Atomic is healthy!");
    
    // 7. Transition to Coordinated Mode
    info!("🔄 Transitioning to COORDINATED MODE...");
    transition_to_coordinated().await?;
    
    info!("🎊 === BOOTSTRAP COMPLETE ===");
    info!("🌍 Ecosystem foundation established");
    info!("📊 biomeOS now operating in Coordinated Mode (gen 1)");
    
    Ok(())
}

async fn germinate_tower_atomic(
    family_id: &str,
    graph_path: &PathBuf,
    registry: &CapabilityRegistry
) -> Result<()> {
    // Load Tower Atomic graph
    let graph = Graph::load_from_file(graph_path)?;
    
    // Assign sockets for BearDog and Songbird
    let beardog_socket = PathBuf::from(format!("/tmp/beardog-{}.sock", family_id));
    let songbird_socket = PathBuf::from(format!("/tmp/songbird-{}.sock", family_id));
    
    info!("📍 BearDog socket: {:?}", beardog_socket);
    info!("📍 Songbird socket: {:?}", songbird_socket);
    
    // Execute graph (germinate Tower Atomic)
    let executor = NeuralExecutor::new(registry.clone());
    executor.execute_graph(graph, family_id).await?;
    
    // Pre-register Tower Atomic capabilities
    registry.register(DiscoveredPrimal {
        id: format!("beardog-{}", family_id),
        capabilities: vec!["crypto.sign".into(), "crypto.verify".into()],
        primary_socket: beardog_socket,
        health: PrimalHealth::Starting,
    }).await?;
    
    registry.register(DiscoveredPrimal {
        id: format!("songbird-{}", family_id),
        capabilities: vec!["btsp.internal".into(), "btsp.external".into()],
        primary_socket: songbird_socket,
        health: PrimalHealth::Starting,
    }).await?;
    
    Ok(())
}

async fn wait_for_tower_health(family_id: &str) -> Result<()> {
    let max_wait = Duration::from_secs(30);
    let check_interval = Duration::from_secs(1);
    let start = std::time::Instant::now();
    
    loop {
        if start.elapsed() > max_wait {
            return Err(anyhow!("Tower Atomic did not become healthy within 30s"));
        }
        
        // Check BearDog health
        let beardog = check_primal_health(&format!("beardog-{}", family_id)).await;
        let songbird = check_primal_health(&format!("songbird-{}", family_id)).await;
        
        if beardog.is_ok() && songbird.is_ok() {
            return Ok(());
        }
        
        sleep(check_interval).await;
    }
}

async fn transition_to_coordinated() -> Result<()> {
    // 1. Discover Tower Atomic via capability
    let tower = discover_capability("btsp.internal").await?;
    info!("✅ Discovered Tower Atomic: {}", tower.id);
    
    // 2. Establish BTSP tunnel (inherit security)
    // This is where biomeOS becomes gen 1 (inherits from Tower)
    let tunnel = establish_btsp_tunnel(&tower).await?;
    info!("✅ Secure tunnel established (inherited security)");
    
    // 3. Re-register with inherited security
    let security_context = SecurityContext::from_tunnel(&tunnel);
    re_register_biomeos_with_security(&security_context).await?;
    
    info!("🎊 biomeOS is now generation 1 (inherited from Tower Atomic)");
    Ok(())
}
```

---

## 📋 IMPLEMENTATION PLAN

### Week 1: Bootstrap Detection

**Files to create**:
- `crates/biomeos-atomic-deploy/src/bootstrap.rs`
- `crates/biomeos-atomic-deploy/src/mode.rs`

**Features**:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum BiomeOsMode {
    Bootstrap,    // No ecosystem, creating foundation
    Coordinated,  // Ecosystem exists, participating as primal
}

impl BiomeOsMode {
    pub async fn detect() -> Self {
        if tower_atomic_exists().await {
            Self::Coordinated
        } else {
            Self::Bootstrap
        }
    }
}
```

### Week 1: Socket Assignment (Nucleation)

**Files to modify**:
- `crates/biomeos-atomic-deploy/src/neural_executor.rs`

**Add**:
```rust
impl NeuralExecutor {
    pub fn assign_socket_path(&self, primal: &str, family_id: &str) -> PathBuf {
        // Deterministic socket assignment (nucleation point)
        let runtime_dir = env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| "/tmp".to_string());
        
        PathBuf::from(format!("{}/{}-{}.sock", runtime_dir, primal, family_id))
    }
    
    pub fn assign_sockets_for_graph(&self, graph: &Graph, family_id: &str) 
        -> HashMap<String, PathBuf> 
    {
        let mut assignments = HashMap::new();
        
        for node in &graph.nodes {
            let socket = self.assign_socket_path(&node.primal, family_id);
            assignments.insert(node.id.clone(), socket);
        }
        
        assignments
    }
}
```

### Week 1: biomeOS Self-Registration

**Files to modify**:
- `crates/biomeos-atomic-deploy/src/neural_api_server.rs`

**Add**:
```rust
impl NeuralApiServer {
    pub async fn register_self_in_registry(&mut self) -> Result<()> {
        // biomeOS registers itself as a discoverable primal
        self.capability_registry.register(DiscoveredPrimal {
            id: format!("biomeos-{}", self.family_id),
            capabilities: vec![
                "primal.germination".to_string(),
                "primal.terraria".to_string(),
                "ecosystem.coordination".to_string(),
                "ecosystem.nucleation".to_string(),
            ],
            primary_socket: self.socket_path.clone(),
            health: if self.mode == BiomeOsMode::Bootstrap {
                PrimalHealth::Bootstrapping
            } else {
                PrimalHealth::Healthy
            },
            registered_at: Utc::now(),
        }).await?;
        
        info!("✅ biomeOS registered in capability registry");
        Ok(())
    }
}
```

### Week 2: Tower Atomic Bootstrap Graph

**File to create**: `graphs/tower_atomic_bootstrap.toml`

```toml
# Tower Atomic Bootstrap Graph
# Used by biomeOS in Bootstrap Mode

[graph]
id = "tower_atomic_bootstrap"
family_id = "${FAMILY_ID}"  # Can be overridden
coordination = "Sequential"
description = "Genesis: Create ecosystem security foundation"

# Phase 1: BearDog (Crypto Foundation)
[[nodes]]
id = "germinate-beardog"
primal = { name = "beardog" }
output = "beardog_genesis"
capabilities = ["crypto.sign", "crypto.verify", "crypto.encrypt", "crypto.decrypt"]

[nodes.operation]
name = "start"

[nodes.operation.params]
mode = "server"
# Socket assigned by biomeOS nucleation

[nodes.constraints]
timeout_ms = 30000

# Phase 2: Songbird (Network + BTSP)
[[nodes]]
id = "germinate-songbird"
primal = { name = "songbird" }
depends_on = ["germinate-beardog"]
output = "songbird_genesis"
capabilities = ["btsp.internal", "btsp.external", "discovery.announce"]

[nodes.operation]
name = "start"

[nodes.operation.params]
mode = "server"
# Socket assigned by biomeOS nucleation
# BEARDOG_SOCKET injected by biomeOS (dependency)

[nodes.constraints]
timeout_ms = 30000

# Phase 3: Validate Tower Atomic
[[nodes]]
id = "validate-tower"
depends_on = ["germinate-beardog", "germinate-songbird"]
output = "tower_validated"

[nodes.operation]
name = "health_check"

[nodes.operation.params]
check_beardog = true
check_songbird = true
check_btsp_tunnel = true  # Can they establish tunnel?
```

---

## 🔄 STARTUP SEQUENCE

### Complete Bootstrap Flow

```
1. biomeOS starts
   └─> Detects mode: No Tower Atomic found
   └─> Enters BOOTSTRAP MODE

2. Bootstrap initialization
   ├─> Generate/load family ID (${FAMILY_ID})
   ├─> Assign biomeOS socket (/tmp/biomeos-${FAMILY_ID}.sock)
   ├─> Create capability registry
   └─> Register self (bootstrap mode)

3. Germinate Tower Atomic
   ├─> Load tower_atomic_bootstrap.toml
   ├─> Assign sockets (nucleation):
   │   ├─> BearDog: /tmp/beardog-${FAMILY_ID}.sock
   │   └─> Songbird: /tmp/songbird-${FAMILY_ID}.sock
   ├─> Deploy BearDog (wait for health)
   ├─> Deploy Songbird with BearDog socket
   └─> Wait for Tower Atomic health

4. Transition to Coordinated Mode
   ├─> Discover Tower Atomic via capability
   ├─> Establish BTSP tunnel (inherit security)
   ├─> Re-register as gen 1 primal
   └─> Update internal mode flag

5. Now operating as regular primal
   └─> Other primals can discover biomeOS
   └─> biomeOS can germinate new primals
   └─> All inherit from Tower Atomic
```

---

## 🌍 SUBSEQUENT STARTUPS

### After Bootstrap (Normal Operation)

```
biomeOS starts
    ↓
Detects mode: Tower Atomic exists
    ↓
Enters COORDINATED MODE directly
    ↓
1. Discover Tower Atomic
2. Establish BTSP tunnel
3. Inherit security (gen 1)
4. Register in ecosystem
5. Resume ecosystem management duties
```

**No re-bootstrapping needed!**

---

## 📊 MODE COMPARISON

| Aspect | Bootstrap Mode | Coordinated Mode |
|--------|---------------|------------------|
| **Trigger** | No Tower Atomic found | Tower Atomic exists |
| **Role** | Genesis primal | Ecosystem manager primal |
| **Security** | Self-secured | Inherited from Tower |
| **Generation** | 0 (temporary) | 1 (permanent) |
| **Registry** | Hosts registry | Uses registry |
| **Responsibilities** | Create foundation | Manage ecosystem |
| **Dependencies** | None (standalone) | Tower Atomic (parent) |

---

## 🎯 SUCCESS CRITERIA

### Bootstrap Mode Complete When:

1. ✅ biomeOS detects absence of Tower Atomic
2. ✅ biomeOS creates capability registry
3. ✅ biomeOS assigns sockets deterministically
4. ✅ biomeOS germinates Tower Atomic successfully
5. ✅ Tower Atomic becomes healthy
6. ✅ biomeOS transitions to Coordinated Mode
7. ✅ biomeOS inherits security from Tower

### Coordinated Mode Complete When:

1. ✅ biomeOS discovers existing Tower Atomic
2. ✅ biomeOS establishes BTSP tunnel
3. ✅ biomeOS inherits security context
4. ✅ biomeOS registers as gen 1 primal
5. ✅ biomeOS can germinate new primals
6. ✅ New primals inherit from Tower

---

## 🌟 THE VISION

```
Day 1 (Bootstrap):
    biomeOS alone (substrate)
        ↓
    Creates Tower Atomic (foundation)
        ↓
    Transitions to coordinated (participant)

Day 2+ (Normal):
    biomeOS starts
        ↓
    Finds Tower Atomic (existing)
        ↓
    Joins as primal (gen 1)

Future (Nested):
    Production biomeOS (gen 1)
        ↓
    Germinates Terraria biomeOS (gen 2)
        ↓
    Both coordinate via BTSP
```

---

**🌱 biomeOS: Bootstrap Substrate → Ecosystem Participant! 🔄✨**

---

*Specification: January 21, 2026*  
*Type: Bootstrap Lifecycle*  
*Status: Groundwork for implementation*  
*Status: IMPLEMENTED (v3.x) — mode detection and socket assignment operational*

