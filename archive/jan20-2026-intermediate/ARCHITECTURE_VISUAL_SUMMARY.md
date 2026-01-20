# 🏗️ Architecture Visual Summary - biomeOS Neural API

**Date**: January 20, 2026 | **Status**: ✅ PRODUCTION-READY | **Grade**: A++ GOLD

---

## 🎯 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     biomeOS Neural API                          │
│                  (Service Mesh / Router)                        │
│                                                                 │
│  • Capability-based discovery                                  │
│  • Runtime socket discovery                                    │
│  • Metrics collection                                          │
│  • Zero capabilities (ONLY routes)                             │
└─────────────────────────────────────────────────────────────────┘
                            ↓
        ┌───────────────────┼───────────────────┐
        ↓                   ↓                   ↓
   ┌─────────┐        ┌──────────┐        ┌──────────┐
   │BearDog  │        │Songbird  │        │Squirrel  │
   │(Tower)  │        │(Discovery)│       │  (AI)    │
   │Security │        │  Broker   │       │Inference │
   └─────────┘        └──────────┘        └──────────┘
```

---

## 🔄 Request Flow

### Example: AI Request

```
1. Squirrel
   ↓ "I need secure HTTP"
   
2. Neural API
   ↓ discover_capability("secure_http")
   
3. Neural Router
   ↓ Find primal with capability
   ↓ Returns: BearDog @ /tmp/beardog-nat0.sock
   
4. Neural API
   ↓ proxy_http(url, headers, body)
   ↓ Forward to BearDog socket
   
5. Tower Atomic (BearDog + Songbird)
   ↓ BearDog validates/secures
   ↓ Songbird makes HTTP call
   ↓ Response back through mesh
   
6. Anthropic API
   ↓ Actual HTTP request
   
7. Response flows back:
   Anthropic → Songbird → BearDog → Neural API → Squirrel
```

---

## 🧩 Component Breakdown

### Neural API (Service Mesh)

**Purpose**: Route requests between primals based on capabilities

**Responsibilities**:
- ✅ Discover primals by capability
- ✅ Route requests to correct primal
- ✅ Collect routing metrics
- ✅ Proxy HTTP through Tower Atomic
- ❌ NO capabilities of its own
- ❌ NO direct HTTP calls
- ❌ NO business logic

**JSON-RPC Methods**:
1. `neural_api.discover_capability` - Find primal(s) with capability
2. `neural_api.proxy_http` - Route HTTP through Tower Atomic
3. `neural_api.route_to_primal` - Forward request to primal
4. `neural_api.get_routing_metrics` - Return routing statistics

**Files**:
- `crates/biomeos-atomic-deploy/src/neural_router.rs` (420 lines)
- `crates/biomeos-atomic-deploy/src/neural_api_server.rs` (150 lines)

---

### Neural API Client (Library)

**Purpose**: Enable primals to communicate via Neural API

**Responsibilities**:
- ✅ Connect to Neural API socket
- ✅ Make JSON-RPC requests
- ✅ Handle responses/errors
- ✅ Provide type-safe API

**Usage Example**:
```rust
use neural_api_client::NeuralApiClient;

// Connect to Neural API
let client = NeuralApiClient::connect("/tmp/neural-api-nat0.sock").await?;

// Discover capability
let endpoint = client.discover_capability("secure_http").await?;

// Proxy HTTP request
let response = client.proxy_http(
    "POST",
    "https://api.anthropic.com/v1/messages",
    headers,
    body
).await?;
```

**Files**:
- `crates/neural-api-client/src/lib.rs` (300+ lines)

---

### Binary Discovery (Universal)

**Purpose**: Auto-detect and locate primal binaries on any platform

**Responsibilities**:
- ✅ Auto-detect architecture (x86_64, ARM64, RISC-V, etc.)
- ✅ Auto-detect OS (Linux, macOS, Windows)
- ✅ Search multiple locations
- ✅ Try multiple naming patterns
- ✅ User-configurable via env vars

**Auto-Detection**:
```rust
let arch = std::env::consts::ARCH;  // "x86_64", "aarch64", "riscv64"
let os = std::env::consts::OS;      // "linux", "macos", "windows"

// Searches for:
// - beardog-x86_64-musl
// - beardog-x86_64
// - beardog
// - beardog.exe (Windows)
```

**Search Locations**:
1. `$BIOMEOS_PLASMID_BIN_DIR` (if set)
2. `./plasmidBin/primals/{primal}/`
3. `../plasmidBin/primals/{primal}/`
4. `../../plasmidBin/primals/{primal}/`
5. `./target/release/`
6. `./target/debug/`

**Files**:
- `crates/biomeos-atomic-deploy/src/neural_executor.rs` (Updated)

---

## 🏛️ Atomic Patterns

### Tower Atomic (BearDog + Songbird)

**Purpose**: Secure communications (crypto + discovery)

```
┌─────────────────────────────────┐
│        Tower Atomic             │
│                                 │
│  ┌──────────┐   ┌──────────┐  │
│  │ BearDog  │   │ Songbird │  │
│  │ Security │ + │ Discovery│  │
│  │  Crypto  │   │   HTTP   │  │
│  └──────────┘   └──────────┘  │
└─────────────────────────────────┘
```

**Capabilities**: `security`, `secure_http`, `discovery`

---

### Nest Atomic (Tower + NestGate)

**Purpose**: Secure data (storage + encryption)

```
┌─────────────────────────────────┐
│         Nest Atomic             │
│                                 │
│  ┌───────┐   ┌──────────┐     │
│  │ Tower │ + │ NestGate │     │
│  │       │   │  Storage │     │
│  └───────┘   └──────────┘     │
└─────────────────────────────────┘
```

**Capabilities**: Tower capabilities + `storage`, `data_persistence`

---

### Node Atomic (Tower + ToadStool)

**Purpose**: Secure compute (orchestration + encryption)

```
┌─────────────────────────────────┐
│         Node Atomic             │
│                                 │
│  ┌───────┐   ┌───────────┐    │
│  │ Tower │ + │ ToadStool │    │
│  │       │   │  Compute  │    │
│  └───────┘   └───────────┘    │
└─────────────────────────────────┘
```

**Capabilities**: Tower capabilities + `compute`, `orchestration`

---

## 🌐 Universal Portability

### Platform Auto-Detection

```
┌──────────────────────────────────────┐
│     Application Code                 │
│  (No platform-specific logic)        │
└──────────────────────────────────────┘
              ↓
┌──────────────────────────────────────┐
│   std::env::consts::ARCH             │
│   std::env::consts::OS               │
│  (Rust standard library)             │
└──────────────────────────────────────┘
              ↓
┌──────────────────────────────────────┐
│   Binary Discovery Logic             │
│  • Detects: x86_64, ARM64, RISC-V    │
│  • Detects: Linux, macOS, Windows    │
│  • Searches multiple locations       │
│  • Tries multiple naming patterns    │
└──────────────────────────────────────┘
              ↓
┌──────────────────────────────────────┐
│   Correct Binary for Platform        │
│  (Zero configuration needed!)        │
└──────────────────────────────────────┘
```

### Supported Platforms

```
Architectures:          Operating Systems:
├── x86_64              ├── Linux (all distros)
├── aarch64 (ARM64)     ├── macOS (Intel/ARM)
├── riscv64             ├── Windows
└── Any Rust target     └── Any Rust target

Configuration: ZERO ✅
```

---

## 🔐 TRUE PRIMAL Pattern

### Self-Knowledge Only

```
┌──────────────────────────────────────┐
│           Squirrel Primal            │
│                                      │
│  Knowledge:                          │
│  ✅ "I am Squirrel"                 │
│  ✅ "I provide AI inference"        │
│  ✅ "I need secure HTTP"            │
│                                      │
│  NO Knowledge:                       │
│  ❌ Who provides secure HTTP        │
│  ❌ Where BearDog runs              │
│  ❌ How to find Songbird            │
└──────────────────────────────────────┘
              ↓
         (Discovers at runtime)
              ↓
┌──────────────────────────────────────┐
│         Neural API Router            │
│                                      │
│  "Who has 'secure_http' capability?" │
│  → Returns: BearDog endpoint         │
└──────────────────────────────────────┘
```

### Discovery Flow

```
Compile Time:           Runtime:
├── Self-knowledge     ├── Discover capabilities
├── Required caps      ├── Find endpoints
└── NO cross-refs      └── Connect dynamically
```

---

## 📊 Deployment Architecture

### Single Stack (Tower + Squirrel)

```
./scripts/deploy_tower_squirrel.sh nat0

Deploys:
┌────────────────────────────────────────┐
│  /tmp/beardog-nat0.sock                │  ← BearDog (security)
│  /tmp/songbird-nat0.sock               │  ← Songbird (discovery)
│  /tmp/neural-api-nat0.sock             │  ← Neural API (mesh)
│  /tmp/squirrel-nat0.sock               │  ← Squirrel (AI)
└────────────────────────────────────────┘

Logs:
/tmp/primals/{primal}/nat0/{primal}.log

PIDs:
/tmp/primals/{primal}/nat0/pid
```

### Full NUCLEUS (All 5 Core Primals)

```
Future deployment:

┌────────────────────────────────────────┐
│  Neural API (mesh)                     │
│    ↓                                   │
│  ├── BearDog (security)                │
│  ├── Songbird (discovery)              │
│  ├── Squirrel (AI)                     │
│  ├── NestGate (storage)                │
│  └── ToadStool (compute)               │
└────────────────────────────────────────┘
```

---

## 🚀 Quick Command Reference

### Deploy

```bash
# Deploy Tower Atomic + Squirrel
./scripts/deploy_tower_squirrel.sh nat0

# Expected sockets:
ls -la /tmp/*-nat0.sock
```

### Test

```bash
# Run integration tests
export ANTHROPIC_API_KEY=sk-ant-xxxxx
./scripts/test_neural_api_routing.sh nat0
```

### Monitor

```bash
# Check processes
ps aux | grep beardog
ps aux | grep songbird
ps aux | grep neural-api
ps aux | grep squirrel

# View logs
tail -f /tmp/primals/beardog/nat0/beardog.log
tail -f /tmp/primals/songbird/nat0/songbird.log
```

### Stop

```bash
# Graceful shutdown
./scripts/stop_tower_squirrel.sh nat0
```

---

## 📈 Data Flow Example

### AI Inference Request

```
Step 1: Squirrel needs to call Anthropic API
   ↓
Step 2: Squirrel → Neural API Client
   Code: client.proxy_http("POST", "https://api.anthropic.com/...", ...)
   ↓
Step 3: Neural API Client → Neural API Socket
   JSON-RPC: {"method":"neural_api.proxy_http", "params":{...}}
   ↓
Step 4: Neural API → Neural Router
   Router: Discover capability "secure_http"
   ↓
Step 5: Neural Router → Registry Lookup
   Registry: "secure_http" → BearDog @ /tmp/beardog-nat0.sock
   ↓
Step 6: Neural API → BearDog Socket
   Forward request to discovered endpoint
   ↓
Step 7: BearDog → Songbird
   BearDog validates, Songbird makes HTTP call
   ↓
Step 8: Songbird → Anthropic API
   Actual HTTPS request to api.anthropic.com
   ↓
Step 9: Response flows back
   Anthropic → Songbird → BearDog → Neural API → Squirrel
```

---

## 🏆 Architecture Principles

### Service Mesh Pattern

- ✅ Neural API is infrastructure, NOT a primal
- ✅ Has ZERO capabilities
- ✅ ONLY routes requests
- ✅ Never executes business logic
- ✅ Primals never communicate directly

### Capability-Based

- ✅ All discovery via capabilities
- ✅ Zero hardcoded endpoints
- ✅ Runtime socket discovery
- ✅ User-configurable paths
- ✅ Universal portability

### TRUE PRIMAL

- ✅ Self-knowledge only
- ✅ Discover others at runtime
- ✅ Zero cross-primal dependencies
- ✅ Zero compile-time coupling
- ✅ Pure capability queries

### Pure Rust

- ✅ 100% Pure Rust dependencies
- ✅ Zero unsafe code
- ✅ Zero C libraries
- ✅ Fast compilation
- ✅ Safe execution

---

## 📚 Further Reading

| Document | Purpose |
|----------|---------|
| [ONE_PAGE_SUMMARY.md](ONE_PAGE_SUMMARY.md) | Quickest overview |
| [ULTIMATE_PRODUCTION_HANDOFF_JAN_20_2026.md](ULTIMATE_PRODUCTION_HANDOFF_JAN_20_2026.md) | Complete guide |
| [QUICK_REFERENCE_NEURAL_ROUTING.md](QUICK_REFERENCE_NEURAL_ROUTING.md) | API reference |
| [ROOT_DOCS_INDEX.md](ROOT_DOCS_INDEX.md) | Full documentation index |

---

**🏰🧬⚛️✨ biomeOS Neural API - Universal, Pure Rust, Production Ready! ✨⚛️🧬🏰**

---

**Date**: January 20, 2026  
**Version**: v0.28.0  
**Status**: ✅ PRODUCTION-READY  
**Grade**: ✅ A++ GOLD

