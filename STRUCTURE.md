# BiomeOS Directory Structure

## Conceptual Model

```
                        ┌─────────────────────────────────────┐
                        │            BIOME OS                  │
                        │    (Substrate / Orchestrator)        │
                        └─────────────────────────────────────┘
                                        │
          ┌─────────────────────────────┼─────────────────────────────┐
          │                             │                             │
          ▼                             ▼                             ▼
   ┌─────────────┐              ┌─────────────┐              ┌─────────────┐
   │   PRIMALS   │              │  CHIMERAS   │              │   NICHES    │
   │  (Species)  │              │  (Hybrids)  │              │  (Biomes)   │
   └─────────────┘              └─────────────┘              └─────────────┘
   Standard single-             Mixed-boundary               Environments
   purpose organisms            amalgams                     for deployment
```

## Directory Layout

```
biomeOS/
│
├── bin/                          # 🔧 RUNTIME BINARIES
│   ├── primals/                  # Standard primal binaries (56 total)
│   │   ├── beardog               # Crypto & identity (4.5M)
│   │   ├── songbird-*            # Orchestration (15 binaries)
│   │   ├── toadstool-*           # Compute (39 binaries)
│   │   └── nestgate              # Storage (3.4M)
│   │
│   ├── chimeras/                 # Compiled chimera orchestrators
│   │   ├── p2p-secure            # BearDog + Songbird
│   │   ├── gaming-mesh           # Songbird[] + BearDog + ToadStool
│   │   └── src/                  # Source code for chimeras
│   │
│   ├── pull-primals.sh           # Build primals from parent repos
│   └── showcase-runner.sh        # Run demos from parent primals
│
├── chimeras/                     # 🧬 CHIMERA DEFINITIONS
│   ├── definitions/              # YAML specifications
│   │   ├── p2p-secure.yaml       # Secure P2P mesh
│   │   ├── ml-pipeline.yaml      # ML training/inference
│   │   └── gaming-mesh.yaml      # Gaming infrastructure (arrays!)
│   │
│   ├── registry/                 # Chimera metadata
│   └── README.md                 # Chimera documentation
│
├── niches/                       # 🌿 NICHE (BIOME) CONFIGURATIONS
│   ├── templates/                # BYOB templates (6 total)
│   │   ├── gaming-tournament.yaml
│   │   ├── research-lab.yaml
│   │   ├── ai-research.yaml
│   │   ├── web-development.yaml
│   │   ├── federation-aware.yaml
│   │   └── custom-generic.yaml
│   │
│   ├── examples/                 # Simple examples
│   │   └── minimal-p2p.yaml
│   │
│   └── README.md                 # Niche documentation
│
├── crates/                       # 🦀 RUST CRATES
│   ├── biomeos-types/            # Core type system (foundation)
│   ├── biomeos-core/             # Business logic
│   ├── biomeos-cli/              # CLI interface
│   ├── biomeos-chimera/          # Chimera compiler
│   ├── biomeos-niche/            # Niche deployment
│   ├── biomeos-primal-sdk/       # Primal integration
│   ├── biomeos-manifest/         # YAML parsing
│   ├── biomeos-ui/               # TUI/Web interface
│   ├── biomeos-federation/       # Federation support
│   └── biomeos-system/           # System integration
│
├── docs/                         # 📚 DOCUMENTATION
│   ├── adrs/                     # Architecture Decision Records
│   ├── api/                      # API documentation
│   └── guides/                   # User guides
│
├── examples/                     # 💡 EXAMPLE CODE
│   ├── chimera_registry_demo.rs
│   ├── full_ecosystem_demo.rs
│   └── *.yaml                    # Example manifests
│
├── templates/                    # 📋 BIOME TEMPLATES (legacy)
│   └── *.yaml                    # Various biome examples
│
├── archive/                      # 📦 HISTORICAL DOCUMENTS
│   └── status-reports/           # 39 archived development reports
│
├── README.md                     # Main documentation
├── STRUCTURE.md                  # This file
└── Cargo.toml                    # Workspace manifest
```

## Key Concepts

### Primals

Standard organisms with **clear boundaries**. Each primal has a single purpose:

| Primal | Icon | Purpose | Binaries |
|--------|------|---------|----------|
| BearDog | 🐕 | Cryptography, identity, BTSP | 1 |
| Songbird | 🎼 | Discovery, orchestration, mesh | 15 |
| ToadStool | 🍄 | Compute, containers, GPU | 39 |
| NestGate | 🏰 | Storage, persistence | 1 |
| Squirrel | 🐿️ | AI routing, MCP, agents | (pending) |

### Chimeras

**Amalgams** that fuse components from multiple primals:

| Chimera | Components | Use Case |
|---------|------------|----------|
| p2p-secure | BearDog + Songbird | Encrypted P2P mesh |
| ml-pipeline | ToadStool + NestGate + Squirrel | ML workflows |
| gaming-mesh | Songbird[] + BearDog + ToadStool | Gaming (with arrays!) |

Chimeras are:
1. Defined in YAML (`chimeras/definitions/`)
2. Compiled into Rust code
3. Built into binaries (`bin/chimeras/`)
4. Deployed within niches

### Niches

**Biomes** - complete environments where organisms operate:

- Define which organisms (primals + chimeras) are present
- Configure interactions between organisms
- Set resource limits and networking
- Customizable via BYOB (Build Your Own Biome)

## Workflow

### 1. Pull Primals

```bash
./bin/pull-primals.sh --all
# Builds 56 binaries from parent ecoPrimal repos
```

### 2. Define a Chimera

```yaml
# chimeras/definitions/my-chimera.yaml
chimera:
  id: "my-chimera"
  name: "My Custom Chimera"
  
components:
  beardog:
    modules: ["btsp", "identity"]
  songbird:
    modules: ["discovery", "mesh"]
    
fusion:
  encryption_layer:
    provider: "beardog.btsp"
    consumers: ["songbird.mesh"]
```

### 3. Build the Chimera

```bash
cargo run -p biomeos-cli --bin biomeos -- chimera build my-chimera
```

### 4. Create a Niche

```yaml
# niches/templates/my-biome.yaml
niche:
  id: "my-biome"
  
organisms:
  chimeras:
    secure_mesh:
      type: "p2p-secure"
  primals:
    storage:
      type: "nestgate"
```

### 5. Deploy

```bash
cargo run -p biomeos-cli --bin biomeos -- niche deploy my-biome
```

## Crate Dependency Graph

```
biomeos-types (foundation)
    │
    ├── biomeos-core
    │       │
    │       ├── biomeos-chimera
    │       ├── biomeos-niche
    │       └── biomeos-primal-sdk
    │               │
    │               └── biomeos-cli
    │
    ├── biomeos-manifest
    ├── biomeos-ui
    ├── biomeos-federation
    └── biomeos-system
```

## File Statistics

| Category | Count | Description |
|----------|-------|-------------|
| Primal binaries | 56 | Built from 4 parent repos |
| Chimera definitions | 3 | YAML specifications |
| Compiled chimeras | 2 | Ready to run |
| Niche templates | 6 | BYOB configurations |
| Rust crates | 10 | Core functionality |
| Examples | 14+ | Demo code |
| Archived reports | 39 | Historical documents |
