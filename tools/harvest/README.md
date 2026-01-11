# 🌾 biomeOS Harvest System

**Modern Rust-based primal binary harvesting** - Evolution toward NUCLEUS

## Purpose

Automate the harvesting (building, collecting, storing) of primal binaries from various sources.

## Features

### ✅ Phase 1 (Current)
- Harvest from local `phase1/` directories
- Build primals in release mode
- Store in `plasmidBin/`
- Track versions and provenance
- Clean old versions
- Verify binary integrity (SHA256)

### 🚧 Phase 2 (Coming Soon)
- Pull from GitHub releases
- Support multiple architectures
- Binary caching
- Differential updates

### 🔮 Phase 3 (NUCLEUS)
- Distributed primal registry
- Automated updates
- Capability-based selection
- Version compatibility checking
- Genetic lineage verification

## Usage

### Harvest single primal from local:
```bash
biomeos-harvest local --primal songbird
```

### Harvest all primals:
```bash
biomeos-harvest all
```

### List harvested primals:
```bash
biomeos-harvest list --verbose
```

### Clean old versions:
```bash
biomeos-harvest clean --keep-latest 2
```

### Future: Harvest from GitHub:
```bash
biomeos-harvest github --repo ecoPrimals/songbird --version v3.20.0
```

## Binary Names (Simple & Multi-functional)

All primals use simple, one-word names:

- `beardog` - Security & cryptography
- `songbird` - Discovery & federation
- `toadstool` - Universal compute
- `nestgate` - Storage & persistence
- `squirrel` - AI & intelligence
- `petaltongue` - Universal UI

## Architecture

```
biomeOS/
├── tools/harvest/          # This tool (Rust)
│   ├── src/main.rs         # Harvest logic
│   └── Cargo.toml          # Dependencies
│
├── plasmidBin/             # Harvested binaries (target)
│   ├── beardog             # Harvested binary
│   ├── beardog.manifest.toml  # Provenance manifest
│   ├── songbird
│   ├── songbird.manifest.toml
│   └── ...
│
└── ../phase1/              # Source repositories
    ├── songbird/           # Local primal source
    ├── beardog/
    └── ...
```

## Manifest Format

Each harvested binary has a `.manifest.toml` file:

```toml
name = "songbird"
version = "v3.20.0"
sha256 = "abc123..."
harvested_at = "2026-01-10T20:30:00Z"

[source]
type = "local"
path = "../phase1/songbird"

# Or for GitHub:
# [source]
# type = "github"
# repo = "ecoPrimals/songbird"
# tag = "v3.20.0"
```

## Evolution Path

```
Phase 1: Local Harvest      ← We are here
    ↓
Phase 2: GitHub Integration  ← Next
    ↓
Phase 3: NUCLEUS             ← Future
    • Distributed registry
    • Automated updates
    • Capability discovery
    • Genetic verification
```

## Why Rust (Not Shell)?

**From "jelly script" to modern idiomatic Rust:**

- ✅ Type safety
- ✅ Error handling (Result<T>)
- ✅ Cross-platform
- ✅ Better testing
- ✅ Maintainable
- ✅ Fast
- ✅ Part of biomeOS ecosystem

## Integration with Niches

Niches (nest, tower, node) reference simple binary names:

```toml
[[primals]]
binary = "songbird"  # Will be found in plasmidBin/ or PATH
```

The harvest system ensures these binaries are available and up-to-date.

## Building

```bash
cd tools/harvest
cargo build --release
```

## Testing

```bash
# List current primals
./target/release/biomeos-harvest list

# Harvest songbird
./target/release/biomeos-harvest local --primal songbird

# Harvest all
./target/release/biomeos-harvest all
```

## Future: NUCLEUS Integration

This harvest system is the foundation for NUCLEUS - the distributed primal management system:

- **Registry**: Track all available primals
- **Discovery**: Find primals by capability
- **Updates**: Automated version management
- **Verification**: Genetic lineage + signatures
- **Federation**: Share primals across the ecosystem

---

**Status**: Phase 1 complete - Local harvesting operational  
**Next**: GitHub integration for remote primal sources  
**Vision**: NUCLEUS - Automated primal lifecycle management

