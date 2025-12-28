# 🧹 Showcase Cleanup & Organization Plan

**Date**: December 28, 2025  
**Goal**: Clean showcase docs, establish runtime discovery patterns, integrate benchScale validation

---

## 🎯 Core Principles

### 1. Zero Hardcoding - Runtime Discovery Only
```bash
# ❌ BAD - Hardcoded
SONGBIRD_ENDPOINT="http://localhost:9000"

# ✅ GOOD - Runtime discovery
SONGBIRD=$(discover_capability "discovery") # Finds any discovery provider
```

**Why**: As primals evolve, change APIs, or users compose new primals, NO code changes needed.

### 2. Primals Have Only Self-Knowledge
```bash
# Primal only knows:
- Its own capabilities ("I provide 'storage'")
- Its own API endpoints ("I serve on :9020")
- Its own health status

# Primal does NOT know:
- Other primals' existence
- Other primals' endpoints
- How to coordinate (biomeOS does this)
```

### 3. benchScale Validates Deployments
```bash
# Every showcase should be deployable via benchScale
benchscale deploy --topology showcase/01-nestgate-demo.yaml
benchscale validate --test showcase/01-nestgate-demo-test.sh
benchscale destroy
```

---

## 📁 New Clean Structure

### Archive Old Docs (Keep for History)
```bash
showcase/
├── archive/                            ← Move all dated docs here
│   ├── 2025-12-24/
│   ├── 2025-12-25/
│   ├── 2025-12-26/
│   ├── 2025-12-27/
│   └── README.md                       ← Index of archived content
```

### New Active Structure
```bash
showcase/
├── README.md                           ← Main entry (clean, current)
├── NO_MOCKS_POLICY.md                  ← Keep (enforced)
├── RUNTIME_DISCOVERY.md                ← NEW: Discovery patterns
│
├── 00-substrate/                       ← biomeOS as substrate
│   ├── 01-hello-biomeos/
│   │   ├── demo.sh
│   │   ├── topology.yaml               ← benchScale deployment
│   │   ├── validate.sh                 ← benchScale validation
│   │   └── README.md
│   ├── 02-deploy-primal/
│   └── 03-runtime-discovery/
│
├── 01-nestgate/                        ← Our local primal showcase
│   ├── 01-hello-nestgate/
│   │   ├── demo.sh                     ← Runtime discovery
│   │   ├── topology.yaml               ← benchScale deployment
│   │   ├── validate.sh
│   │   └── README.md
│   ├── 02-lineage-tracking/
│   └── 03-federation/
│
├── 02-birdsong-p2p/                    ← BirdSong/BTSP deployment
│   ├── 01-deploy-songbird/
│   ├── 02-deploy-beardog-btsp/
│   ├── 03-encrypted-discovery/
│   └── 04-multi-tower-mesh/
│
├── 03-multi-primal/                    ← Multi-primal coordination
│   ├── 01-songbird-nestgate/
│   ├── 02-beardog-nestgate/
│   └── 03-full-ecosystem/
│
├── common/                             ← Shared utilities
│   ├── discovery.sh                    ← Runtime discovery functions
│   ├── deploy.sh                       ← Deployment helpers
│   ├── validate.sh                     ← Validation helpers
│   └── benchscale-helpers.sh           ← benchScale integration
│
└── archive/                            ← Historical docs
    └── [all dated summary docs]
```

---

## 🔧 Cleanup Actions

### Phase 1: Archive Historical Docs (30 min)
```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS/showcase

# Create archive structure
mkdir -p archive/2025-12-{24,25,26,27,28}

# Move dated docs
mv *DEC_24_2025.md archive/2025-12-24/
mv *DEC_25_2025.md archive/2025-12-25/
mv *DEC_26_2025.md archive/2025-12-26/
mv *DEC_27_2025.md archive/2025-12-27/
mv *DEC_28_2025.md archive/2025-12-28/

# Move session summaries
mv SESSION_*.md archive/
mv EXECUTION_*.md archive/
mv VERIFICATION_*.md archive/
mv IMPLEMENTATION_*.md archive/

# Move gap reports (keep recent one)
mv GAPS_DISCOVERED_DEC_24_2025.md archive/2025-12-24/
mv GAPS_SUMMARY_DEC_24_2025.md archive/2025-12-24/

# Keep only:
# - README.md
# - NO_MOCKS_POLICY.md
# - QUICK_ACTION_PLAN_DEC_28_2025.md (most recent)
# - SHOWCASE_BUILDOUT_PLAN_DEC_28_2025.md (most recent)
```

### Phase 2: Create Archive Index (15 min)
```bash
cat > archive/README.md << 'EOF'
# Showcase Archive

Historical documentation and session reports.

## By Date
- **2025-12-24**: Initial showcase buildout
- **2025-12-25**: Christmas milestone - 100% success
- **2025-12-26**: API adapter completion
- **2025-12-27**: P2P coordination implementation
- **2025-12-28**: Comprehensive audit and buildout plan

## Key Milestones
- ✅ NO MOCKS policy enforced (Dec 28)
- ✅ 100% test pass rate achieved (Dec 27)
- ✅ P2P coordination complete (Dec 27)
- ✅ API adapters for all Phase1 primals (Dec 26)

See individual dated folders for complete documentation.
EOF
```

### Phase 3: Create Runtime Discovery Guide (45 min)
```bash
cat > RUNTIME_DISCOVERY.md << 'EOF'
# Runtime Discovery Patterns

**Principle**: Zero hardcoding. Primals discover each other at runtime.

## Discovery Methods

### 1. Capability-Based Discovery
```bash
# Discover by capability, not by name
STORAGE_PROVIDER=$(discover_capability "storage")
# Returns: URL of ANY primal providing "storage"
# Could be: NestGate, MinIO, S3, custom storage primal

COMPUTE_PROVIDER=$(discover_capability "compute")
# Could be: ToadStool, Lambda, custom compute primal
```

### 2. mDNS Discovery
```bash
# Automatic local network discovery
primals=$(mdns_discover "_primal._tcp.local")
# Returns: All primals advertising on local network
```

### 3. Service Registry
```bash
# Query Songbird's registry (if available)
primals=$(curl http://discovery-service/api/v1/services)
# Fallback if Songbird not available: use mDNS
```

### 4. Environment Override (Dev Only)
```bash
# For development/testing only
export STORAGE_ENDPOINT="http://localhost:9020"
# Production uses discovery
```

## Example: Multi-Primal Workflow

```bash
#!/usr/bin/env bash
source common/discovery.sh

# Discover needed capabilities at runtime
STORAGE=$(discover_capability "storage" || fallback_to_filesystem)
COMPUTE=$(discover_capability "compute" || fallback_to_local)
SECURITY=$(discover_capability "security" || fallback_to_os_crypto)

# Store data
store_data "$STORAGE" "my-data.txt"

# Process with compute
result=$(process_data "$COMPUTE" "$STORAGE/my-data.txt")

# Encrypt result
encrypted=$(encrypt_data "$SECURITY" "$result")

# No primal names hardcoded!
# Works with ANY implementation of these capabilities
```

## Why This Matters

**Scenario 1: Primal Evolution**
- NestGate v2.0 changes API from `/api/v1/data` to `/data`
- biomeOS discovery adapts automatically
- Zero code changes needed

**Scenario 2: Custom Primals**
- User creates "MyStorage" primal
- Advertises "storage" capability
- biomeOS discovers and uses it
- Zero configuration needed

**Scenario 3: Multiple Providers**
- 3 ToadStool instances for compute
- biomeOS load balances automatically
- Failover if one goes down
- Zero hardcoded endpoints

## Anti-Patterns (DO NOT DO THIS)

```bash
# ❌ Hardcoded primal name
if [ -f "/usr/bin/nestgate" ]; then

# ❌ Hardcoded endpoint
NESTGATE_URL="http://localhost:9020"

# ❌ Hardcoded API path
curl http://nestgate:9020/api/v1/datasets

# ✅ Instead:
STORAGE=$(discover_capability "storage")
store_data "$STORAGE" "dataset-name"
```
EOF
```

---

## 🏗️ benchScale Integration

### Create Topology Files for Each Demo

**Example**: `01-nestgate/01-hello-nestgate/topology.yaml`
```yaml
# benchScale deployment topology
name: hello-nestgate
description: NestGate standalone demo

nodes:
  - name: nestgate-node
    image: ubuntu:22.04
    resources:
      cpu: 2
      memory: 4096
    services:
      - name: nestgate
        binary: ../../../../primals/nestgate
        port: 9020
        health_check: http://localhost:9020/health
        capabilities:
          - storage
          - lineage

tests:
  - name: health-check
    command: curl -f http://nestgate-node:9020/health
  
  - name: store-data
    command: |
      curl -X POST http://nestgate-node:9020/api/v1/datasets \
        -d '{"name":"test","data":"hello"}'
  
  - name: retrieve-data
    command: |
      curl http://nestgate-node:9020/api/v1/datasets/test | \
        jq -e '.data == "hello"'
```

### Create Validation Scripts

**Example**: `01-nestgate/01-hello-nestgate/validate.sh`
```bash
#!/usr/bin/env bash
# benchScale validation script
set -e

echo "🧪 Validating NestGate deployment..."

# Use benchScale to deploy
cd "$(dirname "$0")"
TOPO="topology.yaml"

echo "Step 1: Deploy via benchScale"
../../../../primalTools/benchscale/benchscale deploy --topology "$TOPO"

echo "Step 2: Wait for healthy"
timeout 30 bash -c 'until curl -f http://nestgate-node:9020/health; do sleep 1; done'

echo "Step 3: Run demo"
./demo.sh

echo "Step 4: Validate results"
# Check data was stored
result=$(curl -s http://nestgate-node:9020/api/v1/datasets/test)
echo "$result" | jq -e '.data != null'

echo "Step 5: Cleanup"
../../../../primalTools/benchscale/benchscale destroy

echo "✅ Validation complete!"
```

### Integration with CI/CD
```yaml
# .github/workflows/showcase-validation.yml
name: Showcase Validation

on: [push, pull_request]

jobs:
  validate-showcases:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Install benchScale
        run: |
          cd primalTools/benchscale
          cargo build --release
      
      - name: Validate Each Showcase
        run: |
          cd phase2/biomeOS/showcase
          for demo in */*/validate.sh; do
            echo "Validating $demo"
            bash "$demo"
          done
```

---

## 📋 Cleanup Script

Create `cleanup-showcase.sh`:
```bash
#!/usr/bin/env bash
# Cleanup and reorganize showcase

set -e
cd "$(dirname "$0")"

echo "🧹 Cleaning up showcase documentation..."

# Create archive directories
mkdir -p archive/2025-12-{24,25,26,27,28}

# Archive dated docs
echo "Archiving dated documentation..."
find . -maxdepth 1 -name "*DEC_24_2025.md" -exec mv {} archive/2025-12-24/ \;
find . -maxdepth 1 -name "*DEC_25_2025.md" -exec mv {} archive/2025-12-25/ \;
find . -maxdepth 1 -name "*DEC_26_2025.md" -exec mv {} archive/2025-12-26/ \;
find . -maxdepth 1 -name "*DEC_27_2025.md" -exec mv {} archive/2025-12-27/ \;
find . -maxdepth 1 -name "*DEC_28_2025.md" -exec mv {} archive/2025-12-28/ \;

# Archive session reports
echo "Archiving session reports..."
mv SESSION_*.md archive/ 2>/dev/null || true
mv EXECUTION_*.md archive/ 2>/dev/null || true
mv VERIFICATION_*.md archive/ 2>/dev/null || true
mv IMPLEMENTATION_*.md archive/ 2>/dev/null || true
mv COMPLETE_*.md archive/ 2>/dev/null || true
mv FINAL_*.md archive/ 2>/dev/null || true
mv PROGRESS_*.md archive/ 2>/dev/null || true
mv SUCCESS_*.md archive/ 2>/dev/null || true
mv PHASE*.md archive/ 2>/dev/null || true

# Keep only current/important docs
echo "Keeping current documentation:"
echo "  ✓ README.md"
echo "  ✓ NO_MOCKS_POLICY.md"
echo "  ✓ QUICK_ACTION_PLAN_DEC_28_2025.md"
echo "  ✓ SHOWCASE_BUILDOUT_PLAN_DEC_28_2025.md"

# Create archive index
cat > archive/README.md << 'EOF'
# Showcase Archive

Historical documentation from showcase development.

## Timeline
- **2025-12-24**: Initial buildout
- **2025-12-25**: 100% test success milestone
- **2025-12-26**: API adapter completion
- **2025-12-27**: P2P coordination complete
- **2025-12-28**: Comprehensive audit & new buildout plan

See dated folders for complete documentation.
EOF

echo "✅ Cleanup complete!"
echo ""
echo "Active docs:"
ls -1 *.md | grep -v archive
echo ""
echo "Archived docs: $(find archive -name "*.md" | wc -l) files"
```

---

## 🚀 Next Steps

### Today (2 hours)
1. **Run cleanup** (30 min)
   ```bash
   cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS/showcase
   bash cleanup-showcase.sh
   ```

2. **Create runtime discovery guide** (45 min)
   - Write `RUNTIME_DISCOVERY.md`
   - Create `common/discovery.sh` utilities

3. **Update main README** (45 min)
   - Clean, current entry point
   - Link to buildout plan
   - Link to runtime discovery guide

### This Week (8 hours)
1. **Create benchScale topology** for each demo (2 hours)
2. **Write validation scripts** (2 hours)
3. **Test with real primals** (2 hours)
4. **Create first substrate demo** (2 hours)

### Next Week (12 hours)
- Build out 00-substrate/ demos
- Build out 01-nestgate/ demos
- Build out 02-birdsong-p2p/ demos
- Full benchScale validation

---

## 📊 Success Metrics

### Documentation Cleanup
- [ ] < 10 docs in root showcase/
- [ ] All dated docs archived
- [ ] Clean README.md entry point
- [ ] Runtime discovery guide complete

### benchScale Integration
- [ ] Every demo has topology.yaml
- [ ] Every demo has validate.sh
- [ ] All demos deployable via benchScale
- [ ] CI/CD validation pipeline

### Runtime Discovery
- [ ] Zero hardcoded primal endpoints
- [ ] Zero hardcoded primal names
- [ ] Capability-based discovery only
- [ ] Works with ANY implementation

---

## 🎯 Philosophy

**Zero Hardcoding**:
> "If a primal name appears in biomeOS code, we failed.  
>  If an endpoint is hardcoded, we failed.  
>  Discover at runtime, adapt to evolution."

**benchScale Validation**:
> "Every showcase must be a real, deployable topology.  
>  If it can't be deployed via benchScale, it's not validated.  
>  Demos that work in dev but fail in deployment prove nothing."

**Primal Self-Knowledge**:
> "Primals know themselves, not others.  
>  'I provide storage' - not 'I connect to Songbird'.  
>  biomeOS handles all coordination."

---

**Status**: Ready to Execute  
**Next**: Run cleanup script, create discovery guide  
**Timeline**: 2 hours today, full implementation this week

🧹 **Let's clean up and build the right foundation!** 🌱

