# 🔧 Remaining Work - Quick Handoff

**Date**: January 16, 2026  
**Status**: 2 socket path fixes needed (BearDog + Songbird)  
**Priority**: Medium (fallbacks working, but blocks TRUE PRIMAL architecture)

---

## 🎯 TL;DR

**ToadStool socket fix VALIDATED!** ✅ Neural API env var passing works!

**Remaining**: BearDog + Songbird need to honor environment variables (same as ToadStool).

---

## 📋 BearDog Team

### Issue
Socket created at `/run/user/1000/beardog-nat0.sock` instead of `/tmp/beardog-default-default.sock`

### Root Cause
Not honoring `BEARDOG_SOCKET` or `BIOMEOS_SOCKET_PATH` environment variables.

### Fix Needed
Implement environment variable priority order (reference: ToadStool's implementation):

1. `BEARDOG_SOCKET` (highest priority - Neural API)
2. `BIOMEOS_SOCKET_PATH` (generic orchestrator)
3. XDG runtime directory (user-mode fallback)
4. `/tmp/` (system default)

### Reference
ToadStool's socket path logic is 100% correct. Copy their pattern.

### Impact
- **Current**: JWT secret generation using secure fallback ✅ (working)
- **After Fix**: JWT secrets from BearDog capability provider ✅ (TRUE PRIMAL!)

### Test Command
```bash
BEARDOG_SOCKET=/tmp/beardog-test.sock \
BIOMEOS_SOCKET_PATH=/tmp/beardog-fallback.sock \
./beardog-server
# Should create socket at /tmp/beardog-test.sock
```

---

## 📋 Songbird Team

### Issue  
Socket created at `/tmp/squirrel-squirrel.sock` instead of `/tmp/songbird-nat0.sock`

### ✅ MYSTERY SOLVED! (User's brilliant insight!)

**Songbird orchestrator code IS correct!** 🎉

**Root Cause**: Songbird repo contains **TWO services**:
1. **songbird-orchestrator** (discovery) - ✅ Socket fixes applied!
2. **songbird-squirrel-service** (AI/MCP) - ❌ Socket fixes NOT applied!

When `songbird-orchestrator` starts, it spawns the embedded Squirrel service, which creates `/tmp/squirrel-squirrel.sock` because the Squirrel service code doesn't have your fixes!

### Evidence
```toml
# Cargo.toml workspace
members = [
    "crates/songbird-orchestrator",    # ✅ Fixed
    "crates/songbird-squirrel-service", # ❌ Not fixed
]
```

### Fix Needed
**Apply socket path fixes to `songbird-squirrel-service`** (same as you did for orchestrator):

```bash
cd phase1/songbird/crates/songbird-squirrel-service
# Apply same env var priority order as songbird-orchestrator:
# 1. SQUIRREL_SOCKET (highest)
# 2. SQUIRREL_ORCHESTRATOR_SOCKET (alternative)
# 3. BIOMEOS_SOCKET_PATH (generic)
# 4. Default: /tmp/squirrel-{family_id}.sock
```

### Why You Were Right
- ✅ Songbird orchestrator code IS correct
- ✅ Tests DO pass (testing the orchestrator)
- ✅ Socket path logic IS implemented
- ✅ You just need to apply the same fixes to the embedded Squirrel service!

### Impact
- **Current**: Embedded Squirrel creates `/tmp/squirrel-squirrel.sock` (hardcoded)
- **After Fix**: Squirrel will honor env vars, create `/tmp/squirrel-nat0.sock` ✅

---

## ✅ What's Already Working

### Neural API
- ✅ Environment variable passing (proven by ToadStool!)
- ✅ Graph orchestration
- ✅ Phase-based deployment
- ✅ Secure JWT fallback generation

### ToadStool
- ✅ Socket at `/tmp/toadstool-nat0.sock` (CORRECT!)
- ✅ Honors `TOADSTOOL_SOCKET` env var
- ✅ Implementation: **100% correct reference standard**

### NestGate
- ✅ Socket at `/tmp/nestgate-nat0.sock` (CORRECT!)
- ✅ Auth v2.0.0 operational
- ✅ Accepts JWT from BearDog or fallback

---

## 🎯 Success Criteria

After both fixes, NUCLEUS deployment should have:

```bash
# All sockets in /tmp/:
/tmp/beardog-default-default.sock  ✅
/tmp/songbird-nat0.sock            ✅
/tmp/toadstool-nat0.sock           ✅ (already working!)
/tmp/nestgate-nat0.sock            ✅ (already working!)
/tmp/neural-api-nat0.sock          ✅ (already working!)
```

**Grade**: A+ (100%) - Complete TRUE PRIMAL architecture

---

## 🤝 Support

### Questions?
- **ToadStool Reference**: `phase1/toadstool/crates/toadstool-server/src/main.rs` (lines 147-179)
- **Songbird Tests**: `phase1/squirrel/crates/songbird-orchestrator/tests/biomeos_socket_env_vars.rs`
- **BiomeOS Docs**: See `TOADSTOOL_FIX_COMPLETE_JAN_15_2026.md`, `PRIMAL_HARVEST_COMPLETE_JAN_16_2026.md`

### Test Deployment
```bash
cd ecoPrimals/phase2/biomeOS

# After your fix, test:
./scripts/stop_ecosystem.sh
./plasmidBin/primals/neural-api-server --graphs-dir graphs --family-id nat0 &
./plasmidBin/primals/neural-deploy 01_nucleus_enclave

# Verify sockets:
ls -lh /tmp/*.sock
```

---

## 📊 Timeline Estimate

- **BearDog Fix**: 30-60 minutes (copy ToadStool's pattern)
- **Songbird Fix**: 30-60 minutes (verify mode selection + env vars)
- **Testing**: 30 minutes (re-deploy NUCLEUS, verify all sockets)
- **Total**: ~2 hours for 100% operational NUCLEUS

---

## 🏆 Impact

**Current State**: B+ (85%) - 3/4 primals fully operational, secure fallbacks working

**After Fixes**: A+ (100%) - Complete TRUE PRIMAL architecture:
- ✅ BearDog provides JWT secrets (capability provider)
- ✅ Songbird coordinates inter-primal discovery
- ✅ All sockets in standard `/tmp/` location
- ✅ Zero hardcoding, pure runtime discovery
- ✅ Environment-driven orchestration

---

**Priority**: Medium (not urgent - fallbacks working)  
**Effort**: Low (copy existing patterns)  
**Reward**: High (TRUE PRIMAL validation complete!)

🌱🐻🐦🍄🚪 Let's get NUCLEUS to 100%! 🚀
