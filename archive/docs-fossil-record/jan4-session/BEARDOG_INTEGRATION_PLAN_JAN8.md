# 🐻 BearDog Integration Plan for Spore Incubation & Federation

**Date:** January 8, 2026  
**Status:** ⏳ **Ready for Integration - Awaiting BearDog API Coordination**

---

## 🎯 Overview

This document outlines how biomeOS will integrate with BearDog for cryptographic operations in the spore incubation and hierarchical federation systems. **No crypto is reimplemented** - all operations are delegated to BearDog's HSM.

---

## 🔐 Integration Points

### 1. **Genetic Lineage Verification**

**Location**: `crates/biomeos-federation/src/subfederation.rs`  
**Function**: `SubFederationManager::create()`  
**Line**: ~257

**Current Code**:
```rust
// TODO: Verify genetic lineage of members using BearDog
// This would call BearDog's API to verify all members share the parent_family lineage
```

**Integration**:
```rust
// Verify genetic lineage of all members
for member_id in &members {
    // Get member's deployed seed from local config
    let member_config = load_node_config(member_id)?;
    let deployed_seed = member_config.lineage.deployed_seed_hash;
    
    // Call BearDog API to verify lineage
    let beardog_client = BearDogClient::from_discovery()?; // Use runtime discovery
    let is_family = beardog_client
        .verify_same_family(&parent_family, &deployed_seed)
        .await?;
    
    if !is_family {
        return Err(FederationError::LineageVerificationFailed(format!(
            "Node {} is not part of family {}",
            member_id, parent_family
        )));
    }
}
```

**BearDog API Required**:
- Endpoint: `/api/v1/lineage/verify_family`
- Method: POST
- Request:
  ```json
  {
    "family_id": "nat0",
    "seed_hash": "deployed_seed_hash_here"
  }
  ```
- Response:
  ```json
  {
    "is_family_member": true,
    "parent_seed_hash": "parent_hash",
    "relationship": "sibling"
  }
  ```

---

### 2. **Sub-Federation Encryption Key Generation**

**Location**: `crates/biomeos-federation/src/subfederation.rs`  
**Function**: `SubFederationManager::create()`  
**Line**: ~262

**Current Code**:
```rust
// TODO: Request encryption key from BearDog for this sub-federation
// This would call BearDog's API to generate/derive a key for this sub-federation
// For now, we just leave encryption_key_ref as None
```

**Integration**:
```rust
// Request encryption key for this sub-federation
let beardog_client = BearDogClient::from_discovery()?;
let key_request = KeyDerivationRequest {
    parent_family: parent_family.clone(),
    subfed_name: name.clone(),
    purpose: "sub-federation-encryption".to_string(),
};

let key_response = beardog_client
    .derive_subfed_key(key_request)
    .await?;

// Store only the key reference (not the key itself)
subfed.set_encryption_key_ref(key_response.key_ref);

info!(
    "Generated encryption key for sub-federation '{}': ref={}",
    name, key_response.key_ref
);
```

**BearDog API Required**:
- Endpoint: `/api/v1/keys/derive_subfed_key`
- Method: POST
- Request:
  ```json
  {
    "parent_family": "nat0",
    "subfed_name": "gaming",
    "purpose": "sub-federation-encryption"
  }
  ```
- Response:
  ```json
  {
    "key_ref": "beardog-hsm-key-12345",
    "algorithm": "AES-256-GCM",
    "created_at": "2026-01-08T20:00:00Z"
  }
  ```

---

### 3. **Deployed Seed Encryption (Future Enhancement)**

**Location**: `crates/biomeos-spore/src/incubation.rs`  
**Function**: `SporeIncubator::store_deployed_seed()`  
**Line**: ~410

**Current Code**:
```rust
// Store deployed seed securely
async fn store_deployed_seed(
    &self,
    config_path: &Path,
    deployed_seed: &[u8],
) -> SporeResult<()> {
    let seed_path = config_path.join(".deployed.seed");
    fs::write(&seed_path, deployed_seed).await?;
    
    // Set secure permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&seed_path).await?.permissions();
        perms.set_mode(0o600); // Read/write for owner only
        fs::set_permissions(&seed_path, perms).await?;
    }
    
    debug!("Stored deployed seed securely");
    Ok(())
}
```

**Future Integration** (for high-security deployments):
```rust
// Encrypt deployed seed with BearDog before storing
let beardog_client = BearDogClient::from_discovery()?;
let encrypted_seed = beardog_client
    .encrypt_data(deployed_seed, &self.spore_seed_hash)
    .await?;

fs::write(&seed_path, encrypted_seed).await?;
```

**BearDog API Required**:
- Endpoint: `/api/v1/encrypt`
- Method: POST
- Request:
  ```json
  {
    "data": "base64_encoded_seed",
    "key_ref": "parent_seed_hash",
    "algorithm": "AES-256-GCM"
  }
  ```
- Response:
  ```json
  {
    "encrypted_data": "base64_encoded_encrypted_seed",
    "nonce": "base64_encoded_nonce"
  }
  ```

---

## 🔧 BearDog Client Implementation

**Location**: `crates/biomeos-federation/src/beardog_client.rs` (to be created)

```rust
//! BearDog client for cryptographic operations
//!
//! This client discovers BearDog via runtime discovery and delegates
//! all cryptographic operations to BearDog's HSM.

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::discovery::PrimalDiscovery;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationRequest {
    pub parent_family: String,
    pub subfed_name: String,
    pub purpose: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationResponse {
    pub key_ref: String,
    pub algorithm: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageVerificationRequest {
    pub family_id: String,
    pub seed_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageVerificationResponse {
    pub is_family_member: bool,
    pub parent_seed_hash: String,
    pub relationship: String,
}

pub struct BearDogClient {
    endpoint: String,
}

impl BearDogClient {
    /// Create a BearDog client from runtime discovery
    pub fn from_discovery() -> Result<Self> {
        let mut discovery = PrimalDiscovery::new();
        discovery.discover().await?;
        
        let beardog = discovery
            .get("beardog")
            .ok_or_else(|| anyhow::anyhow!("BearDog not found via discovery"))?;
        
        let endpoint = match &beardog.endpoints[0] {
            crate::discovery::PrimalEndpoint::UnixSocket { path } => {
                format!("unix://{}", path.display())
            }
            crate::discovery::PrimalEndpoint::Udp { addr } => {
                format!("udp://{}", addr)
            }
            crate::discovery::PrimalEndpoint::Http { url } => url.clone(),
        };
        
        Ok(Self { endpoint })
    }
    
    /// Verify if a seed is part of a family
    pub async fn verify_same_family(
        &self,
        family_id: &str,
        seed_hash: &str,
    ) -> Result<bool> {
        let request = LineageVerificationRequest {
            family_id: family_id.to_string(),
            seed_hash: seed_hash.to_string(),
        };
        
        // Make HTTP request to BearDog
        let client = reqwest::Client::new();
        let response: LineageVerificationResponse = client
            .post(format!("{}/api/v1/lineage/verify_family", self.endpoint))
            .json(&request)
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response.is_family_member)
    }
    
    /// Derive a sub-federation encryption key
    pub async fn derive_subfed_key(
        &self,
        request: KeyDerivationRequest,
    ) -> Result<KeyDerivationResponse> {
        let client = reqwest::Client::new();
        let response: KeyDerivationResponse = client
            .post(format!("{}/api/v1/keys/derive_subfed_key", self.endpoint))
            .json(&request)
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response)
    }
    
    /// Encrypt data using BearDog's HSM
    pub async fn encrypt_data(
        &self,
        data: &[u8],
        key_ref: &str,
    ) -> Result<Vec<u8>> {
        // TODO: Implement encryption API call
        unimplemented!("Encryption API not yet defined")
    }
}
```

---

## 📋 Integration Checklist

### Prerequisites
- [ ] BearDog has stable Unix socket API
- [ ] BearDog implements lineage verification endpoint
- [ ] BearDog implements key derivation endpoint
- [ ] BearDog API documentation is complete

### Phase 1: Discovery Integration
- [ ] Add `beardog_client.rs` to `biomeos-federation`
- [ ] Implement `BearDogClient::from_discovery()`
- [ ] Test discovery in local environment

### Phase 2: Lineage Verification
- [ ] Implement `verify_same_family()` in `SubFederationManager::create()`
- [ ] Add error handling for failed lineage verification
- [ ] Write integration tests

### Phase 3: Key Derivation
- [ ] Implement `derive_subfed_key()` in `SubFederationManager::create()`
- [ ] Store key references in sub-federation manifests
- [ ] Test key derivation workflow

### Phase 4: Seed Encryption (Future)
- [ ] Implement `encrypt_data()` for deployed seeds
- [ ] Update `store_deployed_seed()` to use BearDog encryption
- [ ] Add decryption support for seed reading

---

## 🎯 Design Principles

### 1. **Zero Reimplementation**
- **NO** cryptographic code in biomeOS
- **ALL** crypto operations delegated to BearDog
- Only store key references, never actual keys

### 2. **Runtime Discovery**
- Use `PrimalDiscovery` to find BearDog
- Support Unix sockets, UDP, HTTP fallback
- Fail gracefully if BearDog unavailable

### 3. **Composability**
- Clear API boundaries
- BearDog can evolve independently
- biomeOS only depends on API contract

### 4. **Security**
- Store only key references (not keys)
- Use Unix socket permissions (0600)
- Verify lineage before granting access

---

## 🚀 Next Steps

1. **Coordinate with BearDog Team**
   - Share this integration plan
   - Discuss API endpoints
   - Agree on request/response formats

2. **Create Integration Stubs**
   - Add `beardog_client.rs` with stub implementations
   - Document expected behavior
   - Add TODO comments for actual implementation

3. **Write Integration Tests**
   - Mock BearDog responses
   - Test error handling
   - Validate security properties

4. **Deploy and Validate**
   - Test with real BearDog instance
   - Verify genetic lineage checks
   - Validate key derivation

---

## 📚 Related Documentation

- `SPORE_INCUBATION_HIERARCHICAL_FEDERATION_JAN8.md` - System design
- `SPORE_INCUBATION_IMPLEMENTATION_COMPLETE_JAN8.md` - Implementation details
- BearDog API Documentation (external)

---

## ✅ Status

**Current**: ⏳ Awaiting BearDog API Coordination  
**Blocked By**: BearDog lineage verification and key derivation APIs  
**Estimated Effort**: 2-4 hours once BearDog APIs are ready  

**🌟 biomeOS is ready to integrate - waiting on BearDog!**

