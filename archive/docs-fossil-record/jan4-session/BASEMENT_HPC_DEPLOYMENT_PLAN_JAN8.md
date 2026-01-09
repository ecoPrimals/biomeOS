# 🏗️ Basement HPC Deployment Plan - ecoPrimals Infrastructure

**Date:** January 8, 2026  
**Status:** 🎯 **DEPLOYMENT PLANNING**  
**Hardware Cost:** ~$15,000  
**Nodes:** 11 physical machines + 4 USB spores

---

## 📊 Hardware Inventory & Classification

### **LAN Nodes (6 machines - High-Speed Local)**

| Node | Tower | Compute Nodes | Specs | Role |
|------|-------|---------------|-------|------|
| **Northgate** | ✅ | GPU(5090) + CPU(i9-14900K) | 192GB DDR5 | Flagship AI/LLM hub |
| **Southgate** | ✅ | GPU(3090) + CPU(5800X3D) | 128GB DDR4 | Gaming + heavy compute |
| **Eastgate** | ✅ | GPU(3090) + CPU(i9-12900) | 32GB DDR5 | Utility compute, PCIe |
| **Westgate** | ✅ | CPU(i7-4771) | 32GB DDR3, 76TB ZFS | NAS + cold storage |
| **Strandgate** | ✅ | GPU(3070) + CPU(Dual EPYC 64c) | 256GB ECC | Bio pipeline |
| **BlueGate** | ✅ | GPU(4070) + CPU(TBD) | 128GB DDR4 | General compute |

**Total LAN:**
- **6 Towers** (federation, discovery, P2P)
- **9 Compute Nodes** (5 GPU + 4 CPU)
- **768GB RAM**
- **~80TB storage**

### **Internet Nodes (2 machines - Remote Federation)**

| Node | Tower | Compute Nodes | Location | Specs |
|------|-------|---------------|----------|-------|
| **FlockGate** | ✅ | GPU(3070Ti) + CPU(i9-13900K) | Brother's house | 64GB DDR5 |
| **KinGate** | ✅ | GPU(3070) + CPU(i7-6700K) | Family member | 32GB DDR4 |

**Total Internet:**
- **2 Towers** (federated via BTSP genetic lineage)
- **4 Compute Nodes** (2 GPU + 2 CPU)
- **96GB RAM**

### **Mobile Nodes (2 machines - Portable)**

| Node | Tower | Compute Nodes | Mobility | Specs |
|------|-------|---------------|----------|-------|
| **Swiftgate** | ✅ | GPU(3070) + CPU(5800X) | Laptop/Portable | 64GB DDR4 |
| **Pixel 8a** | ✅ (Lite) | Mobile CPU | Phone | Tensor G2, Android |

**Purpose:**
- **Swiftgate**: Mobile compute, LAN→Internet bridge
- **Pixel 8a**: BearDog HSM seed, mobile mesh node

### **Hardware Security (4 devices)**
- **4x SoloKeys**: FIDO2/HSM roots for BearDog key hierarchy

---

## 🏗️ Deployment Architecture

### **Design Philosophy**

```
┌─────────────────────────────────────────────────────────────┐
│                   BASEMENT HPC MESH                          │
│                                                              │
│  ┌────────────────────────────────────────────────────┐    │
│  │           LAN Federation (6 towers)                 │    │
│  │                                                      │    │
│  │  Northgate ←→ Southgate ←→ Eastgate ←→ Westgate    │    │
│  │       ↕              ↕                                │    │
│  │  Strandgate    BlueGate                             │    │
│  │                                                      │    │
│  │  Discovery: UDP multicast + Songbird mesh           │    │
│  │  Comms: BTSP genetic tunnels                        │    │
│  │  Trust: Genetic family (nat0)                       │    │
│  └────────────────────────────────────────────────────┘    │
│                         ↕                                    │
│              Swiftgate (Mobile Bridge)                       │
│                         ↕                                    │
│  ┌────────────────────────────────────────────────────┐    │
│  │        Internet Federation (2 towers)               │    │
│  │                                                      │    │
│  │      FlockGate (Brother) ←→ KinGate (Family)       │    │
│  │                                                      │    │
│  │  Discovery: Songbird P2P via BTSP                   │    │
│  │  NAT Traversal: Genetic lineage BirdSong           │    │
│  │  Trust: Same genetic family (nat0)                  │    │
│  └────────────────────────────────────────────────────┘    │
│                                                              │
│  ┌────────────────────────────────────────────────────┐    │
│  │     Compute Fabric (15 total compute nodes)         │    │
│  │                                                      │    │
│  │  GPU Nodes (9): AI, LLM, rendering, ML training     │    │
│  │  CPU Nodes (4): Pipelines, storage, orchestration   │    │
│  │  Mobile (2): Bridge, HSM, portable compute          │    │
│  │                                                      │    │
│  │  Workload Manager: Toadstool on each node           │    │
│  │  Security: BearDog crypto lock                      │    │
│  │  Coordination: Towers manage discovery              │    │
│  └────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

---

## 🎯 Deployment Strategy

### **Phase 1: LAN Foundation (6 towers)**

#### **Tower Deployment (Communication Stack)**
Each LAN node gets a **Tower** for federation:

```toml
# tower.toml (for each LAN node)
[niche]
name = "tower"
type = "communication"
family_seed_file = "./.family.seed"  # Genetic lineage: nat0

[[primals]]
binary = "./primals/songbird-orchestrator"  # Discovery, P2P
binary = "./primals/beardog-server"         # Security, crypto
binary = "./primals/biomeos-spore"          # Orchestration
```

**Deployment:**
```bash
# On each LAN machine (Northgate, Southgate, etc.)
export NODE_ID=tower-northgate  # tower-southgate, tower-eastgate, etc.
export FAMILY_ID=nat0           # Same genetic family
biomeos deploy --niche tower --usb /media/liveSpore1
```

**Result:**
- 6 towers federated on LAN
- UDP multicast discovery
- Genetic trust via BearDog
- BTSP P2P tunnels

### **Phase 2: Compute Nodes (15 nodes)**

#### **GPU Compute Nodes (9 total)**

**Northgate** (Flagship AI)
```bash
export NODE_ID=compute-northgate-gpu5090
export FAMILY_ID=nat0
export RESOURCE_TYPE=gpu
export RESOURCE_ID=0
export GPU_MEMORY_GB=24
export TOADSTOOL_SECURE_MODE=true  # BearDog crypto lock
biomeos deploy --niche compute-node
```

**Southgate** (Heavy compute)
```bash
export NODE_ID=compute-southgate-gpu3090
export FAMILY_ID=nat0
export RESOURCE_TYPE=gpu
export RESOURCE_ID=0
export GPU_MEMORY_GB=24
biomeos deploy --niche compute-node
```

**Pattern:** Repeat for each GPU:
- Eastgate (3090)
- Strandgate (3070)
- BlueGate (4070)
- FlockGate (3070Ti)
- KinGate (3070)
- Swiftgate (3070)

#### **CPU Compute Nodes (4 total)**

**Strandgate** (Dual EPYC - Bio Pipeline)
```bash
# Pooled CPU node (64 cores)
export NODE_ID=compute-strandgate-epyc-pool
export FAMILY_ID=nat0
export RESOURCE_TYPE=cpu
export CPU_CORES=64
export MEMORY_LIMIT_GB=256
export POOLED_ENABLED=true
export POOL_COORDINATOR=true
biomeos deploy --niche compute-node
```

**Westgate** (NAS + Cache)
```bash
# Storage-backed CPU node
export NODE_ID=compute-westgate-nas
export FAMILY_ID=nat0
export RESOURCE_TYPE=cpu
export CPU_CORES=8
export STORAGE_GB=76000  # 76TB ZFS
biomeos deploy --niche compute-node
```

### **Phase 3: Internet Federation (2 towers)**

#### **FlockGate (Brother's Node)**
```bash
# Tower deployment
export NODE_ID=tower-flockgate
export FAMILY_ID=nat0
export INTERNET_ENABLED=true
biomeos deploy --niche tower --usb /media/liveSpore2

# GPU compute node
export NODE_ID=compute-flockgate-gpu3070ti
export FAMILY_ID=nat0
export RESOURCE_TYPE=gpu
biomeos deploy --niche compute-node
```

#### **KinGate (Family Node)**
```bash
# Tower deployment
export NODE_ID=tower-kingate
export FAMILY_ID=nat0
export INTERNET_ENABLED=true
biomeos deploy --niche tower --usb /media/liveSpore3

# GPU compute node
export NODE_ID=compute-kingate-gpu3070
export FAMILY_ID=nat0
export RESOURCE_TYPE=gpu
biomeos deploy --niche compute-node
```

**Result:**
- Internet nodes federate via BTSP
- NAT traversal via genetic lineage
- Secure P2P tunnels
- Same trust domain (nat0)

### **Phase 4: Mobile Nodes (2 mobile)**

#### **Swiftgate (Mobile Bridge)**
```bash
# Portable tower (laptop)
export NODE_ID=tower-swiftgate
export FAMILY_ID=nat0
export MOBILE_MODE=true
biomeos deploy --niche tower --usb /media/liveSpore4

# Mobile GPU compute
export NODE_ID=compute-swiftgate-gpu3070
export FAMILY_ID=nat0
export RESOURCE_TYPE=gpu
export MOBILE_MODE=true
biomeos deploy --niche compute-node
```

**Purpose:**
- Bridge LAN ↔ Internet
- Portable compute for demos
- Backup federation node

#### **Pixel 8a (Mobile HSM)**
```bash
# Lightweight tower (Android)
export NODE_ID=tower-pixel8a
export FAMILY_ID=nat0
export BEARDOG_HSM_MODE=hardware  # Use Android Keystore
export MOBILE_MODE=true
biomeos deploy --niche tower-lite  # Minimal tower
```

**Purpose:**
- BearDog HSM seed storage
- Mobile mesh participation
- 2FA for deployments

---

## 📊 Deployment Summary

### **Total Infrastructure**

| Component | Count | Details |
|-----------|-------|---------|
| **Towers** | 10 | 6 LAN + 2 Internet + 2 Mobile |
| **Compute Nodes** | 15 | 9 GPU + 4 CPU + 2 Mobile |
| **Genetic Family** | 1 | `nat0` (basement HPC) |
| **Sub-Federations** | TBD | gaming, bio-pipeline, ml-training |

### **Capabilities**

**Compute Power:**
- **9 GPUs** (RTX 5090, 3090s, 4070, 3070s)
- **~160 CPU cores** (64-core EPYC + others)
- **864GB RAM** total
- **~80TB storage** (ZFS + NVMe)

**Communication:**
- **LAN**: UDP multicast + BTSP
- **Internet**: BTSP P2P with NAT traversal
- **Mobile**: Portable bridges

**Security:**
- **Genetic lineage** trust (BearDog)
- **Crypto lock** on sensitive nodes
- **HSM roots** (4x SoloKeys + Pixel 8a)

---

## 🚀 Deployment Commands

### **Step 1: Prepare Spores (5 USB spores)**
```bash
# Create genetic family seed
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
cargo run --bin spore-tools -- create-family --family-id nat0 --output .family.seed

# Create 5 LiveSpores (genetic siblings)
for i in {1..5}; do
  export SPORE_ID=node-$(echo -n "alpha beta gamma delta epsilon" | cut -d' ' -f$i)
  cargo run --bin biomeos-spore -- build \
    --family-seed .family.seed \
    --niche tower \
    --output /media/liveSpore$i \
    --fresh-bins
done
```

### **Step 2: Deploy LAN Towers (6 nodes)**
```bash
# On each LAN node
# Northgate, Southgate, Eastgate, Westgate, Strandgate, BlueGate
export NODE_ID=tower-$(hostname)
export FAMILY_ID=nat0
biomeos deploy --niche tower --usb /media/liveSpore1
```

### **Step 3: Deploy Compute Nodes (15 nodes)**
```bash
# On each machine with GPU
export NODE_ID=compute-$(hostname)-gpu$(nvidia-smi --query-gpu=index --format=csv,noheader | head -1)
export RESOURCE_TYPE=gpu
export RESOURCE_ID=$(nvidia-smi --query-gpu=index --format=csv,noheader | head -1)
biomeos deploy --niche compute-node

# On CPU-only nodes
export NODE_ID=compute-$(hostname)-cpu
export RESOURCE_TYPE=cpu
export CPU_CORES=$(nproc)
biomeos deploy --niche compute-node
```

### **Step 4: Verify Federation**
```bash
# From any tower
curl --unix-socket /tmp/songbird-tower-$(hostname).sock \
  -d '{"jsonrpc":"2.0","method":"discover_by_family","params":{"family_tags":["nat0"],"timeout_ms":5000},"id":1}' \
  | jq '.result.nodes | length'

# Expected: 10 towers discovered
```

### **Step 5: Submit Test Workload**
```bash
# From any compute node
curl --unix-socket /tmp/compute-node-$(hostname).sock \
  -d '{"jsonrpc":"2.0","method":"workload.submit","params":{"runtime":"native","code":"print(\"Hello from basement HPC!\")"},"id":1}'
```

---

## 🎯 Use Cases

### **1. Distributed AI/LLM Training**
- **Nodes**: Northgate (5090), Southgate (3090), Eastgate (3090)
- **Workload**: Spread model across 3 GPUs
- **Coordination**: Towers manage data flow

### **2. Bio Pipeline (Alignment, Kraken2)**
- **Node**: Strandgate (Dual EPYC, 256GB ECC)
- **Storage**: Westgate (76TB ZFS)
- **Workload**: Genomics preprocessing

### **3. Gaming Federation**
- **Nodes**: Southgate, BlueGate, FlockGate, KinGate
- **Sub-federation**: `gaming` (granular trust)
- **Workload**: Multiplayer game servers

### **4. Mobile Demos**
- **Node**: Swiftgate (portable)
- **Purpose**: Show off mesh at conferences
- **Federation**: Connects to basement via BTSP

---

## 📝 Next Steps

### **Immediate (This Session)**
1. ✅ Design complete
2. ⏳ Create deployment manifests
3. ⏳ Test single tower deployment
4. ⏳ Test tower + compute node

### **Short-term (Next Session)**
1. Deploy all 6 LAN towers
2. Deploy 9 GPU compute nodes
3. Verify federation
4. Submit distributed workload

### **Long-term**
1. Internet nodes (FlockGate, KinGate)
2. Mobile nodes (Swiftgate, Pixel 8a)
3. Sub-federations (gaming, bio, ml)
4. Heterogeneous workloads

---

## 🎊 Status

**Design:** ✅ **COMPLETE**  
**Hardware:** ✅ **AVAILABLE** (~$15k basement HPC)  
**Software:** ✅ **READY** (Tower + Compute Node niches)  
**Deployment:** ⏳ **PENDING** (awaiting commands)

**Next:** Create deployment manifests and test!

---

**Architecture Summary:**
- **10 Towers** (6 LAN, 2 Internet, 2 Mobile)
- **15 Compute Nodes** (9 GPU, 4 CPU, 2 Mobile)
- **1 Genetic Family** (`nat0`)
- **Horizontal + Vertical** architecture unified!

🎊 **From design → production deployment!** 🏗️

