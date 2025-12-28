# 🌱 BiomeOS Showcase Buildout Plan - Dec 28, 2025

**Goal**: Build comprehensive local showcase demonstrating biomeOS as the **substrate** for deploying BirdSong P2P and BTSP, showcasing live primal orchestration with pure Rust infrastructure.

---

## 🎯 Executive Summary

### Current State
- ✅ **Excellent Phase1 Primal Showcases**: Songbird (multi-tower federation), ToadStool (GPU compute), BearDog (BTSP/BirdSong), NestGate (distributed storage), Squirrel (AI routing)
- ✅ **BiomeOS has P2P code**: `showcase/03-p2p-coordination/` with 5 demos
- ❌ **Gap**: biomeOS showcases not leveraging live primals
- ❌ **Gap**: Not demonstrating biomeOS AS the deployment substrate

### Vision
**BiomeOS = Operating System for Primals**
- Deploy primals as services (systemd integration)
- Orchestrate multi-primal workflows
- Enable BirdSong P2P/BTSP as biomeOS-native features
- Pure Rust, live infrastructure, no mocks

---

## 📊 Learning from Phase1 Primal Showcases

### What Works Brilliantly

#### 1. Songbird's Multi-Tower Federation ✨
**Location**: `phase1/songbird/showcase/02-federation/`
- **Success**: Live mesh with sub-millisecond latency (192.168.1.144 ↔ 192.168.1.134)
- **Key**: Real hardware, real network, real results
- **Documentation**: `FEDERATION_SUCCESS.md` shows operational mesh

**Lesson for BiomeOS**: Deploy Songbird towers via biomeOS, manage federation at biomeOS level

#### 2. Songbird's BearDog Integration (BTSP) 🔒
**Location**: `phase1/songbird/showcase/13-beardog-integration/`
- **Success**: Real P2P with genetic cryptography working
- **Key**: `05-full-p2p-test-suite.sh` orchestrates complete flow
- **Achievement**: TRUE P2P OPERATIONAL with real BearDog BTSP v0.9.0

**Lesson for BiomeOS**: BirdSong + BTSP should be biomeOS-native, deployed as substrate services

#### 3. ToadStool's Live Integration 🍄
**Location**: `phase1/toadstool/showcase/inter-primal/`
- **Success**: LIVE ONLY policy - no mocks, real primal coordination
- **Key**: Runtime discovery, capability-based routing
- **Philosophy**: "If the demo can run without the other primal, it's not a showcase"

**Lesson for BiomeOS**: All showcases must use REAL primals from `primals/` directory

#### 4. NestGate's Progressive Learning Path 🏰
**Location**: `phase1/nestgate/showcase/00_START_HERE.md`
- **Success**: 30 demos organized by complexity (5 min → 2+ hours)
- **Key**: Clear learning paths (Level 1-5), quick start in 5 minutes
- **Achievement**: LAN mesh join demo - friend contributes storage in <5 min

**Lesson for BiomeOS**: Progressive showcase from "hello world" to "full ecosystem"

---

## 🏗️ BiomeOS Showcase Architecture

### Core Philosophy
**BiomeOS is the SUBSTRATE, not just an orchestrator**

```
┌─────────────────────────────────────────────────────────┐
│                      USER/DEVELOPER                      │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
                  ┌──────────────┐
                  │   biomeOS    │ ← Operating System Layer
                  │  (Substrate) │    - Deploys primals
                  │              │    - Manages lifecycle
                  │              │    - Orchestrates workflows
                  └──────┬───────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
        ▼                ▼                ▼
   ┌─────────┐     ┌─────────┐     ┌─────────┐
   │Songbird │     │ BearDog │     │NestGate │ ← Primal Services
   │  P2P    │     │  BTSP   │     │ Storage │    (biomeOS manages)
   │Discovery│     │Birdsong │     │ Lineage │
   └─────────┘     └─────────┘     └─────────┘
        ▲                ▲                ▲
        └────────────────┴────────────────┘
                biomeOS Coordinates
```

### Key Differentiation
- **Phase1 Showcases**: Individual primal capabilities
- **BiomeOS Showcase**: How biomeOS deploys and orchestrates ALL primals together

---

## 📁 Proposed Showcase Structure

### Reorganization Plan

```
showcase/
├── 00_START_HERE.md                    ← Main entry point (NEW)
├── NO_MOCKS_POLICY.md                  ← Keep (enforced)
│
├── 00-local-primal-substrate/          ← NEW: Foundation layer
│   ├── 01-hello-biomeos/
│   ├── 02-deploy-single-primal/
│   ├── 03-manage-primal-lifecycle/
│   ├── 04-multi-primal-coordination/
│   └── 05-primal-health-monitoring/
│
├── 01-nestgate-showcase/               ← NEW: Our local star
│   ├── 01-hello-nestgate/
│   ├── 02-lineage-tracking/
│   ├── 03-sovereign-storage/
│   ├── 04-multi-node-federation/
│   └── 05-nestgate-biomeos-integration/
│
├── 02-birdsong-p2p-deployment/         ← ENHANCED: BirdSong as substrate feature
│   ├── 01-deploy-songbird-via-biomeos/
│   ├── 02-deploy-beardog-btsp/
│   ├── 03-birdsong-encrypted-discovery/
│   ├── 04-lineage-gated-relay/
│   └── 05-multi-tower-p2p-mesh/
│
├── 03-p2p-coordination/                ← KEEP: Current demos (but enhance)
│   ├── 01-btsp-tunnel-coordination/
│   ├── 02-birdsong-encryption/
│   ├── 03-lineage-gated-relay/
│   ├── 04-multi-tower-federation/
│   └── 05-full-ecosystem-integration/
│
├── 04-primal-ecosystem-integration/    ← NEW: Multi-primal workflows
│   ├── 01-songbird-nestgate-mesh/
│   ├── 02-beardog-nestgate-encryption/
│   ├── 03-toadstool-distributed-compute/
│   ├── 04-squirrel-ai-orchestration/
│   └── 05-full-ecosystem-ml-pipeline/
│
├── 05-production-deployment/           ← NEW: Real-world scenarios
│   ├── 01-systemd-service-deployment/
│   ├── 02-multi-machine-federation/
│   ├── 03-docker-compose-stack/
│   ├── 04-kubernetes-operator/
│   └── 05-home-lab-deployment/
│
├── api-adapter-test-results/          ← KEEP: Integration validation
├── archive/                            ← KEEP: Historical reference
└── common/                             ← NEW: Shared utilities
    ├── capability-discovery.sh
    ├── primal-health-check.sh
    ├── deploy-primal.sh
    └── gap-reporter.sh
```

---

## 🎯 Priority Showcases (MVP)

### Phase 1: Foundation (Week 1) - 12 hours

#### 00-01: Hello BiomeOS (1 hour)
**File**: `00-local-primal-substrate/01-hello-biomeos/demo.sh`

**What it demonstrates**:
```bash
#!/usr/bin/env bash
# 00-01: Hello BiomeOS - First Steps

echo "🌱 BiomeOS: Operating System for Primals"
echo ""
echo "Step 1: Check biomeOS installation"
cargo --version || { echo "Install Rust first!"; exit 1; }

echo "Step 2: Check available primal binaries"
ls -lh ../../primals/

echo "Step 3: BiomeOS can discover and manage these primals"
echo ""
echo "Available capabilities:"
echo "  🎵 Songbird (24MB) - P2P discovery and federation"
echo "  🐻 BearDog (4.6MB) - BTSP tunnels and BirdSong encryption"
echo "  🏰 NestGate (3.4MB) - Sovereign storage with lineage"
echo "  🍄 ToadStool (20MB) - GPU compute and distributed execution"
echo "  🐿️ Squirrel (2.9MB) - AI routing and MCP agents"
echo ""
echo "✅ BiomeOS ready to orchestrate!"
```

**Success**: User understands biomeOS manages primals as services

#### 00-02: Deploy Single Primal (2 hours)
**File**: `00-local-primal-substrate/02-deploy-single-primal/demo.sh`

**What it demonstrates**:
```bash
#!/usr/bin/env bash
# 00-02: Deploy NestGate via BiomeOS

source ../common/deploy-primal.sh
source ../common/primal-health-check.sh

echo "🌱 Deploying NestGate via BiomeOS..."

# BiomeOS deploys primal as managed service
deploy_primal \
  --name="nestgate" \
  --binary="../../primals/nestgate" \
  --port=9020 \
  --config="configs/nestgate.yaml"

# BiomeOS monitors health
wait_for_healthy "nestgate" "http://localhost:9020/health" 30

echo "✅ NestGate deployed and healthy!"
echo ""
echo "Try: curl http://localhost:9020/health"
echo "Try: curl http://localhost:9020/api/v1/datasets"
```

**Success**: User sees biomeOS deploying and managing a primal

#### 01-01: Hello NestGate (2 hours)
**File**: `01-nestgate-showcase/01-hello-nestgate/demo.sh`

**What it demonstrates**:
```bash
#!/usr/bin/env bash
# 01-01: Hello NestGate - Our Local Storage Star

echo "🏰 NestGate: Sovereign Storage with Lineage"
echo ""

# Ensure NestGate running
NESTGATE_URL="http://localhost:9020"
curl -f "$NESTGATE_URL/health" || {
  echo "❌ NestGate not running"
  echo "Deploy first: cd ../00-local-primal-substrate/02-deploy-single-primal && ./demo.sh"
  exit 1
}

echo "✅ NestGate is healthy"
echo ""

# Store data
echo "📝 Storing data with lineage..."
curl -X POST "$NESTGATE_URL/api/v1/datasets" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "hello-biomeos",
    "data": "First storage via biomeOS!",
    "lineage": {
      "who": "showcase-demo",
      "what": "hello world",
      "when": "2025-12-28T00:00:00Z",
      "why": "demonstration"
    }
  }'

echo ""
echo "📖 Retrieving with lineage..."
curl "$NESTGATE_URL/api/v1/datasets/hello-biomeos" | jq

echo ""
echo "✅ NestGate working!"
echo "   ✓ Sovereign storage (your data)"
echo "   ✓ Lineage tracking (WHO/WHAT/WHEN/WHY)"
echo "   ✓ Ready for multi-node federation"
```

**Success**: User understands NestGate's value proposition

#### 02-01: Deploy Songbird via BiomeOS (3 hours)
**File**: `02-birdsong-p2p-deployment/01-deploy-songbird-via-biomeos/demo.sh`

**What it demonstrates**:
```bash
#!/usr/bin/env bash
# 02-01: Deploy Songbird via BiomeOS

source ../common/deploy-primal.sh

echo "🌱 BiomeOS deploying Songbird P2P discovery..."

# BiomeOS manages Songbird as a service
deploy_primal \
  --name="songbird" \
  --binary="../../primals/songbird" \
  --port=9000 \
  --federation=true \
  --config="configs/songbird-tower-local.yaml"

wait_for_healthy "songbird" "http://localhost:9000/health" 30

echo "✅ Songbird operational!"
echo ""
echo "Songbird provides:"
echo "  🔍 Service discovery (mDNS, broadcast, multicast)"
echo "  🎵 BirdSong protocol (encrypted discovery)"
echo "  🌐 Multi-tower federation"
echo "  📡 Universal Port Authority (UPA)"
echo ""
echo "Check: curl http://localhost:9000/api/v1/services"
```

**Success**: Songbird deployed as biomeOS-managed service

#### 02-02: Deploy BearDog BTSP (4 hours)
**File**: `02-birdsong-p2p-deployment/02-deploy-beardog-btsp/demo.sh`

**What it demonstrates**:
```bash
#!/usr/bin/env bash
# 02-02: Deploy BearDog BTSP via BiomeOS

source ../common/deploy-primal.sh

echo "🌱 BiomeOS deploying BearDog BTSP..."
echo "   🐻 Genetic cryptography"
echo "   🔒 BTSP tunneling"
echo "   🎵 BirdSong encryption"
echo ""

# BiomeOS configures BearDog with Songbird integration
deploy_primal \
  --name="beardog" \
  --binary="../../primals/beardog" \
  --port=9040 \
  --btsp-port=9041 \
  --config="configs/beardog-btsp.yaml" \
  --env="SONGBIRD_ENDPOINT=http://localhost:9000"

wait_for_healthy "beardog" "http://localhost:9040/health" 30

echo "✅ BearDog BTSP operational!"
echo ""
echo "BTSP Tunnel Test:"
# Test tunnel establishment
curl -X POST "http://localhost:9040/api/v1/btsp/tunnels" \
  -H "Content-Type: application/json" \
  -d '{
    "target": "localhost:9041",
    "lineage_proof": "demo-lineage"
  }' | jq

echo ""
echo "✅ BTSP tunnel established!"
echo "   ✓ Genetic cryptography active"
echo "   ✓ Lineage-based trust"
echo "   ✓ Ready for BirdSong"
```

**Success**: BTSP deployed, tunnel working, integrated with Songbird

### Phase 2: Integration (Week 2) - 16 hours

#### 02-03: BirdSong Encrypted Discovery (4 hours)
**Building on Songbird's `showcase/13-beardog-integration/04-birdsong-discovery-test.sh`**

**What it demonstrates**:
- Songbird + BearDog = BirdSong protocol
- Family can decrypt broadcasts
- Others see only noise
- Privacy-preserving service discovery

#### 02-04: Lineage-Gated Relay (4 hours)
**Building on Songbird's `showcase/13-beardog-integration/` relay concepts**

**What it demonstrates**:
- NAT traversal via lineage
- Ancestor nodes volunteer as relays
- No central TURN servers
- Cryptographic family trust

#### 02-05: Multi-Tower P2P Mesh (4 hours)
**Building on Songbird's `showcase/02-federation/FEDERATION_SUCCESS.md`**

**What it demonstrates**:
- Multiple biomeOS instances
- Cross-tower P2P coordination
- Sub-millisecond federation
- Geographic distribution

#### 04-01: Songbird-NestGate Mesh (2 hours)
**Building on NestGate's federation demos**

**What it demonstrates**:
- Service discovery + distributed storage
- Automatic data routing
- CALM federation principles
- Lineage across mesh

#### 04-02: BearDog-NestGate Encryption (2 hours)
**Building on NestGate's encryption showcase**

**What it demonstrates**:
- Encrypted storage at rest
- Keys never leave BearDog
- NestGate stores encrypted blobs
- Full lineage of encrypted operations

### Phase 3: Production (Week 3) - 12 hours

#### 05-01: Systemd Service Deployment (4 hours)
**What it demonstrates**:
- BiomeOS as systemd service manager
- Primal services with auto-restart
- Logging and monitoring
- Production-grade lifecycle

#### 05-02: Multi-Machine Federation (4 hours)
**Building on Songbird's multi-machine setup**

**What it demonstrates**:
- BiomeOS on multiple physical machines
- Cross-machine primal coordination
- Real network performance
- Production topology

#### 05-03: Docker Compose Stack (4 hours)
**What it demonstrates**:
- BiomeOS in containers
- Primal services as Docker services
- Docker networking
- Easy deployment for developers

---

## 🛠️ Implementation Strategy

### Week 1: Foundation (MVP)
**Goal**: 5 working demos showing biomeOS deploying and managing primals

**Deliverables**:
1. `00-01-hello-biomeos` - Introduction
2. `00-02-deploy-single-primal` - NestGate deployment
3. `01-01-hello-nestgate` - NestGate usage
4. `02-01-deploy-songbird` - Songbird deployment
5. `02-02-deploy-beardog-btsp` - BTSP deployment

**Success**: User can deploy primals via biomeOS, see BTSP working

### Week 2: Integration
**Goal**: BirdSong P2P and multi-primal workflows

**Deliverables**:
1. `02-03-birdsong-encrypted-discovery` - P2P privacy
2. `02-04-lineage-gated-relay` - NAT traversal
3. `02-05-multi-tower-p2p-mesh` - Federation
4. `04-01-songbird-nestgate-mesh` - Storage federation
5. `04-02-beardog-nestgate-encryption` - Encrypted storage

**Success**: Full P2P ecosystem working with encryption

### Week 3: Production
**Goal**: Production deployment patterns

**Deliverables**:
1. `05-01-systemd-service-deployment` - Systemd integration
2. `05-02-multi-machine-federation` - Real hardware
3. `05-03-docker-compose-stack` - Container deployment

**Success**: Production-ready deployment examples

---

## 🎯 Success Metrics

### MVP (Week 1)
- [ ] User can deploy NestGate via biomeOS in < 5 minutes
- [ ] User can deploy Songbird via biomeOS in < 5 minutes
- [ ] User can deploy BearDog BTSP via biomeOS in < 10 minutes
- [ ] BTSP tunnel established and validated
- [ ] All demos use REAL primals (no mocks)

### Integration (Week 2)
- [ ] BirdSong encrypted discovery working
- [ ] Lineage-gated relay operational
- [ ] Multi-tower mesh formed
- [ ] NestGate federation working
- [ ] Encrypted storage validated

### Production (Week 3)
- [ ] Systemd services deployed
- [ ] Multi-machine federation running
- [ ] Docker compose stack working
- [ ] Production documentation complete
- [ ] Gap reports generated

---

## 📋 Action Items (Immediate)

### Priority 1: Deploy Real Primals (Today)
```bash
# Stop any mock services
pkill -f "python3.*primal"

# Deploy real primals
./deploy-real-primals.sh

# Verify running
ps aux | grep -E "(beardog|nestgate|songbird)" | grep -v grep

# Test health
curl http://localhost:9040/health  # BearDog
curl http://localhost:9020/health  # NestGate  
curl http://localhost:9000/health  # Songbird
```

### Priority 2: Create Foundation Demos (This Week)
1. Create `00-local-primal-substrate/` directory
2. Write `00-01-hello-biomeos/demo.sh`
3. Write `00-02-deploy-single-primal/demo.sh`
4. Write `01-01-hello-nestgate/demo.sh`
5. Write `02-01-deploy-songbird/demo.sh`
6. Write `02-02-deploy-beardog-btsp/demo.sh`

### Priority 3: Test Integration (Next Week)
1. Test Songbird + BearDog BirdSong
2. Test NestGate + BearDog encryption
3. Test multi-tower coordination
4. Generate gap reports

---

## 🔗 References

### Phase1 Primal Showcases (Study These)
- **Songbird Multi-Tower**: `../../../phase1/songbird/showcase/02-federation/`
- **Songbird BTSP**: `../../../phase1/songbird/showcase/13-beardog-integration/`
- **ToadStool Live Integration**: `../../../phase1/toadstool/showcase/inter-primal/`
- **NestGate Progressive**: `../../../phase1/nestgate/showcase/`
- **BearDog BTSP**: `../../../phase1/beardog/showcase/00-local-primal/06-btsp-tunnel/`

### BiomeOS Resources
- **NO_MOCKS_POLICY.md**: Current showcase policy
- **COMPREHENSIVE_CODE_AUDIT_DEC_28_2025.md**: Full codebase review
- **STATUS_AND_GAPS_DEC_28_2025.md**: Current gaps
- **primals/README.md**: Available primal binaries

### Technical Specs
- **BirdSong Protocol**: Encrypted discovery (family sees, others don't)
- **BTSP**: Biological Trust for Secure Pipes (genetic cryptography)
- **Lineage-Gated Relay**: NAT traversal via family trust
- **CALM**: Consistency as Logical Monotonicity (NestGate)

---

## 🌱 Philosophy

### BiomeOS as Substrate
> "BiomeOS doesn't just discover primals. It IS the substrate that deploys, manages, and coordinates them."

### Live Infrastructure Only
> "If it works with mocks, it proves nothing. If it works with real primals, it proves everything."

### Pure Rust Vision
> "From boot loader to P2P tunnels, pure Rust throughout. Zero bash scripts in production."

### Sovereignty First
> "Every primal maintains its sovereignty. BiomeOS orchestrates without violating."

---

## 🚀 Timeline

**Week 1 (Foundation)**: Dec 28 - Jan 3
- Day 1-2: Deploy real primals, create foundation demos
- Day 3-4: NestGate showcase
- Day 5-6: Songbird deployment
- Day 7: BearDog BTSP deployment

**Week 2 (Integration)**: Jan 4 - Jan 10
- Day 1-2: BirdSong encrypted discovery
- Day 3-4: Lineage-gated relay
- Day 5-6: Multi-tower mesh
- Day 7: Multi-primal workflows

**Week 3 (Production)**: Jan 11 - Jan 17
- Day 1-3: Systemd integration
- Day 4-5: Multi-machine federation
- Day 6-7: Docker compose, documentation

---

**Status**: Ready to Begin  
**Next Action**: Deploy real primals and create `00-01-hello-biomeos`  
**Goal**: BiomeOS as the premier substrate for sovereign P2P infrastructure

🌱 **Let's build the substrate for digital sovereignty!** 🌱

