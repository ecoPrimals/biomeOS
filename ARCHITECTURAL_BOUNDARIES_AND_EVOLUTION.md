# Architectural Boundaries: biomeOS vs Phase1 Primals
**What Belongs Where + Deep Debt Evolution Plan**

**Date**: January 31, 2026  
**Purpose**: Define clear architectural boundaries and identify deep debt evolution paths

---

## 🎯 Core Architectural Principle

**biomeOS** = Orchestrator that lives **ON TOP** of the NUCLEUS atomics  
**Phase1 Primals** = Self-contained primals that form the NUCLEUS atomics

**Philosophy**: Each primal is sovereign. biomeOS coordinates, it doesn't control.

---

## 📦 What Belongs to biomeOS

### **biomeOS Role**: System-Level Orchestration & Coordination

**Current Location**: `/phase2/biomeOS/`

### **Core Responsibilities**:

1. **Orchestration** (Lives ON TOP of NUCLEUS)
   - `biomeos-core`: Universal manager for coordinating primals
   - `biomeos-atomic-deploy`: Deploying atomic compositions
   - `biomeos-nucleus`: NUCLEUS coordination and health

2. **Graph Execution** (Utilizes NUCLEUS)
   - `biomeos-graph`: Neural graph parser and executor
   - `biomeos-api`: REST/WebSocket API for graph submission
   - Routing requests to appropriate primals

3. **Discovery & Federation** (Coordinates Primals)
   - `biomeos-federation`: Cross-primal federation management
   - Service registry aggregation (uses Songbird)
   - Multi-family coordination

4. **Deployment & Packaging** (System Level)
   - `biomeos-deploy`: Deployment orchestration
   - `biomeos-spore`: Live USB spore creation
   - `biomeos-boot`: System boot and initialization
   - `genome-deploy`: genomeBin deployment tool (future)

5. **UI & User Experience** (Human Interface)
   - `biomeos-ui`: Universal UI for ecosystem
   - `biomeos-cli`: Command-line interface
   - Dashboard and monitoring

6. **System Primitives** (Infrastructure)
   - `biomeos-types`: Shared type definitions
   - `biomeos-primal-sdk`: SDK for primal developers
   - `biomeos-test-utils`: Testing infrastructure

7. **Advanced Composition** (Higher-Order Patterns)
   - `biomeos-chimera`: Chimera fusion engine
   - `biomeos-niche`: Niche deployment
   - `biomeos-manifest`: Manifest management
   - `biomeos-compute`: Fractal compute patterns

---

## 🔬 What Belongs to Phase1 Primals

### **Phase1 Primal Locations**: `/phase1/{primal}/`

### **1. BearDog** (`/phase1/beardog/`)

**Role**: Security & Genetic Trust (Part of TOWER atomic)

**Core Responsibilities**:
```
✓ Genetic lineage validation (BirdSong)
✓ HSM management (Hardware + Software)
✓ BTSP security provider
✓ Encryption/decryption (ChaCha20-Poly1305 + Ed25519)
✓ Key derivation (HKDF-SHA256)
✓ Certificate management
✓ Audit logging
✓ IPC server (Unix sockets, abstract sockets)
```

**What Belongs Here**:
- All cryptography implementation
- HSM drivers (Android StrongBox, FIDO2, etc.)
- Genetic engine and family seed handling
- Security protocols and tunnel management
- Self-contained IPC implementation

**Deep Debt to Evolve**:
- [ ] Abstract socket support (`BEARDOG_ABSTRACT_SOCKET` env var) **P0**
- [ ] Windows named pipes (currently TCP fallback)
- [ ] iOS XPC integration
- [ ] StrongBox HSM refinement
- [ ] Bidirectional BTSP tunnels

**TODOs Found**: ~50 across crates

---

### **2. Songbird** (`/phase1/songbird/`)

**Role**: Discovery & Federation (Part of TOWER atomic)

**Core Responsibilities**:
```
✓ mDNS/STUN discovery
✓ Service registry
✓ Network federation
✓ Darkforest beacon
✓ HTTP/WebSocket gateway
✓ TLS handshake
✓ Protocol escalation (HTTP → WS → BTSP)
✓ Rendezvous coordination
```

**What Belongs Here**:
- All discovery protocols (mDNS, STUN, Bluetooth, QR, etc.)
- Service registry implementation
- Network transport layer
- TLS/crypto for HTTP
- Genesis and physical channels
- Self-contained orchestration

**Deep Debt to Evolve**:
- [ ] STUN NAT traversal validation (not fully tested) **P1**
- [ ] UDP hole punching implementation
- [ ] IPv6 support refinement
- [ ] Bluetooth LE discovery enhancement
- [ ] QR code physical channel optimization
- [ ] Leader mode deprecation cleanup

**TODOs Found**: ~80 across crates

---

### **3. Squirrel** (`/phase1/squirrel/`)

**Role**: AI Coordination (Lives ON TOP of NUCLEUS)

**Core Responsibilities**:
```
✓ AI request routing
✓ LLM adapter integration (OpenAI, Anthropic, etc.)
✓ MCP (Model Context Protocol) implementation
✓ Context management
✓ Rate limiting and quota management
✓ Plugin system
✓ AI tool coordination
```

**What Belongs Here**:
- All AI adapter implementations
- MCP protocol and transport
- Context window management
- Plugin architecture
- Tool registry and execution
- Self-contained AI orchestration

**Deep Debt to Evolve**:
- [ ] MCP websocket transport robustness **P1**
- [ ] Enhanced context learning
- [ ] Plugin hot-reload capability
- [ ] Multi-LLM coordination strategies
- [ ] Token optimization algorithms
- [ ] Streaming response handling

**TODOs Found**: ~60 across crates

---

### **4. Toadstool** (`/phase1/toadstool/`)

**Role**: GPU/CPU Compute (Part of NODE atomic)

**Core Responsibilities**:
```
✓ GPU compute (CUDA, Vulkan, OpenCL, Metal)
✓ CPU fallback (barraCUDA)
✓ Tensor operations
✓ ML inference
✓ Hardware detection and capability discovery
✓ Resource management
✓ Unified memory management
✓ Display/input (DRM, framebuffer)
```

**What Belongs Here**:
- All GPU backend implementations
- CPU tensor operations
- Hardware capability detection
- Resource allocation and scheduling
- Display/input infrastructure
- Gaming/compute workload execution
- Self-contained compute engine

**Deep Debt to Evolve**:
- [ ] WASM runtime refinement **P1**
- [ ] Vulkan compute pipeline optimization
- [ ] Neuromorphic Akida integration
- [ ] Unified memory zero-copy paths
- [ ] Display compositing enhancement
- [ ] Input device hot-plug support
- [ ] Gaming mode optimizations

**TODOs Found**: ~90 across crates

---

### **5. NestGate** (`/phase1/nestgate/`)

**Role**: Secure Storage (Part of NEST atomic)

**Core Responsibilities**:
```
✓ Model storage (local + cloud)
✓ Result caching
✓ ZFS backend integration
✓ Azure/S3 cloud storage
✓ Encryption at rest
✓ Access control
✓ Persistence management
✓ Zero-copy operations
```

**What Belongs Here**:
- All storage backend implementations
- Cache management
- Encryption/decryption for storage
- Cloud provider adapters
- Access control and permissions
- Self-contained storage engine

**Deep Debt to Evolve**:
- [ ] ZFS full integration (currently stubs) **P1**
- [ ] Azure backend completion
- [ ] S3 backend refinement
- [ ] Zero-copy validation paths
- [ ] Distributed storage coordination
- [ ] Snapshot and recovery mechanisms

**TODOs Found**: ~40 across crates

---

## 🔄 Architectural Boundaries - Clear Separations

### **What biomeOS Should NOT Do**:

❌ **Implement Security** - That's BearDog's job
- biomeOS uses BTSP, doesn't implement it
- biomeOS validates lineage, doesn't create it

❌ **Implement Discovery** - That's Songbird's job
- biomeOS queries registry, doesn't maintain it
- biomeOS uses mDNS, doesn't implement it

❌ **Implement AI Logic** - That's Squirrel's job
- biomeOS routes AI requests, doesn't handle them
- biomeOS orchestrates, doesn't coordinate LLMs

❌ **Implement Compute** - That's Toadstool's job
- biomeOS assigns work, doesn't execute it
- biomeOS manages pools, doesn't run kernels

❌ **Implement Storage** - That's NestGate's job
- biomeOS references models, doesn't store them
- biomeOS coordinates persistence, doesn't persist

### **What Each Primal Should NOT Do**:

❌ **Orchestrate Other Primals** - That's biomeOS's job
- Primals discover each other (via Songbird)
- Primals communicate directly (via BTSP)
- biomeOS coordinates multi-primal workflows

❌ **Implement UI** - That's biomeOS or PetalTongue
- Primals expose APIs (JSON-RPC, REST)
- biomeOS provides unified UI
- Each primal is headless by default

❌ **Deploy Themselves** - That's genomeBin + biomeOS
- Primals are packaged in genomeBins
- biomeOS orchestrates deployment
- Each primal is self-extracting

---

## 🧬 Deep Debt Evolution Strategy

### **Priority 0: Critical Blockers** (Current Session)

1. **BearDog Abstract Socket Support** 🔥
   - Location: `/phase1/beardog/crates/beardog-ipc/`
   - Issue: `BEARDOG_ABSTRACT_SOCKET` env var not checked
   - Impact: Blocks all Android deployment
   - Effort: 1-2 hours
   - Owner: BearDog primal

### **Priority 1: Platform Completion** (Next 2 Weeks)

2. **STUN Handshake Validation** 🌐
   - Location: `/phase1/songbird/crates/songbird-stun/`
   - Issue: Infrastructure ready, not fully tested
   - Impact: Internet-scale federation not proven
   - Effort: 5 hours
   - Owner: Songbird primal

3. **ZFS Backend Integration** 💾
   - Location: `/phase1/nestgate/crates/nestgate-zfs/`
   - Issue: Currently stubs/dev mode
   - Impact: Production storage not ready
   - Effort: 1 week
   - Owner: NestGate primal

4. **WASM Runtime Robustness** 🌐
   - Location: `/phase1/toadstool/crates/runtime/wasm/`
   - Issue: Component model and execution gaps
   - Impact: WASM workloads limited
   - Effort: 1 week
   - Owner: Toadstool primal

### **Priority 2: Feature Completion** (Next Month)

5. **MCP Protocol Enhancement** 🤖
   - Location: `/phase1/squirrel/crates/core/mcp/`
   - Issue: Websocket transport needs hardening
   - Impact: AI coordination resilience
   - Effort: 1 week
   - Owner: Squirrel primal

6. **Windows Named Pipes** 🪟
   - Location: Multiple primals IPC layers
   - Issue: Currently using TCP fallback
   - Impact: Native Windows IPC performance
   - Effort: 2 weeks (across all primals)
   - Owner: Each primal

7. **iOS XPC Integration** 📱
   - Location: Multiple primals IPC layers
   - Issue: Currently stubbed
   - Impact: Native iOS IPC
   - Effort: 2 weeks (across all primals)
   - Owner: Each primal

### **Priority 3: Optimization & Enhancement** (Next Quarter)

8. **Deprecated Code Cleanup** 🧹
   - All primals have deprecated patterns
   - Leader mode, direct mode, HTTP-first approaches
   - Effort: Ongoing refactoring
   - Owner: Each primal + biomeOS

9. **Unsafe Code Evolution** 🦀
   - Goal: 100% safe Rust everywhere
   - Current: Minimal unsafe (mostly FFI boundaries)
   - Effort: Ongoing evolution
   - Owner: All components

10. **Test Coverage Enhancement** ✅
    - Goal: >90% coverage across all primals
    - Current: Variable (60-85%)
    - Effort: Continuous improvement
    - Owner: All components

---

## 📊 Deep Debt Summary by Primal

| Primal | TODOs | Critical | Platform | Feature | Unsafe | Test Coverage |
|--------|-------|----------|----------|---------|--------|---------------|
| **BearDog** | ~50 | 1 (Android) | 3 (Win/iOS/StrongBox) | 2 (BTSP bi-dir) | 5 (HSM FFI) | ~75% |
| **Songbird** | ~80 | 0 | 3 (UDP/IPv6/BLE) | 5 (Genesis/STUN) | 2 (TLS) | ~70% |
| **Squirrel** | ~60 | 0 | 1 (MCP WS) | 4 (Context/Plugins) | 0 | ~80% |
| **Toadstool** | ~90 | 0 | 4 (WASM/Vulkan/Display) | 6 (Neuro/Gaming) | 8 (GPU FFI) | ~65% |
| **NestGate** | ~40 | 1 (ZFS) | 2 (Azure/S3) | 3 (Zero-copy) | 1 (Storage) | ~70% |
| **biomeOS** | ~58 | 0 | 0 | 8 (UI/Graph/Deploy) | 0 | ~75% |
| **Total** | **~378** | **2** | **13** | **28** | **16** | **~73%** |

---

## 🎯 Evolution Principles

### **Smart Refactoring** (Not Just Splitting)
- Understand domain boundaries before refactoring
- Maintain cohesion within each primal
- Clear interfaces between components
- Don't split for the sake of splitting

### **Modern Idiomatic Rust**
- Prefer safe abstractions over unsafe
- Use async/await properly (not blocking)
- Leverage type system for correctness
- Follow Rust API guidelines

### **Platform Agnostic**
- Runtime detection, not compile-time only
- Capability-based feature selection
- Graceful degradation
- Zero hardcoding

### **Complete Implementations**
- No mocks in production code
- Test mocks isolated to test modules
- Real implementations, not stubs
- Production-grade error handling

### **Primal Autonomy**
- Each primal has self-knowledge only
- Discovery happens at runtime
- No primal knows about biomeOS internals
- biomeOS knows about primal APIs

---

## 🚀 Recommended Evolution Path

### **Week 1** (Current)
1. Fix BearDog abstract socket support
2. Complete TOWER validation (USB + Pixel)
3. Validate NUCLEUS atomics working

### **Week 2**
4. Complete STUN handshake validation
5. Test cross-platform federation over internet
6. Begin ZFS backend integration in NestGate

### **Week 3-4**
7. Complete ZFS backend
8. Enhance WASM runtime in Toadstool
9. Harden MCP transport in Squirrel

### **Month 2**
10. Windows named pipes (all primals)
11. iOS XPC integration (all primals)
12. Deprecated code cleanup

### **Month 3**
13. Unsafe code evolution
14. Test coverage to >90%
15. Production certification

---

## 📝 Key Insights

### **Architectural Clarity**:
- biomeOS = Orchestrator (lives ON TOP)
- Phase1 Primals = Self-contained services (form NUCLEUS)
- Each primal is sovereign, biomeOS coordinates

### **Ownership**:
- Each primal owns its domain completely
- biomeOS owns system-level orchestration
- Clear boundaries, minimal coupling

### **Evolution Strategy**:
- Fix critical blockers first (Android support)
- Complete platform support (STUN, ZFS, WASM)
- Enhance features systematically
- Clean up technical debt continuously

### **Deep Debt Philosophy**:
- Treat TODOs as evolution opportunities
- Smart refactoring, not just splitting
- Modern idiomatic Rust everywhere
- Platform-agnostic by design
- Complete implementations, no mocks in production

---

**Status**: Clear architectural boundaries defined, evolution path established. 🎯

**Next**: Execute Priority 0 (BearDog abstract socket fix), then proceed systematically through the evolution roadmap.
