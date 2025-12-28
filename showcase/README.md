# BiomeOS Showcase Summary

**Demonstrating capability-based orchestration, sovereignty preservation, and ecosystem evolution**

This showcase demonstrates BiomeOS's core philosophy: **discover capabilities at runtime, adapt to whatever is available, preserve sovereignty, and evolve gracefully**.

## Quick Navigation

### Single-Primal Demos ([01-single-primal/](./01-single-primal/))
Discover and integrate individual primals:
- **Songbird**: Universal Port Authority + Multi-tower federation
- **Toadstool**: GPU compute + Distributed execution
- **BearDog**: Entropy hierarchy + Capability-based encryption
- **Nestgate**: Lineage tracking + Sovereign storage + CALM federation
- **Squirrel**: MCP agents + Multi-agent coordination

### Primal Pairs ([02-primal-pairs/](./02-primal-pairs/))
Cross-primal orchestration patterns:
- **Songbird + Toadstool**: Distributed compute via port authority
- **BearDog + Toadstool**: Encrypted workload execution
- **Nestgate + BearDog**: Encrypted sovereign storage
- **Songbird + Nestgate**: Federated data retrieval
- **Squirrel + Toadstool**: AI-driven compute optimization

### Full Ecosystem ([03-full-ecosystem/](./03-full-ecosystem/))
All 5 primals working together:
- **Sovereign ML Pipeline**: Medical imaging training with HIPAA compliance
  - Data with lineage (Nestgate) → Encrypted (BearDog) → AI-optimized (Squirrel) → Routed (Songbird) → GPU trained (Toadstool) → Stored with provenance (Nestgate)

## The BiomeOS Philosophy

### 1. Capability Over Identity
```bash
# BiomeOS doesn't ask: "Is Toadstool running?"
# BiomeOS asks: "Who provides 'compute' capability?"

COMPUTE_SERVICE=$(discover_primal_by_capability "compute")
# Works with Toadstool, CloudGPU, or any compute primal
```

### 2. Runtime Discovery
No configuration files. No hardcoded endpoints. Pure runtime discovery:
- mDNS service discovery
- UDP broadcast discovery
- IP multicast discovery
- Explicit endpoint setting (fallback)

### 3. Interface Adaptation
BiomeOS probes for common patterns:
```bash
# Try multiple API patterns
for path in "/api/v1/tasks" "/api/tasks" "/tasks"; do
    if endpoint_exists($path); then
        USE_ENDPOINT=$path
        break
    fi
done
```

### 4. Graceful Degradation
When capabilities unavailable:
- Encryption missing? → Use OS-level crypto
- Storage missing? → Use local filesystem
- AI missing? → Use rule-based automation
- **System continues operating**

### 5. Evolution Resilience
Works when ecosystem evolves:
- Primals update APIs → BiomeOS adapts via probing
- New primals appear → Discovered via capabilities
- Multiple instances → Load balanced automatically
- Geographic distribution → Federation transparent

## Key Patterns Demonstrated

### Universal Port Authority (Songbird)
> "Once primals understand Songbird, they never set their own ports"

- **Zero port conflicts**: Songbird assigns ALL ports
- **Capability registry**: Discover services by function
- **Multi-tower federation**: Geographic distribution
- **Load balancing**: Automatic across instances

### Entropy Hierarchy (BearDog)
Three levels of key management:
- **EPHEMERAL**: In-memory, wiped on shutdown
- **SESSION**: Persists for session, rotated
- **PERSISTENT**: Long-lived, backed up

BiomeOS discovers and uses appropriate level for each use case.

### Lineage Tracking (Nestgate)
Every data operation captures:
- **WHO**: Actor/service that performed action
- **WHAT**: Data content and operations
- **WHEN**: Timestamps (ISO 8601)
- **WHY**: Purpose, justification, consent

Enables compliance (HIPAA, GDPR) and full audit trails.

### MCP Agent Pattern (Squirrel)
- **Tool discovery**: Agents discover ecosystem as tools
- **Multi-agent coordination**: Complex reasoning via consensus
- **Adaptive learning**: Improves recommendations over time
- **Model agnostic**: Works with Claude, GPT, Llama, etc.

### Zero-Knowledge Compute
Encrypted data processing pattern:
1. Encrypt at source (BearDog)
2. Compute on encrypted (Toadstool)
3. Keys never leave encryption service
4. Decrypt only results
5. Full audit trail (Nestgate)

## Running the Showcases

### Quick Start
```bash
# Run all single-primal demos
cd 01-single-primal
./run-all-single-demos.sh

# Run all primal-pair demos
cd ../02-primal-pairs
./run-all-pair-demos.sh

# Run full ecosystem demo
cd ../03-full-ecosystem
./sovereign-ml-pipeline.sh
```

### With Explicit Endpoints
```bash
# If primals already running
export SONGBIRD_ENDPOINT="http://localhost:8080"
export TOADSTOOL_ENDPOINT="http://localhost:8081"
export BEARDOG_ENDPOINT="http://localhost:8082"
export NESTGATE_ENDPOINT="http://localhost:8083"
export SQUIRREL_ENDPOINT="http://localhost:8084"

./sovereign-ml-pipeline.sh
```

### Discovery-Only Mode
```bash
# Test discovery without execution
export DISCOVERY_ONLY=true
./sovereign-ml-pipeline.sh
```

## Gap Reports

Every demo generates a gap report documenting:
- ✓ What worked out of the box
- ⚠ Where BiomeOS adapted/probed
- ✗ What failed gracefully
- → What evolution would improve

Find reports in each directory's `gaps/` folder.

## Real-World Applications

### 1. Sovereign ML Training
**Use case**: Medical research on PHI
- HIPAA/GDPR compliant by design
- Zero data leaving sovereignty
- Full audit trail for regulators
- Works with any compute provider

### 2. Multi-Party Computation
**Use case**: Financial fraud detection across banks
- Each bank runs local BiomeOS
- Encrypted feature federation
- No raw data sharing
- Industry-wide fraud patterns detected

### 3. Federated Analytics
**Use case**: University research collaboration
- Data stays at each institution
- CALM federation for consistency
- Full lineage across federation
- Reproducible science

### 4. Cloud Bursting with Sovereignty
**Use case**: Local compute, cloud overflow
- Primary: Local Toadstool
- Overflow: Cloud GPU (encrypted)
- Zero config change
- BiomeOS routes intelligently

## Evolution Scenarios

Every demo shows resilience to changes:

### 1. Primal Updates
- **Scenario**: Toadstool updates from v2 → v3
- **Impact**: BiomeOS probes new interface, adapts
- **Result**: Zero BiomeOS changes needed

### 2. Alternate Implementations
- **Scenario**: Replace BearDog with CloudHSM
- **Impact**: BiomeOS discovers 'encryption' capability
- **Result**: Works immediately

### 3. Multiple Instances
- **Scenario**: Deploy 5 Toadstool instances
- **Impact**: Songbird load balances automatically
- **Result**: Improved performance, zero code changes

### 4. Geographic Federation
- **Scenario**: Expand from 1 tower to 3 (EU, US, APAC)
- **Impact**: Songbird + Nestgate federate
- **Result**: Multi-region transparent to BiomeOS

### 5. Partial Failures
- **Scenario**: BearDog temporarily down
- **Impact**: BiomeOS detects, falls back to OS crypto
- **Result**: Degraded security but continues operating

## Success Metrics

BiomeOS showcase demonstrates:
- ✓ **Zero hardcoded primal names**: Only capabilities used
- ✓ **Runtime discovery**: mDNS, broadcast, multicast, explicit
- ✓ **Interface adaptation**: Probes and adapts to APIs
- ✓ **Sovereignty preservation**: Data stays local by default
- ✓ **Full lineage**: WHO/WHAT/WHEN/WHY tracked
- ✓ **Zero-knowledge compute**: Encrypted execution
- ✓ **Evolution resilience**: 5+ scenarios per demo
- ✓ **Graceful degradation**: Works with partial ecosystem

## Architecture Principles

### Indirection is Power
```
BiomeOS → Songbird → Target Primal
```
Indirection enables evolution without breaking BiomeOS.

### Capability > Identity
```
discover_by_capability("compute")  # ✓ Evolvable
discover_by_name("toadstool")      # ✗ Brittle
```

### Zero Coordination Needed
New primal? Just advertise capability. BiomeOS finds and uses it.

### Sovereignty Preserved
Each primal controls its domain:
- Songbird: Port allocation
- BearDog: Key management
- Nestgate: Data storage
- Toadstool: Compute execution
- Squirrel: AI reasoning

BiomeOS orchestrates but never violates sovereignty.

## Next Steps

1. **Run the demos**: Start with single-primal, build to full ecosystem
2. **Review gap reports**: Understand adaptation points
3. **Explore phase2 primals**: loamSpine, rhizoCrypt, sweetGrass, petalTongue
4. **Build custom primals**: Advertise capabilities, BiomeOS discovers
5. **Deploy multi-tower**: Test federation and geographic distribution

## Contributing New Showcases

Template pattern:
```bash
#!/usr/bin/env bash
set -e
source ./common/capability-discovery.sh

# 1. Discover needed capabilities
SERVICE_A=$(discover_primal_by_capability "capability_x")
SERVICE_B=$(discover_primal_by_capability "capability_y")

# 2. Probe interfaces
probe_primal_interface "$SERVICE_A"
probe_primal_interface "$SERVICE_B"

# 3. Execute workflow
# 4. Document gaps
# 5. Demonstrate evolution
```

See existing demos for full implementation.

## Philosophy Summary

**BiomeOS is NOT**:
- ❌ A microservices orchestrator (no hardcoded services)
- ❌ A configuration management tool (runtime discovery)
- ❌ A vendor platform (works with any primal)

**BiomeOS IS**:
- ✅ A capability-based discovery system
- ✅ A sovereignty-preserving orchestrator
- ✅ An evolution-resilient ecosystem enabler
- ✅ A digital sovereignty operating system

The goal: **Enable a competitive ecosystem of primals while preserving user sovereignty and enabling graceful evolution.**

---

**Explore the showcases. Watch BiomeOS discover, adapt, and orchestrate. See sovereignty in action.**
