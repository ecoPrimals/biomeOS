# 🧬 biomeOS Architecture - Atomics & Composition

**Date**: January 11, 2026  
**Critical**: Architectural Foundation Document  

---

## 🎯 **Core Principle: Atomics & Composition**

```
ecoPrimals Atomics → biomeOS Atomics → NUCLEUS
     (Primals)    →    (Niches)     → (Complete System)
```

---

## 🔬 **ecoPrimals Atomics (The Primals)**

Individual, sovereign services that provide ONE core capability:

| Primal | Capability | Function |
|--------|------------|----------|
| **Songbird** | `discovery` | Service registry + P2P tunneling |
| **BearDog** | `security` | Encryption + genetic lineage + HSM |
| **ToadStool** | `compute` | Universal compute (native/WASM/GPU/container) |
| **NestGate** | `storage` | Content-addressed storage + provenance |
| **Squirrel** | `ai` | AI/ML orchestration + optimization |
| **petalTongue** | `visualization` | Universal UI rendering |

**Characteristic**: Each primal is a **single-purpose service**.

---

## 🧬 **biomeOS Atomics (The Niches)**

Composed, encrypted systems that provide ONE **secure** capability:

### **🗼 Tower (Secure Communications)**

**Composition**:
```
Tower = BearDog + Songbird
```

**Function**:
- Secure P2P tunneling (BTSP over BearDog encryption)
- Service discovery within genetic families
- Encrypted communication between Towers
- Gateway for all ecosystem communications

**Interactions**:
- Tower ↔ Tower: Secure relay, multi-hop federation
- Tower ↔ Node: Secure command/control
- Tower ↔ Nest: Secure data transfer

**Socket Endpoints**:
- `beardog-{family}.sock` - Encryption/HSM
- `songbird-{family}.sock` - Discovery/tunneling

---

### **💻 Node (Secure Distributed Compute)**

**Composition**:
```
Node = BearDog + Songbird + ToadStool
```

**Function**:
- Encrypted workload execution
- Distributed compute across multiple Nodes
- Secure task scheduling
- Resource management (CPU/GPU/WASM)

**Interactions**:
- Node ↔ Node: Distributed workload splitting, parallel execution
- Node ↔ Tower: Secure command relay
- Node ↔ Nest: Fetch encrypted data for computation

**Socket Endpoints**:
- `beardog-{family}.sock` - Encryption/HSM
- `songbird-{family}.sock` - Discovery/coordination
- `toadstool-{family}.sock` - Compute execution

---

### **📦 Nest (Secure Federated Data)**

**Composition**:
```
Nest = BearDog + Songbird + NestGate
```

**Function**:
- Encrypted storage (data at rest)
- Federated data replication across Nests
- Content-addressed retrieval
- Provenance tracking

**Interactions**:
- Nest ↔ Nest: Data replication, sharding, redundancy
- Nest ↔ Tower: Secure data routing
- Nest ↔ Node: Provide data for computation

**Socket Endpoints**:
- `beardog-{family}.sock` - Encryption/HSM
- `songbird-{family}.sock` - Discovery/federation
- `nestgate-{family}.sock` - Storage/retrieval

---

## 🌐 **NUCLEUS (Complete biomeOS System)**

**Composition**:
```
NUCLEUS = Tower + Node + Nest
        = (BearDog + Songbird) +
          (BearDog + Songbird + ToadStool) +
          (BearDog + Songbird + NestGate)
```

**Simplified**:
```
NUCLEUS = BearDog + Songbird + ToadStool + NestGate
          (shared security & discovery for all 3 niches)
```

**Function**:
- Complete self-contained biomeOS on a single gate (liveSpore/computer)
- Secure communications (Tower)
- Secure compute (Node)
- Secure storage (Nest)

**Deployment**:
- Can run on ONE computer (single gate)
- Can distribute across multiple gates (federation)
- Each gate can run Tower-only, Node-only, Nest-only, or all three

---

## 🔗 **Interactions Between Atomics**

### **Tower ↔ Tower (Secure Mesh)**
```
Tower_A.songbird → Tower_B.songbird (encrypted tunnel via BearDog)
                    ↓
              Relay messages
              Multi-hop routing
              Federation coordination
```

### **Node ↔ Node (Distributed Compute)**
```
Node_A.toadstool → "Execute task X" → Node_B.toadstool
                         ↓
                   Via Tower (secure)
                         ↓
                   Results returned encrypted
```

**Use Cases**:
- GPU workload on Node_A, CPU on Node_B
- Parallel processing across 10 Nodes
- Failover if Node_A goes offline

### **Nest ↔ Nest (Federated Data)**
```
Nest_A.nestgate → Store("data", replicas=3) → Nest_B.nestgate
                                              → Nest_C.nestgate
                         ↓
                   Via Tower (secure)
                         ↓
                   Content-addressed, replicated, encrypted
```

**Use Cases**:
- Data redundancy across 3 Nests
- Geographic distribution
- Load balancing for reads

### **Node ↔ Nest (Compute on Data)**
```
Node.toadstool → "Fetch dataset X" → Nest.nestgate
                      ↓
                Data returned encrypted
                      ↓
                Node.toadstool → Compute → Results
                      ↓
                "Store results" → Nest.nestgate
```

---

## 📐 **Architectural Hierarchy**

```
Level 1: ecoPrimals Atomics (Individual Services)
  ├─ Songbird (discovery)
  ├─ BearDog (security)
  ├─ ToadStool (compute)
  ├─ NestGate (storage)
  ├─ Squirrel (ai)
  └─ petalTongue (ui)

Level 2: biomeOS Atomics (Secure Niches)
  ├─ Tower = BearDog + Songbird
  ├─ Node  = BearDog + Songbird + ToadStool
  └─ Nest  = BearDog + Songbird + NestGate

Level 3: Complete Systems (Gates)
  └─ NUCLEUS = Tower + Node + Nest
             = BearDog + Songbird + ToadStool + NestGate

Level 4: Federations (Multiple Gates)
  ├─ Ecosystem = Multiple NUCLEUS instances
  ├─ Tower Mesh = Towers relay between gates
  ├─ Compute Grid = Nodes distribute workloads
  └─ Data Fabric = Nests federate storage
```

---

## 🎨 **Key Insight: Encryption-Based Everything**

**ALL biomeOS atomics (Tower, Node, Nest) are encryption-based systems.**

This means:
- Every niche includes BearDog (encryption + HSM)
- Every niche includes Songbird (secure discovery + tunneling)
- Data is ALWAYS encrypted in transit (via Tower/Songbird/BearDog)
- Data is ALWAYS encrypted at rest (via Nest/BearDog)
- Compute is ALWAYS on encrypted data (via Node/BearDog)

**Zero-trust by default. Encryption is not optional—it's foundational.**

---

## 🧩 **Example Deployments**

### **1. Single Gate (NUCLEUS)**
```
Gate_1: Tower + Node + Nest
  All services on one computer
  Self-contained system
  Perfect for: Development, single-user, edge device
```

### **2. Specialized Gates (Federation)**
```
Gate_1: Tower (communications hub)
Gate_2: Node (GPU compute)
Gate_3: Node (CPU compute)
Gate_4: Nest (storage)
Gate_5: Nest (storage replica)

Tower on Gate_1 coordinates all:
  - Gate_2 & Gate_3 distribute compute
  - Gate_4 & Gate_5 replicate data
```

### **3. Hybrid**
```
Gate_1: NUCLEUS (user's laptop)
Gate_2: Node (cloud GPU)
Gate_3: Nest (home NAS)

Laptop runs complete system
Cloud provides extra compute
NAS provides backup storage
```

---

## 🔐 **Security Model**

### **Trust Boundary**
```
ecoPrimals (Level 1):
  ├─ Primals trust each other within a family
  └─ BearDog verifies genetic lineage

biomeOS (Level 2):
  ├─ Niches compose primals with enforced encryption
  └─ Tower acts as the trust anchor

NUCLEUS (Level 3):
  ├─ Complete system with end-to-end encryption
  └─ Can federate with other NUCLEUS instances
```

### **Encryption Layers**
1. **At Rest**: NestGate + BearDog encrypt stored data
2. **In Transit**: Songbird + BearDog encrypt network traffic (BTSP)
3. **In Compute**: ToadStool executes on encrypted data (BearDog provides keys)

---

## 🚀 **Deployment Strategy**

### **Phase 1: Deploy Atomics**
```bash
# Deploy Tower (secure comms)
biomeos deploy-niche graphs/tower_deploy.toml

# Deploy Node (secure compute)
biomeos deploy-niche graphs/node_deploy.toml

# Deploy Nest (secure storage)
biomeos deploy-niche graphs/nest_deploy.toml
```

### **Phase 2: Test Interactions**
```bash
# Test Tower ↔ Tower
biomeos test-interaction tower-tower

# Test Node ↔ Node
biomeos test-interaction node-node

# Test Nest ↔ Nest
biomeos test-interaction nest-nest

# Test Node ↔ Nest
biomeos test-interaction node-nest
```

### **Phase 3: Deploy NUCLEUS**
```bash
# Deploy complete system
biomeos deploy-niche graphs/nucleus_deploy.toml
```

### **Phase 4: Federation**
```bash
# Connect to another NUCLEUS
biomeos federate <peer-nucleus-id>
```

---

## 💡 **Why This Architecture?**

### **Composability**
- Tower, Node, Nest are LEGO blocks
- Mix and match for different use cases
- Scale horizontally (add more Nodes, Nests, Towers)

### **Security by Default**
- Every atomic includes encryption (BearDog)
- Every atomic includes secure discovery (Songbird)
- Zero plaintext data, ever

### **Flexibility**
- Run Tower-only for a relay node
- Run Node-only for a compute worker
- Run Nest-only for a storage node
- Run NUCLEUS for a complete system

### **Federation**
- Towers relay between gates
- Nodes distribute workloads
- Nests replicate data
- Everything encrypted end-to-end

---

## 🎯 **Current Status**

✅ **Primals (Level 1)**: All implemented and running  
⏳ **Niches (Level 2)**: Graphs defined, deployment pending  
⏳ **NUCLEUS (Level 3)**: Graph defined, deployment pending  
⏳ **Federation (Level 4)**: Specification complete, implementation pending  

---

## 📊 **Next Steps**

1. ✅ Fix primal socket paths (XDG-compliant)
2. ⏳ Deploy Tower (BearDog + Songbird)
3. ⏳ Deploy Node (BearDog + Songbird + ToadStool)
4. ⏳ Deploy Nest (BearDog + Songbird + NestGate)
5. ⏳ Test niche interactions
6. ⏳ Deploy NUCLEUS (all 3 atomics)
7. ⏳ Test federation (multiple NUCLEUS instances)

---

**Different orders of the same architecture.** 🍄🐸

Tower, Node, and Nest are the primals of biomeOS!


