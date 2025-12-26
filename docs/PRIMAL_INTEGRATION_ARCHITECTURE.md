# 🌱 BiomeOS Primal Integration Architecture

**Date**: December 24, 2025  
**Version**: 2.0 - Ecological Substrate Model  
**Philosophy**: Cell Senescence, Not Overwatch

---

## 🎯 Core Principles

### 1. Primal Sovereignty First
- Primals are autonomous organisms, not managed services
- BiomeOS is ecological substrate, not orchestrator
- Interactions are **requests**, not commands
- Lifecycle management is **negotiated**, not imposed

### 2. CLI Agnosticism
- No universal primal contract (impossible to enforce)
- Abstract over observed patterns (adaptable)
- Learn and adapt to new primals (future-proof)
- Gracefully handle evolution (primals change)

### 3. Songbird for Port Management
- Primals may have fallback ports
- Songbird assigns, starts, stops, swaps connections
- Dynamic service mesh coordination
- Zero hardcoded endpoints

### 4. Cell Senescence Model
- BiomeOS can suggest lifecycle transitions
- Primals decide if they accept
- Graceful degradation when primals refuse
- Ecosystem health over individual control

---

## 🏗️ Architecture: Primal Adapter Pattern

### Overview
BiomeOS doesn't assume CLI structure. Instead, it learns and adapts.

```rust
/// Primal adapter discovers how to interact with a primal
pub struct PrimalAdapter {
    /// Discovered interface patterns
    pub interface: PrimalInterface,
    /// Lifecycle capabilities (what it supports)
    pub capabilities: LifecycleCapabilities,
    /// Current lifecycle state
    pub state: PrimalState,
}

pub enum PrimalInterface {
    /// Direct binary execution
    Direct { 
        binary: PathBuf,
        args: Vec<String>,
    },
    /// Subcommand-based (serve, start, etc)
    Subcommand {
        binary: PathBuf,
        start_cmd: String,
        stop_cmd: Option<String>,
    },
    /// Service-based (systemd, etc)
    Service {
        service_name: String,
        manager: ServiceManager,
    },
    /// HTTP API-based
    Api {
        endpoint: String,
        lifecycle_endpoints: LifecycleEndpoints,
    },
    /// Unknown - probe and learn
    Unknown {
        binary: PathBuf,
        // Discovered patterns stored here
        learned: LearnedInterface,
    },
}

pub struct LifecycleCapabilities {
    /// Can we request start?
    pub can_start: bool,
    /// Can we request stop?
    pub can_stop: bool,
    /// Can we request restart?
    pub can_restart: bool,
    /// Does it support graceful shutdown?
    pub graceful_shutdown: bool,
    /// Does it support health checks?
    pub health_check: Option<String>,
}
```

---

## 🔄 Lifecycle Management: Cell Senescence Model

### Philosophy
BiomeOS manages primal lifecycle like an ecosystem manages cell senescence:
- **Not**: "I command you to die"
- **But**: "The ecosystem would benefit if you transitioned"

### Lifecycle Negotiation Protocol

```rust
/// Request lifecycle transition (primal can refuse)
pub async fn request_lifecycle_transition(
    &self,
    primal_id: &str,
    transition: LifecycleTransition,
    reason: TransitionReason,
) -> Result<LifecycleResponse> {
    // 1. Check if primal supports this transition
    let adapter = self.adapters.get(primal_id)?;
    if !adapter.supports_transition(&transition) {
        return Ok(LifecycleResponse::NotSupported);
    }
    
    // 2. Request transition (not command)
    let request = LifecycleRequest {
        transition,
        reason,
        urgency: Urgency::Normal,
        requestor: "BiomeOS".into(),
    };
    
    // 3. Primal decides
    let response = adapter.request(request).await?;
    
    // 4. BiomeOS respects decision
    match response {
        LifecycleResponse::Accepted => {
            // Primal agreed, proceed
            self.execute_transition(primal_id, transition).await
        }
        LifecycleResponse::Deferred(duration) => {
            // Primal asks for time
            self.schedule_retry(primal_id, transition, duration)
        }
        LifecycleResponse::Refused(reason) => {
            // Primal refuses, respect it
            self.handle_refusal(primal_id, transition, reason)
        }
    }
}

pub enum LifecycleTransition {
    /// Request primal to start
    Start,
    /// Request graceful shutdown
    GracefulStop,
    /// Request immediate stop (emergency)
    EmergencyStop,
    /// Request restart
    Restart,
    /// Request scale down (reduce resources)
    ScaleDown,
}

pub enum TransitionReason {
    /// Ecosystem health requires it
    EcosystemHealth,
    /// User requested it
    UserRequest,
    /// Resource constraints
    ResourcePressure,
    /// Detected failure/unhealth
    FailureDetected,
    /// Routine maintenance
    Maintenance,
}

pub enum LifecycleResponse {
    /// Primal accepts transition
    Accepted,
    /// Primal defers (needs time)
    Deferred(Duration),
    /// Primal refuses (with reason)
    Refused(String),
    /// Primal doesn't support this
    NotSupported,
}
```

---

## 🎵 Songbird Integration: Dynamic Port Management

### Overview
Songbird manages all port assignments and service mesh coordination.

```rust
/// BiomeOS delegates port management to Songbird
pub struct SongbirdPortManager {
    songbird: SongbirdClient,
}

impl SongbirdPortManager {
    /// Request port for primal (Songbird assigns)
    pub async fn request_port(
        &self,
        primal_id: &str,
        preferred_port: Option<u16>,
    ) -> Result<u16> {
        // Songbird decides port based on:
        // - Availability
        // - Mesh topology
        // - Security policies
        // - Network constraints
        let assignment = self.songbird
            .assign_port(primal_id, preferred_port)
            .await?;
        
        Ok(assignment.port)
    }
    
    /// Request Songbird to route to primal
    pub async fn register_service(
        &self,
        primal_id: &str,
        port: u16,
        capabilities: Vec<String>,
    ) -> Result<ServiceHandle> {
        // Songbird registers in service mesh
        self.songbird
            .register_service(primal_id, port, capabilities)
            .await
    }
    
    /// Request Songbird to swap connections
    pub async fn swap_connection(
        &self,
        from_primal: &str,
        to_primal: &str,
    ) -> Result<()> {
        // Songbird handles connection migration
        self.songbird
            .migrate_connections(from_primal, to_primal)
            .await
    }
}
```

### Port Configuration Flow

```
1. BiomeOS: "Songbird, I need to start Squirrel"
2. Songbird: "I assign port 9010"
3. BiomeOS: Starts Squirrel with PORT=9010
4. Squirrel: Starts on assigned port (or fallback if Songbird unavailable)
5. BiomeOS: "Songbird, Squirrel is on 9010 with capabilities [ai, mcp]"
6. Songbird: Registers in service mesh, enables discovery
7. Other primals: Discover Squirrel through Songbird (no hardcoded ports)
```

---

## 🔍 Primal Adapter Discovery

### How BiomeOS Learns Primal Interfaces

```rust
/// Discover how to interact with a primal
pub async fn discover_primal_interface(
    binary: &Path,
) -> Result<PrimalAdapter> {
    let mut adapter = PrimalAdapter::new(binary);
    
    // 1. Try common patterns (ordered by likelihood)
    let patterns = [
        // Direct execution (like Squirrel)
        || try_direct_execution(binary),
        
        // Subcommand patterns
        || try_subcommand(binary, "serve"),
        || try_subcommand(binary, "start"),
        || try_subcommand(binary, "service"),
        || try_subcommand(binary, "run"),
        
        // Service patterns
        || try_systemd_service(binary),
        || try_docker_service(binary),
        
        // API patterns
        || try_api_lifecycle(binary),
    ];
    
    // 2. Probe each pattern (fast fail)
    for pattern in patterns {
        if let Ok(interface) = pattern().await {
            adapter.interface = interface;
            break;
        }
    }
    
    // 3. Discover capabilities
    adapter.capabilities = discover_capabilities(&adapter.interface).await?;
    
    // 4. Cache for future use
    save_adapter_to_cache(&adapter)?;
    
    Ok(adapter)
}

/// Try direct execution (no subcommands)
async fn try_direct_execution(binary: &Path) -> Result<PrimalInterface> {
    // Check if binary runs directly
    let output = Command::new(binary)
        .arg("--version")
        .timeout(Duration::from_secs(2))
        .output()
        .await?;
    
    if output.status.success() {
        return Ok(PrimalInterface::Direct {
            binary: binary.to_path_buf(),
            args: vec![],
        });
    }
    
    Err(Error::NotDirect)
}

/// Try subcommand pattern
async fn try_subcommand(binary: &Path, cmd: &str) -> Result<PrimalInterface> {
    // Check if binary accepts subcommand
    let output = Command::new(binary)
        .arg(cmd)
        .arg("--help")
        .timeout(Duration::from_secs(2))
        .output()
        .await?;
    
    if output.status.success() {
        return Ok(PrimalInterface::Subcommand {
            binary: binary.to_path_buf(),
            start_cmd: cmd.to_string(),
            stop_cmd: None, // Discover separately
        });
    }
    
    Err(Error::NotSubcommand)
}
```

---

## 📦 Primal Adapter Cache

### Cache Discovered Interfaces

```yaml
# ~/.biomeos/primal_adapters.yaml
adapters:
  squirrel:
    binary: /path/to/squirrel-bin
    interface:
      type: direct
      args: []
    capabilities:
      can_start: true
      can_stop: true
      health_check: "http://localhost:PORT/health"
    last_discovered: 2025-12-24T12:00:00Z
    
  nestgate:
    binary: /path/to/nestgate-bin
    interface:
      type: subcommand
      start_cmd: service
      stop_cmd: null
    capabilities:
      can_start: true
      can_stop: false
      health_check: null
    last_discovered: 2025-12-24T12:00:00Z
```

### Cache Invalidation
- Invalidate on primal version change
- Re-discover periodically (weekly)
- User can force re-discovery

---

## 🌊 Future-Proof Evolution

### How System Adapts to Change

1. **New Primals Emerge**
   - BiomeOS probes interface patterns
   - Discovers capabilities
   - Caches for future use
   - No code changes needed

2. **Existing Primals Evolve**
   - Version detection triggers re-discovery
   - Graceful fallback to old patterns
   - Cache updated automatically

3. **New Interface Patterns**
   - Add new pattern to probe list
   - Existing adapters unaffected
   - System learns incrementally

4. **Breaking Changes**
   - Cache invalidated on version change
   - Re-discovery finds new interface
   - Graceful degradation if incompatible

---

## 🧬 Implementation Plan

### Phase 1: Core Adapter Pattern (1-2 weeks)
- [ ] Implement `PrimalAdapter` struct
- [ ] Implement interface discovery
- [ ] Add common patterns (direct, subcommand)
- [ ] Add adapter cache
- [ ] Test with Squirrel, NestGate, ToadStool

### Phase 2: Lifecycle Negotiation (1-2 weeks)
- [ ] Implement lifecycle request protocol
- [ ] Add capability discovery
- [ ] Add graceful refusal handling
- [ ] Test start/stop/restart flows

### Phase 3: Songbird Integration (1 week)
- [ ] Implement `SongbirdPortManager`
- [ ] Delegate port assignment to Songbird
- [ ] Add service mesh registration
- [ ] Test dynamic port allocation

### Phase 4: Advanced Patterns (2-3 weeks)
- [ ] Add systemd integration
- [ ] Add Docker integration
- [ ] Add API-based lifecycle
- [ ] Add learned pattern storage

### Phase 5: Production Hardening (1-2 weeks)
- [ ] Add retry logic
- [ ] Add timeout handling
- [ ] Add error recovery
- [ ] Add monitoring/telemetry

---

## 📊 Architecture Benefits

### For BiomeOS
- ✅ CLI-agnostic (adapts to any primal)
- ✅ Future-proof (handles new patterns)
- ✅ Graceful degradation (handles failures)
- ✅ Respects sovereignty (negotiated lifecycle)

### For Primals
- ✅ Full autonomy (can refuse requests)
- ✅ No forced interface (use what makes sense)
- ✅ Dynamic port allocation (Songbird manages)
- ✅ Evolutionary freedom (change without breaking ecosystem)

### For Ecosystem
- ✅ Ecological substrate (not orchestrator)
- ✅ Cell senescence model (not overwatch)
- ✅ Service mesh coordination (Songbird)
- ✅ Sustainable growth (primals evolve freely)

---

## 🎓 Key Insights

### Cell Senescence vs Overwatch

**Overwatch Model** (❌ What We Avoid):
- "I control you"
- "You must obey"
- "I decide your lifecycle"
- Central authority

**Cell Senescence Model** (✅ What We Build):
- "The ecosystem needs this"
- "Would you consider?"
- "What do you need?"
- Ecological negotiation

### Primal Sovereignty

Primals are **organisms**, not **services**:
- They have agency
- They make decisions
- They can refuse
- They evolve

BiomeOS is **substrate**, not **controller**:
- Provides environment
- Facilitates connections
- Suggests transitions
- Respects boundaries

---

## 📝 Next Steps

### Immediate (This Week)
1. Implement basic `PrimalAdapter` pattern
2. Add discovery for direct + subcommand interfaces
3. Test with existing phase1bins

### Short-term (2-4 Weeks)
1. Implement lifecycle negotiation protocol
2. Integrate Songbird port management
3. Build adapter cache

### Long-term (1-2 Months)
1. Add advanced patterns (systemd, Docker, API)
2. Build comprehensive test suite
3. Document primal integration guide

---

**Status**: Architecture defined, ready for implementation  
**Philosophy**: Cell senescence, not overwatch  
**Timeline**: 6-8 weeks for full implementation

---

*"BiomeOS is the soil, not the gardener. Primals are the organisms, not the plants we tend."* 🌱✨

