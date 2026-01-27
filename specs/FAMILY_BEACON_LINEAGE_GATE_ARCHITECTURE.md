# Family Beacon + Lineage Gate Architecture

**Version**: 1.0.0  
**Date**: January 27, 2026  
**Status**: SPECIFICATION  
**Author**: biomeOS Team

---

## Overview

This specification defines a two-layer trust architecture:

1. **Family Beacon** (Public) - Family tag advertised for discovery
2. **Lineage Gate** (Private) - Cryptographic lineage verification before any real communication

This separation enables:
- **Fast discovery** via lightweight family tags
- **Strong security** via cryptographic lineage proof
- **Future enclave compute** via session-derived keys

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          DISCOVERY LAYER (Public)                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐    mDNS/UDP Multicast    ┌──────────────┐                │
│  │   Spore A    │  ────────────────────►   │   Spore B    │                │
│  │  family=nat0 │  ◄────────────────────   │  family=nat0 │                │
│  └──────────────┘   "I'm in family nat0"   └──────────────┘                │
│                                                                             │
│  TXT Record: family_id=nat0, node_id=alpha, capabilities=[...]             │
│                                                                             │
│  ⚠️  ANYONE CAN CLAIM family=nat0 - This is just a HINT!                   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ Peer discovered with matching family_id
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         LINEAGE GATE (Semi-Private)                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐                              ┌──────────────┐            │
│  │   Spore A    │                              │   Spore B    │            │
│  │  BearDog A   │                              │  BearDog B   │            │
│  └──────────────┘                              └──────────────┘            │
│         │                                              │                    │
│         │  1. Generate challenge (32-byte nonce)       │                    │
│         ├─────────────────────────────────────────────►│                    │
│         │                                              │                    │
│         │  2. Generate lineage proof + response        │                    │
│         │     response = HMAC(nonce, lineage_key)      │                    │
│         │◄─────────────────────────────────────────────┤                    │
│         │                                              │                    │
│         │  3. Verify via BearDog:                      │                    │
│         │     - Check proof against parent seed        │                    │
│         │     - Verify HMAC with expected sibling key  │                    │
│         │                                              │                    │
│                                                                             │
│  ✅ Only TRUE siblings (derived from same parent) can pass this gate!      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    │ Lineage verified - establish session
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        SESSION LAYER (Private)                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐      Encrypted Channel     ┌──────────────┐              │
│  │   Spore A    │  ════════════════════════  │   Spore B    │              │
│  │ session_key  │                            │ session_key  │              │
│  └──────────────┘                            └──────────────┘              │
│                                                                             │
│  Session key derived from lineage:                                          │
│    session_key = HKDF(shared_lineage_secret, nonce_a || nonce_b)           │
│                                                                             │
│  All further communication encrypted with session_key:                      │
│    - JSON-RPC calls                                                         │
│    - Capability routing                                                     │
│    - Enclave compute commands                                               │
│                                                                             │
│  🔒 ENCLAVE READY: Session keys enable secure multi-party compute!         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Protocol Specification

### Phase 1: Discovery (Family Beacon)

**Purpose**: Announce presence and capability for quick discovery

**Transport**: UDP Multicast (mDNS) or DNS-SD

**Advertised Data**:
```
Service: _biome._tcp.local
TXT Records:
  family_id=nat0
  node_id=node-alpha
  capabilities=crypto,http,discovery
  lineage_mode=sibling
  socket=/tmp/beardog-nat0-node-alpha.sock
```

**Security Model**:
- Family tag is **NOT authenticated** at this layer
- Anyone can claim any family_id
- This is intentional - discovery should be fast
- Authentication happens in Phase 2

### Phase 2: Lineage Gate (Challenge-Response)

**Purpose**: Cryptographically verify genetic relationship

**Transport**: Unix socket (JSON-RPC 2.0)

**Protocol**:

```
Challenger (A)                          Responder (B)
    │                                        │
    │  1. genetic.challenge                  │
    │  {                                     │
    │    "challenger_node_id": "node-alpha", │
    │    "target_family_id": "nat0",         │
    │    "nonce": "32-bytes-hex",            │
    │    "challenge_type": "sibling"         │
    │  }                                     │
    ├───────────────────────────────────────►│
    │                                        │
    │                           BearDog B reads .family.seed
    │                           Derives lineage_key via HKDF
    │                           Computes response = HMAC(nonce, lineage_key)
    │                           Generates lineage proof
    │                                        │
    │  2. genetic.challenge_response         │
    │  {                                     │
    │    "responder_node_id": "node-beta",   │
    │    "response": "64-bytes-hex",         │
    │    "lineage_proof": "base64-proof",    │
    │    "seed_hash_prefix": "16-bytes-hex"  │
    │  }                                     │
    │◄───────────────────────────────────────┤
    │                                        │
    │  3. Verification (via BearDog A)       │
    │                                        │
    │  genetic.verify_challenge_response     │
    │  {                                     │
    │    "nonce": "32-bytes-hex",            │
    │    "response": "64-bytes-hex",         │
    │    "responder_node_id": "node-beta",   │
    │    "lineage_proof": "base64-proof",    │
    │    "parent_seed_path": "optional"      │
    │  }                                     │
    │                                        │
```

**Verification Logic** (BearDog):

```rust
fn verify_challenge_response(
    nonce: &[u8; 32],
    response: &[u8; 64],
    responder_node_id: &str,
    lineage_proof: &str,
    parent_seed_path: Option<&Path>,
) -> Result<LineageVerification> {
    // Case 1: We have the parent seed (can derive expected sibling key)
    if let Some(parent_path) = parent_seed_path {
        let parent_seed = read_seed(parent_path)?;
        
        // Derive expected sibling seed
        let expected_sibling_seed = sha256(parent_seed || responder_node_id);
        
        // Derive expected lineage key
        let expected_key = hkdf(expected_sibling_seed, "lineage-verification");
        
        // Verify HMAC
        let expected_response = hmac_sha512(nonce, expected_key);
        
        if constant_time_eq(response, expected_response) {
            return Ok(LineageVerification {
                valid: true,
                relationship: "verified_sibling",
                trust_level: "family",
            });
        }
    }
    
    // Case 2: We are siblings (share same parent, different derivation)
    // Use lineage proof to verify shared ancestry
    let our_proof = generate_lineage_proof(our_seed)?;
    if verify_shared_ancestry(our_proof, lineage_proof) {
        return Ok(LineageVerification {
            valid: true,
            relationship: "sibling",
            trust_level: "family",
        });
    }
    
    // Case 3: Different family - reject
    Ok(LineageVerification {
        valid: false,
        relationship: "unrelated",
        trust_level: "none",
    })
}
```

### Phase 3: Session Establishment

**Purpose**: Derive session keys for encrypted communication

**Protocol**:

After successful lineage verification, both parties derive a shared session key:

```
session_key = HKDF(
    ikm = lineage_shared_secret,          // Derived from parent seed
    salt = nonce_challenger || nonce_responder,
    info = "biome-session-v1",
    length = 32
)
```

**Session Capabilities**:

| Capability | Description |
|------------|-------------|
| `enclave.execute` | Run compute in secure enclave |
| `enclave.attest` | Get attestation of enclave state |
| `data.share` | Share encrypted data within family |
| `data.seal` | Seal data to specific lineage |

---

## Implementation Plan

### Phase 1: BearDog Methods (Immediate)

Add to `beardog-tunnel/src/unix_socket_ipc/handlers/`:

```rust
// New methods for lineage gate
"genetic.generate_challenge"         // Generate 32-byte nonce
"genetic.respond_to_challenge"       // Compute HMAC response
"genetic.verify_challenge_response"  // Verify response + proof
"genetic.derive_session_key"         // Derive session key from lineage
```

### Phase 2: Songbird Integration (Next)

Modify `discovery_bridge.rs`:

```rust
// After same-family detection (line 176)
let same_family = check_family_tag(&peer.tags);

// NEW: If same family claimed, require lineage gate
if same_family {
    let verified = lineage_gate_challenge_response(
        &peer.socket,
        &peer.node_id,
        &security_client
    ).await?;
    
    if !verified {
        warn!("❌ Peer {} claimed family but failed lineage gate", peer.node_id);
        continue; // Reject - don't add to federation
    }
    
    info!("✅ Peer {} passed lineage gate - TRUE family member", peer.node_id);
}
```

### Phase 3: Session Encryption (Future)

Wrap all JSON-RPC communication in session-encrypted envelope:

```rust
struct EncryptedMessage {
    session_id: String,
    nonce: [u8; 12],        // ChaCha20-Poly1305 nonce
    ciphertext: Vec<u8>,    // Encrypted JSON-RPC
    tag: [u8; 16],          // Authentication tag
}
```

---

## Security Analysis

### Attack Vectors Mitigated

| Attack | Current System | With Lineage Gate |
|--------|---------------|-------------------|
| **Family Tag Spoofing** | ❌ Vulnerable | ✅ Rejected at gate |
| **Replay Attack** | ⚠️ Possible | ✅ Nonce prevents replay |
| **Man-in-the-Middle** | ⚠️ Possible | ✅ Session keys prevent |
| **Unauthorized Federation** | ⚠️ Tag check only | ✅ Crypto verification |

### Trust Hierarchy

```
Trust Level 0: Unknown (no family tag match)
    └─ No federation possible
    
Trust Level 1: Family Claimed (tag match, no lineage verification)
    └─ Initiate lineage gate protocol
    └─ DO NOT allow any capability calls yet
    
Trust Level 2: Lineage Verified (passed challenge-response)
    └─ Full federation access
    └─ Session key established
    └─ Enclave compute allowed
    
Trust Level 3: Parent-Child (parent seed available)
    └─ Highest trust
    └─ Can verify exact derivation path
    └─ Can derive shared secrets deterministically
```

---

## Enclave Compute Integration

Once session keys are established, the family can perform secure multi-party computation:

### Use Cases

1. **Distributed Key Generation**
   - Family members contribute entropy
   - No single member sees full key
   - Lineage ensures only family can participate

2. **Secure Data Aggregation**
   - Each member encrypts data with session key
   - Aggregator computes without seeing plaintext
   - Results revealed only to family

3. **Attestation Chain**
   - Each member attests to their enclave state
   - Chain of attestations proves family integrity
   - Lineage ties attestation to genetic identity

### Future: TEE Integration

```rust
// Enclave execution within verified lineage
let result = enclave.execute_in_family(
    lineage_session,
    |secure_context| {
        // Code runs in TEE
        // Only family members can participate
        // Results encrypted with family session key
    }
).await?;
```

---

## Migration Path

### Current System
```
Discovery → Trust Evaluation (tag check) → Federation
```

### Target System
```
Discovery → Lineage Gate (crypto) → Session → Federation
            │                       │
            └─ Reject if not family └─ Encrypted channel
```

### Backward Compatibility

- Systems without lineage gate continue to work (development mode)
- `REQUIRE_LINEAGE_GATE=true` enables strict mode
- Gradual rollout: warn → soft-fail → hard-fail

---

## References

- `specs/PRIMAL_IPC_PROTOCOL.md` - JSON-RPC over Unix sockets
- `wateringHole/PRIMAL_IPC_PROTOCOL.md` - Ecosystem standard
- `crates/biomeos-spore/src/seed.rs` - Seed derivation
- `beardog-genetics/src/birdsong/lineage_proof.rs` - Lineage proof generation

---

**Status**: SPECIFICATION - Ready for implementation

*Family tag gets you in the door. Lineage gate proves you belong.*

