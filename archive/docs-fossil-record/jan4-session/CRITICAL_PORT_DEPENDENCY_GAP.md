# 🔴 CRITICAL ARCHITECTURAL GAP - Port Dependencies

**Date**: January 4, 2026  
**Status**: ⚠️ **BLOCKER** - Prevents dual spore federation  
**Issue**: Primals still using HTTP ports instead of pure UDP/Unix socket architecture

---

## 🎯 The Vision (Correct Architecture)

### Zero-Port Federation
```
Songbird (UDP Multicast)
    ↓ Discovery via 239.255.42.99:4242
    ↓ No HTTP ports needed!
    ↓
BearDog provides encryption
    ↓ Secures Songbird traffic
    ↓
Primals discover each other
    ↓ Via encrypted UDP multicast
    ↓
Unix sockets for local IPC
    ↓ /tmp/songbird-{family}.sock
    ✅ ZERO HARDCODED PORTS!
```

### Why This Matters
1. **Security**: Ports are attack surfaces
2. **Scalability**: No port conflicts ever
3. **Simplicity**: No port management needed
4. **Sovereignty**: Primals self-configure
5. **Fractal**: Infinite nesting possible

---

## 🔴 Current Gap

### BearDog Still Uses HTTP Ports
**Log shows**:
```
INFO beardog_server: 🚀 BearDog Server Starting on 127.0.0.1:9000
ERROR: Failed to bind to address: Address already in use (os error 98)
```

**Problem**: BearDog hardcodes `127.0.0.1:9000` instead of:
- Reading `BEARDOG_API_BIND_ADDR` env var
- OR using Unix socket exclusively
- OR binding to random available port

**Impact**: Cannot run multiple BearDog instances on same machine

---

## ✅ Correct Architecture

### BearDog Should:
1. **Primary**: Unix socket IPC only (`/tmp/beardog-{family}.sock`)
2. **Optional**: HTTP for external access (if needed)
   - Read `BEARDOG_API_BIND_ADDR` env var
   - Default to random available port
   - Or disable HTTP entirely (Unix socket only!)

### Songbird Should:
1. **UDP Multicast**: Discovery (already works! ✅)
2. **Unix Socket**: Registry API (`/tmp/songbird-{family}.sock`) ✅
3. **No HTTP ports**: Discovery doesn't need them

### Communication Flow:
```
Tower 1 BearDog
    ↓ Unix socket: /tmp/beardog-nat0-tower1.sock
    ↓
Tower 1 Songbird  
    ↓ UDP Multicast: 239.255.42.99:4242
    ↓ Encrypted with BearDog
    ↓
Tower 2 Songbird (discovers Tower 1!)
    ↓ Unix socket: /tmp/songbird-nat0-tower2.sock
    ↓
Tower 2 BearDog
    ↓ Unix socket: /tmp/beardog-nat0-tower2.sock

ZERO HTTP PORTS NEEDED!
```

---

## 📋 Handoff to Primal Teams

### BearDog Team - URGENT FIX

**Issue**: Ignoring `BEARDOG_API_BIND_ADDR` environment variable

**Current Code** (approximate):
```rust
// ❌ WRONG: Hardcoded address
let addr = "127.0.0.1:9000".parse()?;
```

**Should Be**:
```rust
// ✅ CORRECT: Read from environment
let addr = std::env::var("BEARDOG_API_BIND_ADDR")
    .unwrap_or_else(|_| "0.0.0.0:0".to_string())  // Port 0 = random
    .parse()?;
```

**Even Better**:
```rust
// ✅ BEST: Unix socket primary, HTTP optional
let use_http = std::env::var("BEARDOG_HTTP_ENABLED")
    .unwrap_or_else(|_| "false".to_string()) == "true";

if use_http {
    // HTTP for external access only
    let addr = std::env::var("BEARDOG_API_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:0".to_string());
    // Start HTTP server
} 

// Always provide Unix socket
let socket_path = format!("/tmp/beardog-{}.sock", family_id);
// Start Unix socket server
```

**Files to Check**:
- `crates/beardog-tunnel/src/api/server.rs`
- `crates/beardog/src/bin/beardog-server.rs`
- `crates/beardog-core/src/config.rs`

---

### Songbird Team - VERIFY

**Status**: ✅ Songbird already correct!
- UDP multicast working ✅
- Unix socket IPC working ✅
- No hardcoded HTTP ports ✅

**Just Confirm**:
- Can multiple Songbird instances run on same machine?
- Do they create unique Unix sockets?
- Does UDP multicast allow them to discover each other?

---

### biomeOS Team - DOCUMENT

**Current Status**: biomeOS orchestration is correct! ✅
- Concurrent startup ✅
- Capability-based resolution ✅
- Zero hardcoding ✅

**Need to Document**:
1. **Port-Free Architecture Guide**
   - Why we don't use HTTP ports
   - How UDP + Unix sockets replace them
   - Security benefits

2. **Update Integration Tests**
   - Test dual spore on same machine
   - Verify no port conflicts
   - Validate UDP discovery works

---

## 🎯 Success Criteria

### Dual Spore Federation (Same Machine)
- [ ] Spore 1 starts: BearDog + Songbird
- [ ] Spore 2 starts: BearDog + Songbird  
- [ ] No port conflicts (all use Unix sockets + UDP)
- [ ] Songbirds discover each other via UDP multicast
- [ ] BearDogs provide encryption for their respective Songbirds
- [ ] Both towers in family `nat0`
- [ ] Unique node IDs from genetic lineage

### Multi-Tower (Different Machines)
- [ ] Tower 1 on Machine 1
- [ ] Tower 2 on Machine 2
- [ ] Discover each other via UDP multicast over LAN
- [ ] Encrypted communication via BearDog
- [ ] Zero configuration (just family ID)

---

## 💡 Key Insight

**Quote from User**: 
> "songbird uses udp and makes ports irrelevant. besides they are a massive security risk. and beardog makes sure songbird is secure"

**Exactly!** This is the core architecture:
1. Songbird = Discovery (UDP multicast, no ports)
2. BearDog = Security (encrypts Songbird traffic)
3. Unix sockets = Local IPC (no ports)
4. **HTTP ports = Legacy, unnecessary, security risk**

---

## 🚀 Next Steps

### Immediate (BearDog)
1. Fix `BEARDOG_API_BIND_ADDR` env var reading
2. Consider making HTTP optional (Unix socket primary)
3. Test dual instance on same machine

### Validation
1. Deploy both spores on same machine
2. Verify no port conflicts
3. Confirm UDP discovery works
4. Test encrypted communication

### Production
1. Remove all HTTP port dependencies
2. Pure Unix socket + UDP architecture
3. Document as reference implementation

---

**Status**: ⚠️ **Waiting on BearDog fix** - Critical blocker for dual spore federation

**Timeline**: 1-2 hours to fix BearDog port handling

**Impact**: Once fixed, dual spore federation will work perfectly!

---

**Remember**: Ports are the old way. UDP + Unix sockets + BearDog encryption = The future! 🚀

