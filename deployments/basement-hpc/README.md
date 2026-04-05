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
| **Gate A** | `northgate.toml` | ✅ | GPU+CPU | i9-14900K, RTX 5090, 192GB | AI/LLM hub |
| **Gate B** | `southgate.toml` | ✅ | GPU+CPU | 5800X3D, RTX 3090, 128GB | Gaming + compute |
| **Gate C** | `eastgate.toml` | ✅ | GPU+CPU | i9-12900, RTX 3090, 32GB | Utility compute |
| **Gate D** | `westgate.toml` | ✅ | CPU | i7-4771, 76TB ZFS, 32GB | NAS + storage |
| **Gate E** | `primary.toml` | ✅ | CPU+GPU | Dual EPYC 64c, RTX 3070, 256GB | Bio pipeline |
| **Gate F** | `bluegate.toml` | ✅ | GPU+CPU | TBD, RTX 4070, 128GB | General compute |

### **Internet Nodes (2 machines)**

| Node | Config | Tower | Compute | Location | Specs |
|------|--------|-------|---------|----------|-------|
| **Gate G** | `flockgate.toml` | ✅ | GPU+CPU | Remote site | i9-13900K, RTX 3070Ti, 64GB |
| **Gate H** | `kingate.toml` | ✅ | GPU+CPU | Remote site | i7-6700K, RTX 3070, 32GB |

### **Mobile Nodes (2 devices)**

| Node | Config | Tower | Compute | Mobility | Specs |
|------|--------|-------|---------|----------|-------|
| **Gate I** | `swiftgate.toml` | ✅ | GPU+CPU | Portable | 5800X, RTX 3070, 64GB |
| **Pixel 8a** | `pixel8a.toml` | ✅ Lite | Mobile | Phone | Tensor G2, Android |

---

## 🚀 Deployment Process

### **Phase 1: Prepare Genetic Family**

```bash
cd "$BIOMEOS_REPO"  # e.g. ~/Development/ecoPrimals/primals/biomeOS

# Create genetic family seed for basement HPC (32 random bytes)
openssl rand -hex 16 > .family.seed
chmod 600 .family.seed

# Backup seed (CRITICAL!)
cp .family.seed ~/.secrets/basement-hpc-${FAMILY_ID}.seed
chmod 400 ~/.secrets/basement-hpc-${FAMILY_ID}.seed
```

### **Phase 2: Build LiveSpores (5 USBs)**

```bash
# Build 5 genetic sibling spores (requires parent spore first; use create_livespore.sh for initial)
for i in {1..5}; do
  SPORE_NAME=$(echo "alpha beta gamma delta epsilon" | cut -d' ' -f$i)
  ./scripts/create_sibling_spore.sh /path/to/parent/biomeOS /media/liveSpore$i node-$SPORE_NAME
done

# Verify all spores are genetic siblings
for spore in /media/liveSpore{1..5}/biomeOS; do
  biomeos verify-lineage "$spore" --detailed
done
```

### **Phase 3: Deploy LAN Towers (6 nodes)**

Deploy towers on each LAN machine:

```bash
# On Gate B (and similarly for other nodes)
export NODE_ID=tower-gate-b
export FAMILY_ID=${FAMILY_ID}
biomeos nucleus start --mode tower --node-id tower-gate-b
# Or graph-based: biomeos deploy graphs/tower_atomic_bootstrap.toml
```

### **Phase 4: Verify LAN Federation**

```bash
# From any LAN tower (e.g., Gate B)
curl --unix-socket /tmp/songbird-tower-gate-b.sock \
  -d '{"jsonrpc":"2.0","method":"discover_by_family","params":{"family_tags":["${FAMILY_ID}"],"timeout_ms":5000},"id":1}' \
  | jq '.result.nodes'

# Expected: 6 towers discovered
# - tower-gate-b
# - tower-gate-c
# - tower-gate-d
# - tower-gate-e
# - tower-gate-a
# - tower-gate-f
```

### **Phase 5: Deploy Compute Nodes (13 nodes on LAN)**

Deploy compute nodes on each machine (see individual config files).

**Example: Gate B**
```bash
# Compute nodes use biomeos nucleus or graph-based deployment
export NODE_ID=compute-gate-b-rtx5090
biomeos deploy graphs/node_atomic_compute.toml
```

Repeat for all LAN nodes using their respective config files.

### **Phase 6: Deploy Internet Nodes (2 nodes)**

```bash
# On Gate G (remote site)
export NODE_ID=tower-gate-g
export FAMILY_ID=${FAMILY_ID}
biomeos nucleus start --mode tower --node-id tower-gate-g

# On Gate H (remote site)
export NODE_ID=tower-gate-h
export FAMILY_ID=${FAMILY_ID}
biomeos nucleus start --mode tower --node-id tower-gate-h
```

### **Phase 7: Verify Internet Federation**

```bash
# From any LAN tower
curl --unix-socket /tmp/songbird-tower-gate-b.sock \
  -d '{"jsonrpc":"2.0","method":"discover_by_family","params":{"family_tags":["${FAMILY_ID}"],"timeout_ms":10000},"id":1}' \
  | jq '.result.nodes | map(.node_id)'

# Expected: 8 towers (6 LAN + 2 Internet)
```

### **Phase 8: Deploy Mobile Nodes (2 devices)**

```bash
# Gate I (portable laptop)
export NODE_ID=tower-gate-i
export FAMILY_ID=${FAMILY_ID}
biomeos nucleus start --mode tower --node-id tower-gate-i

# Pixel 8a (Android phone) - use tower-lite mode if available
export NODE_ID=tower-pixel8a
export FAMILY_ID=${FAMILY_ID}
biomeos nucleus start --mode tower --node-id tower-pixel8a
```

---

## 🧪 Testing & Verification

### **Test 1: Federation Discovery**
```bash
# Count federated towers
curl --unix-socket /tmp/songbird-tower-gate-b.sock \
  -d '{"jsonrpc":"2.0","method":"discover_by_family","params":{"family_tags":["${FAMILY_ID}"]},"id":1}' \
  | jq '.result.nodes | length'

# Expected: 10 (6 LAN + 2 Internet + 2 Mobile)
```

### **Test 2: Compute Node Discovery**
```bash
# List all compute nodes
for tower in gate-b gate-c gate-d gate-e gate-a gate-f; do
  curl --unix-socket /tmp/songbird-tower-$tower.sock \
    -d '{"jsonrpc":"2.0","method":"list_compute_nodes","params":{},"id":1}' \
    | jq ".result.nodes[] | {node_id, resource_type}"
done
```

### **Test 3: Submit Workload**
```bash
# Submit test workload to Gate B GPU
curl --unix-socket /tmp/compute-node-gate-b-rtx5090.sock \
  -d '{"jsonrpc":"2.0","method":"workload.submit","params":{"runtime":"native","code":"println!(\"Hello from RTX 5090!\");","language":"rust"},"id":1}'
```

### **Test 4: Genetic Lineage Verification**
```bash
# Verify all nodes share genetic lineage
for tower in gate-b gate-c gate-d gate-e gate-a gate-f gate-g gate-h gate-i; do
  biomeos verify-lineage /path/to/tower-$tower/spore --detailed
done
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
        - 'gate-b:8080'
        - 'gate-c:8080'
        - 'gate-d:8080'
        - 'gate-e:8080'
        - 'gate-a:8080'
        - 'gate-f:8080'

  # LAN Compute Nodes
  - job_name: 'compute-lan'
    static_configs:
      - targets:
        - 'gate-b:9091'  # GPU
        - 'gate-b:9092'  # CPU
        - 'gate-c:9091'
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

Create granular trust domains (via Songbird/BearDog family tags):
```bash
# Sub-federations use family tags; configure via tower.toml or Songbird discovery
# Example: set FAMILY_ID per sub-fed or use biomeos nucleus --node-id for tower identity
biomeos nucleus start --mode tower --node-id tower-gate-c
# Trust is established via genetic lineage (biomeos verify-lineage) and family_id
```

---

## 🎯 Use Cases

### **1. Distributed AI Training**
- **Nodes**: Gate A (5090), Gate B (3090), Gate C (3090)
- **Workload**: Spread LLM across 3 GPUs
- **Coordination**: Towers manage gradient sync

### **2. Gaming Federation**
- **Nodes**: Gate B, Gate F, Gate G, Gate H
- **Sub-fed**: `gaming`
- **Workload**: Multiplayer servers, game state sync

### **3. Bio Pipeline**
- **Node**: Gate E (64 cores, 256GB ECC)
- **Storage**: Gate D (76TB ZFS)
- **Workload**: Alignment, Kraken2, preprocessing

---

## 📝 Notes

- All configs use environment variables for flexibility
- Genetic lineage (`${FAMILY_ID}`) provides baseline trust
- Sub-federations enable granular access control
- Mobile nodes bridge LAN ↔ Internet

**Status:** ✅ **Ready for deployment!**

