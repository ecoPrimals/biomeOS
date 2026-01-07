# 🔐 Trust Policy Evolution - Configurable, Genetically Secured, Contact Key Exchange

**Date**: January 7, 2026  
**Status**: 🎯 **ARCHITECTURAL VISION**  
**Priority**: **CRITICAL** - Foundation for fractal, isomorphic federation

---

## 🎯 Vision

Instead of just fixing the immediate `trust_level` schema mismatch (integer vs string), evolve the trust system to be:

1. **Configurable**: Not hardcoded - different orgs/federations/people define their own trust policies
2. **Genetically Secured**: Trust policies tied to family seed, cryptographically verifiable
3. **Dynamic**: Can be updated as relationships evolve
4. **Contact Key Exchange**: When LAN towers connect, mix and share a contact key for NAT/P2P

**Philosophy**: "Trust is not a number, it's a relationship secured by shared genetic lineage and cryptographic proof."

---

## 📊 Current State Analysis

### BearDog (`beardog-node-registry/src/node_registry/types/trust.rs`)

**Current Implementation**:
```rust
pub enum TrustLevel {
    Unknown = 0,    // Hardcoded
    Basic = 1,      // Hardcoded
    Medium = 2,     // Hardcoded
    High = 3,       // Hardcoded
    Explicit = 4,   // Hardcoded
}
```

**Features**:
- ✅ Integer representation (compact)
- ✅ Trust propagation with decay
- ✅ `TrustStore` for relationship management
- ❌ Hardcoded levels (not configurable)
- ❌ No capability-based permissions
- ❌ No genetic seed integration

---

### Songbird (`songbird-types/src/trust.rs`)

**Current Implementation**:
```rust
pub enum TrustLevel {
    None = 0,       // Hardcoded
    Limited = 1,    // Hardcoded (same family)
    Elevated = 2,   // Hardcoded (human approved)
    Highest = 3,    // Hardcoded (human entropy)
}

pub struct TrustEvaluation {
    pub level: TrustLevel,
    pub allowed_capabilities: Vec<String>,   // ✅ CONFIGURABLE!
    pub denied_capabilities: Vec<String>,    // ✅ CONFIGURABLE!
    pub elevation_path: Option<ElevationPath>,  // ✅ PROGRESSIVE!
    // ...
}
```

**Features**:
- ✅ String names ("none", "limited", "elevated", "highest")
- ✅ **Capability-based permissions** (already implemented!)
- ✅ Wildcard patterns ("data/*", "birdsong/*")
- ✅ Progressive trust elevation
- ✅ Philosophy: "Same family = can hear the song, NOT enter the nest"
- ❌ Trust levels still hardcoded (0-3)
- ❌ No genetic seed policy integration

---

## 🚧 The Immediate Problem

**Schema Mismatch**:
- BearDog returns: `{"trust_level": 0}` (integer)
- Songbird expects: `{"trust_level": "anonymous"}` (string)
- Result: Parse error, federation blocked

**Naive Fix**: Convert BearDog to return strings
**Problem**: Still hardcoded, not extensible

---

## 🎯 The Evolution Plan

### Phase 1: Short-Term Fix (Backward Compatible) ⏰ **1-2 days**

**Goal**: Unblock federation while maintaining extensibility

**Solution**: Dual representation in JSON-RPC responses

```rust
// BearDog JSON-RPC response (backward compatible)
{
    "trust_level": 1,              // Integer (for compact representation)
    "trust_level_name": "limited", // String (for human readability)
    "reason": "same_genetic_family",
    "peer_family": "nat0",
    "our_family": "nat0",
    "capabilities": {              // NEW: Capability hints
        "allowed": ["birdsong/*", "coordination/*", "health"],
        "denied": ["data/*", "commands/*", "keys/*"]
    }
}
```

**Songbird Update**:
- Accept both integer and string
- Prioritize integer (compact), fall back to string
- Parse `capabilities` if present (policy hints from BearDog)

**Benefits**:
- ✅ Unblocks federation NOW
- ✅ Backward compatible
- ✅ Forward compatible with Phase 2
- ✅ Both integers and strings have their place

**Files to Update**:
1. BearDog: `crates/beardog-tunnel/src/unix_socket_ipc.rs` (trust evaluation response)
2. Songbird: `crates/songbird-universal/src/trust_types.rs` (parsing logic)

---

### Phase 2: Trust Policies (Genetically Secured) ⏰ **1-2 weeks**

**Goal**: Make trust policies configurable and secured by family seed

#### 2.1 Trust Policy Definition

```rust
/// Trust policy secured by genetic seed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustPolicy {
    /// Family ID this policy applies to
    pub family_id: String,
    
    /// Policy version (incremented on updates)
    pub version: u32,
    
    /// Genetic seed signature (proves policy is from family progenitor)
    pub signature: Vec<u8>,
    
    /// Trust tiers (configurable, not hardcoded)
    pub tiers: Vec<TrustTier>,
    
    /// Default tier for unknown peers
    pub default_tier_index: usize,
    
    /// Policy metadata
    pub metadata: HashMap<String, String>,
}

/// A single trust tier (replaces hardcoded levels)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustTier {
    /// Tier index (0 = lowest, higher = more trust)
    pub index: u8,
    
    /// Human-readable name
    pub name: String,
    
    /// Description
    pub description: String,
    
    /// Allowed capabilities (wildcard patterns)
    pub allowed_capabilities: Vec<String>,
    
    /// Denied capabilities (explicit denials override allows)
    pub denied_capabilities: Vec<String>,
    
    /// Requirements to achieve this tier
    pub requirements: Vec<TierRequirement>,
    
    /// How to elevate to next tier
    pub elevation_path: Option<ElevationPath>,
}

/// Requirement to achieve a trust tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TierRequirement {
    /// Same genetic family
    SameFamily,
    
    /// Known family (in whitelist)
    KnownFamily { family_ids: Vec<String> },
    
    /// Human approval required
    HumanApproval { method: String },
    
    /// Human entropy required
    HumanEntropy { min_bits: u32, method: String },
    
    /// Contact key established
    ContactKeyEstablished,
    
    /// Time-based requirement
    RelationshipAge { min_duration: std::time::Duration },
    
    /// Custom requirement (extensible)
    Custom { requirement_type: String, params: HashMap<String, String> },
}
```

#### 2.2 Policy Storage and Distribution

**Storage**:
```rust
// Each family has a signed trust policy
pub struct FamilyTrustPolicyStore {
    /// Policies indexed by family_id
    policies: HashMap<String, TrustPolicy>,
    
    /// Genetic key verifier (validates policy signatures)
    verifier: GeneticKeyVerifier,
}
```

**Distribution**:
- Trust policy embedded in UDP discovery packets (optional, signed)
- Requested via BearDog `trust.get_policy` JSON-RPC method
- Cached locally, validated with genetic signature

#### 2.3 Policy Evaluation

```rust
impl TrustPolicy {
    /// Evaluate trust for a peer
    pub fn evaluate_peer(
        &self,
        peer_id: &str,
        peer_family: Option<&str>,
        peer_evidence: &[TierEvidence],
    ) -> TrustEvaluation {
        // Find highest tier peer qualifies for
        for tier in self.tiers.iter().rev() {
            if self.peer_meets_requirements(peer_family, peer_evidence, &tier.requirements) {
                return TrustEvaluation {
                    level: tier.index,
                    level_name: tier.name.clone(),
                    confidence: 1.0,
                    reason: format!("Meets requirements for tier: {}", tier.name),
                    allowed_capabilities: tier.allowed_capabilities.clone(),
                    denied_capabilities: tier.denied_capabilities.clone(),
                    elevation_path: tier.elevation_path.clone(),
                    metadata: HashMap::new(),
                };
            }
        }
        
        // Fall back to default tier
        self.get_default_tier_evaluation()
    }
    
    fn peer_meets_requirements(
        &self,
        peer_family: Option<&str>,
        evidence: &[TierEvidence],
        requirements: &[TierRequirement],
    ) -> bool {
        requirements.iter().all(|req| match req {
            TierRequirement::SameFamily => {
                peer_family.map_or(false, |pf| pf == self.family_id)
            }
            TierRequirement::HumanApproval { .. } => {
                evidence.iter().any(|e| matches!(e, TierEvidence::HumanApproval { .. }))
            }
            TierRequirement::ContactKeyEstablished => {
                evidence.iter().any(|e| matches!(e, TierEvidence::ContactKeyExchange { .. }))
            }
            // ... other requirements
            _ => false,
        })
    }
}
```

---

### Phase 3: Contact Key Exchange (NAT/P2P) ⏰ **2-3 weeks**

**Goal**: When LAN towers connect and confirm genetic lineage, mix and share a contact key for NAT traversal and P2P

#### 3.1 Contact Key Protocol

```rust
/// Contact key exchange protocol
pub struct ContactKeyExchange {
    /// Our ephemeral DH public key
    pub our_public_key: Vec<u8>,
    
    /// Peer's ephemeral DH public key (received)
    pub peer_public_key: Option<Vec<u8>>,
    
    /// Derived shared secret (for NAT/P2P)
    pub shared_secret: Option<Vec<u8>>,
    
    /// Family lineage proof (proves we're same family or authorized)
    pub lineage_proof: GeneticLineageProof,
    
    /// Exchange timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ContactKeyExchange {
    /// Initiate contact key exchange
    pub async fn initiate(
        our_family_seed: &FamilySeed,
        peer_id: &str,
    ) -> Result<Self> {
        // Generate ephemeral DH keypair
        let (secret_key, public_key) = generate_dh_keypair();
        
        // Create lineage proof (signed with family key)
        let lineage_proof = GeneticLineageProof::create(
            our_family_seed,
            peer_id,
            &public_key,
        )?;
        
        Ok(Self {
            our_public_key: public_key,
            peer_public_key: None,
            shared_secret: None,
            lineage_proof,
            timestamp: chrono::Utc::now(),
        })
    }
    
    /// Complete exchange with peer's public key
    pub fn complete(&mut self, peer_public_key: Vec<u8>) -> Result<()> {
        // Verify peer's lineage proof (same family or authorized)
        // ... verification logic ...
        
        // Derive shared secret (ECDH or similar)
        let shared_secret = ecdh_derive(
            &self.our_secret_key,
            &peer_public_key,
        )?;
        
        self.peer_public_key = Some(peer_public_key);
        self.shared_secret = Some(shared_secret);
        
        Ok(())
    }
    
    /// Use shared secret for NAT traversal
    pub fn derive_nat_key(&self) -> Result<Vec<u8>> {
        let shared_secret = self.shared_secret.as_ref()
            .ok_or("Exchange not complete")?;
        
        // HKDF derivation for specific use case
        hkdf_derive(
            shared_secret,
            b"nat_traversal",
            32, // Key size
        )
    }
    
    /// Use shared secret for P2P encryption
    pub fn derive_p2p_key(&self) -> Result<Vec<u8>> {
        let shared_secret = self.shared_secret.as_ref()
            .ok_or("Exchange not complete")?;
        
        hkdf_derive(
            shared_secret,
            b"p2p_encryption",
            32,
        )
    }
}
```

#### 3.2 Integration with Trust Evaluation

**When LAN towers connect**:
1. UDP multicast discovery (already working)
2. Songbird queries BearDog: `trust.evaluate_peer` (already working)
3. BearDog responds: "Same family, trust level 1 (Limited)"
4. **NEW**: Songbird initiates contact key exchange:
   ```json
   {
     "method": "contact.initiate_exchange",
     "params": {
       "peer_id": "tower2",
       "our_public_key": "<ephemeral_dh_public>",
       "lineage_proof": "<signed_proof>"
     }
   }
   ```
5. Peer responds with their public key
6. Both derive shared secret
7. Shared secret stored in `TrustStore` for NAT/P2P operations
8. Trust evaluation updated: "Contact key established" → enables NAT traversal

#### 3.3 Use Cases for Contact Key

**NAT Traversal**:
- Use shared secret to authenticate NAT hole-punching
- Prevents MITM attacks during NAT setup
- Enables secure P2P even behind restrictive NATs

**P2P Encryption**:
- Derive P2P encryption keys from shared secret
- No need for centralized key distribution
- Perfect forward secrecy (ephemeral keys)

**Trust Evidence**:
- "Contact key established" becomes evidence for trust elevation
- Policy can require contact key for higher trust tiers
- Cryptographic proof of relationship

---

## 📋 Example: Configurable Trust Policy

### Default biomeOS Policy (nat0 family)

```yaml
# trust-policy-nat0.yaml
family_id: nat0
version: 1
signature: <signed_by_family_progenitor>

tiers:
  # Tier 0: No trust
  - index: 0
    name: "none"
    description: "No trust - different family or unknown"
    allowed_capabilities: []
    denied_capabilities: ["*"]
    requirements: []
    elevation_path: null
  
  # Tier 1: Same family (auto-accepted)
  - index: 1
    name: "limited"
    description: "Same genetic family - BirdSong coordination only"
    allowed_capabilities:
      - "birdsong/*"
      - "coordination/*"
      - "health"
      - "capabilities"
      - "discovery"
    denied_capabilities:
      - "data/*"
      - "commands/*"
      - "keys/*"
      - "federation/admin"
    requirements:
      - type: "SameFamily"
    elevation_path:
      next_tier: 2
      requirements: ["HumanApproval"]
      method: "user_consent_ui"
  
  # Tier 2: Human approved + contact key
  - index: 2
    name: "elevated"
    description: "Human approved - full federation with NAT/P2P"
    allowed_capabilities:
      - "birdsong/*"
      - "coordination/*"
      - "federation/*"
      - "data/read"
      - "nat_traversal"    # NEW!
      - "p2p_direct"       # NEW!
    denied_capabilities:
      - "data/write"
      - "commands/sensitive"
      - "keys/*"
    requirements:
      - type: "SameFamily"
      - type: "HumanApproval"
        method: "user_consent_ui"
      - type: "ContactKeyEstablished"  # NEW!
    elevation_path:
      next_tier: 3
      requirements: ["HumanEntropy"]
      method: "solokey_or_phone_hsm"
  
  # Tier 3: Human entropy (full trust)
  - index: 3
    name: "highest"
    description: "Human entropy - all operations including sensitive"
    allowed_capabilities: ["*"]
    denied_capabilities: []
    requirements:
      - type: "SameFamily"
      - type: "HumanApproval"
      - type: "HumanEntropy"
        min_bits: 256
        method: "solokey_or_phone_hsm"
      - type: "ContactKeyEstablished"
    elevation_path: null

default_tier_index: 0
metadata:
  created_by: "biomeOS progenitor"
  created_at: "2026-01-07T00:00:00Z"
  description: "Default trust policy for nat0 family"
```

---

## 🎯 Benefits of This Evolution

### 1. **Configurable** ✅
- Organizations define their own trust tiers
- Capabilities per tier are customizable
- Not limited to 4 hardcoded levels

### 2. **Genetically Secured** ✅
- Policies signed with family seed
- Only progenitor can create/update policy
- Cryptographically verifiable lineage

### 3. **Dynamic** ✅
- Policies can be updated (versioned)
- Trust relationships evolve over time
- Evidence-based elevation (human approval, contact keys, etc.)

### 4. **Contact Key Exchange** ✅
- Secure NAT traversal
- P2P encryption with PFS
- Cryptographic proof of relationship

### 5. **Fractal & Isomorphic** ✅
- Same policy architecture for:
  - Single node
  - LAN federation
  - WAN federation
  - HPC clusters
- Scales from 2 nodes to 1000+ nodes

---

## 🚀 Implementation Roadmap

### Week 1: Phase 1 (Unblock Federation)
- [ ] BearDog: Add dual representation (int + string) to JSON-RPC responses
- [ ] BearDog: Add capability hints to trust evaluation responses
- [ ] Songbird: Accept both int and string trust_level
- [ ] Test dual-tower local federation
- [ ] Deploy to USB spores

### Week 2-3: Phase 2 (Trust Policies)
- [ ] Define `TrustPolicy`, `TrustTier`, `TierRequirement` types
- [ ] Implement policy storage and signature verification
- [ ] BearDog: Load policy from family seed-secured file
- [ ] BearDog: Evaluate peers against configurable policy
- [ ] Songbird: Request and cache peer policies
- [ ] Create default biomeOS policy (nat0 family)
- [ ] Test with custom policies (different org requirements)

### Week 4-5: Phase 3 (Contact Key Exchange)
- [ ] Define `ContactKeyExchange` protocol
- [ ] Implement DH key exchange with lineage proofs
- [ ] BearDog: `contact.initiate_exchange` JSON-RPC method
- [ ] Songbird: Automatic contact key exchange after same-family discovery
- [ ] Store shared secrets in `TrustStore`
- [ ] Use contact keys for NAT traversal (integration point)
- [ ] Use contact keys for P2P encryption
- [ ] Test LAN federation with contact key exchange

### Week 6: Integration & Testing
- [ ] E2E tests: Policy-based federation
- [ ] E2E tests: Contact key exchange and NAT traversal
- [ ] Security audit: Policy signature verification
- [ ] Performance tests: Policy evaluation at scale
- [ ] Documentation: Trust policy guide for operators
- [ ] Documentation: Contact key exchange protocol spec

---

## 📊 Comparison: Before vs After

| Aspect | Before | After Phase 1 | After Phase 3 |
|--------|--------|---------------|---------------|
| **Trust Levels** | Hardcoded 0-3 | Dual int+string | Configurable tiers |
| **Capabilities** | Implied | Hinted | Policy-defined |
| **Genetic Security** | None | None | Signed policies |
| **Contact Keys** | None | None | Automatic exchange |
| **NAT/P2P** | Manual | Manual | Cryptographically secure |
| **Extensibility** | Low | Medium | High |
| **Federation** | ❌ Blocked | ✅ Working | ✅ Secure & Scalable |

---

## 🎊 Success Criteria

**Phase 1** (Week 1):
- ✅ Dual-tower federation working
- ✅ Both integer and string trust levels accepted
- ✅ Capability hints present in responses
- ✅ No breaking changes

**Phase 2** (Week 3):
- ✅ Custom trust policies loadable
- ✅ Policies signed and verified
- ✅ Configurable trust tiers working
- ✅ Two orgs with different policies can federate

**Phase 3** (Week 5):
- ✅ Contact keys automatically exchanged
- ✅ NAT traversal using shared secrets
- ✅ P2P encryption with PFS
- ✅ Trust evidence includes contact key status

---

## 💡 Key Insights

### User's Vision ✨
"Different orgs, federations, people, and whatnot will have different trust priorities and permissions. So we need to evolve to have this configurable and secured by a genetic seed. So that if it's updated as we go, for example when LAN towers connect, we can confirm the connection, and they mix and share a contact key for NAT/P2P."

### Architectural Principles
1. **Don't patch, evolve**: Fix the schema AND design for the future
2. **Both integers and strings have use cases**: Integers for compact representation, strings for readability
3. **Trust is a relationship, not a number**: Policies, evidence, cryptographic proof
4. **Genetic lineage is the root of trust**: All policies secured by family seed
5. **Contact keys enable secure P2P**: Cryptographic proof of relationship for NAT/P2P

---

## 📚 Related Documents

- `FEDERATION_BLOCKED_ROOT_CAUSE_ANALYSIS.md` - Current schema mismatch analysis
- `DUAL_TOWER_TEST_RESULTS.md` - Federation test results
- `PROTOCOL_MISMATCH_DEEP_DEBT.md` - Protocol evolution history
- `SONGBIRD_BIOMEOS_NEURALAPI_SYNERGY.md` - NeuralAPI integration vision

---

**Version**: v1.0  
**Date**: January 7, 2026  
**Status**: 🎯 **ARCHITECTURAL VISION - READY FOR IMPLEMENTATION**

---

*"Trust is not a number, it's a relationship secured by shared genetic lineage and cryptographic proof."* 🔐

