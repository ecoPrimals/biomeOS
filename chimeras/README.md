# 🧬 Chimera System

Chimeras are organisms formed by mixing genetic material from multiple primals.

## Two Levels of Fusion

### Level 1: Orchestrated Chimeras (Symbiosis)
Multiple primals running together, coordinated by BiomeOS.

```
┌─────────────┐     ┌─────────────┐
│   BearDog   │◄───►│   Songbird  │
│  (process)  │     │  (process)  │
└─────────────┘     └─────────────┘
       │                   │
       └───────┬───────────┘
               │
       ┌───────▼───────┐
       │  Orchestrator │
       │  (p2p-secure) │
       └───────────────┘
```

**Use when:** You want existing primals to work together
**Example:** `p2p-secure` orchestrates BearDog + Songbird processes

### Level 2: Fused Chimeras (True Genetics)
A **new primal** built by mixing crates from multiple parent primals.

```
┌─────────────────────────────────────┐
│            PLATYPUS                 │
│  ┌─────────────┬─────────────┐     │
│  │ beardog-    │ songbird-   │     │
│  │ crypto      │ mesh        │     │
│  └─────────────┴─────────────┘     │
│         (single binary)             │
└─────────────────────────────────────┘
```

**Use when:** You're creating something genuinely new
**Example:** A custom primal that deeply integrates cryptography with networking

---

## Deep Fusion: Creating New Primals

### The Platypus Pattern

Nature creates weird niches. The platypus isn't a duck + beaver - it's a genuinely new species with mixed genetics.

Similarly, a **fused chimera** isn't BearDog + Songbird running together - it's a new primal that:
- Uses `beardog-crypto` crate directly
- Uses `songbird-mesh` crate directly
- Creates novel behavior impossible with orchestration alone

### Scaffold a Fused Chimera

```bash
# Generate a new fused primal from parent genetics
biomeos chimera scaffold --fused \
    --name platypus \
    --from beardog:crypto,identity \
    --from songbird:mesh,discovery \
    --niche "aquatic-secure-mesh"
```

This creates:

```
chimeras/fused/platypus/
├── Cargo.toml          # Dependencies on parent crates
├── src/
│   ├── lib.rs          # Fused genetics
│   ├── crypto.rs       # From beardog
│   ├── mesh.rs         # From songbird
│   └── platypus.rs     # Novel integration
└── README.md           # Niche documentation
```

### Example: Platypus Cargo.toml

```toml
[package]
name = "platypus"
version = "0.1.0"
edition = "2021"
description = "Aquatic secure mesh - fused from BearDog + Songbird genetics"

[dependencies]
# Parent genetics
beardog-crypto = { path = "../../../../beardog/crates/beardog-crypto" }
beardog-identity = { path = "../../../../beardog/crates/beardog-identity" }
songbird-mesh = { path = "../../../../songbird/crates/songbird-mesh" }
songbird-discovery = { path = "../../../../songbird/crates/songbird-discovery" }

# Own mutations
tokio = { version = "1.0", features = ["full"] }
```

### Example: Fused Code

```rust
//! Platypus - A fused chimera
//! 
//! Not BearDog + Songbird running together.
//! A genuinely new organism with mixed genetics.

use beardog_crypto::{GeneticKeys, BTSP};
use beardog_identity::DID;
use songbird_mesh::{MeshTopology, PeerConnection};
use songbird_discovery::ServiceRegistry;

/// A platypus node - impossible with orchestration alone
pub struct Platypus {
    /// Genetic keys (from BearDog)
    keys: GeneticKeys,
    
    /// Identity (from BearDog)  
    did: DID,
    
    /// Mesh topology (from Songbird)
    mesh: MeshTopology,
    
    /// Service registry (from Songbird)
    registry: ServiceRegistry,
    
    /// NOVEL: Encrypted mesh state that evolves
    /// This doesn't exist in either parent!
    genetic_mesh_state: EncryptedEvolvingState,
}

impl Platypus {
    /// Novel behavior: Genetic mesh discovery
    /// 
    /// Peers are discovered through Songbird, but trust is
    /// established through BearDog's genetic lineage verification.
    /// Neither parent can do this alone.
    pub async fn genetic_discover(&self) -> Vec<TrustedPeer> {
        let candidates = self.registry.discover_peers().await;
        
        candidates.into_iter()
            .filter(|peer| self.keys.verify_lineage(&peer.did))
            .map(|peer| TrustedPeer::from_verified(peer, &self.keys))
            .collect()
    }
}
```

---

## Directory Structure

```
chimeras/
├── definitions/              # Level 1: Orchestrated chimeras (YAML)
│   ├── p2p-secure.yaml
│   ├── gaming-mesh.yaml
│   └── ml-pipeline.yaml
│
├── fused/                    # Level 2: Fused chimeras (Rust crates)
│   ├── platypus/             # Example: BearDog + Songbird genetics
│   ├── mycorrhiza/           # Example: NestGate + Squirrel + RhizoCrypt
│   └── tardigrade/           # Example: ToadStool + extreme resilience
│
├── registry/                 # Metadata for all chimeras
│   └── chimera-registry.yaml
│
└── README.md                 # This file
```

---

## When to Use Which

| Scenario | Level | Example |
|----------|-------|---------|
| Quick integration of existing primals | Orchestrated | p2p-secure |
| Need novel behavior from mixed genetics | Fused | platypus |
| Unknown niche, experimenting | Fused | (nature will tell you) |
| Production deployment of known pattern | Orchestrated | gaming-mesh |
| Research, weird edge cases | Fused | single-celled bouncy ball |

---

## The Biological Metaphor

**Primals** = Species (BearDog, Songbird, ToadStool)
**Orchestrated Chimeras** = Symbiosis (clownfish + anemone)
**Fused Chimeras** = Hybrid species (platypus, ligers)
**Niches** = Ecosystems where organisms thrive

Nature doesn't know the niche ahead of time. Evolution experiments.
BiomeOS lets you experiment too.

---

## Creating a Fused Chimera

### 1. Identify parent genetics

```bash
# List available crates from each primal
ls ../beardog/crates/
ls ../songbird/crates/
ls ../toadstool/crates/
```

### 2. Create the fused crate

```bash
mkdir -p chimeras/fused/my-chimera/src
cd chimeras/fused/my-chimera
```

### 3. Define dependencies

```toml
# Cargo.toml
[package]
name = "my-chimera"
version = "0.1.0"
edition = "2021"

[dependencies]
# Pick the genetics you need
beardog-crypto = { path = "../../../../beardog/crates/beardog-crypto" }
songbird-mesh = { path = "../../../../songbird/crates/songbird-mesh" }
```

### 4. Write the fusion

```rust
// src/lib.rs
use beardog_crypto::*;
use songbird_mesh::*;

pub struct MyChimera {
    // Mix the genetics...
}
```

### 5. Build and deploy

```bash
cargo build --release
cp target/release/my-chimera ../../bin/chimeras/
```

---

## Future: Genetic Templates

Eventually, common fusion patterns will become templates:

```bash
# Create a "secure-mesh" type chimera
biomeos chimera new --template secure-mesh --name my-secure-net

# Create a "persistent-compute" type chimera  
biomeos chimera new --template persistent-compute --name my-ml-runner

# Create from scratch for unknown niche
biomeos chimera new --blank --name experiment-x
```

The weird niches are where innovation happens. 🦆🦫
