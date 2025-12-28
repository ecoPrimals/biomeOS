# BiomeOS Showcase: Primal Pairs

**Cross-primal orchestration demonstrating capability-based composition**

## Philosophy

BiomeOS orchestrates multiple primals WITHOUT direct knowledge of their identities. Instead, it uses **capability-based discovery** and **dynamic adaptation** to compose workflows.

Key principles:
- **Indirection is power**: BiomeOS → Service Registry → Target Primal
- **Capability over identity**: Ask for "encryption" not "BearDog"
- **Zero coordination needed**: New primals auto-discovered
- **Sovereignty preserved**: Each primal controls its implementation

## Demonstrations

### 1. Songbird + Toadstool: Universal Port Authority

**Pattern**: Service registry + dynamic compute allocation

```bash
./songbird-toadstool-distributed-compute.sh
```

**What it shows:**
- Songbird assigns ALL ports (zero conflicts)
- Toadstool registers "compute" capability
- BiomeOS discovers compute WITHOUT knowing "Toadstool"
- Task routing via capability, not name
- Multiple compute instances load balanced automatically

**Architecture:**
```
BiomeOS (only knows Songbird)
   ↓
Songbird (port authority + service registry)
   ↓ (discovers "compute" capability)
Toadstool (assigned port dynamically)
```

**Key insight from Songbird showcase:**
> "Once primals understand Songbird, they never set their own ports"

**Evolution scenarios:**
1. **Multiple Toadstool instances**: Each registers, gets unique port, auto load-balanced
2. **Alternate compute primal**: Registers "compute", BiomeOS uses immediately
3. **Toadstool API changes**: Re-registers with new capabilities, transparent to BiomeOS
4. **Multi-tower federation**: Cross-tower routing automatic, BiomeOS unchanged

---

### 2. BearDog + Toadstool: Encrypted Workload Execution

**Pattern**: Capability-based encryption + compute

```bash
./beardog-toadstool-encrypted-workload.sh
```

**What it shows:**
- BiomeOS discovers "encryption" capability (not "BearDog")
- BearDog's entropy hierarchy (ephemeral → session → persistent)
- Encrypted task submission to compute
- Zero-knowledge execution (compute never sees plaintext)
- Secure result return

**Architecture:**
```
BiomeOS
   ↓
BearDog (encrypt task)
   ↓
Toadstool (execute encrypted)
   ↓
BearDog (decrypt result)
   ↓
BiomeOS (receives plaintext result)
```

**Evolution scenarios:**
1. **Alternate encryption**: CloudHSM, YubiKey, etc. - BiomeOS adapts
2. **New algorithms**: Discovered via capability query
3. **Encryption unavailable**: Graceful degradation to OS-level crypto
4. **Zero-knowledge compute**: Homomorphic encryption (future)

---

### 3. Nestgate + BearDog: Encrypted Sovereign Storage

**Pattern**: Lineage-aware encrypted persistence

```bash
./nestgate-beardog-encrypted-storage.sh
```

**What it shows:**
- Nestgate's lineage tracking (who, what, when, why)
- Encryption at rest via BearDog
- Sovereignty enforcement (data never leaves without consent)
- Audit trail with cryptographic attestation

---

### 4. Songbird + Nestgate: Federated Data Retrieval

**Pattern**: Service discovery + distributed storage

```bash
./songbird-nestgate-federated-data.sh
```

**What it shows:**
- Multi-tower data federation
- Capability-based storage discovery
- Cross-tower lineage preservation
- Sovereignty across federation boundaries

---

### 5. Squirrel + Toadstool: AI-Driven Compute Optimization

**Pattern**: MCP agents + dynamic workload placement

```bash
./squirrel-toadstool-ai-compute.sh
```

**What it shows:**
- Squirrel MCP agents analyze workload
- Dynamic compute resource selection
- Multi-agent coordination via Songbird
- Adaptive optimization over time

---

## Common Patterns

### Capability-Based Discovery

All demos follow this pattern:

```rust
// BiomeOS doesn't know "BearDog", asks for capability
let encryption_service = discover_by_capability("encryption").await?;

// Works with BearDog, CloudHSM, YubiKey, or future primals
let encrypted = encryption_service.encrypt(data).await?;
```

### Interface Adaptation

BiomeOS probes for interface patterns:

```rust
// Try common interface patterns
for path in ["/api/v1/encrypt", "/api/encrypt", "/encrypt"] {
    if endpoint_exists(path).await {
        return Ok(path);
    }
}
```

### Graceful Degradation

When capabilities unavailable:

```rust
let encryption_service = match discover_by_capability("encryption").await {
    Ok(svc) => svc,
    Err(_) => {
        warn!("No encryption service, using OS-level crypto");
        return fallback_encryption();
    }
};
```

---

## Running the Demos

### Prerequisites

```bash
# Ensure primals available (will auto-discover)
ls ../../../phase1/{songbird,toadstool,beardog,nestgate,squirrel}/target/release/

# Or set explicit endpoints
export SONGBIRD_ENDPOINT="http://localhost:8080"
export TOADSTOOL_ENDPOINT="http://localhost:8081"
```

### Run All Demos

```bash
./run-all-pair-demos.sh
```

### Run Specific Demo

```bash
./songbird-toadstool-distributed-compute.sh
```

Each demo generates a gap report in `gaps/` documenting what worked, what didn't, and where BiomeOS needed to adapt.

---

## Gap Reports

Each demo creates a detailed gap report:

```
gaps/
├── songbird-toadstool-orchestration-gaps.md
├── beardog-toadstool-encryption-gaps.md
├── nestgate-beardog-storage-gaps.md
├── songbird-nestgate-federation-gaps.md
└── squirrel-toadstool-optimization-gaps.md
```

These reports show:
- ✓ What worked out of the box
- ⚠ Where BiomeOS adapted/probed
- ✗ What failed gracefully
- → What evolution would improve

---

## Evolution Strategy

### When Primals Change

1. **API versioning**: BiomeOS probes for v1, v2, v3, etc.
2. **Capability changes**: Re-query capabilities, adapt
3. **New endpoints**: Interface probing discovers them
4. **Breaking changes**: Graceful degradation, log warnings

### When Ecosystem Grows

1. **New primals**: Auto-discovered via capability
2. **Multiple instances**: Songbird load balances
3. **Multi-tower**: Federation transparent to BiomeOS
4. **Alternate implementations**: Work via same capabilities

### When Requirements Change

1. **New capability needed**: Discover or gracefully degrade
2. **Performance constraints**: Query resource availability
3. **Security requirements**: Probe for encryption levels
4. **Compliance needs**: Lineage tracking via Nestgate

---

## Sovereignty Preservation

Every demo maintains digital sovereignty:

1. **Local-first**: Primals run locally, no cloud required
2. **Consent-based sharing**: Data stays unless explicitly shared
3. **Lineage tracking**: Full audit trail of all operations
4. **No telemetry**: Zero metrics sent without explicit opt-in
5. **Runtime discovery**: No hardcoded primal endpoints

---

## Next Steps

After exploring primal pairs:

1. **Full Ecosystem Demo**: All 5 primals orchestrated (`../03-full-ecosystem/`)
2. **BiomeOS Features**: Multi-tower federation, primal adapter evolution (`../04-biomeos-features/`)
3. **Phase2 Primals**: loamSpine, rhizoCrypt, sweetGrass, petalTongue integration

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                       BiomeOS                           │
│                                                         │
│  • Capability-based discovery                          │
│  • Interface adaptation                                │
│  • Orchestration logic                                 │
│  • Graceful degradation                                │
└────────────┬───────────────────────────┬────────────────┘
             │                           │
     ┌───────▼────────┐         ┌───────▼────────┐
     │   Songbird     │         │   BearDog      │
     │                │         │                │
     │ • Port authority│        │ • Encryption   │
     │ • Service reg.  │        │ • Entropy mgmt │
     │ • Federation    │        │ • Key lifecycle│
     └────────┬────────┘        └────────────────┘
              │
      ┌───────▼────────┬────────────┬─────────────┐
      │                │            │             │
┌─────▼─────┐  ┌──────▼──────┐ ┌───▼────┐  ┌────▼─────┐
│ Toadstool │  │  Nestgate   │ │Squirrel│  │ Phase2   │
│           │  │             │ │        │  │ Primals  │
│ • Compute │  │ • Storage   │ │ • AI   │  │          │
│ • GPU     │  │ • Lineage   │ │ • MCP  │  │ • loam   │
│ • Distrib.│  │ • Sovereign │ │ • Agent│  │ • rhizo  │
└───────────┘  └─────────────┘ └────────┘  └──────────┘
```

**Key insight**: BiomeOS only directly knows Songbird and BearDog (core capabilities). All other primals discovered via capability queries through Songbird.

---

## Lessons from Phase1 Primals

### From Songbird
- Universal Port Authority pattern prevents all conflicts
- Multi-tower federation enables geographic distribution
- Service registry + capability query = zero-coordination ecosystem

### From Toadstool
- GPU compute + distributed execution = serious workloads
- Resource-aware scheduling improves utilization
- Pluggable compute backends (local, cloud, hybrid)

### From BearDog
- Entropy hierarchy (ephemeral → session → persistent) = security + performance
- Capability-based crypto = works with any provider
- Zero-knowledge patterns enable private compute

### From Nestgate
- Lineage tracking = full audit trail
- Sovereignty enforcement = data control
- CALM federation = conflict-free replication

### From Squirrel
- MCP agent pattern = composable AI workflows
- Multi-agent coordination = complex reasoning
- Adaptive optimization = learning systems

---

## Contributing New Pair Demos

Template:

```bash
#!/usr/bin/env bash
# PrimalA + PrimalB: <Purpose>

set -e
source ../01-single-primal/common/capability-discovery.sh

# 1. Discover PrimalA by capability (not name)
PRIMAL_A=$(discover_primal_by_capability "capability_x")

# 2. Discover PrimalB by capability (not name)  
PRIMAL_B=$(discover_primal_by_capability "capability_y")

# 3. Compose workflow using adapted interfaces
# 4. Document gaps and evolution scenarios
# 5. Demonstrate resilience to changes
```

See existing demos for full pattern.
