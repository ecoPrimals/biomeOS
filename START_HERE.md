# Start Here - biomeOS

**Last Updated**: January 29, 2026  
**Status**: Production Ready - Universal IPC v3.0

---

## What is biomeOS?

biomeOS is the **ecosystem orchestrator** for ecoPrimals - a federation of autonomous Rust programs (primals) that communicate via capability-based discovery and Universal IPC v3.0.

### Key Concepts

- **Primals**: Self-contained Rust binaries with specific capabilities
- **Atomics**: Primal combinations (Tower = BearDog + Songbird)
- **NUCLEUS**: Complete system (Tower + Node + Nest)
- **Neural API**: Semantic routing via `capability.call`
- **Universal IPC v3.0**: Multi-transport communication (Unix/Abstract/TCP)

---

## Quick Start

### 1. Deploy Tower Atomic (5 minutes)

```bash
cd livespore-usb/$(uname -m)/scripts/
FAMILY_ID=my_family ./start_tower.sh

# Verify
echo '{"jsonrpc":"2.0","method":"health","id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/beardog-my_family.sock
```

### 2. Deploy Full NUCLEUS (10 minutes)

```bash
cd livespore-usb/$(uname -m)/scripts/
FAMILY_ID=my_family ./deploy_atomic.sh nucleus

# All sockets created at:
# /run/user/$UID/biomeos/{primal}-{family_id}.sock
```

### 3. Use capability.call (Neural API)

```bash
# Discover who provides "crypto" capability
echo '{"jsonrpc":"2.0","method":"capability.discover","params":{"capability":"crypto"},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/neural-api.sock

# Call capability (routed automatically)
echo '{"jsonrpc":"2.0","method":"capability.call","params":{"capability":"crypto","operation":"sha256","data":"hello"},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/neural-api.sock
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        NUCLEUS                               │
├─────────────────────────────────────────────────────────────┤
│  Layer 2: biomeOS + Neural API                              │
│           (semantic translation, capability routing)         │
├─────────────────────────────────────────────────────────────┤
│  Layer 1: Atomics                                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │  Tower   │  │   Node   │  │   Nest   │  │ Squirrel │   │
│  │ BearDog  │  │  Tower   │  │  Tower   │  │  AI/MCP  │   │
│  │ Songbird │  │ Toadstool│  │ NestGate │  │          │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
├─────────────────────────────────────────────────────────────┤
│  Layer 0: Primals (evolve independently)                    │
└─────────────────────────────────────────────────────────────┘
```

---

## Key Documents

| Document | Purpose |
|----------|---------|
| `README.md` | Complete overview |
| `CURRENT_STATUS.md` | Latest status |
| `specs/PRIMAL_DEPLOYMENT_STANDARD.md` | Deployment standard |
| `specs/EVOLUTION_PATH.md` | Scripts to graphs migration |

---

## Deployment Paths

### For x86_64 (Linux/USB)

```bash
cd livespore-usb/x86_64/scripts/
./deploy_atomic.sh nucleus
```

### For aarch64 (Pixel/ARM)

```bash
adb push pixel8a-deploy /data/local/tmp/biomeos
adb shell /data/local/tmp/biomeos/start_nucleus_mobile.sh
```

### Graph-based (Phase 2)

```bash
./deploy_atomic.sh --graph nucleus
# Uses Neural API for deployment orchestration
```

---

## Universal IPC v3.0 - Transport Discovery

Primals discover communication endpoints with automatic fallback:

```
Tier 1 (Native - High Performance):
  1. $PRIMAL_SOCKET environment variable
  2. $XDG_RUNTIME_DIR/biomeos/primal.sock
  3. /run/user/$UID/biomeos/primal.sock
  4. @biomeos_primal (Abstract socket - Linux/Android)

Tier 2 (Universal - Cross-Device):
  5. TCP localhost:910X
  6. TCP remote:910X (federation)
```

### Cross-Device Example

```bash
# On Desktop (local AI)
SQUIRREL_SOCKET=tcp://localhost:9104 ./squirrel

# On Pixel 8a (API gateway)
SONGBIRD_TCP_HOST=0.0.0.0 ./songbird

# Desktop Squirrel can now route HTTP through Pixel's Songbird
```

---

## Standards

| Standard | Description |
|----------|-------------|
| **ecoBin v2.0** | 100% Pure Rust, zero C deps |
| **Universal IPC v3.0** | Multi-transport (Unix/Abstract/TCP) |
| **PRIMAL_DEPLOYMENT_STANDARD** | Deterministic cross-platform |
| **Semantic Method Naming** | capability.call routing |
| **AGPL-3.0-only** | License requirement |

---

## Need Help?

1. Check `CURRENT_STATUS.md` for latest status
2. See `docs/handoffs/` for evolution reports
3. Review `specs/` for standards
4. Explore `livespore-usb/` for deployment scripts

---

**Status**: Production Ready  
**IPC**: Universal IPC v3.0  
**Primals**: 6/6 ecoBin v2.0 compliant  
**Tests**: 800+ passing  
**Security**: A++ LEGENDARY
