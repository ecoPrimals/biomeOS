# Beacon Capability Translations
## Semantic → BearDog Method Mapping

**Version**: 1.0.0  
**Date**: February 4, 2026  
**Status**: SPECIFICATION

---

## Architectural Principle

```
biomeOS = Ecosystem orchestration (manages meetings, address books, clusters)
BearDog = Crypto primitives (encrypt, decrypt, sign, verify, derive)

biomeOS calls capability.call("beacon.encrypt", params)
    → CapabilityTranslationRegistry translates to:
        method: "beacon.encrypt" or "chacha20_poly1305_encrypt"
        socket: "/run/user/.../beardog-{family}.sock"
    → BearDog executes crypto primitive
    → Result flows back to biomeOS for orchestration
```

---

## Beacon Capability Translations

### Core Beacon Primitives (BearDog)

| Semantic Capability | BearDog Method | Parameters | Returns |
|---------------------|----------------|------------|---------|
| `beacon.generate` | `beacon.generate` | `{}` | `{beacon_id, seed_hex}` |
| `beacon.get_id` | `beacon.get_id` | `{}` | `{beacon_id}` |
| `beacon.get_seed` | `beacon.get_seed` | `{}` | `{seed_hex}` |
| `beacon.encrypt` | `beacon.encrypt` | `{plaintext_b64, seed_hex?}` | `{ciphertext_b64, nonce_b64}` |
| `beacon.decrypt` | `beacon.decrypt` | `{ciphertext_b64, nonce_b64, seed_hex?}` | `{plaintext_b64}` |
| `beacon.try_decrypt` | `beacon.try_decrypt` | `{ciphertext_b64, seed_hex}` | `{decrypted: bool, payload?}` |

**Note**: If `seed_hex` is not provided, BearDog uses its own beacon seed.

### Crypto Primitives (BearDog)

| Semantic Capability | BearDog Method | Parameters | Returns |
|---------------------|----------------|------------|---------|
| `crypto.encrypt` | `chacha20_poly1305_encrypt` | `{plaintext, context}` | `{ciphertext}` |
| `crypto.decrypt` | `chacha20_poly1305_decrypt` | `{ciphertext, context}` | `{plaintext}` |
| `crypto.encrypt_with_lineage` | `encrypt_with_lineage_key` | `{plaintext, context}` | `{ciphertext}` |
| `crypto.decrypt_with_lineage` | `decrypt_with_lineage_key` | `{ciphertext, context}` | `{plaintext}` |
| `crypto.blake3_hash` | `blake3_hash` | `{data}` | `{hash}` |
| `crypto.hmac_sha256` | `hmac_sha256` | `{key, message}` | `{mac}` |
| `crypto.derive_child_seed` | `derive_child_seed` | `{parent_seed, context}` | `{child_seed}` |

### TLS Crypto (BearDog)

| Semantic Capability | BearDog Method | Parameters |
|---------------------|----------------|------------|
| `crypto.x25519_generate_ephemeral` | `x25519_generate_ephemeral` | `{}` |
| `crypto.x25519_derive_secret` | `x25519_derive_secret` | `{private_key, public_key}` |
| `crypto.sign_ed25519` | `ed25519_sign` | `{message, private_key}` |
| `crypto.verify_ed25519` | `ed25519_verify` | `{message, signature, public_key}` |

---

## biomeOS Orchestration (NOT Capabilities)

These are **ecosystem concepts** orchestrated by biomeOS using the primitives above.
They are **NOT** BearDog methods:

| Ecosystem Concept | biomeOS Orchestration |
|-------------------|----------------------|
| `initiate_meeting` | 1. `beacon.get_id` → 2. `beacon.get_seed` → 3. `crypto.encrypt` → 4. network exchange → 5. `crypto.decrypt` → 6. store locally |
| `sync_address_book` | Compare lineage hints → merge meetings → union endpoints |
| `try_all_met_seeds` | Load each met seed → `beacon.try_decrypt` in loop |
| `create_cluster` | Derive cluster seed → register members → share beacon |

### Meeting Protocol Flow (biomeOS Orchestrates)

```rust
// biomeOS BeaconGeneticsManager.initiate_meeting()
async fn initiate_meeting(&mut self, peer: &str, name: &str) -> Result<BeaconId> {
    // Step 1: Get our beacon ID (BearDog primitive)
    let our_id = self.capability_caller
        .call("beacon.get_id", json!({})).await?;
    
    // Step 2: Get our seed for exchange (BearDog primitive)
    let our_seed = self.capability_caller
        .call("beacon.get_seed", json!({})).await?;
    
    // Step 3: Encrypt for transport (BearDog primitive)
    let encrypted = self.capability_caller
        .call("crypto.encrypt", json!({
            "plaintext": our_seed,
            "context": "beacon-exchange-v1"
        })).await?;
    
    // Step 4: Exchange via network (Songbird primitive)
    let response = self.capability_caller
        .call("network.beacon_exchange", json!({
            "endpoint": peer,
            "payload": encrypted
        })).await?;
    
    // Step 5: Decrypt peer's seed (BearDog primitive)
    let peer_seed = self.capability_caller
        .call("crypto.decrypt", json!({
            "ciphertext": response.peer_encrypted_seed,
            "context": "beacon-exchange-v1"
        })).await?;
    
    // Step 6: Store locally (filesystem, not a capability)
    self.store_met_seed_local(&peer_beacon_id, &peer_seed).await?;
    
    Ok(peer_beacon_id)
}
```

---

## Registration at Bootstrap

When deploying Tower Atomic, these translations should be registered:

```rust
// In bootstrap or graph execution
let beardog_socket = format!("/run/user/{}/biomeos/beardog-{}.sock", uid, family_id);

// Beacon capabilities
registry.register_translation("beacon.generate", "beardog", "beacon.generate", &beardog_socket, None);
registry.register_translation("beacon.get_id", "beardog", "beacon.get_id", &beardog_socket, None);
registry.register_translation("beacon.get_seed", "beardog", "beacon.get_seed", &beardog_socket, None);
registry.register_translation("beacon.encrypt", "beardog", "beacon.encrypt", &beardog_socket, None);
registry.register_translation("beacon.decrypt", "beardog", "beacon.decrypt", &beardog_socket, None);
registry.register_translation("beacon.try_decrypt", "beardog", "beacon.try_decrypt", &beardog_socket, None);

// Crypto capabilities
registry.register_translation("crypto.encrypt", "beardog", "chacha20_poly1305_encrypt", &beardog_socket, None);
registry.register_translation("crypto.decrypt", "beardog", "chacha20_poly1305_decrypt", &beardog_socket, None);
registry.register_translation("crypto.encrypt_with_lineage", "beardog", "encrypt_with_lineage_key", &beardog_socket, None);
registry.register_translation("crypto.decrypt_with_lineage", "beardog", "decrypt_with_lineage_key", &beardog_socket, None);
registry.register_translation("crypto.blake3_hash", "beardog", "blake3_hash", &beardog_socket, None);
```

---

## BearDog Evolution Needed

BearDog needs to implement these RPC handlers if not already present:

### Already Implemented (Phase 1)
- `chacha20_poly1305_encrypt`
- `chacha20_poly1305_decrypt`
- `blake3_hash`
- `x25519_generate_ephemeral`
- `x25519_derive_secret`
- `ed25519_sign`
- `ed25519_verify`

### Needs Verification/Implementation
| Method | Purpose | Priority |
|--------|---------|----------|
| `beacon.generate` | Generate new beacon seed | **High** |
| `beacon.get_id` | Get public beacon ID | **High** |
| `beacon.get_seed` | Get beacon seed (for meeting exchange) | **High** |
| `beacon.encrypt` | Encrypt with beacon seed | **High** |
| `beacon.decrypt` | Decrypt with beacon seed | **High** |
| `beacon.try_decrypt` | Try decrypt, return success/failure | **High** |
| `encrypt_with_lineage_key` | Encrypt using lineage-derived key | **Medium** |
| `decrypt_with_lineage_key` | Decrypt using lineage-derived key | **Medium** |

---

## Songbird Evolution Needed

Songbird needs to implement this RPC handler:

| Method | Purpose | Priority |
|--------|---------|----------|
| `network.beacon_exchange` | Exchange beacon seeds during meeting | **Medium** |

**Specification**:
```json
{
  "method": "network.beacon_exchange",
  "params": {
    "endpoint": "192.168.1.100:9900",
    "payload": {
      "beacon_id": "d03029e5...",
      "encrypted_seed": "base64..."
    }
  }
}
```

**Returns**:
```json
{
  "peer_beacon_id": "c86cb868...",
  "peer_encrypted_seed": "base64..."
}
```

---

## Summary

| Layer | Responsibility |
|-------|----------------|
| **biomeOS** | Orchestrates meetings, manages address books, handles seeds |
| **NeuralAPI** | Routes capability.call to correct primal |
| **CapabilityTranslationRegistry** | Maps semantic → actual methods |
| **BearDog** | Executes crypto primitives (encrypt, decrypt, sign, derive) |
| **Songbird** | Executes network primitives (exchange, broadcast, discover) |

**Key Insight**: BearDog doesn't know what a "meeting" is. It just encrypts and decrypts.
biomeOS knows the ecosystem concepts and orchestrates using BearDog's primitives.
