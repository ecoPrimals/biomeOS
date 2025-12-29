# Capability-Based Deployment

**Agnostic orchestration: No hardcoded primal names**

---

## Philosophy

> **Primals have self-knowledge. BiomeOS has no primal knowledge.**

Instead of hardcoding primal names like "songbird" or "beardog", we deploy based on **required capabilities**:

- ❌ **Bad**: "Deploy songbird, beardog, nestgate"
- ✅ **Good**: "Deploy with P2P, Identity, Storage capabilities"

At runtime, biomeOS discovers which primals are available and starts those that provide the required capabilities.

---

## Benefits

### 1. Agnostic Orchestration
- No hardcoded primal names
- Works with custom/evolved primals
- User compositions require no code changes

### 2. Evolution-Friendly
- New primals discovered automatically
- Old primals can be replaced
- Capabilities remain stable as implementations evolve

### 3. Flexible Deployment
- Same profile works with different primal sets
- Mix official and custom primals
- No coupling between biomeOS and specific primals

---

## Capabilities

### Core Capabilities

```rust
pub enum Capability {
    /// P2P coordination and mDNS discovery
    P2PCoordination,
    
    /// Identity and authentication
    Identity,
    
    /// Encrypted storage
    Storage,
    
    /// Compute orchestration
    Compute,
    
    /// Time-series and lineage tracking
    TemporalTracking,
    
    /// Encryption services
    Encryption,
    
    /// State management
    StateManagement,
    
    /// UI/visualization
    Visualization,
    
    /// Custom capability
    Custom(String),
}
```

### Example Providers

**Note**: These are examples only - actual primals are discovered at runtime!

| Capability | Example Providers |
|------------|-------------------|
| P2PCoordination | songbird, custom-p2p |
| Identity | beardog, custom-identity |
| Storage | nestgate, custom-storage |
| Compute | toadstool, custom-compute |
| TemporalTracking | sweetgrass, custom-temporal |
| Encryption | rhizocrypt, custom-crypto |
| StateManagement | loamspine, custom-state |
| Visualization | petaltongue, custom-ui |

---

## Capability Profiles

### Minimal Federation
```rust
CapabilityProfile::minimal_federation()
```

**Required**:
- P2PCoordination

**Use Case**: Basic P2P network

---

### Full Federation
```rust
CapabilityProfile::full_federation()
```

**Required**:
- P2PCoordination
- Identity

**Optional**:
- Storage

**Use Case**: Secure P2P with identity

---

### Compute Node
```rust
CapabilityProfile::compute_node()
```

**Required**:
- P2PCoordination
- Compute

**Optional**:
- Identity

**Use Case**: Distributed computation

---

### Storage Node
```rust
CapabilityProfile::storage_node()
```

**Required**:
- P2PCoordination
- Storage
- Encryption

**Optional**:
- Identity

**Use Case**: Secure distributed storage

---

### Full Ecosystem
```rust
CapabilityProfile::full_ecosystem()
```

**Required**:
- P2PCoordination
- Identity
- Storage
- Encryption

**Optional**:
- Compute
- TemporalTracking
- Visualization

**Use Case**: Complete biome with all features

---

## Usage

### Provision with Capability Profile

```bash
cd validation

# Minimal federation (P2P only)
cargo run --bin provision-with-capabilities minimal-federation

# Full federation (P2P + Identity + Storage)
cargo run --bin provision-with-capabilities full-federation

# Compute node
cargo run --bin provision-with-capabilities compute-node

# Storage node
cargo run --bin provision-with-capabilities storage-node

# Full ecosystem
cargo run --bin provision-with-capabilities full-ecosystem
```

### Runtime Discovery

Once deployed, biomeOS discovers primals at runtime:

```
1. biomeOS starts
2. Scans primalBins/ directory
3. Each primal reports its capabilities
4. biomeOS matches capabilities to profile
5. Required capabilities started automatically
6. Optional capabilities started if available
```

---

## Custom Profiles

Create custom capability profiles:

```rust
use biomeos_validation::{Capability, CapabilityProfile};

let mut profile = CapabilityProfile::new("my-custom");
profile
    .require(Capability::P2PCoordination)
    .require(Capability::Storage)
    .optional(Capability::Compute);
```

---

## How It Works

### Deployment Time

1. **Profile Selected**: Choose capability profile (e.g., "minimal-federation")
2. **VMs Provisioned**: Create VMs with appropriate resources
3. **Capability Manifest Deployed**: Profile written to `/opt/biomeos/capabilities.manifest`
4. **BiomeOS Deployed**: Core orchestration deployed
5. **No Primal Names**: No hardcoded primal binaries deployed

### Runtime Discovery

1. **BiomeOS Starts**: Reads capability manifest
2. **Scans primalBins/**: Discovers available primals
3. **Capability Matching**: Matches primals to required capabilities
4. **Start Required**: Launches primals for required capabilities
5. **Start Optional**: Launches primals for optional capabilities if available
6. **Dynamic**: As primals evolve, new versions discovered automatically

---

## Principles

### ✅ Agnostic

```rust
// Bad (hardcoded names)
deployment.primals = vec!["songbird", "beardog"];

// Good (capabilities)
deployment.capability_profile = CapabilityProfile::minimal_federation();
```

### ✅ Evolution-Friendly

New primals providing existing capabilities work automatically:

```
Old: songbird provides P2PCoordination
New: custom-p2p provides P2PCoordination
Result: Works with no code changes!
```

### ✅ User Compositions

Users can create custom primals:

```
User creates: my-awesome-p2p
Reports: P2PCoordination capability
BiomeOS: Discovers and uses automatically
```

---

## Example: Minimal Federation

### Profile
```rust
CapabilityProfile::minimal_federation()
  Required: P2PCoordination
```

### Deployment
```bash
cargo run --bin provision-with-capabilities minimal-federation
```

### Result
```
VMs provisioned
Capability manifest: P2PCoordination required
BiomeOS deployed
```

### Runtime
```
BiomeOS starts
Scans primalBins/
Finds: songbird (provides P2PCoordination)
Starts: songbird orchestrate
Result: Federation forms!
```

### With Custom Primal
```
User adds: custom-p2p binary to primalBins/
Reports: P2PCoordination capability
BiomeOS scans: Finds custom-p2p
Uses: custom-p2p instead of songbird
No code changes needed!
```

---

## Status

| Component | Status |
|-----------|--------|
| **Capability Enum** | ✅ Defined |
| **Profiles** | ✅ 5 profiles ready |
| **Deployment** | ✅ Capability-based |
| **Discovery** | 🚧 TODO (runtime) |
| **Matching** | 🚧 TODO (runtime) |

**Current**: Capability profiles deployed to VMs  
**Next**: Implement runtime discovery in biomeOS core  

---

## Architecture

```
Deployment Time:
  CapabilityProfile → capabilities.manifest → VM

Runtime:
  BiomeOS → Read manifest → Scan primalBins/ → Match capabilities → Start primals
```

**Key**: No primal names at deployment time, only capabilities!

---

**Principle**: Agnostic orchestration ✅  
**Status**: Capability-based deployment ready 🌟  
**Next**: Runtime discovery in biomeOS core 🚀

