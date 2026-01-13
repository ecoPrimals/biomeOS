# Hardcoding Elimination - Quick Wins Complete

**Date**: January 13, 2026  
**Session**: Hardcoding Evolution - Infant Bootstrapping  
**Status**: ✅ FOUNDATION LAID

---

## ✅ Quick Win 1: FamilyId Discovery Chain

**File**: `crates/biomeos-types/src/identifiers.rs`

**Added Methods**:

### `FamilyId::from_env()`
```rust
/// Get family ID from environment variable
pub fn from_env() -> Option<Self> {
    std::env::var("BIOMEOS_FAMILY_ID")
        .ok()
        .map(Self::new)
}
```

**Usage**: `export BIOMEOS_FAMILY_ID=my-family`

---

### `FamilyId::discover_local()`
```rust
/// Discover local family ID from config
pub fn discover_local() -> Option<Self> {
    // Checks: ~/.config/biomeos/family.txt
}
```

**Usage**: Automatic discovery from config file

---

### `FamilyId::generate()`
```rust
/// Generate a new random family ID
pub fn generate() -> Self {
    // Uses first 8 chars of UUID for memorable IDs
}
```

**Usage**: Automatic fallback when no ID found

---

### `FamilyId::get_or_create()`
```rust
/// Get or create family ID with fallback chain
pub fn get_or_create() -> Self {
    Self::from_env()
        .or_else(Self::discover_local)
        .unwrap_or_else(Self::generate)
}
```

**Priority**:
1. Environment variable (`BIOMEOS_FAMILY_ID`)
2. Local config file (`~/.config/biomeos/family.txt`)
3. Generate new random ID

**Usage**: This is the recommended way to get family ID!

---

### `FamilyId::new_for_test()`
```rust
#[cfg(test)]
pub fn new_for_test() -> Self {
    Self::new("test-family")
}
```

**Usage**: Replace all `"nat0"` hardcoding in tests

---

## ✅ Quick Win 2: BiomeOS Standard API

**File**: `crates/biomeos-types/src/primal/standard_api.rs` (NEW)

**Created Trait**:

```rust
#[async_trait]
pub trait BiomeOSStandardAPI: Send + Sync {
    /// Get primal identity (who am I?)
    async fn biomeos_identity(&self) -> Result<PrimalIdentity, ...>;
    
    /// Get capabilities (what can I do?)
    async fn biomeos_capabilities(&self) -> Result<Vec<PrimalCapability>, ...>;
    
    /// Health check (how am I?)
    async fn biomeos_health(&self) -> Result<HealthStatus, ...>;
    
    /// Get known peers (who do I know?)
    async fn biomeos_peers(&self) -> Result<Vec<PeerInfo>, ...>;
}
```

**Standard JSON-RPC Methods**:
- `biomeos.identity` - Self-reported identity
- `biomeos.capabilities` - What I can do
- `biomeos.health` - How I'm doing
- `biomeos.peers` - Who I know

---

### `PrimalIdentity`
```rust
pub struct PrimalIdentity {
    pub name: String,              // Self-reported name
    pub version: String,           // Semantic version
    pub capabilities: Vec<PrimalCapability>,
    pub description: Option<String>,
}
```

**Usage**: Primals announce themselves, no hardcoding!

---

### `HealthStatus`
```rust
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}
```

**Usage**: Standard health reporting across all primals

---

### `PeerInfo`
```rust
pub struct PeerInfo {
    pub name: String,
    pub capabilities: Vec<PrimalCapability>,
    pub endpoint: String,          // Socket path or URL
    pub last_seen: Option<String>,
}
```

**Usage**: Discovered peers, not hardcoded!

---

## 🎯 Impact

### **Before**:
```rust
// BAD: Hardcoded family
let family = "nat0";

// BAD: Hardcoded primal knowledge
if primal_name == "beardog" {
    // assume it has security capability
}
```

### **After**:
```rust
// GOOD: Discovered family
let family = FamilyId::get_or_create();

// GOOD: Ask primal for identity
let identity = primal.biomeos_identity().await?;
if identity.capabilities.contains(&PrimalCapability::Security) {
    // discovered it has security capability
}
```

---

## 📊 Metrics

| Metric | Before | After | Status |
|--------|--------|-------|--------|
| Family ID hardcoding | 157 instances | 0 target | 🔄 In progress |
| Primal name checks | 1,693 instances | 0 target | 🔄 Ready to fix |
| Standard API | None | ✅ Defined | ✅ Complete |
| Discovery helpers | None | ✅ 5 methods | ✅ Complete |

---

## 🧬 Philosophy Embodied

### **Infant Bootstrapping**:

1. **Born with zero knowledge**
   ```rust
   // At birth, primal knows nothing
   struct MyPrimal;
   ```

2. **Discover environment**
   ```rust
   // Where am I? What family?
   let family = FamilyId::get_or_create();
   let paths = SystemPaths::new()?;
   ```

3. **Announce self**
   ```rust
   // Who am I? What can I do?
   impl BiomeOSStandardAPI for MyPrimal {
       async fn biomeos_identity(&self) -> Result<PrimalIdentity> {
           Ok(PrimalIdentity {
               name: "my-primal".into(),
               capabilities: vec![/* what I can do */],
               ...
           })
       }
   }
   ```

4. **Discover peers**
   ```rust
   // Who else is here?
   let peers = discovery_client
       .find_by_capability(PrimalCapability::Security)
       .await?;
   ```

5. **Compose**
   ```rust
   // Work with discovered peers
   for peer in peers {
       let transport = PrimalTransport::connect(&peer.endpoint).await?;
       // collaborate!
   }
   ```

---

## 🚀 Next Steps

### **Immediate** (1-2 hours):
1. Replace `"nat0"` in tests with `FamilyId::new_for_test()`
2. Update client discovery to use `FamilyId::get_or_create()`
3. Add environment variable documentation

### **Critical Violations** (2-3 hours):
4. Fix `petaltongue_bridge.rs::extract_primal_name()`
5. Fix `discovery.rs` capability inference
6. Remove all `match primal_name` hardcoding

### **Standard API Implementation** (3-4 hours):
7. Implement `BiomeOSStandardAPI` in core primals
8. Add JSON-RPC handlers for standard methods
9. Update documentation

---

## ✅ Checklist

- [x] FamilyId environment discovery
- [x] FamilyId local config discovery
- [x] FamilyId generation fallback
- [x] FamilyId test helper
- [x] BiomeOS Standard API trait
- [x] PrimalIdentity type
- [x] HealthStatus enum
- [x] PeerInfo type
- [x] Documentation
- [x] biomeos-types builds successfully
- [ ] Replace "nat0" in tests
- [ ] Fix primal name inference
- [ ] Implement standard API in primals

---

## 📝 Files Changed

**Modified**:
- `crates/biomeos-types/src/identifiers.rs` - Added FamilyId discovery
- `crates/biomeos-types/src/primal/mod.rs` - Added standard_api module

**Created**:
- `crates/biomeos-types/src/primal/standard_api.rs` - Standard API trait

**Next to Modify**:
- `crates/biomeos-ui/src/petaltongue_bridge.rs` - Fix name inference
- `crates/biomeos-federation/src/discovery.rs` - Fix capability inference
- All test files with `"nat0"` - Use helper

---

**Status**: ✅ QUICK WINS COMPLETE  
**Build**: ✅ biomeos-types compiles  
**Foundation**: ✅ Laid for TRUE PRIMAL evolution  

**Next Session**: Fix critical violations, implement standard API

🧬 **"Born knowing nothing, discovering everything"** 🌱

