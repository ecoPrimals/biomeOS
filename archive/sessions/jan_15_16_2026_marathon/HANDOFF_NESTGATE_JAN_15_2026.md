# 🗄️ NestGate Handoff - Ready for BearDog Evolution

**Date**: January 15, 2026  
**From**: biomeOS Integration Team  
**To**: NestGate Evolution Team  
**Status**: ✅ Harvested & Ready for Security Evolution

---

## 🎯 **Mission**

Evolve NestGate from JWT-based HTTP security to **BearDog-based Unix socket security** for TRUE PRIMAL architecture.

---

## 📊 **Current Status**

### **What Works:**
- ✅ Built and harvested from phase1
- ✅ Binaries in `plasmidBin/primals/`
- ✅ Grade: B+ (88/100) - Production capable
- ✅ 3,607 tests passing (100%)
- ✅ ZFS integration complete
- ✅ Data sovereignty architecture
- ✅ Encryption at rest

### **Current Architecture:**
```
NestGate HTTP Server (port-based)
  ↓
JWT Authentication
  ↓
Storage Backend (ZFS, filesystem, etc.)
```

**Issues:**
- ❌ Uses HTTP (not TRUE PRIMAL)
- ❌ Uses JWT (should use BearDog)
- ❌ Port-based (should use Unix sockets)
- ❌ Hardcoded security (should discover BearDog)

---

## 🚀 **Evolution Goal**

### **Target Architecture:**
```
NestGate Unix Socket Server
  ↓
BearDog Security Provider (discovered at runtime)
  ↓
Songbird Discovery (for coordination)
  ↓
Storage Backend (ZFS, filesystem, etc.)
```

**Benefits:**
- ✅ TRUE PRIMAL (discovers BearDog, not hardcoded)
- ✅ Unix sockets (faster, more secure than HTTP)
- ✅ BearDog integration (unified security)
- ✅ No ports (port-free architecture)
- ✅ JSON-RPC 2.0 (isomorphic protocol)

---

## 🔧 **Technical Evolution Required**

### **1. Replace HTTP with Unix Socket Server**

**Current** (`code/crates/nestgate-bin/src/main.rs` or similar):
```rust
// HTTP server on port
let app = Router::new()
    .route("/api/v1/...", ...)
    .layer(Extension(jwt_validator));

axum::Server::bind(&"0.0.0.0:8080".parse()?)
    .serve(app.into_make_service())
    .await?;
```

**Target**:
```rust
use tokio::net::UnixListener;
use serde_json::{json, Value};

// Unix socket server
let socket_path = std::env::var("NESTGATE_SOCKET_PATH")
    .unwrap_or_else(|_| format!("/tmp/nestgate-{}.sock", family_id));

let listener = UnixListener::bind(&socket_path)?;
info!("🗄️ NestGate listening on {}", socket_path);

// JSON-RPC 2.0 handler
loop {
    let (stream, _) = listener.accept().await?;
    tokio::spawn(handle_jsonrpc_connection(stream, beardog_client.clone()));
}
```

---

### **2. Replace JWT with BearDog Security**

**Current**:
```rust
// JWT validation
let jwt_secret = std::env::var("JWT_SECRET")?;
let validator = JwtValidator::new(jwt_secret);

// Middleware
.layer(Extension(validator))
```

**Target**:
```rust
// Discover BearDog at runtime
let beardog_socket = discover_security_provider().await?;
let beardog_client = BeardogClient::connect(beardog_socket).await?;

// Validate requests via BearDog
async fn validate_request(request: &Value, beardog: &BeardogClient) -> Result<Identity> {
    let token = request["auth_token"].as_str()?;
    beardog.validate_token(token).await
}
```

**BearDog Discovery** (from biomeOS patterns):
```rust
async fn discover_security_provider() -> Result<PathBuf> {
    // Priority 1: Environment variable
    if let Ok(path) = std::env::var("NESTGATE_SECURITY_PROVIDER") {
        return Ok(PathBuf::from(path));
    }

    // Priority 2: Songbird discovery
    if let Ok(songbird) = discover_songbird().await {
        if let Ok(providers) = songbird.discover_by_capability("security").await {
            if let Some(beardog) = providers.first() {
                return Ok(PathBuf::from(&beardog.socket_path));
            }
        }
    }

    // Priority 3: Standard locations
    let family_id = std::env::var("FAMILY_ID").unwrap_or_else(|_| "default".to_string());
    let search_paths = vec![
        format!("/tmp/beardog-{}-default.sock", family_id),
        format!("/run/user/{}/beardog-{}-default.sock", get_uid(), family_id),
    ];

    for path in search_paths {
        if Path::new(&path).exists() {
            return Ok(PathBuf::from(path));
        }
    }

    Err(anyhow::anyhow!("BearDog not found. Is it running?"))
}
```

---

### **3. Implement JSON-RPC 2.0 Protocol**

**Request Format**:
```json
{
  "jsonrpc": "2.0",
  "method": "nestgate.create_dataset",
  "params": {
    "name": "tank/data",
    "compression": true,
    "auth_token": "beardog-token-here"
  },
  "id": 1
}
```

**Response Format**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "dataset_id": "tank/data",
    "created_at": "2026-01-15T02:00:00Z",
    "status": "active"
  },
  "id": 1
}
```

**Handler Pattern**:
```rust
async fn handle_jsonrpc_request(
    request: JsonRpcRequest,
    beardog: &BeardogClient,
    storage: &StorageBackend,
) -> Result<JsonRpcResponse> {
    // Validate via BearDog
    let identity = validate_request(&request.params, beardog).await?;

    // Route to handler
    let result = match request.method.as_str() {
        "nestgate.create_dataset" => create_dataset(&request.params, storage).await?,
        "nestgate.list_datasets" => list_datasets(&request.params, storage).await?,
        "nestgate.create_snapshot" => create_snapshot(&request.params, storage).await?,
        "nestgate.health_check" => health_check().await?,
        _ => return Err(anyhow::anyhow!("Unknown method: {}", request.method)),
    };

    Ok(JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(result),
        error: None,
        id: request.id,
    })
}
```

---

### **4. Register with Songbird**

**On Startup**:
```rust
async fn register_with_songbird(socket_path: &Path, family_id: &str) -> Result<()> {
    // Discover Songbird
    let songbird_socket = format!("/run/user/{}/songbird-{}.sock", get_uid(), family_id);
    if !Path::new(&songbird_socket).exists() {
        warn!("Songbird not found, running standalone");
        return Ok(());
    }

    let songbird = SongbirdClient::connect(&songbird_socket).await?;

    // Register capabilities
    songbird.register_primal(json!({
        "primal_type": "nestgate",
        "socket_path": socket_path.display().to_string(),
        "capabilities": ["storage", "persistence", "zfs", "snapshots", "encryption"],
        "family_id": family_id,
        "version": env!("CARGO_PKG_VERSION"),
    })).await?;

    info!("✅ Registered with Songbird");
    Ok(())
}
```

---

## 📁 **Files to Modify**

### **Priority 1: Core Server** (Week 1)
- `code/crates/nestgate-bin/src/main.rs` - Replace HTTP with Unix socket
- `code/crates/nestgate-core/src/server.rs` - JSON-RPC handler
- `code/crates/nestgate-core/src/auth.rs` - Replace JWT with BearDog

### **Priority 2: Discovery** (Week 2)
- `code/crates/nestgate-core/src/discovery.rs` - BearDog discovery
- `code/crates/nestgate-core/src/songbird.rs` - Songbird registration

### **Priority 3: Protocol** (Week 3)
- `code/crates/nestgate-core/src/jsonrpc.rs` - JSON-RPC types
- `code/crates/nestgate-core/src/handlers/` - Convert REST to JSON-RPC

### **Priority 4: Testing** (Week 4)
- `tests/integration/beardog_integration.rs` - BearDog tests
- `tests/integration/unix_socket_tests.rs` - Socket tests
- Update all existing tests for new protocol

---

## 🧪 **Testing Strategy**

### **Phase 1: Standalone Testing**
```bash
# 1. Start BearDog
export BEARDOG_FAMILY_ID=test
export BEARDOG_SOCKET_PATH=/tmp/beardog-test-default.sock
beardog-server &

# 2. Start NestGate (new Unix socket version)
export NESTGATE_FAMILY_ID=test
export NESTGATE_SOCKET_PATH=/tmp/nestgate-test.sock
export NESTGATE_SECURITY_PROVIDER=/tmp/beardog-test-default.sock
nestgate service start

# 3. Test JSON-RPC call
echo '{"jsonrpc":"2.0","method":"nestgate.health_check","params":{},"id":1}' | \
  nc -U /tmp/nestgate-test.sock
```

### **Phase 2: NUCLEUS Integration**
```bash
# Deploy full Nest atomic (Tower + NestGate)
target/release/nucleus execute --graph nest-atomic --family test
```

---

## 📊 **Current Deep Debt** (from CURRENT_STATUS.md)

### **Critical Issues:**
1. **Error Handling** (D+ 65/100): 2,579 unwrap()/expect() calls
2. **Hardcoding** (F 45/100): 2,949 hardcoded values
3. **Test Coverage** (C+ 78/100): 70% vs 90% target

### **Recommended Approach:**
**Don't fix all deep debt now.** Focus on:
1. ✅ Unix socket + BearDog integration (TRUE PRIMAL)
2. ✅ JSON-RPC protocol (isomorphic)
3. ✅ Songbird registration (discovery)
4. ⏳ Deep debt (tackle incrementally)

**Why:** Get NestGate working in NUCLEUS first, then evolve quality over time.

---

## 🎯 **Success Criteria**

### **Phase 1: Basic Integration** (Week 1-2)
- ✅ NestGate starts on Unix socket
- ✅ Discovers BearDog at runtime
- ✅ Validates requests via BearDog
- ✅ Responds with JSON-RPC 2.0

### **Phase 2: Full Integration** (Week 3-4)
- ✅ Registers with Songbird
- ✅ All storage operations work
- ✅ NUCLEUS can deploy Nest atomic
- ✅ Tests passing

### **Phase 3: Production Ready** (Week 5-8)
- ✅ Error handling improved (B+ grade)
- ✅ Hardcoding reduced (C+ grade)
- ✅ Test coverage increased (B grade)
- ✅ Documentation updated

---

## 💡 **Key Principles**

As you evolve NestGate, remember:

1. **Discovery Over Configuration**: Don't hardcode BearDog socket, discover it
2. **Capability-Based**: Query Songbird for "security" capability
3. **Graceful Degradation**: Work standalone if BearDog/Songbird unavailable
4. **TRUE PRIMAL**: Zero hardcoding, runtime discovery
5. **Incremental Evolution**: Get it working first, optimize later

---

## 🤝 **Collaboration**

### **Resources Available:**
- BearDog client patterns: `biomeOS/crates/biomeos-core/src/beardog_client.rs`
- Songbird client patterns: `biomeOS/crates/biomeos-core/src/songbird_client.rs`
- JSON-RPC patterns: `biomeOS/crates/biomeos-atomic-deploy/src/neural_api_server.rs`
- Unix socket patterns: All biomeOS primals

### **If You Need Help:**
- BearDog API questions → BearDog team
- Songbird discovery questions → Songbird team
- NUCLEUS deployment questions → biomeOS team

---

## 📚 **Related Documents**

In `phase1/nestgate/`:
- `CURRENT_STATUS.md` - Full status and deep debt analysis
- `README.md` - Architecture overview
- `docs/` - Comprehensive documentation

In `phase2/biomeOS/`:
- `NEURAL_API_EVOLUTION_JAN_15_2026.md` - JSON-RPC patterns
- `whitePaper/RootPulse/` - Advanced coordination patterns

---

## 🚀 **Recommended Evolution Path**

### **Week 1: Unix Socket Foundation**
1. Replace HTTP server with Unix socket listener
2. Implement basic JSON-RPC handler
3. Test standalone (no BearDog yet)

### **Week 2: BearDog Integration**
1. Implement BearDog discovery
2. Replace JWT validation with BearDog calls
3. Test with BearDog running

### **Week 3: Songbird Registration**
1. Discover Songbird at startup
2. Register NestGate capabilities
3. Test full Tower + NestGate (Nest atomic)

### **Week 4: NUCLEUS Integration**
1. Create Neural API graph for Nest deployment
2. Test via `nucleus execute`
3. Verify persistence works end-to-end

---

## 🎉 **What We Achieved**

**Before:**
- NestGate in phase1 (isolated)
- HTTP-based architecture
- JWT security
- Not integrated with biomeOS

**After:**
- ✅ Binaries harvested to plasmidBin/
- ✅ Clear evolution path defined
- ✅ BearDog integration patterns provided
- ✅ Ready for TRUE PRIMAL evolution

**Next:**
- Evolve to Unix sockets + BearDog
- Integrate with NUCLEUS
- Deploy full Nest atomic

---

## 📈 **Timeline Estimate**

- **Week 1-2**: Unix socket + BearDog (core evolution)
- **Week 3-4**: Songbird + NUCLEUS integration
- **Week 5-8**: Deep debt reduction (incremental)

**Total**: 4-8 weeks to production-ready Nest atomic

---

**Version**: 1.0.0  
**Handoff Date**: January 15, 2026  
**Status**: ✅ Ready for Evolution

🗄️ **Happy evolving, NestGate team!** 🚀

---

## 🔗 **Quick Reference**

**Current Binaries:**
- `plasmidBin/primals/nestgate` (4.7MB)
- `plasmidBin/primals/nestgate-client` (4.7MB)

**Source:**
- `phase1/nestgate/` (full codebase)

**Key Contacts:**
- BearDog team: Security integration
- Songbird team: Discovery patterns
- biomeOS team: NUCLEUS deployment

**The path is clear. Let's build the Nest! 🏠**

