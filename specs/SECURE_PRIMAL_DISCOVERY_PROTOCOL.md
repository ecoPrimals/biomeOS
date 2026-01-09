# 🔒 Secure Primal Discovery Protocol (SPDP)

**Version**: 1.0.0  
**Date**: January 9, 2026  
**Status**: 🚨 **SPECIFICATION** - Implementation Required  
**Priority**: HIGH - Security Critical

---

## 🎯 Purpose

Define a secure, authenticated, capability-based discovery protocol that:
- ✅ Leverages existing primal capabilities (no reimplementation)
- ✅ Prevents socket hijacking attacks
- ✅ Supports multiple instance selection
- ✅ Provides cryptographic verification
- ✅ Enables auditable discovery

---

## 🏗️ Architecture: Primal Responsibilities

### **Core Principle**: Don't Reimplement - Delegate to Primals!

| Capability | Responsible Primal | What It Provides |
|------------|-------------------|------------------|
| **Cryptographic Identity** | 🐻 BearDog | Socket signature, process verification |
| **Discovery & Announcement** | 🐦 Songbird | UDP multicast, registry, capability indexing |
| **Trust Evaluation** | 🐻 BearDog | Family verification, lineage validation |
| **Communication** | 🐦 Songbird | Unix socket routing, JSON-RPC transport |
| **Encryption** | 🐻 BearDog | Message signing, tunnel establishment |

**biomeOS's Role**: Coordinate discovery protocol using primal capabilities

---

## 📋 Protocol Layers

### **Layer 1: Physical Discovery** (Songbird)

**Method**: UDP Multicast (BirdSong Protocol)

```rust
// Songbird broadcasts primal announcements
{
  "protocol": "birdsong",
  "version": "1.0",
  "primal": "beardog",
  "node_id": "node-alpha",
  "family_id": "nat0",
  "capabilities": ["encryption", "identity", "trust"],
  "endpoints": [
    {"type": "unix_socket", "path": "/tmp/beardog-nat0-node-alpha.sock"},
    {"type": "udp", "addr": "224.0.0.251:5353"}
  ],
  "signature": "<ed25519_signature>",  // Signed by BearDog
  "timestamp": "2026-01-09T01:00:00Z"
}
```

**Songbird APIs (Unix Socket JSON-RPC)**:
- `discover_by_family(family_id)` → Returns all primals in family
- `discover_by_capability(capability)` → Returns primals with capability
- `announce_capabilities(primal_info)` → Register this primal
- `subscribe_discoveries()` → Stream of new primal announcements

**Reference**: `ecoPrimals/phase1/songbird/`

---

### **Layer 2: Identity Verification** (BearDog)

**Method**: Cryptographic Challenge-Response

```rust
// Step 1: Request identity proof from discovered socket
{
  "jsonrpc": "2.0",
  "method": "identity.get_proof",
  "params": {
    "challenge": "<random_nonce>"
  },
  "id": 1
}

// Step 2: Socket returns signed proof
{
  "jsonrpc": "2.0",
  "result": {
    "primal_name": "beardog",
    "node_id": "node-alpha",
    "family_id": "nat0",
    "version": "0.15.2",
    "process_id": 12345,
    "socket_path": "/tmp/beardog-nat0-node-alpha.sock",
    "owner_uid": 1000,
    "owner_gid": 1000,
    "started_at": "2026-01-09T00:00:00Z",
    "challenge": "<same_nonce>",
    "signature": "<ed25519_signature>"  // Signed by BearDog's family key
  },
  "id": 1
}

// Step 3: Verify signature via BearDog
{
  "jsonrpc": "2.0",
  "method": "security.verify_primal_identity",
  "params": {
    "identity_proof": "<proof_from_step2>",
    "family_id": "nat0"
  },
  "id": 2
}
```

**BearDog APIs (Unix Socket JSON-RPC)**:
- `identity.get_proof(challenge)` → Returns signed identity
- `security.verify_primal_identity(proof, family)` → Verifies signature
- `federation.verify_family_member(seed, reference)` → Genetic lineage
- `security.lineage(family_id, seed_hash, node_id)` → Lineage info

**Reference**: `ecoPrimals/phase1/beardog/`

---

### **Layer 3: Capability Verification** (Query Socket)

**Method**: Direct Socket Query (via biomeOS)

```rust
// Step 1: Connect to discovered socket
let client = UnixSocketClient::new("/tmp/beardog-nat0-node-alpha.sock");

// Step 2: Query capabilities
{
  "jsonrpc": "2.0",
  "method": "get_capabilities",
  "params": {},
  "id": 1
}

// Step 3: Validate response matches Songbird announcement
{
  "jsonrpc": "2.0",
  "result": {
    "primal": "beardog",
    "version": "0.15.2",
    "family_id": "nat0",
    "node_id": "node-alpha",
    "protocols": ["tarpc", "json-rpc"],
    "provided_capabilities": [
      {"type": "security", "methods": ["evaluate", "lineage"], "version": "1.0"},
      {"type": "encryption", "methods": ["encrypt", "decrypt"], "version": "1.0"},
      {"type": "trust", "methods": ["evaluate", "lineage"], "version": "1.0"}
    ]
  },
  "id": 1
}
```

**biomeOS's Role**: Query socket, validate against announcement

---

### **Layer 4: Trust Evaluation** (BearDog)

**Method**: Family Lineage & Trust Policy

```rust
// Query BearDog for trust evaluation
{
  "jsonrpc": "2.0",
  "method": "security.evaluate",
  "params": {
    "subject": {
      "primal": "beardog",
      "node_id": "node-beta",
      "family_id": "nat0",
      "socket_path": "/tmp/beardog-nat0-node-beta.sock"
    },
    "context": "primal_discovery",
    "required_relationship": "sibling"  // parent/child/sibling/any
  },
  "id": 1
}

// BearDog returns trust evaluation
{
  "jsonrpc": "2.0",
  "result": {
    "trusted": true,
    "relationship": "sibling",
    "trust_level": "family",
    "verified_via": "genetic_lineage_hkdf",
    "parent_family": "nat0",
    "reason": "Verified sibling via shared parent seed"
  },
  "id": 1
}
```

**BearDog APIs**:
- `security.evaluate(subject, context, policy)` → Trust decision
- `federation.verify_family_member(seed, reference)` → Lineage check

---

## 🔐 Complete Discovery Flow

### **Secure Discovery Sequence**

```
┌─────────────────────────────────────────────────────────────────┐
│ Phase 1: Physical Discovery (Songbird)                         │
├─────────────────────────────────────────────────────────────────┤
│ 1. biomeOS → Songbird: discover_by_family("nat0")              │
│ 2. Songbird → biomeOS: [primal_announcements]                  │
│    - Each announcement signed by BearDog                        │
│    - Includes socket path, capabilities, metadata               │
└─────────────────────────────────────────────────────────────────┘
         ↓
┌─────────────────────────────────────────────────────────────────┐
│ Phase 2: Identity Verification (BearDog)                       │
├─────────────────────────────────────────────────────────────────┤
│ For each discovered primal:                                     │
│ 3. biomeOS → Socket: identity.get_proof(challenge)             │
│ 4. Socket → biomeOS: signed_proof                              │
│ 5. biomeOS → BearDog: security.verify_primal_identity(proof)   │
│ 6. BearDog → biomeOS: verification_result                      │
│    - Validates signature                                        │
│    - Checks family membership                                   │
│    - Confirms socket authenticity                               │
└─────────────────────────────────────────────────────────────────┘
         ↓
┌─────────────────────────────────────────────────────────────────┐
│ Phase 3: Capability Verification (biomeOS)                     │
├─────────────────────────────────────────────────────────────────┤
│ 7. biomeOS → Socket: get_capabilities()                        │
│ 8. Socket → biomeOS: capabilities                              │
│ 9. biomeOS: Validate capabilities match announcement           │
│    - Cross-check with Songbird announcement                     │
│    - Verify version compatibility                               │
│    - Check required capabilities present                        │
└─────────────────────────────────────────────────────────────────┘
         ↓
┌─────────────────────────────────────────────────────────────────┐
│ Phase 4: Trust Evaluation (BearDog)                            │
├─────────────────────────────────────────────────────────────────┤
│ 10. biomeOS → BearDog: security.evaluate(subject, policy)      │
│ 11. BearDog → biomeOS: trust_decision                          │
│     - Evaluates genetic lineage                                 │
│     - Checks trust policy                                       │
│     - Returns trust level + reason                              │
└─────────────────────────────────────────────────────────────────┘
         ↓
┌─────────────────────────────────────────────────────────────────┐
│ Phase 5: Registration (biomeOS)                                │
├─────────────────────────────────────────────────────────────────┤
│ 12. biomeOS registers verified primal                           │
│     - Adds to local registry                                    │
│     - Tags with trust level                                     │
│     - Logs audit trail                                          │
│     - Available for use                                         │
└─────────────────────────────────────────────────────────────────┘
```

---

## 💻 Implementation: biomeos-federation

### **Updated PrimalDiscovery**

**File**: `crates/biomeos-federation/src/discovery.rs`

```rust
use crate::beardog_client::BearDogClient;
use crate::songbird_client::SongbirdClient;

pub struct SecurePrimalDiscovery {
    songbird: SongbirdClient,
    beardog: BearDogClient,
    verified_primals: HashMap<String, Vec<VerifiedPrimal>>,
}

#[derive(Debug, Clone)]
pub struct VerifiedPrimal {
    pub name: String,
    pub node_id: String,
    pub family_id: String,
    pub endpoints: Vec<PrimalEndpoint>,
    pub capabilities: CapabilitySet,
    pub identity_proof: IdentityProof,
    pub trust_level: TrustLevel,
    pub discovered_at: DateTime<Utc>,
    pub verified_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct IdentityProof {
    pub primal_name: String,
    pub process_id: u32,
    pub owner_uid: u32,
    pub socket_path: PathBuf,
    pub signature: Vec<u8>,
    pub verified_by_beardog: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrustLevel {
    Unverified,       // Not yet verified
    Authenticated,    // Identity verified
    Trusted,          // Family member (sibling)
    HighlyTrusted,    // Direct parent/child
}

impl SecurePrimalDiscovery {
    /// Create new secure discovery
    pub async fn new() -> Result<Self> {
        // Bootstrap: Find Songbird and BearDog first
        let songbird = SongbirdClient::from_bootstrap().await?;
        let beardog = BearDogClient::from_bootstrap().await?;
        
        Ok(Self {
            songbird,
            beardog,
            verified_primals: HashMap::new(),
        })
    }
    
    /// Discover primals securely
    pub async fn discover_secure(&mut self, family_id: &str) -> Result<Vec<VerifiedPrimal>> {
        info!("Starting secure discovery for family: {}", family_id);
        
        // Phase 1: Physical discovery via Songbird
        let announcements = self.songbird
            .discover_by_family(family_id)
            .await?;
        
        info!("Discovered {} primal announcements", announcements.len());
        
        let mut verified = Vec::new();
        
        for announcement in announcements {
            // Phase 2: Identity verification via BearDog
            match self.verify_primal_identity(&announcement).await {
                Ok(identity_proof) => {
                    // Phase 3: Capability verification
                    match self.verify_capabilities(&announcement).await {
                        Ok(capabilities) => {
                            // Phase 4: Trust evaluation via BearDog
                            match self.evaluate_trust(&announcement, &identity_proof).await {
                                Ok(trust_level) => {
                                    let verified_primal = VerifiedPrimal {
                                        name: announcement.primal.clone(),
                                        node_id: announcement.node_id.clone(),
                                        family_id: announcement.family_id.clone(),
                                        endpoints: announcement.endpoints.clone(),
                                        capabilities,
                                        identity_proof,
                                        trust_level,
                                        discovered_at: announcement.timestamp,
                                        verified_at: Utc::now(),
                                    };
                                    
                                    info!(
                                        "✅ Verified primal: {} (node: {}, trust: {:?})",
                                        verified_primal.name,
                                        verified_primal.node_id,
                                        verified_primal.trust_level
                                    );
                                    
                                    verified.push(verified_primal.clone());
                                    
                                    // Add to registry
                                    self.verified_primals
                                        .entry(verified_primal.name.clone())
                                        .or_insert_with(Vec::new)
                                        .push(verified_primal);
                                }
                                Err(e) => {
                                    warn!(
                                        "⚠️  Trust evaluation failed for {} (node: {}): {}",
                                        announcement.primal, announcement.node_id, e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            warn!(
                                "⚠️  Capability verification failed for {} (node: {}): {}",
                                announcement.primal, announcement.node_id, e
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "⚠️  Identity verification failed for {} (node: {}): {}",
                        announcement.primal, announcement.node_id, e
                    );
                }
            }
        }
        
        info!("Secure discovery complete: {} verified primals", verified.len());
        
        Ok(verified)
    }
    
    /// Verify primal identity via BearDog
    async fn verify_primal_identity(
        &self,
        announcement: &PrimalAnnouncement,
    ) -> Result<IdentityProof> {
        // Generate challenge
        let challenge = generate_random_nonce();
        
        // Connect to primal's socket
        let socket_path = announcement.get_unix_socket_path()
            .ok_or_else(|| anyhow!("No Unix socket endpoint"))?;
        
        let client = UnixSocketClient::new(&socket_path);
        
        // Request identity proof
        let response = client
            .call_method("identity.get_proof", json!({
                "challenge": challenge
            }))
            .await?;
        
        let proof: IdentityProof = serde_json::from_value(response)?;
        
        // Verify signature via BearDog
        let verified = self.beardog
            .verify_primal_identity(&proof, &announcement.family_id)
            .await?;
        
        if !verified {
            return Err(anyhow!("BearDog rejected identity proof"));
        }
        
        Ok(proof)
    }
    
    /// Verify capabilities match announcement
    async fn verify_capabilities(
        &self,
        announcement: &PrimalAnnouncement,
    ) -> Result<CapabilitySet> {
        let socket_path = announcement.get_unix_socket_path()
            .ok_or_else(|| anyhow!("No Unix socket endpoint"))?;
        
        let client = UnixSocketClient::new(&socket_path);
        
        // Query actual capabilities
        let response = client
            .call_method("get_capabilities", json!({}))
            .await?;
        
        let actual_caps: CapabilitySet = parse_capabilities(&response)?;
        
        // Verify matches announcement
        if !actual_caps.matches(&announcement.capabilities) {
            return Err(anyhow!(
                "Capability mismatch: announced {:?}, actual {:?}",
                announcement.capabilities,
                actual_caps
            ));
        }
        
        Ok(actual_caps)
    }
    
    /// Evaluate trust via BearDog
    async fn evaluate_trust(
        &self,
        announcement: &PrimalAnnouncement,
        identity_proof: &IdentityProof,
    ) -> Result<TrustLevel> {
        let trust_result = self.beardog
            .evaluate_trust(
                &announcement.primal,
                &announcement.node_id,
                &announcement.family_id,
                identity_proof,
            )
            .await?;
        
        Ok(match trust_result.relationship.as_str() {
            "sibling" => TrustLevel::Trusted,
            "parent" | "child" => TrustLevel::HighlyTrusted,
            _ => TrustLevel::Authenticated,
        })
    }
    
    /// Get all verified instances of a primal
    pub fn get_all(&self, name: &str) -> Vec<&VerifiedPrimal> {
        self.verified_primals
            .get(name)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }
    
    /// Get primal by selection criteria
    pub fn get_by_criteria(
        &self,
        name: &str,
        criteria: &SelectionCriteria,
    ) -> Option<&VerifiedPrimal> {
        let all = self.get_all(name);
        
        match criteria {
            SelectionCriteria::Any => {
                // Return highest trust level
                all.into_iter()
                    .max_by_key(|p| &p.trust_level)
            }
            SelectionCriteria::ByNodeId(node_id) => {
                all.into_iter()
                    .find(|p| &p.node_id == node_id)
            }
            SelectionCriteria::ByFamily(family) => {
                all.into_iter()
                    .filter(|p| &p.family_id == family)
                    .max_by_key(|p| &p.trust_level)
            }
            SelectionCriteria::BySocket(path) => {
                all.into_iter()
                    .find(|p| p.identity_proof.socket_path == *path)
            }
            SelectionCriteria::MinTrustLevel(min_trust) => {
                all.into_iter()
                    .filter(|p| p.trust_level >= *min_trust)
                    .max_by_key(|p| &p.trust_level)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum SelectionCriteria {
    Any,                           // Highest trust level
    ByNodeId(String),             // Specific node
    ByFamily(String),             // Specific family
    BySocket(PathBuf),            // Specific socket
    MinTrustLevel(TrustLevel),    // Minimum trust required
}
```

---

## 🧪 Testing Strategy

### **Unit Tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_identity_verification_rejects_invalid_signature() {
        let discovery = SecurePrimalDiscovery::new().await.unwrap();
        
        let fake_announcement = create_fake_announcement();
        
        let result = discovery.verify_primal_identity(&fake_announcement).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("BearDog rejected"));
    }
    
    #[tokio::test]
    async fn test_capability_mismatch_rejected() {
        let discovery = SecurePrimalDiscovery::new().await.unwrap();
        
        let announcement = create_announcement_with_caps(vec!["encryption"]);
        // But socket returns different capabilities
        
        let result = discovery.verify_capabilities(&announcement).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Capability mismatch"));
    }
    
    #[tokio::test]
    async fn test_trust_evaluation_for_siblings() {
        let discovery = SecurePrimalDiscovery::new().await.unwrap();
        
        let announcement = create_sibling_announcement();
        let proof = create_valid_proof();
        
        let trust_level = discovery.evaluate_trust(&announcement, &proof).await.unwrap();
        
        assert_eq!(trust_level, TrustLevel::Trusted);
    }
}
```

### **Integration Tests**

```rust
#[tokio::test]
async fn test_secure_discovery_e2e() {
    // Start real Songbird and BearDog
    let songbird = start_songbird("nat0").await;
    let beardog = start_beardog("nat0").await;
    
    // Announce a test primal
    songbird.announce_capabilities(create_test_primal()).await.unwrap();
    
    // Discover securely
    let mut discovery = SecurePrimalDiscovery::new().await.unwrap();
    let verified = discovery.discover_secure("nat0").await.unwrap();
    
    assert_eq!(verified.len(), 1);
    assert_eq!(verified[0].trust_level, TrustLevel::Trusted);
}
```

### **Security Tests**

```rust
#[tokio::test]
async fn test_socket_hijacking_prevented() {
    // Create malicious socket
    let evil_socket = create_evil_socket("/tmp/beardog-nat0-evil.sock").await;
    
    // Announce it (with invalid signature)
    announce_via_songbird(evil_socket).await;
    
    // Try to discover
    let mut discovery = SecurePrimalDiscovery::new().await.unwrap();
    let verified = discovery.discover_secure("nat0").await.unwrap();
    
    // Evil socket should NOT be in verified list
    assert!(!verified.iter().any(|p| p.identity_proof.socket_path.to_str().unwrap().contains("evil")));
}
```

---

## 📊 Performance Considerations

### **Caching Strategy**

```rust
pub struct VerifiedPrimalCache {
    cache: HashMap<String, (VerifiedPrimal, Instant)>,
    ttl: Duration,
}

impl VerifiedPrimalCache {
    /// Get cached primal if still valid
    pub fn get(&self, key: &str) -> Option<&VerifiedPrimal> {
        self.cache.get(key).and_then(|(primal, cached_at)| {
            if cached_at.elapsed() < self.ttl {
                Some(primal)
            } else {
                None
            }
        })
    }
    
    /// Cache verified primal
    pub fn insert(&mut self, key: String, primal: VerifiedPrimal) {
        self.cache.insert(key, (primal, Instant::now()));
    }
}
```

### **Parallel Verification**

```rust
/// Verify multiple primals concurrently
pub async fn discover_secure_parallel(
    &mut self,
    family_id: &str,
) -> Result<Vec<VerifiedPrimal>> {
    let announcements = self.songbird.discover_by_family(family_id).await?;
    
    // Verify all in parallel
    let futures = announcements.iter().map(|announcement| {
        self.verify_primal(announcement)
    });
    
    let results = join_all(futures).await;
    
    Ok(results.into_iter().filter_map(Result::ok).collect())
}
```

---

## 🚀 Migration Path

### **Phase 1: Add Secure Discovery (Week 1)**
- [ ] Implement `SecurePrimalDiscovery`
- [ ] Add `VerifiedPrimal` types
- [ ] Integrate Songbird discovery client
- [ ] Integrate BearDog verification
- [ ] Add unit tests

### **Phase 2: Update Clients (Week 2)**
- [ ] Update `BearDogClient::from_discovery()` to use secure discovery
- [ ] Add `SelectionCriteria` support
- [ ] Update `verify-lineage` tool
- [ ] Add integration tests

### **Phase 3: Deprecate Insecure (Week 3)**
- [ ] Mark old `PrimalDiscovery` as deprecated
- [ ] Add migration warnings
- [ ] Update all biomeOS tools
- [ ] Document migration guide

### **Phase 4: Remove Insecure (Week 4)**
- [ ] Remove old discovery code
- [ ] Clean up legacy APIs
- [ ] Update documentation
- [ ] Celebrate secure discovery! 🎉

---

## 🎯 Success Criteria

✅ **Security**: No socket hijacking possible  
✅ **Verification**: All primals cryptographically verified  
✅ **Performance**: Discovery completes in < 1 second  
✅ **Control**: Explicit instance selection supported  
✅ **Delegation**: Uses Songbird for discovery, BearDog for crypto  
✅ **Testing**: 100% coverage for security-critical paths  
✅ **Documentation**: Clear API contracts for all primals  

---

## 📚 References

- **Songbird**: `ecoPrimals/phase1/songbird/`
- **BearDog**: `ecoPrimals/phase1/beardog/`
- **Primal APIs**: `docs/jan4-session/PRIMAL_API_HANDOFF_TO_BEARDOG_SONGBIRD_JAN8.md`
- **Control Gap Analysis**: `docs/EMERGENT_BEHAVIOR_AND_CONTROL_GAP_JAN9.md`

---

**This specification delegates to primals, doesn't reimplement them!**

🔒 **Secure by Design, Composable by Nature** 🚀

