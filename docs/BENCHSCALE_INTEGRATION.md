# BiomeOS + benchScale Integration

**Capability-Based Validation Infrastructure**

---

## Philosophy

benchScale is NOT special to BiomeOS. Like primals, benchScale is discovered and consumed agnostically:

- **No hardcoding**: BiomeOS discovers benchScale capabilities at runtime
- **No standardization**: Adapts to benchScale's API (CLI + Rust lib)
- **Capability-based**: Queries what benchScale can do, then uses it
- **Optional tool**: BiomeOS works without benchScale (degrades gracefully)

---

## Architecture

```
┌──────────────────────────────────────────────────┐
│           BiomeOS (Orchestrator)                 │
│      "I need validation infrastructure"          │
└────────────────────┬─────────────────────────────┘
                     │
                     │ Runtime Discovery
                     ▼
┌──────────────────────────────────────────────────┐
│         benchScale (Validation Tool)             │
│  Capabilities:                                   │
│  - create_lab(topology_yaml)                     │
│  - validate_deployment(niche_manifest)           │
│  - inject_chaos(scenario)                        │
│  - measure_performance(metrics)                  │
└──────────────────────────────────────────────────┘
                     │
          ┌──────────┴──────────┐
          ▼                     ▼
   Docker Backend       libvirt Backend
   (containers)         (real VMs)
```

---

## Discovery Pattern

### BiomeOS Side

```rust
// BiomeOS discovers benchScale like it discovers primals
use biomeos_core::primal_adapter::{discover_tool_interface, ToolAdapter};

// 1. Find benchScale binary (no hardcoding!)
let benchscale_binary = find_tool_binary("benchscale")
    .context("benchScale not found - validation features disabled")?;

// 2. Discover capabilities
let adapter = discover_tool_interface(&benchscale_binary).await?;

// 3. Query capabilities
if adapter.has_capability("create_lab") {
    // Use it!
} else {
    // Graceful degradation
}
```

### benchScale Side

```bash
# benchScale exposes capabilities (like primals do)
benchscale --capabilities

# Output:
# capabilities:
#   - create_lab
#   - destroy_lab
#   - validate_deployment
#   - inject_chaos
#   - measure_performance
# backends:
#   - docker (available: yes)
#   - libvirt (available: yes)
```

---

## Integration Points

### 1. Niche Validation

When deploying a niche, BiomeOS can optionally validate it with benchScale:

```bash
# Deploy RootPulse niche with validation
biomeos niche deploy rootpulse-niche.yaml --validate

# BiomeOS coordinates:
# 1. Discovers benchScale
# 2. Creates validation lab
# 3. Deploys niche in lab
# 4. Runs validation tests
# 5. Reports results
# 6. Destroys lab (cleanup)
```

### 2. Local Development

Developers use benchScale for local testing:

```bash
# Create local test environment
benchscale create biomeos-dev topologies/biomeos-dev.yaml

# Deploy BiomeOS into lab
benchscale deploy biomeos-dev ./niches/rootpulse/

# Validate
benchscale validate biomeos-dev --niche rootpulse

# Cleanup
benchscale destroy biomeos-dev
```

### 3. CI/CD Pipeline

```yaml
# .github/workflows/test.yml
- name: Validate BiomeOS Niches
  run: |
    # Build benchScale
    cd primalTools/benchscale && cargo build --release
    
    # Create validation lab
    ./target/release/benchscale create ci-lab topologies/ci-validation.yaml
    
    # Deploy each niche
    for niche in niches/*/; do
      echo "Validating $niche..."
      ./target/release/benchscale deploy ci-lab "$niche"
      ./target/release/benchscale test ci-lab --niche $(basename "$niche")
    done
    
    # Cleanup
    ./target/release/benchscale destroy ci-lab
```

---

## Capability-Based Discovery

### BiomeOS Adapter for benchScale

```rust
// crates/biomeos-core/src/validation/benchscale_adapter.rs

use crate::primal_adapter::{ToolAdapter, ToolCapabilities};

pub struct BenchScaleAdapter {
    binary_path: PathBuf,
    capabilities: ToolCapabilities,
}

impl BenchScaleAdapter {
    /// Discover benchScale at runtime
    pub async fn discover() -> Result<Self> {
        // 1. Find binary (no hardcoding!)
        let binary_path = find_tool_binary("benchscale")?;
        
        // 2. Query capabilities
        let output = Command::new(&binary_path)
            .arg("--capabilities")
            .output()
            .await?;
        
        let capabilities = parse_capabilities(&output.stdout)?;
        
        Ok(Self { binary_path, capabilities })
    }
    
    /// Create validation lab for niche
    pub async fn create_validation_lab(
        &self,
        niche_name: &str,
        topology: &Path,
    ) -> Result<LabHandle> {
        if !self.capabilities.has("create_lab") {
            bail!("benchScale doesn't support lab creation");
        }
        
        // No hardcoding! Pass paths dynamically
        let output = Command::new(&self.binary_path)
            .arg("create")
            .arg(format!("biomeos-{}", niche_name))
            .arg(topology)
            .output()
            .await?;
        
        // Parse lab handle
        Ok(LabHandle::parse(&output.stdout)?)
    }
    
    /// Deploy niche into lab
    pub async fn deploy_niche(
        &self,
        lab: &LabHandle,
        niche_manifest: &Path,
    ) -> Result<()> {
        // benchScale runs BiomeOS in the lab
        // BiomeOS then deploys the niche
        // All validated!
        todo!()
    }
}
```

---

## Validation Topologies

### Example: RootPulse 3-Tower Federation

```yaml
# topologies/rootpulse-federation.yaml
name: rootpulse-federation
description: "3-tower RootPulse deployment validation"

nodes:
  - name: tower-1
    backend: libvirt  # Real VM!
    base_image: ubuntu-22.04
    ram_mb: 4096
    vcpus: 2
    services:
      - biomeos  # BiomeOS deployment
    environment:
      BIOMEOS_NICHE: rootpulse
      ROOTPULSE_ROLE: tower
      
  - name: tower-2
    backend: libvirt
    base_image: ubuntu-22.04
    ram_mb: 4096
    vcpus: 2
    services:
      - biomeos
    environment:
      BIOMEOS_NICHE: rootpulse
      ROOTPULSE_ROLE: tower
      
  - name: tower-3
    backend: libvirt
    base_image: ubuntu-22.04
    ram_mb: 4096
    vcpus: 2
    services:
      - biomeos
    environment:
      BIOMEOS_NICHE: rootpulse
      ROOTPULSE_ROLE: tower

network:
  type: lan
  latency: 10ms
  bandwidth: 1gbps

validation:
  # What to test
  tests:
    - name: primal_discovery
      description: "All primals discovered on each tower"
      
    - name: federation_established
      description: "Towers form federation via Songbird"
      
    - name: rootpulse_commit
      description: "Commit propagates across federation"
      
    - name: lineage_verification
      description: "BearDog lineage proofs valid"
      
  # Success criteria
  success:
    - all_primals_healthy: true
    - federation_size: 3
    - commit_sync_time_ms: <1000
    - lineage_valid: true
```

---

## Usage Examples

### 1. Validate RootPulse Niche

```bash
# Create validation lab
benchscale create rootpulse-test topologies/rootpulse-federation.yaml

# BiomeOS discovers benchScale and uses it
biomeos niche deploy rootpulse-niche.yaml --validate-with rootpulse-test

# benchScale reports:
# ✅ 3 towers created
# ✅ BiomeOS deployed on each
# ✅ RootPulse niche coordinated
# ✅ Federation established
# ✅ Validation tests passed
# ✅ Lab destroyed (cleanup)
```

### 2. Chaos Engineering

```bash
# Create lab
benchscale create chaos-test topologies/rootpulse-federation.yaml

# Deploy niche
biomeos niche deploy rootpulse-niche.yaml --target chaos-test

# Inject chaos
benchscale chaos inject chaos-test \
  --type network_partition \
  --nodes tower-1,tower-2 \
  --duration 30s

# Verify recovery
benchscale validate chaos-test --expect recovery

# Cleanup
benchscale destroy chaos-test
```

### 3. Performance Benchmarking

```bash
# Create benchmark lab
benchscale create perf-test topologies/rootpulse-single-tower.yaml

# Deploy and measure
biomeos niche deploy rootpulse-niche.yaml --target perf-test

# Run performance tests
benchscale benchmark perf-test \
  --workload commit_throughput \
  --duration 60s

# Results:
# Commits/sec: 1,234
# Latency p50: 10ms
# Latency p99: 45ms
# Memory: 2.1GB
# CPU: 45% (2 cores)
```

---

## Key Principles

### 1. No Hardcoding

```rust
// ❌ Wrong: Hardcoded path
let benchscale = Command::new("/usr/local/bin/benchscale");

// ✅ Right: Discovered at runtime
let benchscale_bin = find_tool_binary("benchscale")?;
let benchscale = Command::new(benchscale_bin);
```

### 2. Graceful Degradation

```rust
// BiomeOS works without benchScale
match BenchScaleAdapter::discover().await {
    Ok(adapter) => {
        // Enhanced validation available
        adapter.validate_niche(&manifest).await?;
    }
    Err(_) => {
        // No benchScale found - that's OK!
        tracing::warn!("benchScale not found - validation disabled");
        // Continue deployment without validation
    }
}
```

### 3. Capability-Based

```rust
// Don't assume what benchScale can do
if adapter.has_capability("inject_chaos") {
    adapter.inject_chaos(&scenario).await?;
} else {
    tracing::warn!("Chaos injection not available");
}
```

---

## Implementation Status

### Ready Now ✅
- ✅ benchScale v2.0.0 (production-ready)
- ✅ Docker backend (containers)
- ✅ libvirt backend (real VMs)
- ✅ CloudInit support
- ✅ 90.24% test coverage
- ✅ Topology definitions

### Needs Implementation 🔄
- 🔄 BiomeOS adapter for benchScale
- 🔄 Niche validation workflows
- 🔄 benchScale `--capabilities` flag
- 🔄 BiomeOS deployment in benchScale labs
- 🔄 Chaos injection scenarios
- 🔄 Performance benchmarking

---

## Next Steps

1. Add `--capabilities` flag to benchScale
2. Create BiomeOS adapter (`benchscale_adapter.rs`)
3. Implement niche validation workflow
4. Create validation topologies for each niche
5. Add benchScale integration to showcase
6. Document capability-based validation

---

**Status**: Integration design complete  
**benchScale**: Production-ready  
**BiomeOS Integration**: Ready to implement  

🔬 **Validation Through Capability-Based Discovery!**

