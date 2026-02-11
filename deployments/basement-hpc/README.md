# 🏗️ Basement HPC Deployment Configurations

**Purpose:** Production deployment manifests for basement HPC infrastructure  
**Hardware Cost:** ~$15,000  
**Nodes:** 10 machines (6 LAN, 2 Internet, 2 Mobile)  
**Status:** Production Ready

---

## 📋 Quick Reference

### **LAN Nodes (6 machines)**

| Node | Config | Tower | Compute | Specs | Purpose |
|------|--------|-------|---------|-------|---------|
| **Northgate** | `northgate.toml` | ✅ | GPU+CPU | i9-14900K, RTX 5090, 192GB | AI/LLM hub |
| **Southgate** | `southgate.toml` | ✅ | GPU+CPU | 5800X3D, RTX 3090, 128GB | Gaming + compute |
| **Eastgate** | `eastgate.toml` | ✅ | GPU+CPU | i9-12900, RTX 3090, 32GB | Utility compute |
| **Westgate** | `westgate.toml` | ✅ | CPU | i7-4771, 76TB ZFS, 32GB | NAS + storage |
| **Strandgate** | `strandgate.toml` | ✅ | CPU+GPU | Dual EPYC 64c, RTX 3070, 256GB | Bio pipeline |
| **BlueGate** | `bluegate.toml` | ✅ | GPU+CPU | TBD, RTX 4070, 128GB | General compute |

### **Internet Nodes (2 machines)**

| Node | Config | Tower | Compute | Location | Specs |
|------|--------|-------|---------|----------|-------|
| **FlockGate** | `flockgate.toml` | ✅ | GPU+CPU | Brother's house | i9-13900K, RTX 3070Ti, 64GB |
| **KinGate** | `kingate.toml` | ✅ | GPU+CPU | Family | i7-6700K, RTX 3070, 32GB |

### **Mobile Nodes (2 devices)**

| Node | Config | Tower | Compute | Mobility | Specs |
|------|--------|-------|---------|----------|-------|
| **Swiftgate** | `swiftgate.toml` | ✅ | GPU+CPU | Portable | 5800X, RTX 3070, 64GB |
| **Pixel 8a** | `pixel8a.toml` | ✅ Lite | Mobile | Phone | Tensor G2, Android |

---

## 🚀 Deployment Process

### **Phase 1: Prepare Genetic Family**

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# Create genetic family seed for basement HPC
cargo run --bin spore-tools -- create-family \
  --family-id ${FAMILY_ID} \
  --description "Basement HPC - LAN + Internet + Mobile" \
  --output .family.seed

# Backup seed (CRITICAL!)
cp .family.seed ~/.secrets/basement-hpc-${FAMILY_ID}.seed
chmod 400 ~/.secrets/basement-hpc-${FAMILY_ID}.seed
```

### **Phase 2: Build LiveSpores (5 USBs)**

```bash
# Build 5 genetic sibling spores
for i in {1..5}; do
  SPORE_NAME=$(echo "alpha beta gamma delta epsilon" | cut -d' ' -f$i)
  cargo run --bin biomeos-spore -- build \
    --family-seed .family.seed \
    --niche tower \
    --spore-id node-$SPORE_NAME \
    --output /media/liveSpore$i \
    --fresh-bins \
    --verify
done

# Verify all spores are genetic siblings
cargo run --bin spore-tools -- verify-lineage \
  --spores /media/liveSpore{1..5} \
  --family-id ${FAMILY_ID}
```

### **Phase 3: Deploy LAN Towers (6 nodes)**

Deploy towers on each LAN machine:

```bash
# On Northgate
export NODE_ID=tower-northgate
export FAMILY_ID=${FAMILY_ID}
biomeos deploy --niche tower --config deployments/basement-hpc/northgate.toml --usb /media/liveSpore1

# On Southgate
export NODE_ID=tower-southgate
export FAMILY_ID=${FAMILY_ID}
biomeos deploy --niche tower --config deployments/basement-hpc/southgate.toml --usb /media/liveSpore1

# On Eastgate
export NODE_ID=tower-eastgate
export FAMILY_ID=${FAMILY_ID}
biomeos deploy --niche tower --config deployments/basement-hpc/eastgate.toml --usb /media/liveSpore1

# On Westgate
export NODE_ID=tower-westgate
export FAMILY_ID=${FAMILY_ID}
biomeos deploy --niche tower --config deployments/basement-hpc/westgate.toml --usb /media/liveSpore1

# On Strandgate
export NODE_ID=tower-strandgate
export FAMILY_ID=${FAMILY_ID}
biomeos deploy --niche tower --config deployments/basement-hpc/strandgate.toml --usb /media/liveSpore1

# On BlueGate
export NODE_ID=tower-bluegate
export FAMILY_ID=${FAMILY_ID}
biomeos deploy --niche tower --config deployments/basement-hpc/bluegate.toml --usb /media/liveSpore1
```

### **Phase 4: Verify LAN Federation**

```bash
# From any LAN tower (e.g., Northgate)
curl --unix-socket /tmp/songbird-tower-northgate.sock \
  -d '{"jsonrpc":"2.0","method":"discover_by_family","params":{"family_tags":["${FAMILY_ID}"],"timeout_ms":5000},"id":1}' \
  | jq '.result.nodes'

# Expected: 6 towers discovered
# - tower-northgate
# - tower-southgate
# - tower-eastgate
# - tower-westgate
# - tower-strandgate
# - tower-bluegate
```

### **Phase 5: Deploy Compute Nodes (13 nodes on LAN)**

Deploy compute nodes on each machine (see individual config files).

**Example: Northgate**
```bash
# GPU node
export NODE_ID=compute-northgate-rtx5090
export RESOURCE_TYPE=gpu
export RESOURCE_ID=0
biomeos deploy --niche compute-node --config deployments/basement-hpc/northgate.toml

# CPU node
export NODE_ID=compute-northgate-i9
export RESOURCE_TYPE=cpu
biomeos deploy --niche compute-node --config deployments/basement-hpc/northgate.toml
```

Repeat for all LAN nodes using their respective config files.

### **Phase 6: Deploy Internet Nodes (2 nodes)**

```bash
# On FlockGate (brother's house)
export NODE_ID=tower-flockgate
export FAMILY_ID=${FAMILY_ID}
export INTERNET_ENABLED=true
biomeos deploy --niche tower --config deployments/basement-hpc/flockgate.toml --usb /media/liveSpore2

# On KinGate (family)
export NODE_ID=tower-kingate
export FAMILY_ID=${FAMILY_ID}
export INTERNET_ENABLED=true
biomeos deploy --niche tower --config deployments/basement-hpc/kingate.toml --usb /media/liveSpore3
```

### **Phase 7: Verify Internet Federation**

```bash
# From any LAN tower
curl --unix-socket /tmp/songbird-tower-northgate.sock \
  -d '{"jsonrpc":"2.0","method":"discover_by_family","params":{"family_tags":["${FAMILY_ID}"],"timeout_ms":10000},"id":1}' \
  | jq '.result.nodes | map(.node_id)'

# Expected: 8 towers (6 LAN + 2 Internet)
```

### **Phase 8: Deploy Mobile Nodes (2 devices)**

```bash
# Swiftgate (portable laptop)
export NODE_ID=tower-swiftgate
export FAMILY_ID=${FAMILY_ID}
export MOBILE_MODE=true
biomeos deploy --niche tower --config deployments/basement-hpc/swiftgate.toml --usb /media/liveSpore4

# Pixel 8a (Android phone)
# Use Termux or Android app
export NODE_ID=tower-pixel8a
export FAMILY_ID=${FAMILY_ID}
export MOBILE_MODE=true
export BEARDOG_HSM_MODE=hardware
biomeos deploy --niche tower-lite --config deployments/basement-hpc/pixel8a.toml --usb /media/liveSpore5
```

---

## 🧪 Testing & Verification

### **Test 1: Federation Discovery**
```bash
# Count federated towers
curl --unix-socket /tmp/songbird-tower-northgate.sock \
  -d '{"jsonrpc":"2.0","method":"discover_by_family","params":{"family_tags":["${FAMILY_ID}"]},"id":1}' \
  | jq '.result.nodes | length'

# Expected: 10 (6 LAN + 2 Internet + 2 Mobile)
```

### **Test 2: Compute Node Discovery**
```bash
# List all compute nodes
for tower in northgate southgate eastgate westgate strandgate bluegate; do
  curl --unix-socket /tmp/songbird-tower-$tower.sock \
    -d '{"jsonrpc":"2.0","method":"list_compute_nodes","params":{},"id":1}' \
    | jq ".result.nodes[] | {node_id, resource_type}"
done
```

### **Test 3: Submit Workload**
```bash
# Submit test workload to Northgate GPU
curl --unix-socket /tmp/compute-node-northgate-rtx5090.sock \
  -d '{"jsonrpc":"2.0","method":"workload.submit","params":{"runtime":"native","code":"println!(\"Hello from RTX 5090!\");","language":"rust"},"id":1}'
```

### **Test 4: Genetic Lineage Verification**
```bash
# Verify all nodes share genetic lineage
cargo run --bin spore-tools -- verify-deployed \
  --family-id ${FAMILY_ID} \
  --towers tower-{northgate,southgate,eastgate,westgate,strandgate,bluegate,flockgate,kingate,swiftgate,pixel8a}
```

---

## 📊 Monitoring

### **Prometheus Metrics**

Scrape targets (add to `prometheus.yml`):
```yaml
scrape_configs:
  # LAN Towers
  - job_name: 'towers-lan'
    static_configs:
      - targets:
        - 'northgate:8080'
        - 'southgate:8080'
        - 'eastgate:8080'
        - 'westgate:8080'
        - 'strandgate:8080'
        - 'bluegate:8080'

  # LAN Compute Nodes
  - job_name: 'compute-lan'
    static_configs:
      - targets:
        - 'northgate:9091'  # GPU
        - 'northgate:9092'  # CPU
        - 'southgate:9091'
        # ... etc
```

### **Grafana Dashboards**

Import dashboards:
- `biomeOS-tower-dashboard.json` (tower federation)
- `toadstool-compute-dashboard.json` (workload execution)
- `beardog-security-dashboard.json` (crypto operations)

---

## 🔐 Security

### **Genetic Family Seed**
- **CRITICAL**: Backup `.family.seed` securely
- **Storage**: `~/.secrets/basement-hpc-${FAMILY_ID}.seed` (chmod 400)
- **Recovery**: Store on SoloKeys (FIDO2/HSM)

### **Sub-Federations**

Create granular trust domains:
```bash
# Gaming sub-federation
cargo run --bin subfed-tools -- create \
  --parent-family ${FAMILY_ID} \
  --subfed-name gaming \
  --members tower-southgate,tower-bluegate,tower-flockgate,tower-kingate

# Bio pipeline sub-federation
cargo run --bin subfed-tools -- create \
  --parent-family ${FAMILY_ID} \
  --subfed-name bio-pipeline \
  --members tower-strandgate,tower-westgate
```

---

## 🎯 Use Cases

### **1. Distributed AI Training**
- **Nodes**: Northgate (5090), Southgate (3090), Eastgate (3090)
- **Workload**: Spread LLM across 3 GPUs
- **Coordination**: Towers manage gradient sync

### **2. Gaming Federation**
- **Nodes**: Southgate, BlueGate, FlockGate, KinGate
- **Sub-fed**: `gaming`
- **Workload**: Multiplayer servers, game state sync

### **3. Bio Pipeline**
- **Node**: Strandgate (64 cores, 256GB ECC)
- **Storage**: Westgate (76TB ZFS)
- **Workload**: Alignment, Kraken2, preprocessing

---

## 📝 Notes

- All configs use environment variables for flexibility
- Genetic lineage (`${FAMILY_ID}`) provides baseline trust
- Sub-federations enable granular access control
- Mobile nodes bridge LAN ↔ Internet

**Status:** ✅ **Ready for deployment!**

