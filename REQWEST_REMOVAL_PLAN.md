# 🔥 REQWEST REMOVAL PLAN - ecoBin Compliance

**Date**: January 25, 2026  
**Goal**: Strip ALL reqwest, validate Pure Rust Tower Atomic → GitHub  
**Status**: EXECUTING  

---

## 🎯 Mission

1. **Remove ALL reqwest** from codebase (predates Tower Atomic)
2. **Replace with Songbird delegation** (Pure Rust TLS 1.3)
3. **Validate Tower Atomic** → GitHub HTTPS
4. **Achieve ecoBin compliance**

---

## 📊 reqwest Usage Analysis

### Cargo.toml Files Using reqwest
```
crates/biomeos-core/Cargo.toml        - REMOVE
crates/biomeos-federation/Cargo.toml  - REMOVE  
crates/neural-api-client/Cargo.toml   - REMOVE
crates/biomeos-cli/Cargo.toml         - REMOVE
crates/biomeos-test-utils/Cargo.toml  - REMOVE (test only, ok to keep)
```

### Source Files Using reqwest (40+)
```
crates/biomeos-core/src/
├── primal_client/         - DEPRECATED (marked, can remove)
├── api_adapter/           - DEPRECATED (marked, can remove)
├── clients/               - REMOVE reqwest, add Songbird
├── primal_health.rs       - REMOVE reqwest
├── atomic_client.rs       - Already Pure Rust ✅
├── discovery_http.rs      - REMOVE reqwest
└── ecosystem_*.rs         - REMOVE reqwest
```

---

## 🔧 Execution Plan

### Phase 1: Remove Deprecated Modules (LOW RISK) ✅
These are already marked DEPRECATED, safe to remove:

1. **primal_client/** - Uses reqwest, marked deprecated
2. **api_adapter/** - Uses reqwest, marked deprecated
3. **encrypted_storage/** - Uses HTTP, marked deprecated

**Action**: Delete entire directories

### Phase 2: Replace Active reqwest Usage (MEDIUM RISK)
Files that need Songbird delegation:

1. **clients/** - Universal client layer
2. **primal_health.rs** - Health checking
3. **discovery_http.rs** - HTTP-based discovery
4. **ecosystem_*.rs** - Licensing/integration

**Action**: Replace reqwest with Songbird IPC calls

### Phase 3: Update Cargo.toml (LOW RISK)
Remove reqwest dependency from all production crates

### Phase 4: Validate Tower Atomic (CRITICAL)
Test Pure Rust HTTPS to GitHub

---

## 🚀 Implementation Strategy

### 1. Create Songbird HTTP Client Wrapper
**File**: `crates/biomeos-core/src/http_client.rs` (NEW)

```rust
//! Pure Rust HTTP client via Songbird delegation
//! 
//! Replaces reqwest with Tower Atomic pattern

use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::path::PathBuf;
use biomeos_nucleus::call_unix_socket_rpc;

/// Pure Rust HTTP client using Songbird + BearDog (Tower Atomic)
pub struct HttpClient {
    songbird_socket: PathBuf,
}

impl HttpClient {
    pub fn new() -> Result<Self> {
        let socket = PathBuf::from("/tmp/songbird-nat0.sock");
        Ok(Self { songbird_socket: socket })
    }

    pub async fn get(&self, url: &str) -> Result<HttpResponse> {
        self.request("GET", url, None).await
    }

    pub async fn post(&self, url: &str, body: Option<Value>) -> Result<HttpResponse> {
        self.request("POST", url, body).await
    }

    async fn request(&self, method: &str, url: &str, body: Option<Value>) -> Result<HttpResponse> {
        let mut params = json!({ "url": url, "method": method });
        if let Some(b) = body {
            params["body"] = b;
        }

        let response = call_unix_socket_rpc(
            &self.songbird_socket,
            "http.request",
            params,
        ).await?;

        Ok(HttpResponse {
            status: response["status"].as_u64().unwrap_or(0) as u16,
            body: response["body"].as_str().unwrap_or("").to_string(),
            headers: response["headers"].clone(),
        })
    }
}

pub struct HttpResponse {
    pub status: u16,
    pub body: String,
    pub headers: Value,
}
```

### 2. Remove Deprecated Modules
```bash
rm -rf crates/biomeos-core/src/primal_client/
rm -rf crates/biomeos-core/src/api_adapter/
rm -rf crates/biomeos-core/src/encrypted_storage/
```

### 3. Update Active Files
Replace reqwest usage in:
- `clients/transport/http.rs` 
- `primal_health.rs`
- `discovery_http.rs`
- `ecosystem_integration.rs`

### 4. Update Cargo.toml Files
Remove `reqwest = ...` from all production crates

---

## ✅ Success Criteria

1. ✅ **Zero reqwest** in production code
2. ✅ **Cargo check passes** 
3. ✅ **Tests pass** (biomeos-nucleus: 18/18)
4. ✅ **Tower Atomic → GitHub** HTTPS working
5. ✅ **ecoBin compliant** (Pure Rust, musl builds)

---

## 🎯 Validation Test

After removal, test Tower Atomic:

```bash
# 1. Ensure Songbird + BearDog running
ps aux | grep songbird
ps aux | grep beardog

# 2. Test HTTPS to GitHub via Songbird
echo '{"jsonrpc":"2.0","method":"http.request","params":{"url":"https://api.github.com","method":"GET"},"id":1}' \
  | nc -U /tmp/songbird-nat0.sock

# Expected: HTTP 200 OK with GitHub API response
```

---

## 📋 Execution Order

1. ✅ Create HttpClient wrapper (Songbird delegation)
2. ✅ Remove deprecated modules (primal_client, api_adapter)
3. ✅ Update active files (clients, health, discovery)
4. ✅ Remove reqwest from Cargo.toml
5. ✅ Run cargo check
6. ✅ Run tests
7. ✅ Validate Tower Atomic → GitHub
8. ✅ Commit: "feat: achieve ecoBin compliance - remove reqwest, Pure Rust via Tower Atomic"

---

**LET'S DO THIS!** 🚀

Starting with Phase 1: Remove deprecated modules...

