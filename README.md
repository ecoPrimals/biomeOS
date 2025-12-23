# 🌱 BiomeOS - The ecoPrimals Substrate

**BiomeOS is the orchestration layer that makes ecoPrimals accessible.**

It enables primal mixing, chimera creation, and niche deployment - allowing you to compose specialized organisms from the ecoPrimals ecosystem.

---

## 🎯 Quick Start

```bash
# 1. Pull primal binaries from parent repos
./bin/pull-primals.sh --all

# 2. List available chimeras
cargo run -p biomeos-cli --bin biomeos -- chimera list

# 3. Run a chimera
./bin/chimeras/p2p-secure

# 4. List showcases from parent primals
./bin/showcase-runner.sh list
```

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         BIOME OS                                │
│                    (Substrate / Orchestrator)                   │
└─────────────────────────────────────────────────────────────────┘
                               │
       ┌───────────────────────┼───────────────────────┐
       │                       │                       │
       ▼                       ▼                       ▼
┌─────────────┐         ┌─────────────┐         ┌─────────────┐
│   PRIMALS   │         │  CHIMERAS   │         │   NICHES    │
│  (Species)  │         │  (Hybrids)  │         │  (Biomes)   │
└─────────────┘         └─────────────┘         └─────────────┘
Standard single-        Mixed-boundary          Deployment
purpose organisms       amalgams                environments
```

### Primals (bin/primals/)

Standard organisms with clear boundaries:

| Primal | Purpose | Status |
|--------|---------|--------|
| 🐕 **BearDog** | Cryptography, identity, BTSP | ✅ Built |
| 🎼 **Songbird** | Discovery, orchestration, mesh | ✅ Built |
| 🍄 **ToadStool** | Compute, containers, GPU | ✅ Built |
| 🏰 **NestGate** | Storage, persistence | ✅ Built |
| 🐿️ **Squirrel** | AI routing, MCP, agents | ⏳ Pending |

### Chimeras (chimeras/)

Two levels of chimera fusion:

**Level 1: Orchestrated** - Multiple primals coordinated together

| Chimera | Components | Use Case |
|---------|------------|----------|
| 🔐 **p2p-secure** | BearDog + Songbird | Encrypted P2P mesh |
| 🎮 **gaming-mesh** | Songbird[] + BearDog + ToadStool | Gaming infrastructure |
| 🧠 **ml-pipeline** | ToadStool + NestGate + Squirrel | ML workflows |

**Level 2: Fused** - Deep genetic mixing into NEW primals

| Chimera | Genetics | Novel Capability |
|---------|----------|------------------|
| 🦆🦫 **platypus** | beardog-crypto + songbird-mesh | Genetic mesh discovery |

The platypus pattern: Not duck + beaver cooperating. A genuinely new species.

### Niches (niches/)

Complete environments where organisms operate:

- **gaming-tournament** - Deploy gaming-mesh chimera with anti-cheat
- **research-lab** - ML pipeline with persistent storage
- **web-development** - Standard web dev environment

---

## 📦 Directory Structure

```
biomeOS/
├── bin/                      # Runtime binaries
│   ├── primals/              # 56 primal binaries
│   ├── chimeras/             # Compiled chimera binaries
│   ├── pull-primals.sh       # Build primals from parent repos
│   └── showcase-runner.sh    # Run demos from parent primals
│
├── chimeras/                 # Chimera system
│   ├── definitions/          # YAML chimera specifications
│   └── registry/             # Chimera metadata
│
├── niches/                   # Niche (biome) system
│   ├── templates/            # BYOB templates
│   └── examples/             # Simple examples
│
├── crates/                   # Rust crates
│   ├── biomeos-types/        # Core type system
│   ├── biomeos-core/         # Business logic
│   ├── biomeos-cli/          # CLI interface
│   ├── biomeos-chimera/      # Chimera compiler
│   ├── biomeos-niche/        # Niche deployment
│   └── ...
│
├── docs/                     # Documentation
├── examples/                 # Example code
└── archive/                  # Historical status reports
```

---

## 🧬 Creating Chimeras

### Orchestrated vs Fused

**Orchestrated chimeras** (Level 1): YAML definitions, multiple processes
**Fused chimeras** (Level 2): Rust crates, single binary with mixed genetics

Choose fused when you need capabilities neither parent has alone.

### Orchestrated: Define a chimera

```yaml
# chimeras/definitions/my-chimera.yaml
chimera:
  id: "my-chimera"
  name: "My Custom Chimera"
  version: "1.0.0"

components:
  beardog:
    version: ">=2.0.0"
    modules:
      - name: "btsp"
        description: "Transport security"
  songbird:
    version: ">=2.0.0"
    modules:
      - name: "mesh"
        description: "Mesh networking"

fusion:
  encryption_layer:
    provider: "beardog.btsp"
    consumers: ["songbird.mesh"]
```

### 2. Build the chimera

```bash
cargo run -p biomeos-cli --bin biomeos -- chimera build my-chimera
```

### 3. Run it

```bash
./bin/chimeras/my-chimera
```

### Fused: Create a new primal from mixed genetics

```bash
# Create fused chimera directory
mkdir -p chimeras/fused/my-platypus/src
cd chimeras/fused/my-platypus
```

```toml
# Cargo.toml - Mix genetics from parents
[package]
name = "my-platypus"
version = "0.1.0"
edition = "2021"

[workspace]  # Standalone crate

[dependencies]
# From BearDog
beardog-crypto = { path = "../../../../beardog/crates/beardog-crypto" }

# From Songbird  
songbird-mesh = { path = "../../../../songbird/crates/songbird-mesh" }
```

```rust
// src/lib.rs - Fuse into novel capabilities
use beardog_crypto::GeneticKeys;
use songbird_mesh::MeshNode;

pub struct MyPlatypus {
    keys: GeneticKeys,
    mesh: MeshNode,
    // Novel capability that neither parent has alone
}
```

See `chimeras/fused/platypus/` for a complete example.

---

## 🌿 Deploying Niches

### 1. Create a niche template

```yaml
# niches/templates/my-biome.yaml
niche:
  id: "my-biome"
  name: "My Custom Biome"
  category: "development"

organisms:
  chimeras:
    secure_mesh:
      type: "p2p-secure"
      config:
        max_peers: 100
  primals:
    storage:
      type: "nestgate"
      config:
        volume_size: "10Gi"
```

### 2. Deploy

```bash
cargo run -p biomeos-cli --bin biomeos -- niche deploy my-biome
```

---

## 🛠️ CLI Commands

```bash
# Chimera management
biomeos chimera list              # List all chimeras
biomeos chimera show p2p-secure   # Show chimera details
biomeos chimera build gaming-mesh # Build chimera binary

# Niche management
biomeos niche list                # List niche templates
biomeos niche show gaming-tournament # Show niche details

# Primal management
biomeos primal list               # List installed primals
biomeos primal pull --all         # Build all from parent repos

# System commands
biomeos health                    # System health check
biomeos discover --capability compute  # Service discovery
```

---

## 🔧 Development

### Building

```bash
# Check all crates
cargo check --workspace

# Build CLI
cargo build -p biomeos-cli

# Run tests
cargo test --workspace
```

### Adding a new chimera

1. Create YAML definition in `chimeras/definitions/`
2. Run `biomeos chimera build <id>`
3. Test with `./bin/chimeras/<id>`

### Crate dependencies

```
biomeos-types       ← Foundation types
    ↓
biomeos-core        ← Business logic
    ↓
biomeos-chimera     ← Chimera compiler
biomeos-niche       ← Niche deployment
    ↓
biomeos-cli         ← CLI interface
```

---

## 🌐 ecoPrimals Ecosystem

BiomeOS is part of the Phase 2 "Memory & Attribution Layer":

| Project | Description |
|---------|-------------|
| **RhizoCrypt** | DAG engine for content-addressed storage |
| **LoamSpine** | Permanence layer with Merkle proofs |
| **SweetGrass** | Attribution and provenance tracking |
| **BiomeOS** | Orchestration substrate (you are here) |

Parent primals (Gen 1):
- **BearDog** - Genetic cryptography
- **Songbird** - Service mesh
- **ToadStool** - Universal compute
- **NestGate** - Distributed storage
- **Squirrel** - AI coordination

---

## 📊 Current Status

| Component | Count | Status |
|-----------|-------|--------|
| Primal binaries | 56 | ✅ Ready |
| Chimera definitions | 3 | ✅ Parseable |
| Compiled chimeras | 2 | ✅ Running |
| Niche templates | 6 | ✅ Available |
| Parent showcases | 26 | ✅ Accessible |

---

## 📄 License

Part of the ecoPrimals ecosystem.

---

*BiomeOS - Where primals evolve into chimeras, and chimeras thrive in niches.* 🌱
