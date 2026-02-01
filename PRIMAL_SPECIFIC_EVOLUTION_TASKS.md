# Primal-Specific Evolution Tasks

**Date**: January 31, 2026  
**Source**: ISOMORPHIC_IPC_IMPLEMENTATION_GUIDE.md

═══════════════════════════════════════════════════════════════════

## 🐻 BEARDOG - HIGH PRIORITY

**Status**: 🟡 PARTIAL (Has platform traits, needs isomorphic IPC)  
**Effort**: 4-6 hours  
**Blocks**: TOWER atomic  
**Next Session**: YES

### What You Have
- ✅ Universal platform traits (`PlatformStream`, `PlatformListener`)
- ✅ Android abstract socket support
- ✅ Unix socket IPC server
- ✅ Platform abstraction layer

### What You Need
1. **Add Try→Detect→Adapt pattern** to IPC server start
2. **Add TCP fallback server** (same JSON-RPC protocol)
3. **Add platform constraint detection** (SELinux check)
4. **Add discovery file system** (XDG-compliant)

### Files to Modify
- `crates/beardog-tunnel/src/unix_socket_ipc/server.rs` - Add fallback logic
- New: `crates/beardog-tunnel/src/tcp_ipc/server.rs` - TCP server
- New: `crates/beardog-tunnel/src/discovery.rs` - Discovery files

### Reference
Copy pattern from songbird's `server.rs` (lines 250-446)

### Testing
1. Build for ARM64
2. Deploy to Pixel 8a
3. Verify logs show TCP fallback
4. Test with songbird communication

**Priority**: 🔴 **CRITICAL** - Do this first!

═══════════════════════════════════════════════════════════════════

## 🍄 TOADSTOOL - MEDIUM PRIORITY

**Status**: 🔴 TODO (Has IPC, needs isomorphic pattern)  
**Effort**: 6-8 hours  
**Blocks**: NODE atomic  
**Next Session**: After beardog

### What You Have
- ✅ IPC infrastructure (`runtime/display/src/ipc/`)
- ✅ IPC server/client/types
- ✅ WebSocket support

### What You Need
1. **Evolve existing IPC server** with Try→Detect→Adapt
2. **Add TCP fallback** for Unix socket failures
3. **Add client discovery** for endpoint finding
4. **Test with TOWER** communication

### Files to Modify
- `crates/runtime/display/src/ipc/server.rs` - Add fallback pattern
- `crates/runtime/display/src/ipc/client.rs` - Add discovery
- New: `crates/runtime/display/src/ipc/tcp_server.rs` - TCP variant

### Reference
Study songbird's pattern, adapt to toadstool's architecture

### Testing
1. Build for ARM64
2. Test standalone
3. Test with TOWER atomic (beardog + songbird + toadstool)
4. Validate ML inference over IPC

**Priority**: 🟡 **MEDIUM** - Do after beardog

═══════════════════════════════════════════════════════════════════

## 🌐 NESTGATE - MEDIUM PRIORITY

**Status**: 🔴 TODO (Needs new IPC implementation)  
**Effort**: 6-8 hours  
**Blocks**: NEST atomic  
**Next Session**: After beardog + toadstool

### What You Have
- ✅ Universal filesystem detection
- ✅ 44% platform code reduction
- ✅ Cross-platform architecture

### What You Need
1. **Create new IPC module** following songbird pattern
2. **Implement server** with Try→Detect→Adapt
3. **Implement client discovery**
4. **Add gateway-specific routing**

### Files to Create
- New: `crates/nestgate-ipc/src/server.rs` - IPC server
- New: `crates/nestgate-ipc/src/client.rs` - IPC client
- New: `crates/nestgate-ipc/src/discovery.rs` - Discovery

### Reference
Start from songbird template, adapt for gateway needs

### Testing
1. Build for ARM64
2. Test standalone
3. Test NEST atomic routing
4. Validate federation communication

**Priority**: 🟡 **MEDIUM** - Do after toadstool

═══════════════════════════════════════════════════════════════════

## 🐿️ SQUIRREL - LOW PRIORITY

**Status**: 🔴 TODO (Has transport, needs IPC integration)  
**Effort**: 4-6 hours  
**Blocks**: Nothing critical  
**Next Session**: Future

### What You Have
- ✅ Universal transport stack
- ✅ NUCLEUS socket standardization
- ✅ Track 4 infrastructure

### What You Need
1. **Integrate isomorphic IPC** with existing transport
2. **Add Try→Detect→Adapt** for IPC layer
3. **Test data layer** communication

### Files to Modify
- Integrate with existing transport modules
- Add IPC-specific adaptations
- Connect to discovery system

### Reference
Adapt songbird pattern to data layer needs

### Testing
1. Build for ARM64
2. Test data operations
3. Test with other primals
4. Validate storage IPC

**Priority**: 🟢 **LOW** - Do when ecosystem complete

═══════════════════════════════════════════════════════════════════

## 📊 IMPLEMENTATION ROADMAP

### Session 1: beardog (HIGH)
**Duration**: 4-6 hours  
**Goal**: TOWER atomic complete  
**Outcome**: beardog + songbird fully operational

### Session 2: toadstool (MEDIUM)
**Duration**: 6-8 hours  
**Goal**: NODE atomic enabled  
**Outcome**: TOWER + toadstool working

### Session 3: nestgate (MEDIUM)
**Duration**: 6-8 hours  
**Goal**: NEST atomic enabled  
**Outcome**: Gateway/routing operational

### Session 4: squirrel (LOW)
**Duration**: 4-6 hours  
**Goal**: Complete ecosystem  
**Outcome**: All primals isomorphic

**Total**: 20-28 hours across 3-4 sessions

═══════════════════════════════════════════════════════════════════

## ✅ QUICK START FOR EACH PRIMAL

### beardog Team: Start Here
1. Read: `ISOMORPHIC_IPC_IMPLEMENTATION_GUIDE.md`
2. Study: songbird's `server.rs` (lines 250-446)
3. Copy: Try→Detect→Adapt pattern
4. Test: On Pixel 8a (device available)
5. Validate: Logs show TCP fallback

### toadstool Team: Start Here
1. Read: Implementation guide
2. Review: Your existing IPC in `runtime/display/src/ipc/server.rs`
3. Evolve: Add fallback pattern
4. Test: With TOWER atomic
5. Validate: NODE atomic works

### nestgate Team: Start Here
1. Read: Implementation guide
2. Study: songbird reference
3. Create: New IPC module
4. Test: Gateway routing
5. Validate: NEST atomic works

### squirrel Team: Start Here
1. Read: Implementation guide
2. Review: Universal transport
3. Integrate: IPC with transport
4. Test: Data operations
5. Validate: Full ecosystem

═══════════════════════════════════════════════════════════════════

**Document**: ISOMORPHIC_IPC_IMPLEMENTATION_GUIDE.md  
**Status**: Ready for Distribution  
**Pattern**: Validated in Production  
**Success Rate**: 100% (songbird)

Share this with all primal teams! 🚀
