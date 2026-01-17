# 🏠 NestGate Atomic Integration - Handoff

**Date**: January 12, 2026  
**From**: biomeOS Team  
**To**: NestGate Team  
**Priority**: HIGH  
**Status**: Unix Socket Configuration Needed

---

## 🎯 Quick Status

**What's Working**:
- ✅ NestGate v0.1.0 binary built and harvested
- ✅ Security-first design (JWT validation, no hardcoded localhost)
- ✅ Tower Atomic operational (BearDog + Songbird)

**What's Needed**:
- ⏳ Unix socket binding mode (currently using HTTP port 8080)
- ⏳ BearDog genetic key integration (when available)
- ⏳ Environment variable configuration guide

---

## 🔐 Security Architecture (Important!)

### Two-Tier Security Model

NestGate's current JWT security is **excellent design** - it provides:
- ✅ Standalone operation (no dependencies)
- ✅ Failsafe security by default
- ✅ Similar to HTTP as a fallback protocol

**However**, in the ecoPrimals ecosystem, we evolve to **BearDog genetic keys** when available:

```
Security Evolution:

Standalone Mode (Failsafe):
  JWT authentication ← Current implementation ✅
  HTTP binding ← Fallback protocol
  
Ecosystem Mode (Production):
  BearDog genetic keys ← Higher security level
  JSON-RPC over Unix sockets ← Primary protocol
  tarpc for high-performance IPC ← Secondary protocol
```

### Architecture Analogy

```
JWT : BearDog :: HTTP : JSON-RPC/tarpc

JWT/HTTP     = Standalone, failsafe, universal compatibility
BearDog/RPC  = Ecosystem-native, higher security, better performance
```

**Both are valid** - JWT for standalone/development, BearDog for production ecosystems.

---

## 🔌 Unix Socket Configuration Request

### Current Behavior

```bash
# What we see when launching NestGate:
$ export NESTGATE_SOCKET="/run/user/1000/nestgate-nat0.sock"
$ export NESTGATE_FAMILY_ID="nat0"
$ plasmidBin/nestgate service start

# Output:
🚀 Starting NestGate service on 127.0.0.1:8080
Error: Failed to bind to 127.0.0.1:8080: Address already in use
```

**Issue**: NestGate is binding to HTTP port instead of Unix socket.

### Expected Behavior

```bash
# What we want:
$ export NESTGATE_SOCKET="/run/user/1000/nestgate-nat0.sock"
$ export NESTGATE_FAMILY_ID="nat0"
$ export NESTGATE_NODE_ID="nest1"
$ plasmidBin/nestgate service start

# Expected output:
🏠 NestGate v2.0.0
🔌 Binding to Unix socket: /run/user/1000/nestgate-nat0.sock
✅ JSON-RPC server ready
✅ Registered with Songbird
```

---

## 📋 Configuration Requirements

### Environment Variables Needed

NestGate should support the **biomeOS socket configuration standard**:

| Variable | Purpose | Example | Priority |
|----------|---------|---------|----------|
| `NESTGATE_SOCKET` | Explicit socket path | `/run/user/1000/nestgate-nat0.sock` | 1 (Highest) |
| `NESTGATE_FAMILY_ID` | Genetic family identifier | `nat0`, `lan0` | N/A |
| `NESTGATE_NODE_ID` | Node identifier (multi-instance) | `nest1`, `nest-alpha` | N/A |
| `NESTGATE_DB_HOST` | Database host (if external DB) | `localhost`, `db.example.com` | Required |
| `NESTGATE_JWT_SECRET` | JWT secret (standalone mode) | `$(openssl rand -base64 48)` | Standalone |
| `NESTGATE_SECURITY_PROVIDER` | BearDog socket path (ecosystem mode) | `/run/user/1000/beardog-nat0.sock` | Ecosystem |

### Socket Path Priority (3-Tier Fallback)

```rust
// Recommended implementation (same as other primals)

fn get_socket_path() -> PathBuf {
    // 1. NESTGATE_SOCKET (highest priority - explicit override)
    if let Ok(socket_path) = std::env::var("NESTGATE_SOCKET") {
        return PathBuf::from(socket_path);
    }
    
    // 2. XDG Runtime Directory (preferred for production)
    let uid = get_uid(); // See note below
    let family_id = std::env::var("NESTGATE_FAMILY_ID")
        .unwrap_or_else(|_| "default".to_string());
    let xdg_runtime_dir = PathBuf::from(format!("/run/user/{}", uid));
    
    if xdg_runtime_dir.exists() {
        return xdg_runtime_dir.join(format!("nestgate-{}.sock", family_id));
    }
    
    // 3. Temp Directory (last resort)
    let node_id = std::env::var("NESTGATE_NODE_ID")
        .unwrap_or_else(|_| "default".to_string());
    PathBuf::from(format!("/tmp/nestgate-{}-{}.sock", family_id, node_id))
}

// Safe UID detection (no unsafe code)
fn get_uid() -> u32 {
    std::env::var("UID")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            std::fs::read_to_string("/proc/self/loginuid")
                .ok()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(1000) // Safe default
        })
}
```

---

## 🔐 BearDog Integration (When Available)

### Security Provider Discovery

When `NESTGATE_SECURITY_PROVIDER` is set, use BearDog for authentication instead of JWT:

```rust
pub enum SecurityMode {
    Standalone {
        jwt_secret: String,
    },
    Ecosystem {
        beardog_socket: PathBuf,
        genetic_family: String,
    },
}

impl SecurityMode {
    pub fn from_env() -> Result<Self> {
        // Check for BearDog (ecosystem mode)
        if let Ok(beardog_socket) = std::env::var("NESTGATE_SECURITY_PROVIDER") {
            let family = std::env::var("NESTGATE_FAMILY_ID")
                .unwrap_or_else(|_| "default".to_string());
            
            return Ok(SecurityMode::Ecosystem {
                beardog_socket: PathBuf::from(beardog_socket),
                genetic_family: family,
            });
        }
        
        // Fallback to JWT (standalone mode)
        let jwt_secret = std::env::var("NESTGATE_JWT_SECRET")?;
        
        // Validate JWT secret (your current validation is perfect! ✅)
        if jwt_secret == "CHANGE_ME_IN_PRODUCTION" {
            return Err(anyhow!("JWT secret must be set"));
        }
        
        Ok(SecurityMode::Standalone { jwt_secret })
    }
}
```

### Authentication Flow

```rust
async fn authenticate_request(&self, request: &Request) -> Result<Principal> {
    match &self.security_mode {
        SecurityMode::Standalone { jwt_secret } => {
            // Current JWT validation (keep this!)
            self.validate_jwt(request, jwt_secret)
        }
        SecurityMode::Ecosystem { beardog_socket, genetic_family } => {
            // Delegate to BearDog for genetic key validation
            self.validate_via_beardog(request, beardog_socket, genetic_family).await
        }
    }
}
```

**Key Insight**: BearDog genetic keys provide **cryptographic lineage verification** - not just authentication, but proof of genetic family membership.

---

## 📊 Comparison: Other Primals

### How BearDog, Songbird, and ToadStool Do It

All three now implement the biomeOS socket standard. Here's the pattern:

| Primal | Version | Socket Compliance | Status |
|--------|---------|-------------------|--------|
| **BearDog** | v0.16.1 | ✅ 3-tier fallback, XDG-compliant | Production-ready |
| **Songbird** | v3.22.0 | ✅ Pure Rust Unix socket server | Production-ready |
| **ToadStool** | v2.2.1 | ✅ Unix sockets + JSON-RPC | Operational |
| **NestGate** | v0.1.0 | ⏳ HTTP mode (needs Unix socket) | Pending |

### Reference Implementation

See Songbird v3.22.0 for the **gold standard**:
- Pure Rust Unix socket server (no `jsonrpsee` dependencies)
- Graceful fallback logic
- Atomic flags for concurrent-safe state
- XDG-compliant paths

---

## 🧪 Testing Checklist

### Required Test Scenarios

**Test 1: Environment Variable Override** ✅
```bash
export NESTGATE_SOCKET=/tmp/test-nestgate.sock
export NESTGATE_FAMILY_ID=test0
nestgate service start

# Should bind to: /tmp/test-nestgate.sock
```

**Test 2: XDG Runtime Directory** ✅
```bash
export NESTGATE_FAMILY_ID=nat0
export UID=1000
nestgate service start

# Should bind to: /run/user/1000/nestgate-nat0.sock (if XDG exists)
```

**Test 3: Fallback to /tmp** ✅
```bash
export NESTGATE_FAMILY_ID=test0
export NESTGATE_NODE_ID=node1
export UID=99999  # Non-existent
nestgate service start

# Should bind to: /tmp/nestgate-test0-node1.sock
```

**Test 4: BearDog Integration** (Phase 2)
```bash
export NESTGATE_SOCKET=/run/user/1000/nestgate-nat0.sock
export NESTGATE_SECURITY_PROVIDER=/run/user/1000/beardog-nat0.sock
export NESTGATE_FAMILY_ID=nat0
nestgate service start

# Should:
# 1. Bind to Unix socket ✅
# 2. Use BearDog for authentication ✅
# 3. Validate genetic lineage ✅
```

---

## 🎯 Integration with biomeOS Atomics

### Nest Atomic Architecture

```
Nest Atomic = Tower + NestGate
            = (BearDog + Songbird) + NestGate

Security Flow:
  BearDog: Encryption & genetic key validation
  Songbird: Service registry & discovery
  NestGate: Federated storage with BearDog-backed auth
  
Communication:
  Primary: JSON-RPC over Unix sockets
  Secondary: tarpc (high-performance IPC)
  Fallback: HTTP (standalone mode only)
```

### How It Works

1. **biomeOS launches Tower** (BearDog + Songbird)
2. **biomeOS launches NestGate** with:
   ```bash
   export NESTGATE_SOCKET="/run/user/1000/nestgate-nat0.sock"
   export NESTGATE_SECURITY_PROVIDER="/run/user/1000/beardog-nat0.sock"
   export NESTGATE_FAMILY_ID="nat0"
   ```
3. **NestGate starts**:
   - Binds to Unix socket ✅
   - Connects to BearDog for genetic key validation ✅
   - Registers with Songbird (capability: `storage.manage`) ✅
4. **Other primals discover NestGate** via Songbird
5. **All communication encrypted** via BearDog

---

## 🚀 What We Need from NestGate

### Phase 1: Unix Socket Support (Immediate)

**Required**:
1. ✅ Support `NESTGATE_SOCKET` environment variable (highest priority)
2. ✅ Implement 3-tier fallback (env var → XDG → /tmp)
3. ✅ JSON-RPC server over Unix socket
4. ✅ Parent directory creation (if socket path doesn't exist)
5. ✅ Old socket cleanup (prevent "address already in use")

**Timeline**: 2-4 hours (based on other primal implementations)

### Phase 2: BearDog Integration (Short-term)

**Required**:
1. ⏳ Support `NESTGATE_SECURITY_PROVIDER` environment variable
2. ⏳ BearDog genetic key authentication (when available)
3. ⏳ Graceful fallback to JWT (when BearDog unavailable)

**Timeline**: 1-2 days (includes testing)

### Phase 3: Songbird Registration (Short-term)

**Required**:
1. ⏳ Auto-register with Songbird on startup
2. ⏳ Announce capabilities: `storage.manage`, `storage.prepare`, `persistence`
3. ⏳ Health monitoring integration

**Timeline**: 1 day

---

## 📚 Reference Documents

### For NestGate Team

**Socket Configuration Examples**:
- BearDog v0.16.1 implementation (reference: phase1/beardog)
- Songbird v3.22.0 pure Rust Unix socket (reference: phase1/songbird)
- ToadStool v2.2.1 socket config (reference: phase1/toadstool)

**biomeOS Documentation**:
- `PRIMAL_SOCKET_CONFIG_HANDOFF.md` - Original socket standards
- `TOWER_ATOMIC_SUCCESS_JAN12.md` - Working example (BearDog + Songbird)
- `ATOMIC_DEPLOYMENT_PROGRESS_JAN12.md` - Current testing status

**Specifications**:
- `specs/ATOMIC_DEPLOYMENT_SYSTEM_SPEC.md` - Atomic architecture
- `specs/LIVESPORE_ARCHITECTURE_SPEC.md` - Future evolution

---

## 🎉 What's Great About NestGate

**Security-First Design** ✅
- JWT validation prevents insecure defaults
- No hardcoded localhost (forces explicit config)
- Database host must be set explicitly

**Modern Rust** ✅
- Comprehensive error handling
- Clear error messages
- Production-grade validation

**Ecosystem-Ready** ✅
- Already has service start command
- Good configuration architecture
- Ready for Unix socket integration

---

## 💡 Implementation Guidance

### Minimal Change Approach

You likely already have most of the pieces! The change is probably:

```rust
// Current (HTTP binding):
let addr = "127.0.0.1:8080".parse()?;
let listener = TcpListener::bind(addr).await?;

// Desired (Unix socket binding):
let socket_path = get_socket_path(); // Using 3-tier fallback
ensure_parent_directory(&socket_path)?;
remove_stale_socket(&socket_path)?;
let listener = UnixListener::bind(&socket_path)?;
```

**That's it!** The rest of your architecture (JSON-RPC, authentication, storage) stays the same.

---

## 🔗 Parallel Evolution

### Important Context

biomeOS is evolving **3 systems in parallel**:

1. **Atomic Deployment** (Current focus)
   - Tower ✅ Operational
   - Node ✅ Operational
   - Nest ⏳ Needs NestGate Unix sockets

2. **Neural API Integration** (Parallel)
   - AI-driven deployment
   - Graph learning
   - Adaptive orchestration

3. **LiveSpore** (Parallel)
   - Portable deployment (USB/bare metal/on-top-of-OS)
   - 12-week roadmap
   - 990-line specification

**These inform each other** - what we learn from atomic deployment guides Neural API and LiveSpore evolution.

---

## 📞 Contact & Coordination

**Status**: Ready for NestGate team implementation  
**Priority**: HIGH (blocks Nest atomic deployment)  
**Timeline**: Phase 1 (Unix sockets) - 2-4 hours  
**Confidence**: Very high (based on 3 successful primal integrations)

**Questions?** Happy to clarify any part of this handoff!

---

## 🎯 Success Criteria

**Phase 1 Complete When**:
- ✅ NestGate binds to Unix socket (not HTTP port)
- ✅ `NESTGATE_SOCKET` environment variable works
- ✅ XDG-compliant paths supported
- ✅ 3-tier fallback implemented
- ✅ Can deploy Nest Atomic successfully

**Phase 2 Complete When**:
- ✅ BearDog genetic key authentication working
- ✅ Graceful fallback to JWT standalone mode
- ✅ Can demonstrate encrypted, genetic-lineage-verified storage operations

---

**Different orders of the same architecture.** 🍄🐸

**We're excited to see NestGate join the ecosystem!** 🏠

---

*biomeOS: Pure Rust, Self-Sovereign, Federated Operating System*

**Prepared by**: biomeOS Team  
**Date**: January 12, 2026  
**Status**: Ready for handoff

