# 🗺️ biomeOS Development Roadmap

**Version**: 1.0.0  
**Date**: January 9, 2026  
**Status**: Phase 1 Complete → Phase 2 Starting

---

## 🎊 Current Status

### **✅ Phase 1: Neural API Foundation - COMPLETE!**

**Achievements**:
- ✅ Graph-based orchestration (`biomeos-graph` crate)
- ✅ Capability-based primal selection
- ✅ Multi-layer coordination patterns
- ✅ Metrics collection & learning foundation
- ✅ Real primal testing (BearDog verified via Unix socket)
- ✅ Tower, Node, Nest niche foundations
- ✅ NUCLEUS complete specification (all primals integrated)

**Statistics**:
- **Crates**: 15+ implemented
- **Graphs**: 9 TOML definitions
- **Niches**: 3 production-ready (Tower, Node, Nest)
- **Primals**: 4 Phase 1 primals integrated
- **Tests**: Integration tests passing
- **Documentation**: 96.7% spec completion (29/30)

---

## 🚀 Phase 2: Secure Discovery & Primal Evolution

**Timeline**: Weeks 1-4 (January 2026)  
**Goal**: Implement NUCLEUS, evolve primal APIs, achieve E2E federation

### **Week 1-2: NUCLEUS Implementation** 🚧 **CURRENT FOCUS**

#### **biomeOS Tasks**

| Task | Owner | Priority | Status |
|------|-------|----------|--------|
| Implement `SecurePrimalDiscovery` | biomeOS | 🔴 HIGH | 📋 Spec Complete |
| Add authentication (UID, signatures) | biomeOS | 🔴 HIGH | 📋 Spec Complete |
| Add multiple instance support | biomeOS | 🔴 HIGH | 📋 Spec Complete |
| Add `SelectionCriteria` API | biomeOS | 🔴 HIGH | 📋 Spec Complete |
| Update `BearDogClient::from_discovery()` | biomeOS | 🟡 MEDIUM | 📋 Spec Complete |
| Update `verify-lineage` tool | biomeOS | 🟡 MEDIUM | ✅ Tool exists |
| Unit tests (authentication, selection) | biomeOS | 🔴 HIGH | ⏳ Pending |
| Integration tests (E2E discovery) | biomeOS | 🔴 HIGH | ⏳ Pending |
| Security tests (hijacking prevention) | biomeOS | 🔴 HIGH | ⏳ Pending |

**Deliverables**:
- ✅ `crates/biomeos-federation/src/secure_discovery.rs`
- ✅ `SecurePrimalDiscovery` with 5-layer verification
- ✅ `VerifiedPrimal` with trust levels
- ✅ `SelectionCriteria` for explicit instance selection
- ✅ Comprehensive test suite

---

### **Week 3: Primal API Evolution** ⏳ **NEXT**

#### **Toadstool Tasks** (ecoPrimals/phase1/toadstool/)

| Task | Owner | Priority | Status |
|------|-------|----------|--------|
| Unix socket JSON-RPC server | Toadstool Team | 🔴 HIGH | 📋 Handoff created |
| Implement `get_capabilities` API | Toadstool Team | 🔴 HIGH | ⏳ Pending |
| Implement `submit_workload` API | Toadstool Team | 🔴 HIGH | ⏳ Pending |
| Implement `get_workload_status` API | Toadstool Team | 🟡 MEDIUM | ⏳ Pending |
| HTTP deprecation (port-free mode) | Toadstool Team | 🟡 MEDIUM | ⏳ Pending |
| Integration tests with biomeOS | Both Teams | 🔴 HIGH | ⏳ Pending |

**Handoff Document**: `docs/jan4-session/TOADSTOOL_UNIX_SOCKET_HANDOFF_JAN9.md`

**Deliverables**:
- ✅ `/tmp/toadstool-${NODE_ID}.sock` (Unix socket)
- ✅ JSON-RPC 2.0 server
- ✅ 3+ APIs functional
- ✅ Port-free architecture

---

#### **NestGate Tasks** (ecoPrimals/phase1/nestgate/)

| Task | Owner | Priority | Status |
|------|-------|----------|--------|
| Unix socket JSON-RPC server | NestGate Team | 🔴 HIGH | ⏳ Pending |
| Implement `get_capabilities` API | NestGate Team | 🔴 HIGH | ⏳ Pending |
| Implement `store_with_provenance` API | NestGate Team | 🔴 HIGH | ⏳ Pending |
| Implement `get_storage_status` API | NestGate Team | 🟡 MEDIUM | ⏳ Pending |
| HTTP deprecation (port-free mode) | NestGate Team | 🟡 MEDIUM | ⏳ Pending |
| Integration tests with biomeOS | Both Teams | 🔴 HIGH | ⏳ Pending |

**Deliverables**:
- ✅ `/tmp/nestgate-${NODE_ID}.sock` (Unix socket)
- ✅ JSON-RPC 2.0 server
- ✅ 3+ APIs functional
- ✅ Port-free architecture

---

### **Week 4: E2E Testing & Validation** ⏳ **FUTURE**

#### **Hardware Testing**

| Test Scenario | Hardware | Priority | Status |
|---------------|----------|----------|--------|
| Tower + NUCLEUS on USB spores | 3x USB spores | 🔴 HIGH | ✅ Hardware available |
| LAN federation (2 nodes) | 2x computers | 🔴 HIGH | ✅ Hardware available |
| Node + NUCLEUS (local compute) | Local machine | 🟡 MEDIUM | ✅ Hardware available |
| Nest + NUCLEUS (local data) | Local machine | 🟡 MEDIUM | ✅ Hardware available |
| Complete ecosystem (Tower+Node+Nest) | All hardware | 🟢 LOW | ✅ Hardware available |

**Hardware Available**:
- ✅ 3x USB spores (node-alpha, node-gamma, node-delta)
- ✅ Pixel 8a (hardware TPM + biometric entropy)
- ✅ Multiple computers for LAN testing

**Test Scenarios**:
1. **Secure Discovery**: NUCLEUS discovers all primals, verifies identities
2. **Genetic Lineage**: All spores verified as siblings (family: nat0)
3. **BTSP Tunnels**: Encrypted P2P communication established
4. **Multi-Niche**: Tower + Node + Nest deployed and federated

**Deliverables**:
- ✅ E2E test suite
- ✅ Hardware validation report
- ✅ Performance benchmarks
- ✅ Security audit results

---

## 🎨 Phase 2.5: petalTongue UI Integration

**Timeline**: Week 2-3 (January 2026)  
**Goal**: Integrate petalTongue as universal user interface

### **petalTongue Integration** (biomeOS + petalTongue)

#### **biomeOS Tasks**

| Task | Owner | Priority | Status |
|------|-------|----------|--------|
| Add topology data API | biomeOS Team | 🔴 HIGH | ⏳ Pending |
| Implement `/api/v1/topology` endpoint | biomeOS Team | 🔴 HIGH | ⏳ Pending |
| Test with petalTongue binary | biomeOS Team | 🔴 HIGH | ⏳ Pending |
| Document topology data format | biomeOS Team | 🟡 MEDIUM | ⏳ Pending |

#### **petalTongue Tasks**

| Task | Owner | Priority | Status |
|------|-------|----------|--------|
| Verify Unix socket evolution | petalTongue Team | 🟡 MEDIUM | ⏳ Pending |
| Add JSON-RPC server | petalTongue Team | 🟡 MEDIUM | ⏳ Pending |
| Test with biomeOS topology data | petalTongue Team | 🟡 MEDIUM | ⏳ Pending |
| NUCLEUS integration | petalTongue Team | 🟡 MEDIUM | ⏳ Pending |

**Use Cases**:
- Universal user interface for ecosystem
- Real-time topology visualization
- Multi-modal rendering (visual + audio + text)
- Network to Toadstool for GPU rendering (future)

**Deliverables**:
- ✅ petalTongue binary in `plasmidBins/`
- ✅ UI niche manifest (`niches/ui.toml`)
- ✅ UI deployment graphs
- ✅ Integration plan document
- ✅ E2E UI testing

---

## 🔮 Phase 3: Compute Enclaves & Data Federation

**Timeline**: Weeks 5-8 (February 2026)  
**Goal**: Encrypted compute + data federation

### **Compute Enclaves** (Toadstool + BearDog)

#### **BearDog Tasks**

| Task | Owner | Priority | Status |
|------|-------|----------|--------|
| Design compute enclave API | BearDog Team | 🔴 HIGH | 📋 Spec exists |
| Implement `create_compute_enclave` | BearDog Team | 🔴 HIGH | ⏳ Pending |
| Implement encrypted memory | BearDog Team | 🔴 HIGH | ⏳ Pending |
| Implement encrypted I/O | BearDog Team | 🔴 HIGH | ⏳ Pending |
| Workload signature verification | BearDog Team | 🟡 MEDIUM | ⏳ Pending |

#### **Toadstool Tasks**

| Task | Owner | Priority | Status |
|------|-------|----------|--------|
| Integrate with BearDog enclave API | Toadstool Team | 🔴 HIGH | ⏳ Pending |
| Implement `submit_workload_encrypted` | Toadstool Team | 🔴 HIGH | ⏳ Pending |
| Support encrypted memory execution | Toadstool Team | 🔴 HIGH | ⏳ Pending |
| Test secure ML training | Both Teams | 🟡 MEDIUM | ⏳ Pending |

**Use Cases**:
- Encrypted ML training (model protection)
- Secure AI inference (data privacy)
- Confidential computing (multi-party)

**Deliverables**:
- ✅ Compute enclave API
- ✅ Encrypted memory support
- ✅ Encrypted I/O support
- ✅ E2E secure ML training demo

---

### **Data Federation** (NestGate + BearDog + Songbird)

#### **NestGate Tasks**

| Task | Owner | Priority | Status |
|------|-------|----------|--------|
| Implement data provenance tracking | NestGate Team | 🔴 HIGH | ⏳ Pending |
| Implement adaptive compression | NestGate Team | 🔴 HIGH | ⏳ Pending |
| Implement sharding & replication | NestGate Team | 🟡 MEDIUM | ⏳ Pending |
| Integrate with BearDog encryption | NestGate Team | 🔴 HIGH | ⏳ Pending |
| Integrate with Songbird federation | NestGate Team | 🔴 HIGH | ⏳ Pending |

#### **BearDog Tasks**

| Task | Owner | Priority | Status |
|------|-------|----------|--------|
| Implement data encryption API | BearDog Team | 🔴 HIGH | ⏳ Pending |
| Implement ownership verification | BearDog Team | 🔴 HIGH | ⏳ Pending |
| Implement access control (lineage) | BearDog Team | 🟡 MEDIUM | ⏳ Pending |

#### **Songbird Tasks**

| Task | Owner | Priority | Status |
|------|-------|----------|--------|
| Implement data transfer via BTSP | Songbird Team | 🔴 HIGH | ✅ BTSP exists |
| Coordinate shard placement | Songbird Team | 🟡 MEDIUM | ⏳ Pending |
| Manage replication | Songbird Team | 🟡 MEDIUM | ⏳ Pending |

**Use Cases**:
- Distributed genomic data (8:1 compression!)
- Family photo federation (encrypted, provenance)
- Multi-node backup (sharding + replication)

**Deliverables**:
- ✅ Data provenance system
- ✅ Adaptive compression (8:1 genomics)
- ✅ Sharding & replication
- ✅ E2E encrypted data federation demo

---

## 🌟 Phase 4: Neural API Evolution

**Timeline**: Weeks 9-12 (March 2026)  
**Goal**: Advanced orchestration patterns

### **Parallel Execution**

| Task | Owner | Priority | Status |
|------|-------|----------|--------|
| Implement `ParallelGraphExecutor` | biomeOS | 🟡 MEDIUM | ⏳ Pending |
| Concurrent node execution | biomeOS | 🟡 MEDIUM | ⏳ Pending |
| Resource contention handling | biomeOS | 🟢 LOW | ⏳ Pending |
| Tests & benchmarks | biomeOS | 🟡 MEDIUM | ⏳ Pending |

### **DAG Execution**

| Task | Owner | Priority | Status |
|------|-------|----------|--------|
| Implement `DAGGraphExecutor` | biomeOS | 🟡 MEDIUM | ⏳ Pending |
| Dependency resolution | biomeOS | 🟡 MEDIUM | ⏳ Pending |
| Conditional branching | biomeOS | 🟢 LOW | ⏳ Pending |
| Tests & benchmarks | biomeOS | 🟡 MEDIUM | ⏳ Pending |

### **Adaptive Learning**

| Task | Owner | Priority | Status |
|------|-------|----------|--------|
| Analyze metrics for bottlenecks | biomeOS | 🟢 LOW | ⏳ Pending |
| Optimize graph execution paths | biomeOS | 🟢 LOW | ⏳ Pending |
| Adaptive primal selection | biomeOS | 🟢 LOW | ⏳ Pending |
| Self-healing orchestration | biomeOS | 🟢 LOW | ⏳ Pending |

**Deliverables**:
- ✅ Parallel graph execution
- ✅ DAG execution
- ✅ Adaptive learning from metrics
- ✅ Self-healing orchestration

---

## 📊 Milestone Summary

| Phase | Timeline | Status | Completion |
|-------|----------|--------|------------|
| **Phase 1: Neural API Foundation** | Dec 2025 - Jan 2026 | ✅ COMPLETE | 100% |
| **Phase 2: Secure Discovery** | Jan 2026 (Weeks 1-4) | 🚧 IN PROGRESS | 25% (Spec) |
| **Phase 3: Enclaves & Federation** | Feb 2026 (Weeks 5-8) | ⏳ PLANNED | 0% |
| **Phase 4: Neural API Evolution** | Mar 2026 (Weeks 9-12) | ⏳ PLANNED | 0% |

---

## 🎯 Critical Path

### **Immediate (This Week)**
1. 🔴 Implement `SecurePrimalDiscovery` in `biomeos-federation`
2. 🔴 Add authentication (socket owner, signatures)
3. 🔴 Add multiple instance support
4. 🔴 Write comprehensive tests

### **Short-Term (Next 2 Weeks)**
1. 🔴 Toadstool Unix socket evolution (handoff to team)
2. 🔴 NestGate Unix socket evolution (handoff to team)
3. 🔴 Update `verify-lineage` to use NUCLEUS
4. 🟡 E2E testing with USB spores

### **Medium-Term (Next Month)**
1. 🟡 Compute enclave API design (BearDog + Toadstool)
2. 🟡 Data federation API design (NestGate + BearDog + Songbird)
3. 🟡 Hardware testing & validation
4. 🟢 Performance benchmarks

---

## 🤝 Team Coordination

### **biomeOS Team**
**Focus**: NUCLEUS implementation, Neural API evolution  
**Current**: Implementing `SecurePrimalDiscovery`  
**Blockers**: None (specs complete)

### **Songbird Team** (ecoPrimals/phase1/songbird/)
**Focus**: BirdSong P2P, BTSP coordination  
**Current**: Production ready (v3.19.0)  
**Blockers**: None

### **BearDog Team** (ecoPrimals/phase1/beardog/)
**Focus**: Encryption, genetic lineage, BTSP  
**Current**: Production ready (v0.15.2)  
**Blockers**: None

### **Toadstool Team** (ecoPrimals/phase1/toadstool/)
**Focus**: Compute orchestration, Unix socket evolution  
**Current**: HTTP mode (needs Unix socket)  
**Blockers**: 🔴 Unix socket handoff pending  
**Handoff**: `docs/jan4-session/TOADSTOOL_UNIX_SOCKET_HANDOFF_JAN9.md`

### **NestGate Team** (ecoPrimals/phase1/nestgate/)
**Focus**: Storage, provenance, compression  
**Current**: Needs Unix socket evolution  
**Blockers**: 🔴 Unix socket handoff needed

---

## 📈 Success Metrics

### **Phase 2 Success Criteria**
- ✅ NUCLEUS implemented with 5-layer verification
- ✅ All primals discoverable via NUCLEUS
- ✅ Socket hijacking prevented (security tests pass)
- ✅ Multiple instance selection working
- ✅ Toadstool + NestGate on Unix sockets
- ✅ E2E federation on USB spores

### **Phase 3 Success Criteria**
- ✅ Compute enclaves functional (encrypted memory + I/O)
- ✅ Data federation functional (provenance + compression)
- ✅ Secure ML training demo working
- ✅ Encrypted data transfer demo working

### **Phase 4 Success Criteria**
- ✅ Parallel graph execution working
- ✅ DAG execution working
- ✅ Adaptive learning from metrics
- ✅ Self-healing orchestration

---

## 🎊 Bottom Line

**Current Status**: Phase 1 Complete, Phase 2 Starting

**Immediate Focus**:
1. Implement NUCLEUS (biomeOS)
2. Evolve Toadstool to Unix sockets (Toadstool Team)
3. Evolve NestGate to Unix sockets (NestGate Team)

**Timeline**: 4 weeks to Phase 2 complete

**Hardware**: ✅ Ready (3x USB spores + Pixel 8a)

**Specs**: ✅ 96.7% complete (29/30)

**Team**: ✅ Coordinated (handoffs created)

---

**The architecture is complete. The specs are done. Now we implement!**

🚀 **Phase 2: Secure Discovery - Let's Go!** 🌱

