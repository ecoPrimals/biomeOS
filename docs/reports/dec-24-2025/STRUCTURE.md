# BiomeOS Directory Structure

## Conceptual Model

```
                        ┌─────────────────────────────────────┐
                        │            BIOME OS                  │
                        │   Universal Capability Orchestrator  │
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
├── src/                          # 🎯 MAIN LIBRARY
│   ├── lib.rs                    # Universal adapter exports
│   ├── universal_adapter.rs      # Core adapter implementation
│   └── bin/                      # Binary entry points
│
├── crates/                       # 🦀 RUST CRATES (10 total)
│   ├── biomeos-types/            # Core type system (foundation)
│   │   └── src/
│   │       ├── config/           # Configuration types
│   │       ├── error/            # Error types with AI context
│   │       ├── health.rs         # Health status types
│   │       ├── primal/           # Primal types & capabilities
│   │       ├── service/          # Service types
│   │       └── manifest/         # Manifest types
│   │
│   ├── biomeos-core/             # Business logic & universal manager
│   │   └── src/
│   │       ├── universal_biomeos_manager/  # Core manager
│   │       ├── integration/      # Live service integration
│   │       ├── ai_first_api.rs   # AI-first response system
│   │       ├── byob.rs           # Build Your Own Biome
│   │       └── sovereignty_guardian.rs     # Data sovereignty
│   │
│   ├── biomeos-primal-sdk/       # Primal capabilities & SDK
│   ├── biomeos-cli/              # CLI interface with TUI
│   ├── biomeos-chimera/          # Chimera compiler & registry
│   ├── biomeos-niche/            # Niche deployment
│   ├── biomeos-ui/               # Desktop UI library
│   ├── biomeos-system/           # System integration
│   ├── biomeos-manifest/         # YAML parsing & validation
│   └── biomeos-federation/       # Federation support
│
├── ui/                           # 🖥️ DESKTOP UI
│   └── src/
│       ├── app.rs                # Main application (545 lines)
│       ├── types.rs              # UI type definitions
│       ├── desktop/              # Desktop mode components
│       │   ├── background.rs     # Grid rendering
│       │   ├── taskbar.rs        # System taskbar
│       │   ├── launcher.rs       # App launcher
│       │   ├── windows.rs        # Window management
│       │   └── notifications.rs  # System notifications
│       └── views/                # View implementations
│
├── chimeras/                     # 🧬 CHIMERA DEFINITIONS
│   ├── definitions/              # YAML specifications (3)
│   │   ├── p2p-secure.yaml       # Secure P2P mesh
│   │   ├── ml-pipeline.yaml      # ML training/inference
│   │   └── gaming-mesh.yaml      # Gaming infrastructure
│   │
│   ├── fused/                    # Fused chimeras (Level 2)
│   │   └── platypus/             # Example: BearDog + Songbird fusion
│   │
│   └── registry/                 # Chimera metadata
│
├── niches/                       # 🌿 NICHE (BIOME) CONFIGURATIONS
│   ├── templates/                # BYOB templates (6)
│   │   ├── gaming-tournament.yaml
│   │   ├── research-lab.yaml
│   │   ├── ai-research.yaml
│   │   ├── web-development.yaml
│   │   ├── federation-aware.yaml
│   │   └── custom-generic.yaml
│   │
│   └── examples/                 # Simple examples
│
├── examples/                     # 💡 WORKING EXAMPLES (8)
│   ├── simple_working_demo.rs
│   ├── config_builder_demo.rs
│   ├── biomeos_enhanced_demo.rs
│   ├── enhanced_functionality_demo.rs
│   ├── chimera_registry_demo.rs
│   ├── full_ecosystem_demo.rs
│   ├── working_unified_demo.rs
│   └── universal_biomeos_demo.rs
│
├── tests/                        # 🧪 INTEGRATION TESTS (8 files)
│   ├── common/mod.rs             # Shared test utilities
│   ├── chaos_testing.rs          # Chaos/resilience tests
│   ├── e2e_testing_suite.rs      # E2E tests
│   ├── health_monitoring_integration_tests.rs
│   ├── modern_e2e_tests.rs
│   ├── modern_integration_tests.rs
│   ├── modern_unit_tests.rs
│   └── simple_e2e_tests.rs
│
├── docs/                         # 📚 DOCUMENTATION
│   ├── README.md                 # Documentation hub
│   ├── adrs/                     # Architecture Decision Records
│   ├── api/                      # API documentation
│   ├── guides/                   # User guides
│   └── ecoSystemHandOff/         # Ecosystem integration docs
│
├── specs/                        # 📐 SPECIFICATIONS (25+)
│   ├── ARCHITECTURE_OVERVIEW.md
│   ├── BIOME_YAML_SPECIFICATION.md
│   ├── BYOB_BUILD_YOUR_OWN_BIOME_SPECIFICATION.md
│   ├── DIGITAL_SOVEREIGNTY_LICENSING.md
│   └── ...
│
├── bin/                          # 🔧 RUNTIME SCRIPTS
│   ├── pull-primals.sh           # Build primals from parent repos
│   └── showcase-runner.sh        # Run demos
│
├── archive/                      # 📦 HISTORICAL DOCUMENTS
│   └── status-reports/           # Archived development reports
│
├── README.md                     # Main documentation
├── STRUCTURE.md                  # This file
├── Cargo.toml                    # Workspace manifest
└── Cargo.lock                    # Dependency lock
```

## Key Concepts

### Primals

Standard organisms with **clear boundaries**. Each primal has a single purpose:

| Primal | Icon | Purpose | Status |
|--------|------|---------|--------|
| BearDog | 🐕 | Cryptography, identity, BTSP | ✅ Available |
| Songbird | 🎼 | Discovery, orchestration, mesh | ✅ Available |
| ToadStool | 🍄 | Compute, containers, GPU | ✅ Available |
| NestGate | 🏰 | Storage, persistence | ✅ Available |
| Squirrel | 🐿️ | AI routing, MCP, agents | ⏳ Pending |

### Chimeras

**Amalgams** that fuse components from multiple primals:

| Level | Type | Example | Description |
|-------|------|---------|-------------|
| 1 | Orchestrated | p2p-secure | Multiple processes coordinated |
| 2 | Fused | platypus | Single binary with mixed genetics |

### Niches

**Biomes** - complete environments where organisms operate:
- Define which organisms are present
- Configure interactions between organisms
- Set resource limits and networking
- Customizable via BYOB (Build Your Own Biome)

## Crate Dependency Graph

```
biomeos-types (foundation)
    │
    ├── biomeos-primal-sdk
    │
    ├── biomeos-core ─────────────┐
    │       │                     │
    │       ├── biomeos-chimera   │
    │       ├── biomeos-niche     │
    │       └── biomeos-ui        │
    │               │             │
    │               └─────────────┼── biomeos-cli
    │                             │
    ├── biomeos-manifest          │
    ├── biomeos-system            │
    └── biomeos-federation ───────┘
```

## Statistics

| Category | Count | Description |
|----------|-------|-------------|
| Rust crates | 10 | Core functionality |
| Working examples | 8 | Demo code |
| Integration test files | 8 | Test files |
| **Total test cases** | **134** | All passing |
| Line coverage | 44.7% | Via llvm-cov |
| Clippy warnings | 22 | Pedantic level |
| Chimera definitions | 3 | YAML specifications |
| Niche templates | 6 | BYOB configurations |
| Specifications | 25+ | Technical docs |

### Test Coverage by Crate

| Crate | Tests | Notes |
|-------|-------|-------|
| biomeos-types | 59 | Core types, health, capabilities |
| biomeos-core | 50 | Manager, BYOB, AI-first API |
| biomeos-chimera | 17 | Builder, registry, errors |
| biomeos-manifest | 8 | YAML parsing, validation |

## Workflow

### 1. Build the workspace

```bash
cargo build --workspace
```

### 2. Run tests

```bash
cargo test --lib --tests
```

### 3. Check coverage

```bash
cargo llvm-cov --workspace
```

### 4. Try an example

```bash
cargo run --example simple_working_demo
```

### 5. Define a Chimera

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
```

### 6. Create a Niche

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

## Code Quality Standards

- **Max file size**: 1000 lines (smart refactoring applied)
- **Clippy level**: Pedantic (22 warnings remaining)
- **Test coverage**: 44.7% (improving)
- **Unsafe code**: None in production paths
- **Hardcoded values**: Deprecated, use capability discovery
