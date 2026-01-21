# biomeOS - Neural API and Ecosystem Orchestration

**Version**: 3.0.0  
**Status**: 🚧 **Active Development** - Tower Atomic HTTP Evolution  
**Last Updated**: January 21, 2026

---

## 🌍 Overview

**biomeOS** is the orchestration layer for the ecoPrimals ecosystem. It provides:

1. **Neural API** - Capability-based mesh for primal discovery and deployment
2. **Graph-Based Deployment** - TOML-defined deployment graphs with DAG execution
3. **Capability Registry** - Event-driven discovery for instant primal lookups
4. **Environment Management** - Dynamic environment variable passing to primals
5. **Atomic Deployments** - BearDog + Songbird (Tower Atomic) for secure communication

---

## 🚨 CURRENT STATUS (January 21, 2026)

### ✅ Working

- ✅ Neural API capability registry (event-driven discovery)
- ✅ Graph deployment system with environment variables
- ✅ Tower Atomic architecture (BearDog crypto + Songbird networking)
- ✅ Unix socket JSON-RPC for inter-primal communication
- ✅ Squirrel AI orchestration (Tier 2 - local AI)

### 🚧 In Progress

- 🚧 **Tower Atomic HTTP Implementation** (BearDog + Songbird co-evolution)
  - **Blocker**: Songbird needs Pure Rust HTTP/HTTPS client with BearDog crypto
  - **Timeline**: 1-2 weeks
  - **Teams**: BearDog Team + Songbird Team
  - **See**: `HANDOFF_SONGBIRD_BEARDOG_TOWER_ATOMIC_HTTP_JAN_21_2026.md`

### ⏸️ Paused (Waiting for Tower Atomic)

- ⏸️ Squirrel external AI integration (Anthropic, OpenAI)
- ⏸️ End-to-end validation of Pure Rust HTTP delegation
- ⏸️ ecoBin compliance validation for networking stack

---

## 📁 Key Documentation

### Current Session

- **`HANDOFF_SONGBIRD_BEARDOG_TOWER_ATOMIC_HTTP_JAN_21_2026.md`** ⭐ (PRIMARY HANDOFF)
- `SESSION_SUMMARY_JAN_21_2026_TOWER_ATOMIC_BLOCKER.md` (Session overview)
- `TOWER_ATOMIC_HTTP_IMPLEMENTATION_BLOCKER_JAN_21_2026.md` (Technical details)
- `SESSION_BLOCKER_JAN_21_2026_TOWER_ATOMIC_HTTP.md` (Decision summary)

### Infrastructure Evolution

- `ENVIRONMENT_VARIABLES_WORKING_JAN_21_2026.md` (Neural API env var passing)
- `SQUIRREL_EVENT_DRIVEN_DISCOVERY_FIX_JAN_20_2026.md` (Discovery optimization)
- `NEURAL_API_ENVIRONMENT_VARIABLES_NEEDED_JAN_21_2026.md` (Requirements doc)

### Integration Work

- `SONGBIRD_SQUIRREL_INTEGRATION_COMPLETE_JAN_20_2026.md` (RPC integration)
- `SQUIRREL_ANTHROPIC_INTEGRATION_JAN_20_2026.md` (Architecture clarification)
- `SONGBIRD_V4_REHARVEST_COMPLETE_JAN_20_2026.md` (Songbird v4.3.0)

### Architecture

- `wateringHole/ECOBIN_ARCHITECTURE_STANDARD.md` (ecoBin definition)
- `wateringHole/PRIMAL_IPC_PROTOCOL.md` (IPC standard)
- `TOWER_ATOMIC_ARCHITECTURE.md` (Tower Atomic design)

---

## 🚀 Quick Start

### Deploy Neural API

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
cargo build --release -p biomeos-atomic-deploy
./target/release/neural-api-server
```

### Deploy a Graph

```bash
# Example: Deploy Tower Atomic + Squirrel
./target/release/neural-deploy \
  --graph-id tower_squirrel \
  --family-id nat0 \
  --socket /tmp/neural-api-nat0.sock
```

### Query Capabilities

```bash
echo '{
  "jsonrpc":"2.0",
  "method":"neural_api.discover_capability",
  "params":{"capability":"http.request"},
  "id":1
}' | nc -U /tmp/neural-api-nat0.sock | jq '.'
```

---

## 🏗️ Architecture

### Neural API Layers

```
┌─────────────────────────────────────────────────────────┐
│                   Neural API                            │
│          (Capability Mesh & Orchestration)              │
├─────────────────────────────────────────────────────────┤
│  - Capability Registry (event-driven discovery)         │
│  - Graph Execution Engine (TOML-based deployments)      │
│  - Environment Management (dynamic env vars)            │
│  - Health Monitoring (primal lifecycle)                 │
└─────────────────────────────────────────────────────────┘
                          ↕
┌─────────────────────────────────────────────────────────┐
│                Tower Atomic (BearDog + Songbird)        │
├─────────────────────────────────────────────────────────┤
│  BearDog (Pure Rust Crypto)  ◄──RPC──►  Songbird        │
│  - ed25519, x25519                   (TLS/HTTP)         │
│  - ChaCha20, BLAKE3                  - Unix sockets     │
│  - JWT, signatures                   - JSON-RPC         │
│                                      - 🚧 HTTP (in dev) │
└─────────────────────────────────────────────────────────┘
                          ↕
┌─────────────────────────────────────────────────────────┐
│                   Primal Ecosystem                      │
├─────────────────────────────────────────────────────────┤
│  - Squirrel (AI orchestration)                          │
│  - NestGate (IPC abstraction)                           │
│  - ToadStool (local AI)                                 │
│  - petalTongue (configuration)                          │
│  - sourDough (primal scaffolding)                       │
└─────────────────────────────────────────────────────────┘
```

### Graph-Based Deployment

Primals are deployed using TOML graphs:

```toml
[graph]
id = "tower_squirrel"
coordination = "Sequential"

[[nodes]]
id = "start-beardog"
primal = { by_capability = "security" }
capabilities = ["crypto.sign", "crypto.verify"]

[nodes.operation]
name = "start"
[nodes.operation.params]
mode = "server"
family_id = "nat0"

[[nodes]]
id = "start-songbird"
depends_on = ["start-beardog"]
primal = { by_capability = "discovery" }
capabilities = ["http.request", "discovery.announce"]

[nodes.operation.environment]
SONGBIRD_SECURITY_PROVIDER = "/tmp/beardog-nat0.sock"
```

---

## 📊 Project Structure

```
biomeOS/
├── crates/
│   ├── biomeos-atomic-deploy/    # Neural API server & executor
│   ├── biomeos-graph/             # Graph parsing & execution
│   ├── biomeos-ui/                # UI components (inactive)
│   └── biomeos-db/                # Database layer (inactive)
│
├── graphs/                        # Deployment graph definitions
│   ├── tower_squirrel.toml       # Tower Atomic + Squirrel
│   └── ...
│
├── wateringHole/                  # Standards & documentation
│   ├── ECOBIN_ARCHITECTURE_STANDARD.md
│   ├── PRIMAL_IPC_PROTOCOL.md
│   └── ...
│
├── HANDOFF_*.md                   # Team handoffs
├── SESSION_*.md                   # Session summaries
└── README.md                      # This file
```

---

## 🔬 Development

### Build

```bash
cargo build --release
```

### Test

```bash
cargo test --workspace
```

### Run

```bash
# Start Neural API
./target/release/neural-api-server

# Deploy a graph
./target/release/neural-deploy --graph-id <graph_id>
```

---

## 🎯 Roadmap

### Week 1-2 (Current)

- 🚧 BearDog: Implement TLS crypto RPC methods
- 🚧 Songbird: Implement Pure Rust HTTP/HTTPS client
- ✅ Neural API: Environment variable passing (complete)

### Week 3-4

- ⏳ Tower Atomic: Integration testing
- ⏳ Squirrel: Resume external AI integration
- ⏳ ecoBin: Validate cross-compilation

### Month 2

- ⏳ NestGate evolution (platform-agnostic IPC)
- ⏳ ToadStool integration (local AI)
- ⏳ petalTongue evolution (configuration management)

---

## 📚 Learn More

- **Architecture**: See `wateringHole/` for standards
- **Handoffs**: See `HANDOFF_*.md` for team coordination
- **Sessions**: See `SESSION_*.md` for progress logs
- **Integration**: See individual primal docs in `phase1/`

---

## 🤝 Contributing

This is an active, evolving ecosystem. Key principles:

1. **Pure Rust**: Zero C dependencies
2. **TRUE PRIMAL**: Self-knowledge only, discover at runtime
3. **ecoBin Compliance**: Cross-compiles everywhere
4. **Capability-Based**: No hardcoding, discover via capabilities
5. **Event-Driven**: No blocking I/O, use async/await

---

## 📞 Contact

**Current Status**: See session summaries in root  
**Blockers**: See `SESSION_BLOCKER_*.md` files  
**Handoffs**: See `HANDOFF_*.md` files

---

**🌍 biomeOS: Orchestrating the Pure Rust Primal Ecosystem 🦀**

---

*Last Updated: January 21, 2026*  
*Version: 3.0.0*  
*Status: Tower Atomic HTTP Evolution in Progress*
