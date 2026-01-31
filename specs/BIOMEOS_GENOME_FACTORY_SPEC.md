# biomeOS Genome Factory Specification
**Version**: 1.0.0  
**Status**: Design Complete  
**Role**: Universal genomeBin Production System  
**Date**: January 31, 2026

---

## Overview

The **biomeOS Genome Factory** is the cellular machinery of the ecoPrimals ecosystem - a system that produces genomeBin wrappers for ANY primal binary, enables fractal atomic composition, and provides self-replication capabilities.

---

## Biological Metaphor

| Biology | ecoPrimals | Function |
|---------|------------|----------|
| **DNA** | Source code | Information blueprint |
| **RNA** | Compiled binary | Transportable code |
| **DNA Replicase** | **biomeOS** | Produces genomeBins |
| **Ribosome** | genomeBin v3.0 engine | Deploys/executes |
| **Protein** | Running primal | Active service |
| **Mitosis** | Self-replication | biomeOS copies itself |
| **Horizontal Gene Transfer** | Federation | Genome exchange |

**biomeOS is the DNA Replicase** - the enzyme that replicates and produces genomes.

---

## Core Capabilities

### 1. Universal genomeBin Production

**Any primal binary → biomeOS → genomeBin**

```
Input:
  - Primal binaries (x86_64, aarch64, etc.)
  - Metadata (name, version, capabilities)
  
Processing:
  - Compress with zstd
  - Generate SHA256 checksums
  - Create manifest
  - Embed in genomeBin v3.0 format
  
Output:
  - Self-extracting genomeBin binary
  - Works on any platform
  - Fractal composition ready
```

### 2. Fractal Atomic Composition

**Individual genomes → Atomic genomes**

```
TOWER = beardog.genome + songbird.genome
NODE  = TOWER + toadstool.genome
NEST  = TOWER + nestgate.genome
NUCLEUS = TOWER + NODE + NEST (all 5 primals)
```

### 3. Self-Replication

**biomeOS can create its own genomeBin**

- Introspects own binary
- Creates `biomeos-self.genome`
- Enables bootstrap to bare-metal
- Autonomous ecosystem reproduction

### 4. Federation Genome Exchange

**biomeOS instances exchange genomes**

- P2P discovery via Songbird
- Secure transfer via BearDog
- Genetic lineage verification
- Automatic storage in plasmidBin/

---

## Architecture

### Component Overview

```
┌─────────────────────────────────────────────────┐
│                    biomeOS                      │
│                                                 │
│  ┌──────────────────────────────────────────┐  │
│  │   biomeos-genomebin-v3                   │  │
│  │   (Core engine - embedded)               │  │
│  │   - GenomeBin struct                     │  │
│  │   - Builder API                          │  │
│  │   - Composer API                         │  │
│  │   - Runtime stub                         │  │
│  └──────────────────────────────────────────┘  │
│                      ↓                          │
│  ┌──────────────────────────────────────────┐  │
│  │   biomeos-genome-factory                 │  │
│  │   (Orchestration layer)                  │  │
│  │   - GenomeFactory API                    │  │
│  │   - Atomic composition                   │  │
│  │   - Self-replication                     │  │
│  │   - Federation exchange                  │  │
│  └──────────────────────────────────────────┘  │
│                      ↓                          │
│  ┌──────────────────────────────────────────┐  │
│  │   neuralAPI (REST endpoints)             │  │
│  │   - POST /genome/create                  │  │
│  │   - POST /genome/compose                 │  │
│  │   - POST /genome/self-replicate          │  │
│  │   - GET  /genome/:id/verify              │  │
│  │   - POST /genome/request (federation)    │  │
│  └──────────────────────────────────────────┘  │
│                      ↓                          │
│  ┌──────────────────────────────────────────┐  │
│  │   plasmidBin/ (genome storage)           │  │
│  │   - Individual primal genomes            │  │
│  │   - Atomic genomes (TOWER, NODE, NEST)   │  │
│  │   - NUCLEUS genome (complete ecosystem)  │  │
│  │   - Federation cache                     │  │
│  └──────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

### Crate Structure

```
biomeOS/
├── crates/
│   ├── biomeos-genomebin-v3/      # Core genomeBin engine
│   │   ├── src/
│   │   │   ├── lib.rs             # GenomeBin struct
│   │   │   ├── builder.rs         # Build API
│   │   │   ├── composer.rs        # Compose atomics
│   │   │   ├── runtime.rs         # Extract & run
│   │   │   └── verify.rs          # Checksums
│   │   └── stub/
│   │       └── main.rs            # Runtime stub
│   │
│   ├── biomeos-genome-factory/    # Factory orchestration
│   │   └── src/
│   │       ├── lib.rs             # GenomeFactory
│   │       ├── create.rs          # Create genomes
│   │       ├── compose.rs         # Compose atomics
│   │       ├── replicate.rs       # Self-replication
│   │       └── federation.rs      # Peer exchange
│   │
│   └── biomeos-atomic-deploy/     # Enhanced
│       └── src/
│           └── genome_routes.rs   # neuralAPI endpoints
```

---

## REST API Specification

### Endpoint: Create genomeBin

**POST /genome/create**

Create genomeBin from binary files.

**Request**:
```json
{
  "name": "beardog",
  "binaries": {
    "x86_64": "/path/to/beardog-x86_64",
    "aarch64": "/path/to/beardog-aarch64"
  },
  "metadata": {
    "version": "0.9.0",
    "description": "BearDog Security Primal",
    "nucleus_atomic": "TOWER-component",
    "capabilities": ["encryption", "identity", "hsm"]
  }
}
```

**Response**:
```json
{
  "genome_id": "beardog-0.9.0",
  "path": "plasmidBin/beardog.genome",
  "size": 3145728,
  "architectures": ["x86_64", "aarch64"],
  "checksums": {
    "x86_64": "ec34f3d0e267c29085b8ba2153b5c2f2f8974c1db80f8c3f2ba6e5d1f1ed4083",
    "aarch64": "3e83f0c75c010c9c91521375ffb485c3839eb5a1403668a2733d665a882fa508"
  }
}
```

---

### Endpoint: Compose Atomic

**POST /genome/compose**

Compose atomic genomeBin from individual genomes.

**Request**:
```json
{
  "name": "tower",
  "nucleus_type": "TOWER",
  "genomes": [
    "plasmidBin/beardog.genome",
    "plasmidBin/songbird.genome"
  ]
}
```

**Response**:
```json
{
  "genome_id": "tower-atomic",
  "path": "plasmidBin/tower.genome",
  "size": 33554432,
  "embedded_genomes": ["beardog", "songbird"],
  "nucleus_type": "TOWER"
}
```

---

### Endpoint: Self-Replicate

**POST /genome/self-replicate**

biomeOS creates its own genomeBin.

**Request**: *(empty body)*

**Response**:
```json
{
  "genome_id": "biomeos-self",
  "path": "plasmidBin/biomeos-self.genome",
  "size": 5242880,
  "architectures": ["x86_64"],
  "capabilities": ["orchestration", "genome-factory", "federation"]
}
```

---

### Endpoint: Verify genomeBin

**GET /genome/:id/verify**

Verify integrity of genomeBin.

**Response**:
```json
{
  "genome_id": "beardog",
  "valid": true,
  "checksums": {
    "x86_64": {
      "expected": "ec34f3d0...",
      "actual": "ec34f3d0...",
      "valid": true
    },
    "aarch64": {
      "expected": "3e83f0c7...",
      "actual": "3e83f0c7...",
      "valid": true
    }
  },
  "embedded_genomes": [],
  "manifest_valid": true
}
```

---

### Endpoint: Request from Peer

**POST /genome/request**

Request genomeBin from peer biomeOS instance.

**Request**:
```json
{
  "primal": "custom-workload",
  "peer": "remote-biomeos.example.com",
  "verify_lineage": true
}
```

**Response**:
```json
{
  "genome_id": "custom-workload",
  "path": "plasmidBin/custom-workload.genome",
  "size": 10485760,
  "peer": "remote-biomeos.example.com",
  "lineage_verified": true,
  "transferred_at": "2026-01-31T12:00:00Z"
}
```

---

## CLI Interface

### Command: create

Create genomeBin from binaries.

```bash
biomeos genome create <name> \
  --x86-64 <path> \
  --aarch64 <path> \
  [--nucleus <type>] \
  [--version <ver>] \
  [--output <path>]

# Example
biomeos genome create beardog \
  --x86-64 /tmp/beardog-x86 \
  --aarch64 /tmp/beardog-arm \
  --nucleus TOWER-component \
  --version 0.9.0
```

### Command: compose

Compose atomic genomeBin.

```bash
biomeos genome compose <name> \
  --add <genome-path> \
  --add <genome-path> \
  --nucleus-type <type> \
  [--output <path>]

# Example
biomeos genome compose tower \
  --add plasmidBin/beardog.genome \
  --add plasmidBin/songbird.genome \
  --nucleus-type TOWER
```

### Command: self-replicate

Create biomeOS's own genomeBin.

```bash
biomeos genome self-replicate

# Output: plasmidBin/biomeos-self.genome
```

### Command: verify

Verify genomeBin integrity.

```bash
biomeos genome verify <genome-id>

# Example
biomeos genome verify beardog

# Output:
# ✅ beardog.genome is valid
# ✅ x86_64: ec34f3d0... OK
# ✅ aarch64: 3e83f0c7... OK
```

### Command: request

Request genome from peer.

```bash
biomeos genome request <primal> \
  --peer <biomeos-instance> \
  [--verify-lineage]

# Example
biomeos genome request custom-app \
  --peer usb-biomeos.local \
  --verify-lineage
```

---

## Federation Protocol

### Discovery

1. **Songbird mDNS/STUN**: Discover peer biomeOS instances
2. **BearDog handshake**: Establish encrypted channel
3. **Lineage verification**: Validate genetic trust

### Request Flow

```
Client biomeOS                    Peer biomeOS
      │                                │
      ├──(1) Discover via Songbird────►│
      │◄────(2) Peer info──────────────┤
      │                                │
      ├──(3) BearDog handshake────────►│
      │◄────(4) Encrypted channel──────┤
      │                                │
      ├──(5) Request genome───────────►│
      │     { "primal": "name" }       │
      │◄────(6) genomeBin data─────────┤
      │                                │
      ├──(7) Verify checksums──────────┤
      ├──(8) Verify lineage────────────┤
      │                                │
      └──(9) Store in plasmidBin/──────┘
```

### Security

- ✅ All transfers via BearDog encryption
- ✅ Genetic lineage verification
- ✅ SHA256 checksum validation
- ✅ Optional signature verification (v3.1)

---

## Use Cases

### 1. Developer Workflow

```bash
# Developer builds primal
cd ~/projects/my-primal
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu

# Create genomeBin via biomeOS
biomeos genome create my-primal \
  --x86-64 target/x86_64/release/my-primal \
  --aarch64 target/aarch64/release/my-primal

# Result: plasmidBin/my-primal.genome ready!
```

### 2. Atomic Deployment

```bash
# Compose TOWER atomic
biomeos genome compose tower \
  --add plasmidBin/beardog.genome \
  --add plasmidBin/songbird.genome

# Deploy to device
scp plasmidBin/tower.genome pixel:/tmp/
ssh pixel './tower.genome'

# Both primals deployed atomically!
```

### 3. Ecosystem Bootstrap

```bash
# biomeOS self-replicates
biomeos genome self-replicate

# Bootstrap new device
scp plasmidBin/biomeos-self.genome server:/tmp/
ssh server './biomeos-self.genome'

# New biomeOS running, can produce genomes!
```

### 4. Federation Sync

```bash
# Pixel requests genome from USB
ssh pixel 'biomeos genome request gpu-workload \
  --peer usb-biomeos.local \
  --verify-lineage'

# Genome transferred securely
ssh pixel './plasmidBin/gpu-workload.genome'
```

---

## Implementation Phases

### Phase 1: Core Integration (2 weeks)
- [ ] Integrate genomeBin v3.0 engine
- [ ] Create genome-factory crate
- [ ] Add neuralAPI endpoints
- [ ] Implement CLI commands
- [ ] Unit tests

### Phase 2: Self-Replication (1 week)
- [ ] Implement introspection
- [ ] Create biomeos.genome
- [ ] Test bootstrap scenarios
- [ ] Document self-deployment

### Phase 3: Atomic Composition (1 week)
- [ ] Implement fractal composition
- [ ] Create TOWER/NODE/NEST genomes
- [ ] Test atomic deployment
- [ ] Validate lineage

### Phase 4: Federation (2 weeks)
- [ ] Implement peer discovery
- [ ] Add genome request/response
- [ ] Secure transfer via BearDog
- [ ] Lineage verification
- [ ] Integration tests

### Phase 5: Production (1 week)
- [ ] Signature support (Ed25519)
- [ ] Delta updates
- [ ] Rollback mechanisms
- [ ] Performance optimization
- [ ] Documentation

**Total**: 7 weeks

---

## Quality Metrics

### Code Quality
- ✅ 100% Pure Rust
- ✅ Zero unsafe code
- ✅ Comprehensive error handling
- ✅ Full test coverage (>90%)

### Deep Debt Compliance
- ✅ No external dependencies (for core)
- ✅ Runtime discovery (no hardcoding)
- ✅ Capability-based
- ✅ Platform-agnostic

### Performance
- Target: <200ms genome creation
- Target: <100ms verification
- Target: <50ms query
- Target: <2s federation transfer (1MB)

---

## References

- **genomeBin v3.0**: `specs/GENOMEBIN_V3_SPECIFICATION.md`
- **Architecture Design**: `docs/architecture/BIOMEOS_GENOME_FACTORY.md`
- **Evolution Doc**: `docs/evolution/GENOMEBIN_V3_BINARY_ISOMORPHIC.md`
- **API Reference**: `crates/biomeos-genome-factory/README.md`

---

## Changelog

### v1.0.0 (2026-01-31) - Design Complete
- Complete specification
- REST API design
- CLI interface design
- Federation protocol
- Implementation roadmap

---

**Status**: 🚀 **Design Complete - Implementation Ready**  
**Timeline**: 7 weeks for complete implementation  
**Impact**: 🔥 Self-Replicating Autonomous Ecosystem
