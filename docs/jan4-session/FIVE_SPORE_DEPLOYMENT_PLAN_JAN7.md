# 🌱 Five Spore Deployment Plan - Architectural Naming

**Date**: January 7, 2026  
**Goal**: Test complete biomeOS ecosystem with proper naming  
**Devices**: 5 USB drives

---

## 🏗️ Architecture Clarification

### **Tower** (Vertical - Communication Stack)
- Songbird (discovery, federation)
- BearDog (security, crypto)
- Communication infrastructure
- Can be **shared** by multiple nodes on same PC

### **Node** (Horizontal - Compute Deployment)
- Compute workload (Toadstool when integrated)
- Isomorphic deployments
- Multiple nodes per PC possible
- Each node can use a shared tower for efficiency

**Example**: Gaming competition PC
- **1 Tower** (shared communication)
- **Multiple Nodes** (compute instances)
- All nodes federate via the shared tower

---

## 📋 Deployment Strategy

### **LiveSpores** (3 for active deployment)

| USB | Mount | Node Name | Tower | Purpose | Type |
|-----|-------|-----------|-------|---------|------|
| sdc1 | `/media/eastgate/biomeOS1` | `node-alpha` | `tower-alpha` | Local testing | LiveSpore |
| sde1 | `/media/eastgate/biomeOS21` | `node-beta` | `tower-beta` | Local testing | LiveSpore |
| sdd1 | `/media/eastgate/BEA6-BBCE` | `node-gamma` | `tower-gamma` | **LAN deployment** | LiveSpore |

### **ColdSpores** (2 for archival)

| USB | Mount | Node Name | Tower | Purpose | Type |
|-----|-------|-----------|-------|---------|------|
| sdf1 | `/media/eastgate/BEA6-BBCE1` | `node-delta` | N/A | Archive/Backup | ColdSpore |
| sdg1 | `/media/eastgate/BEA6-BBCE2` | `node-epsilon` | N/A | Archive/Backup | ColdSpore |

---

## 🎯 Naming Convention

### **Node Naming**
- Format: `node-{name}` (e.g., `node-alpha`, `node-beta`)
- Represents compute deployment
- Horizontal scalability
- Greek alphabet for uniqueness: alpha, beta, gamma, delta, epsilon

### **Tower Naming**
- Format: `tower-{name}` (e.g., `tower-alpha`, `tower-beta`)
- Represents communication stack
- Vertical service (Songbird, BearDog)
- Matches node name for 1:1 deployments
- Can be shared across nodes (advanced use case)

### **Spore Label**
- LiveSpore: `biomeOS-{node}` (e.g., `biomeOS-alpha`)
- ColdSpore: `biomeOS-{node}-cold` (e.g., `biomeOS-delta-cold`)

---

## 🔄 Current State → Clean State

### **Before** (Legacy Naming)
- `tower1`, `tower2`, `tower3` (conflates tower + node)
- Unclear separation of concerns

### **After** (Architectural Naming)
- **Nodes**: `node-alpha`, `node-beta`, `node-gamma`, etc.
- **Towers**: `tower-alpha`, `tower-beta`, `tower-gamma`, etc.
- Clear separation: tower = communication, node = compute

---

## 🚀 Deployment Commands

### **Clean Existing Spores**
```bash
# Remove old biomeOS directories
rm -rf /media/eastgate/biomeOS1/biomeOS
rm -rf /media/eastgate/biomeOS21/biomeOS  
rm -rf /media/eastgate/BEA6-BBCE/biomeOS
```

### **Create LiveSpores**

#### 1. node-alpha (Local Test 1)
```bash
cargo run --release --bin biomeos -- spore create \
    --mount /media/eastgate/biomeOS1 \
    --label biomeOS-alpha \
    --node node-alpha \
    --spore-type live
```

#### 2. node-beta (Local Test 2)
```bash
cargo run --release --bin biomeos -- spore create \
    --mount /media/eastgate/biomeOS21 \
    --label biomeOS-beta \
    --node node-beta \
    --spore-type live
```

#### 3. node-gamma (LAN Deployment)
```bash
cargo run --release --bin biomeos -- spore create \
    --mount /media/eastgate/BEA6-BBCE \
    --label biomeOS-gamma \
    --node node-gamma \
    --spore-type live
```

### **Create ColdSpores**

#### 4. node-delta (Archive 1)
```bash
cargo run --release --bin biomeos -- spore create \
    --mount /media/eastgate/BEA6-BBCE1 \
    --label biomeOS-delta-cold \
    --node node-delta \
    --spore-type cold
```

#### 5. node-epsilon (Archive 2)
```bash
cargo run --release --bin biomeos -- spore create \
    --mount /media/eastgate/BEA6-BBCE2 \
    --label biomeOS-epsilon-cold \
    --node node-epsilon \
    --spore-type cold
```

---

## 🧪 Testing Strategy

### **Phase 1: Local Dual-Node Federation** (alpha + beta)
1. Deploy `node-alpha` and `node-beta` locally
2. Verify genetic family trust (same `.family.seed` via cloning)
3. Test auto-acceptance via BearDog tags
4. Validate UDP discovery + BTSP tunnels

### **Phase 2: LAN Federation** (gamma)
1. Deploy `node-gamma` to remote PC on LAN
2. Verify cross-host federation
3. Test BTSP over network
4. Validate port-free architecture

### **Phase 3: ColdSpore Validation** (delta + epsilon)
1. Verify ColdSpores have NO `deploy.sh`
2. Verify archival README
3. Test integrity checks
4. Practice conversion: ColdSpore → LiveSpore

---

## 📊 Expected Topology

```
Local PC:
┌────────────────────────────────────┐
│  node-alpha (tower-alpha)          │
│  ├─ BearDog (unix socket)          │
│  ├─ Songbird (UDP discovery)       │
│  └─ Tower orchestrator             │
├────────────────────────────────────┤
│  node-beta (tower-beta)            │
│  ├─ BearDog (unix socket)          │
│  ├─ Songbird (UDP discovery)       │
│  └─ Tower orchestrator             │
└────────────────────────────────────┘
        ↕ BTSP Tunnel (port-free)
┌────────────────────────────────────┐
│ Remote PC (LAN):                   │
│  node-gamma (tower-gamma)          │
│  ├─ BearDog (unix socket)          │
│  ├─ Songbird (UDP discovery)       │
│  └─ Tower orchestrator             │
└────────────────────────────────────┘

Archive:
┌────────────────────────────────────┐
│ ColdSpore: node-delta              │
│ (genetic material only)            │
├────────────────────────────────────┤
│ ColdSpore: node-epsilon            │
│ (genetic material only)            │
└────────────────────────────────────┘
```

---

## 🎯 Success Criteria

✅ **5 spores created** (3 Live, 2 Cold)  
✅ **Naming consistency** (node-{name}, tower-{name})  
✅ **Local federation** (alpha ↔ beta)  
✅ **LAN federation** (alpha/beta ↔ gamma)  
✅ **ColdSpore validation** (archival mode)  
✅ **Genetic lineage** (all share family seed)  
✅ **Port-free architecture** (BTSP tunnels working)  

---

## 🌱 Future: Multi-Node, Shared Tower

**When Toadstool is integrated**:

```
Gaming Competition PC:
┌────────────────────────────────────┐
│ tower-competition (SHARED)         │
│  ├─ BearDog (unix socket)          │
│  └─ Songbird (UDP discovery)       │
├────────────────────────────────────┤
│ node-compete-1 (Toadstool)         │ ← Uses tower-competition
│ node-compete-2 (Toadstool)         │ ← Uses tower-competition
│ node-compete-3 (Toadstool)         │ ← Uses tower-competition
│ node-compete-4 (Toadstool)         │ ← Uses tower-competition
└────────────────────────────────────┘
```

**Isomorphic**: All nodes identical, work together seamlessly.  
**Efficient**: Single tower for all nodes (shared communication).

---

## 🎊 Let's Deploy!

**Command Sequence**:
1. Clean existing spores
2. Create 3 LiveSpores (alpha, beta, gamma)
3. Create 2 ColdSpores (delta, epsilon)
4. Verify all 5 spores
5. Deploy locally (alpha + beta)
6. Test federation
7. Deploy to LAN (gamma)
8. Celebrate! 🎊

