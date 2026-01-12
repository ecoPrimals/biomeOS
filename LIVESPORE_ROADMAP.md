# 🌱 LiveSpore Implementation Roadmap

**Version**: 1.0.0  
**Date**: January 12, 2026  
**Status**: 🔬 PLANNING  
**Timeline**: 12 weeks

---

## 🎯 Vision

**LiveSpore** is a portable, self-bootstrapping NUCLEUS deployment that can:
- ✅ Run from USB without installation (Cold Spore)
- ✅ Install to bare metal (Live Spore)
- ✅ Run on top of existing OS (Sibling Spore)
- ✅ Discover and federate across all modes

**NOT a traditional live USB. 100% pure Rust. JSON-RPC & tarpc. Capability-based.**

---

## 📚 Key Documents

| Document | Purpose | Location |
|----------|---------|----------|
| **Architecture Spec** | Complete technical specification | `specs/LIVESPORE_ARCHITECTURE_SPEC.md` |
| **Primal Responsibilities** | What biomeOS vs primals implement | `specs/LIVESPORE_PRIMAL_RESPONSIBILITIES.md` |
| **This Roadmap** | Progress tracking | `LIVESPORE_ROADMAP.md` |

---

## 📊 5-Phase Implementation Plan

### **Phase 1: Runtime Adaptation** (Weeks 1-2)

**Goal**: Make biomeOS deployment-mode aware.

**biomeOS Work**:
- [ ] Implement `DeploymentMode` enum and detection
- [ ] Add adaptive socket path configuration
- [ ] Extend all primals to support mode-specific paths
- [ ] Create unit tests for mode detection

**Primal Work Required**: None (uses existing socket configuration)

**Deliverables**:
```rust
// New enum in biomeOS
pub enum DeploymentMode {
    ColdSpore { media_path: PathBuf, persistence: bool },
    LiveSpore { root_partition: PathBuf },
    SiblingSpore { install_dir: PathBuf, host_os: HostOS },
}
```

**Tests**:
- [x] Unit: Mode detection logic
- [ ] Integration: Socket paths for each mode
- [ ] E2E: Launch primals in all 3 modes

**Status**: 🟡 Not Started  
**Blockers**: None  
**Dependencies**: None (builds on existing socket config)

---

### **Phase 2: Spore Tooling** (Weeks 3-5)

**Goal**: Create LiveSpore packaging and deployment tools.

**biomeOS Work**:
- [ ] Create `biomeos-spore` crate
- [ ] Implement `spore-detector` binary (mode detection)
- [ ] Implement `spore-deployer` binary (coordinate deployment)
- [ ] Implement `spore-packager` binary (create LiveSpore packages)
- [ ] Create integration tests

**Primal Work Required**:
- [ ] **ToadStool**: Implement `hardware.detect` capability
  - `hardware.list_block_devices()`
  - `hardware.get_system_info()`
  - `hardware.detect_removable_media()`

**Deliverables**:
```
New crate: biomeos-spore/
├── src/
│   ├── detector.rs      (deployment mode detection)
│   ├── deployer.rs      (coordinate primal launch)
│   ├── packager.rs      (create .tar.zst packages)
│   └── extractor.rs     (extract packages)
├── bin/
│   ├── spore-detector.rs
│   ├── spore-deployer.rs
│   └── spore-packager.rs
```

**Tests**:
- [ ] Unit: Detector, packager, extractor
- [ ] Integration: End-to-end packaging
- [ ] E2E: Boot from packaged LiveSpore

**Status**: 🟡 Not Started  
**Blockers**: Waiting for Phase 1  
**Dependencies**: ToadStool `hardware.detect` capability

**Handoff Document**: `LIVESPORE_TOADSTOOL_HANDOFF.md` (to be created)

---

### **Phase 3: Cross-Mode Discovery** (Weeks 6-7)

**Goal**: Enable LiveSpore instances to discover each other.

**biomeOS Work**:
- [ ] Implement `spore-bridge` binary (mDNS discovery)
- [ ] Add JSON-RPC over TCP for LAN communication
- [ ] Integrate with BearDog for WAN tunnels
- [ ] Integrate with Songbird for federation

**Primal Work Required**:
- [ ] **Songbird**: Extend for cross-mode federation (may already support)
- [ ] **BearDog**: Ensure tunnel support for cross-mode (may already support)

**Deliverables**:
```rust
// New binary
spore-bridge
  - mDNS announcement (_biomeos._tcp.local.)
  - Peer discovery
  - JSON-RPC over TCP
  - BearDog tunnel coordination
```

**Tests**:
- [ ] Unit: mDNS announcement and discovery
- [ ] Integration: LAN peer discovery
- [ ] E2E: Mac Sibling ↔ Linux Live ↔ USB Cold

**Status**: 🟡 Not Started  
**Blockers**: Waiting for Phase 2  
**Dependencies**: Songbird, BearDog (likely already capable)

---

### **Phase 4: Installer** (Weeks 8-9)

**Goal**: Create interactive installer for bare metal installations.

**biomeOS Work**:
- [ ] Implement `spore-installer` binary (coordinator)
- [ ] Coordinate installation workflow
- [ ] Binary copying and graph generation
- [ ] Integration tests

**Primal Work Required**:
- [ ] **petalTongue**: Implement `installer.ui` capability
  - `installer.show_welcome()`
  - `installer.show_disk_selection(disks)`
  - `installer.confirm(disk, strategy)`
  - `installer.show_progress(status, percent)`
  - `installer.show_success()`
  
- [ ] **NestGate**: Implement `storage.prepare` capability
  - `storage.prepare(disk, strategy)` → Partition, format, mount
  - `storage.finalize(partitions)` → Install bootloader, unmount
  - `storage.install_bootloader(disk, os)`

- [ ] **Squirrel** (optional): Implement `installer.suggest` capability
  - `installer.suggest_partition_strategy(disk)`

**Deliverables**:
```rust
// New binary
spore-installer
  - Coordinates: petalTongue (UI) + NestGate (storage) + ToadStool (hardware)
  - Workflow: Detect → Select → Confirm → Execute → Success
```

**Tests**:
- [ ] Unit: Workflow coordination logic
- [ ] Integration: Mocked primal responses
- [ ] E2E: Real installation to test disk/VM

**Status**: 🟡 Not Started  
**Blockers**: Waiting for Phase 3  
**Dependencies**: petalTongue `installer.ui`, NestGate `storage.prepare`

**Handoff Documents**:
- `LIVESPORE_PETALTONGUE_HANDOFF.md` (to be created)
- `LIVESPORE_NESTGATE_HANDOFF.md` (to be created)
- `LIVESPORE_SQUIRREL_HANDOFF.md` (to be created, optional)

---

### **Phase 5: Integration & Testing** (Weeks 10-12)

**Goal**: End-to-end testing and documentation.

**biomeOS Work**:
- [ ] E2E tests for all 3 deployment modes
- [ ] Cross-mode federation tests
- [ ] Performance benchmarks
- [ ] User documentation
- [ ] Video demonstrations

**Primal Work Required**: None (all capabilities should be implemented)

**Test Matrix**:
```
┌─────────────┬──────────┬──────────┬──────────────┐
│ Test        │ Cold     │ Live     │ Sibling      │
├─────────────┼──────────┼──────────┼──────────────┤
│ Boot        │ ✅ USB   │ ✅ Disk  │ ✅ Launch    │
│ Deploy      │ ⏳       │ ⏳       │ ⏳           │
│ Discover    │ ⏳       │ ⏳       │ ⏳           │
│ Federate    │ ⏳       │ ⏳       │ ⏳           │
│ UI          │ ⏳       │ ⏳       │ ⏳           │
│ Persist     │ ⏳       │ ⏳       │ ⏳           │
│ Cross-mode  │ ⏳       │ ⏳       │ ⏳           │
└─────────────┴──────────┴──────────┴──────────────┘
```

**Deliverables**:
- [ ] Comprehensive test suite (100% coverage)
- [ ] User guide: "Getting Started with LiveSpore"
- [ ] Video: LiveSpore demo (all 3 modes)
- [ ] Performance report

**Status**: 🟡 Not Started  
**Blockers**: Waiting for Phase 4  
**Dependencies**: All primals complete

---

## 🎯 Current Status

**Current Phase**: Pre-Phase 1 (Planning Complete)  
**Next Milestone**: Phase 1 Week 1  
**Overall Progress**: 0% (0/5 phases)

**Recent Activity**:
- ✅ January 12, 2026: Architectural spec complete
- ✅ January 12, 2026: Primal responsibility matrix created
- ✅ January 12, 2026: Roadmap created
- 🟡 Awaiting decision to proceed with Phase 1

---

## 📋 Primal Coordination Status

| Primal | Capabilities Needed | Timeline | Status | Handoff Doc |
|--------|---------------------|----------|--------|-------------|
| **ToadStool** | `hardware.detect` | Phase 2 | 🟡 Pending | Not created |
| **petalTongue** | `installer.ui` | Phase 4 | 🟡 Pending | Not created |
| **NestGate** | `storage.prepare`, `storage.bootloader` | Phase 4 | 🟡 Pending | Not created |
| **Squirrel** | `installer.suggest` (optional) | Phase 5 | 🟡 Optional | Not created |
| **Songbird** | Federation (may exist) | Phase 3 | 🟢 Likely ready | Not needed |
| **BearDog** | Tunnels (may exist) | Phase 3 | 🟢 Likely ready | Not needed |

---

## 🚧 Known Blockers

**Phase 1**: None (ready to start)  
**Phase 2**: ToadStool `hardware.detect` capability  
**Phase 3**: None (Songbird/BearDog likely ready)  
**Phase 4**: petalTongue `installer.ui`, NestGate `storage.prepare`  
**Phase 5**: None

---

## 📈 Success Metrics

### Technical Metrics
- [ ] 100% pure Rust (no C dependencies for core)
- [ ] < 50 MB LiveSpore package size (compressed)
- [ ] < 5 seconds boot time (Cold Spore to NUCLEUS)
- [ ] < 1 second mode detection
- [ ] 100% test coverage (unit + E2E + chaos)

### User Metrics
- [ ] Run from USB without installation
- [ ] Install to bare metal in < 5 minutes
- [ ] Run on Mac/Linux/Windows
- [ ] Auto-discover other biomeOS nodes
- [ ] Seamless cross-mode federation

---

## 🔗 Related Documentation

### Root Level
- `START_HERE.md` - Main project entry point
- `STATUS.md` - Current project status
- `LIVESPORE_ROADMAP.md` - This document

### Specs
- `specs/LIVESPORE_ARCHITECTURE_SPEC.md` - Complete technical spec
- `specs/LIVESPORE_PRIMAL_RESPONSIBILITIES.md` - Capability matrix
- `specs/ATOMIC_DEPLOYMENT_SYSTEM_SPEC.md` - Atomic deployment (reused!)
- `specs/NUCLEUS_ORCHESTRATION_SPEC.md` - NUCLEUS graphs (reused!)

### Handoffs (To Be Created)
- `LIVESPORE_TOADSTOOL_HANDOFF.md` - Phase 2
- `LIVESPORE_PETALTONGUE_HANDOFF.md` - Phase 4
- `LIVESPORE_NESTGATE_HANDOFF.md` - Phase 4
- `LIVESPORE_SQUIRREL_HANDOFF.md` - Phase 5 (optional)

---

## 🎉 What Makes This Special

**LiveSpore is NOT a traditional live USB system.**

Traditional Approach:
- ISO with SquashFS (C-based tools)
- OverlayFS for persistence (kernel module)
- Bash scripts for initramfs
- gRPC for services
- Hardcoded paths

**biomeOS LiveSpore**:
- ✅ Tarball + zstd (pure Rust)
- ✅ NestGate for persistence (federated storage)
- ✅ Pure Rust bootstrap
- ✅ JSON-RPC + tarpc
- ✅ Capability-based discovery, adaptive paths

**Different orders of the same architecture.** 🍄🐸

---

## 📞 Team Coordination

**biomeOS Team**: Ready to implement Phases 1-3 independently  
**ToadStool Team**: Needed for Phase 2 (`hardware.detect`)  
**petalTongue Team**: Needed for Phase 4 (`installer.ui`)  
**NestGate Team**: Needed for Phase 4 (`storage.prepare`)  
**Squirrel Team**: Optional for Phase 5 (`installer.suggest`)

---

## 🏁 Next Steps

### Immediate (This Week)
1. ✅ Create architectural spec
2. ✅ Define primal responsibilities
3. ✅ Create this roadmap
4. ⏳ Decision: Proceed with Phase 1?

### If Approved
1. ⏳ Start Phase 1 implementation
2. ⏳ Create ToadStool handoff document
3. ⏳ Begin `DeploymentMode` enum implementation

---

**Status**: 🔬 Planning Complete - Awaiting Go/No-Go Decision  
**Timeline**: 12 weeks from Phase 1 start  
**Risk**: Low (reuses existing infrastructure)  
**Innovation**: High (pure Rust, capability-based, multi-mode)

---

*biomeOS: The portable, self-sovereign operating system.*

**Last Updated**: January 12, 2026  
**Next Review**: Upon Phase 1 completion

