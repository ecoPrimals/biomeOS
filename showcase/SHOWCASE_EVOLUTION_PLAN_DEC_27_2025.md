# BiomeOS Showcase Evolution Plan - Dec 27, 2025

**Goal**: Build comprehensive showcase demonstrating BiomeOS discovering and orchestrating real Phase1 primals, inspired by successful patterns from Songbird multi-tower federation and Toadstool compute demos.

---

## 📊 Current State Analysis

### Existing Primal Showcases (Phase 1)

**Songbird** (`phase1/songbird/showcase/`):
- ✅ **10-inter-primal-foundation**: Universal Port Authority pattern
  - Registration protocol with dynamic port assignment
  - Capability-based discovery
  - Zero compile-time dependencies
  - Successfully demonstrated with Toadstool integration
- ✅ **05-albatross-multiplex**: Protocol multiplexing
  - HTTPS, tRPC, JSON-RPC protocol support
  - Cross-tower federation benchmarks
  - Intelligent protocol selection

**Toadstool** (`phase1/toadstool/showcase/`):
- ✅ **inter-primal/01-songbird-distributed-compute**: Compute orchestration via Songbird
- ✅ **inter-primal/01-beardog-encrypted-workload**: Secure compute with BearDog encryption
- ✅ **inter-primal/03-nestgate-persistent-results**: Compute with persistent storage
- ✅ **inter-primal/05-full-ecosystem-ml**: Full ML pipeline across all primals

**BearDog**, **Nestgate**, **Squirrel**: Have showcase directories

### BiomeOS Showcase (`phase2/biomeOS/showcase/`)

**Current Structure**:
- `00-local-capabilities/`: BiomeOS-only features (manifest parsing, capability matching)
- `01-single-primal/`: Individual primal discovery (partially complete)
- `02-primal-pairs/`: Two-primal workflows
- `03-p2p-coordination/`: P2P mesh examples (5 demos complete)
- Various other patterns

**Gap**: Need to leverage successful primal patterns into BiomeOS orchestration

---

## 🎯 Evolution Strategy

### Phase 1: Enhance Single-Primal Showcases (Current Priority)

**Objective**: Demonstrate BiomeOS discovering and using each Phase1 primal's BEST capabilities

#### 1.1 Songbird Showcase (`01-single-primal/songbird-discovery.sh`)

**Current**: Basic discovery demo  
**Enhance with**:
- Service registration via universal port authority
- Multi-protocol support demonstration (HTTPS, tRPC, JSON-RPC)
- Dynamic port allocation
- Heartbeat and health monitoring

**Inspired by**: `songbird/showcase/10-inter-primal-foundation/`

**Demo Flow**:
```bash
# 1. BiomeOS discovers Songbird
# 2. Registers itself as a service
# 3. Gets dynamic port assignment
# 4. Demonstrates capability query
# 5. Shows heartbeat mechanism
# 6. Queries other registered services
```

#### 1.2 Toadstool Compute Showcase (`01-single-primal/toadstool-compute.sh`)

**Current**: Basic compute demo  
**Enhance with**:
- GPU workload demonstration
- Distributed compute task submission
- Real-time task monitoring
- Benchmark results retrieval

**Inspired by**: `toadstool/showcase/scripts/demo-gpu-basic.sh`, `demo-distributed-compute.sh`

**Demo Flow**:
```bash
# 1. BiomeOS discovers Toadstool
# 2. Queries compute capabilities (CPU, GPU, specialized)
# 3. Submits benchmark task
# 4. Monitors execution progress
# 5. Retrieves and validates results
# 6. Shows performance metrics
```

#### 1.3 BearDog Security Showcase (`01-single-primal/beardog-security.sh`)

**Current**: Basic crypto demo  
**Enhance with**:
- Entropy hierarchy demonstration
- Multi-layer encryption
- Secure key exchange
- Signature verification workflow

**Inspired by**: BearDog's entropy hierarchy and capability discovery

**Demo Flow**:
```bash
# 1. BiomeOS discovers BearDog
# 2. Demonstrates entropy hierarchy (ephemeral -> session -> persistent)
# 3. Encrypts data with layered approach
# 4. Shows key rotation
# 5. Verifies signatures
# 6. Demonstrates secure shutdown with key wipe
```

#### 1.4 Nestgate Storage Showcase (`01-single-primal/nestgate-storage.sh`)

**Current**: Basic storage demo  
**Enhance with**:
- Volume lifecycle management
- Lineage-gated access control
- Data sovereignty demonstration
- Backup and recovery

**Demo Flow**:
```bash
# 1. BiomeOS discovers Nestgate
# 2. Creates volume with lineage metadata
# 3. Stores data with sovereignty controls
# 4. Demonstrates access control
# 5. Shows backup/restore
# 6. Verifies data integrity
```

#### 1.5 Squirrel AI Showcase (`01-single-primal/squirrel-ai.sh`)

**Current**: Basic AI demo  
**Enhance with**:
- MCP protocol demonstration
- Multi-agent orchestration
- Tool execution pipeline
- Agent lifecycle management

**Demo Flow**:
```bash
# 1. BiomeOS discovers Squirrel
# 2. Creates AI agent with specific tools
# 3. Demonstrates MCP tool execution
# 4. Shows agent reasoning
# 5. Orchestrates multi-agent workflow
# 6. Clean agent shutdown
```

---

### Phase 2: Cross-Primal Workflows (Inspired by Existing Patterns)

#### 2.1 Songbird + Toadstool: Distributed Compute via Port Authority

**Pattern**: Universal Port Authority enables zero-config compute orchestration

**Demo** (`02-primal-pairs/01-songbird-toadstool-compute.sh`):
```bash
# 1. Start Songbird (port authority)
# 2. Toadstool registers via Songbird
# 3. BiomeOS submits task to Songbird
# 4. Songbird routes to Toadstool (by capability)
# 5. Task executes, results return via Songbird
# 6. BiomeOS never knew about Toadstool directly
```

**Key Insight**: BiomeOS only talks to Songbird, which handles all routing

#### 2.2 BearDog + Toadstool: Encrypted Workload

**Pattern**: Secure compute with end-to-end encryption

**Demo** (`02-primal-pairs/beardog-toadstool-secure-compute.sh`):
```bash
# 1. BiomeOS encrypts task data via BearDog
# 2. Submits encrypted task to Toadstool
# 3. Toadstool executes in secure enclave
# 4. Results encrypted via BearDog
# 5. BiomeOS decrypts final results
```

**Inspired by**: `toadstool/showcase/inter-primal/01-beardog-encrypted-workload/`

#### 2.3 Toadstool + Nestgate: Compute with Persistence

**Pattern**: Task execution with result persistence

**Demo** (`02-primal-pairs/toadstool-nestgate-persistent.sh`):
```bash
# 1. BiomeOS submits compute task to Toadstool
# 2. Toadstool executes and generates results
# 3. Results automatically stored in Nestgate
# 4. BiomeOS retrieves from Nestgate later
# 5. Shows result lineage tracking
```

**Inspired by**: `toadstool/showcase/inter-primal/03-nestgate-persistent-results/`

#### 2.4 Full Ecosystem ML Pipeline

**Pattern**: Complete ML workflow across all primals

**Demo** (`02-primal-pairs/full-ecosystem-ml.sh`):
```bash
# 1. Songbird: Service discovery and routing
# 2. BearDog: Secure data and model storage
# 3. Nestgate: Training data persistence
# 4. Toadstool: Model training (GPU compute)
# 5. Squirrel: Inference and agent deployment
# 6. BiomeOS orchestrates entire pipeline
```

**Inspired by**: `toadstool/showcase/inter-primal/05-full-ecosystem-ml/`

---

### Phase 3: BiomeOS-Specific Capabilities

#### 3.1 Multi-Tower Federation via P2P

**Current**: `03-p2p-coordination/` has 5 complete demos

**Enhance with**:
- Real primal integration (not just BiomeOS-to-BiomeOS)
- BirdSong protocol demonstration
- BTSP tunnel coordination
- Multi-tower service discovery

#### 3.2 Primal Adapter Evolution

**Current**: `03-primal-adapter/` has basic demo

**Enhance with**:
- Runtime interface discovery
- Stop command auto-detection
- Multiple discovery methods (mDNS, broadcast, multicast)
- Graceful degradation

#### 3.3 Chimera Patterns (Phase2 Primals)

**Showcase**: `03-chimera-patterns/`

**Add**:
- loamSpine embedded workflow
- rhizoCrypt integration
- sweetGrass demonstration
- petalTongue UI integration

---

## 📋 Implementation Priority

### Immediate (Next Session)

1. **Enhance `01-single-primal/songbird-discovery.sh`**
   - Add universal port authority demo
   - Show service registration
   - Demonstrate dynamic port allocation

2. **Enhance `01-single-primal/toadstool-compute.sh`**
   - Add GPU capability demonstration
   - Show distributed compute
   - Include benchmark results

3. **Create comprehensive README** for `01-single-primal/`
   - Document each primal's best capabilities
   - Show how BiomeOS discovers and uses them
   - Include troubleshooting guide

### Short-term (This Week)

4. **Build 3 key cross-primal demos**:
   - Songbird + Toadstool (distributed compute)
   - BearDog + Toadstool (encrypted workload)
   - Full ecosystem ML pipeline

5. **Integrate P2P coordination** with real primals
   - Multi-tower Songbird federation
   - Cross-tower task routing
   - Distributed service registry

### Medium-term (Next Week)

6. **Phase2 primal showcases**:
   - loamSpine federation management
   - rhizoCrypt encryption layer
   - sweetGrass ecosystem coordination
   - petalTongue UI demonstrations

7. **Performance benchmarks**:
   - Cross-primal latency
   - Throughput measurements
   - Resource usage analysis

---

## 🎓 Key Principles (from Successful Patterns)

### 1. Universal Port Authority (Songbird Pattern)

**Never hardcode ports**:
- Songbird assigns ALL ports dynamically
- Primals discover each other via Songbird
- Zero port conflicts, infinite scalability

**BiomeOS Application**:
- BiomeOS can discover Songbird first
- Register itself for port assignment
- Discover other services via Songbird

### 2. Capability-Based Discovery

**Never hardcode service names**:
- Discover by capability, not identity
- "compute" → finds Toadstool
- "encryption" → finds BearDog

**BiomeOS Application**:
- Query by capability requirements
- Automatic service matching
- Load balancing across multiple providers

### 3. Zero Compile-Time Dependencies

**Pure runtime interaction**:
- HTTP/JSON for all communication
- No code imports between primals
- Each primal knows only itself

**BiomeOS Application**:
- Adapters use only HTTP/JSON
- No primal-specific code in BiomeOS
- Runtime interface discovery

### 4. Demonstrate Real Value

**Show actual workflows**:
- Toadstool: Real GPU compute, not mock tasks
- BearDog: Real encryption, not fake keys
- Nestgate: Real storage, not memory buffers

**BiomeOS Application**:
- Use real primal binaries from ../../primalBins/
- Show actual capabilities, not simulations
- Document real gaps, not theoretical issues

---

## 📁 File Structure Evolution

```
showcase/
├── 00-local-capabilities/          # BiomeOS-only (current)
├── 01-single-primal/               # Individual primal demos
│   ├── README.md                   # ← ENHANCE: Comprehensive guide
│   ├── songbird-discovery.sh       # ← ENHANCE: Port authority demo
│   ├── toadstool-compute.sh        # ← ENHANCE: GPU + distributed
│   ├── beardog-security.sh         # ← ENHANCE: Entropy hierarchy
│   ├── nestgate-storage.sh         # ← ENHANCE: Lineage + sovereignty
│   ├── squirrel-ai.sh              # ← ENHANCE: MCP + multi-agent
│   └── common/
│       ├── primal-runner.sh        # ← NEW: Common primal startup
│       └── gap-reporter.sh         # ← NEW: Standardized gap logging
│
├── 02-primal-pairs/                # Two-primal workflows
│   ├── README.md                   # ← ENHANCE: Pattern documentation
│   ├── 01-songbird-toadstool/      # ← NEW: Port authority compute
│   ├── 02-beardog-toadstool/       # ← NEW: Encrypted workload
│   ├── 03-toadstool-nestgate/      # ← NEW: Persistent compute
│   ├── 04-full-ecosystem-ml/       # ← NEW: Complete ML pipeline
│   └── patterns/
│       └── UNIVERSAL_PATTERNS.md   # ← NEW: Reusable patterns
│
├── 03-p2p-coordination/            # Multi-tower (current - 5 demos)
│   └── 06-real-primal-federation/  # ← NEW: Real primal integration
│
├── 04-chimera-patterns/            # Phase2 primals
│   ├── loamspine/                  # ← NEW: Federation mgmt demos
│   ├── rhizocrypt/                 # ← NEW: Encryption layer demos
│   ├── sweetgrass/                 # ← NEW: Ecosystem demos
│   └── petaltongue/                # ← NEW: UI integration demos
│
└── 05-complete-ecosystem/          # Full system demos
    ├── 01-autonomous-ml-pipeline/  # ← NEW: End-to-end ML
    ├── 02-sovereign-data-flow/     # ← NEW: Data sovereignty demo
    └── 03-multi-tower-federation/  # ← NEW: Geographic distribution
```

---

## 🎯 Success Metrics

### For Each Demo

- ✅ **Uses real primal binaries** (no mocks)
- ✅ **Shows actual capabilities** (not simulations)
- ✅ **Documents real gaps** (integration issues)
- ✅ **Runs in <5 minutes** (quick validation)
- ✅ **Clear output** (what's happening, why)
- ✅ **Automated cleanup** (no manual intervention)

### For Overall Showcase

- ✅ **New user can run all demos** in 30 minutes
- ✅ **Demonstrates BiomeOS value** (orchestration, discovery, sovereignty)
- ✅ **Shows primal ecosystem** (how they work together)
- ✅ **Documents integration patterns** (reusable knowledge)
- ✅ **Identifies real gaps** (honest assessment)

---

## 🚀 Next Steps

1. **Review and approve** this plan
2. **Start with `01-single-primal/songbird-discovery.sh`** enhancement
3. **Build one cross-primal demo** (Songbird + Toadstool)
4. **Document patterns** as we go
5. **Iterate based on findings**

---

**Philosophy**: Learn from what's working (Songbird's universal port authority, Toadstool's compute demos), apply to BiomeOS orchestration, document real integration patterns.

**Status**: Ready to execute ✅

