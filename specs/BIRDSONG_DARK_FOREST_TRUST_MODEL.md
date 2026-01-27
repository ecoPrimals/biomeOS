# BirdSong Dark Forest Trust Model

**Version**: 1.0.0  
**Date**: January 27, 2026  
**Status**: ARCHITECTURAL SPECIFICATION  
**Principle**: *Hear everything, reveal nothing. Trust through lineage, not payment.*

---

## Core Philosophy

### The Dark Forest

In the dark forest, **everyone broadcasts**, but broadcasts reveal **nothing useful** to outsiders:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           THE DARK FOREST                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   🔊 Spore A broadcasts: [encrypted_beacon]                                │
│   🔊 Spore B broadcasts: [encrypted_beacon]                                │
│   🔊 Spore C broadcasts: [encrypted_beacon]                                │
│   🔊 Attacker broadcasts: [encrypted_beacon]                               │
│                                                                             │
│   Everyone can HEAR all broadcasts.                                         │
│   But hearing reveals NOTHING.                                              │
│                                                                             │
│   Only those who can DECRYPT know:                                          │
│     - Who is family                                                         │
│     - What capabilities exist                                               │
│     - Where to connect                                                      │
│                                                                             │
│   Attackers hear noise. Family hears signal.                               │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Key Difference from Previous Model

| Aspect | Previous (Beacon+Gate) | Dark Forest |
|--------|----------------------|-------------|
| **Discovery** | `family_id=nat0` (plaintext) | `[encrypted_blob]` (opaque) |
| **Reveals** | Family membership hint | Nothing |
| **Attack surface** | Know who to target | Can't identify targets |
| **Validation** | After discovery | Before understanding discovery |

---

## Architecture

### Layer 1: Encrypted Beacons (Hear Nothing)

**Beacon Structure**:
```
beacon = ENCRYPT(
    key = family_broadcast_key,        // Derived from family seed
    plaintext = {
        "family_hash": hash(family_id),  // Not the actual ID
        "node_nonce": random_32_bytes,
        "timestamp": unix_epoch,
        "capabilities_hash": hash(caps),
        "socket_hint": encrypted_socket_path
    }
)
```

**What Others See**:
```
[32 bytes of noise][16 byte tag]
```

**What Family Sees** (after decryption):
```json
{
    "family_hash": "a1b2c3...",
    "node_nonce": "d4e5f6...",
    "timestamp": 1706400000,
    "capabilities_hash": "789abc...",
    "socket_hint": "/tmp/beardog-nat0-node-alpha.sock"
}
```

### Layer 2: Challenge-Before-Reveal

Even after decrypting a beacon, don't assume family. **Challenge first**:

```
Family Member A                        Potential Family B
      │                                       │
      │  1. Decrypts B's beacon               │
      │     (knows B might be family)         │
      │                                       │
      │  2. Sends encrypted challenge         │
      │     (encrypted with family key)       │
      ├──────────────────────────────────────►│
      │                                       │
      │     B decrypts challenge              │
      │     (if B is family, can decrypt)     │
      │     B generates lineage proof         │
      │                                       │
      │  3. Returns encrypted response        │
      │◄──────────────────────────────────────┤
      │                                       │
      │  4. Verifies lineage proof            │
      │     NOW trust is established          │
      │                                       │
```

**Key Insight**: The challenge itself is encrypted. Non-family can't even understand the challenge, let alone respond correctly.

### Layer 3: Lineage Relay (Grandma's Phonebook)

**Scenario**: You want to contact your cousin. You don't have their address.

**Traditional Model** (Paid Relay):
```
You → Pay $$ → Relay Service → Cousin
       ↑
       External trust anchor
```

**Dark Forest Model** (Lineage Relay):
```
You → Grandma → Cousin
       │          │
       │          └─ Cousin independently verifies YOU
       │             (doesn't just trust grandma's word)
       │
       └─ Grandma vouches for introduction
          (encrypted, only family can read)
```

**How It Works**:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         LINEAGE RELAY PROTOCOL                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  You (node-alpha)          Grandma (node-gamma)         Cousin (node-delta) │
│        │                          │                            │            │
│        │  1. Request intro        │                            │            │
│        │  "I need delta's addr"   │                            │            │
│        ├─────────────────────────►│                            │            │
│        │                          │                            │            │
│        │                          │  2. Grandma sends YOU to   │            │
│        │                          │     Cousin (encrypted)     │            │
│        │                          ├───────────────────────────►│            │
│        │                          │                            │            │
│        │  3. Grandma returns      │                            │            │
│        │     Cousin's beacon      │                            │            │
│        │◄─────────────────────────┤                            │            │
│        │                          │                            │            │
│        │  4. You contact Cousin directly                       │            │
│        │     (using decrypted beacon)                          │            │
│        ├──────────────────────────────────────────────────────►│            │
│        │                                                       │            │
│        │  5. Cousin INDEPENDENTLY verifies YOU                 │            │
│        │     (doesn't just trust grandma's intro)              │            │
│        │     Uses lineage challenge-response                   │            │
│        │◄──────────────────────────────────────────────────────┤            │
│        │                                                       │            │
│                                                                             │
│  ✅ Trust established: You ↔ Cousin                                        │
│  ✅ Grandma was relay but NOT trust anchor                                 │
│  ✅ Attacker intercepting grandma's relay learns NOTHING                   │
│     (all messages encrypted with family key)                               │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Layer 4: Physical Anchor Entry (SoloKey + LiveSpore)

**Scenario**: You have a SoloKey hardware token and a LiveSpore USB. You want to connect to your family's distributed network.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                      PHYSICAL ANCHOR ENTRY                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                  │
│  │   SoloKey    │    │  LiveSpore   │    │  Known HPC   │                  │
│  │  (hardware)  │    │   (USB)      │    │  (entry)     │                  │
│  └──────┬───────┘    └──────┬───────┘    └──────┬───────┘                  │
│         │                   │                   │                          │
│         │  1. Hardware auth │                   │                          │
│         │  (prove physical  │                   │                          │
│         │   possession)     │                   │                          │
│         ├──────────────────►│                   │                          │
│         │                   │                   │                          │
│         │                   │  2. LiveSpore     │                          │
│         │                   │     has family    │                          │
│         │                   │     seed          │                          │
│         │                   │                   │                          │
│         │                   │  3. Connect to    │                          │
│         │                   │     KNOWN entry   │                          │
│         │                   │     point (HPC)   │                          │
│         │                   ├──────────────────►│                          │
│         │                   │                   │                          │
│         │                   │  4. Entry point   │                          │
│         │                   │     validates     │                          │
│         │                   │     lineage       │                          │
│         │                   │◄──────────────────┤                          │
│         │                   │                   │                          │
│         │                   │  5. Entry point   │                          │
│         │                   │     provides      │                          │
│         │                   │     family map    │                          │
│         │                   │     (encrypted)   │                          │
│         │                   │◄──────────────────┤                          │
│         │                   │                   │                          │
│                                                                             │
│  Now LiveSpore knows:                                                       │
│    - All family members (encrypted beacons)                                 │
│    - Network topology                                                       │
│    - Available capabilities                                                 │
│                                                                             │
│  BUT: Each connection still requires independent lineage validation!        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Layer 5: Every Server is a Relay

**The Family Network IS the Relay Network**:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     FAMILY AS RELAY INFRASTRUCTURE                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Traditional:                                                               │
│    You → [Paid CDN] → [Paid DNS] → [Paid Relay] → Destination              │
│           $$$           $$$           $$$                                   │
│                                                                             │
│  Dark Forest:                                                               │
│    You → [Family Node] → [Family Node] → Destination                        │
│           lineage         lineage                                           │
│                                                                             │
│  Every family member:                                                       │
│    ✅ Can route traffic for other family members                           │
│    ✅ Validates lineage at each hop                                        │
│    ✅ Encrypted end-to-end (relay sees nothing useful)                     │
│    ✅ No payment required - family helps family                            │
│                                                                             │
│  Route Selection:                                                           │
│    - Prefer geographically close family                                     │
│    - Prefer low-latency paths                                               │
│    - Avoid overloaded nodes                                                 │
│    - All routes equally trusted (lineage validated)                        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Protocol Details

### Encrypted Beacon Generation

```rust
fn generate_encrypted_beacon(family_seed: &[u8; 32], node_id: &str) -> Vec<u8> {
    // Derive broadcast key from family seed
    let broadcast_key = hkdf_sha256(
        family_seed,
        b"birdsong-broadcast-v1",
        32
    );
    
    // Create plaintext beacon
    let beacon = BeaconPlaintext {
        family_hash: blake3::hash(family_seed)[..16].to_vec(),
        node_nonce: random_32_bytes(),
        timestamp: unix_timestamp(),
        capabilities_hash: hash_capabilities(&my_capabilities),
        socket_hint: encrypt_socket_path(&my_socket, family_seed),
    };
    
    // Encrypt with ChaCha20-Poly1305
    let nonce = random_12_bytes();
    let ciphertext = chacha20_poly1305_encrypt(
        &broadcast_key,
        &nonce,
        &serialize(beacon),
        b"birdsong-beacon"  // AAD
    );
    
    // Return: nonce || ciphertext || tag
    [nonce, ciphertext].concat()
}
```

### Beacon Decryption (Family Only)

```rust
fn try_decrypt_beacon(encrypted: &[u8], family_seed: &[u8; 32]) -> Option<BeaconPlaintext> {
    // Derive same broadcast key
    let broadcast_key = hkdf_sha256(
        family_seed,
        b"birdsong-broadcast-v1",
        32
    );
    
    // Extract nonce (first 12 bytes)
    let nonce = &encrypted[..12];
    let ciphertext = &encrypted[12..];
    
    // Try to decrypt
    let plaintext = chacha20_poly1305_decrypt(
        &broadcast_key,
        nonce,
        ciphertext,
        b"birdsong-beacon"
    ).ok()?;
    
    // Verify family hash matches
    let beacon: BeaconPlaintext = deserialize(&plaintext).ok()?;
    let expected_hash = blake3::hash(family_seed)[..16].to_vec();
    
    if beacon.family_hash != expected_hash {
        return None;  // Wrong family
    }
    
    Some(beacon)
}
```

### Lineage Relay Introduction

```rust
// Grandma introduces You to Cousin
async fn relay_introduction(
    requester: &NodeId,       // You
    target: &NodeId,          // Cousin
    family_seed: &[u8; 32],
) -> Result<EncryptedIntroduction> {
    // Verify requester is family (already done via connection)
    
    // Get target's current beacon
    let target_beacon = self.get_beacon(target).await?;
    
    // Create introduction packet
    let intro = Introduction {
        from_node: requester.clone(),
        to_node: target.clone(),
        introducer: self.node_id.clone(),
        target_beacon: target_beacon,
        timestamp: unix_timestamp(),
        // Proof that introducer knows both parties
        introducer_proof: generate_introduction_proof(
            family_seed,
            requester,
            target,
        ),
    };
    
    // Encrypt for requester
    let encrypted = encrypt_for_node(
        &intro,
        requester,
        family_seed,
    );
    
    // Also notify target that requester may contact them
    notify_incoming_introduction(target, requester, family_seed).await?;
    
    Ok(encrypted)
}

// Cousin validates You (independent of grandma's intro)
async fn validate_introduced_peer(
    introducer_claim: &Introduction,
    peer_challenge: &[u8],
    family_seed: &[u8; 32],
) -> Result<bool> {
    // DON'T just trust grandma's word
    // Independently verify the peer's lineage
    
    let proof = request_lineage_proof(&introducer_claim.from_node).await?;
    
    verify_lineage_proof(
        proof,
        family_seed,
        &introducer_claim.from_node,
    )
}
```

---

## Security Properties

### What Attackers Can Do

| Attack | Result |
|--------|--------|
| Listen to beacons | Hear encrypted noise |
| Replay beacons | Timestamp prevents replay |
| Impersonate family | Can't decrypt challenges |
| Compromise relay | See encrypted traffic only |
| MITM introduction | Independent validation catches |

### What Attackers CANNOT Do

1. **Identify family members** - Beacons are encrypted
2. **Understand traffic** - All comms encrypted with family key
3. **Inject into network** - Can't pass lineage validation
4. **Use compromised relay** - E2E encryption, relay is blind
5. **Social engineer via intro** - Independent validation required

### Trust Hierarchy

```
Trust Level 0: Non-family (default)
    └─ Can hear beacons but can't decrypt
    └─ Exists in dark forest, harmless
    
Trust Level 1: Can decrypt beacon
    └─ Has family broadcast key
    └─ NOT yet validated - could be stolen key
    
Trust Level 2: Passed lineage challenge
    └─ Independently verified genetic relationship
    └─ Full family member
    └─ Can relay for other family
    
Trust Level 3: Physical anchor + lineage
    └─ Hardware authentication (SoloKey)
    └─ Lineage verification
    └─ Highest trust - can be network entry point
```

---

## Implementation Roadmap

### Phase 1: Encrypted Beacons
- [ ] Add `beacon.generate_encrypted` to BearDog
- [ ] Add `beacon.try_decrypt` to BearDog
- [ ] Update Songbird discovery to use encrypted beacons
- [ ] Broadcast encrypted beacons via mDNS

### Phase 2: Challenge-Before-Reveal
- [ ] Encrypted challenge generation
- [ ] Challenge must be decrypted before response
- [ ] Reject connections that can't decrypt challenge

### Phase 3: Lineage Relay
- [ ] Add `relay.introduce` method
- [ ] Add `relay.forward` for encrypted routing
- [ ] Independent validation after introduction
- [ ] Family routing table

### Phase 4: Physical Anchor
- [ ] SoloKey integration for hardware auth
- [ ] Entry point discovery protocol
- [ ] Family map distribution (encrypted)

### Phase 5: Full Mesh Relay
- [ ] Every node can relay
- [ ] Route optimization
- [ ] Bandwidth sharing
- [ ] No paid relay infrastructure needed

---

## Comparison to Traditional Models

| Aspect | Traditional Internet | Dark Forest BirdSong |
|--------|---------------------|---------------------|
| Discovery | DNS (public) | Encrypted beacons |
| Routing | Paid CDN/relay | Family relay (free) |
| Trust | Certificates (bought) | Lineage (genetic) |
| Privacy | IP visible | All traffic encrypted |
| MITM | Certificate pinning | Lineage validation |
| Entry | ISP → DNS → Server | SoloKey → Entry → Family |
| Cost | Pay per hop | Free (family helps family) |

---

## Summary

**Hear everything, reveal nothing.**

1. **Beacons are encrypted** - Outsiders hear noise
2. **Challenge before reveal** - Decrypt to even understand the question
3. **Lineage relay** - Family introduces family, but verification is independent
4. **Physical anchors** - Hardware + seed = entry point
5. **Family IS infrastructure** - No paid relays, family routes for family

*The dark forest protects its own. You can hear us, but you'll never find us.*

