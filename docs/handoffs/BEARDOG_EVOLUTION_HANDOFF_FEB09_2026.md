# BearDog Evolution Handoff - February 9, 2026

**Team**: BearDog (phase1/beardog)
**Priority**: LOW - BearDog is the reference primal, stable
**Codebase**: `ecoPrimals/phase1/beardog/`

---

## Context

BearDog is the cryptography primal. All 8 methods work correctly (SHA3-256, SHA3-512,
Ed25519 sign/verify, X25519 key exchange, ChaCha20 encrypt/decrypt, HMAC). It is the
foundation of the genetic trust model and TLS delegation.

BearDog has no bypasses, no workarounds, no known bugs. It is the reference
implementation for ecoBin v2.0 compliance.

---

## Evolution Items

### 1. Multi-Family Socket Support (NEW)

**What**: Accept `--family-id` flag, create `beardog-{family_id}.sock`.

**Why**: Multi-family support requires each family to have its own BearDog instance
with its own key material derived from the family seed.

**Note**: BearDog is special because its key material is family-specific. A BearDog
instance serving family A MUST NOT share keys with family B. Each family gets its
own BearDog instance with independently derived keys.

**Estimated**: 10 lines

### 2. Secret Storage (FUTURE)

**What**: Store encrypted secrets (API keys, JWT secrets) via BearDog, backed by NestGate.

**Why**: Currently API keys are in plaintext `testing-secrets/api-keys.toml`.
BearDog could encrypt them and NestGate could store the ciphertext.

**How**:
- `secrets.store(key, value)` -> ChaCha20 encrypt -> NestGate `storage.store`
- `secrets.retrieve(key)` -> NestGate `storage.retrieve` -> ChaCha20 decrypt
- Family-scoped: encryption key derived from family seed

**Estimated**: 100 lines (BearDog) + NestGate integration

### 3. `discover_capabilities` JSON-RPC Method (CONSISTENCY)

**What**: Like Songbird, add `discover_capabilities` handler.

**Response**:
```json
{
  "capabilities": [
    "crypto.sha256", "crypto.sha512",
    "crypto.sign", "crypto.verify",
    "crypto.key_exchange",
    "crypto.encrypt", "crypto.decrypt",
    "crypto.hmac",
    "jwt.provision"
  ]
}
```

**Estimated**: 30 lines

---

## Status

BearDog is STABLE. No urgent evolution needed. Multi-family socket is the only
blocking item for the multi-family architecture.
