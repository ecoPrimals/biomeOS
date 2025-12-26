# BiomeOS Showcase Buildout Plan

**Date:** December 26, 2025  
**Status:** Comprehensive plan for local and inter-primal showcases  
**Goal:** Build world-class demonstrations like Songbird/ToadStool/BearDog

---

## 🎯 Executive Summary

After reviewing the **mature showcase frameworks** in:
- **Songbird** - Multi-tower federation, BTSP/BirdSong integration (13+ showcase categories)
- **ToadStool** - Local compute capabilities, GPU demos, inter-primal integration
- **BearDog** - Progressive Level 0→4 local-to-ecosystem demos
- **NestGate** - Local storage through full federation (6 progressive levels)
- **Squirrel** - Local-first AI through federated routing

**BiomeOS needs a similar progressive structure** that:
1. **Starts local** - Show BiomeOS capabilities standalone
2. **Adds single primals** - One primal at a time
3. **Builds to pairs** - Two primals working together
4. **Full ecosystem** - All Phase 1 primals coordinated
5. **Pure Rust BTSP/BirdSong** - BiomeOS as P2P coordinator

---

## 📊 Current State Analysis

### ✅ What We Have
- `00-local-capabilities/` - Manifest parsing, capability matching, sovereignty (good start!)
- `01-single-primal/` - Individual primal demos (Songbird, NestGate, ToadStool, BearDog, Squirrel)
- `02-primal-pairs/` - Pair combinations (good structure)
- API adapters for all Phase 1 primals
- CLI adapters for BTSP/BirdSong integration
- Chimera system for composition

### ⚠️ What's Missing
1. **Local BiomeOS capabilities** - Need more Level 0 demos
2. **Progressive learning path** - Not as clear as BearDog's Level 0→4
3. **Pure Rust P2P coordination** - BTSP/BirdSong orchestration showcase
4. **Inter-primal showcases** - Following Songbird's federation patterns
5. **Real integration tests** - Move from mocks to live services

---

## 🏗️ Proposed Showcase Structure

### Level 0: BiomeOS Local Capabilities (30 minutes)
**Path:** `showcase/00-local-biomeos/`

**Status:** Expand existing `00-local-capabilities/`

```
00-local-biomeos/
├── README.md (Level 0 overview, like BearDog's)
├── 01-hello-biomeos/
│   ├── run.sh
│   └── README.md (Generate biome.yaml, validate, show config)
├── 02-manifest-parsing/
│   ├── run.sh
│   └── README.md (Parse various biome.yaml formats)
├── 03-capability-discovery/
│   ├── run.sh
│   └── README.md (Discover what BiomeOS can do)
├── 04-chimera-composition/
│   ├── run.sh
│   └── README.md (Compose primals into chimeras)
├── 05-sovereignty-guardian/
│   ├── run.sh
│   └── README.md (Show sovereignty enforcement)
├── 06-health-monitoring/
│   ├── run.sh
│   └── README.md (BiomeOS health checks)
└── RUN_ALL_LOCAL.sh
```

**Key Demos**:
1. **Hello BiomeOS** - Generate first biome.yaml, start ecosystem
2. **Manifest Parsing** - Parse development/production/AI research biomes
3. **Capability Discovery** - Zero-hardcoding discovery pattern
4. **Chimera Composition** - Build secure-storage chimera (BearDog + NestGate)
5. **Sovereignty Guardian** - Show primal autonomy enforcement
6. **Health Monitoring** - BiomeOS monitoring all primals

**Learning Outcome**: Understand BiomeOS core without any primals running

---

### Level 1: Single Primal Integration (1 hour)
**Path:** `showcase/01-single-primal/` (exists, enhance)

**Status:** Enhance existing demos

```
01-single-primal/
├── README.md (Progressive single primal demos)
├── 01-songbird-discovery/
│   ├── run.sh
│   └── README.md (BiomeOS discovers Songbird, registers services)
├── 02-nestgate-storage/
│   ├── run.sh
│   └── README.md (BiomeOS provisions NestGate volume)
├── 03-toadstool-compute/
│   ├── run.sh
│   └── README.md (BiomeOS submits compute task to ToadStool)
├── 04-beardog-security/
│   ├── run.sh
│   └── README.md (BiomeOS requests BearDog encryption)
├── 05-squirrel-ai/
│   ├── run.sh
│   └── README.md (BiomeOS routes AI request via Squirrel)
└── RUN_ALL_SINGLE.sh
```

**Key Enhancement**: Each demo shows:
- BiomeOS discovering the primal (no hardcoding!)
- BiomeOS adapting to primal's API (API adapter showcase)
- BiomeOS CLI adapter (for BTSP/BirdSong where relevant)
- Health monitoring and failure handling

**Learning Outcome**: BiomeOS can orchestrate each primal independently

---

### Level 2: Primal Pairs (1.5 hours)
**Path:** `showcase/02-primal-pairs/` (exists, enhance)

**Status:** Follow Songbird's inter-primal patterns

```
02-primal-pairs/
├── README.md (Pair integration showcase)
├── 01-songbird-beardog/
│   ├── run-btsp.sh (BiomeOS coordinates BTSP tunnel)
│   ├── run-birdsong.sh (BiomeOS enables BirdSong discovery)
│   └── README.md
├── 02-songbird-nestgate/
│   ├── run-federation.sh (BiomeOS coordinates storage federation)
│   └── README.md
├── 03-songbird-toadstool/
│   ├── run-compute-mesh.sh (BiomeOS coordinates compute discovery)
│   └── README.md
├── 04-songbird-squirrel/
│   ├── run-ai-routing.sh (BiomeOS coordinates AI mesh)
│   └── README.md
├── 05-beardog-nestgate/
│   ├── run-encrypted-storage.sh (BiomeOS creates encrypted volumes)
│   └── README.md
├── 06-beardog-toadstool/
│   ├── run-encrypted-compute.sh (BiomeOS secure compute jobs)
│   └── README.md
├── 07-nestgate-toadstool/
│   ├── run-persistent-results.sh (BiomeOS compute → storage pipeline)
│   └── README.md
├── 08-toadstool-squirrel/
│   ├── run-ai-compute.sh (BiomeOS AI + GPU coordination)
│   └── README.md
└── RUN_ALL_PAIRS.sh
```

**Key Pattern** (following Songbird #13-beardog-integration):
- Real integration tests (not mocks!)
- Show BiomeOS as coordinator
- Demonstrate sovereignty (primals choose to work together)
- Health monitoring and graceful degradation

**Learning Outcome**: BiomeOS coordinates two primals seamlessly

---

### Level 3: P2P Coordination (Pure Rust BTSP/BirdSong) ⭐
**Path:** `showcase/03-p2p-coordination/` (NEW!)

**Status:** **THIS IS THE KILLER FEATURE** - Show BiomeOS doing BTSP/BirdSong coordination in pure Rust

```
03-p2p-coordination/
├── README.md (BiomeOS as P2P coordinator)
├── 01-btsp-tunnel-coordination/
│   ├── run.sh
│   ├── README.md
│   └── src/
│       └── main.rs (BiomeOS Rust code coordinating BTSP)
├── 02-birdsong-encryption/
│   ├── run.sh
│   ├── README.md
│   └── src/
│       └── main.rs (BiomeOS Rust code for BirdSong)
├── 03-lineage-gated-relay/
│   ├── run.sh
│   ├── README.md
│   └── src/
│       └── main.rs (BiomeOS coordinating lineage relay)
├── 04-multi-tower-p2p/
│   ├── run.sh
│   ├── README.md
│   └── src/
│       └── main.rs (BiomeOS coordinating multi-tower P2P)
├── 05-roaming-device/
│   ├── run.sh
│   ├── README.md
│   └── src/
│       └── main.rs (BiomeOS handling connection migration)
└── RUN_ALL_P2P.sh
```

**Key Implementation**:

```rust
// Example: BiomeOS coordinating BTSP tunnel

use biomeos_core::api_adapter::BearDogAdapter;
use biomeos_core::api_adapter::SongbirdAdapter;

async fn coordinate_btsp_tunnel(biome: &BiomeOS) -> Result<()> {
    // BiomeOS discovers BearDog and Songbird
    let beardog = biome.discover_primal("security").await?;
    let songbird = biome.discover_primal("discovery").await?;
    
    // BiomeOS requests BTSP tunnel from BearDog
    let tunnel = beardog.create_btsp_tunnel(
        "node-a", 
        "node-b",
        lineage_proof
    ).await?;
    
    // BiomeOS registers tunnel with Songbird
    songbird.register_secure_transport(tunnel).await?;
    
    // Pure Rust coordination!
    Ok(())
}
```

**Learning Outcome**: BiomeOS is a P2P coordinator in pure Rust, not just an orchestrator!

---

### Level 4: Primal Triples & Full Ecosystem (2 hours)
**Path:** `showcase/04-complete-ecosystem/`

**Status:** Build on pairs to show full integration

```
04-complete-ecosystem/
├── README.md (Full ecosystem demos)
├── 01-secure-storage-pipeline/
│   ├── run.sh (Songbird + BearDog + NestGate)
│   └── README.md
├── 02-secure-compute-pipeline/
│   ├── run.sh (Songbird + BearDog + ToadStool)
│   └── README.md
├── 03-ai-compute-storage/
│   ├── run.sh (ToadStool + Squirrel + NestGate)
│   └── README.md
├── 04-full-phase1-ecosystem/
│   ├── run.sh (All 5 Phase 1 primals)
│   └── README.md
├── 05-multi-tower-federation/
│   ├── run.sh (3+ BiomeOS instances coordinating)
│   └── README.md
└── RUN_FULL_ECOSYSTEM.sh
```

**Key Demo** (Following Songbird's federation success):
- Multiple BiomeOS instances
- Each coordinating different primals
- Cross-tower service discovery
- Load balancing across BiomeOS instances
- Failover when one BiomeOS instance dies

**Learning Outcome**: BiomeOS scales to production ecosystems

---

### Level 5: Real-World Scenarios (2 hours)
**Path:** `showcase/05-real-world/` (NEW!)

**Status:** Production-like scenarios

```
05-real-world/
├── README.md (Real-world use cases)
├── 01-ai-research-team/
│   ├── biome.yaml (AI research biome from specs/)
│   ├── run.sh
│   └── README.md (GPU compute + AI routing + data storage)
├── 02-secure-enterprise/
│   ├── biome.yaml (Enterprise biome from specs/)
│   ├── run.sh
│   └── README.md (Max security + compliance + audit)
├── 03-development-environment/
│   ├── biome.yaml (Dev biome from specs/)
│   ├── run.sh
│   └── README.md (Fast iteration + minimal resources)
├── 04-gaming-tournament/
│   ├── biome.yaml
│   ├── run.sh
│   └── README.md (Real-time compute + low latency)
├── 05-biotech-pipeline/
│   ├── biome.yaml
│   ├── run.sh
│   └── README.md (Data processing + secure storage)
└── RUN_ALL_SCENARIOS.sh
```

**Key Feature**: Use the example biome.yaml files from `specs/examples/`

**Learning Outcome**: BiomeOS solves real problems

---

## 🚀 Implementation Priority

### Phase 1: Foundation (Week 1) ✅ PARTIALLY COMPLETE
- [x] Level 0 structure (enhance existing `00-local-capabilities/`)
- [x] Level 1 structure (enhance existing `01-single-primal/`)
- [ ] README.md files for each level
- [ ] RUN_ALL scripts for each level

### Phase 2: Core Integration (Week 2) 🔄 IN PROGRESS
- [ ] Enhance Level 1 demos (real discovery, not hardcoded)
- [ ] Enhance Level 2 demos (follow Songbird's patterns)
- [ ] Add health monitoring to all demos
- [ ] Add graceful degradation tests

### Phase 3: P2P Coordination (Week 3) ⭐ **PRIORITY**
- [ ] Create Level 3 (P2P coordination)
- [ ] Implement BTSP tunnel coordination in Rust
- [ ] Implement BirdSong encryption coordination in Rust
- [ ] Test with real BearDog + Songbird
- [ ] Document pure Rust P2P capabilities

### Phase 4: Full Ecosystem (Week 4)
- [ ] Create Level 4 (complete ecosystem)
- [ ] Multi-BiomeOS federation demo
- [ ] Failover and recovery demos
- [ ] Performance benchmarks

### Phase 5: Real-World Scenarios (Week 5)
- [ ] Create Level 5 (real-world scenarios)
- [ ] Use example biome.yaml files
- [ ] Production-like configurations
- [ ] End-to-end validation

---

## 📚 Learning from Successful Showcases

### Songbird's Success Patterns
**What makes Songbird's showcases great:**
1. **Progressive complexity** - Isolated → Federation → Inter-primal → Multi-protocol
2. **Real integration** - Tests with actual services, not mocks
3. **Clear README.md** - Each level has comprehensive documentation
4. **Quick Start scripts** - RUN_ALL.sh for easy testing
5. **BTSP/BirdSong demos** - Show killer P2P features

**Apply to BiomeOS:**
- Level 0→5 progression
- Real primal integration (no mocks in showcase)
- Comprehensive README at each level
- Easy automation scripts
- **Pure Rust P2P coordination showcase** ⭐

### BearDog's Success Patterns
**What makes BearDog's showcases great:**
1. **Local-first** - Level 0 teaches fundamentals with zero dependencies
2. **Clear learning path** - Beginner → Intermediate → Advanced
3. **Time estimates** - Each demo has time estimate
4. **Completion tracking** - Progress bars and certificates
5. **Key concepts** - Each demo teaches specific concepts

**Apply to BiomeOS:**
- Start with BiomeOS-only demos (Level 0)
- Clear difficulty progression
- Time estimates for each demo
- Learning outcomes clearly stated
- Completion tracking

### ToadStool's Success Patterns
**What makes ToadStool's showcases great:**
1. **Local capabilities first** - Show ToadStool before ecosystem
2. **GPU demos** - Real hardware integration
3. **Inter-primal progression** - Standalone → Songbird → Multi-primal
4. **Real workloads** - Actual ML jobs, not toys
5. **Performance benchmarks** - Show real capabilities

**Apply to BiomeOS:**
- BiomeOS local capabilities first
- Show real primal coordination (not simplified)
- Build from single to multi-primal
- Use real biome.yaml files
- Performance metrics for coordination overhead

---

## 🎯 Success Criteria

### Level 0 Complete When:
- [ ] Can generate and validate biome.yaml without any primals
- [ ] Can discover BiomeOS capabilities
- [ ] Can compose chimeras conceptually
- [ ] Can show sovereignty guardian principles
- [ ] All demos run in < 30 minutes total

### Level 1 Complete When:
- [ ] BiomeOS discovers each Phase 1 primal
- [ ] BiomeOS adapts to each primal's API
- [ ] Health monitoring works for each primal
- [ ] Graceful degradation demonstrated
- [ ] All demos run in < 1 hour total

### Level 2 Complete When:
- [ ] All primal pairs demonstrated
- [ ] BiomeOS coordinates pair interactions
- [ ] Sovereignty respected (primals choose cooperation)
- [ ] Real integration (no mocks)
- [ ] All demos run in < 1.5 hours total

### Level 3 Complete When: ⭐ **CRITICAL**
- [ ] BiomeOS coordinates BTSP tunnels in pure Rust
- [ ] BiomeOS coordinates BirdSong encryption in pure Rust
- [ ] BiomeOS handles lineage-gated relay
- [ ] BiomeOS manages multi-tower P2P
- [ ] BiomeOS handles roaming devices
- [ ] All demos use Rust code (not shell scripts calling CLIs)

### Level 4 Complete When:
- [ ] Full Phase 1 ecosystem running
- [ ] Multiple BiomeOS instances coordinating
- [ ] Cross-tower service discovery working
- [ ] Failover and recovery demonstrated
- [ ] All demos run in < 2 hours total

### Level 5 Complete When:
- [ ] All example biome.yaml files work
- [ ] Production-like scenarios run
- [ ] Real-world use cases validated
- [ ] Performance benchmarks collected
- [ ] All demos run in < 2 hours total

---

## 🔧 Technical Implementation

### BTSP Coordination in BiomeOS (Pure Rust)

**File:** `crates/biomeos-core/src/p2p_coordination/btsp.rs`

```rust
//! BTSP tunnel coordination in pure Rust
//! BiomeOS acts as P2P coordinator, not just orchestrator

use crate::api_adapter::{BearDogAdapter, SongbirdAdapter};
use anyhow::{Context, Result};

pub struct BtspCoordinator {
    beardog: BearDogAdapter,
    songbird: SongbirdAdapter,
}

impl BtspCoordinator {
    /// Create BTSP tunnel between two nodes
    pub async fn create_tunnel(
        &self,
        node_a: &str,
        node_b: &str,
        lineage_proof: LineageProof,
    ) -> Result<TunnelInfo> {
        // 1. Request tunnel from BearDog
        let tunnel_request = self.beardog
            .request_btsp_tunnel(node_a, node_b, &lineage_proof)
            .await
            .context("Failed to request BTSP tunnel from BearDog")?;
        
        // 2. Register tunnel endpoints with Songbird
        self.songbird
            .register_secure_transport(&tunnel_request.endpoint_a)
            .await
            .context("Failed to register tunnel endpoint A")?;
        
        self.songbird
            .register_secure_transport(&tunnel_request.endpoint_b)
            .await
            .context("Failed to register tunnel endpoint B")?;
        
        // 3. Enable BirdSong encryption for discovery
        self.songbird
            .enable_birdsong(&tunnel_request.encryption_key)
            .await
            .context("Failed to enable BirdSong")?;
        
        Ok(TunnelInfo {
            tunnel_id: tunnel_request.id,
            status: TunnelStatus::Active,
            endpoints: vec![
                tunnel_request.endpoint_a,
                tunnel_request.endpoint_b,
            ],
        })
    }
    
    /// Monitor tunnel health
    pub async fn monitor_tunnel(&self, tunnel_id: &str) -> Result<TunnelHealth> {
        // BiomeOS actively monitors tunnel health
        let beardog_health = self.beardog
            .check_tunnel_health(tunnel_id)
            .await?;
        
        let songbird_health = self.songbird
            .check_transport_health(tunnel_id)
            .await?;
        
        Ok(TunnelHealth {
            tunnel_id: tunnel_id.to_string(),
            encryption_health: beardog_health,
            transport_health: songbird_health,
            overall_status: Self::compute_overall_health(
                beardog_health,
                songbird_health
            ),
        })
    }
}
```

**This is BiomeOS's unique value: Pure Rust P2P coordination!**

### BirdSong Coordination in BiomeOS (Pure Rust)

**File:** `crates/biomeos-core/src/p2p_coordination/birdsong.rs`

```rust
//! BirdSong discovery coordination in pure Rust
//! BiomeOS enables privacy-preserving service discovery

use crate::api_adapter::{BearDogAdapter, SongbirdAdapter};
use anyhow::{Context, Result};

pub struct BirdSongCoordinator {
    beardog: BearDogAdapter,
    songbird: SongbirdAdapter,
}

impl BirdSongCoordinator {
    /// Enable BirdSong discovery (encrypted broadcasts)
    pub async fn enable_birdsong(&self, family_id: &str) -> Result<DiscoveryMode> {
        // 1. Request BirdSong keys from BearDog
        let birdsong_keys = self.beardog
            .generate_birdsong_keys(family_id)
            .await
            .context("Failed to generate BirdSong keys")?;
        
        // 2. Configure Songbird for encrypted discovery
        self.songbird
            .set_discovery_mode(DiscoveryMode::BirdSong {
                encryption_key: birdsong_keys.broadcast_key,
                lineage_filter: birdsong_keys.lineage_proof,
            })
            .await
            .context("Failed to enable BirdSong mode")?;
        
        // 3. Verify encryption working
        let test_broadcast = self.songbird
            .test_encrypted_broadcast()
            .await?;
        
        if !test_broadcast.encrypted {
            anyhow::bail!("BirdSong encryption verification failed");
        }
        
        Ok(DiscoveryMode::BirdSong)
    }
    
    /// Handle lineage-gated relay (NAT traversal without TURN)
    pub async fn coordinate_lineage_relay(
        &self,
        requester: &str,
        target: &str,
    ) -> Result<RelayInfo> {
        // 1. Verify lineage relationship
        let lineage = self.beardog
            .verify_lineage(requester, target)
            .await?;
        
        if !lineage.is_ancestor {
            anyhow::bail!("Lineage verification failed - not an ancestor");
        }
        
        // 2. Request relay from ancestor node
        let relay_offer = self.songbird
            .request_lineage_relay(requester, target, lineage.proof)
            .await?;
        
        // 3. Establish relay connection
        let relay = self.songbird
            .accept_relay_offer(&relay_offer)
            .await?;
        
        Ok(RelayInfo {
            relay_node: relay_offer.relay_node,
            requester: requester.to_string(),
            target: target.to_string(),
            status: RelayStatus::Active,
        })
    }
}
```

---

## 📊 Showcase Metrics

### Current State
```
Level 0 (Local): 50% complete (basic demos exist)
Level 1 (Single): 60% complete (structure exists, needs enhancement)
Level 2 (Pairs): 40% complete (structure exists, needs real integration)
Level 3 (P2P): 0% complete ⭐ (PRIORITY - UNIQUE VALUE PROP)
Level 4 (Ecosystem): 20% complete (basic demos exist)
Level 5 (Real-World): 0% complete (planned)

Overall: ~34% complete
```

### Target State (End of Q1 2026)
```
Level 0 (Local): 100% complete
Level 1 (Single): 100% complete
Level 2 (Pairs): 100% complete
Level 3 (P2P): 100% complete ⭐ (SHOWCASE READY)
Level 4 (Ecosystem): 100% complete
Level 5 (Real-World): 100% complete

Overall: 100% complete
```

---

## 🎯 Next Actions (Prioritized)

### Immediate (This Week)
1. **Create Level 3 structure** (P2P coordination showcase) ⭐
2. **Implement BTSP coordination** in `crates/biomeos-core/src/p2p_coordination/btsp.rs`
3. **Implement BirdSong coordination** in `crates/biomeos-core/src/p2p_coordination/birdsong.rs`
4. **Create demo 01** in `showcase/03-p2p-coordination/01-btsp-tunnel-coordination/`
5. **Test with real BearDog + Songbird**

### Short-Term (Next 2 Weeks)
1. Complete all Level 3 (P2P) demos
2. Enhance Level 1 demos (real discovery, no hardcoding)
3. Enhance Level 2 demos (follow Songbird patterns)
4. Write comprehensive README.md for each level

### Medium-Term (Next Month)
1. Complete Level 4 (full ecosystem)
2. Create Level 5 (real-world scenarios)
3. Performance benchmarks for all demos
4. Documentation and video walkthroughs

---

## 💡 Key Insights

### BiomeOS's Unique Value: Pure Rust P2P Coordination

**What makes BiomeOS special:**
1. **Not just orchestration** - Active P2P coordination
2. **Pure Rust implementation** - Not shell scripts calling CLIs
3. **Sovereignty-respecting** - Primals choose cooperation
4. **Zero-hardcoding** - Capability-based discovery
5. **Production-ready** - Real error handling, health monitoring

**This is our differentiator!** Show BiomeOS doing BTSP/BirdSong coordination in pure Rust code, not just calling BearDog/Songbird CLIs.

### Following Best Practices from Mature Showcases

**From Songbird:**
- Progressive complexity (Isolated → Federation → Internet)
- Real integration tests (no mocks in showcase)
- Comprehensive documentation

**From BearDog:**
- Local-first learning (Level 0 teaches fundamentals)
- Clear progression (Beginner → Advanced)
- Completion tracking

**From ToadStool:**
- Local capabilities first
- Real workloads (not toys)
- Performance benchmarks

---

## ✅ Completion Criteria

BiomeOS showcase is **complete** when:

- [ ] All 5 levels implemented (0→5)
- [ ] Each level has comprehensive README.md
- [ ] All demos run successfully
- [ ] Real integration (no mocks in showcase)
- [ ] **Level 3 (P2P) showcases pure Rust coordination** ⭐
- [ ] Performance benchmarks collected
- [ ] Video walkthroughs recorded
- [ ] Documentation index complete

**Target:** End of Q1 2026  
**Current Progress:** ~34%  
**Priority:** **Level 3 (P2P coordination)** - This is our unique value proposition!

---

*"A showcase isn't just demos - it's proof that the vision works."*


