# 02 - Capability Composition Demo

**Duration**: 3 minutes  
**Prerequisites**: NestGate running, Songbird running (optional)

---

## Overview

This demo shows BiomeOS **composing multiple capabilities** to accomplish complex tasks.

**What it demonstrates**:
- Discover multiple capabilities simultaneously
- Compose primals without hardcoding
- Coordinate cross-primal workflows
- Graceful degradation when capabilities missing

---

## The Power of Composition

### Traditional Approach (Hardcoded)
```bash
# ❌ Hardcoded integration
nestgate_health=$(curl http://localhost:9020/health)
songbird_status=$(curl http://localhost:9000/status)
beardog encrypt --key /path/to/key --data myfile.txt

# Breaks when:
# - Ports change
# - Services move
# - APIs evolve
# - New primals added
```

### BiomeOS Approach (Capability-Based)
```bash
# ✅ Discover and compose
STORAGE=$(discover_capability "storage")
ORCHESTRATOR=$(discover_capability "orchestration")
CRYPTO=$(discover_capability "encryption")

# Compose them together
echo "Orchestrating encrypted storage..."
$ORCHESTRATOR coordinate \
  --storage $STORAGE \
  --encryption $CRYPTO \
  --operation "secure_backup"

# Works regardless of implementation!
```

---

## Run the Demo

```bash
cd showcase/00-substrate/02-capability-composition
./demo.sh
```

---

## What You'll See

### Phase 1: Discovery
```
🔍 Discovering required capabilities...
✅ Storage: http://localhost:9020 (NestGate)
✅ Orchestration: https://localhost:8080 (Songbird)  
✅ Encryption: /path/to/beardog (CLI)
```

### Phase 2: Composition
```
🎯 Composing workflow: Secure Data Storage

Step 1: Generate test data
  ✓ Created test payload (1KB)

Step 2: Encrypt with BearDog
  ✓ Encrypted via lineage-based keys

Step 3: Store via NestGate
  ✓ Stored in ZFS dataset with snapshots

Step 4: Coordinate via Songbird (if available)
  ✓ Registered in federation
  
✅ Secure storage complete!
```

### Phase 3: Verification
```
🔍 Verifying composition...
✅ Data stored in NestGate
✅ Encryption verified
✅ Coordination logged (if Songbird available)

📊 Composition Summary:
  Capabilities used: 3
  Primals coordinated: NestGate, BearDog, (Songbird)
  Hardcoded connections: 0
  Lines of glue code: 0
```

---

## Architecture

```
┌─────────────────────────────────────────┐
│      BiomeOS Composition Engine         │
│                                         │
│  1. Discover capabilities               │
│  2. Validate availability               │
│  3. Compose workflow                    │
│  4. Execute coordinated operations      │
└─────────────────────────────────────────┘
         │           │          │
         ▼           ▼          ▼
    ┌────────┐  ┌────────┐  ┌────────┐
    │Encrypt │  │ Store  │  │Coord.  │
    │BearDog │  │NestGate│  │Songbird│
    │  CLI   │  │  REST  │  │  mDNS  │
    └────────┘  └────────┘  └────────┘
```

---

## Use Cases Demonstrated

### Use Case 1: Encrypted Storage
```bash
# User: "Store this file securely"
# BiomeOS:
#   1. Discovers encryption capability (BearDog)
#   2. Discovers storage capability (NestGate)
#   3. Composes: encrypt → store
#   4. Returns handle
```

### Use Case 2: Coordinated Backup
```bash
# User: "Backup to federated storage"
# BiomeOS:
#   1. Discovers orchestration (Songbird)
#   2. Discovers storage (NestGate)
#   3. Composes: coordinate → replicate → verify
#   4. Federation handles distribution
```

### Use Case 3: Compute on Encrypted Data
```bash
# User: "Run analysis on private data"
# BiomeOS:
#   1. Discovers compute (Toadstool)
#   2. Discovers encryption (BearDog)
#   3. Discovers storage (NestGate)
#   4. Composes: retrieve → decrypt → compute → encrypt → store
#   5. Data never exposed in plaintext
```

---

## Key Concepts

### 1. Capability Graph
BiomeOS builds a graph of available capabilities:

```
       orchestration (Songbird)
            ↙     ↘
    storage        encryption
  (NestGate)       (BearDog)
      ↓                ↓
    ZFS         lineage-based
   snapshots      privacy
```

### 2. Composition Patterns

**Sequential**: A → B → C
```bash
generate_data | encrypt | store
```

**Parallel**: A ‖ B → C
```bash
(encrypt_data & compress_data) → store
```

**Coordinated**: Orchestrator → {A, B, C}
```bash
songbird_coordinate {
  nestgate: store_replica_1
  nestgate: store_replica_2
  beardog: verify_lineage
}
```

### 3. Zero Glue Code

Traditional systems need glue code:
```python
# ❌ Traditional
nestgate_client = NestgateClient("localhost:9020")
beardog_cli = BeardogCLI("/usr/bin/beardog")
songbird_api = SongbirdAPI("localhost:9000")

# Custom integration logic (100+ lines)
data = encrypt_with_beardog(beardog_cli, payload)
handle = store_with_nestgate(nestgate_client, data)
songbird_api.register(handle)
```

BiomeOS approach:
```bash
# ✅ BiomeOS
biomeOS execute \
  --workflow "secure_storage" \
  --discover-capabilities

# That's it! Zero glue code.
```

---

## Graceful Degradation

### Scenario: Songbird Not Available
```
🔍 Discovering capabilities...
✅ Storage: Available
✅ Encryption: Available
⚠️  Orchestration: Not available

📋 Workflow adjusted:
   Original: Encrypt → Store → Coordinate
   Adjusted: Encrypt → Store (local only)
   
✅ Workflow complete (degraded mode)
   Note: Federation coordination skipped
```

### Scenario: All Capabilities Available
```
🔍 Discovering capabilities...
✅ Storage: Available
✅ Encryption: Available
✅ Orchestration: Available

📋 Workflow: Full federation mode
   Encrypt → Store → Coordinate → Replicate
   
✅ Workflow complete (full federation)
```

---

## Technical Details

### Discovery Process
```bash
# 1. Query all capabilities
capabilities=$(discover_all_capabilities)

# 2. Filter by requirement
required=("storage" "encryption")
optional=("orchestration" "compute")

# 3. Build execution plan
if has_capability "orchestration"; then
    plan="federated"
else
    plan="local"
fi

# 4. Execute composed workflow
execute_plan $plan
```

### Composition Engine
```rust
// BiomeOS composition (simplified)
struct Workflow {
    required: Vec<Capability>,
    optional: Vec<Capability>,
    steps: Vec<Step>,
}

impl Workflow {
    async fn execute(&self) -> Result<()> {
        // Discover capabilities
        let available = discover_all().await?;
        
        // Validate requirements
        for cap in &self.required {
            if !available.contains(cap) {
                return Err("Missing required capability");
            }
        }
        
        // Execute steps with available capabilities
        for step in &self.steps {
            step.execute(&available).await?;
        }
        
        Ok(())
    }
}
```

---

## Success Criteria

✅ **Discovery works**: Finds all available capabilities  
✅ **Composition works**: Coordinates multiple primals  
✅ **Zero hardcoding**: No primal names in workflow  
✅ **Graceful degradation**: Works with subset of capabilities  
✅ **Extensible**: Adding new primals requires zero code changes  

---

## Real-World Applications

### Application 1: Secure Research Data Pipeline
```
User uploads sensitive research data
  ↓
BiomeOS discovers: storage + encryption + compute
  ↓
Workflow: Encrypt → Store → Coordinate compute jobs
  ↓
Results returned, source data never exposed
```

### Application 2: Federated Backup
```
User requests backup
  ↓
BiomeOS discovers: orchestration + storage (multiple)
  ↓
Songbird coordinates replication across towers
  ↓
NestGate stores encrypted shards
  ↓
Verify integrity via BearDog lineage
```

### Application 3: Privacy-Preserving Collaboration
```
Researcher A and B want to collaborate
  ↓
BiomeOS discovers: encryption + orchestration + compute
  ↓
BearDog: Lineage-based access control
Songbird: Coordinate federation
Toadstool: Execute joint analysis
  ↓
Results shared, raw data stays sovereign
```

---

## Next Steps

After this demo:
- **03-primal-evolution**: See how API changes don't break BiomeOS
- **04-custom-primals**: Add your own primal to the ecosystem
- **05-federation**: Multi-tower coordination

---

## Files in This Demo

- `demo.sh` - Main demonstration
- `README.md` - This file
- `workflows/secure_storage.sh` - Example workflow
- `validate.sh` - Validation script (for benchScale)

---

**Philosophy**: *"BiomeOS composes capabilities, not services. The result is a system that never breaks as primals evolve."*

