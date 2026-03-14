# Niches: Biome Environments for Primals and Chimeras

A **niche** is a **biome** - the complete environment where primals and chimeras operate together. The BYOB (Build Your Own Biome) system creates niches.

## Conceptual Hierarchy

```
┌────────────────────────────────────────────────────────────────────┐
│                              BIOME OS                              │
│                    (Substrate / Orchestrator)                       │
├────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                    NICHE: Gaming-Tournament                   │  │
│  │                       (The Biome)                             │  │
│  │  ┌────────────┐ ┌────────────┐ ┌─────────────────────────┐   │  │
│  │  │  BearDog   │ │  NestGate  │ │    CHIMERA:             │   │  │
│  │  │  (Primal)  │ │  (Primal)  │ │    Gaming-Mesh          │   │  │
│  │  │            │ │            │ │  ┌─────────┬──────────┐ │   │  │
│  │  │ Anti-cheat │ │ Replays    │ │  │Songbird │ToadStool │ │   │  │
│  │  │ Signing    │ │ Storage    │ │  │  Array  │   GPU    │ │   │  │
│  │  └────────────┘ └────────────┘ │  └─────────┴──────────┘ │   │  │
│  │                                 └─────────────────────────┘   │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │                   NICHE: Research-Lab                         │  │
│  │                       (The Biome)                             │  │
│  │  ┌────────────┐ ┌─────────────────────────────┐              │  │
│  │  │  Squirrel  │ │    CHIMERA: ML-Pipeline     │              │  │
│  │  │  (Primal)  │ │  ┌─────────┬──────────────┐ │              │  │
│  │  │ Inference  │ │  │ToadStool│   NestGate   │ │              │  │
│  │  │ Routing    │ │  │ Compute │ Checkpoints  │ │              │  │
│  │  └────────────┘ │  └─────────┴──────────────┘ │              │  │
│  │                  └─────────────────────────────┘              │  │
│  └──────────────────────────────────────────────────────────────┘  │
│                                                                     │
└────────────────────────────────────────────────────────────────────┘
```

## Directory Structure

```
niches/
├── templates/        # Reusable niche templates (BYOB starting points)
│   ├── gaming-tournament.yaml
│   ├── research-lab.yaml
│   ├── ai-research.yaml
│   ├── web-development.yaml
│   ├── federation-aware.yaml
│   └── custom-generic.yaml
│
├── examples/         # Example configurations and tutorials
│   └── minimal-p2p.yaml
│
└── rootpulse/        # RootPulse niche (compute orchestration)
    ├── rootpulse-niche.yaml
    ├── QUICKSTART.md
    └── examples/
```

## Niche vs Chimera vs Primal

| Concept | Definition | Analogy |
|---------|------------|---------|
| **Primal** | Standard organism with clear boundaries | A single species |
| **Chimera** | Amalgam of primal components | A hybrid species |
| **Niche** | Environment with primals + chimeras | An ecosystem |

## BYOB: Build Your Own Biome

The BYOB system lets users create custom niches:

```bash
# List available templates
biomeos niche list-templates

# Create new niche from template
biomeos niche create gaming-tournament --name "my-tournament"

# Add a chimera to the niche
biomeos niche add-chimera my-tournament gaming-mesh

# Add a standalone primal
biomeos niche add-primal my-tournament nestgate --config replays.yaml

# Deploy the niche
biomeos niche deploy my-tournament
```

## Niche Template Structure

```yaml
niche:
  id: "gaming-tournament"
  name: "Gaming Tournament"
  description: "Complete gaming tournament infrastructure"
  version: "1.0.0"

# What this niche contains
organisms:
  # Chimeras (pre-fused primal combinations)
  chimeras:
    - id: "gaming-mesh"
      config:
        songbird_count: 8
        gpu_required: true
  
  # Standalone primals
  primals:
    - type: "beardog"
      role: "anti-cheat"
      config:
        sign_all_actions: true
    
    - type: "nestgate"
      role: "replay-storage"
      config:
        retention_days: 30

# Niche-level configuration
environment:
  networking:
    mode: "mesh"
    latency_target_ms: 5
  
  security:
    encryption: "beardog"
    trust_model: "tofu"
  
  resources:
    cpu_cores: 16
    memory_gb: 32
    storage_gb: 500

# How organisms interact within the niche
interactions:
  - from: "gaming-mesh"
    to: "beardog:anti-cheat"
    type: "action_verification"
  
  - from: "gaming-mesh"
    to: "nestgate:replay-storage"
    type: "state_persistence"
```

## Niche Lifecycle

1. **Define** - Create or customize from template
2. **Validate** - Check all organisms and dependencies
3. **Deploy** - Start all organisms in the niche
4. **Monitor** - Track health and performance
5. **Scale** - Add/remove organism instances
6. **Snapshot** - Save niche state
7. **Destroy** - Clean shutdown

## Why Niches?

### Composability
- Mix chimeras and primals freely
- Each niche is a complete, isolated environment
- Niches can federate with other niches

### Reproducibility
- Template-based creation
- Version-controlled configurations
- Shareable via BYOB marketplace

### Isolation
- Niches don't interfere with each other
- Security boundaries at niche level
- Resource limits per niche

### Evolution
- Upgrade organisms within a niche
- Swap chimeras for new versions
- A/B test different configurations

