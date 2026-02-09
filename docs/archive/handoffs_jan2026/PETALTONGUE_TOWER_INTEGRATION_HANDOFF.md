# 🎨 petalTongue NUCLEUS Integration Handoff
## February 1, 2026 - Use TOWER Atomic for TLS/HTTP

**Status**: ⏳ **ARCHITECTURAL REFACTOR NEEDED**  
**Priority**: 🟡 **MEDIUM** (1-2 hours)  
**Pattern**: TRUE PRIMAL - Use TOWER for Crypto/TLS

═══════════════════════════════════════════════════════════════════

## 🎯 **ROOT CAUSE: ARCHITECTURAL DEBT**

### **Current Problem**

**petalTongue has direct TLS dependencies**:
```toml
# Current Cargo.toml (WRONG):
openssl = "0.10"  # ❌ C library
# OR
reqwest = { features = ["rustls-tls"] }  # ❌ ring is C!
```

**Why This Is Wrong**:
- ❌ Duplicates TOWER functionality
- ❌ C dependencies (OpenSSL or ring)
- ❌ Violates NUCLEUS architecture
- ❌ Cross-compilation issues
- ❌ Not TRUE PRIMAL

**Build Error Result**:
```
error: failed to run custom build command for `openssl-sys v0.9.111`
Could not find directory of OpenSSL installation
```

### **Why Not "Just Use rustls"?**

**rustls uses ring** (C crypto library):
- ❌ ring = BoringSSL fork (Google's C crypto)
- ❌ Not "pure Rust" despite marketing
- ❌ Still violates TRUE PRIMAL principles
- ❌ Still has cross-compilation issues

**From ring's README**:
> "ring uses BoringSSL's cryptographic primitives"  
> "BoringSSL is Google's fork of OpenSSL"

**Conclusion**: Switching OpenSSL → rustls = same architectural debt!

═══════════════════════════════════════════════════════════════════

## ✅ **PROPER SOLUTION: USE TOWER ATOMIC**

### **NUCLEUS Architecture Pattern**

```
┌─────────────────────────────────────────┐
│  petalTongue (Universal UI)             │
│  - Rendering                            │
│  - User interaction                     │
│  - NO crypto, NO TLS                    │
└──────────────┬──────────────────────────┘
               │ (uses via capability)
               ↓
┌─────────────────────────────────────────┐
│  TOWER Atomic                           │
│  ┌─────────────────┬─────────────────┐  │
│  │  beardog        │  songbird       │  │
│  │  (Sovereign     │  (HTTP/TLS      │  │
│  │   Crypto)       │   Orchestrator) │  │
│  └─────────────────┴─────────────────┘  │
└─────────────────────────────────────────┘
               │
               ↓
         Secure HTTP/TLS
```

### **Responsibilities by Layer**

**petalTongue (UI Layer)**:
- ✅ Rendering graphics (using toadstool/NODE)
- ✅ User interaction
- ✅ UI state management
- ❌ NO crypto
- ❌ NO TLS
- ❌ NO direct HTTP (routes through TOWER)

**TOWER Atomic (Crypto/Security Layer)**:
- ✅ beardog: Sovereign cryptography
- ✅ songbird: HTTP client/server with TLS
- ✅ JWT tokens
- ✅ Certificate management
- ✅ All network security

**Result**: petalTongue has ZERO crypto dependencies!

═══════════════════════════════════════════════════════════════════

## 🔧 **IMPLEMENTATION PLAN**

### **Step 1: Remove Direct HTTP/TLS Dependencies**

**File**: `petalTongue/Cargo.toml`

```toml
# REMOVE these:
openssl = "0.10"  # ❌ Delete
reqwest = "..."   # ❌ Delete (or keep for non-TLS only)
hyper-tls = "..." # ❌ Delete

# KEEP/ADD these:
# For discovering songbird:
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }

# For IPC with songbird (if needed):
# Use Unix sockets or discovery mechanism
```

### **Step 2: Implement HTTP via Songbird Discovery**

**File**: `crates/petal-tongue-http/src/client.rs` (new)

```rust
use serde::{Deserialize, Serialize};

/// HTTP request routed through songbird (TOWER)
#[derive(Debug, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,      // "GET", "POST", etc.
    pub url: String,         // Full URL
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// HTTP client that routes through songbird (TOWER atomic)
pub struct TowerHttpClient {
    songbird_endpoint: String,
}

impl TowerHttpClient {
    /// Discover songbird via capability system
    pub async fn new() -> Result<Self> {
        // Strategy 1: Check for songbird socket (Unix)
        let socket_path = std::env::var("XDG_RUNTIME_DIR")
            .map(|dir| format!("{}/biomeos/songbird.sock", dir))
            .ok();
        
        if let Some(path) = socket_path {
            if std::path::Path::new(&path).exists() {
                return Ok(Self {
                    songbird_endpoint: format!("unix://{}", path),
                });
            }
        }
        
        // Strategy 2: Check for TCP discovery file
        let tcp_file = std::env::var("XDG_RUNTIME_DIR")
            .map(|dir| format!("{}/songbird-ipc-port", dir))
            .ok();
        
        if let Some(file) = tcp_file {
            if let Ok(content) = std::fs::read_to_string(&file) {
                // Format: "tcp:127.0.0.1:36343"
                return Ok(Self {
                    songbird_endpoint: content.trim().to_string(),
                });
            }
        }
        
        Err("songbird (TOWER) not found - HTTP requires TOWER atomic".into())
    }
    
    /// Make HTTP request via songbird
    pub async fn request(
        &self,
        method: &str,
        url: &str,
        headers: Vec<(String, String)>,
        body: Option<Vec<u8>>,
    ) -> Result<HttpResponse> {
        let request = HttpRequest {
            method: method.to_string(),
            url: url.to_string(),
            headers,
            body,
        };
        
        // Send to songbird via IPC (JSON-RPC or tarpc)
        // songbird handles TLS via beardog crypto
        self.send_to_songbird(request).await
    }
    
    async fn send_to_songbird(&self, request: HttpRequest) -> Result<HttpResponse> {
        // Implementation depends on IPC method:
        // - Unix socket + JSON-RPC
        // - TCP + tarpc
        // - Abstract namespace socket
        todo!("Connect to songbird and forward request")
    }
}
```

### **Step 3: Update petalTongue to Use Tower Client**

**File**: `crates/petal-tongue/src/http.rs`

```rust
use crate::http_client::TowerHttpClient;

pub struct PetalTongue {
    http_client: TowerHttpClient,
    // ... other fields
}

impl PetalTongue {
    pub async fn new() -> Result<Self> {
        // Discover and connect to TOWER for HTTP
        let http_client = TowerHttpClient::new().await?;
        
        Ok(Self {
            http_client,
            // ...
        })
    }
    
    /// Fetch resource via TOWER (beardog crypto + songbird HTTP)
    pub async fn fetch(&self, url: &str) -> Result<Vec<u8>> {
        let response = self.http_client
            .request("GET", url, vec![], None)
            .await?;
        
        if response.status != 200 {
            return Err(format!("HTTP {}", response.status).into());
        }
        
        Ok(response.body)
    }
    
    /// Post data via TOWER
    pub async fn post(&self, url: &str, body: Vec<u8>) -> Result<Vec<u8>> {
        let headers = vec![
            ("Content-Type".to_string(), "application/json".to_string()),
        ];
        
        let response = self.http_client
            .request("POST", url, headers, Some(body))
            .await?;
        
        Ok(response.body)
    }
}
```

### **Step 4: Test Without TLS Dependencies**

```bash
# Build should work without OpenSSL/ring:
cd petalTongue
cargo build --release --target x86_64-unknown-linux-musl
cargo build --release --target aarch64-unknown-linux-musl

# Verify no C dependencies:
ldd target/x86_64-unknown-linux-musl/release/petal-tongue
# Expected: "not a dynamic executable" (static binary)

# Test with TOWER:
# 1. Start beardog + songbird (TOWER atomic)
# 2. Start petalTongue
# 3. Make HTTP request -> should route through songbird
```

═══════════════════════════════════════════════════════════════════

## 📊 **VERIFICATION CHECKLIST**

### **Code Quality**

- [ ] Remove all OpenSSL dependencies
- [ ] Remove all rustls/ring dependencies  
- [ ] Implement TowerHttpClient
- [ ] Update all HTTP calls to use Tower client
- [ ] Add songbird discovery mechanism
- [ ] Test IPC connection to songbird

### **Build Validation**

- [ ] Build x86_64-unknown-linux-musl (no OpenSSL errors)
- [ ] Build aarch64-unknown-linux-musl (no OpenSSL errors)
- [ ] Verify static linking (`ldd` shows "not a dynamic executable")
- [ ] Check dependencies (`cargo tree` shows no OpenSSL/ring)

### **Runtime Testing**

- [ ] Test with TOWER atomic running (beardog + songbird)
- [ ] Verify HTTP GET via songbird
- [ ] Verify HTTP POST via songbird
- [ ] Verify HTTPS via beardog crypto
- [ ] Test discovery mechanism (Unix socket + TCP fallback)

### **Documentation**

- [ ] Update README: "Uses TOWER for HTTP/TLS"
- [ ] Document capability discovery
- [ ] Add examples of HTTP via TOWER
- [ ] Note: No crypto dependencies (by design!)

═══════════════════════════════════════════════════════════════════

## 🎯 **BENEFITS**

### **Architectural**

- ✅ TRUE PRIMAL pattern (no crypto in UI layer)
- ✅ TOWER provides all security
- ✅ Single source of crypto truth (beardog)
- ✅ No duplicate TLS implementations
- ✅ Clean layer separation

### **Technical**

- ✅ Zero C dependencies in petalTongue
- ✅ Cross-compilation works automatically
- ✅ Smaller binary (no crypto lib)
- ✅ Faster builds (less code)
- ✅ Static linking guaranteed

### **Security**

- ✅ Sovereign crypto via beardog (not Google/OpenSSL)
- ✅ Single audit point (TOWER only)
- ✅ No hidden C crypto (ring/BoringSSL)
- ✅ TRUE PRIMAL security model

═══════════════════════════════════════════════════════════════════

## ⏱️ **ESTIMATED TIMELINE**

### **Phase 1: Remove Dependencies** (30 minutes)

- Remove OpenSSL/rustls from Cargo.toml
- Remove direct HTTP client code
- Update imports

### **Phase 2: Implement Tower Client** (45 minutes)

- Create TowerHttpClient struct
- Implement songbird discovery
- Implement IPC request forwarding
- Add error handling

### **Phase 3: Integration** (30 minutes)

- Update petalTongue to use Tower client
- Replace all HTTP calls
- Test discovery mechanism

### **Phase 4: Testing** (15 minutes)

- Build for both targets
- Verify static linking
- Runtime testing with TOWER

**Total**: 2 hours

═══════════════════════════════════════════════════════════════════

## 🚨 **CRITICAL NOTES**

### **Why This Matters**

**Deep Debt Context**:
- OpenSSL = C library (obvious)
- rustls = "Pure Rust" (marketing)
- ring = C library (BoringSSL fork - hidden!)

**Truth**: rustls is NOT pure Rust, it's Rust wrapper around C crypto!

**NUCLEUS Pattern**: UI layer has ZERO crypto
- petalTongue = UI only
- TOWER = crypto/security
- Clean separation = TRUE PRIMAL

### **Ecosystem Impact**

**Before**:
```
petalTongue ──> OpenSSL/ring ──> C crypto
beardog     ──> Sovereign crypto (unused!)
```

**After**:
```
petalTongue ──> TOWER ──> beardog (sovereign!)
           (no crypto)
```

**Result**: TRUE PRIMAL architecture! 🎊

═══════════════════════════════════════════════════════════════════

## 📚 **REFERENCES**

### **TOWER Atomic**

- `PIXEL_TOWER_ATOMIC_TCP_SUCCESS.md` - TOWER universal deployment
- beardog: Sovereign cryptography provider
- songbird: HTTP/TLS orchestration

### **Capability Discovery**

- Strategy 1: Unix socket (`$XDG_RUNTIME_DIR/biomeos/songbird.sock`)
- Strategy 2: TCP discovery file (`$XDG_RUNTIME_DIR/songbird-ipc-port`)
- Strategy 3: Abstract namespace (`@songbird` on Android)

### **IPC Protocols**

- JSON-RPC 2.0 (universal)
- tarpc (Rust-native, faster)
- Both supported by songbird

═══════════════════════════════════════════════════════════════════

## 🎯 **SUCCESS CRITERIA**

### **Build**

```bash
$ cargo build --release --target aarch64-unknown-linux-musl
    Finished `release` profile [optimized] target(s)
# ✅ No OpenSSL errors!

$ ldd target/aarch64-unknown-linux-musl/release/petal-tongue
# ✅ not a dynamic executable
```

### **Dependencies**

```bash
$ cargo tree | grep -E "openssl|ring|boring"
# ✅ (empty - no crypto deps!)
```

### **Runtime**

```bash
$ ./petal-tongue --help
petalTongue v1.0.0 - Universal UI
Uses TOWER atomic for HTTP/TLS (beardog crypto)

$ ./petal-tongue start
✅ Discovered songbird: tcp:127.0.0.1:36343
✅ Connected to TOWER for HTTP/TLS
✅ petalTongue ready!
```

═══════════════════════════════════════════════════════════════════

**Created**: February 1, 2026  
**Priority**: 🟡 MEDIUM (2 hours)  
**Pattern**: TRUE PRIMAL - Use TOWER for crypto  
**Impact**: Removes C dependencies, proper NUCLEUS architecture  

🎨🏰 **PETALTONGUE: USE TOWER FOR TLS - TRUE PRIMAL!** 🏰🎨
