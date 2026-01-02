# 🏗️ biomeOS Core - FormatAdapter Evolution Path

**Date**: January 3, 2026  
**Issue**: `FormatAdapter` trait is not dyn-compatible due to generic method  
**Status**: 🔄 Evolution path defined  

---

## 🚨 The Problem

### Error:
```rust
error[E0038]: the trait `FormatAdapter` is not dyn compatible
  --> crates/biomeos-core/src/primal_client/client.rs:23:25
   |
23 |     format_adapter: Arc<dyn FormatAdapter>,
   |                         ^^^^^^^^^^^^^^^^^ `FormatAdapter` is not dyn compatible

note: for a trait to be dyn compatible it needs to allow building a vtable
  --> crates/biomeos-core/src/primal_client/adapters/format/mod.rs:21:14
   |
21 |     async fn parse<T>(&self, response: Response) -> Result<T>
   |              ^^^^^ ...because method `parse` has generic type parameters
```

### Root Cause:
```rust
// Current (doesn't work):
pub trait FormatAdapter: Send + Sync {
    async fn parse<T>(&self, response: Response) -> Result<T>  // ❌ Generic type parameter
    where
        T: DeserializeOwned;
}

// Used as:
format_adapter: Arc<dyn FormatAdapter>  // ❌ Can't make vtable with generic method
```

---

## 🎯 Solution Options

### Option 1: Enum-Based Dispatch (Recommended)

**Pros**:
- No dyn trait needed
- Zero runtime overhead
- Type-safe at compile time
- Easy to add new adapters

**Implementation**:
```rust
// adapters/format/mod.rs

#[derive(Clone)]
pub enum FormatAdapter {
    Auto(AutoFormatAdapter),
    Unwrapped(UnwrappedFormatAdapter),
    Wrapped(WrappedFormatAdapter),
}

impl FormatAdapter {
    pub async fn parse<T>(&self, response: Response) -> Result<T>
    where
        T: DeserializeOwned,
    {
        match self {
            FormatAdapter::Auto(adapter) => adapter.parse(response).await,
            FormatAdapter::Unwrapped(adapter) => adapter.parse(response).await,
            FormatAdapter::Wrapped(adapter) => adapter.parse(response).await,
        }
    }
}

// client.rs
pub struct UniversalPrimalClient {
    format_adapter: FormatAdapter,  // ✅ Concrete type, not trait object
    protocol_adapter: ProtocolAdapter,  // ✅ Can do same for protocol
}
```

---

### Option 2: Type Erasure with `Box<dyn Any>`

**Pros**:
- Keeps trait-based design
- Runtime flexibility

**Cons**:
- More complex
- Runtime overhead
- Less type-safe

**Implementation**:
```rust
pub trait FormatAdapter: Send + Sync {
    fn parse_boxed(&self, response: Response) -> Pin<Box<dyn Future<Output = Result<Box<dyn Any>>> + Send>>;
}

// Then downcast from Box<dyn Any> to concrete type
```

---

### Option 3: Associated Type + Sealed Trait

**Pros**:
- Type-safe
- Clean API

**Cons**:
- More complex type system
- Harder to understand

**Implementation**:
```rust
pub trait FormatAdapter: Send + Sync {
    type ParseFuture<T>: Future<Output = Result<T>> + Send
    where
        T: DeserializeOwned;
        
    fn parse<T>(&self, response: Response) -> Self::ParseFuture<T>
    where
        T: DeserializeOwned;
}
```

---

## ✅ Recommended Approach: Option 1 (Enum)

### Step 1: Define Format Adapter Enum

**File**: `biomeos-core/src/primal_client/adapters/format/mod.rs`

```rust
use super::auto::AutoFormatAdapter;
use super::unwrapped::UnwrappedFormatAdapter;
use super::wrapped::WrappedFormatAdapter;

/// Format adapter enum (replaces trait object)
#[derive(Clone)]
pub enum FormatAdapter {
    /// Auto-detect format from response
    Auto(AutoFormatAdapter),
    
    /// Expect unwrapped responses
    Unwrapped(UnwrappedFormatAdapter),
    
    /// Expect wrapped responses
    Wrapped(WrappedFormatAdapter),
}

impl FormatAdapter {
    /// Parse response into expected type
    pub async fn parse<T>(&self, response: Response) -> Result<T>
    where
        T: DeserializeOwned,
    {
        match self {
            FormatAdapter::Auto(adapter) => adapter.parse(response).await,
            FormatAdapter::Unwrapped(adapter) => adapter.parse(response).await,
            FormatAdapter::Wrapped(adapter) => adapter.parse(response).await,
        }
    }
    
    /// Create auto-detecting adapter (default)
    pub fn auto() -> Self {
        FormatAdapter::Auto(AutoFormatAdapter::new())
    }
    
    /// Create unwrapped adapter
    pub fn unwrapped() -> Self {
        FormatAdapter::Unwrapped(UnwrappedFormatAdapter::new())
    }
    
    /// Create wrapped adapter
    pub fn wrapped() -> Self {
        FormatAdapter::Wrapped(WrappedFormatAdapter::new())
    }
}

impl Default for FormatAdapter {
    fn default() -> Self {
        Self::auto()
    }
}
```

---

### Step 2: Update Individual Adapters

**Remove trait, make concrete types**:

```rust
// auto.rs
#[derive(Clone)]
pub struct AutoFormatAdapter {
    // ...
}

impl AutoFormatAdapter {
    pub fn new() -> Self {
        Self { /* ... */ }
    }
    
    pub async fn parse<T>(&self, response: Response) -> Result<T>
    where
        T: DeserializeOwned,
    {
        // Implementation
    }
}

// unwrapped.rs
#[derive(Clone)]
pub struct UnwrappedFormatAdapter;

impl UnwrappedFormatAdapter {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn parse<T>(&self, response: Response) -> Result<T>
    where
        T: DeserializeOwned,
    {
        response.json::<T>().await
            .map_err(|e| Error::ParseError(e.to_string()))
    }
}

// wrapped.rs
#[derive(Clone)]
pub struct WrappedFormatAdapter;

impl WrappedFormatAdapter {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn parse<T>(&self, response: Response) -> Result<T>
    where
        T: DeserializeOwned,
    {
        #[derive(Deserialize)]
        struct Wrapped<D> {
            success: bool,
            data: Option<D>,
            error: Option<String>,
        }
        
        let wrapped = response.json::<Wrapped<T>>().await
            .map_err(|e| Error::ParseError(e.to_string()))?;
        
        if wrapped.success {
            wrapped.data.ok_or_else(|| Error::ParseError("Missing data".to_string()))
        } else {
            Err(Error::ApiError(wrapped.error.unwrap_or_default()))
        }
    }
}
```

---

### Step 3: Update Client

**File**: `biomeos-core/src/primal_client/client.rs`

```rust
pub struct UniversalPrimalClient {
    endpoint: String,
    protocol_adapter: ProtocolAdapter,  // ← Make this an enum too
    format_adapter: FormatAdapter,      // ← Now concrete enum
    http_client: reqwest::Client,
}

impl UniversalPrimalClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            protocol_adapter: ProtocolAdapter::default(),
            format_adapter: FormatAdapter::default(),
            http_client: reqwest::Client::new(),
        }
    }
    
    pub fn with_format_adapter(mut self, adapter: FormatAdapter) -> Self {
        self.format_adapter = adapter;
        self
    }
    
    pub async fn request<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}/{}", self.endpoint, path);
        let response = self.http_client.get(&url).send().await?;
        
        // Use concrete enum, not trait object
        self.format_adapter.parse(response).await
    }
}
```

---

### Step 4: Same Pattern for Protocol Adapter

```rust
// adapters/protocol/mod.rs

#[derive(Clone)]
pub enum ProtocolAdapter {
    Http(HttpProtocolAdapter),
    Tarpc(TarpcProtocolAdapter),
    // Grpc(GrpcProtocolAdapter),  // Future
}

impl ProtocolAdapter {
    pub async fn request(&self, /* ... */) -> Result<Response> {
        match self {
            ProtocolAdapter::Http(adapter) => adapter.request(/* ... */).await,
            ProtocolAdapter::Tarpc(adapter) => adapter.request(/* ... */).await,
        }
    }
}
```

---

## 🎯 Benefits of Enum Approach

### 1. Zero Runtime Overhead
```rust
// No vtable lookup, direct match
match format_adapter {
    FormatAdapter::Auto(a) => a.parse(response).await,  // Direct call
    // ...
}
```

### 2. Compile-Time Type Safety
```rust
// Compiler knows all variants
let adapter = FormatAdapter::unwrapped();  // ✅ Type-safe
```

### 3. Easy to Extend
```rust
pub enum FormatAdapter {
    Auto(AutoFormatAdapter),
    Unwrapped(UnwrappedFormatAdapter),
    Wrapped(WrappedFormatAdapter),
    Custom(CustomFormatAdapter),  // ← Just add new variant
}
```

### 4. Pattern Matching Power
```rust
// Can match and handle differently
match format_adapter {
    FormatAdapter::Auto(_) => {
        info!("Using auto-detection");
        // ...
    }
    FormatAdapter::Unwrapped(_) => {
        info!("Using unwrapped format");
        // ...
    }
    _ => {}
}
```

---

## 📋 Implementation Checklist

- [ ] Step 1: Create `FormatAdapter` enum
- [ ] Step 2: Update `AutoFormatAdapter` (remove trait, make concrete)
- [ ] Step 3: Update `UnwrappedFormatAdapter` (remove trait, make concrete)
- [ ] Step 4: Update `WrappedFormatAdapter` (remove trait, make concrete)
- [ ] Step 5: Update `UniversalPrimalClient` (use enum, not `Arc<dyn>`)
- [ ] Step 6: Create `ProtocolAdapter` enum (same pattern)
- [ ] Step 7: Update all usage sites
- [ ] Step 8: Run tests
- [ ] Step 9: Update documentation

---

## 🚀 Timeline

**Estimated Effort**: 2-3 hours

**Phase 1** (30 min): Create enums and basic structure  
**Phase 2** (60 min): Update all adapters  
**Phase 3** (30 min): Update client and usage sites  
**Phase 4** (30 min): Testing and documentation  

---

## 💡 Why This is Better Than Traits

### Traits (Current, Broken):
```rust
format_adapter: Arc<dyn FormatAdapter>  // ❌ Can't have generic methods
```

### Enum (Proposed, Works):
```rust
format_adapter: FormatAdapter  // ✅ Concrete type, can have generic methods
```

### Key Insight:
**Trait objects are for when you don't know all implementations at compile time.**

In our case:
- We DO know all format adapters (Auto, Unwrapped, Wrapped)
- They're all in this crate
- We don't need runtime extensibility
- **→ Use enum, not trait object!**

---

## 🎊 After This Fix

### biomeOS API Can Use Universal Primal Client:
```rust
use biomeos_core::primal_client::UniversalPrimalClient;

let client = UniversalPrimalClient::new("http://localhost:9000")
    .with_format_adapter(FormatAdapter::unwrapped());

let identity: BeardogIdentity = client
    .request("api/v1/trust/identity")
    .await?;

// ✅ Type-safe, zero overhead, clean API!
```

---

**Status**: 🎯 Clear evolution path defined  
**Next**: Implement enum-based adapters  
**Benefit**: Universal Primal Client will work for biomeOS API!

🏗️🎯🚀 **Evolution through better design patterns!** 🚀🎯🏗️

