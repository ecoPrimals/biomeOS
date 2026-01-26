# 🌱 biomeOS - Start Here

**Last Updated**: January 26, 2026  
**Status**: ✅ **PRODUCTION READY - Tower Atomic Operational**  
**Current State**: Pure Rust TLS 1.3 → GitHub API verified!

---

## 🎉 BREAKTHROUGH: Tower Atomic Working!

**January 26, 2026** - Full end-to-end validation complete:

```
User Request
  ↓ capability.call("secure_http", "http.request")
Neural API (semantic routing) ✅
  ↓ Translation: "generate_keypair" → "crypto.x25519_generate_ephemeral"
Songbird (Pure Rust TLS 1.3) ✅
  ↓
BearDog (Pure Rust crypto) ✅
  ↓
GitHub API → 200 OK ✅
```

---

## Quick Status

| Component | Status | Notes |
|-----------|--------|-------|
| **biomeOS** | ✅ 100% | Graph-based semantic translation |
| **Neural API** | ✅ 100% | 39 semantic mappings, capability.call |
| **BearDog** | ✅ 100% | Pure Rust crypto, auto-registration |
| **Songbird** | ✅ 100% | Pure Rust TLS 1.3, Neural API mode |
| **Tower Atomic** | ✅ 100% | GitHub API verified! |

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

### Run Tower Atomic

```bash
# Terminal 1: Neural API
export RUST_LOG=info
export BIOMEOS_MODE=coordinated
./target/release/biomeos neural-api --socket /tmp/neural-api.sock

# Terminal 2: BearDog
export NEURAL_API_SOCKET=/tmp/neural-api.sock
./target/release/beardog server --socket /tmp/beardog.sock

# Terminal 3: Songbird
export NEURAL_API_SOCKET=/tmp/neural-api.sock
./target/release/songbird server
```

### Test capability.call

```bash
# Direct crypto
echo '{"jsonrpc":"2.0","method":"capability.call","params":{"capability":"crypto","operation":"sha256","args":{"data":"aGVsbG8gd29ybGQ="}},"id":1}' | nc -U /tmp/neural-api.sock

# GitHub API via Tower Atomic
echo '{"jsonrpc":"2.0","method":"capability.call","params":{"capability":"secure_http","operation":"http.request","args":{"url":"https://api.github.com/zen","method":"GET"}},"id":1}' | nc -U /tmp/neural-api.sock
```

### Full Integration Test

```bash
./test_tower_atomic_full.sh
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

### Architecture

- **`BIOMEOS_ATOMICS_ARCHITECTURE.md`** - Atomics system design
- **`TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md`** - Zero coupling pattern
- **`GENOMEBIN_ARCHITECTURE_STANDARD.md`** - Binary standards

### Session Archive

Historical session docs are in `archive/session_jan_26_2026_tower_atomic/`

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
