# 🔍 Emergent Behavior & Control Gap Analysis

**Date**: January 9, 2026  
**Status**: 🚨 **CRITICAL ARCHITECTURAL INSIGHT**  
**Priority**: HIGH - Security & Control Implications

---

## 🎯 The Core Question

> "Is this emergent behavior from the Neural API? Can we control which BearDog instance to use? Can BearDog be hijacked?"

**Findings**:
1. ❌ **NOT Neural API** - This is the `biomeos-federation` discovery system
2. ⚠️ **Limited control** - Discovery is semi-random (first-found or last-overwrites)
3. 🚨 **Security gap** - Yes, BearDog could theoretically be hijacked
4. ✅ **Good news** - We can fix this with architectural evolution

---

## 🔬 What Actually Happened: Technical Trace

### **Call Stack**

```
verify-lineage tool
    ↓
BearDogClient::from_discovery()      [biomeos-federation/src/beardog_client.rs]
    ↓
PrimalDiscovery::discover()          [biomeos-federation/src/discovery.rs]
    ↓
discover_unix_sockets()              [Line 81-107]
    ↓
register_unix_socket_primal()        [Line 110-166]
    ↓
HashMap.insert(primal_name, discovered)  [Line 165]
```

**NOT the Neural API graph system!**

---

### **Discovery Implementation** (biomeos-federation/src/discovery.rs)

```rust
async fn discover_unix_sockets(&mut self) -> FederationResult<()> {
    let socket_dir = PathBuf::from("/tmp");
    
    // Reads ALL .sock files in /tmp
    let mut entries = tokio::fs::read_dir(&socket_dir).await?;
    
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        
        if filename.ends_with(".sock") {
            self.register_unix_socket_primal(&path).await;  // ← Registers each
        }
    }
    
    Ok(())
}

async fn register_unix_socket_primal(&mut self, socket_path: &PathBuf) {
    // Extract primal name from filename
    // "beardog-nat0-test-federation.sock" → "beardog"
    let primal_name = filename
        .split('-')
        .next()  // ← Takes FIRST part only!
        .unwrap_or(filename)
        .trim_end_matches(".sock")
        .to_string();
    
    // Insert into HashMap
    self.discovered_primals.insert(primal_name, discovered);  // ← LAST ONE WINS!
}
```

**Critical Insight**: If multiple `beardog-*.sock` files exist, **only the last one discovered is kept!**

---

### **Client Selection** (biomeos-federation/src/beardog_client.rs)

```rust
impl BearDogClient {
    pub async fn from_discovery() -> Result<Self> {
        let mut discovery = PrimalDiscovery::new();
        discovery.discover().await?;
        
        // Gets THE SINGLE BearDog entry (whichever was registered last)
        let beardog = discovery.get("beardog")
            .ok_or_else(|| anyhow!("BearDog not found"))?;
        
        // Uses the first endpoint (only one anyway)
        let endpoint = match &beardog.endpoints[0] {
            PrimalEndpoint::UnixSocket { path } => {
                BearDogEndpoint::UnixSocket(path.clone())
            }
            // ...
        };
        
        Ok(Self { endpoint })
    }
}
```

**No control over which instance!**

---

## 🎊 Why It Worked This Time

### **Filesystem Read Order**

```bash
$ ls -t /tmp/beardog*.sock  # -t = sort by modification time
/tmp/beardog-test-lineage-check.sock      # Created 20:41 (newest)
/tmp/beardog-nat0-test-federation.sock    # Created 14:43 (older)
/tmp/beardog-default-test-node.sock       # Created 14:25 (older)
/tmp/beardog-default-test-federation.sock # Created 14:14 (oldest)
```

**But tokio::fs::read_dir() doesn't guarantee order!**

From Tokio docs:
> "The order in which this iterator returns entries is undefined."

**We got lucky!** The filesystem happened to return `beardog-nat0-test-federation.sock` last, so it was the one registered.

---

## 🚨 The Security Gap: Can BearDog Be Hijacked?

### **Scenario: Malicious Process**

```rust
// Evil program creates a fake BearDog socket
use tokio::net::UnixListener;

#[tokio::main]
async fn main() {
    // Create a socket that looks like BearDog
    let listener = UnixListener::bind("/tmp/beardog-malicious.sock").unwrap();
    
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        
        // Intercept requests
        // - Log all API calls (including seed verification!)
        // - Return fake results
        // - Steal cryptographic secrets
        // - Impersonate legitimate BearDog
        
        handle_evil_request(socket).await;
    }
}
```

**Result**: ✅ **Yes, BearDog can be hijacked!**

---

### **Attack Vectors**

1. **Socket Name Squatting**:
   - Attacker creates `/tmp/beardog-evil.sock` before legitimate BearDog starts
   - Discovery finds it and registers it
   - All tools use the malicious instance

2. **Socket Replacement**:
   - Attacker kills legitimate BearDog process
   - Immediately creates `/tmp/beardog-nat0-test-federation.sock`
   - Tools re-discover and connect to malicious instance

3. **Filesystem Race**:
   - Both legitimate and malicious sockets exist
   - Discovery order is undefined
   - Malicious socket might be selected

4. **Namespace Pollution**:
   - Attacker creates dozens of `/tmp/beardog-*.sock` files
   - Last one discovered wins (or first, undefined order!)
   - Legitimate BearDog might not even be found

---

### **What's Missing: No Authentication**

Current discovery:
```rust
// Just checks if socket exists and can be opened
if filename.ends_with(".sock") {
    self.register_unix_socket_primal(&path).await;
}
```

**No verification of**:
- ✗ Socket owner (UID/GID)
- ✗ Process identity (PID)
- ✗ Cryptographic proof
- ✗ Capabilities (doesn't actually query the socket!)
- ✗ Family context
- ✗ Version compatibility

---

## 📊 Comparison: Neural API vs Federation Discovery

| Aspect | Neural API (Graph) | Federation Discovery | This Case |
|--------|-------------------|---------------------|-----------|
| **Used?** | ❌ No | ✅ Yes | Federation |
| **Control** | ✅ Explicit via graph | ⚠️ First-found | Semi-random |
| **Selection** | ✅ Capability-based | ⚠️ Name-based | Name-based |
| **Multiple Instances** | ✅ Can select specific | ❌ Last-one-wins | Last-one-wins |
| **Authentication** | ⚠️ TODO | ❌ None | None |
| **Isolation** | ✅ Per-graph context | ❌ Global discovery | Global |

**Key Insight**: The Neural API graph system has better control primitives, but wasn't used here!

---

## 🎯 Neural API Would Enable Better Control

### **How Neural API Could Handle This**

**Graph Definition** (graphs/verify_lineage.toml):
```toml
[graph]
name = "verify_lineage"
coordination = "sequential"

[[nodes]]
id = "select_beardog"
primal = { by_capability = "encryption" }  # OR: by_id = "beardog-nat0"
operation = { name = "get_capabilities" }

  [nodes.constraints]
  # Could add: require_family = "nat0"
  # Could add: require_owner_uid = 1000
  # Could add: require_version = ">=0.15.0"

[[nodes]]
id = "verify_lineage"
primal = { by_id = "$select_beardog.primal_id" }  # Reuse the same instance!
operation = { name = "federation.verify_family_member" }
depends_on = ["select_beardog"]
```

**This would give us**:
- ✅ Explicit instance selection
- ✅ Reuse same instance for all calls
- ✅ Validation before use
- ✅ Clear dependency tracking
- ✅ Auditable execution path

---

## 🔮 What We Need: Control & Security Evolution

### **Phase 1: Authentication & Verification**

**Add to PrimalDiscovery**:
```rust
async fn register_unix_socket_primal(&mut self, socket_path: &PathBuf) {
    // 1. Check socket owner
    let metadata = tokio::fs::metadata(socket_path).await?;
    let owner_uid = metadata.uid();
    
    if owner_uid != expected_uid {
        warn!("Socket {} has unexpected owner {}", socket_path, owner_uid);
        return; // Skip untrusted sockets
    }
    
    // 2. Query capabilities (actually call the socket!)
    let client = UnixSocketClient::new(socket_path);
    let caps = match client.call_method("get_capabilities", json!({})).await {
        Ok(c) => c,
        Err(_) => {
            warn!("Socket {} doesn't respond to capabilities", socket_path);
            return; // Not a valid primal
        }
    };
    
    // 3. Verify cryptographic identity
    let identity = client.call_method("get_identity", json!({})).await?;
    if !verify_primal_signature(&identity) {
        warn!("Socket {} failed signature verification", socket_path);
        return;
    }
    
    // 4. Check family context (if applicable)
    if let Ok(family) = client.call_method("get_family_id", json!({})).await {
        metadata.insert("family_id", family);
    }
    
    // NOW register it
    self.discovered_primals.insert(primal_name, discovered);
}
```

---

### **Phase 2: Multiple Instance Support**

**Change from HashMap<String, Primal> to HashMap<String, Vec<Primal>>**:

```rust
pub struct PrimalDiscovery {
    // OLD: discovered_primals: HashMap<String, DiscoveredPrimal>,
    discovered_primals: HashMap<String, Vec<DiscoveredPrimal>>,  // ← Multiple instances!
}

impl PrimalDiscovery {
    /// Get ALL instances of a primal
    pub fn get_all(&self, name: &str) -> Vec<&DiscoveredPrimal> {
        self.discovered_primals
            .get(name)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }
    
    /// Get primal by specific criteria
    pub fn get_by_family(&self, name: &str, family_id: &str) -> Option<&DiscoveredPrimal> {
        self.get_all(name)
            .into_iter()
            .find(|p| p.metadata.get("family_id") == Some(&family_id.to_string()))
    }
    
    /// Get primal by socket path
    pub fn get_by_socket(&self, name: &str, socket_path: &Path) -> Option<&DiscoveredPrimal> {
        self.get_all(name)
            .into_iter()
            .find(|p| match &p.endpoints[0] {
                PrimalEndpoint::UnixSocket { path } => path == socket_path,
                _ => false,
            })
    }
}
```

---

### **Phase 3: Explicit Instance Selection**

**Add to BearDogClient**:
```rust
impl BearDogClient {
    /// Create with explicit selection criteria
    pub async fn from_discovery_with_criteria(criteria: SelectionCriteria) -> Result<Self> {
        let mut discovery = PrimalDiscovery::new();
        discovery.discover().await?;
        
        let beardog = match criteria {
            SelectionCriteria::Any => {
                // Current behavior: first available
                discovery.get("beardog")
            }
            SelectionCriteria::ByFamily(family_id) => {
                // Select by family context
                discovery.get_by_family("beardog", &family_id)
            }
            SelectionCriteria::BySocket(path) => {
                // Select specific socket
                discovery.get_by_socket("beardog", &path)
            }
            SelectionCriteria::ByOwner(uid) => {
                // Select by process owner
                discovery.get_all("beardog")
                    .into_iter()
                    .find(|p| p.metadata.get("owner_uid") == Some(&uid.to_string()))
            }
        }.ok_or_else(|| anyhow!("No BearDog matching criteria"))?;
        
        Ok(Self { endpoint: /* ... */ })
    }
}

pub enum SelectionCriteria {
    Any,
    ByFamily(String),
    BySocket(PathBuf),
    ByOwner(u32),
}
```

---

### **Phase 4: Isolation & Namespaces**

**Socket namespacing**:
```rust
// Instead of /tmp/beardog-*.sock (global namespace)
// Use per-user or per-deployment namespaces:

// Per-user:
// /run/user/{uid}/biomeos/beardog-{family}.sock

// Per-deployment:
// /var/run/biomeos/{deployment_id}/beardog-{family}.sock

// This prevents cross-user hijacking
```

**Process isolation**:
```rust
// Check socket owner matches current user
if socket_metadata.uid() != current_uid() {
    return Err("Cannot use socket from different user");
}
```

---

### **Phase 5: Capability Verification**

**Actually query the socket before trusting it**:
```rust
async fn verify_primal_capabilities(&self, socket_path: &Path) -> Result<CapabilitySet> {
    let client = UnixSocketClient::new(socket_path);
    
    // Call the actual API
    let response = client.call_method("get_capabilities", json!({}))
        .await
        .context("Failed to query capabilities")?;
    
    // Parse and validate
    let caps: Vec<String> = serde_json::from_value(response["capabilities"].clone())?;
    
    Ok(CapabilitySet::from_vec(
        caps.into_iter().map(Capability::Custom).collect()
    ))
}
```

---

## 🎊 Answers to Your Questions

### **1. Is this emergent behavior from the Neural API?**

❌ **No** - This is the `biomeos-federation` discovery system, NOT the Neural API.

The Neural API graph system (`biomeos-graph`) has:
- ✅ Explicit primal selection via graph definitions
- ✅ Capability-based discovery with validation
- ✅ Per-graph execution contexts
- ✅ Auditable execution paths

But `verify-lineage` didn't use it! It used the lower-level `BearDogClient::from_discovery()`.

---

### **2. Do we have enough control to choose a specific instance?**

⚠️ **Currently: No**

Current limitations:
- ❌ Can't select specific BearDog instance
- ❌ Can't specify family context
- ❌ Can't filter by owner
- ❌ Discovery order is undefined
- ❌ Last-one-wins (or first-one, undefined!)

**But we CAN add it!** See Phase 2 & 3 above.

---

### **3. Can BearDog be hijacked?**

🚨 **Yes, currently vulnerable**

Attack vectors:
- ✅ Socket name squatting
- ✅ Socket replacement after kill
- ✅ Filesystem race conditions
- ✅ Namespace pollution

**Mitigations needed**:
- Add socket owner verification (Phase 1)
- Add capability verification (Phase 5)
- Add cryptographic identity (Phase 1)
- Add namespace isolation (Phase 4)

---

### **4. Is this good or bad emergent behavior?**

**Mixed**:

✅ **Good**: Automatic discovery and reuse of infrastructure  
✅ **Good**: Zero configuration needed  
✅ **Good**: Works across different deployments  

❌ **Bad**: No authentication or verification  
❌ **Bad**: No control over which instance  
❌ **Bad**: Security vulnerability  
❌ **Bad**: Non-deterministic selection  

---

## 🚀 Recommended Evolution Path

### **Immediate (This Week)**

1. **Add Authentication** (Phase 1)
   - Verify socket owner (UID check)
   - Query capabilities before trusting
   - Add warning logs for untrusted sockets

2. **Document Limitations**
   - Clear docs on discovery behavior
   - Security considerations
   - Best practices for deployment

### **Short-Term (Next Sprint)**

3. **Multiple Instance Support** (Phase 2)
   - Track all discovered instances
   - Provide selection methods
   - Default to "safest" choice

4. **Explicit Selection** (Phase 3)
   - Add `SelectionCriteria` API
   - Let tools choose specific instances
   - Update `verify-lineage` to use it

### **Medium-Term (Next Month)**

5. **Isolation** (Phase 4)
   - Per-user socket namespaces
   - Process owner checks
   - Deployment-scoped discovery

6. **Capability Verification** (Phase 5)
   - Actually query socket APIs
   - Verify primal identity
   - Check version compatibility

### **Long-Term (Neural API Integration)**

7. **Graph-Based Discovery**
   - Migrate tools to use Neural API graphs
   - Explicit instance selection in graphs
   - Auditable primal selection
   - Validation before execution

---

## 🎯 Bottom Line

**Your insight is 100% correct!**

This IS emergent behavior from composable systems:
- ✅ Discovery system worked
- ✅ Reused infrastructure
- ✅ Got correct result

But we lack control mechanisms:
- ❌ No authentication
- ❌ No explicit selection
- ❌ Security vulnerability exists
- ❌ Non-deterministic behavior

**The good news**: We identified this BEFORE it became a production issue!

**Next steps**:
1. Add authentication (Phase 1) - **HIGH PRIORITY**
2. Add explicit selection (Phase 2-3) - **HIGH PRIORITY**
3. Evolve toward Neural API integration - **STRATEGIC**

---

**This is exactly the kind of architectural investigation that prevents security issues!** 🎉

🔍 **Emergent Behavior Identified → Control Mechanisms Designed → System Evolution Planned** 🚀

