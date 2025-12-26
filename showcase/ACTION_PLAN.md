# 🎯 BiomeOS Showcase - Revised Action Plan

**Date**: December 24, 2025  
**Status**: Architecture evolved - Primal Adapter Pattern

---

## 🌱 Philosophy Shift

### Previous Approach ❌
- Assume universal CLI contract
- Try to standardize primal interfaces
- BiomeOS expects specific commands

### New Approach ✅
- **CLI Agnostic**: BiomeOS learns each primal's interface
- **Primal Sovereignty**: Respect autonomy, negotiate lifecycle
- **Songbird for Ports**: Dynamic port allocation, zero hardcoding
- **Cell Senescence**: Request transitions, don't command

---

## 🔍 What We Discovered

### ✅ What Works
- BiomeOS core architecture is solid
- Graceful degradation works perfectly
- Capability-based discovery functional
- Squirrel integrates successfully

### 🌱 What We're Building Differently
- Primal Adapter Pattern (learn interfaces)
- Lifecycle Negotiation Protocol (request, don't command)
- Songbird Port Manager (dynamic allocation)
- Future-proof evolution (handle primal changes)

---

## 🚀 Immediate Actions (Next 1-2 Weeks)

### Action 1: Implement Primal Adapter Pattern ⭐
**Priority**: Critical  
**Time**: 3-5 days

**What we're building**:
```rust
pub struct PrimalAdapter {
    interface: PrimalInterface,  // How to talk to this primal
    capabilities: LifecycleCapabilities,  // What it supports
    state: PrimalState,  // Current lifecycle
}

pub enum PrimalInterface {
    Direct { binary, args },  // Like Squirrel
    Subcommand { binary, start_cmd, stop_cmd },  // Like NestGate
    Service { service_name, manager },  // systemd, etc
    Api { endpoint, lifecycle_endpoints },  // HTTP-based
    Unknown { binary, learned },  // Probe and learn
}
```

**Implementation**:
- [ ] Core adapter types
- [ ] Interface discovery (probe patterns)
- [ ] Adapter cache (persist learnings)
- [ ] Test with phase1bins

**Output**: `biomeos-core/src/primal_adapter/`

---

### Action 2: Implement Lifecycle Negotiation
**Priority**: High  
**Time**: 2-4 days

**What we're building**:
```rust
// Request lifecycle transition (primal can refuse)
pub async fn request_lifecycle_transition(
    primal_id: &str,
    transition: LifecycleTransition,
    reason: TransitionReason,
) -> Result<LifecycleResponse>

pub enum LifecycleResponse {
    Accepted,  // Primal agrees
    Deferred(Duration),  // Primal asks for time
    Refused(String),  // Primal refuses (we respect it)
    NotSupported,  // Primal doesn't support this
}
```

**Philosophy**: Cell senescence, not overwatch
- Request, don't command
- Primals can refuse
- Graceful degradation
- Respect sovereignty

**Implementation**:
- [ ] Lifecycle request protocol
- [ ] Response handling
- [ ] Refusal graceful handling
- [ ] Test negotiation flows

**Output**: `biomeos-core/src/lifecycle_negotiation/`

---

### Action 3: Integrate Songbird Port Manager
**Priority**: High  
**Time**: 2-3 days

**What we're building**:
```rust
pub struct SongbirdPortManager {
    songbird: SongbirdClient,
}

impl SongbirdPortManager {
    // Request port assignment from Songbird
    async fn request_port(&self, primal_id: &str) -> Result<u16>;
    
    // Register service in mesh
    async fn register_service(&self, primal_id: &str, port: u16) -> Result<()>;
    
    // Swap connections dynamically
    async fn swap_connection(&self, from: &str, to: &str) -> Result<()>;
}
```

**Flow**:
1. BiomeOS asks Songbird for port
2. Songbird assigns based on mesh topology
3. BiomeOS starts primal with assigned port
4. BiomeOS registers primal in Songbird mesh
5. Other primals discover through Songbird (zero hardcoding!)

**Implementation**:
- [ ] SongbirdPortManager
- [ ] Port request protocol
- [ ] Service registration
- [ ] Fallback when Songbird unavailable

**Output**: `biomeos-core/src/port_management/`

---

### Action 4: Request Phase 1 CLI Documentation
**Priority**: High  
**Time**: Ongoing (waiting on teams)

**What we need from Phase 1 teams**:
- Start command (actual working command)
- Port configuration method (flag, env var, config file)
- Health check endpoint
- Version/help commands
- Graceful shutdown support

**Output**: `/docs/PHASE1_INTEGRATION_GAPS.md` (sent to teams)

---

### Action 5: Update Showcase with Adapters
**Priority**: Medium  
**Time**: 2-3 days (after adapters built)

Once adapters are working:
```bash
# Showcase uses adapter pattern
./showcase/02-multi-primal/adaptive-orchestration.sh

# BiomeOS discovers how to start each primal
# Uses learned interfaces from cache
# Falls back to probing if unknown
```

Files to create:
- `showcase/03-adaptive-orchestration/` (new scenario)
- `showcase/helpers/start-with-adapter.sh`

---

## 📋 Architecture Benefits

### For BiomeOS ✅
- CLI agnostic (works with any primal)
- Future-proof (handles primal evolution)
- Graceful degradation (missing features = no crash)
- Respects sovereignty (negotiated lifecycle)

### For Primals ✅
- Full autonomy (use your own CLI)
- Can refuse requests (sovereignty!)
- Dynamic port allocation (Songbird manages)
- Evolutionary freedom (change without breaking ecosystem)

### For Ecosystem ✅
- Ecological substrate (not orchestrator)
- Cell senescence model (not overwatch)
- Service mesh coordination (Songbird handles ports)
- Sustainable growth (primals evolve independently)

---

## 🎯 Revised Timeline

### Week 1-2 (Immediate)
**Core Adapter Pattern**:
- [ ] Implement `PrimalAdapter` types
- [ ] Add interface discovery (probe patterns)
- [ ] Add adapter cache
- [ ] Test with Squirrel (already works)
- [ ] Test with other phase1bins

**Estimated**: 3-5 days focused work

### Week 3-4 (Short-term)
**Lifecycle & Ports**:
- [ ] Implement lifecycle negotiation
- [ ] Integrate Songbird port manager
- [ ] Add graceful refusal handling
- [ ] Test full negotiation flows

**Estimated**: 4-6 days focused work

### Week 5-6 (Polish)
**Showcase & Documentation**:
- [ ] Update showcase with adapters
- [ ] Create adaptive orchestration demo
- [ ] Document integration patterns
- [ ] Test with all available primals

**Estimated**: 3-5 days focused work

### Total Estimated Time
**6-8 weeks** for complete primal adapter system

---

## 💡 Parallel Tracks

### Track 1: BiomeOS Development (Our Work)
- Implement primal adapter pattern
- Build lifecycle negotiation
- Integrate Songbird port management
- Update showcase demos

### Track 2: Phase 1 Integration (Collaborative)
- Request CLI documentation from teams
- Test with provided interfaces
- Adapt to discovered patterns
- Provide integration feedback

### Track 3: Documentation (Ongoing)
- Document adapter architecture
- Create primal integration guide
- Write lifecycle negotiation protocol
- Build showcase narrative

---

## 📝 Immediate Next Steps

### This Week (BiomeOS Team)
1. Send integration gap report to Phase 1 teams ✅
2. Start implementing `PrimalAdapter` types
3. Build interface discovery (probe patterns)
4. Test with Squirrel (known working)

### This Month (BiomeOS Team)
1. Complete adapter pattern
2. Add lifecycle negotiation
3. Integrate Songbird port manager
4. Update showcase demos

### Ongoing (Collaborative)
1. Collect Phase 1 CLI documentation
2. Test with real primal interfaces
3. Refine adapter patterns
4. Build integration examples

---

**Status**: Architecture redefined, ready to implement  
**Philosophy**: Primal sovereignty + ecological substrate  
**Timeline**: 6-8 weeks for full adaptive system  
**Confidence**: Very high (respects autonomy, future-proof)

---

*"We found the gaps. Now we fix them."* 🔧✅

