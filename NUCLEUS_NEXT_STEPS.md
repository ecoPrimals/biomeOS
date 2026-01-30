# 🎯 NUCLEUS Integration - Next Steps

**Date:** January 30, 2026  
**Status:** Integration test complete, configuration refinement needed  
**Time to Full Integration:** ~6 hours

---

## 📋 **Immediate Actions**

### **1. Socket Path Standardization** (1-2 hours)

Fix socket path inconsistencies across all primals.

#### **For each primal (BearDog, Songbird, Toadstool, NestGate):**

```rust
// Add to main.rs or config.rs:
use std::env;

fn get_socket_path(primal_name: &str) -> PathBuf {
    let socket_dir = env::var("BIOMEOS_SOCKET_DIR")
        .unwrap_or_else(|_| {
            let uid = unsafe { libc::getuid() };
            format!("/run/user/{}/biomeos", uid)
        });
    
    // Create directory if it doesn't exist
    std::fs::create_dir_all(&socket_dir).ok();
    
    PathBuf::from(socket_dir).join(format!("{}.sock", primal_name))
}

// Use in socket creation:
let socket_path = get_socket_path("beardog"); // or "songbird", etc.
```

#### **Environment Variable Support:**

```bash
# Add to all primal startup scripts:
export BIOMEOS_SOCKET_DIR="/run/user/$(id -u)/biomeos"
```

#### **Test:**

```bash
# Restart all primals
pkill -f "beardog.*server|songbird.*server"

# Start with env var
BIOMEOS_SOCKET_DIR="/run/user/$(id -u)/biomeos" ./beardog server &
BIOMEOS_SOCKET_DIR="/run/user/$(id -u)/biomeos" ./songbird server &

# Verify sockets
ls -la /run/user/$(id -u)/biomeos/*.sock
# Should see: beardog.sock, songbird.sock, toadstool.sock, nestgate.sock
```

---

### **2. Fix Cross-Primal Socket Detection** (30 mins)

Update primals to look for dependencies at standardized paths.

#### **Songbird → BearDog Connection:**

```rust
// In songbird/src/beardog_client.rs or similar:
fn get_beardog_socket() -> PathBuf {
    env::var("BEARDOG_SOCKET")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let uid = unsafe { libc::getuid() };
            PathBuf::from(format!("/run/user/{}/biomeos/beardog.sock", uid))
        })
}
```

#### **Toadstool → Songbird Registration:**

```rust
// In toadstool/src/discovery.rs or similar:
fn get_songbird_socket() -> PathBuf {
    env::var("SONGBIRD_SOCKET")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let uid = unsafe { libc::getuid() };
            PathBuf::from(format!("/run/user/{}/biomeos/songbird.sock", uid))
        })
}
```

---

### **3. Implement Capability Registry** (2-3 hours)

Create a centralized capability → socket mapping file.

#### **Registry File:**

```bash
# Location: /run/user/$UID/biomeos/capabilities.json

{
  "version": "1.0.0",
  "updated_at": "2026-01-30T01:10:00Z",
  "capabilities": {
    "crypto.sign": {
      "socket": "/run/user/1000/biomeos/beardog.sock",
      "primal": "beardog",
      "version": "0.18.0",
      "methods": ["crypto.sign_ed25519", "crypto.verify_ed25519", ...]
    },
    "http.request": {
      "socket": "/run/user/1000/biomeos/songbird.sock",
      "primal": "songbird",
      "version": "1.1.0",
      "methods": ["http.get", "http.post", "http.request"]
    },
    "compute.gpu": {
      "socket": "/run/user/1000/biomeos/toadstool.sock",
      "primal": "toadstool",
      "version": "0.1.0",
      "methods": ["compute.schedule", "compute.estimate"]
    },
    "storage.persist": {
      "socket": "/run/user/1000/biomeos/nestgate.sock",
      "primal": "nestgate",
      "version": "0.1.0",
      "methods": ["storage.store", "storage.retrieve", "storage.list"]
    },
    "ai.coordinate": {
      "socket": "/run/user/1000/biomeos/squirrel.sock",
      "primal": "squirrel",
      "version": "0.1.0",
      "methods": ["ai.query", "ai.local", "ai.online"]
    }
  }
}
```

#### **Registry Library (Shared):**

```rust
// Create: crates/biomeos-registry/src/lib.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct CapabilityRegistry {
    pub version: String,
    pub updated_at: String,
    pub capabilities: HashMap<String, CapabilityInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CapabilityInfo {
    pub socket: PathBuf,
    pub primal: String,
    pub version: String,
    pub methods: Vec<String>,
}

impl CapabilityRegistry {
    pub fn load() -> Result<Self> {
        let registry_path = Self::get_registry_path();
        let contents = std::fs::read_to_string(&registry_path)?;
        Ok(serde_json::from_str(&contents)?)
    }
    
    pub fn register_capability(
        &mut self,
        capability: String,
        info: CapabilityInfo
    ) -> Result<()> {
        self.capabilities.insert(capability, info);
        self.updated_at = chrono::Utc::now().to_rfc3339();
        self.save()
    }
    
    pub fn get_socket(&self, capability: &str) -> Option<&PathBuf> {
        self.capabilities.get(capability).map(|info| &info.socket)
    }
    
    fn get_registry_path() -> PathBuf {
        let uid = unsafe { libc::getuid() };
        PathBuf::from(format!("/run/user/{}/biomeos/capabilities.json", uid))
    }
    
    fn save(&self) -> Result<()> {
        let path = Self::get_registry_path();
        std::fs::create_dir_all(path.parent().unwrap())?;
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}
```

#### **Usage in Squirrel:**

```rust
// In squirrel/src/discovery.rs:

use biomeos_registry::CapabilityRegistry;

pub async fn discover_http_provider() -> Result<PathBuf> {
    // Try registry first (fast)
    if let Ok(registry) = CapabilityRegistry::load() {
        if let Some(socket) = registry.get_socket("http.request") {
            if socket.exists() {
                return Ok(socket.clone());
            }
        }
    }
    
    // Fallback to socket scanning
    scan_for_capability("http.request").await
}
```

#### **Registration on Primal Startup:**

```rust
// In each primal's main.rs, after socket creation:

let mut registry = CapabilityRegistry::load()
    .unwrap_or_else(|_| CapabilityRegistry::default());

registry.register_capability(
    "http.request".to_string(),
    CapabilityInfo {
        socket: socket_path.clone(),
        primal: "songbird".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        methods: vec![
            "http.get".to_string(),
            "http.post".to_string(),
            "http.request".to_string(),
        ],
    },
)?;
```

---

### **4. Update Squirrel Configuration** (30 mins)

Add explicit socket configuration alongside capability discovery.

```rust
// In squirrel config:

#[derive(Debug, Deserialize)]
pub struct SquirrelConfig {
    // API Keys
    pub anthropic_api_key: Option<String>,
    pub openai_api_key: Option<String>,
    
    // Provider Sockets (explicit configuration)
    pub http_provider_socket: Option<PathBuf>,
    pub local_ai_provider_socket: Option<PathBuf>,
    pub storage_provider_socket: Option<PathBuf>,
    
    // Discovery settings
    pub enable_capability_discovery: bool,
    pub discovery_timeout_seconds: u64,
}

impl SquirrelConfig {
    pub fn from_env() -> Self {
        Self {
            anthropic_api_key: env::var("ANTHROPIC_API_KEY").ok(),
            openai_api_key: env::var("OPENAI_API_KEY").ok(),
            http_provider_socket: env::var("HTTP_PROVIDER_SOCKET").ok().map(PathBuf::from),
            local_ai_provider_socket: env::var("LOCAL_AI_PROVIDER_SOCKET").ok().map(PathBuf::from),
            storage_provider_socket: env::var("STORAGE_PROVIDER_SOCKET").ok().map(PathBuf::from),
            enable_capability_discovery: env::var("DISABLE_CAPABILITY_DISCOVERY").is_err(),
            discovery_timeout_seconds: env::var("DISCOVERY_TIMEOUT_SECONDS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
        }
    }
}
```

---

### **5. Re-run Integration Test** (1 hour)

With fixes in place, re-run the comprehensive integration test.

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# Clean up any running primals
pkill -f "beardog.*server|songbird.*server|toadstool.*server|nestgate.*server|squirrel.*server"

# Start with proper configuration
BIOMEOS_SOCKET_DIR="/run/user/$(id -u)/biomeos" \
ANTHROPIC_API_KEY="$(grep anthropic_api_key ../../testing-secrets/api-keys.toml | cut -d'"' -f2)" \
OPENAI_API_KEY="$(grep openai_api_key ../../testing-secrets/api-keys.toml | cut -d'"' -f2)" \
./scripts/quick_start_nucleus_test.sh

# In another terminal, run test scenarios:

# 1. Health checks
echo '{"jsonrpc":"2.0","method":"health","id":1}' | nc -U /run/user/$(id -u)/biomeos/squirrel.sock

# 2. GPU query
echo '{"jsonrpc":"2.0","method":"compute.gpu_status","id":2}' | nc -U /run/user/$(id -u)/biomeos/toadstool.jsonrpc.sock

# 3. Online AI query (Anthropic)
echo '{"jsonrpc":"2.0","method":"ai.query","params":{"provider":"anthropic","prompt":"Hello!"},"id":3}' | nc -U /run/user/$(id -u)/biomeos/squirrel.sock

# 4. Storage test
echo '{"jsonrpc":"2.0","method":"storage.store","params":{"key":"test","value":"data"},"id":4}' | nc -U /run/user/$(id -u)/biomeos/nestgate.sock

# All should succeed! ✅
```

---

## 📚 **Documentation to Update**

After fixes:

1. **README.md** - Add socket configuration section
2. **Each primal's README** - Document `BIOMEOS_SOCKET_DIR` env var
3. **Deployment guides** - Update startup commands
4. **Integration test docs** - Update with successful results

---

## 🎯 **Success Criteria**

Integration test v2 should achieve:

- ✅ All 5 primals start successfully
- ✅ All sockets visible in `/run/user/$UID/biomeos/`
- ✅ Health checks pass for all primals
- ✅ Squirrel discovers Songbird's HTTP capability
- ✅ Online AI query works (Anthropic/OpenAI)
- ✅ Local AI inference ready (model loading TBD)
- ✅ Model persistence works (NestGate)
- ✅ Capability routing functional

**Result:** Full NUCLEUS + AI integration operational! 🎉

---

## 💡 **Alternative: Quick Validation Path**

If full integration work isn't immediate priority, validate individual components:

### **Test 1: GPU Compute (Toadstool)**

```bash
# Already working! Test GPU capabilities directly
echo '{"jsonrpc":"2.0","method":"health","id":1}' | nc -U /run/user/$(id -u)/biomeos/toadstool.jsonrpc.sock
```

### **Test 2: Discovery (Songbird)**

```bash
# Test service discovery
echo '{"jsonrpc":"2.0","method":"health","id":1}' | nc -U /run/user/$(id -u)/biomeos/songbird.sock
```

### **Test 3: AI Coordinator (Squirrel)**

```bash
# Test with manual socket configuration
export HTTP_PROVIDER_SOCKET="/run/user/$(id -u)/biomeos/songbird.sock"
# Restart Squirrel with explicit config
```

---

## 🚀 **Ready to Proceed!**

All components are built and tested. Configuration refinement is straightforward.

**Choose your path:**
1. **Full integration** (6 hours) - Complete NUCLEUS + AI
2. **Quick validation** (1 hour) - Test individual components
3. **Production prep** (ongoing) - Service orchestration, monitoring

**The foundation is solid. Time to polish the configuration! 🦀✨**
