# Genetic Lineage Evolution Spec

**Date**: February 5, 2026  
**Status**: ✅ IMPLEMENTED - Evolved Genetic Model  
**Version**: 3.0.0

---

## Executive Summary

The genetic seed model has been evolved from a flawed "cloned family seed" pattern to a proper **Mitochondrial + Nuclear DNA** architecture:

| Seed Type | Role | Shared? | Syncable? |
|-----------|------|---------|-----------|
| **Beacon Seed** (Mitochondrial) | Dark Forest encryption, family recognition | ✅ Shared | ✅ Yes |
| **Lineage Seed** (Nuclear DNA) | Device identity, ancestry proof | ❌ Unique | ❌ Never |

---

## Evolved Architecture

### Biological Analogy

```
MITOCHONDRIAL DNA (Beacon Seed)
├── Inherited from "mother" (genesis device)
├── Shared across all family members
├── Contains: encryption key, address book
├── CAN be synced, transferred, evolved
└── Enables: family recognition in Dark Forest

NUCLEAR DNA (Lineage Seed)  
├── Mixed at conception (device enrollment)
├── Unique to each individual device
├── Derived from: family_seed + device_entropy
├── NEVER copied, always derived fresh
└── Enables: individual authentication, ancestry proof
```

### Seed Files

```
.beacon.seed      (32 bytes) - Mitochondrial - SHARED
.lineage.seed     (32 bytes) - Nuclear DNA - UNIQUE per device
.family.seed      DEPRECATED - replaced by .beacon.seed
.known_beacons.json          - Address book (syncable)
```

---

## Current Implementation Status

### Validated Configuration (Feb 5, 2026)

```
TOWER (x86_64):
├── .beacon.seed  = 8ff3b864a4bc589a... (SHARED)
├── .lineage.seed = 5772c07f24654deb... (UNIQUE)
└── .known_beacons.json (knows pixel8a)

PIXEL (aarch64):
├── .beacon.seed  = 8ff3b864a4bc589a... (SHARED - same as Tower!)
├── .lineage.seed = 3795d0cac4fb6576... (UNIQUE - different from Tower!)
└── .known_beacons.json (knows tower)
```

### Cross-Device Verification

```bash
# Tower → Pixel beacon exchange
Tower generates beacon → Pixel decrypts → is_family: true ✅
Pixel generates beacon → Tower decrypts → is_family: true ✅
```

---

## Key Concepts

### 1. Mitochondrial Beacon (Shared)

The beacon seed is like mitochondrial DNA:
- **Inherited** from the genesis device
- **Shared** across all family members
- **Enables** Dark Forest encrypted beacons
- **CAN evolve** as address book grows

```rust
// Beacon encryption uses shared mito seed
let beacon = BearDog::birdsong_encrypt(
    payload,
    mito_beacon_seed,  // Shared across family
    node_id,
);

// Any family member can decrypt
let decrypted = BearDog::birdsong_decrypt(
    beacon,
    mito_beacon_seed,  // Same shared seed
);
// decrypted.is_family == true
```

### 2. Nuclear Lineage (Unique)

The lineage seed is like nuclear DNA:
- **Derived** from family seed + device-specific entropy
- **Unique** to each device instance
- **Proves** individual ancestry
- **NEVER** copied or transferred

```rust
// Derivation (at device enrollment)
let lineage_seed = HKDF::derive(
    family_seed,           // Shared root
    device_entropy,        // Hardware UUID, enrollment time
    b"ecoPrimals-device-lineage-v1",
);

// Each device gets unique lineage
assert!(tower.lineage_seed != pixel.lineage_seed);
```

### 3. Address Book (Syncable)

The `.known_beacons.json` is the evolving "social network":
- **Contains** known family members and friends
- **Syncable** across family devices
- **Evolves** as new connections are made

```json
{
  "mito_beacon_id": "8ff3b864...",
  "this_node": {
    "node_id": "pixel8a",
    "lineage_id": "3795d0ca..."
  },
  "family_members": {
    "tower": {
      "lineage_id": "5772c07f...",
      "capabilities": ["ai-server", "gpu"]
    }
  },
  "friends": {
    "alice": {
      "beacon_id": "alice_beacon...",
      "introduced_by": "tower"
    }
  }
}
```

---

## Sync Model

### Edge Device Meets New Friend

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│  Pixel (mobile edge)                                            │
│  ├── beacon.seed (mito) - shared family encryption              │
│  ├── lineage.seed (unique to Pixel)                             │
│  │                                                              │
│  │  [Pixel meets "alice" at coffee shop]                        │
│  │                                                              │
│  └── address_book.friends.add("alice", alice_beacon)            │
│            │                                                    │
│            │ SYNC (via mesh, p2p, or sneakernet)                │
│            ▼                                                    │
│  Basement HPC                                                   │
│  ├── beacon.seed (mito) - same family encryption!               │
│  ├── lineage.seed (unique to HPC)                               │
│  │                                                              │
│  └── address_book now includes alice!                           │
│      HPC can decrypt alice's beacons                            │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Mitochondrial Evolution

The beacon seed (mito) can evolve over time:
1. **Add friends** - new beacons get added to address book
2. **Mix with friend's mito** - create shared encryption with non-family
3. **Propagate** - sync changes across family devices

---

## Security Properties

### Compromise Scenarios

| Scenario | Impact |
|----------|--------|
| Beacon seed leaks | Family can regenerate, friends notified |
| Lineage seed leaks | Only that device compromised, others safe |
| Address book leaks | Social graph visible, but not keys |

### Trust Escalation

```
ANONYMOUS → BEACON_KNOWN → LINEAGE_VERIFIED → FULL_TRUST
    │              │                │              │
    │              │                │              └─ bidirectional lineage proof
    │              │                └─ lineage signature verified
    │              └─ can decrypt their beacons
    └─ no relationship
```

---

## Implementation Files

### biomeOS Core
- `crates/biomeos-spore/src/beacon_genetics/mod.rs` - Manager
- `crates/biomeos-spore/src/beacon_genetics/derivation.rs` - LineageDeriver
- `crates/biomeos-spore/src/beacon_genetics/capability.rs` - DirectBeardogCaller

### Seed Files
- `livespore-usb/.beacon.seed` - Shared mitochondrial
- `livespore-usb/.lineage.seed` - Unique nuclear
- `livespore-usb/.known_beacons.json` - Address book

### Primal Support
- **BearDog**: `birdsong.encrypt`, `birdsong.decrypt`, `genetic.derive_lineage_key`
- **Songbird**: `birdsong.generate_encrypted_beacon`, `birdsong.decrypt_beacon`

---

## Migration from Old Model

### Deprecated Files
- `.family.seed` → Use `.beacon.seed` instead
- Old address book format → v3.0.0 schema

### Migration Steps
1. Copy `.family.seed` to `.beacon.seed` (one time)
2. Run enrollment to derive unique `.lineage.seed`
3. Update `.known_beacons.json` to v3.0.0 schema
4. Mark `.family.seed` as deprecated

---

## Future Evolution

### Phase 2: Lineage Certificates
- Sign lineage with parent's key
- Chain of custody verification
- Revocation support

### Phase 3: Merkle Lineage Tree
- Full ancestry proof
- Efficient verification
- DAG-based social graph

### Phase 4: Hardware Binding
- TPM-backed lineage seeds
- Secure enclave storage
- Hardware attestation

---

**Author**: biomeOS Team  
**License**: AGPL-3.0-only
