# Ecosystem Harvest & Deployment Plan - January 15, 2026

**Status**: Ready for Review & Harvest  
**Goal**: Harvest updated primal binaries → plasmidBin/ → Deploy via Neural API  
**Date**: 2026-01-15

---

## 🎯 Executive Summary

Three major primals have evolved in parallel with biomeOS:
1. **Squirrel** (phase1): Meta-AI routing, MCP integration, 99% pure Rust
2. **ToadStool** (phase1): Universal compute, collaborative intelligence
3. **PetalTongue** (phase2): Universal benchTop UI (v2.3.0)

**Next Steps**:
1. Review their updates
2. Harvest fresh binaries to `plasmidBin/`
3. Deploy coordinated ecosystem via Neural API
4. Validate TRUE PRIMAL inter-primal interactions

---

## 📊 Current State Analysis

### Squirrel (phase1/squirrel)

**Latest Commit**: `43cc95e5` - Root Documentation Cleanup & Update (Jan 15)

**Recent Evolution** (Last 10 commits):
- ✅ Deep evolution session (99% pure Rust achieved!)
- ✅ Zero-copy optimization
- ✅ Native async traits
- ✅ Ecosystem API improvements
- ✅ Clippy pedantic compliance

**Key Features**:
- MCP server (Model Context Protocol)
- Multi-provider AI routing
- Plugin system
- Federation ready
- PrimalPulse integration

**Status**: **Production-Ready** ✅
- Grade: A++ (world-class)
- Tests: All passing
- Documentation: Comprehensive

**Current Binary**: `plasmidBin/primals/squirrel` (17MB, Jan 14 13:14)
**Action**: ✅ **Pull latest & rebuild**

---

### ToadStool (phase1/toadstool)

**Latest Commit**: `5ed874c6` - Clean and update root documentation (Jan 15)

**Recent Evolution** (Last 10 commits):
- ✅ Collaborative intelligence evolution
- ✅ Neural API adapter specification
- ✅ Builder patterns & modern Rust idioms
- ✅ Resource planning API
- ✅ Graph types evolution

**Key Features**:
- Universal compute platform
- GPU/CPU orchestration
- BearDog encrypted workloads
- Songbird discovery integration
- Squirrel MCP coordination
- Collaborative intelligence

**Status**: **Production-Ready** ✅
- Grade: A+ (97/100)
- Tests: Comprehensive coverage
- Documentation: Complete

**Current Binary**: `plasmidBin/primals/toadstool` (6.6MB, Jan 14 13:35)
**Action**: ✅ **Pull latest & rebuild**

---

### PetalTongue (phase2/petalTongue)

**Latest Commit**: `b230f51` - Universal benchTop + comprehensive analysis (Jan 15)

**Recent Evolution** (Last 10 commits):
- ✅ Universal benchTop (v2.3.0)
- ✅ Rich TUI implementation
- ✅ Comprehensive TUI testing (57 tests)
- ✅ Management views complete
- ✅ biomeOS handoff ready

**Key Features**:
- Universal benchTop UI (think PopOS Cosmic, Discord, Steam for primals!)
- Rich TUI (terminal UI)
- GUI support (framebuffer, display)
- Multi-modal visualization (audio, visual)
- Live graph rendering
- Inter-primal coordination UI
- Sensory capabilities (audio sonification)
- Deploy anywhere (desktop, embedded, headless)

**Status**: **Production-Ready** ✅
- Version: v2.3.0
- Tests: 57 TUI tests passing
- Documentation: Comprehensive

**Current Binaries**:
- `plasmidBin/primals/petal-tongue` (33MB, Jan 12 09:53)
- `plasmidBin/primals/petal-tongue-headless` (3.1MB, Jan 12 12:12)
- `plasmidBin/primals/petaltongue` (2.6MB, Jan 12 12:12)

**Action**: ✅ **Pull latest & rebuild**

---

## 🌳 Inter-Primal Relationships

### Existing Working Integrations

**Songbird ↔ BearDog** (Encrypted Discovery):
- Status: ✅ Working (Jan 3, 2026)
- Protocol: BirdSong v2 (ChaCha20-Poly1305)
- Auto-trust: Same family

**biomeOS ↔ All Primals** (Health Monitoring):
- Status: ✅ Ready
- Protocol: Unix socket JSON-RPC
- Capabilities: Health checks, discovery

### New Integrations (This Deployment)

**biomeOS ↔ Squirrel** (Meta-AI Coordination):
- Capability: `meta_ai`, `ai_routing`, `tool_orchestration`
- Use: AI-powered deployment decisions
- Protocol: MCP + Unix socket

**biomeOS ↔ ToadStool** (Compute Orchestration):
- Capability: `compute`, `gpu`, `collaborative_intelligence`
- Use: Resource allocation, workload scheduling
- Protocol: Unix socket JSON-RPC

**biomeOS ↔ PetalTongue** (Universal UI):
- Capability: `visualization`, `ui`, `sensory`
- Use: BenchTop interface for all primals
- Protocol: WebSocket + SSE (real-time events)

**Squirrel ↔ ToadStool** (AI-Driven Compute):
- Flow: Squirrel routes AI workloads → ToadStool executes
- Coordination: biomeOS orchestrates
- Discovery: Songbird announces, BearDog trusts

**PetalTongue ↔ All Primals** (Universal UI):
- Flow: PetalTongue visualizes all primal states
- Real-time: SSE/WebSocket from Neural API
- Control: User actions → biomeOS → Target primal

---

## 🔨 Harvest Plan

### Phase 1: Pull Latest Updates (15 minutes)

```bash
# 1. Squirrel
cd /home/eastgate/Development/ecoPrimals/phase1/squirrel
git pull
cargo test --workspace --lib  # Verify tests pass
cargo build --release

# 2. ToadStool
cd /home/eastgate/Development/ecoPrimals/phase1/toadstool
git pull
cargo test --workspace --lib  # Verify tests pass
cargo build --release

# 3. PetalTongue
cd /home/eastgate/Development/ecoPrimals/phase2/petalTongue
git pull
cargo test --workspace --lib  # Verify tests pass
cargo build --release          # GUI version
cargo build --release --bin petal-tongue-headless  # Headless
```

### Phase 2: Harvest Binaries (5 minutes)

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/primals

# Squirrel
cp /home/eastgate/Development/ecoPrimals/phase1/squirrel/target/release/squirrel .
chmod +x squirrel

# ToadStool
cp /home/eastgate/Development/ecoPrimals/phase1/toadstool/target/release/toadstool .
chmod +x toadstool

# PetalTongue (3 variants)
cp /home/eastgate/Development/ecoPrimals/phase2/petalTongue/target/release/petal-tongue .
cp /home/eastgate/Development/ecoPrimals/phase2/petalTongue/target/release/petal-tongue-headless .
cp /home/eastgate/Development/ecoPrimals/phase2/petalTongue/target/release/petaltongue .
chmod +x petal-tongue petal-tongue-headless petaltongue

# Verify
ls -lh
```

### Phase 3: Version Verification (5 minutes)

```bash
# Check versions and capabilities
./squirrel --version
./toadstool --version
./petal-tongue --version

# Verify Unix socket creation (quick smoke test)
# Start each primal briefly to ensure sockets work
./squirrel &
SQUIRREL_PID=$!
sleep 2
ls -l /tmp/squirrel-*.sock  # Should exist
kill $SQUIRREL_PID

# Repeat for others
```

---

## 🚀 Deployment Plan (Neural API Orchestration)

### Deployment Architecture

**BenchTop Vision**: PetalTongue as the "desktop" for primals
- Think: PopOS Cosmic (modern Linux desktop)
- Think: Discord (real-time communication UI)
- Think: Steam (game library + launcher)
- **For Primals**: Universal interface to discover, launch, monitor, control ALL primals

### Phase 1: NUCLEUS Enclave (Tower + Node + Nest)

**Goal**: Secure bootstrapping with genetic lineage

**Graph**: `graphs/nucleus_enclave.toml`

```toml
[graph]
id = "nucleus-enclave-deployment"
name = "NUCLEUS Enclave Bootstrap"
version = "1.0.0"
family_id = "{{ family_id }}"

[[nodes]]
id = "derive_tower_seed"
node_type = "crypto.derive_child_seed"
[nodes.config]
parent_seed_path = "/usb/family_seed.key"
child_name = "tower"
output_path = "/tmp/tower_seed.key"

[[nodes]]
id = "derive_node_seed"
node_type = "crypto.derive_child_seed"
depends_on = ["derive_tower_seed"]
[nodes.config]
parent_seed_path = "/usb/family_seed.key"
child_name = "node"
output_path = "/tmp/node_seed.key"

[[nodes]]
id = "derive_nest_seed"
node_type = "crypto.derive_child_seed"
depends_on = ["derive_node_seed"]
[nodes.config]
parent_seed_path = "/usb/family_seed.key"
child_name = "nest"
output_path = "/tmp/nest_seed.key"

[[nodes]]
id = "launch_tower"
node_type = "primal.launch"
depends_on = ["derive_tower_seed"]
[nodes.config]
primal_name = "songbird"  # Tower = Songbird (discovery)
seed_path = "/tmp/tower_seed.key"
socket_path = "/tmp/songbird-{{ family_id }}.sock"

[[nodes]]
id = "launch_node"
node_type = "primal.launch"
depends_on = ["derive_node_seed", "launch_tower"]
[nodes.config]
primal_name = "toadstool"  # Node = ToadStool (compute)
seed_path = "/tmp/node_seed.key"
socket_path = "/tmp/toadstool-{{ family_id }}.sock"

[[nodes]]
id = "launch_nest"
node_type = "primal.launch"
depends_on = ["derive_nest_seed", "launch_node"]
[nodes.config]
primal_name = "nestgate"  # Nest = NestGate (storage)
seed_path = "/tmp/nest_seed.key"
socket_path = "/tmp/nestgate-{{ family_id }}.sock"

[[nodes]]
id = "verify_nucleus"
node_type = "health.check_all"
depends_on = ["launch_tower", "launch_node", "launch_nest"]
[nodes.config]
timeout_seconds = 30
```

### Phase 2: Security & Intelligence Layer

**Graph**: `graphs/security_intelligence.toml`

```toml
[[nodes]]
id = "launch_beardog"
node_type = "primal.launch"
depends_on = ["verify_nucleus"]
[nodes.config]
primal_name = "beardog-server"
capability = "security"

[[nodes]]
id = "launch_squirrel"
node_type = "primal.launch"
depends_on = ["verify_nucleus"]
[nodes.config]
primal_name = "squirrel"
capability = "meta_ai"
```

### Phase 3: Universal UI (BenchTop)

**Graph**: `graphs/benchtop_deployment.toml`

```toml
[[nodes]]
id = "launch_petaltongue"
node_type = "primal.launch"
depends_on = ["launch_beardog", "launch_squirrel"]
[nodes.config]
primal_name = "petal-tongue"
capability = "visualization"
mode = "gui"  # or "tui" or "headless"

[[nodes]]
id = "configure_neural_api_events"
node_type = "neural_api.configure_events"
depends_on = ["launch_petaltongue"]
[nodes.config]
enable_websocket = true
enable_sse = true
broadcast_to = ["petal-tongue"]
```

### Phase 4: Full Ecosystem Validation

**Test Scenarios**:

1. **Capability Discovery** (TRUE PRIMAL validation):
   - PetalTongue discovers all running primals via Songbird
   - Queries capabilities via biomeOS
   - Displays in benchTop UI

2. **AI-Driven Compute** (Squirrel → ToadStool):
   - User requests AI workload in PetalTongue
   - Squirrel routes to appropriate provider
   - ToadStool executes compute
   - Results streamed back to PetalTongue

3. **Secure Coordination** (BearDog lineage):
   - All primals verify same family via BearDog
   - Encrypted communication (BirdSong)
   - Auto-trust within family

4. **Real-Time Monitoring** (Neural API → PetalTongue):
   - Primal health updates via SSE
   - Graph execution status via WebSocket
   - Live visualization in benchTop

---

## 📋 Execution Checklist

### Pre-Deployment

- [ ] Review Squirrel updates (README, STATUS)
- [ ] Review ToadStool updates (STATUS, docs)
- [ ] Review PetalTongue updates (SESSION_COMPLETE)
- [ ] Pull latest from all three repos
- [ ] Run tests for all three (`cargo test --workspace --lib`)

### Harvest

- [ ] Build Squirrel release binary
- [ ] Build ToadStool release binary
- [ ] Build PetalTongue binaries (3 variants)
- [ ] Copy all to `plasmidBin/primals/`
- [ ] Verify permissions (`chmod +x`)
- [ ] Smoke test (start/stop each primal)

### Deployment

- [ ] Create/update Neural API graphs:
  - [ ] `nucleus_enclave.toml`
  - [ ] `security_intelligence.toml`
  - [ ] `benchtop_deployment.toml`
- [ ] Execute NUCLEUS enclave deployment
- [ ] Verify 5-layer NUCLEUS protocol
- [ ] Launch security & intelligence layer
- [ ] Launch PetalTongue benchTop UI
- [ ] Configure real-time events

### Validation

- [ ] Test capability discovery (Songbird)
- [ ] Test genetic lineage (BearDog)
- [ ] Test AI routing (Squirrel)
- [ ] Test compute orchestration (ToadStool)
- [ ] Test real-time UI (PetalTongue)
- [ ] Test inter-primal coordination
- [ ] Document any issues

### Documentation

- [ ] Update `INTER_PRIMAL_INTERACTIONS.md`
- [ ] Document deployment process
- [ ] Capture benchTop screenshots/recordings
- [ ] Update biomeOS STATUS

---

## 🎊 Expected Outcomes

### Functional

1. **NUCLEUS Enclave Running**:
   - Tower (Songbird): Discovery, mesh coordination
   - Node (ToadStool): Compute orchestration
   - Nest (NestGate): Storage, persistence

2. **Full Ecosystem Operational**:
   - 9 primals coordinated by biomeOS
   - All capabilities advertised
   - Zero hardcoded dependencies

3. **BenchTop UI Active**:
   - PetalTongue displays all primals
   - Real-time status updates (SSE)
   - User can launch/stop/monitor primals
   - Visual + audio feedback

### Architectural

1. **TRUE PRIMAL Validated**:
   - Runtime discovery only
   - Capability-based coordination
   - No hardcoded endpoints

2. **Security Proven**:
   - Genetic lineage verified
   - Encrypted discovery working
   - Auto-trust within family

3. **Scalability Demonstrated**:
   - 9 primals coordinated
   - Real-time event streaming
   - Concurrent operations

---

## 🚨 Known Challenges & Mitigations

### Challenge 1: Binary Size

**Issue**: PetalTongue GUI is 33MB (includes graphics, UI)

**Mitigation**:
- Headless variant available (3.1MB)
- TUI variant available (2.6MB)
- Only deploy GUI where needed

### Challenge 2: Concurrent Startup

**Issue**: Starting 9 primals simultaneously may cause race conditions

**Mitigation**:
- Neural API graph enforces dependency order
- Health checks between phases
- Exponential backoff for discovery

### Challenge 3: Socket Path Conflicts

**Issue**: Multiple instances may conflict on socket paths

**Mitigation**:
- Family ID in socket paths (`/tmp/{primal}-{family}.sock`)
- Cleanup on shutdown
- `SO_REUSEADDR` for quick restart

---

## 📊 Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Deployment Time | < 30 seconds | Time from graph start → all healthy |
| Discovery Time | < 5 seconds | Songbird broadcast → primal registered |
| Health Check Latency | < 100ms | Unix socket JSON-RPC round-trip |
| UI Responsiveness | < 16ms | PetalTongue frame time (60 FPS) |
| Event Latency | < 500ms | Neural API event → PetalTongue display |
| Zero Failures | 100% | All primals start successfully |
| Zero Hardcoding | 100% | All discovery via Songbird/BearDog |

---

## 🎯 Next Steps After Deployment

1. **Week 1**: Stability testing
   - Run ecosystem for 24+ hours
   - Monitor for crashes, memory leaks
   - Collect metrics

2. **Week 2**: Phase 3 interactions
   - Implement `wateringHole/INTER_PRIMAL_INTERACTIONS.md` Phase 3
   - rhizoCrypt ↔ LoamSpine
   - Songbird federation

3. **Week 3**: Production hardening
   - Chaos testing
   - Network partition recovery
   - Automatic restart on failure

4. **Week 4**: User validation
   - Deploy to test users
   - Gather feedback on benchTop UI
   - Iterate on UX

---

## 📄 References

- **biomeOS**: `BIOMEOS_READINESS_ASSESSMENT.md`
- **Squirrel**: `phase1/squirrel/FINAL_SESSION_COMPLETE_JAN_15_2026.md`
- **ToadStool**: `phase1/toadstool/STATUS.md`
- **PetalTongue**: `phase2/petalTongue/SESSION_COMPLETE_JAN_15_2026.md`
- **Inter-Primal Plan**: `wateringHole/INTER_PRIMAL_INTERACTIONS.md`
- **Neural API**: `crates/biomeos-atomic-deploy/src/neural_api_server.rs`

---

**Status**: **READY FOR EXECUTION** ✅  
**Recommendation**: **Proceed with harvest and deployment!** 🚀

Let's build the benchTop! 🌳✨

