# 🧠 Neural API: Routing Architecture - January 20, 2026

**Date**: January 20, 2026  
**Status**: ⚠️  **CRITICAL INSIGHT - ONLY BEGUN INFRASTRUCTURE**  
**Realization**: Neural API is BOTH Deployment AND Interaction Layer!

---

## 🎯 **THE INSIGHT**

### **We've Built Deployment, But Not Routing!**

**What We Have** ✅:
- Neural API launches primals (capability-based discovery)
- Process management (spawn, track PID, verify sockets)
- Health checking (socket-based)
- Graph execution (TOML deployment graphs)

**What We're Missing** ⚠️:
- **Primal-to-primal routing** (interaction layer!)
- **Request proxying** (through Tower Atomic)
- **Capability-based routing** (not just discovery)
- **API gateway pattern** (service mesh)

---

## 🏗️ **THE ARCHITECTURE**

### **Three-Layer Model** (from whitePaper/neuralAPI/)

```
┌────────────────────────────────────────────────┐
│   Layer 3: Niche APIs (RootPulse, etc)        │
│   • High-level abstractions                    │
│   • Multi-primal workflows                     │
│   • User-facing APIs                           │
└────────────────────────────────────────────────┘
                    ↓ ↑
┌────────────────────────────────────────────────┐
│   Layer 2: biomeOS / Neural API ← WE ARE HERE  │
│   • Deployment (✅ DONE)                       │
│   • Routing (⚠️  NOT DONE!)                    │
│   • Learning (⏳ Future)                       │
│   • Optimization (⏳ Future)                   │
└────────────────────────────────────────────────┘
                    ↓ ↑
┌────────────────────────────────────────────────┐
│   Layer 1: Primals (BearDog, Songbird, etc)   │
│   • Individual capabilities                    │
│   • Self-contained services                    │
│   • Report metrics                             │
└────────────────────────────────────────────────┘
```

**Key Realization**:
- **Primals don't talk to each other directly** (in production)
- **Neural API is the router/coordinator** (Layer 2)
- **Niche APIs abstract complexity** (Layer 3)

---

## 🔄 **THE ROUTING PATTERN**

### **Current Wrong Approach**:
```
Squirrel → (direct) → Tower Atomic → Anthropic API
         ❌ Breaks TRUE PRIMAL pattern
         ❌ Squirrel knows about Tower
         ❌ Tight coupling
```

### **Correct Approach**:
```
Squirrel → Neural API → Tower Atomic → Anthropic API
           ✅ Capability-based routing
           ✅ Squirrel is ignorant (TRUE PRIMAL)
           ✅ Loose coupling via API
```

---

## 📊 **EXAMPLE: Squirrel Makes AI Request**

### **What Should Happen**:

1. **Squirrel** wants to call Anthropic API
2. **Squirrel** sends request TO Neural API:
   ```json
   {
     "jsonrpc": "2.0",
     "method": "neural_api.proxy_http",
     "params": {
       "capability": "secure_http",
       "url": "https://api.anthropic.com/v1/messages",
       "method": "POST",
       "body": {...},
       "headers": {...}
     },
     "id": 1
   }
   ```

3. **Neural API** receives request:
   - Looks up capability: `secure_http` → Tower Atomic
   - Discovers Tower Atomic is BearDog + Songbird
   - Routes request to Songbird (HTTP handler)

4. **Songbird** (via Tower Atomic):
   - Gets request from Neural API
   - Uses BearDog for TLS/crypto
   - Makes actual HTTPS call to Anthropic
   - Returns response to Neural API

5. **Neural API**:
   - Receives response from Songbird
   - Returns to Squirrel
   - **Logs metrics** (for learning!)

6. **Squirrel**:
   - Receives response
   - **Doesn't know** Tower Atomic was involved
   - **TRUE PRIMAL** ✅

---

## 🎯 **NEURAL API ROUTING METHODS** (Need to Implement)

### **Core Routing**:

```rust
// 1. Proxy HTTP through Tower Atomic
neural_api.proxy_http(capability, url, method, body, headers)
→ Routes to Tower Atomic (BearDog + Songbird)

// 2. Store data through Nest Atomic
neural_api.store_data(capability, key, value)
→ Routes to Nest Atomic (BearDog + NestGate)

// 3. Execute compute through Node Atomic
neural_api.execute_compute(capability, function, args)
→ Routes to Node Atomic (BearDog + ToadStool)

// 4. Generic RPC routing
neural_api.call_capability(capability, method, params)
→ Routes to primal(s) providing capability
```

### **Current Methods** (Deployment Only):

```rust
// ✅ Deployment methods (WORKING):
neural_api.execute_graph(graph_id, family_id)
neural_api.get_execution_status(execution_id)
neural_api.list_graphs()
neural_api.get_topology()
neural_api.get_primals()

// ⚠️  Routing methods (MISSING):
neural_api.proxy_http(...)        // NOT IMPLEMENTED
neural_api.store_data(...)         // NOT IMPLEMENTED
neural_api.execute_compute(...)    // NOT IMPLEMENTED
neural_api.call_capability(...)    // NOT IMPLEMENTED
```

---

## 🏗️ **ARCHITECTURE COMPARISON**

### **Without Neural API Routing** (Current):
```
┌─────────┐      ┌──────────┐     ┌────────────┐
│ Squirrel│─────→│ Songbird │────→│ Anthropic  │
└─────────┘      │(directly)│     │    API     │
   ❌ Knows      └──────────┘     └────────────┘
   about Tower   └─BearDog         
                    (crypto)
```

**Problems**:
- ❌ Squirrel knows about Songbird/BearDog
- ❌ Tight coupling
- ❌ No metrics/learning
- ❌ Not TRUE PRIMAL

### **With Neural API Routing** (Correct):
```
┌─────────┐     ┌────────────┐     ┌──────────┐     ┌────────────┐
│ Squirrel│────→│ Neural API │────→│ Songbird │────→│ Anthropic  │
└─────────┘     │ (Layer 2)  │     │(Tower    │     │    API     │
   ✅ Ignorant  │ • Routes   │     │ Atomic)  │     └────────────┘
   (TRUE        │ • Logs     │     └──────────┘
   PRIMAL)      │ • Learns   │     └─BearDog
                └────────────┘        (crypto)
```

**Benefits**:
- ✅ Squirrel is ignorant (capability-based)
- ✅ Loose coupling (via API)
- ✅ Metrics collected (for learning)
- ✅ TRUE PRIMAL pattern
- ✅ Service mesh architecture

---

## 📝 **WHAT NEEDS TO BE BUILT**

### **Phase 1: Core Routing Infrastructure** (2-3 Days)

**File**: `crates/biomeos-atomic-deploy/src/neural_api_server.rs`

**Add Methods**:

```rust
/// Proxy HTTP request through Tower Atomic
async fn proxy_http(&self, params: &Value) -> Result<Value> {
    // 1. Extract capability ("secure_http")
    let capability = params["capability"].as_str()?;
    
    // 2. Discover Tower Atomic (BearDog + Songbird)
    let tower = self.discover_capability(capability).await?;
    
    // 3. Forward request to Songbird
    let response = self.forward_to_primal(
        &tower.songbird_socket,
        "http.proxy",
        params
    ).await?;
    
    // 4. Log metrics
    self.log_routing_metric(capability, &response).await?;
    
    Ok(response)
}

/// Generic capability-based routing
async fn call_capability(&self, params: &Value) -> Result<Value> {
    let capability = params["capability"].as_str()?;
    let method = params["method"].as_str()?;
    let call_params = &params["params"];
    
    // Discover primal(s) providing capability
    let primals = self.discover_capability(capability).await?;
    
    // Route to appropriate primal
    let response = self.route_to_primal(&primals, method, call_params).await?;
    
    // Log and return
    self.log_routing_metric(capability, &response).await?;
    Ok(response)
}

/// Discover primal(s) by capability
async fn discover_capability(&self, capability: &str) -> Result<DiscoveredPrimals> {
    // Check deployed atomics
    let topology = self.get_topology().await?;
    
    // Map capability to atomic/primal
    match capability {
        "secure_http" => {
            // Tower Atomic = BearDog + Songbird
            self.find_tower_atomic(&topology).await
        }
        "secure_storage" => {
            // Nest Atomic = Tower + NestGate
            self.find_nest_atomic(&topology).await
        }
        "secure_compute" => {
            // Node Atomic = Tower + ToadStool
            self.find_node_atomic(&topology).await
        }
        _ => Err(anyhow!("Unknown capability: {}", capability))
    }
}

/// Forward request to primal via Unix socket
async fn forward_to_primal(
    &self,
    socket_path: &str,
    method: &str,
    params: &Value
) -> Result<Value> {
    // Connect to primal's Unix socket
    let mut stream = UnixStream::connect(socket_path).await?;
    
    // Send JSON-RPC request
    let request = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    
    stream.write_all(serde_json::to_string(&request)?.as_bytes()).await?;
    
    // Read response
    let mut response = String::new();
    stream.read_to_string(&mut response).await?;
    
    Ok(serde_json::from_str(&response)?)
}
```

**Add to Method Router**:
```rust
match request.method.as_str() {
    // ... existing methods
    
    // NEW: Routing methods
    "neural_api.proxy_http" => self.proxy_http(&request.params).await?,
    "neural_api.store_data" => self.store_data(&request.params).await?,
    "neural_api.execute_compute" => self.execute_compute(&request.params).await?,
    "neural_api.call_capability" => self.call_capability(&request.params).await?,
    
    _ => // error
}
```

---

### **Phase 2: Atomic Discovery** (1-2 Days)

**Implement**:
- `find_tower_atomic()` - Discover BearDog + Songbird
- `find_nest_atomic()` - Discover Tower + NestGate
- `find_node_atomic()` - Discover Tower + ToadStool

**Use Cases**:
- Tower Atomic → Secure HTTP (Squirrel → Anthropic)
- Nest Atomic → Secure Storage (any primal → data)
- Node Atomic → Secure Compute (any primal → computation)

---

### **Phase 3: Metrics & Learning** (Future)

**Implement**:
- Request/response logging
- Latency tracking
- Pattern discovery
- Automatic optimization

---

## 🎯 **USE CASE: Squirrel AI Request**

### **Full Flow** (With Routing):

```bash
# 1. Deploy Tower Atomic + Squirrel via Neural API
./biomeos neural-api execute tower_squirrel nat0

# Result: 3 primals running
# - BearDog (security) @ /tmp/beardog-nat0.sock
# - Songbird (discovery/HTTP) @ /tmp/songbird-nat0.sock
# - Squirrel (AI) @ /tmp/squirrel-nat0.sock

# 2. Squirrel makes AI request VIA Neural API
echo '{
  "jsonrpc": "2.0",
  "method": "neural_api.proxy_http",
  "params": {
    "capability": "secure_http",
    "url": "https://api.anthropic.com/v1/messages",
    "method": "POST",
    "headers": {"x-api-key": "..."},
    "body": {
      "model": "claude-3-opus-20240229",
      "messages": [{"role": "user", "content": "Hello!"}]
    }
  },
  "id": 1
}' | nc -U /tmp/neural-api-nat0.sock

# 3. Neural API routes through Tower Atomic:
#    Neural API → Songbird → BearDog (crypto) → Anthropic
#                   ↓
#            Returns response to Squirrel

# 4. Squirrel receives response
# ✅ Squirrel never knew about Tower Atomic!
# ✅ TRUE PRIMAL pattern maintained!
# ✅ Metrics logged for learning!
```

---

## 📊 **COMPARISON WITH ROOTPULSE**

### **RootPulse** (Niche API - Layer 3):
```rust
// User calls high-level API
rootpulse.commit("Feature complete").await?;

// Behind the scenes (via Neural API):
// 1. Neural API routes to rhizoCrypt (DAG)
// 2. Neural API routes to LoamSpine (linear)
// 3. Neural API routes to NestGate (storage)
// 4. Neural API routes to BearDog (crypto)
// 5. Neural API routes to SweetGrass (attribution)
// 6. Neural API routes to Songbird (federation)

// User never sees individual primals!
```

**RootPulse** is a Niche API that coordinates multiple primals through Neural API routing!

---

## ✅ **WHAT WE'VE BUILT (So Far)**

### **Deployment** ✅ (90% Complete):
- ✅ Capability-based discovery (security → beardog)
- ✅ Process spawning (tokio::process::Command)
- ✅ Socket verification (3s timeout)
- ✅ Health checking (socket-based)
- ✅ PID tracking
- ✅ Graph execution (TOML)

**Result**: Neural API can deploy primals! 🎉

---

## ⚠️  **WHAT WE NEED TO BUILD**

### **Routing** ⚠️ (0% Complete):
- ⚠️  HTTP proxying (through Tower Atomic)
- ⚠️  Capability-based routing (discovery → forward)
- ⚠️  Request/response forwarding (Unix sockets)
- ⚠️  Metrics logging (for learning)
- ⚠️  Service mesh pattern (API gateway)

**Result**: Neural API will route primal interactions! 🎯

---

## 🎯 **NEXT STEPS**

### **Immediate** (This Week):

1. **Implement `proxy_http` method** (2-3 hours)
   - Extract capability from params
   - Discover Tower Atomic
   - Forward request to Songbird
   - Return response

2. **Test Squirrel → Neural API → Tower → Anthropic** (1 hour)
   - Deploy Tower + Squirrel
   - Squirrel calls `neural_api.proxy_http`
   - Validate end-to-end flow

3. **Document routing architecture** (1 hour)
   - Update Neural API spec
   - Add examples
   - Update whitepaper

### **Short Term** (Next Week):

4. **Implement generic routing** (`call_capability`)
5. **Add metrics logging**
6. **Test with RootPulse** (if ready)

### **Long Term** (Future):

7. **Learning mechanisms**
8. **Automatic optimization**
9. **Pattern discovery**

---

## 💡 **KEY INSIGHTS**

### **1. Neural API is TWO Things**:
- ✅ **Deployment Layer** (what we built)
- ⚠️  **Routing/Interaction Layer** (what we need)

### **2. TRUE PRIMAL Pattern Requires Routing**:
- Primals DON'T talk directly (except discovery)
- Neural API routes ALL interactions
- Maintains ignorance (TRUE PRIMAL)

### **3. Service Mesh Architecture**:
- Neural API = API Gateway + Service Mesh
- Capability-based routing
- Metrics for learning
- Automatic optimization (future)

### **4. We've Only Begun**:
- Deployment: 90% complete ✅
- Routing: 0% complete ⚠️
- Learning: 0% complete ⏳
- Optimization: 0% complete ⏳

**Overall Progress**: **~25% of Neural API vision** 📊

---

## 🎊 **CONCLUSION**

### **Status**: ⚠️  **CRITICAL REALIZATION**

**What We Thought**:
- Neural API just deploys primals
- Primals interact directly
- We're almost done

**What's Actually True**:
- Neural API is BOTH deployment AND routing
- Primals interact VIA Neural API
- We've built foundation, but core routing missing

**Impact**:
- ✅ Excellent foundation (deployment works!)
- ⚠️  Missing critical feature (routing)
- 🎯 Clear path forward (implement routing)

**Estimate**:
- Routing implementation: 3-5 days
- Full feature parity: 2-3 weeks
- Learning/optimization: 4-8 weeks

---

🏰🧠⚛️✨ **Neural API: The Brain That Routes, Learns, and Evolves!** ✨⚛️🧠🏰

**Next: Implement primal-to-primal routing layer!** 🚀

---

**Architecture**: Clarified  
**Vision**: Expanded  
**Path**: Clear  
**Status**: 25% Complete, but foundation is solid! ✅

