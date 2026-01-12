# 🔧 beardog.rs Migration Guide - JSON-RPC over Unix Sockets

**File**: `crates/biomeos-core/src/clients/beardog.rs`  
**Current Size**: 895 lines  
**HTTP References**: 34  
**Priority**: 🔴 **CRITICAL** (Security & Performance)

---

## 📊 **Current State Analysis**

### **HTTP Usage Patterns**

```bash
# HTTP references found:
- PrimalHttpClient: 6 references
- self.http.post(): 15 references  
- self.http.get(): 3 references
- HTTP in docs/examples: 10 references
```

### **Current Structure**

```rust
pub struct BearDogClient {
    http: PrimalHttpClient,  // ❌ HTTP client
    endpoint: String,
}

impl BearDogClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        // ❌ Hardcoded HTTP endpoint
    }
    
    pub async fn encrypt(&self, data: &str, key_id: &str) -> Result<EncryptedData> {
        let response = self.http.post("/api/v1/crypto/encrypt", body).await?;  // ❌ HTTP REST
        // ...
    }
}
```

---

## 🎯 **Target State**

### **New Structure**

```rust
use crate::clients::transport::{PrimalClient, TransportPreference};

pub struct BearDogClient {
    transport: PrimalClient,  // ✅ Protocol-agnostic
    family_id: String,
}

impl BearDogClient {
    /// Auto-discover BearDog via Unix socket
    pub async fn discover(family_id: &str) -> Result<Self> {
        let transport = PrimalClient::discover("beardog", family_id).await?;  // ✅ Auto-discovery
        Ok(Self {
            transport,
            family_id: family_id.to_string(),
        })
    }
    
    /// Legacy: Create from explicit endpoint (HTTP fallback)
    #[deprecated(note = "Use BearDogClient::discover() for Unix socket support")]
    pub async fn from_endpoint(endpoint: impl Into<String>, family_id: &str) -> Result<Self> {
        let transport = PrimalClient::discover_with_preference(
            "beardog",
            family_id,
            TransportPreference::Http
        ).await?;
        Ok(Self {
            transport,
            family_id: family_id.to_string(),
        })
    }
    
    pub async fn encrypt(&self, data: &str, key_id: &str) -> Result<EncryptedData> {
        let response = self.transport.call_method(  // ✅ JSON-RPC
            "encryption.encrypt",
            serde_json::json!({
                "plaintext": base64::encode(data),  // BearDog expects base64
                "key_ref": key_id,
                "algorithm": "AES-256-GCM"
            })
        ).await?;
        // Parse JSON-RPC response...
    }
}
```

---

## 🔄 **API Mapping**

### **BearDog Real JSON-RPC APIs** (from v0.15.2)

| Old HTTP REST | New JSON-RPC | Parameters | Response |
|---------------|--------------|------------|----------|
| `POST /api/v1/crypto/encrypt` | `encryption.encrypt` | `plaintext` (base64), `key_ref`, `algorithm` | `{ciphertext, nonce, tag}` |
| `POST /api/v1/crypto/decrypt` | `encryption.decrypt` | `ciphertext` (base64), `nonce`, `tag`, `key_ref` | `{plaintext, verified}` |
| `POST /api/v1/crypto/sign` | `signing.sign` | `message` (base64), `key_ref`, `algorithm` | `{signature, algorithm}` |
| `POST /api/v1/crypto/verify` | `signing.verify` | `message`, `signature`, `public_key` | `{valid, algorithm}` |
| `POST /api/v1/crypto/generate-key` | `keys.generate` | `algorithm`, `key_id` | `{key_ref, algorithm, created_at}` |
| `POST /api/v1/tunnel/establish` | `btsp.tunnel_establish` | `peer_id`, `endpoint` | `{tunnel_id, peer_id, established_at}` |
| `GET /api/v1/tunnel/status/{id}` | `btsp.tunnel_status` | `tunnel_id` | `{tunnel_id, state, peer_id, ...}` |
| `POST /api/v1/tunnel/close` | `btsp.tunnel_close` | `tunnel_id` | `{success}` |
| `POST /api/v1/access/validate` | `access.validate` | `operation`, `resource`, `context` | `{allowed, reason}` |
| `GET /health` | `capabilities` | `{}` | `{primal, version, capabilities}` |

---

## 📋 **Step-by-Step Migration**

### **Step 1: Update Imports** (Lines 1-55)

**Replace**:
```rust
use crate::clients::base::PrimalHttpClient;
```

**With**:
```rust
use crate::clients::transport::{PrimalClient, TransportPreference};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
```

---

### **Step 2: Update Struct** (Lines 73-77)

**Replace**:
```rust
#[derive(Debug, Clone)]
pub struct BearDogClient {
    http: PrimalHttpClient,
    endpoint: String,
}
```

**With**:
```rust
#[derive(Debug, Clone)]
pub struct BearDogClient {
    transport: PrimalClient,
    family_id: String,
}
```

---

### **Step 3: Add Auto-Discovery Constructor** (Lines 79-90)

**Replace**:
```rust
impl BearDogClient {
    /// Create a new BearDog client
    pub fn new(endpoint: impl Into<String>) -> Self {
        let endpoint = endpoint.into();
        Self {
            http: PrimalHttpClient::new(&endpoint),
            endpoint,
        }
    }
```

**With**:
```rust
impl BearDogClient {
    /// Auto-discover BearDog via Unix socket
    ///
    /// # Arguments
    /// * `family_id` - Genetic family ID
    ///
    /// # Returns
    /// BearDogClient configured with JSON-RPC over Unix socket
    ///
    /// # Example
    /// ```no_run
    /// use biomeos_core::clients::beardog::BearDogClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let beardog = BearDogClient::discover("nat0").await?;
    ///     let encrypted = beardog.encrypt("secret", "my-key").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn discover(family_id: &str) -> Result<Self> {
        let transport = PrimalClient::discover("beardog", family_id).await
            .context("Failed to discover BearDog Unix socket")?;
        
        Ok(Self {
            transport,
            family_id: family_id.to_string(),
        })
    }
    
    /// Create from explicit endpoint (HTTP fallback)
    ///
    /// **DEPRECATED**: Use `discover()` for Unix socket support
    #[deprecated(note = "Use BearDogClient::discover() for Unix socket support")]
    pub async fn from_endpoint(endpoint: impl Into<String>, family_id: &str) -> Result<Self> {
        let transport = PrimalClient::discover_with_preference(
            "beardog",
            family_id,
            TransportPreference::Http
        ).await
            .context("Failed to create HTTP client")?;
        
        Ok(Self {
            transport,
            family_id: family_id.to_string(),
        })
    }
    
    /// Legacy constructor for backward compatibility
    ///
    /// **DEPRECATED**: Use `discover()` instead
    #[deprecated(note = "Use BearDogClient::discover() instead")]
    pub fn new(endpoint: impl Into<String>) -> Self {
        // This is now a sync wrapper that will panic if called
        // Users should migrate to async discover()
        panic!("BearDogClient::new() is deprecated. Use BearDogClient::discover() instead.");
    }
```

---

### **Step 4: Update `encrypt()` Method** (Lines 115-125)

**Replace**:
```rust
pub async fn encrypt(&self, data: &str, key_id: &str) -> Result<EncryptedData> {
    let body = serde_json::json!({
        "data": data,
        "key_id": key_id
    });

    let response = self.http.post("/api/v1/crypto/encrypt", body).await?;

    serde_json::from_value(response)
        .map_err(|e| anyhow::anyhow!("Failed to parse encrypted data: {}", e))
}
```

**With**:
```rust
pub async fn encrypt(&self, data: &str, key_id: &str) -> Result<EncryptedData> {
    // BearDog expects base64-encoded plaintext
    let plaintext_b64 = BASE64.encode(data.as_bytes());
    
    let response = self.transport.call_method(
        "encryption.encrypt",
        serde_json::json!({
            "plaintext": plaintext_b64,
            "key_ref": key_id,
            "algorithm": "AES-256-GCM"
        })
    ).await
        .context("Failed to call encryption.encrypt")?;

    // Parse JSON-RPC response
    let ciphertext = response["ciphertext"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing ciphertext in response"))?
        .to_string();
    
    let nonce = response["nonce"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing nonce in response"))?
        .to_string();
    
    let tag = response["tag"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing tag in response"))?
        .to_string();
    
    Ok(EncryptedData {
        ciphertext,
        key_id: key_id.to_string(),
        algorithm: "AES-256-GCM".to_string(),
        iv: Some(nonce),
    })
}
```

---

### **Step 5: Update Remaining Methods**

Apply the same pattern to all other methods:

1. **`decrypt()`** (Lines 138-150) → `encryption.decrypt`
2. **`sign()`** (Lines 168-170) → `signing.sign`
3. **`verify()`** (Lines 198-200) → `signing.verify`
4. **`generate_key()`** (Lines 222-224) → `keys.generate`
5. **`validate_access()`** (Lines 258-261) → `access.validate`
6. **`establish_tunnel()`** (Lines 297-318) → `btsp.tunnel_establish`
7. **`get_tunnel_status()`** (Lines 344-348) → `btsp.tunnel_status`
8. **`close_tunnel()`** (Lines 377-379) → `btsp.tunnel_close`

---

### **Step 6: Update `PrimalClient` Implementation** (Lines 437-469)

**Replace**:
```rust
#[async_trait]
impl PrimalClient for BearDogClient {
    fn name(&self) -> &str {
        "beardog"
    }

    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    async fn is_available(&self) -> bool {
        self.health_check().await.is_ok()
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let response = self.http.get("/health").await?;
        Ok(HealthStatus {
            healthy: response["status"] == "healthy",
            message: response["message"]
                .as_str()
                .unwrap_or("Unknown")
                .to_string(),
            details: Some(response),
        })
    }

    async fn request(&self, method: &str, path: &str, body: Option<Value>) -> Result<Value> {
        match method {
            "GET" => self.http.get(path).await,
            "POST" => self.http.post(path, body.unwrap_or(Value::Null)).await,
            _ => anyhow::bail!("Unsupported method: {}", method),
        }
    }
}
```

**With**:
```rust
#[async_trait]
impl PrimalClient for BearDogClient {
    fn name(&self) -> &str {
        "beardog"
    }

    fn endpoint(&self) -> &str {
        self.transport.transport_type()  // "unix-socket" or "http"
    }

    async fn is_available(&self) -> bool {
        self.health_check().await.is_ok()
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let response = self.transport.call_method("capabilities", serde_json::json!({})).await?;
        
        Ok(HealthStatus {
            healthy: response.get("primal").is_some(),
            message: format!(
                "BearDog {} via {}",
                response["version"].as_str().unwrap_or("unknown"),
                self.transport.transport_type()
            ),
            details: Some(response),
        })
    }

    async fn request(&self, method: &str, path: &str, body: Option<Value>) -> Result<Value> {
        // Generic JSON-RPC pass-through (for backward compatibility)
        self.transport.call_method(path, body.unwrap_or(Value::Null)).await
    }
}
```

---

### **Step 7: Update Documentation** (Lines 1-47)

Update all doc examples to use `discover()` instead of `new(endpoint)`:

**Before**:
```rust
//! let beardog = BearDogClient::new("http://localhost:9000");
```

**After**:
```rust
//! let beardog = BearDogClient::discover("nat0").await?;
```

---

## 🧪 **Testing Strategy**

### **Unit Tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discover_beardog() {
        // Requires real BearDog running
        let result = BearDogClient::discover("test-family").await;
        // Should find Unix socket or fail gracefully
    }

    #[tokio::test]
    async fn test_encrypt_decrypt_roundtrip() {
        let beardog = BearDogClient::discover("test-family").await
            .expect("BearDog not available");
        
        let plaintext = "test data";
        let encrypted = beardog.encrypt(plaintext, "test-key").await
            .expect("Encryption failed");
        
        let decrypted = beardog.decrypt(&encrypted.ciphertext, "test-key").await
            .expect("Decryption failed");
        
        assert_eq!(plaintext, decrypted);
    }
}
```

### **Integration Tests**

```bash
# Start BearDog with Unix socket
beardog-server --family nat0 --socket /run/user/$(id -u)/beardog-nat0.sock

# Run tests
cargo test --package biomeos-core --test beardog_integration
```

---

## ✅ **Migration Checklist**

- [ ] Update imports (add `PrimalClient`, `TransportPreference`, `base64`)
- [ ] Update struct (replace `http` with `transport`, add `family_id`)
- [ ] Add `discover()` method
- [ ] Add `from_endpoint()` method (deprecated)
- [ ] Deprecate `new()` constructor
- [ ] Update `encrypt()` → `encryption.encrypt`
- [ ] Update `decrypt()` → `encryption.decrypt`
- [ ] Update `sign()` → `signing.sign`
- [ ] Update `verify()` → `signing.verify`
- [ ] Update `generate_key()` → `keys.generate`
- [ ] Update `validate_access()` → `access.validate`
- [ ] Update `establish_tunnel()` → `btsp.tunnel_establish`
- [ ] Update `get_tunnel_status()` → `btsp.tunnel_status`
- [ ] Update `close_tunnel()` → `btsp.tunnel_close`
- [ ] Update `PrimalClient` trait implementation
- [ ] Update all documentation examples
- [ ] Add unit tests for Unix socket discovery
- [ ] Add integration tests with real BearDog
- [ ] Test HTTP fallback (deprecated path)
- [ ] Update CHANGELOG.md
- [ ] Commit with detailed migration notes

---

## 🎯 **Expected Outcomes**

### **Performance**
- **Before**: ~10ms latency (HTTP localhost)
- **After**: ~0.1ms latency (Unix socket)
- **Improvement**: **100x faster** ⚡

### **Security**
- **Before**: ❌ Cleartext HTTP, TCP port exposed (9000)
- **After**: ✅ Unix socket with file permissions (0600), no network

### **Code Quality**
- **Before**: ❌ Hardcoded HTTP endpoints
- **After**: ✅ Auto-discovery, capability-based

---

## 📚 **References**

- [REAL_PRIMAL_APIS_DISCOVERED_JAN8.md](../archive/docs-fossil-record/jan4-session/REAL_PRIMAL_APIS_DISCOVERED_JAN8.md)
- [BearDog v0.15.2 Release Notes](https://github.com/ecoPrimals/beardog/releases/tag/v0.15.2)
- [Transport Abstraction](../crates/biomeos-core/src/clients/transport/mod.rs)

---

**Ready to migrate! Let's make beardog.rs fast, secure, and modern! 🚀**

