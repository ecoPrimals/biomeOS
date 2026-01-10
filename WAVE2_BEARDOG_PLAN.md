# 🎯 Wave 2: Smart Refactor beardog.rs - Execution Plan

**Target File**: `crates/biomeos-core/src/clients/beardog.rs`  
**Current Size**: 895 lines  
**Target**: <500 lines per module  
**Status**: Ready to begin

---

## 📊 **Current Structure Analysis**

### **File Breakdown** (895 lines total)

**Main Components**:
1. **BearDogClient struct** (lines 74-77) - Main client
2. **Encryption APIs** (~150 lines) - encrypt, decrypt, key management
3. **Identity APIs** (~150 lines) - verify identity, get identity
4. **Federation APIs** (~200 lines) - verify_family_member, derive_subfed_key, etc.
5. **Trust APIs** (~100 lines) - trust evaluation, access control
6. **BTSP Tunnel APIs** (~150 lines) - establish_tunnel, get_tunnel_status, close_tunnel
7. **Audit APIs** (~50 lines) - audit logging
8. **Helper Types** (~100 lines) - TunnelInfo, EncryptedData, Signature, etc.

### **Natural Domain Boundaries**

The file already has clear conceptual boundaries:
1. **Identity** - Who are you? (DID, verification)
2. **Security** - Encrypt/decrypt data (keys, crypto operations)
3. **Federation** - Family membership, sub-federation keys
4. **Trust** - Access control, trust evaluation
5. **Error Handling** - BearDog-specific errors

---

## 🏗️ **Refactoring Strategy**

### **Target Structure**

```
crates/biomeos-core/src/clients/beardog/
├── mod.rs              # Main client, discovery, re-exports
├── identity.rs         # Identity verification APIs
├── security.rs         # Encryption/decryption APIs  
├── federation.rs       # Federation & family APIs
├── trust.rs            # Trust evaluation & access control
├── types.rs            # Shared types (TunnelInfo, etc.)
└── error.rs            # BearDog-specific errors (future)
```

### **Module Responsibilities**

#### **mod.rs** (~150 lines)
- `BearDogClient` struct definition
- Constructor (`new()`)
- Discovery helpers
- Health check implementation
- Re-export all types
- Module documentation

#### **identity.rs** (~150 lines)
- `verify_identity()` - Verify a DID
- `get_identity()` - Get current identity
- `create_keypair()` - Generate new keys
- Identity-related types
- Documentation & examples

#### **security.rs** (~200 lines)
- `encrypt()` - Encrypt data
- `decrypt()` - Decrypt data
- `sign()` - Generate signature
- `verify_signature()` - Verify signature
- `get_key_info()` - Key metadata
- Security-related types (EncryptedData, Signature, KeyInfo)
- Documentation & examples

#### **federation.rs** (~250 lines)
- `verify_family_member()` - Check family membership
- `derive_subfed_key()` - Derive sub-federation key
- `verify_genetic_lineage()` - Verify lineage
- `establish_tunnel()` - BTSP tunnel setup
- `get_tunnel_status()` - Tunnel status
- `close_tunnel()` - Close tunnel
- Federation types (TunnelInfo, TunnelStatus)
- Documentation & examples

#### **trust.rs** (~100 lines)
- `evaluate_trust()` - Trust evaluation
- `check_access()` - Access control
- `audit_log()` - Audit logging
- Trust types (AccessRequest, AccessDecision, AuditEntry)
- Documentation & examples

#### **types.rs** (~50 lines)
- Shared types used across modules
- Common enums and structs
- Serialization traits

---

## 🎯 **Deep Debt Principles Applied**

### **1. Domain-Focused Modules**
Each module handles ONE domain:
- `identity.rs` - WHO (identity & authentication)
- `security.rs` - WHAT (data protection)
- `federation.rs` - WHERE (network & family)
- `trust.rs` - IF (access control)

### **2. Capability-Based Discovery**
```rust
// mod.rs
impl BearDogClient {
    /// Discover BearDog via capability (no hardcoding!)
    pub async fn discover() -> Result<Self> {
        let registry = PrimalRegistry::new("../plasmidBin");
        let beardog = registry
            .get_best_for_capability(CapabilityTaxonomy::Encryption)?
            .ok_or_else(|| anyhow!("No encryption provider found"))?;
        
        // Use discovered endpoint
        Ok(Self::new(beardog.get_endpoint()?))
    }
}
```

### **3. Clean Module Boundaries**
```rust
// mod.rs (orchestrates, delegates)
pub struct BearDogClient { /* ... */ }

impl BearDogClient {
    pub async fn encrypt(&self, data: &str, key_id: &str) -> Result<EncryptedData> {
        security::encrypt(self, data, key_id).await
    }
}

// security.rs (implements)
pub(crate) async fn encrypt(
    client: &BearDogClient,
    data: &str,
    key_id: &str,
) -> Result<EncryptedData> {
    // Implementation
}
```

### **4. No Duplication**
- Common types in `types.rs`
- Shared error handling in `mod.rs` or `error.rs`
- HTTP client reused across modules

---

## 📝 **Migration Steps**

### **Step 1: Create Module Structure** (5 min)
```bash
mkdir -p crates/biomeos-core/src/clients/beardog
touch crates/biomeos-core/src/clients/beardog/mod.rs
touch crates/biomeos-core/src/clients/beardog/identity.rs
touch crates/biomeos-core/src/clients/beardog/security.rs
touch crates/biomeos-core/src/clients/beardog/federation.rs
touch crates/biomeos-core/src/clients/beardog/trust.rs
touch crates/biomeos-core/src/clients/beardog/types.rs
```

### **Step 2: Extract Types** (10 min)
Move all struct/enum definitions to `types.rs`:
- TunnelInfo
- TunnelStatus
- EncryptedData
- Signature
- KeyInfo
- AccessRequest
- AccessDecision
- AuditEntry

### **Step 3: Create mod.rs** (15 min)
- Move `BearDogClient` struct
- Keep constructor and discovery
- Add delegation methods
- Re-export all types

### **Step 4: Extract Identity APIs** (15 min)
Move to `identity.rs`:
- `verify_identity()`
- `get_identity()`
- `create_keypair()`

### **Step 5: Extract Security APIs** (20 min)
Move to `security.rs`:
- `encrypt()`
- `decrypt()`
- `sign()`
- `verify_signature()`
- `get_key_info()`

### **Step 6: Extract Federation APIs** (25 min)
Move to `federation.rs`:
- `verify_family_member()`
- `derive_subfed_key()`
- `verify_genetic_lineage()`
- `establish_tunnel()`
- `get_tunnel_status()`
- `close_tunnel()`

### **Step 7: Extract Trust APIs** (15 min)
Move to `trust.rs`:
- `evaluate_trust()`
- `check_access()`
- `audit_log()`

### **Step 8: Update Imports** (10 min)
- Update `crates/biomeos-core/src/clients/mod.rs`
- Ensure `pub use` statements work
- Fix any broken imports

### **Step 9: Testing** (15 min)
```bash
cargo test -p biomeos-core --lib
cargo build -p biomeos-core
```

### **Step 10: Documentation** (10 min)
- Update module docs
- Add examples
- Document migration

---

## ⏱️ **Time Estimate**

| Step | Duration |
|------|----------|
| Structure creation | 5 min |
| Extract types | 10 min |
| Create mod.rs | 15 min |
| Extract identity | 15 min |
| Extract security | 20 min |
| Extract federation | 25 min |
| Extract trust | 15 min |
| Update imports | 10 min |
| Testing | 15 min |
| Documentation | 10 min |
| **Total** | **~2.5 hours** |

---

## ✅ **Success Criteria**

1. ✅ No file >500 lines
2. ✅ Each module has single responsibility
3. ✅ All imports work correctly
4. ✅ All tests pass
5. ✅ Zero build errors
6. ✅ Zero linter errors
7. ✅ Documentation complete

---

## 🎯 **Benefits**

### **Before**
```
beardog.rs (895 lines)
❌ Hard to navigate
❌ Mixed concerns
❌ Long compile times
```

### **After**
```
beardog/
├── mod.rs (150 lines) ✅
├── identity.rs (150 lines) ✅
├── security.rs (200 lines) ✅
├── federation.rs (250 lines) ✅
├── trust.rs (100 lines) ✅
└── types.rs (50 lines) ✅

✅ Easy to navigate
✅ Clear boundaries
✅ Faster compilation
✅ Better testing
```

---

## 🚀 **Ready to Execute!**

**Estimated Time**: 2.5 hours  
**Complexity**: Medium  
**Risk**: Low (no logic changes, just reorganization)  
**Impact**: High (better maintainability)

All set to begin! 🎯

