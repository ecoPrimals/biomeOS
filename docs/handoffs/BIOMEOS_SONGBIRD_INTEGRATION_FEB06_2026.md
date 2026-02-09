# biomeOS ↔ Songbird Integration Handoff

**Date**: February 6, 2026  
**Status**: ✅ Integration Testing Infrastructure Ready  
**Purpose**: Document integration test preparation and semantic translation wiring

---

## Executive Summary

biomeOS is now ready to support Songbird's Sovereign Onion Service evolution. All semantic capability translations have been wired up and tested, enabling Songbird to delegate crypto operations to BearDog via biomeOS Neural API.

### Key Achievements

1. **BeardogCryptoClient** created in Songbird (`songbird-sovereign-onion`)
2. **12 integration tests** verify semantic translations in biomeOS
3. **Deployment graph** ready for Sovereign Onion Service
4. **TRUE PRIMAL pattern** enforced via capability routing

---

## Architecture: TRUE PRIMAL Pattern

```
┌────────────────────────────────────────────────────────────────┐
│                    biomeOS (Neural API)                         │
│                                                                 │
│  capability.call("onion.encrypt", {...})                       │
│         ↓                                                       │
│  CapabilityTranslationRegistry                                 │
│         ↓                                                       │
│  Translation: onion.encrypt → crypto.chacha20_poly1305_encrypt │
│  Provider: beardog                                             │
│         ↓                                                       │
│  Route to BearDog socket                                       │
└─────────────────────┬──────────────────────┬───────────────────┘
                      │                      │
              ┌───────▼───────┐      ┌───────▼───────┐
              │   BearDog     │      │   Songbird    │
              │  (Security)   │      │  (Network)    │
              │               │      │               │
              │ • Ed25519     │      │ • Onion Svc   │
              │ • X25519      │      │ • Mesh Relay  │
              │ • ChaCha20    │      │ • Hole Punch  │
              │ • SHA3-256 ✨  │      │ • STUN        │
              │ • HMAC        │      │               │
              └───────────────┘      └───────────────┘
```

---

## Files Created/Modified

### biomeOS (Phase 2)

**New Files**:
- `crates/biomeos-atomic-deploy/src/capability_translation_integration_tests.rs`
  - 12 integration tests for Sovereign Onion translations

**Already Configured**:
- `crates/biomeos-atomic-deploy/src/capability_translation.rs` - All onion translations
- `crates/biomeos-atomic-deploy/src/neural_api_server/routing.rs` - Direct method routing
- `graphs/sovereign_onion_genome.toml` - Deployment graph

### Songbird (Phase 1)

**New Files**:
- `crates/songbird-sovereign-onion/src/beardog_crypto.rs` - BearDog crypto client

**Modified Files**:
- `crates/songbird-sovereign-onion/src/error.rs` - Added RPC/connection error variants
- `crates/songbird-sovereign-onion/src/lib.rs` - Exported beardog_crypto module
- `crates/songbird-sovereign-onion/Cargo.toml` - Added base64 dependency

---

## Capability Translations Verified

### BearDog Crypto (Security Domain)

| Semantic Capability | Actual Method | Purpose |
|---------------------|---------------|---------|
| `crypto.sha3_256` | `crypto.sha3_256` | .onion address checksum |
| `onion.hash_checksum` | `crypto.sha3_256` | Alias for .onion |
| `onion.generate_identity` | `crypto.ed25519_generate_keypair` | .onion identity |
| `onion.session_key` | `crypto.x25519_generate_ephemeral` | Session keys |
| `onion.derive_shared` | `crypto.x25519_derive_secret` | ECDH |
| `onion.encrypt` | `crypto.chacha20_poly1305_encrypt` | Data encryption |
| `onion.decrypt` | `crypto.chacha20_poly1305_decrypt` | Data decryption |
| `onion.hkdf_extract` | `crypto.hmac_sha256` | Key derivation |
| `onion.hkdf_expand` | `crypto.hmac_sha256` | Key derivation |

### Songbird Network (Network Domain)

| Semantic Capability | Actual Method | Purpose |
|---------------------|---------------|---------|
| `mesh.status` | `mesh.status` | Mesh network status |
| `mesh.find_path` | `mesh.find_path` | Path discovery |
| `mesh.announce` | `mesh.announce` | Relay announcement |
| `mesh.peers` | `mesh.list_peers` | Peer listing |
| `punch.request` | `punch.request` | Hole punch initiation |
| `punch.status` | `punch.status` | Punch attempt status |
| `stun.discover` | `stun.get_public_address` | Public IP discovery |
| `stun.detect_nat_type` | `stun.detect_nat_type` | NAT classification |
| `relay.serve` | `relay.serve` | Start relay service |
| `relay.status` | `relay.status` | Relay health |

---

## BeardogCryptoClient Usage

The new `BeardogCryptoClient` in Songbird enables TRUE PRIMAL crypto delegation:

```rust
use songbird_sovereign_onion::BeardogCryptoClient;

// Create client from environment
let client = BeardogCryptoClient::from_env()?;

// Generate Ed25519 identity for .onion address
let keypair = client.ed25519_generate_keypair()?;

// Derive .onion address checksum (SHA3-256)
let checksum = client.sha3_256(&data)?;

// Session key exchange (X25519)
let ephemeral = client.x25519_generate_ephemeral()?;
let shared = client.x25519_derive_secret(&our_secret, &their_public)?;

// Encrypt data (ChaCha20-Poly1305)
let ciphertext = client.chacha20_poly1305_encrypt(&key, &nonce, &plaintext)?;
let plaintext = client.chacha20_poly1305_decrypt(&key, &nonce, &ciphertext)?;
```

### Environment Variables

The client resolves BearDog socket in this order:
1. `BEARDOG_SOCKET` - Direct BearDog socket
2. `CRYPTO_PROVIDER_SOCKET` - biomeOS-wired provider
3. XDG runtime fallback

---

## Integration Tests

All 12 tests pass in `capability_translation_integration_tests.rs`:

```
test_onion_crypto_translations_registered
test_onion_capabilities_route_to_beardog
test_mesh_capabilities_registered
test_mesh_capabilities_route_to_songbird
test_hole_punch_capabilities
test_stun_capabilities
test_relay_capabilities
test_provider_capability_summary
test_semantic_to_actual_mapping
test_domain_categorization
test_true_primal_pattern_compliance
test_sovereign_onion_full_capability_set
```

---

## BearDog Status: ✅ READY

**SHA3-256 Already Implemented** in BearDog:

```rust
// BearDog crates/beardog-tunnel/src/unix_socket_ipc/handlers/crypto_handler.rs
"crypto.sha3_256" => handle_sha3_256(params)
"beardog.crypto.sha3_256" => handle_sha3_256(params)  // Direct Songbird calls
```

All onion crypto methods available:
- `beardog.crypto.sha3_256` - .onion address checksum
- `beardog.crypto.sign_ed25519` - Identity signing
- `beardog.crypto.verify_ed25519` - Signature verification
- `beardog.crypto.x25519_generate_ephemeral` - Session keys
- `beardog.crypto.x25519_derive_secret` - ECDH
- `beardog.crypto.chacha20_poly1305_encrypt` - Data encryption
- `beardog.crypto.chacha20_poly1305_decrypt` - Data decryption
- `beardog.crypto.hmac_sha256` - HKDF

### Songbird Team (~4 hours, after BearDog)

1. **Refactor direct crypto** in `sovereign-onion` to use `BeardogCryptoClient`
2. **Update tests** to mock BearDog responses
3. **Integration test** with live BearDog

### biomeOS Team (30 min)

**Environment wiring** verification:
- Ensure `CRYPTO_PROVIDER_SOCKET` is set when starting Songbird
- Test deployment graph execution

---

## Testing the Integration

### Step 1: Start BearDog

```bash
# In biomeOS
cargo run --bin neural-api-server
```

### Step 2: Test Capability Discovery

```bash
echo '{"jsonrpc":"2.0","method":"capability.discover","params":{"capability":"crypto.sha3_256"},"id":1}' | nc -U /tmp/neural-api.sock
```

### Step 3: Test Direct Method Call

```bash
echo '{"jsonrpc":"2.0","method":"mesh.status","params":{},"id":1}' | nc -U /tmp/neural-api.sock
```

### Step 4: Deploy Sovereign Onion Graph

```bash
cargo run --bin neural-deploy -- --graph graphs/sovereign_onion_genome.toml
```

---

## Quality Metrics

| Metric | Value |
|--------|-------|
| biomeOS Tests | 661+ passing |
| Songbird Tests | 27/27 passing |
| Integration Tests | 12/12 passing |
| Clippy Warnings | 0 new |
| TRUE PRIMAL Compliance | 100% |

---

## Next Steps

1. ~~**BearDog**: Implement `crypto.sha3_256`~~ ✅ **DONE** - Already implemented!
2. **Songbird**: Refactor to use `BeardogCryptoClient` (~4.5 hours remaining)
   - See: `songbird/BEARDOG_CRYPTO_REFACTOR_HANDOFF_FEB06_2026.md`
3. ~~**biomeOS**: Update tower graph~~ ✅ **DONE** - `tower_atomic_bootstrap.toml` updated
4. **All Teams**: End-to-end NAT traversal test (after Songbird refactor)

---

## Songbird Refactoring Debt Summary

### Files Requiring Changes

| File | Debt Type | Effort |
|------|-----------|--------|
| `keys.rs` | Ed25519, X25519, HMAC-SHA256 | ~1.5 hours |
| `address.rs` | SHA3-256 | ~30 min |
| `crypto.rs` | ChaCha20-Poly1305 | ~30 min |
| `service.rs` | Wire up client | ~1 hour |
| Tests | Mock BearDog | ~1 hour |

### Specific Changes Needed

1. **`keys.rs`**:
   - `OnionIdentity::generate()` → `generate_via_beardog()`
   - `EphemeralKeypair::generate()` → `generate_via_beardog()`
   - `SessionKeys::derive()` → `derive_via_beardog()`
   - Change structs from `ed25519_dalek`/`x25519_dalek` types to `[u8; 32]`

2. **`address.rs`**:
   - `derive_onion_address()` → `derive_onion_address_via_beardog()`
   - `validate_onion_address()` → `validate_onion_address_via_beardog()`

3. **`crypto.rs`**:
   - `encrypt_data()` → `encrypt_data_via_beardog()`
   - `decrypt_data()` → `decrypt_data_via_beardog()`

### Recommended Pattern: Hybrid Mode

Follow the `songbird-tls/src/cert/generator.rs` pattern:

```rust
pub enum CryptoMode {
    Standalone,  // Offline/testing
    BearDog,     // TRUE PRIMAL production
    Auto,        // Try BearDog, fallback to standalone
}
```

This allows testing without BearDog while ensuring TRUE PRIMAL in production.

### Tower Graph Updates (Feb 6, 2026)

`graphs/tower_atomic_bootstrap.toml` now includes:
- **BearDog**: All sovereign onion crypto capabilities (SHA3-256, Ed25519, X25519, ChaCha20)
- **Songbird**: mesh, onion, relay, stun, punch capabilities
- **Environment**: `CRYPTO_PROVIDER_SOCKET`, `SONGBIRD_ONION_ENABLED`, `SONGBIRD_MESH_ENABLED`

---

## References

### biomeOS (This Repo)
- `graphs/tower_atomic_bootstrap.toml` - Updated tower graph with sovereign onion
- `graphs/sovereign_onion_genome.toml` - Dedicated sovereign onion deployment
- `docs/handoffs/BEARDOG_ONION_CRYPTO_HANDOFF_FEB06_2026.md` - BearDog handoff
- `docs/handoffs/SOVEREIGN_BEACON_MESH_HANDOFF_FEB06_2026.md` - Full handoff
- `crates/biomeos-atomic-deploy/src/capability_translation_integration_tests.rs` - Integration tests

### Songbird (Phase 1)
- `BEARDOG_CRYPTO_REFACTOR_HANDOFF_FEB06_2026.md` - **Detailed refactoring guide**
- `crates/songbird-sovereign-onion/src/beardog_crypto.rs` - Ready-to-use BearDog client
- `crates/songbird-tls/src/cert/generator.rs` - Reference hybrid pattern

### BearDog (Phase 1)
- `crates/beardog-tunnel/src/unix_socket_ipc/handlers/crypto_handler.rs` - All crypto methods

---

**Status**: ✅ Ready for Songbird Refactoring → Integration Testing

**All debt catalogued and handoff document delivered to Songbird team.**

🧬 biomeOS | 🐦 Songbird | 🐻🐕 BearDog | ✅ TRUE PRIMAL
