# NUCLEUS Atomic Composition Patterns

**Date**: January 29, 2026  
**Status**: ✅ Production Ready  
**Version**: 1.0.0

---

## 🎯 Executive Summary

NUCLEUS is biomeOS's deployment architecture based on **atomic composition patterns**. Each atomic is a self-contained, validated unit that can be composed into larger systems while maintaining security, isolation, and capability-based discovery.

```
                    ┌─────────────────────────────────────┐
                    │           NUCLEUS Complete          │
                    │   (Tower + Node + Nest + Squirrel)  │
                    └─────────────────────────────────────┘
                                      │
          ┌───────────────────────────┼───────────────────────────┐
          │                           │                           │
          ▼                           ▼                           ▼
    ┌───────────┐              ┌───────────┐              ┌───────────┐
    │   Tower   │              │   Node    │              │   Nest    │
    │  Atomic   │              │  Atomic   │              │  Atomic   │
    └─────┬─────┘              └─────┬─────┘              └─────┬─────┘
          │                           │                           │
    ┌─────┴─────┐              ┌─────┴─────┐              ┌─────┴─────┐
    │  BearDog  │              │  Tower +  │              │  Tower +  │
    │ + Songbird│              │ Toadstool │              │ NestGate  │
    └───────────┘              └───────────┘              └───────────┘
```

---

## 🏗️ Atomic Definitions

### Tower Atomic (Foundation)

**Purpose**: Security + Network foundation for all other atomics.

| Component | Capability | Status |
|-----------|------------|--------|
| **BearDog** | `crypto`, `security`, `tls_crypto`, `lineage` | ✅ |
| **Songbird** | `http`, `tls`, `discovery`, `secure_http` | ✅ |
| **Neural API** | `orchestration`, `routing`, `capability_discovery` | ✅ |

**Graph**: `graphs/tower_atomic_bootstrap.toml`

```bash
# Deploy Tower Atomic
./scripts/bootstrap_tower_atomic.sh
```

**Validated Capabilities**:
- ✅ TLS 1.3 handshake (93% success rate)
- ✅ HTTPS client (366ms avg latency)
- ✅ Genetic lineage verification
- ✅ Birdsong encrypted communication
- ✅ JSON-RPC over Unix sockets

---

### Node Atomic (Compute)

**Purpose**: Tower + local compute capabilities.

| Component | Capability | Status |
|-----------|------------|--------|
| **Tower** | (inherited) | ✅ |
| **Toadstool** | `compute`, `workload`, `ai_local`, `orchestration` | ✅ |

**Graph**: `graphs/node_atomic_compute.toml`

```bash
# Deploy Node Atomic (requires Tower first)
./plasmidBin/neural-api-server \
  --graph graphs/node_atomic_compute.toml \
  --family-id $FAMILY_ID
```

**Validated Capabilities**:
- ✅ Toadstool health check (56ms)
- ✅ GPU detection (RTX 2070 SUPER)
- ✅ JSON-RPC dual format support
- ✅ Resource estimation

---

### Nest Atomic (Storage)

**Purpose**: Tower + federated storage capabilities.

| Component | Capability | Status |
|-----------|------------|--------|
| **Tower** | (inherited) | ✅ |
| **NestGate** | `storage`, `persistence`, `provenance` | 🟡 |

**Graph**: `graphs/nest_deploy.toml`

```bash
# Deploy Nest Atomic (requires Tower first)
NESTGATE_JWT_SECRET=$(openssl rand -base64 48) \
./plasmidBin/primals/nestgate server \
  --socket /run/user/1000/biomeos/nestgate-$FAMILY_ID.sock
```

**Validated Capabilities**:
- ✅ `storage.store` - Accepts data
- ✅ `storage.list` - Returns keys
- 🟡 `storage.retrieve` - Returns null (persistence pending)

---

### NUCLEUS Complete

**Purpose**: Full ecosystem with all capabilities.

| Layer | Components | Capabilities |
|-------|------------|--------------|
| **Tower** | BearDog + Songbird | Security, Network |
| **Node** | + Toadstool | Local Compute, GPU |
| **Nest** | + NestGate | Storage, Provenance |
| **AI** | + Squirrel | External AI APIs |

**Graph**: `graphs/nucleus_complete.toml`

```bash
# Deploy Complete NUCLEUS
./plasmidBin/neural-api-server \
  --graph graphs/nucleus_complete.toml \
  --family-id $FAMILY_ID
```

---

## 🔄 Composition Patterns

### 1. Layered Composition (Recommended)

Each atomic builds on the previous:

```
1. Tower Atomic (foundation)
   ↓
2. Node Atomic (Tower + compute)
   ↓
3. Nest Atomic (Tower + storage)
   ↓
4. NUCLEUS Complete (all)
```

**Advantages**:
- Clear dependencies
- Incremental validation
- Easy troubleshooting

### 2. Parallel Composition

Deploy independent atomics simultaneously:

```
Tower ──┬── Node (parallel)
        └── Nest (parallel)
```

**Use Case**: Large deployments where Tower is stable

### 3. HSM-Anchored Composition

Use external device as trust root:

```
Pixel 8a (HSM)
    │
    └── Tower ──┬── Node
                └── Nest
```

**Use Case**: Mobile root of trust, hardware security

---

## 📋 Capability Routing

### Semantic Translation

Neural API translates high-level capabilities to primal methods:

```
capability.call("crypto", "sha256", data)
         │
         ▼
    Neural API
         │
         ├── Lookup: "crypto" → "beardog"
         ├── Translate: "sha256" → "crypto.sha256"
         │
         ▼
    BearDog.crypto.sha256(data)
```

### Capability Registry

```toml
# From tower_atomic_bootstrap.toml
[nodes.capabilities_provided]
"sha256" = "crypto.sha256"
"sha384" = "crypto.sha384"
"generate_keypair" = "crypto.x25519_generate_ephemeral"
"http.request" = "http.request"
"http.post" = "http.post"
```

### Cross-Atomic Routing

```
Squirrel → capability.call("http.request", ...) → Neural API → Songbird
    │
    └── capability.call("crypto", "sign") → Neural API → BearDog
```

---

## 🧪 Validation Matrix

### Tower Atomic

| Test | Method | Expected | Actual |
|------|--------|----------|--------|
| BearDog Health | `beardog.health` | `healthy` | ✅ |
| Songbird Health | `songbird.health` | `healthy` | ✅ |
| TLS Handshake | `http.request` to HTTPS | 200 | ✅ |
| Lineage Verify | `federation.verify_family_member` | `is_family_member: true` | ✅ |
| Birdsong Encrypt | `birdsong.encrypt` | ciphertext | ✅ |

### Node Atomic

| Test | Method | Expected | Actual |
|------|--------|----------|--------|
| Toadstool Health | `toadstool.health` | `healthy` | ✅ |
| GPU Detection | `toadstool.query_capabilities` | GPU info | ✅ |
| Resource Estimate | `resources.estimate` | metrics | ✅ |

### Nest Atomic

| Test | Method | Expected | Actual |
|------|--------|----------|--------|
| NestGate Health | `nestgate.health` | `healthy` | ✅ |
| Store Data | `storage.store` | success | ✅ |
| List Keys | `storage.list` | array | ✅ |
| Retrieve Data | `storage.retrieve` | data | 🟡 (null) |

### NUCLEUS Complete

| Test | Method | Expected | Actual |
|------|--------|----------|--------|
| All Primals | `lifecycle.status` | all healthy | ✅ |
| AI Query | `query_ai` via Squirrel | response | ✅ 560ms |
| Multi-AI | Anthropic + HuggingFace | coordinated | ✅ 9/9 |

---

## 🚀 Deployment Commands

### Quick Start

```bash
# 1. Set family ID
export FAMILY_ID=$(cat .family.seed | head -c 16)

# 2. Deploy Tower Atomic
./scripts/bootstrap_tower_atomic.sh

# 3. Verify
echo '{"jsonrpc":"2.0","method":"lifecycle.status","id":1}' | \
  nc -U /run/user/1000/biomeos/neural-api-$FAMILY_ID.sock
```

### Full NUCLEUS

```bash
# Deploy all atomics
./plasmidBin/neural-api-server \
  --graph graphs/nucleus_complete.toml \
  --socket /run/user/1000/biomeos/neural-api-$FAMILY_ID.sock \
  --family-id $FAMILY_ID
```

### Validation Scripts

```bash
# Quick validation
./scripts/validate_nucleus_quick.sh

# Full validation with AI
./scripts/validate_multi_ai.sh
```

---

## 📊 Performance Metrics

| Metric | Value | Notes |
|--------|-------|-------|
| **Tower Bootstrap** | ~5s | BearDog + Songbird + Neural API |
| **HTTPS Latency** | 366ms avg | To api.github.com |
| **AI E2E Latency** | 560ms avg | To Anthropic API |
| **Local Compute** | 56ms | Toadstool health |
| **JSON-RPC Overhead** | ~12 calls/TLS | Crypto operations |

---

## 🔗 Related Documentation

| Document | Description |
|----------|-------------|
| `specs/NUCLEUS_DEPLOYMENT_SPEC.md` | Deployment specification |
| `specs/NUCLEUS_BONDING_MODEL.md` | Covalent/Ionic bonding |
| `specs/PLASMODIUM_OVER_NUCLEUS_SPEC.md` | Over-NUCLEUS collective coordination |

---

## 📋 Handoff Status

| Component | Status | Handoff |
|-----------|--------|---------|
| Tower Atomic | ✅ Production | N/A |
| Node Atomic | ✅ Production | N/A |
| Nest Atomic | 🟡 Persistence | `NESTGATE_PERSISTENCE_HANDOFF.md` |
| Songbird TCP | ❌ Pending | `SONGBIRD_TCP_GATEWAY_HANDOFF.md` |
| Songbird STUN | ❌ Pending | `SONGBIRD_STUN_RENDEZVOUS_HANDOFF.md` |
| BearDog Android | ❌ Pending | `BEARDOG_ANDROID_CROSS_COMPILE_HANDOFF.md` |

---

**Status**: ✅ Tower + Node validated | 🟡 Nest persistence pending | ❌ LAN federation pending (ecosystem evolution targets)

*Updated: April 22, 2026*

