# 🖥️ Compute Node Niche - Horizontal Architecture Design

**Date:** January 8, 2026  
**Status:** 🎯 **DESIGN PHASE**  
**Related Niches:** Tower (vertical comms), Compute Node (horizontal compute)

---

## 🎯 Vision: Horizontal vs Vertical Architecture

### **Tower Niche** (Vertical - Communication Stack)
```
    Songbird (Discovery & P2P)
         ↕
    BearDog (Security & Crypto)
         ↕
    biomeOS (Orchestration)
         
Purpose: Inter-node communication, federation, discovery
```

### **Compute Node Niche** (Horizontal - Compute Fabric)
```
┌─────────────────────────────────────────────┐
│          Compute Node Pool                   │
│                                              │
│  ┌──────┐  ┌──────┐  ┌──────┐  ┌──────┐   │
│  │ GPU  │  │ CPU  │  │ WASM │  │Edge  │   │
│  │Node  │  │Node  │  │Node  │  │Node  │   │
│  └──────┘  └──────┘  └──────┘  └──────┘   │
│      ↕          ↕          ↕         ↕      │
│  ┌──────────────────────────────────────┐  │
│  │     Toadstool (Workload Manager)     │  │
│  └──────────────────────────────────────┘  │
│                    ↕                        │
│         BearDog (Crypto Lock)               │
│                    ↕                        │
│         Songbird (If needed)                │
└─────────────────────────────────────────────┘

Purpose: Distributed compute, workload execution, resource pooling
```

---

## 📐 Architecture Principles

### **1. Horizontal Scaling**
- Multiple nodes on same computer (different GPUs, CPUs)
- Nested nodes (GPU → sub-GPUs → cores)
- Pooled nodes (multiple CPUs act as one)
- Spread nodes (one logical node across physical resources)

### **2. Distinct from Towers**
- **Towers**: Communicate between machines (federation)
- **Nodes**: Execute workloads (computation)
- **Relationship**: Nodes CAN spawn Towers if needed

### **3. Minimal Communication**
- Nodes primarily execute, not communicate
- If complex communication needed → spawn Songbird or Tower
- Keep nodes lightweight and focused

### **4. Resource-Centric**
- Node identity tied to resources (GPU ID, CPU cores, memory)
- Dynamic pooling and splitting
- Runtime reconfiguration

---

## 🏗️ Compute Node Components

### **Core Stack**
```
┌─────────────────────────────────────────────┐
│            Compute Node                      │
├─────────────────────────────────────────────┤
│  Toadstool                                   │
│  • Workload Manager                          │
│  • Multi-runtime (Native, WASM, Container)   │
│  • Resource Tracking                         │
│  • GPU Compute                               │
├─────────────────────────────────────────────┤
│  BearDog (Optional)                          │
│  • Crypto Lock for sensitive workloads       │
│  • Encrypted memory                          │
│  • Secure enclaves                           │
├─────────────────────────────────────────────┤
│  Songbird (Conditional)                      │
│  • Only if node needs complex comms          │
│  • Spawned on-demand                         │
│  • Not in default compute node               │
└─────────────────────────────────────────────┘
```

---

## 🎨 Node Topology Patterns

### **1. Multi-Node (Same Computer)**
```
Computer: workstation-alpha
├── compute-node-gpu0 (NVIDIA RTX 4090)
├── compute-node-gpu1 (AMD Radeon)
├── compute-node-cpu0 (8 cores)
└── compute-node-cpu1 (8 cores)

Each runs independent Toadstool instance
BearDog shared via Unix socket
```

### **2. Nested Nodes (Hierarchical)**
```
compute-node-gpu0 (Parent)
├── sub-node-stream0 (SM 0-15)
├── sub-node-stream1 (SM 16-31)
└── sub-node-stream2 (SM 32-47)

Parent Toadstool manages sub-nodes
Sub-nodes report to parent
```

### **3. Pooled Nodes (Aggregated)**
```
compute-pool-cpu (Logical Node)
├── physical-node-cpu0 (8 cores)
├── physical-node-cpu1 (8 cores)
└── physical-node-cpu2 (8 cores)

One Toadstool, multiple executors
Acts as single 24-core node
```

### **4. Spread Node (Distributed)**
```
compute-node-ml-training (Logical)
├── gpu-node-alpha (RTX 4090 - layers 0-10)
├── gpu-node-beta (RTX 4090 - layers 11-20)
└── cpu-node-gamma (64 cores - data preprocessing)

One Toadstool coordinator
Multiple physical resources
May spawn Tower for inter-machine comms
```

---

## 📦 BYOB Manifest: `compute-node.toml`

```toml
[niche]
name = "compute-node"
version = "1.0.0"
type = "compute"
description = "Horizontal compute architecture for workload execution"

# =============================================================================
# PRIMALS CONFIGURATION
# =============================================================================

[[primals]]
binary = "./primals/toadstool"
provides = [
    "universal-compute",
    "workload-scheduling",
    "multi-runtime-execution",
    "gpu-compute",
    "resource-management"
]
requires = []  # Minimal, self-sufficient

[primals.env]
TOADSTOOL_NODE_ID = "${NODE_ID}"  # From deployment
TOADSTOOL_RESOURCE_TYPE = "${RESOURCE_TYPE}"  # gpu|cpu|wasm|container
TOADSTOOL_RESOURCE_ID = "${RESOURCE_ID}"  # Device ID or pool ID
TOADSTOOL_MODE = "compute"  # vs "daemon" mode
RUST_LOG = "info"

# Optional: If secure workloads needed
[[primals]]
binary = "./primals/beardog-server"
provides = ["security", "encryption"]
requires = []
optional = true  # Only if TOADSTOOL_SECURE_MODE=true

[primals.env]
BEARDOG_NODE_ID = "${NODE_ID}"
BEARDOG_FAMILY_SEED_FILE = "./.family.seed"
BEARDOG_MODE = "crypto-lock"  # Minimal mode for compute

# =============================================================================
# NODE CONFIGURATION
# =============================================================================

[node]
# Node topology
topology = "standalone"  # standalone|nested|pooled|spread

# Resource binding
resource_type = "gpu"  # gpu|cpu|wasm|container|hybrid
resource_id = "0"  # Device ID, core range, or pool ID

# Capacity
max_concurrent_workloads = 4
memory_limit_gb = 16
cpu_cores = 8
gpu_memory_gb = 24  # If GPU node

# Isolation
isolation_mode = "process"  # process|container|vm|none
sandbox_enabled = true

# =============================================================================
# WORKLOAD EXECUTION
# =============================================================================

[execution]
# Runtimes to enable
runtimes = ["native", "wasm", "container"]

# Execution policy
preemption_enabled = false
priority_scheduling = true
fair_scheduling = true

# Resource limits per workload
[execution.limits]
max_cpu_percent = 80
max_memory_percent = 90
max_execution_time_seconds = 3600
max_gpu_memory_percent = 95

# =============================================================================
# COMMUNICATION (Minimal)
# =============================================================================

[communication]
# Node communication strategy
strategy = "minimal"  # minimal|local|mesh

# When to spawn Songbird
spawn_songbird_if = [
    "multi_machine_coordination",
    "p2p_data_transfer",
    "discovery_needed"
]

# Unix socket for local IPC
unix_socket = "/tmp/compute-node-${NODE_ID}.sock"

# Only bind HTTP if monitoring needed
http_enabled = false
http_port = 9090  # Prometheus metrics if enabled

# =============================================================================
# MONITORING
# =============================================================================

[monitoring]
# Resource tracking
track_cpu = true
track_memory = true
track_gpu = true
track_disk_io = true
track_network_io = false  # Compute nodes don't network much

# Metrics
prometheus_enabled = true
prometheus_port = 9090

# Logging
log_level = "info"
log_file = "/var/log/compute-node-${NODE_ID}.log"

# =============================================================================
# SECURITY (if BearDog enabled)
# =============================================================================

[security]
# Crypto Lock for sensitive workloads
crypto_lock_enabled = true
encrypted_memory = true
secure_enclave = false  # Requires hardware support

# Workload verification
verify_workload_signatures = true
trusted_deployers = ["${FAMILY_ID}"]  # From genetic lineage

# =============================================================================
# FEDERATION (Tower Spawning)
# =============================================================================

[federation]
# When to spawn a Tower for communication
spawn_tower_if = [
    "remote_coordination_needed",
    "p2p_federation_required",
    "discovery_beyond_local"
]

# Genetic lineage for trust
family_seed_file = "./.family.seed"
family_id = "${FAMILY_ID}"

# Tower configuration (if spawned)
[federation.tower]
songbird_enabled = true
beardog_enabled = true
bmeos_enabled = false  # Lightweight tower

# =============================================================================
# ADVANCED: NESTED NODES
# =============================================================================

[nested]
# Enable nested sub-nodes
enabled = false

# Sub-node configuration
[nested.sub_nodes]
count = 4  # Number of sub-nodes to spawn
resource_split = "equal"  # equal|weighted|dynamic

# Sub-node naming
naming_pattern = "${NODE_ID}-sub{index}"

# =============================================================================
# ADVANCED: POOLED NODES
# =============================================================================

[pooled]
# Enable pooling with other nodes
enabled = false

# Pool configuration
pool_id = "cpu-pool-1"
coordinator = true  # Is this the pool coordinator?

# Pool members (if coordinator)
[pooled.members]
nodes = ["compute-node-cpu0", "compute-node-cpu1", "compute-node-cpu2"]

# =============================================================================
# ADVANCED: SPREAD NODES
# =============================================================================

[spread]
# Enable spreading across machines
enabled = false

# Spread configuration
logical_node_id = "ml-training-node"
physical_node_id = "gpu-alpha"  # This machine's contribution

# Coordinator node
coordinator_endpoint = "udp://192.168.1.100:4433"  # Tower BTSP

# Resource contribution
[spread.contribution]
resource_type = "gpu"
resource_amount = "1x RTX 4090"
layers = "0-10"  # For ML models
```

---

## 🔄 Deployment Scenarios

### **Scenario 1: Gaming PC (Multi-GPU)**
```yaml
# Deploy 3 compute nodes on one machine
Nodes:
  - compute-node-rtx4090  (primary GPU)
  - compute-node-amd6900  (secondary GPU)
  - compute-node-cpu      (fallback CPU)

Tower: gaming-tower-alpha
Purpose: AI-assisted gaming, real-time rendering
Comm: Local IPC only, no Songbird needed
```

### **Scenario 2: ML Training (Nested)**
```yaml
# One parent node, 4 sub-nodes per GPU
Parent: ml-train-parent
Sub-nodes:
  - ml-train-gpu0-stream0
  - ml-train-gpu0-stream1
  - ml-train-gpu0-stream2
  - ml-train-gpu0-stream3

Tower: ml-tower-beta
Purpose: Parallel model training
Comm: Parent-to-Tower, subs report to parent
```

### **Scenario 3: School Lab (Pooled)**
```yaml
# 20 computers pool CPU resources
Pool: school-cpu-pool
Members: 20x compute-node-cpu (8 cores each)
Total: 160 cores available

Tower: school-tower-gamma
Purpose: Distributed student workloads
Comm: Tower coordinates, nodes execute
```

### **Scenario 4: Distributed Training (Spread)**
```yaml
# ML training across friend's computers
Logical Node: distributed-ml-training
Physical:
  - friend-alpha (RTX 4090, layers 0-10)
  - friend-beta (RTX 4090, layers 11-20)
  - friend-gamma (64-core CPU, preprocessing)

Towers: Each friend has gaming-tower
Comm: Towers federate via genetic lineage
     Spread node coordinates via BTSP
```

---

## 🎯 Key Design Decisions

### **1. Minimal Communication**
**Why**: Compute nodes execute, not communicate  
**How**: Default no Songbird, spawn only if needed  
**Benefit**: Lightweight, focused on computation

### **2. BearDog Optional**
**Why**: Not all workloads need crypto  
**How**: Enable via `TOADSTOOL_SECURE_MODE=true`  
**Benefit**: Resource-efficient

### **3. Tower Spawning**
**Why**: Complex comms need full stack  
**How**: Node can spawn Tower on-demand  
**Benefit**: Flexibility without bloat

### **4. Resource-Centric Identity**
**Why**: Nodes are resources, not services  
**How**: `compute-node-gpu0` not `tower-alpha`  
**Benefit**: Clear purpose and scaling

---

## 🚀 Implementation Phases

### **Phase 1: Basic Compute Node** (This Session)
- ✅ Design complete
- ⏳ BYOB manifest for compute-node
- ⏳ Toadstool standalone mode
- ⏳ Unix socket IPC
- ⏳ Basic workload execution

### **Phase 2: BearDog Integration**
- Crypto Lock for sensitive workloads
- Encrypted memory via BearDog
- Workload signature verification

### **Phase 3: Multi-Node**
- Deploy multiple nodes on one machine
- Resource isolation
- Fair scheduling

### **Phase 4: Nested Nodes**
- Parent-child hierarchy
- Sub-node management
- Resource splitting

### **Phase 5: Pooled & Spread**
- Pooled nodes (aggregation)
- Spread nodes (distribution)
- Tower spawning for coordination

---

## 📊 Comparison: Tower vs Node

| Aspect | Tower (Vertical) | Node (Horizontal) |
|--------|------------------|-------------------|
| **Purpose** | Communication | Computation |
| **Components** | Songbird + BearDog + biomeOS | Toadstool (+ optional BearDog) |
| **Networking** | Heavy (P2P, federation) | Minimal (local IPC) |
| **Scaling** | 1 per machine | Multiple per machine |
| **Identity** | By location (tower-alpha) | By resource (gpu0, cpu-pool) |
| **Longevity** | Long-running service | Task-oriented |
| **Communication** | Always-on mesh | On-demand spawning |

---

## 🎊 Next Steps

### **Immediate**
1. Create `compute-node.toml` manifest
2. Test Toadstool in standalone compute mode
3. Deploy single node locally
4. Verify workload execution

### **Short-term**
1. Add BearDog Crypto Lock
2. Test multi-node on same machine
3. Implement nested node support
4. Test pooled CPU nodes

### **Long-term**
1. Spread nodes across machines
2. Tower spawning on-demand
3. Gaming federation with compute nodes
4. School lab deployment

---

**Status**: 🎯 **Design Complete**  
**Next**: Implement Phase 1 - Basic Compute Node  
**Goal**: Lightweight, horizontal compute architecture distinct from Towers

🍄 **ToadStool**: From universal compute → focused execution nodes!  
🌱 **biomeOS**: From towers → compute fabric!  
🎊 **Together**: Vertical comms + Horizontal compute = Complete ecosystem!

