# Hardcoding Analysis - January 12, 2026

**Status**: Analysis Complete  
**Priority**: HIGH (Violates TRUE PRIMAL principle)  
**Principle Violated**: "Primal code only has self knowledge and discovers other primals in runtime"

---

## 📊 Analysis Summary

**Total Instances**: 1,263 matches across 135 files  
**Critical Violations**: 15 instances  
**Medium Priority**: ~30 instances (mock/demo data)  
**Low Priority**: ~1,200 instances (tests, docs, type definitions)

---

## 🚨 CRITICAL VIOLATIONS (HIGH PRIORITY)

### 1. Name Inference from Socket Filenames ❌

**Location**: `crates/biomeos-ui/src/petaltongue_bridge.rs:496-510`

**Violation**:
```rust
fn extract_primal_name(&self, socket_name: &str) -> String {
    if socket_name.contains("songbird") {
        "Songbird".to_string()
    } else if socket_name.contains("beardog") {
        "BearDog".to_string()
    } else if socket_name.contains("toadstool") {
        "ToadStool".to_string()
    } else if socket_name.contains("nestgate") {
        "NestGate".to_string()
    } else if socket_name.contains("squirrel") {
        "Squirrel".to_string()
    } else {
        "Unknown".to_string()
    }
}
```

**Problem**: Assumes knowledge of all primal names  
**Impact**: New primals won't be discovered  
**Severity**: HIGH

**Evolution**:
```rust
async fn query_primal_identity(&self, socket_path: &str) -> Result<String> {
    // Connect and query primal for its identity
    let stream = UnixStream::connect(socket_path).await?;
    let response = json_rpc_call(&stream, "get_identity", json!({})).await?;
    Ok(response["name"].as_str().unwrap_or("Unknown").to_string())
}
```

---

### 2. Capability Inference from Names ❌

**Location**: `crates/biomeos-federation/src/discovery.rs:125-146`

**Violation**:
```rust
// Infer primal type from name
let primal_type = match primal_name.as_str() {
    "songbird" => "federation",
    "beardog" => "security",
    "loamspine" => "storage",
    "toadstool" => "orchestration",
    _ => "unknown",
};

// Infer capabilities from primal type
let capabilities = match primal_type {
    "federation" => CapabilitySet::from_vec(vec![
        Capability::Discovery,
        Capability::Voice,
        Capability::Video,
    ]),
    "security" => CapabilitySet::from_vec(vec![
        Capability::Custom("encryption".to_string()),
        Capability::Custom("authentication".to_string()),
    ]),
    "storage" => CapabilitySet::from_vec(vec![
        Capability::Storage,
        Capability::Sync
    ]),
    _ => CapabilitySet::new(),
};
```

**Problem**: Hardcoded name → capability mapping  
**Impact**: Primals can't advertise their own capabilities  
**Severity**: CRITICAL (TRUE PRIMAL violation)

**Evolution**:
```rust
async fn query_primal_capabilities(&self, socket_path: &PathBuf) -> Result<CapabilitySet> {
    // Connect to primal
    let stream = UnixStream::connect(socket_path).await?;
    
    // Query for capabilities via JSON-RPC
    let response = json_rpc_call(&stream, "list_capabilities", json!({})).await?;
    
    // Parse capabilities from response
    let caps: Vec<String> = serde_json::from_value(response["capabilities"].clone())?;
    Ok(CapabilitySet::from_strings(caps))
}
```

---

### 3. Hardcoded Localhost Fallbacks ❌

**Location**: `crates/biomeos-core/src/discovery_http.rs:324-367`

**Violation**:
```rust
// BearDog (Security)
let beardog_url = std::env::var("BEARDOG_ENDPOINT").unwrap_or_else(|_| {
    #[cfg(debug_assertions)]
    {
        "http://localhost:9000".to_string()  // ❌ Hardcoded port
    }
    #[cfg(not(debug_assertions))]
    {
        String::new()
    }
});

if !beardog_url.is_empty() {
    builder = builder.add_primal(
        endpoint,
        PrimalId::new_unchecked("beardog-local"),  // ❌ Hardcoded ID
        "BearDog".to_string(),  // ❌ Hardcoded name
        PrimalType::Security,  // ❌ Hardcoded type
    );
}
```

**Problem**: Hardcoded URLs, IDs, names, and types  
**Impact**: Can't adapt to different deployments  
**Severity**: HIGH

**Evolution**:
```rust
pub async fn discover_via_environment() -> DiscoveryResult<Vec<DiscoveredPrimal>> {
    let mut primals = Vec::new();
    
    // Scan for *_ENDPOINT environment variables
    for (key, value) in std::env::vars() {
        if key.ends_with("_ENDPOINT") && !value.is_empty() {
            // Query the primal for its identity and capabilities
            if let Ok(primal_info) = query_primal_info(&value).await {
                primals.push(primal_info);
            }
        }
    }
    
    Ok(primals)
}

async fn query_primal_info(endpoint: &str) -> Result<DiscoveredPrimal> {
    let client = reqwest::Client::new();
    let response = client.post(endpoint)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "get_primal_info",
            "params": {},
            "id": 1
        }))
        .send()
        .await?;
    
    let info: PrimalInfo = response.json().await?;
    Ok(DiscoveredPrimal {
        name: info.name,
        id: PrimalId::new(&info.id),
        primal_type: info.primal_type,
        capabilities: info.capabilities,
        endpoint: Endpoint::new(endpoint)?,
    })
}
```

---

## ⚠️ MEDIUM PRIORITY

### 4. Mock/Demo Data in API Handlers

**Locations**:
- `crates/biomeos-api/src/handlers/live_discovery.rs:23-31`
- `crates/biomeos-api/src/handlers/topology.rs:102-120`
- `crates/biomeos-api/src/handlers/discovery.rs:154-172`

**Status**: Demo/placeholder data  
**Severity**: MEDIUM (should use real discovery)

**Evolution**: Replace with calls to NUCLEUS client

---

### 5. Socket Environment Variable Mapping

**Location**: `crates/biomeos-atomic-deploy/src/primal_launcher.rs:156-163`

**Current**:
```rust
"beardog-server" => "BEARDOG_SOCKET",
"songbird-orchestrator" => "SONGBIRD_SOCKET",
"toadstool" => "TOADSTOOL_SOCKET",
"nestgate" => "NESTGATE_SOCKET",
```

**Status**: Configuration mapping (acceptable pattern)  
**Severity**: LOW (this is OK for launcher configuration)

**Note**: This is acceptable as it's configuration for the launcher itself, not capability discovery

---

## ✅ ACCEPTABLE (LOW PRIORITY)

### 6. Type Enum Matching

**Example**: `crates/biomeos-api/src/handlers/topology.rs:204-212`

```rust
biomeos_core::PrimalType::Security => "beardog",
biomeos_core::PrimalType::Orchestration => "songbird",
biomeos_core::PrimalType::Storage => "nestgate",
biomeos_core::PrimalType::Compute => "toadstool",
```

**Status**: Type → display name mapping  
**Severity**: LOW (acceptable for UI display)

---

## 🎯 Evolution Plan

### Phase 1: Critical Path Evolution (12-16 hours)

1. **Add JSON-RPC capability queries** (4-5h)
   - Implement `get_primal_info` method
   - Implement `list_capabilities` method
   - Add to all primal interfaces

2. **Evolve discovery.rs** (3-4h)
   - Remove name-based capability inference
   - Query primals for capabilities
   - Update registration flow

3. **Evolve petaltongue_bridge.rs** (2-3h)
   - Remove name extraction
   - Query primals for identity
   - Dynamic primal discovery

4. **Evolve discovery_http.rs** (3-4h)
   - Remove hardcoded endpoints
   - Dynamic environment scanning
   - Query-based primal info

### Phase 2: Medium Priority (6-8 hours)

5. **Replace API handler mocks** (3-4h)
   - Use NUCLEUS client
   - Real discovery calls
   - Remove placeholder data

6. **Add primal self-description** (3-4h)
   - Implement info endpoints in primals
   - Standardize capability advertisement
   - Add to primal SDK

### Phase 3: Documentation & Verification (2-3 hours)

7. **Document changes**
8. **Verify all tests pass**
9. **Measure impact**

**Total Estimate**: 20-27 hours

---

## 📋 Standard Primal JSON-RPC Methods

All primals should implement these for discovery:

```rust
// Get basic primal information
{
    "jsonrpc": "2.0",
    "method": "get_primal_info",
    "params": {},
    "id": 1
}

// Response:
{
    "jsonrpc": "2.0",
    "result": {
        "id": "beardog-nat0",
        "name": "BearDog",
        "version": "0.1.0",
        "primal_type": "Security",
        "family": "nat0"
    },
    "id": 1
}

// List capabilities
{
    "jsonrpc": "2.0",
    "method": "list_capabilities",
    "params": {},
    "id": 2
}

// Response:
{
    "jsonrpc": "2.0",
    "result": {
        "capabilities": [
            "encryption",
            "authentication",
            "key_management",
            "seed_derivation"
        ]
    },
    "id": 2
}

// Health check
{
    "jsonrpc": "2.0",
    "method": "health",
    "params": {},
    "id": 3
}

// Response:
{
    "jsonrpc": "2.0",
    "result": {
        "status": "healthy",
        "load": 0.3,
        "uptime_seconds": 12345
    },
    "id": 3
}
```

---

## 🎓 Deep Debt Principles Applied

### TRUE PRIMAL Principle ✅

**Before** (Violation):
- biomeOS knows all primal names
- biomeOS infers capabilities from names
- biomeOS hardcodes connection details

**After** (Compliant):
- Primals announce their own identity
- Primals advertise their own capabilities
- Discovery is query-based, not assumed

### Infant Discovery Pattern ✅

**Before** (Parent knows child):
```rust
if socket_name.contains("beardog") {
    capabilities = vec!["encryption", "auth"];  // ❌ Parent assumes
}
```

**After** (Child tells parent):
```rust
let info = query_primal_info(socket_path).await?;  // ✅ Child announces
capabilities = info.capabilities;
```

---

## 📊 Impact Assessment

### Before Evolution
- ❌ 15 critical hardcoding violations
- ❌ Can't discover new/unknown primals
- ❌ Capabilities inferred, not queried
- ❌ Hardcoded names and ports

### After Evolution (Estimated)
- ✅ Zero hardcoded primal assumptions
- ✅ Dynamic primal discovery
- ✅ Self-announced capabilities
- ✅ Environment-based configuration

### Metrics
- **Code Removed**: ~100 lines of hardcoded mappings
- **Code Added**: ~200 lines of query-based discovery
- **Net Change**: +100 lines (better abstraction)
- **Flexibility**: ∞ (unlimited primals)

---

## 🚀 Next Steps

1. **Implement standard JSON-RPC methods** in primal SDK
2. **Evolve discovery.rs** to query-based
3. **Evolve petaltongue_bridge.rs** to query-based
4. **Evolve discovery_http.rs** to dynamic
5. **Update API handlers** to use NUCLEUS
6. **Test with real primals**
7. **Document new discovery protocol**

---

**Analysis Complete**: January 12, 2026  
**Critical Violations**: 15  
**Evolution Estimate**: 20-27 hours  
**Priority**: HIGH (TRUE PRIMAL compliance)  

**"Different orders of the same architecture."** 🍄🐸

