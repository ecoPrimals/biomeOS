# 🐻 BearDog Zero-Hardcoding Request - Dynamic Port Support

**Date**: January 3, 2026  
**From**: biomeOS Team  
**To**: BearDog Team  
**Priority**: HIGH  
**Status**: 🔴 **BLOCKER** for Fractal Scaling

---

## 🎯 Issue Summary

**Problem**: BearDog hardcodes port `9000`, blocking fractal scaling and cloud-native deployments.

**Impact**:
- ❌ Cannot run multiple BearDog instances on same machine
- ❌ Port conflicts in containers/Kubernetes
- ❌ Blocks horizontal scaling
- ❌ Not cloud-native

**Solution**: Implement `HTTP_PORT=0` support (OS assigns available port)

---

## 📊 Current Behavior

### Test Results (January 3, 2026)

```bash
# Test: Dynamic port allocation
HTTP_PORT=0 ./beardog-server

# Expected:
✅ OS assigns available port (e.g., 37284)
✅ Logs show: "Server ready at http://127.0.0.1:37284"
✅ Service discoverable via mDNS

# Actual:
✅ BearDog starts
❌ Ignores HTTP_PORT=0
❌ Binds to hardcoded 9000
✅ Logs show: "Server ready at http://127.0.0.1:9000"
```

### Current Code (Suspected)

```rust
// Hardcoded port
let addr = SocketAddr::from(([127, 0, 0, 1], 9000));
let server = HttpServer::new(|| App::new()).bind(addr)?;
```

---

## ✅ Reference Implementation: Songbird

Songbird team completed zero-hardcoding migration (Jan 1, 2026).

**Documentation**: `/home/eastgate/Development/ecoPrimals/phase1/songbird/`
- `ZERO_HARDCODING_COMPLETE.md`
- `ZERO_HARDCODING_HANDOFF_TO_BIOMEOS.md`

### Songbird's Approach

```rust
use songbird_config::zero_hardcoding::EndpointConfig;

// Load config from environment
let config = EndpointConfig::from_env();

// HTTP_PORT env var, or 0 (OS assigns)
let addr = config.http_socket_addr(); // Returns SocketAddr with PORT=0

// Bind server
let server = HttpServer::new(|| App::new()).bind(addr)?;

// Get actual assigned port
let actual_addr = server.addrs()[0];
info!("✅ Server listening on {}", actual_addr);

// Announce for discovery
announce_via_mdns("_security._tcp", actual_addr.port());
```

**Result**: Songbird can run infinite instances, each with unique port!

---

## 🔧 Requested Implementation

### 1. Environment Variable Support

**Variable**: `HTTP_PORT`  
**Values**:
- `9000` (default) - Use specific port
- `0` (magic) - OS assigns available port
- `<any>` - Use specified port

**Example**:
```bash
# Default (backward compatible)
./beardog-server
# Binds to 9000

# Custom port
HTTP_PORT=8200 ./beardog-server
# Binds to 8200

# Dynamic port (NEW!)
HTTP_PORT=0 ./beardog-server
# OS assigns (e.g., 41237)
```

### 2. Code Changes

**In `beardog-server` main.rs or config**:

```rust
use std::net::SocketAddr;

// Read from environment
let port = std::env::var("HTTP_PORT")
    .unwrap_or_else(|_| "9000".to_string())
    .parse::<u16>()
    .unwrap_or(9000);

// Determine bind address
let bind_addr = std::env::var("BIND_ADDR")
    .unwrap_or_else(|_| "127.0.0.1".to_string());

// Create socket address (port 0 = OS assigns)
let addr: SocketAddr = format!("{}:{}", bind_addr, port)
    .parse()
    .expect("Invalid address format");

// Bind server
let server = HttpServer::new(|| {
    App::new()
        .service(health_handler)
        .service(trust_endpoints)
        .service(lineage_endpoints)
        .service(birdsong_endpoints)
})
.bind(addr)?;

// Get actual bound address
let actual_addr = server.addrs()[0];
info!("✅ BearDog API Server listening on {}", actual_addr);

if port == 0 {
    info!("🎯 Dynamic port assigned: {}", actual_addr.port());
}

// Announce for discovery (see next section)
announce_service(&actual_addr);

server.run().await
```

### 3. Service Discovery Announcement

**For Songbird to discover BearDog's dynamic port**:

```rust
use mdns_sd::{ServiceDaemon, ServiceInfo};

fn announce_service(addr: &SocketAddr) {
    // Create mDNS daemon
    let mdns = ServiceDaemon::new().expect("Failed to create mDNS daemon");
    
    // Register security capability
    let service_type = "_security._tcp.local.";
    let instance_name = format!("beardog-{}", hostname());
    let host_name = format!("{}.local.", hostname());
    
    let properties = [
        ("capability", "security"),
        ("primal", "beardog"),
        ("version", env!("CARGO_PKG_VERSION")),
        ("family", "iidn"), // If available
    ];
    
    let service_info = ServiceInfo::new(
        service_type,
        &instance_name,
        &host_name,
        addr.ip().to_string().as_str(),
        addr.port(),
        &properties[..],
    ).expect("Failed to create service info");
    
    mdns.register(service_info)
        .expect("Failed to register mDNS service");
    
    info!("📡 Announced security capability via mDNS");
    info!("   Service: {}", instance_name);
    info!("   Endpoint: http://{}:{}", addr.ip(), addr.port());
}
```

**Dependencies** (add to `Cargo.toml`):
```toml
[dependencies]
mdns-sd = "0.10"
```

---

## 🧪 Testing Strategy

### Test 1: Default Port (Backward Compatibility)

```bash
./beardog-server

# Expected logs:
✅ BearDog API Server listening on 127.0.0.1:9000
📡 Announced security capability via mDNS

# Verify:
curl http://localhost:9000/health
# Should return 200 OK
```

### Test 2: Custom Port

```bash
HTTP_PORT=8200 ./beardog-server

# Expected logs:
✅ BearDog API Server listening on 127.0.0.1:8200
📡 Announced security capability via mDNS

# Verify:
curl http://localhost:8200/health
# Should return 200 OK
```

### Test 3: Dynamic Port (NEW!)

```bash
HTTP_PORT=0 ./beardog-server

# Expected logs:
✅ BearDog API Server listening on 127.0.0.1:47281
🎯 Dynamic port assigned: 47281
📡 Announced security capability via mDNS

# Verify:
# Use mDNS discovery or check logs for actual port
curl http://localhost:47281/health
# Should return 200 OK
```

### Test 4: Multiple Instances (FRACTAL SCALING!)

```bash
# Terminal 1
HTTP_PORT=0 ./beardog-server
# Gets port 41234

# Terminal 2
HTTP_PORT=0 ./beardog-server
# Gets port 41235 (different!)

# Terminal 3
HTTP_PORT=0 ./beardog-server
# Gets port 41236 (different!)

# Result: 3 BearDog instances running simultaneously!
```

### Test 5: Songbird Discovery

```bash
# Start BearDog with dynamic port
HTTP_PORT=0 ./beardog-server
# Gets port 39472

# Start Songbird (no SECURITY_ENDPOINT set)
./songbird-orchestrator

# Expected Songbird logs:
🔍 Discovering security capability via mDNS...
✅ Found security provider at http://127.0.0.1:39472
✅ SecurityCapabilityClient connected

# Songbird discovers BearDog automatically!
```

---

## 📊 Benefits

### 1. Fractal Scaling ✅
- Run N instances on same machine
- Each gets unique port
- Perfect for load balancing

### 2. Cloud-Native ✅
- Kubernetes: `containerPort: 0`
- Docker: `-p 0:0`
- Auto-assigns available ports

### 3. Development ✅
- Test multiple configurations
- No port conflicts
- Parallel testing

### 4. Security ✅
- Randomized ports
- Harder to target
- Defense in depth

### 5. Simplicity ✅
- No port management
- OS handles availability
- Discovery handles routing

---

## 🎯 Acceptance Criteria

### Must Have

- [ ] `HTTP_PORT` environment variable supported
- [ ] `HTTP_PORT=0` enables dynamic allocation
- [ ] Logs show actual bound port
- [ ] mDNS announcement for discovery
- [ ] Backward compatible (default 9000)
- [ ] All tests passing

### Nice to Have

- [ ] `BIND_ADDR` environment variable (0.0.0.0 vs 127.0.0.1)
- [ ] IPv6 support
- [ ] UDP announcement (alternative to mDNS)
- [ ] Registry integration (Consul, etcd)

---

## 📚 Additional Resources

### Songbird's Implementation

**Files to Reference**:
- `/home/eastgate/Development/ecoPrimals/phase1/songbird/crates/songbird-config/src/zero_hardcoding/endpoints.rs`
- `/home/eastgate/Development/ecoPrimals/phase1/songbird/ZERO_HARDCODING_COMPLETE.md`

**Key Modules**:
- `EndpointConfig` - Port 0 support
- `CapabilityEndpoints` - Discovery config
- `UniversalAdapter` - mDNS discovery

### Industry Standards

- **Port 0**: IETF RFC 1700 (ephemeral ports)
- **mDNS**: IETF RFC 6762 (multicast DNS)
- **Service Discovery**: DNS-SD (RFC 6763)

---

## 🚀 Timeline

**Estimated Effort**: 2-3 hours  
**Priority**: HIGH  
**Target**: Next BearDog release

**Breakdown**:
- Environment variable support: 30 min
- mDNS announcement: 1 hour
- Testing: 1 hour
- Documentation: 30 min

---

## 📞 Contact

**Questions?**  
- Reference: Songbird's zero-hardcoding docs
- Slack: #beardog-development
- GitHub: Issue #XXX (to be created)

**Success Story?**  
- Show us your fractal scaling setup!
- Share performance benchmarks
- Help improve the ecosystem

---

## 🎊 Bottom Line

**This is a game-changer for the entire ecosystem!**

With PORT=0 support:
- ✅ BearDog becomes cloud-native
- ✅ Fractal scaling enabled
- ✅ Development experience improved
- ✅ Production deployments simplified

**Songbird has proven this works** - now let's bring BearDog up to the same standard!

---

**Status**: 🔴 REQUESTED  
**Priority**: HIGH  
**Impact**: Ecosystem-wide

🐻 **Let's make BearDog fractal-scaling ready!** 🚀

