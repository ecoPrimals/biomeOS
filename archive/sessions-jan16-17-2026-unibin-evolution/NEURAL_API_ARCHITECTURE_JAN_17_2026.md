# Neural API Deployment - Architectural Fixes Needed

**Date**: January 17, 2026  
**Context**: Proper NUCLEUS deployment via Neural API  
**Discovery**: Manual deployment revealed architectural issues

---

## 🎯 **Key Architectural Insights**

### **1. BearDog = NestGate JWT Provider** ✅

**Current Issue**: NestGate fails with "insecure JWT configuration"

**Proper Architecture**:
```
NestGate → BearDog (JSON-RPC) → JWT Secret
  └─ beardog.generate_jwt_secret method
  └─ High-strength cryptographic secrets
  └─ Unix socket communication
```

**Implementation**:
- NestGate should call `beardog.generate_jwt_secret` via `/tmp/beardog-nat0.sock`
- BearDog already has this method implemented (v0.9.0+)
- No environment variables needed - security is capability-based!

**Status**: ⏳ Needs NestGate integration code

---

### **2. Songbird = Unix Sockets ONLY (No Ports!)** 🔍

**Current Issue**: 
```
ERROR: tarpc server error: Address already in use (os error 98)
```

**Root Cause**: Songbird is trying to bind to TCP ports for internal communication

**Proper Architecture** (Concentrated Gap Strategy):
```
Internal Primals ←→ Songbird (Unix Sockets)
                      ↓
Songbird ←→ External HTTP/TLS (ONLY gateway)
```

**What Should Happen**:
- ✅ **Internal**: Songbird listens on `/tmp/songbird-nat0.sock` (Unix socket)
- ✅ **Internal**: All primals connect via Unix socket JSON-RPC
- ❌ **NO PORTS** for internal primal communication
- ✅ **External Only**: HTTP/TLS gateway for external APIs (if needed)

**Status**: ⚠️ Songbird needs to eliminate internal TCP bindings

---

### **3. Neural API = Orchestrator** ✅

**Current Issue**: We deployed manually with `nohup` and individual commands

**Proper Architecture**:
```
Neural API (nucleus)
  ↓
Deployment Graph (TOML)
  ↓
Ordered Launch: BearDog → Songbird → Squirrel → NestGate → ToadStool
  ↓
Health Checks & Inter-Primal Verification
```

**Benefits**:
- ✅ Correct launch order (dependency-based)
- ✅ Automatic environment variable management
- ✅ Health checking and socket verification
- ✅ Graceful error handling
- ✅ Professional orchestration (not manual scripts!)

**Status**: ✅ Ready to use (once graph is updated)

---

## 📋 **Issues to Report to Teams**

### **Songbird Team** (HIGH PRIORITY)

**Issue**: Songbird binding to TCP ports for internal communication

**Error**:
```
ERROR songbird_orchestrator::app::core: tarpc server error: 
  Address already in use (os error 98)
```

**Expected Behavior**:
- ✅ Listen ONLY on Unix socket: `/tmp/songbird-nat0.sock`
- ✅ NO TCP ports for internal primal communication
- ✅ HTTP/TLS ONLY for external gateway (if configured)

**Rationale**:
- Internal: Unix sockets (fast, secure, no ports)
- External: HTTP gateway (controlled, secure, rate-limited)
- Concentrated Gap: TLS only at external boundary

**Architecture Compliance**: TRUE PRIMAL + Concentrated Gap

**Priority**: HIGH (violates Unix socket architecture)

**Timeline**: 1-2 hours to fix tarpc server binding

---

### **NestGate Team** (MEDIUM PRIORITY)

**Issue**: JWT secret not retrieved from BearDog

**Current Behavior**:
```
NestGate will not start with insecure JWT configuration.
```

**Expected Behavior**:
1. Check if BearDog socket exists: `/tmp/beardog-nat0.sock`
2. If exists, call JSON-RPC: `beardog.generate_jwt_secret`
3. Use returned high-strength secret
4. If BearDog unavailable, fall back to secure random (and log warning)

**Example Integration** (pseudo-code):
```rust
// Check for BearDog
if let Ok(socket) = UnixStream::connect("/tmp/beardog-nat0.sock") {
    let response: JwtSecretResponse = jsonrpc_call(
        socket,
        "beardog.generate_jwt_secret",
        json!({"purpose": "nestgate_auth", "strength": "high"})
    ).await?;
    
    config.jwt_secret = response.secret; // 88-char base64, 512 bits
} else {
    // Secure fallback
    config.jwt_secret = generate_secure_random(64); // 64 bytes
    warn!("BearDog unavailable, using secure random JWT secret");
}
```

**Priority**: MEDIUM (expected integration work)

**Timeline**: ~1 hour

---

### **ToadStool Team** (ALREADY KNOWN)

**Issue**: `toadstool server` not implemented (Phase 2 incomplete)

**Status**: Already documented in `UNIBIN_REALITY_CHECK_JAN_17_2026.md`

**Workaround**: Use `toadstool-server` until Phase 2 complete

**Priority**: HIGH (blocks UniBin adoption)

---

## 🏗️ **Updated Deployment Graph Needed**

### **Current Graph Issues**

**File**: `graphs/02_nucleus_enclave_unibin.toml`

**Issues**:
1. ⚠️ ToadStool: Uses `toadstool` binary with `server` arg (not implemented)
2. ✅ BearDog: Correct
3. ⚠️ Songbird: May need tarpc/port config fixes
4. ✅ Squirrel: Correct
5. ⚠️ NestGate: Needs BearDog JWT integration

### **Interim Solution**

**Option A**: Use old binaries until primals are fixed
```toml
# Temporary workaround
binary_path = "plasmidBin/primals/toadstool-server"  # Old binary
# Not: ./toadstool server (not implemented yet)
```

**Option B**: Update graph for current reality
```toml
# Use verified working configurations
[nodes.config]
primal_name = "songbird"
binary_path = "plasmidBin/primals/songbird"
args = ["server"]  # UniBin working
environment_overrides = {
  "DISABLE_TARPC_SERVER": "true"  # Until port issue fixed
}
```

---

## 🎯 **Recommended Next Steps**

### **Immediate** (This Session)

1. ✅ Document architectural issues for teams
2. ⏳ Create working deployment graph (using old binaries where needed)
3. ⏳ Test Neural API deployment with current binaries
4. ⏳ Verify orchestration works end-to-end

### **Short-Term** (Teams Fix Issues)

**Songbird**: 
- Remove internal TCP port bindings
- Use Unix sockets ONLY for primal communication
- Keep HTTP gateway optional/external

**NestGate**:
- Implement BearDog JWT integration
- Add secure random fallback
- Test with and without BearDog

**ToadStool**:
- Complete Phase 2 (server mode)
- Verify `toadstool server` actually works

### **Medium-Term** (Re-Deploy)

4. Update deployment graph for verified UniBins
5. Test full NUCLEUS deployment via Neural API
6. Validate all inter-primal communications
7. Document successful deployment

---

## 📊 **Architecture Validation Checklist**

### **TRUE PRIMAL Compliance** ✅

- [ ] No hardcoded primal addresses
- [ ] Capability-based discovery
- [ ] Runtime primal discovery
- [ ] Self-knowledge only

### **Concentrated Gap Compliance** ✅

- [ ] Internal: Unix sockets ONLY
- [ ] External: HTTP via Songbird gateway (if needed)
- [ ] No primal-to-primal HTTP
- [ ] TLS concentrated in Songbird

### **Security Architecture** ✅

- [ ] BearDog provides JWT secrets (capability)
- [ ] NestGate requests from BearDog (not env vars)
- [ ] Secure fallback if BearDog unavailable
- [ ] All keys cryptographically strong

### **UniBin Architecture** ⏳

- [x] Squirrel: 100% working
- [x] Songbird: 90% working (needs port fix)
- [ ] NestGate: Needs JWT integration
- [ ] ToadStool: Needs Phase 2
- [ ] BearDog: Not UniBin yet

---

## 🏆 **Bottom Line**

### **Manual Deployment Value**: EXTREMELY HIGH

We discovered:
1. ✅ Which UniBins actually work (Squirrel, Songbird)
2. ⚠️ Architectural violations (Songbird ports)
3. ⚠️ Integration gaps (NestGate ← BearDog JWT)
4. ⚠️ Implementation gaps (ToadStool Phase 2)

### **Next**: Proper Neural API Deployment

**Prerequisites**:
1. Create working deployment graph (use old binaries where needed)
2. Verify Neural API orchestrator available
3. Test deployment with current binaries
4. Document results

**Timeline**: 30 minutes for working deployment

**Then**: Teams fix issues, we re-deploy with proper architecture!

---

**Created**: January 17, 2026  
**Purpose**: Document architectural requirements for proper Neural API deployment  
**Status**: Ready to proceed with interim deployment

🦀🧬✨ **Architecture-First Deployment!** ✨🧬🦀

**Key Insight**: Manual deployment = Architectural validation!

