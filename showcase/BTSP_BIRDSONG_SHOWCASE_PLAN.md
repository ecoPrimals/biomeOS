# 🎯 BiomeOS Local Showcase Buildout - BTSP & BirdSong P2P Focus

**Date**: December 31, 2025  
**Primary Goal**: BTSP tunnels and BirdSong P2P utilizing BearDog + Songbird  
**Status**: Planning → Execution  

---

## 🔍 **CURRENT STATE ANALYSIS**

### Available Binaries (`../../primalBins/`)
✅ **beardog** - Security & encryption CLI  
✅ **songbird-orchestrator** - P2P coordination (mDNS/UDP)  
✅ **nestgate** - Storage primal  
✅ **nestgate-client** - Storage client  
✅ **toadstool-cli** - Compute CLI  
✅ **squirrel-cli** - AI routing CLI  
✅ **petal-tongue** - UI (v0.1.0-stable)  
✅ **loamspine-service** - Embed runtime  
✅ **rhizocrypt-service** - Crypto service  
✅ **sweet-grass-service** - State sync  

### Existing Showcases Review

#### Phase1 Primals (Excellent work!)
**Songbird** (`phase1/songbird/showcase/`):
- ✅ 15 showcase categories  
- ✅ **Multi-tower federation working** (showcase/02-federation/)
- ✅ Inter-primal coordination (showcase/03-inter-primal/)
- ✅ **BearDog integration** (showcase/13-beardog-integration/)
- ✅ Capability-based discovery (showcase/14-capability-based-discovery/)
- ✅ Physical genesis patterns (showcase/14-physical-genesis/)

**Toadstool** (`phase1/toadstool/showcase/`):
- ✅ GPU compute demos working
- ✅ Inter-primal patterns (showcase/inter-primal/)
- ✅ **NestGate integration** (showcase/nestgate-compute/)
- ✅ Real-world ML pipelines

**NestGate** (`phase1/nestgate/showcase/`):
- ✅ **100% local showcase complete**
- ✅ Live multi-node demos
- ✅ Ecosystem integration patterns
- ✅ Real NCBI data management

**BearDog** (`phase1/beardog/showcase/`):
- ✅ **BTSP tunnel demos** (showcase/00-local-primal/06-btsp-tunnel/)
- ✅ **Songbird-BTSP integration** (showcase/02-ecosystem-integration/01-songbird-btsp/)
- ✅ Key lineage & entropy mixing
- ✅ Multi-primal workflows

#### BiomeOS Local (`phase2/biomeOS/showcase/`)
**Current Structure**:
- `00-local-capabilities/` - BiomeOS local features
- `01-single-primal/` - Individual primal demos
- `02-primal-pairs/` - Primal coordination patterns
- **`03-p2p-coordination/`** - **BTSP & BirdSong focus** 🎯

**Status**: Good foundation, needs **BTSP/BirdSong expansion**

---

## 🎯 **PRIMARY GOAL: BTSP & BIRDSONG P2P**

### What We're Building
**Showcase Category**: `showcase/03-p2p-coordination/`

**Focus Demos** (Priority Order):
1. **01-btsp-tunnel-coordination** 🔥 - BTSP tunnel lifecycle (BearDog + Songbird)
2. **02-birdsong-encryption** 🔥 - Encrypted P2P messaging
3. **03-lineage-gated-relay** - Sovereign routing with lineage proofs
4. **04-multi-tower-federation** - Geographic distribution
5. **05-full-ecosystem-integration** - All primals orchestrated

### Why BTSP & BirdSong?
- **BTSP**: BearDog Transport Security Protocol - Sovereign tunnels
- **BirdSong**: Songbird's P2P protocol - Encrypted peer discovery
- **BearDog**: Provides entropy, keys, encryption
- **Songbird**: Provides peer discovery, coordination, relay
- **BiomeOS**: Orchestrates both into seamless P2P network

---

## 📋 **SHOWCASE BUILDOUT PLAN**

### Phase 1: Foundation (Days 1-2) ⚡ **START HERE**

#### 1.1 Create Common Discovery Library
**File**: `showcase/common/discovery.sh`

**Purpose**: Runtime primal discovery (zero hardcoding)

```bash
#!/usr/bin/env bash
# BiomeOS Runtime Discovery Library

discover_beardog() {
    # Find beardog binary
    if [ -f "../../primalBins/beardog" ]; then
        echo "../../primalBins/beardog"
    else
        echo "$(which beardog 2>/dev/null)"
    fi
}

discover_songbird() {
    # Find songbird orchestrator
    if [ -f "../../primalBins/songbird-orchestrator" ]; then
        echo "../../primalBins/songbird-orchestrator"
    else
        echo "$(which songbird-orchestrator 2>/dev/null)"
    fi
}

discover_nestgate() {
    # Check for running NestGate instance via mDNS or port
    curl -s http://localhost:9020/health &>/dev/null && echo "http://localhost:9020" || echo ""
}

check_primal_health() {
    local primal_name=$1
    local endpoint=$2
    curl -s "${endpoint}/health" | jq -r '.status' 2>/dev/null || echo "unknown"
}

wait_for_primal() {
    local name=$1
    local endpoint=$2
    local max_wait=30
    
    echo "⏳ Waiting for $name at $endpoint..."
    for i in $(seq 1 $max_wait); do
        if curl -s "${endpoint}/health" &>/dev/null; then
            echo "✅ $name is ready!"
            return 0
        fi
        sleep 1
    done
    echo "❌ $name failed to start within ${max_wait}s"
    return 1
}
```

#### 1.2 Update Local Capabilities Showcase
**Directory**: `showcase/00-local-capabilities/`

**Action**: Ensure demonstrates BiomeOS substrate features (not primal features)

**Focus**:
- Manifest parsing
- Capability matching
- Discovery patterns
- Sovereignty guardian
- Client registry

#### 1.3 Create NestGate Local Showcase
**Directory**: `showcase/01-nestgate/`

**Purpose**: Show NestGate as "local star" - storage sovereignty

**Demos** (copy patterns from `phase1/nestgate/showcase/`):
1. `01-sovereign-storage/` - Local storage with sovereignty
2. `02-zfs-snapshots/` - Snapshot management
3. `03-lineage-collaboration/` - Shared storage with lineage
4. `04-federation-replication/` - Multi-node storage
5. `05-benchscale-validation/` - Deployment validation

**Scripts**: Use real `../../primalBins/nestgate` binary

---

### Phase 2: BTSP Core (Days 3-4) 🔥 **PRIMARY FOCUS**

#### 2.1 Demo: BTSP Tunnel Coordination
**Directory**: `showcase/03-p2p-coordination/01-btsp-tunnel-coordination/`

**What It Shows**:
- BearDog generates tunnel keys
- Songbird discovers peers
- BiomeOS coordinates tunnel lifecycle
- Health monitoring & recovery
- Graceful degradation

**Scripts**:
```bash
showcase/03-p2p-coordination/01-btsp-tunnel-coordination/
├── README.md                    # Full documentation
├── demo.sh                      # Main demo script
├── setup-peers.sh               # Start peer nodes
├── establish-tunnel.sh          # Create BTSP tunnel
├── monitor-tunnel.sh            # Health monitoring
├── simulate-degradation.sh      # Test recovery
├── teardown.sh                  # Clean shutdown
├── topology.yaml                # BenchScale config
└── validate.sh                  # E2E validation
```

**demo.sh** (outline):
```bash
#!/usr/bin/env bash
set -euo pipefail
source ../../common/discovery.sh

echo "🔐 BTSP Tunnel Coordination Demo"
echo "================================"

# 1. Discover primals
echo "📡 Discovering primals..."
BEARDOG=$(discover_beardog)
SONGBIRD=$(discover_songbird)

[ -z "$BEARDOG" ] && echo "❌ BearDog not found" && exit 1
[ -z "$SONGBIRD" ] && echo "❌ Songbird not found" && exit 1

echo "✅ BearDog: $BEARDOG"
echo "✅ Songbird: $SONGBIRD"

# 2. Generate tunnel keys with BearDog
echo ""
echo "🔑 Generating tunnel keys..."
TUNNEL_KEY=$($BEARDOG keygen --purpose btsp-tunnel --constraints "tunnel:p2p")
echo "✅ Tunnel key: ${TUNNEL_KEY:0:16}..."

# 3. Start Songbird for peer discovery
echo ""
echo "🎼 Starting Songbird orchestrator..."
$SONGBIRD start-tower --mode local &
SONGBIRD_PID=$!
sleep 2

# 4. Establish BTSP tunnel
echo ""
echo "🌉 Establishing BTSP tunnel..."
# Use BiomeOS to coordinate (or direct beardog + songbird integration)
./establish-tunnel.sh "$TUNNEL_KEY"

# 5. Monitor tunnel health
echo ""
echo "📊 Monitoring tunnel health..."
./monitor-tunnel.sh &
MONITOR_PID=$!

# 6. Simulate degradation & recovery
echo ""
echo "⚠️  Simulating network degradation..."
sleep 5
./simulate-degradation.sh

echo ""
echo "🔄 Validating automatic recovery..."
sleep 5

# 7. Show tunnel stats
echo ""
echo "📈 Tunnel Statistics:"
# Query tunnel metrics

# 8. Cleanup
echo ""
echo "🧹 Cleaning up..."
kill $MONITOR_PID $SONGBIRD_PID 2>/dev/null || true
./teardown.sh

echo ""
echo "✅ Demo complete!"
echo "   - BTSP tunnel established"
echo "   - Health monitoring active"
echo "   - Recovery validated"
echo "   - Zero hardcoded endpoints"
```

**Key Patterns**:
- ✅ Runtime discovery (no hardcoding)
- ✅ Real binaries (no mocks)
- ✅ Health monitoring
- ✅ Graceful degradation
- ✅ BenchScale topology for deployment

#### 2.2 Demo: BirdSong Encryption
**Directory**: `showcase/03-p2p-coordination/02-birdsong-encryption/`

**What It Shows**:
- End-to-end encrypted P2P messaging
- Key exchange via BearDog
- Message routing via Songbird
- Perfect forward secrecy
- Audit trail

**Focus**: Copy patterns from `phase1/beardog/showcase/02-ecosystem-integration/01-songbird-btsp/`

---

### Phase 3: Advanced P2P (Days 5-7)

#### 3.1 Demo: Lineage-Gated Relay
**Directory**: `showcase/03-p2p-coordination/03-lineage-gated-relay/`

**What It Shows**:
- Data routing with lineage verification
- Multi-hop relay with sovereignty
- BearDog lineage proofs at each hop
- Songbird coordinates routing
- Audit trail preservation

#### 3.2 Demo: Multi-Tower Federation
**Directory**: `showcase/03-p2p-coordination/04-multi-tower-federation/`

**What It Shows**:
- Geographic distribution (multiple Songbird towers)
- Cross-tower BTSP tunnels
- Automatic peer discovery across towers
- Load distribution
- Failover & recovery

**Inspiration**: `phase1/songbird/showcase/02-federation/` (working multi-tower)

#### 3.3 Demo: Full Ecosystem Integration
**Directory**: `showcase/03-p2p-coordination/05-full-ecosystem-integration/`

**What It Shows**:
- All 4 primals: NestGate + BearDog + Songbird + Toadstool
- BTSP tunnels for all communication
- Encrypted storage (NestGate + BearDog)
- Distributed compute (Toadstool via Songbird)
- Complete sovereign workflow

---

## 🔧 **EXECUTION STRATEGY**

### Day 1: Foundation Setup
1. ✅ Review all existing showcases (DONE - analysis above)
2. ⚡ Create `common/discovery.sh` library
3. ⚡ Update `00-local-capabilities/` demos
4. ⚡ Create `01-nestgate/` local showcase structure

### Day 2: BTSP Core (Priority 1) 🔥
1. ⚡ Build `01-btsp-tunnel-coordination/demo.sh`
2. ⚡ Integrate real BearDog + Songbird binaries
3. ⚡ Create establish-tunnel.sh script
4. ⚡ Create monitoring scripts
5. ⚡ Test end-to-end workflow

### Day 3: BTSP Polish
1. Add recovery scenarios
2. Create BenchScale topology
3. Write comprehensive README
4. Add validation tests
5. Document gaps discovered

### Day 4: BirdSong Encryption
1. Build `02-birdsong-encryption/demo.sh`
2. Implement encrypted messaging
3. Add key rotation demo
4. Create audit trail logging
5. Test with real primals

### Days 5-7: Advanced Demos
1. Lineage-gated relay
2. Multi-tower federation
3. Full ecosystem integration
4. BenchScale validation
5. Complete documentation

---

## 📖 **PATTERNS TO COPY**

### From Songbird Showcase
**Directory**: `phase1/songbird/showcase/13-beardog-integration/`

**Files to Adapt**:
- `03-btsp-live-integration-test.sh` → Our tunnel demo
- `04-birdsong-discovery-test.sh` → Our discovery patterns
- `05-full-p2p-test-suite.sh` → Our E2E tests

**Key Learnings**:
- Songbird's multi-tower federation is proven ✅
- BearDog-Songbird integration patterns exist ✅
- Discovery patterns are solid ✅

### From BearDog Showcase
**Directory**: `phase1/beardog/showcase/00-local-primal/06-btsp-tunnel/`

**Files to Adapt**:
- Full working BTSP tunnel example
- Key generation patterns
- Entropy mixing demonstrations
- Audit trail logging

**Key Learnings**:
- BTSP is production-ready ✅
- Key lineage works ✅
- Entropy hierarchy is sound ✅

### From Toadstool Showcase
**Directory**: `phase1/toadstool/showcase/inter-primal/`

**Files to Adapt**:
- Songbird distributed compute patterns
- NestGate ML pipeline integration
- Cross-primal coordination

**Key Learnings**:
- Toadstool + Songbird coordination works ✅
- Compute distribution is proven ✅
- NestGate persistence integration exists ✅

### From NestGate Showcase
**Directory**: `phase1/nestgate/showcase/01_nestgate_songbird_live/`

**Files to Adapt**:
- Live multi-node patterns
- Federation replication
- Real data management

**Key Learnings**:
- NestGate is production-ready ✅
- Multi-node replication works ✅
- Ecosystem integration proven ✅

---

## 🎯 **SUCCESS METRICS**

### Week 1 (BTSP Foundation)
- [ ] `common/discovery.sh` created & tested
- [ ] `01-nestgate/` local showcase complete (5 demos)
- [ ] `01-btsp-tunnel-coordination/` working end-to-end
- [ ] `02-birdsong-encryption/` functional
- [ ] All demos use real binaries (NO MOCKS)

### Week 2 (Advanced P2P)
- [ ] `03-lineage-gated-relay/` functional
- [ ] `04-multi-tower-federation/` working across nodes
- [ ] `05-full-ecosystem-integration/` complete workflow
- [ ] BenchScale topologies for all demos
- [ ] Validation tests passing

### Week 3 (Polish & Validation)
- [ ] All demos documented (README + comments)
- [ ] BenchScale validation on real VMs
- [ ] Performance benchmarks collected
- [ ] Gap reports generated
- [ ] Showcase presentation ready

---

## 🚫 **ANTI-PATTERNS TO AVOID**

### From Audit Learnings
❌ **NO MOCKS** - Use real binaries only  
❌ **NO HARDCODING** - Runtime discovery always  
❌ **NO PRIMAL NAME COUPLING** - Capabilities only  
❌ **NO FAKE METRICS** - Real performance data  
❌ **NO "IT WOULD WORK IF..."** - It works or document gap  

### Quality Standards
✅ Every demo has `topology.yaml` (BenchScale deployable)  
✅ Every demo has `validate.sh` (E2E testable)  
✅ Every demo has README with architecture diagram  
✅ Every demo documents gaps discovered  
✅ Every demo uses `common/discovery.sh`  

---

## 📁 **FILE STRUCTURE**

```
showcase/
├── common/
│   └── discovery.sh                 ⚡ NEW - Runtime discovery library
│
├── 00-local-capabilities/          ✅ EXISTS - Update to use discovery.sh
│   ├── 01-manifest-parsing.sh
│   ├── 02-capability-matching.sh
│   └── ...
│
├── 01-nestgate/                     ⚡ NEW - NestGate local showcase
│   ├── 01-sovereign-storage/
│   ├── 02-zfs-snapshots/
│   ├── 03-lineage-collaboration/
│   ├── 04-federation-replication/
│   └── 05-benchscale-validation/
│
├── 02-birdsong-p2p/                ✅ EXISTS - Basic P2P primitives
│   └── ... (existing structure)
│
├── 03-p2p-coordination/            🔥 PRIMARY FOCUS - BTSP & BirdSong
│   ├── 01-btsp-tunnel-coordination/    ⚡ BUILD FIRST (Days 2-3)
│   │   ├── README.md
│   │   ├── demo.sh
│   │   ├── setup-peers.sh
│   │   ├── establish-tunnel.sh
│   │   ├── monitor-tunnel.sh
│   │   ├── simulate-degradation.sh
│   │   ├── teardown.sh
│   │   ├── topology.yaml
│   │   └── validate.sh
│   │
│   ├── 02-birdsong-encryption/          ⚡ BUILD SECOND (Day 4)
│   │   ├── README.md
│   │   ├── demo.sh
│   │   ├── establish-channel.sh
│   │   ├── send-encrypted.sh
│   │   ├── rotate-keys.sh
│   │   ├── verify-security.sh
│   │   ├── topology.yaml
│   │   └── validate.sh
│   │
│   ├── 03-lineage-gated-relay/          (Day 5)
│   ├── 04-multi-tower-federation/       (Days 6-7)
│   └── 05-full-ecosystem-integration/   (Days 6-7)
│
└── README.md                        ✅ EXISTS - Update with BTSP focus
```

---

## 🎓 **TECHNICAL PATTERNS**

### Pattern 1: Runtime Discovery
```bash
# DON'T DO THIS ❌
BEARDOG="/usr/local/bin/beardog"
SONGBIRD="http://localhost:8080"

# DO THIS ✅
source ../common/discovery.sh
BEARDOG=$(discover_beardog)
SONGBIRD=$(discover_songbird)
```

### Pattern 2: Real Binaries
```bash
# DON'T DO THIS ❌
mock_beardog_response() {
    echo '{"status":"ok"}'
}

# DO THIS ✅
$BEARDOG keygen --purpose btsp-tunnel
```

### Pattern 3: Health Monitoring
```bash
# Always check primal health
wait_for_primal "Songbird" "$SONGBIRD_ENDPOINT"
check_primal_health "BearDog" "$BEARDOG_CLI"
```

### Pattern 4: Graceful Degradation
```bash
# Handle failures gracefully
if ! establish_tunnel "$KEY"; then
    echo "⚠️  Tunnel establishment failed"
    echo "   Attempting recovery..."
    retry_with_backoff establish_tunnel "$KEY" 3
fi
```

---

## 📊 **PRIORITY MATRIX**

### Must Have (Week 1) 🔥
1. **common/discovery.sh** - Foundation for everything
2. **01-btsp-tunnel-coordination/** - Primary goal demo
3. **02-birdsong-encryption/** - Primary goal demo
4. **01-nestgate/** - Local star showcase

### Should Have (Week 2)
5. **03-lineage-gated-relay/** - Sovereignty demonstration
6. **04-multi-tower-federation/** - Distribution proof
7. BenchScale topologies for all demos

### Nice to Have (Week 3)
8. **05-full-ecosystem-integration/** - Complete workflow
9. Performance benchmarks & analysis
10. Presentation materials

---

## 🚀 **IMMEDIATE NEXT STEPS**

### Today (Day 1)
1. ⚡ Create `showcase/common/discovery.sh`
2. ⚡ Test discovery with existing binaries
3. ⚡ Create `showcase/01-nestgate/README.md` structure
4. ⚡ Begin `showcase/03-p2p-coordination/01-btsp-tunnel-coordination/`

### Tomorrow (Day 2)
1. ⚡ Complete BTSP tunnel demo script
2. ⚡ Test with real BearDog + Songbird
3. ⚡ Add health monitoring
4. ⚡ Document gaps discovered

### Day 3
1. Polish BTSP tunnel demo
2. Add recovery scenarios
3. Create BenchScale topology
4. Begin BirdSong encryption demo

---

## ✅ **READY TO BUILD**

**Status**: ✅ Planning complete  
**Binaries**: ✅ All available in `../../primalBins/`  
**Patterns**: ✅ Proven in phase1 showcases  
**Architecture**: ✅ BTSP + BirdSong documented  
**Goal**: ✅ Clear (BTSP tunnels + BirdSong P2P)  

**Next Action**: Create `common/discovery.sh` and begin Day 1 tasks

---

🔐 **Let's build the most sovereignty-respecting P2P showcase in existence!** 🌱

