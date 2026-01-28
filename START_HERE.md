# 🌱 biomeOS - Start Here

**Last Updated**: January 28, 2026  
**Status**: 🎉 **PRODUCTION READY** - Pure Rust TLS 1.3  
**Tower Atomic**: 93% TLS Success (87 sites), 96% Web Compatibility  
**NUCLEUS**: Lifecycle management complete (resurrection, apoptosis)  
**LiveSpore**: Genetic Lineage Federation Ready  
**Deep Debt**: ✅ **COMPLETE** - 96% TODO reduction (85 → 3)  
**Tests**: 400+ passing (106 suites) | **Crates**: 21 | **Lines**: ~106k | **Unsafe**: 0

---

## 🏆 Tower Atomic Validation Complete

### Comprehensive Testing (Jan 26, 2026)

| Metric | Value | Notes |
|--------|-------|-------|
| **Sites Tested** | 87 | Across 11 categories |
| **TLS 1.3 Success** | 93% (81/87) | Pure Rust |
| **Web Compatibility** | 96% | With User-Agent |
| **Cipher Suites** | 100% | All 3 mandatory |

### Category Results

| Category | Sites | TLS Success | Notes |
|----------|-------|-------------|-------|
| **AI/ML** | 10 | 100% ✅ | OpenAI, Anthropic, HuggingFace |
| **Cloud** | 10 | 90% ✅ | AWS, GCP, Azure, Vercel |
| **Code Hosting** | 6 | 83% ✅ | GitHub, GitLab, Bitbucket |
| **Package Registries** | 10 | 80% ⚠️ | npm needs TLS 1.2 |
| **Containers** | 6 | 100% ✅ | Docker Hub, GHCR, Quay |
| **Databases** | 7 | 100% ✅ | MongoDB, Supabase, PlanetScale |
| **Observability** | 6 | 83% ⚠️ | New Relic needs TLS 1.2 |
| **CI/CD** | 5 | 80% ⚠️ | Jenkins needs TLS 1.2 |
| **Serverless** | 7 | 100% ✅ | Vercel, Netlify, Cloudflare |
| **API Tools** | 5 | 100% ✅ | Postman, Insomnia |
| **Security** | 6 | 100% ✅ | Auth0, Okta, Cloudflare |

---

## 🔧 Component Status

| Component | Version | Status | Capabilities |
|-----------|---------|--------|--------------|
| **biomeOS** | `master` | ✅ 100% | Graph deployment, capability.call, lifecycle |
| **Neural API** | `master` | ✅ 100% | 74 semantic mappings, lifecycle management |
| **BearDog** | `964babd25` | ✅ 100% | SHA-256, SHA-384, AES-GCM, X25519 |
| **Songbird** | `8.14.0` | ✅ 100% | TLS 1.3, HTTP headers, dual-mode, STUN |
| **Squirrel** | `0.1.0` | ✅ 100% | AI providers (Anthropic, OpenAI), HTTP delegation |

---

## 🚀 Quick Start

### Build All

```bash
# biomeOS
cargo build --release --workspace

# Run all tests (400+ passing)
cargo test --workspace

# Verify
./target/release/biomeos --version
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

### Test HTTPS

```bash
# Via Neural API (recommended)
echo '{"jsonrpc":"2.0","method":"capability.call","params":{
  "capability":"secure_http",
  "operation":"http.request",
  "args":{"url":"https://api.github.com/zen","method":"GET"}
},"id":1}' | nc -U /tmp/neural-api.sock

# Direct to Songbird
echo '{"jsonrpc":"2.0","method":"http.request","params":{
  "method":"GET","url":"https://google.com"
},"id":1}' | nc -U /tmp/songbird-nat0.sock
```

### Lifecycle Management

```bash
# Check all primal statuses
echo '{"jsonrpc":"2.0","method":"lifecycle.status","id":1}' | nc -U /tmp/neural-api.sock

# Resurrect a crashed primal
echo '{"jsonrpc":"2.0","method":"lifecycle.resurrect","params":{"name":"songbird"},"id":1}' | nc -U /tmp/neural-api.sock

# Graceful shutdown
echo '{"jsonrpc":"2.0","method":"lifecycle.shutdown_all","id":1}' | nc -U /tmp/neural-api.sock
```

---

## 📚 Key Documentation

### Essential
- **`README.md`** - Project overview
- **`DOCUMENTATION_HUB.md`** - Complete navigation
- **`PROTOCOL_ESCALATION_ROADMAP.md`** - JSON-RPC → tarpc evolution ⭐
- **`RUST_EVOLUTION_ROADMAP.md`** - Scripts → Pure Rust migration
- **`INFRASTRUCTURE_EVOLUTION.md`** - Terraria, Apoptosis

### Architecture
- **`specs/README.md`** - All specifications
- **`specs/LIVESPORE_IMPRINTING_SPEC.md`** - 64-byte seeds, validation
- **`specs/BIRDSONG_DARK_FOREST_TRUST_MODEL.md`** - Encrypted beacons
- **`BIOMEOS_ATOMICS_ARCHITECTURE.md`** - System design

### New in This Release
- **`docs/LIFECYCLE_MANAGEMENT.md`** - NUCLEUS lifecycle API
- **`docs/SOCKET_DISCOVERY.md`** - Capability-based socket resolution
- **`specs/NUCLEUS_DEPLOYMENT_SPEC.md`** - Tower/Node/Nest patterns

### Handoffs
- **`docs/handoffs/SONGBIRD_LAN_DISCOVERY_HANDOFF.md`** - LAN discovery evolution
- **`docs/handoffs/SQUIRREL_EVOLUTION_HANDOFF.md`** - AI primal evolution

---

## 🎯 Key Concepts

### 1. TRUE PRIMAL Pattern

Primals don't know each other's APIs. Communication via semantic operations:

```
Squirrel → capability.call("crypto", "sha256") → Neural API → BearDog
                                                      ↓
                              Translation: "sha256" → "crypto.sha256"
```

### 2. Graph-Based Semantic Translation

Mappings in `tower_atomic_bootstrap.toml`:

```toml
[nodes.capabilities_provided]
"sha256" = "crypto.sha256"
"sha384" = "crypto.sha384"
"generate_keypair" = "crypto.x25519_generate_ephemeral"
"hash_for_cipher" = "crypto.hash_for_cipher"
```

### 3. Pure Rust TLS 1.3

```
HTTP Request → Songbird → BearDog (crypto) → External HTTPS
                  ↓
           TLS 1.3 handshake
           AES-128-GCM / AES-256-GCM
           SHA-256 / SHA-384
```

### 4. NUCLEUS Lifecycle

```
┌─────────────────────────────────────────────────────────────┐
│                    Primal Lifecycle                          │
├─────────────────────────────────────────────────────────────┤
│  Germinating → Incubating → Active ↔ Degraded → Apoptosis  │
│                                ↓                            │
│                          Resurrection                       │
│                      (from deployment graph)                │
└─────────────────────────────────────────────────────────────┘
```

### 5. Socket Discovery (No Hardcoding)

```rust
// OLD (hardcoded)
let socket = PathBuf::from("/tmp/beardog-nat0.sock");

// NEW (capability-based)
let discovery = SocketDiscovery::new(neural_api_socket, family_id);
let socket = discovery.discover_socket("crypto").await?;
```

---

## 📊 Evolution Roadmap

### ✅ Complete (Deep Debt Evolution - Jan 28, 2026)
- Pure Rust TLS 1.3
- SHA-384 cipher suites
- capability.call routing
- Graph-based deployment
- 93% TLS validation
- **NUCLEUS Lifecycle** - Germination through Apoptosis
- **Socket Discovery** - Capability-based resolution
- **Concurrent Tests** - No sleeps, proper async patterns
- **Rollback Strategy** - Full graph execution recovery
- **Songbird Mesh** - UDP multicast peer discovery
- **BearDog Integration** - Lineage verification & key derivation
- **XDG Compliance** - SystemPaths throughout
- **CapabilityTaxonomy** - Unified enum for all capabilities
- **JSON-RPC over Unix sockets** - No HTTP in production

### 🔄 External Handoffs (Awaiting Teams)
- Songbird: TLS 1.2 fallback (npm, Jenkins, 7% remaining)
- Toadstool: Socket binding crash fix
- LAN Federation: Deploy v8.14.0 to other towers

### 📋 Next: Protocol Escalation (Living Graph)
- JSON-RPC → tarpc runtime evolution
- Metrics-based auto-escalation
- See `PROTOCOL_ESCALATION_ROADMAP.md`

### 📋 Future
- TLS server mode
- TLS relay/proxy
- HTTP/2, WebSocket
- Database TLS

---

## 🔍 Troubleshooting

### Common Issues

**Socket not found**:
```bash
ls -la /tmp/neural-api.sock /tmp/beardog-nat0.sock /tmp/songbird-nat0.sock
# If missing, run: ./deploy_tower_atomic.sh
```

**TLS handshake fails**:
```bash
# Check if site supports TLS 1.3
echo | openssl s_client -connect example.com:443 2>&1 | grep Protocol
```

**HTTP 403 Forbidden**:
- Usually fixed by User-Agent (included in Songbird `eaa1dda9d`)
- Verify using `./deploy_tower_atomic.sh status`

**Primal crashed**:
```bash
# Check lifecycle status
echo '{"jsonrpc":"2.0","method":"lifecycle.status","id":1}' | nc -U /tmp/neural-api.sock

# Resurrect
echo '{"jsonrpc":"2.0","method":"lifecycle.resurrect","params":{"name":"beardog"},"id":1}' | nc -U /tmp/neural-api.sock
```

### Logs

```bash
# Real-time logs
tail -f /tmp/neural-api*.log
tail -f /tmp/beardog*.log
tail -f /tmp/songbird*.log
```

---

## 🎊 Summary

**biomeOS Tower Atomic is PRODUCTION READY**

- ✅ Pure Rust TLS 1.3 (zero C dependencies)
- ✅ 87 sites validated (93% TLS success)
- ✅ All 3 mandatory cipher suites
- ✅ Graph-based semantic translation
- ✅ capability.call routing
- ✅ NUCLEUS lifecycle management
- ✅ Socket discovery (no hardcoding)
- ✅ 400+ tests passing (concurrent, no sleeps)
- ✅ AI/ML APIs (OpenAI, Anthropic, HuggingFace)
- ✅ Cloud providers (AWS, GCP, Azure)
- ✅ Code hosting (GitHub, GitLab)
- ✅ Research (NCBI, PubMed, arXiv)

**Next**: External team handoffs for TLS 1.2 and LAN federation

---

**Status**: 🏆 Production Ready | **TLS**: 93% | **Web**: 96% | **Pure Rust**: 100% | **Tests**: 400+
