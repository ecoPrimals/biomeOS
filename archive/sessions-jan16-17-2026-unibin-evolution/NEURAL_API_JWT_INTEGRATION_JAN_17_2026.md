# Neural API - BearDog JWT Integration Requirement

**Date**: January 17, 2026  
**Priority**: HIGH  
**Component**: Neural API (nucleus orchestrator)  
**Requirement**: Automatic JWT secret provisioning from BearDog to NestGate

---

## 🎯 **Executive Summary**

The Neural API orchestrator should automatically handle JWT secret provisioning from BearDog to NestGate during NUCLEUS deployment. This is proper orchestration architecture - the orchestrator manages inter-primal dependencies, not individual primals!

**Current Issue**: NestGate fails with "insecure JWT configuration"  
**Solution**: Neural API fetches JWT from BearDog, passes to NestGate  
**Architecture**: Orchestrator-managed integration (clean separation of concerns)

---

## 🏗️ **Architecture: Orchestrator Responsibility**

### **Why Neural API Should Handle This**

**Orchestrator Pattern** ✅:
```
Neural API (Orchestrator)
  ↓
1. Launch BearDog (security provider)
2. Wait for BearDog socket ready
3. Call beardog.generate_jwt_secret (JSON-RPC)
4. Pass JWT secret to NestGate (env var or config)
5. Launch NestGate with JWT configured
6. Verify NestGate starts successfully
```

**NOT Primal's Responsibility** ❌:
```
NestGate (Primal)
  ↓
1. Try to connect to BearDog...
2. Hope BearDog is there...
3. Implement BearDog client logic...
4. Handle errors and retries...
5. Tightly couple to BearDog...
```

**Rationale**:
- ✅ **Separation of Concerns**: Orchestrator manages dependencies, primals focus on their capabilities
- ✅ **Clean Architecture**: NestGate doesn't need BearDog client code
- ✅ **Easier Testing**: Can test NestGate with any JWT secret
- ✅ **Flexibility**: Can switch JWT providers without changing NestGate
- ✅ **Proper Orchestration**: This is literally what orchestrators are for!

---

## 📋 **Implementation Steps for Neural API**

### **1. Add BearDog JWT Client to Neural API**

**File**: `crates/biomeos-atomic-deploy/src/beardog_jwt_client.rs` (new)

```rust
use serde::{Deserialize, Serialize};
use tokio::net::UnixStream;

#[derive(Debug, Serialize)]
struct JwtSecretRequest {
    purpose: String,
    strength: String,
}

#[derive(Debug, Deserialize)]
struct JwtSecretResponse {
    secret: String,
    purpose: String,
    strength: String,
    byte_length: usize,
}

pub async fn fetch_jwt_secret_from_beardog(
    socket_path: &str,
    purpose: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Connect to BearDog via Unix socket
    let stream = UnixStream::connect(socket_path).await?;
    
    // Send JSON-RPC request
    let request = json!({
        "jsonrpc": "2.0",
        "method": "beardog.generate_jwt_secret",
        "params": {
            "purpose": purpose,
            "strength": "high"  // 512 bits, production-ready
        },
        "id": 1
    });
    
    // ... send request, parse response ...
    
    Ok(response.secret)
}
```

---

### **2. Update Graph Executor for JWT Provisioning**

**File**: `crates/biomeos-atomic-deploy/src/neural_executor.rs`

**Add JWT provisioning step**:

```rust
async fn launch_primal(&self, node: &GraphNode) -> Result<()> {
    let config = &node.config;
    
    // Check if this primal needs JWT secret
    if config.requires_jwt_secret {
        info!("Primal {} requires JWT secret, fetching from BearDog...", config.primal_name);
        
        // Find BearDog socket from dependencies
        let beardog_socket = self.find_dependency_socket("beardog")?;
        
        // Fetch JWT secret
        let jwt_secret = fetch_jwt_secret_from_beardog(
            &beardog_socket,
            &format!("{}_authentication", config.primal_name)
        ).await?;
        
        info!("✅ JWT secret obtained from BearDog ({} bytes)", jwt_secret.len());
        
        // Add to environment
        let mut env = config.environment.clone();
        env.insert(
            format!("{}_JWT_SECRET", config.primal_name.to_uppercase()),
            jwt_secret
        );
        
        // Launch with JWT configured
        self.launch_with_env(config, env).await?;
    } else {
        // Normal launch
        self.launch_with_env(config, config.environment.clone()).await?;
    }
    
    Ok(())
}
```

---

### **3. Update Deployment Graph Schema**

**File**: `graphs/02_nucleus_enclave_unibin.toml`

**Add JWT requirement flag**:

```toml
# Phase 4: Storage Foundation (Nest = NestGate)
[[nodes]]
id = "launch_nestgate"
node_type = "primal.launch"
description = "Launch NestGate (Nest atomic) for sovereign storage and persistence"
depends_on = ["launch_toadstool"]

[nodes.config]
primal_name = "nestgate"
binary_path = "plasmidBin/primals/nestgate"
args = ["service", "start"]
family_id = "nat0"
socket_path = "/tmp/nestgate-nat0.sock"
security_provider = "/tmp/beardog-nat0.sock"
capabilities = ["storage", "persistence"]
startup_timeout_seconds = 30

# JWT Configuration (Orchestrator-managed)
requires_jwt_secret = true           # ← NEW FLAG
jwt_purpose = "nestgate_authentication"
jwt_strength = "high"                 # 512 bits

[[nodes.health_check]]
type = "socket_exists"
path = "/tmp/nestgate-nat0.sock"
```

---

### **4. Handle Fallback Scenarios**

**Secure Fallback** (if BearDog unavailable):

```rust
async fn provision_jwt_secret(&self, config: &PrimalConfig) -> Result<String> {
    // Try BearDog first (preferred)
    if let Some(beardog_socket) = self.find_optional_dependency_socket("beardog") {
        match fetch_jwt_secret_from_beardog(&beardog_socket, &config.jwt_purpose).await {
            Ok(secret) => {
                info!("✅ JWT secret from BearDog (preferred)");
                return Ok(secret);
            }
            Err(e) => {
                warn!("⚠️ BearDog JWT fetch failed: {}, using secure fallback", e);
            }
        }
    }
    
    // Secure fallback: generate cryptographically strong random
    let secret = generate_secure_random_base64(64)?; // 64 bytes = 512 bits
    warn!("⚠️ Using secure random JWT secret (BearDog unavailable)");
    
    Ok(secret)
}

fn generate_secure_random_base64(bytes: usize) -> Result<String> {
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    let mut secret = vec![0u8; bytes];
    rng.fill_bytes(&mut secret);
    Ok(base64::encode(&secret))
}
```

---

## 📊 **Benefits of This Approach**

### **For NestGate** ✅
- ✅ No BearDog client code needed
- ✅ No error handling for BearDog communication
- ✅ Simpler, more focused codebase
- ✅ Easier to test (just provide JWT secret)
- ✅ Can run standalone with any JWT secret

### **For Neural API** ✅
- ✅ Proper orchestration responsibility
- ✅ Centralized dependency management
- ✅ Consistent error handling
- ✅ Flexibility to change JWT providers
- ✅ Better logging and observability

### **For Ecosystem** ✅
- ✅ Clean separation of concerns
- ✅ Reusable pattern for other primals
- ✅ Orchestrator manages all inter-primal integration
- ✅ Primals stay focused on their capabilities
- ✅ Professional architecture

---

## 🎯 **Deployment Flow with JWT Provisioning**

### **Correct Orchestrated Sequence**

```
1. Neural API starts
   ↓
2. Launch BearDog (Phase 0: Security Foundation)
   → BearDog starts, creates /tmp/beardog-nat0.sock
   ↓
3. Health check: BearDog socket exists
   ↓
4. Launch Songbird (depends on BearDog)
   → Songbird starts
   ↓
5. Launch Squirrel (depends on Songbird)
   → Squirrel starts
   ↓
6. Launch ToadStool (depends on Squirrel)
   → ToadStool starts
   ↓
7. Prepare NestGate launch (requires JWT):
   a. Neural API connects to BearDog socket
   b. Neural API: "beardog.generate_jwt_secret"
   c. BearDog: "Here's your 512-bit secret: ..."
   d. Neural API sets NESTGATE_JWT_SECRET env var
   ↓
8. Launch NestGate (with JWT configured)
   → NestGate starts successfully! ✅
   ↓
9. Health check: All primals running
   ↓
10. NUCLEUS deployed! 🎉
```

**Key**: Neural API handles step 7 automatically - NestGate just receives configured JWT!

---

## 📋 **Implementation Checklist**

### **Phase 1: Core JWT Client** (30 min)
- [ ] Create `beardog_jwt_client.rs` in Neural API
- [ ] Implement JSON-RPC over Unix socket
- [ ] Add error handling and retries
- [ ] Add secure random fallback
- [ ] Unit tests

### **Phase 2: Graph Executor Integration** (30 min)
- [ ] Add JWT provisioning logic to `neural_executor.rs`
- [ ] Check `requires_jwt_secret` flag in graph
- [ ] Fetch from BearDog if flag set
- [ ] Pass via environment variable
- [ ] Integration tests

### **Phase 3: Graph Schema Update** (15 min)
- [ ] Add `requires_jwt_secret` to NestGate node
- [ ] Add `jwt_purpose` and `jwt_strength` fields
- [ ] Update graph documentation
- [ ] Validate graph still parses

### **Phase 4: Testing** (30 min)
- [ ] Test with BearDog available
- [ ] Test with BearDog unavailable (fallback)
- [ ] Test JWT secret strength (512 bits)
- [ ] Test NestGate starts successfully
- [ ] End-to-end NUCLEUS deployment

**Total Estimated Time**: ~2 hours

---

## 🚀 **Expected Results**

### **Before** (Current State)
```bash
$ nucleus deploy --family nat0
...
✅ BearDog: Running
✅ Songbird: Running
✅ Squirrel: Running
✅ ToadStool: Running
❌ NestGate: FAILED
   Error: "NestGate will not start with insecure JWT configuration"
```

### **After** (With JWT Provisioning)
```bash
$ nucleus deploy --family nat0
...
✅ BearDog: Running
   → JWT provider ready
✅ Songbird: Running
✅ Squirrel: Running
✅ ToadStool: Running
🔐 Fetching JWT secret from BearDog...
   → JWT secret obtained (88 chars, 512 bits)
✅ NestGate: Running
   → Launched with BearDog-provided JWT
✅ NUCLEUS: Complete! All 5 primals operational
```

---

## 📚 **Related Documentation**

- **BearDog JWT API**: `/ecoPrimals/phase1/beardog/docs/JWT_SECRET_GENERATION_COMPLETE.md`
- **Neural API Architecture**: `NEURAL_API_ARCHITECTURE_JAN_17_2026.md`
- **Deployment Graphs**: `graphs/02_nucleus_enclave_unibin.toml`
- **UniBin Reality Check**: `UNIBIN_REALITY_CHECK_JAN_17_2026.md`

---

## 🏆 **Bottom Line**

### **Neural API's Responsibility**: Orchestration & Integration

**Orchestrator Should**:
- ✅ Manage deployment order
- ✅ Handle inter-primal dependencies
- ✅ Provision secrets and configuration
- ✅ Verify health and connectivity
- ✅ Provide error handling and retries

**Primals Should**:
- ✅ Focus on their core capabilities
- ✅ Accept configuration via env/config
- ✅ Expose JSON-RPC APIs
- ✅ Handle their own business logic

**This is Clean Architecture** ✅

---

**Created**: January 17, 2026  
**Priority**: HIGH  
**Timeline**: ~2 hours implementation  
**Impact**: Completes NUCLEUS orchestration, eliminates NestGate startup failure

🦀🧬✨ **Orchestrator-Managed JWT Integration!** ✨🧬🦀

**Let the orchestrator orchestrate!**

