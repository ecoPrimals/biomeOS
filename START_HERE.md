# 🌱 biomeOS - Start Here

**Last Updated**: January 26, 2026 (13:45 UTC)  
**Status**: 🟡 **IN PROGRESS - Auth Tag Verification Issue**  
**Current State**: Key derivation working! Auth tag mismatch in application data decryption.

---

## 🔧 Current Status (13:45 UTC)

### Commits Applied Today
- **BearDog `fb7513739`**: RFC 8446 compliant `derive_application_secrets` API
- **Songbird `73431b6db`**: Pass `cipher_suite` to `tls_derive_application_secrets`

### TLS Pipeline Status
```
Songbird ─► capability.call("crypto", "generate_keypair") ─► Neural API ─► BearDog ✅
Songbird ─► capability.call("crypto", "derive_secret") ─► Neural API ─► BearDog ✅
Songbird ─► capability.call("tls_crypto", "derive_handshake_secrets") ─► Neural API ─► BearDog ✅
Songbird ─► capability.call("crypto", "decrypt_aes_128_gcm") ─► Neural API ─► BearDog ⚠️
                                                                              ↑
                                                            AUTH TAG FAILURE:
                                                            Keys derived but don't
                                                            match server's keys
```

**What Works:**
- ✅ All crypto operations via capability.call
- ✅ Correct key length (16 bytes for AES-128-GCM)
- ✅ BearDog API now accepts `handshake_secret` (RFC 8446 compliant)
- ✅ Graph-based semantic translation (45+ mappings)
- ✅ plasmidBin deployment model

**Current Issue (Auth Tag Verification):**
- ⚠️ AES-128-GCM decryption returns "authentication tag verification failed"
- This means derived keys don't match what the server used
- See `SONGBIRD_TLS_HANDOFF_JAN26.md` for investigation details

---

## Quick Status

| Component | Status | Notes |
|-----------|--------|-------|
| **biomeOS** | ✅ 100% | Graph-based semantic translation |
| **Neural API** | ✅ 100% | 45+ semantic mappings, capability.call |
| **BearDog** | ✅ 100% | RFC 8446 API compliant |
| **Songbird** | ⚠️ 98% | Auth tag issue in TLS decryption |
| **Tower Atomic** | ⚠️ 95% | Architecture validated, key derivation issue |

---

## Quick Start

### Build

```bash
# biomeOS
cargo build --release -p biomeos-unibin

# BearDog (in ecoPrimals/phase1/beardog)
cargo build --release -p beardog-cli

# Songbird (in ecoPrimals/phase1/songbird)
cargo build --release -p songbird-orchestrator
```

### Deploy Tower Atomic

```bash
# One-command deployment
./deploy_tower_atomic.sh

# Check status
./deploy_tower_atomic.sh status

# Stop
./deploy_tower_atomic.sh stop
```

### Test capability.call

```bash
# Direct crypto
echo '{"jsonrpc":"2.0","method":"capability.call","params":{"capability":"crypto","operation":"sha256","args":{"data":"aGVsbG8gd29ybGQ="}},"id":1}' | nc -U /tmp/neural-api.sock

# GitHub API via Tower Atomic (Pure Rust TLS 1.3)
echo '{"jsonrpc":"2.0","method":"capability.call","params":{"capability":"secure_http","operation":"http.request","args":{"url":"https://api.github.com/zen","method":"GET"}},"id":1}' | nc -U /tmp/neural-api.sock

# OpenAI (requires API key)
echo '{"jsonrpc":"2.0","method":"capability.call","params":{"capability":"secure_http","operation":"http.request","args":{"url":"https://api.openai.com/v1/models","method":"GET","headers":{"Authorization":"Bearer sk-..."}}},"id":1}' | nc -U /tmp/neural-api.sock
```

---

## Key Concepts

### 1. TRUE PRIMAL Pattern

Primals don't know each other's APIs. Communication via semantic operations:

```rust
// Caller uses semantic name:
capability.call("crypto", "generate_keypair")

// Neural API translates via graph:
"generate_keypair" → "crypto.x25519_generate_ephemeral"

// BearDog receives actual method
```

### 2. Graph-Based Semantic Translation

Mappings defined in `tower_atomic_bootstrap.toml`:

```toml
[nodes.capabilities_provided]
"crypto.generate_keypair" = "crypto.x25519_generate_ephemeral"
"crypto.encrypt" = "crypto.chacha20_poly1305_encrypt"
"sha256" = "crypto.sha256"
```

### 3. UniBin Architecture

Single binary with subcommands:

```bash
biomeos neural-api    # Neural API server
biomeos deploy        # Graph deployment
biomeos doctor        # Health diagnostics
biomeos cli           # Interactive CLI
```

---

## Project Structure

```
biomeOS/
├── crates/
│   ├── biomeos/                  # UniBin main binary
│   ├── biomeos-atomic-deploy/    # Neural API + Graph execution ⭐
│   ├── biomeos-core/             # Core types and traits
│   ├── biomeos-nucleus/          # Discovery and IPC
│   └── ... (13 crates total)
├── graphs/
│   └── tower_atomic_bootstrap.toml  # ⭐ Semantic mappings
├── specs/                        # Technical specifications
└── archive/                      # Historical documentation

⭐ = Key files for Tower Atomic
```

---

## Documentation

### Essential

- **`README.md`** - Project overview
- **`DOCUMENTATION_HUB.md`** - Complete doc organization
- **`TOWER_ATOMIC_STATUS.md`** - Current Tower Atomic status
- **`INFRASTRUCTURE_EVOLUTION.md`** - Evolution roadmap (Terraria, Apoptosis)

### Architecture

- **`BIOMEOS_ATOMICS_ARCHITECTURE.md`** - Atomics system design
- **`TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md`** - Zero coupling pattern
- **`GENOMEBIN_ARCHITECTURE_STANDARD.md`** - Binary standards

### Deployment

- **`deploy_tower_atomic.sh`** - Production deployment script
- **`graphs/tower_atomic_bootstrap.toml`** - Graph configuration

### Session Archive

Historical session docs are in `archive/`

---

## Recent Fixes

### January 26, 2026

1. **Neural API**: Fixed translation lookup to try both `{capability}.{operation}` and `{operation}`
2. **Songbird**: Fixed `SongbirdHttpClient` to use `BearDogProvider::from_env()` (Neural API mode by default)

---

## Getting Help

### Common Commands

```bash
./target/release/biomeos doctor      # Diagnostics
./target/release/biomeos deploy -n   # Dry-run deployment
cargo test --workspace              # Run tests
```

### Logs

- Neural API: `/tmp/neural-api*.log`
- BearDog: `/tmp/beardog*.log`
- Songbird: `/tmp/songbird*.log`

---

**Status**: ✅ Tower Atomic 100% Ready - GitHub API connectivity verified via Pure Rust TLS 1.3 🚀
