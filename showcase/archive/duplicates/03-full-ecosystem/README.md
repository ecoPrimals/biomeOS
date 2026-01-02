# BiomeOS Showcase: Full Ecosystem Integration

**All 5 Phase1 primals orchestrated via capability-based discovery**

## Philosophy

This directory demonstrates BiomeOS's ultimate power: orchestrating complex, multi-service workflows WITHOUT hardcoding primal names or endpoints. BiomeOS discovers and adapts to whatever capabilities are available in the ecosystem.

## The Grand Demo: Sovereign ML Training Pipeline

```bash
./sovereign-ml-pipeline.sh
```

### What It Demonstrates

A complete ML training pipeline that:
1. Stores sensitive medical data with lineage (**Nestgate**)
2. Encrypts data before compute (**BearDog**)
3. Designs optimal pipeline (**Squirrel** AI agent)
4. Coordinates all services (**Songbird** registry)
5. Executes GPU training (**Toadstool** compute)
6. Stores encrypted model with full provenance (**Nestgate**)

**BiomeOS NEVER mentions "Nestgate", "BearDog", "Squirrel", "Songbird", or "Toadstool" by name!**

### Architecture

```
                         BiomeOS
                           |
                    (capability queries)
                           |
                     ┌─────▼─────┐
                     │ Songbird  │
                     │ (registry)│
                     └─────┬─────┘
                           |
        ┌──────────┬───────┼───────┬──────────┐
        │          │       │       │          │
   ┌────▼───┐ ┌───▼──┐ ┌──▼───┐ ┌─▼──────┐ ┌─▼───────┐
   │Nestgate│ │BearDog│ │Squirrel│Toadstool││ (future)│
   │(store) │ │(crypt)│ │ (AI)  ││ (GPU)  ││ primals │
   └────────┘ └───────┘ └───────┘ └────────┘ └─────────┘
```

### The Pipeline Flow

```
┌─────────────────────────────────────────────────────────┐
│ SOVEREIGN ML TRAINING PIPELINE                          │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  1. Data Ingestion (Nestgate)                          │
│     ↓                                                   │
│  2. AI Planning (Squirrel analyzes requirements)       │
│     ↓                                                   │
│  3. Encryption (BearDog with AES-256-GCM)              │
│     ↓                                                   │
│  4. Service Discovery (Songbird finds GPU compute)     │
│     ↓                                                   │
│  5. Training Execution (Toadstool GPU, encrypted)      │
│     ↓                                                   │
│  6. Model Storage (Nestgate with full lineage)         │
│                                                         │
│ Result: HIPAA/GDPR compliant ML, full audit trail     │
└─────────────────────────────────────────────────────────┘
```

### Key Properties

#### 1. Capability-Based Discovery
BiomeOS discovers services by asking:
- "Who provides 'storage'?" → Finds Nestgate
- "Who provides 'encryption'?" → Finds BearDog
- "Who provides 'ai_agent'?" → Finds Squirrel
- "Who provides 'service_registry'?" → Finds Songbird
- "Who provides 'compute' with GPU?" → Finds Toadstool

**Works with ANY primal that advertises these capabilities!**

#### 2. Sovereignty Preservation
- **Data stays local**: Unless explicit consent given
- **Full lineage**: WHO, WHAT, WHEN, WHY tracked
- **Compliance**: HIPAA, GDPR, purpose limitation
- **Audit trail**: Complete provenance for regulations

#### 3. Zero-Knowledge Compute
- Training data encrypted before leaving storage
- Compute service NEVER sees plaintext
- Model encrypted at rest
- Keys isolated to encryption service

#### 4. Intelligent Optimization
- AI agent analyzes requirements
- Recommends optimal resource allocation
- Balances security vs performance
- Adapts to ecosystem changes

#### 5. Evolution Resilience
The demo shows 4 evolution scenarios:
1. **Replace Toadstool with cloud GPU**: BiomeOS unchanged
2. **Upgrade to homomorphic encryption**: Automatic discovery
3. **Multi-tower federation**: Geographic distribution transparent
4. **Partial failures**: Graceful degradation

---

## Running the Demo

### Prerequisites

```bash
# Check for phase1 primal binaries
ls ../../../../phase1/{songbird,toadstool,beardog,nestgate,squirrel}/target/release/

# Or set explicit endpoints
export SONGBIRD_ENDPOINT="http://localhost:8080"
export NESTGATE_ENDPOINT="http://localhost:8081"
export BEARDOG_ENDPOINT="http://localhost:8082"
export SQUIRREL_ENDPOINT="http://localhost:8083"
export TOADSTOOL_ENDPOINT="http://localhost:8084"
```

### Run the Demo

```bash
./sovereign-ml-pipeline.sh
```

The demo will:
1. Bootstrap all 5 primals
2. Discover capabilities dynamically
3. Execute complete ML pipeline
4. Generate full audit trail
5. Demonstrate evolution scenarios
6. Clean shutdown with lineage preservation

### Graceful Degradation

If any primal is unavailable:
- **No storage**: Falls back to local filesystem (no lineage)
- **No encryption**: Uses OS-level crypto (degraded security)
- **No AI**: Uses rule-based pipeline design
- **No registry**: Direct service communication
- **No compute**: Training deferred

The demo continues and shows what's possible with available capabilities.

---

## Gap Report

After running, check:
```
gaps/full-ecosystem-ml-pipeline-gaps.md
```

This documents:
- ✓ What worked perfectly
- ⚠ Where BiomeOS adapted
- ✗ What failed gracefully
- → What evolution would improve

---

## Real-World Applications

### 1. Medical Research on PHI
**Scenario**: Train diagnostic AI on patient records

**BiomeOS Solution**:
- Nestgate stores records with consent tracking
- BearDog encrypts before compute
- Toadstool trains on encrypted data
- Full HIPAA compliance audit trail

**Benefits**:
- Researchers never see raw PHI
- Data sovereignty preserved
- Complete lineage for regulators
- Works with any hospital's systems

### 2. Financial ML on Sensitive Data
**Scenario**: Fraud detection across institutions

**BiomeOS Solution**:
- Each institution runs local BiomeOS
- Nestgate federates encrypted features
- Squirrel coordinates multi-party learning
- Models trained without data sharing

**Benefits**:
- Zero data leaving institutions
- Regulatory compliance (SOX, PCI DSS)
- Competitive advantage preserved
- Fraud patterns detected across industry

### 3. Government AI on Classified Data
**Scenario**: Intelligence analysis at multiple classification levels

**BiomeOS Solution**:
- Separate BearDog instances per classification
- Nestgate enforces clearance-based access
- Toadstool in secure enclaves
- Lineage proves proper handling

**Benefits**:
- No accidental spillage
- Complete audit trail
- Works with existing systems
- Scales across agencies

### 4. Federated Learning Across Institutions
**Scenario**: University collaboration on rare disease research

**BiomeOS Solution**:
- Each university's local BiomeOS + Nestgate
- Songbird federation across sites
- Squirrel coordinates distributed training
- Models improve without data centralization

**Benefits**:
- Research advances faster
- Patient privacy preserved
- Institutional sovereignty maintained
- Reproducible science (full lineage)

---

## Why This Architecture Matters

### Traditional ML Platforms

❌ **Cloud-first**:
- Data must upload to vendor
- Sovereignty lost
- Compliance nightmares

❌ **Vendor lock-in**:
- Hardcoded integrations
- Can't switch providers
- Monopoly pricing

❌ **Opaque**:
- No lineage tracking
- Audit trail missing
- Regulatory problems

❌ **Fragile**:
- Single point of failure
- Upgrades break integrations
- Expensive to change

### BiomeOS Sovereign ML

✅ **Local-first**:
- Data stays sovereign
- Upload by consent only
- Compliance by design

✅ **Capability-based**:
- Works with any primal
- Competitive ecosystem
- Market-driven pricing

✅ **Transparent**:
- Full lineage tracked
- Complete audit trail
- Regulatory compliance

✅ **Resilient**:
- Graceful degradation
- Evolution transparent
- Adapt to ecosystem changes

---

## Technical Deep Dive

### How BiomeOS Discovers Services

1. **Bootstrap**: Start with service registry (Songbird)
2. **Capability query**: Ask "who has 'storage'?"
3. **Interface probing**: Try common API patterns
4. **Adaptation**: Use whatever works
5. **Caching**: Remember successful patterns

No configuration files. No hardcoded endpoints. Pure runtime discovery.

### How Data Sovereignty is Preserved

1. **Consent tracking**: Nestgate records all permissions
2. **Purpose limitation**: Data only used for stated purpose
3. **Local-first**: Nothing leaves without consent
4. **Lineage**: WHO/WHAT/WHEN/WHY always tracked
5. **Federation**: CALM ensures consistency

Even across multiple towers, sovereignty never violated.

### How Encryption Provides Zero-Knowledge

1. **Encrypt at source**: Before data enters compute
2. **Key isolation**: Only BearDog holds keys
3. **Ephemeral plaintext**: Immediately wiped after use
4. **Result encryption**: Model encrypted before storage
5. **Audit trail**: All key access logged

Compute provider never has opportunity to see plaintext.

### How AI Optimizes the Pipeline

1. **Requirement analysis**: Parse compliance, performance, security needs
2. **Ecosystem scan**: Discover available capabilities
3. **Resource optimization**: GPU vs CPU, local vs remote
4. **Security design**: Encryption levels, key management
5. **Recommendation**: Complete pipeline configuration

Squirrel treats entire ecosystem as tools for problem-solving.

### How the Ecosystem Evolves

**Adding new primals**: Just advertise capabilities, BiomeOS discovers

**Replacing primals**: New primal with same capability, BiomeOS switches

**Upgrading primals**: Re-advertise with new features, BiomeOS adapts

**Partial failures**: BiomeOS degrades gracefully, logs issues

**Geographic expansion**: Songbird federation, transparent to BiomeOS

No recompilation. No configuration changes. Runtime adaptation.

---

## Extending the Pipeline

### Add Phase2 Primals

```bash
# loamSpine: Distributed memory/cache
# rhizoCrypt: Advanced cryptography
# sweetGrass: Data transformation
# petalTongue: Natural language interfaces

# BiomeOS discovers and uses them automatically!
```

### Add Custom Primals

Create any primal that advertises capabilities:
- `storage`, `compute`, `encryption`, `ai_agent`, etc.
- BiomeOS finds and uses it
- No BiomeOS changes needed

### Multi-Tower Deployment

Deploy BiomeOS + primals across multiple geographic locations:
- Songbird federates service registries
- Nestgate federates data (CALM)
- BiomeOS routes intelligently
- Sovereignty preserved everywhere

---

## Success Metrics

After running the demo, BiomeOS should demonstrate:

1. **✓ Zero hardcoded primal names**: BiomeOS only knows capabilities
2. **✓ Complete sovereignty**: Data never left without consent
3. **✓ Full lineage**: Every operation tracked (WHO/WHAT/WHEN/WHY)
4. **✓ Zero-knowledge compute**: Toadstool never saw plaintext
5. **✓ Intelligent optimization**: Squirrel recommended optimal config
6. **✓ Graceful degradation**: Works with partial ecosystem
7. **✓ Evolution resilience**: 4 scenarios demonstrated

If all ✓, BiomeOS has achieved its design goals!

---

## Next Steps

1. **Review single-primal showcases**: `../01-single-primal/`
   - Deep dive on each primal's unique capabilities

2. **Explore primal pairs**: `../02-primal-pairs/`
   - How primals compose into powerful patterns

3. **BiomeOS-specific features**: `../04-biomeos-features/`
   - VM federation, primal adapter evolution, multi-tower

4. **Phase2 primal integration**: `../05-phase2-integration/`
   - loamSpine, rhizoCrypt, sweetGrass, petalTongue

---

## Contributing

To add new ecosystem demos:

1. Copy `sovereign-ml-pipeline.sh` as template
2. Define new multi-primal workflow
3. Use `discover_primal_by_capability` (no hardcoding!)
4. Document evolution scenarios
5. Generate gap report
6. Add to this README

Examples of new demos:
- Distributed data processing pipeline
- Multi-party secure computation
- Federated analytics across towers
- Real-time streaming with sovereignty
- Blockchain integration via primals

The pattern is: BiomeOS orchestrates, primals provide capabilities, ecosystem evolves.


