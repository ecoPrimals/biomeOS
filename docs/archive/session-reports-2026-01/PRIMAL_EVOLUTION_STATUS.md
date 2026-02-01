# Primal Evolution Status - Isomorphic IPC & Deep Debt

**Date**: January 31, 2026  
**Focus**: Which primals need isomorphic IPC evolution

═══════════════════════════════════════════════════════════════════

## 🎯 ISOMORPHIC IPC STATUS

### ✅ COMPLETE - songbird
**Status**: Production Ready (v3.33.0)
**Features**:
- ✅ Phase 1: Automatic TCP fallback (Jan 31, 16:34)
- ✅ Phase 2: Client discovery (Jan 31, 16:38)
- ✅ Phase 3: Connection handling (Jan 31, 16:49)
- ✅ Try→Detect→Adapt→Succeed pattern
- ✅ Validated on Android/SELinux
**Grade**: A++ (205/100)

### 🟡 PARTIAL - beardog
**Status**: Has platform abstraction, needs isomorphic IPC
**What It Has**:
- ✅ Universal platform traits (Jan 30)
- ✅ Android abstract socket support
- ✅ Unix socket IPC
- ✅ Platform-specific implementations
**What It Needs**:
- ⏳ Try→Detect→Adapt pattern
- ⏳ Automatic TCP fallback
- ⏳ Discovery file system
- ⏳ Client-side discovery
**Priority**: HIGH (part of TOWER atomic)
**Effort**: 4-6 hours (copy songbird pattern)

### 🔴 NEEDS EVOLUTION - toadstool
**Status**: No IPC detected
**Current Focus**: barraCUDA ML ops (73.2% complete)
**What It Needs**:
- ⏳ Isomorphic IPC for NODE atomic
- ⏳ Communication with TOWER
- ⏳ Platform-agnostic transport
**Priority**: MEDIUM (needed for NODE atomic)
**Effort**: 6-8 hours (new implementation)

### 🔴 NEEDS EVOLUTION - nestgate
**Status**: Has universal filesystem, needs IPC
**Recent Work**: 
- ✅ Universal filesystem detection (Jan 31)
- ✅ 44% platform code reduction
- ✅ Cross-platform filesystem
**What It Needs**:
- ⏳ Isomorphic IPC for NEST atomic
- ⏳ Gateway communication
- ⏳ Multi-transport support
**Priority**: MEDIUM (needed for NEST atomic)
**Effort**: 6-8 hours (new implementation)

### 🔴 NEEDS EVOLUTION - squirrel
**Status**: Has universal transport, needs isomorphic IPC
**Recent Work**:
- ✅ Universal transport stack (Jan 30)
- ✅ NUCLEUS socket standardization
- ✅ Track 4 infrastructure (20% milestone)
**What It Needs**:
- ⏳ Isomorphic IPC pattern
- ⏳ Automatic fallback logic
- ⏳ Discovery integration
**Priority**: LOW (data layer, less critical)
**Effort**: 4-6 hours (integrate with existing transport)


═══════════════════════════════════════════════════════════════════

## 📊 PRIORITY MATRIX

### Immediate (Next Session)
1. **beardog** - HIGH Priority
   - Part of TOWER atomic (with songbird)
   - Has foundation (platform traits)
   - Direct pattern copy from songbird
   - **Effort**: 4-6 hours
   - **Impact**: Unblocks full TOWER testing

### Short-Term (1-2 weeks)
2. **toadstool** - MEDIUM Priority
   - Part of NODE atomic (TOWER + toadstool)
   - Currently focused on ML ops
   - New IPC implementation needed
   - **Effort**: 6-8 hours
   - **Impact**: Enables NODE atomic

3. **nestgate** - MEDIUM Priority
   - Part of NEST atomic
   - Has universal filesystem work
   - New IPC implementation needed
   - **Effort**: 6-8 hours
   - **Impact**: Enables NEST atomic

### Long-Term (Future)
4. **squirrel** - LOW Priority
   - Data layer (less critical)
   - Has transport infrastructure
   - Integration with existing code
   - **Effort**: 4-6 hours
   - **Impact**: Nice to have

═══════════════════════════════════════════════════════════════════

## 🔬 IMPLEMENTATION STRATEGY

### Pattern: Copy from songbird

**Core Files to Replicate**:
1. `ipc/pure_rust_server/server.rs` - Try→Detect→Adapt pattern
2. `crypto/socket_discovery.rs` - Client discovery
3. `beardog_client/core.rs` - IpcEndpoint support
4. `beardog_client/rpc.rs` - Polymorphic streams

**Key Methods**:
- `start()` - Entry point with fallback logic
- `try_unix_server()` - Unix socket attempt
- `is_platform_constraint()` - SELinux detection  
- `start_tcp_fallback()` - TCP adaptation
- `discover_ipc_endpoint()` - Client discovery

**Validation Steps**:
1. Build for ARM64 + x86_64
2. Test on Linux (Unix sockets)
3. Test on Android (TCP fallback)
4. Verify logs show automatic adaptation
5. Test inter-primal communication

═══════════════════════════════════════════════════════════════════

## 📋 ATOMIC DEPENDENCIES

### TOWER Atomic (beardog + songbird)
- ✅ songbird: READY
- 🟡 beardog: Needs isomorphic IPC
- **Blocker**: beardog evolution (4-6 hours)

### NODE Atomic (TOWER + toadstool)
- ✅ songbird: READY
- 🟡 beardog: Needs isomorphic IPC
- 🔴 toadstool: Needs isomorphic IPC
- **Blocker**: 2 primals need evolution (10-14 hours)

### NEST Atomic (networking gateway)
- 🔴 nestgate: Needs isomorphic IPC
- **Blocker**: 1 primal needs evolution (6-8 hours)

═══════════════════════════════════════════════════════════════════

## 🎯 RECOMMENDED EVOLUTION ORDER

### Phase 1: TOWER Completion (HIGH PRIORITY)
**Target**: Full TOWER atomic operational
**Tasks**:
1. Evolve beardog with isomorphic IPC (4-6 hours)
2. Test beardog ↔ songbird communication
3. Validate STUN handshake
4. Deploy to production

**Outcome**: Complete TOWER atomic (beardog + songbird)

### Phase 2: NODE Enablement (MEDIUM PRIORITY)
**Target**: NODE atomic operational
**Tasks**:
1. Evolve toadstool with isomorphic IPC (6-8 hours)
2. Test TOWER ↔ toadstool communication
3. Validate ML inference with IPC
4. Deploy NODE atomic

**Outcome**: Complete NODE atomic (TOWER + toadstool)

### Phase 3: NEST Enablement (MEDIUM PRIORITY)
**Target**: NEST atomic operational
**Tasks**:
1. Evolve nestgate with isomorphic IPC (6-8 hours)
2. Test gateway communication
3. Validate routing and federation
4. Deploy NEST atomic

**Outcome**: Complete NEST atomic

### Phase 4: Data Layer (LOW PRIORITY)
**Target**: Full ecosystem isomorphic
**Tasks**:
1. Evolve squirrel with isomorphic IPC (4-6 hours)
2. Integrate with existing transport
3. Test data layer communication
4. Complete ecosystem

**Outcome**: All primals isomorphic

═══════════════════════════════════════════════════════════════════

## 🎓 EFFORT ESTIMATES

**Total Effort to Complete All Primals**:
- beardog: 4-6 hours
- toadstool: 6-8 hours
- nestgate: 6-8 hours  
- squirrel: 4-6 hours
- **Total**: 20-28 hours (~3-4 sessions)

**Critical Path (TOWER only)**:
- beardog: 4-6 hours
- **Total**: 4-6 hours (1 session)

**Recommended**: Focus on beardog first (unblocks TOWER atomic)

═══════════════════════════════════════════════════════════════════

## 📈 CURRENT STATUS SUMMARY

| Primal | IPC Status | Priority | Effort | Blocks |
|--------|-----------|----------|--------|--------|
| **songbird** | ✅ COMPLETE | N/A | 0h | Nothing |
| **beardog** | 🟡 PARTIAL | HIGH | 4-6h | TOWER |
| **toadstool** | 🔴 TODO | MEDIUM | 6-8h | NODE |
| **nestgate** | 🔴 TODO | MEDIUM | 6-8h | NEST |
| **squirrel** | 🔴 TODO | LOW | 4-6h | - |

**Next Action**: Evolve beardog (4-6 hours to unblock TOWER)

═══════════════════════════════════════════════════════════════════

**Status**: Analysis Complete  
**Recommendation**: Prioritize beardog for TOWER atomic  
**Timeline**: 1 session for beardog, 3-4 sessions for full ecosystem
