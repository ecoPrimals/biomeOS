# 🏗️ Architecture Layers - biomeOS Orchestration Model

**Date**: January 4, 2026  
**Purpose**: Clarify the two-level orchestration model in biomeOS

---

## 🎯 Core Principle

**biomeOS uses a TWO-LEVEL orchestration model**:

1. **Level 1: Infrastructure (biomeOS)** - Orchestrates primals
2. **Level 2: Application (ToadStool)** - Orchestrates workloads

---

## 📊 The Two-Level Model

```
┌─────────────────────────────────────────────────────────────┐
│                    User's Application                        │
│                      (biome.yaml)                            │
└──────────────────────────┬──────────────────────────────────┘
                           ↓
┌────────────────────────────────────────────────────────────┐
│              Level 2: Application Orchestration             │
│                       (ToadStool)                           │
│                                                             │
│  Responsibilities:                                          │
│  • Parse biome.yaml                                         │
│  • Execute containers/WASM/Python/Native/GPU                │
│  • Resource allocation                                      │
│  • Application lifecycle                                    │
│  • BYOB (Bring Your Own Binary)                             │
│                                                             │
│  Provides: Compute, Storage, Orchestration                  │
│  Requires: Discovery, Security                              │
└────────────────────────────────────────────────────────────┘
                           ↑
                           │ (capability queries)
                           ↓
┌────────────────────────────────────────────────────────────┐
│              biomeOS Capability Registry                    │
│                 (Unix Socket IPC)                           │
│                                                             │
│  /tmp/biomeos-registry-{family}.sock                        │
│                                                             │
│  API:                                                       │
│  • register(id, provides, requires)                         │
│  • get_provider(capability) → PrimalInfo                    │
│  • list_primals() → Vec<PrimalInfo>                         │
└────────────────────────────────────────────────────────────┘
                           ↑
                           │ (primal registration)
                           ↓
┌────────────────────────────────────────────────────────────┐
│              Level 1: Infrastructure Orchestration          │
│                       (biomeOS)                             │
│                                                             │
│  Responsibilities:                                          │
│  • Parse tower.toml                                         │
│  • Spawn primals (ToadStool, BearDog, Songbird)             │
│  • Monitor primal health                                    │
│  • Restart failed primals                                   │
│  • Capability routing                                       │
│  • Graceful shutdown                                        │
│                                                             │
│  Does NOT:                                                  │
│  • Execute user workloads                                   │
│  • Parse biome.yaml                                         │
│  • Manage application resources                             │
└────────────────────────────────────────────────────────────┘
                           ↑
                           │ (tower.toml)
                           ↓
┌────────────────────────────────────────────────────────────┐
│                  Infrastructure Config                       │
│                     (tower.toml)                            │
└────────────────────────────────────────────────────────────┘
```

---

## 📁 Configuration Files

### tower.toml (Infrastructure-Level)

**Who reads it**: biomeOS  
**Purpose**: Define primal infrastructure

```toml
# Infrastructure orchestration
family_id = "nat0"

[primals.toadstool]
binary = "primals/toadstool"
provides = ["Compute", "Storage", "Orchestration"]
requires = ["Discovery", "Security"]
env = { TOADSTOOL_PORT = "8080" }

[primals.beardog]
binary = "primals/beardog"
provides = ["Security", "Encryption", "Trust"]
requires = ["Discovery"]
env = { BEARDOG_API_BIND_ADDR = "0.0.0.0:9000" }

[primals.songbird]
binary = "primals/songbird"
provides = ["Discovery", "ConnectionManagement"]
env = { SONGBIRD_PORT = "8081" }
```

---

### biome.yaml (Application-Level)

**Who reads it**: ToadStool  
**Purpose**: Define user application workloads

```yaml
# Application orchestration
apiVersion: biomeOS/v1
kind: Biome

primals:
  web-app:
    runtime: container
    image: myapp:latest
    env:
      DATABASE_URL: postgres://db:5432/mydb
    
  ai-service:
    runtime: python
    script: inference.py
    gpu: true
    resources:
      gpus: 1
      memory: "8Gi"
  
  data-processor:
    runtime: wasm
    binary: processor.wasm
    env:
      INPUT_PATH: /data/input
      OUTPUT_PATH: /data/output
```

---

## 🔄 Deployment Flow

### Step 1: Infrastructure Deployment (biomeOS)

```bash
$ tower run --config tower.toml
```

**What happens**:
1. biomeOS reads `tower.toml`
2. Creates capability registry at `/tmp/biomeos-registry-{family}.sock`
3. Spawns primals in dependency order:
   - Songbird (provides: Discovery)
   - BearDog (provides: Security, requires: Discovery)
   - ToadStool (provides: Compute, requires: Discovery + Security)

---

### Step 2: Primal Registration

**Songbird**:
```json
{
  "method": "register",
  "id": "songbird@tower1",
  "params": {
    "provides": ["Discovery", "ConnectionManagement"],
    "requires": [],
    "socket_path": "/tmp/songbird-nat0.sock"
  }
}
```

**BearDog**:
```json
{
  "method": "register",
  "id": "beardog@tower1",
  "params": {
    "provides": ["Security", "Encryption", "Trust"],
    "requires": ["Discovery"],
    "http_endpoint": "http://localhost:9000"
  }
}
```

Then BearDog queries for Discovery provider:
```json
{
  "method": "get_provider",
  "capability": "Discovery"
}
```

Response:
```json
{
  "status": "success",
  "data": {
    "id": "songbird@tower1",
    "socket_path": "/tmp/songbird-nat0.sock"
  }
}
```

**ToadStool**:
```json
{
  "method": "register",
  "id": "toadstool@tower1",
  "params": {
    "provides": ["Compute", "Storage", "Orchestration"],
    "requires": ["Discovery", "Security"]
  }
}
```

---

### Step 3: Application Deployment (ToadStool)

```bash
$ toadstool run biome.yaml
```

**What happens**:
1. ToadStool reads `biome.yaml`
2. For each service in the manifest:
   - Queries biomeOS: "Who provides Security?" → BearDog
   - Queries biomeOS: "Who provides Discovery?" → Songbird
   - Allocates resources (CPU, memory, GPU)
   - Spawns container/WASM/Python process
   - Connects to BearDog for encryption
   - Connects to Songbird for service discovery
3. Monitors application health

---

## 🎯 Analogy

| biomeOS Model | Industry Analogy |
|---------------|------------------|
| **biomeOS** (Level 1) | **Kubernetes** - Orchestrates infrastructure pods |
| **ToadStool** (Level 2) | **Docker Compose** - Orchestrates application containers |
| **tower.toml** | **Kubernetes deployment YAML** |
| **biome.yaml** | **Docker Compose file** |
| **Capability Registry** | **Kubernetes Service Discovery** |

---

## 🏗️ Primal Roles

### biomeOS
- **Role**: Infrastructure Orchestrator
- **Config**: tower.toml
- **Orchestrates**: Primals (ToadStool, BearDog, Songbird)
- **Provides**: Capability registry, primal lifecycle, health monitoring
- **Does NOT**: Execute user workloads, parse biome.yaml

### ToadStool
- **Role**: Workload Orchestrator
- **Config**: biome.yaml
- **Orchestrates**: Application workloads (containers, WASM, Python)
- **Provides**: Compute, Storage, Orchestration capabilities
- **Requires**: Discovery (from Songbird), Security (from BearDog)

### Songbird
- **Role**: Discovery Orchestrator
- **Provides**: UDP multicast discovery, peer registry, Unix socket IPC
- **Protocol**: BirdSong (encrypted discovery)

### BearDog
- **Role**: Security Orchestrator
- **Provides**: Encryption, trust evaluation, key management
- **Protocol**: BTSP (BearDog Tunnel Secure Protocol)

---

## ❌ Common Misconceptions

### Misconception 1: "biomeOS executes workloads"

**Wrong**: biomeOS orchestrates **primals**, not **workloads**

**Correct**: 
- biomeOS spawns ToadStool
- ToadStool executes workloads

---

### Misconception 2: "biomeOS parses biome.yaml"

**Wrong**: biomeOS only reads `tower.toml`

**Correct**:
- biomeOS parses `tower.toml` (primal infrastructure)
- ToadStool parses `biome.yaml` (application workloads)

---

### Misconception 3: "ToadStool is just another primal"

**Partially Correct**: ToadStool **IS** a primal (from biomeOS perspective)

**But Also**: ToadStool is **THE** workload orchestrator (from user perspective)

ToadStool has dual responsibility:
1. As a primal: Register with biomeOS, provide Compute capability
2. As orchestrator: Execute user workloads from biome.yaml

---

## 🚀 Implementation Status

| Component | Status | Location |
|-----------|--------|----------|
| **biomeOS Capability Registry** | ✅ Complete | `biomeos-core/src/capability_registry.rs` |
| **tower CLI** | ✅ Complete | `biomeos-core/src/bin/tower.rs` |
| **ToadStool Executor** | ✅ Complete | `phase1/toadstool/crates/cli/src/executor/` |
| **ToadStool biomeOS Client** | ❌ Needed | (3-4 hours) |
| **Documentation** | ✅ This Document | `docs/ARCHITECTURE_LAYERS.md` |

---

## 📋 Next Steps

### For biomeOS Team
- ✅ Capability registry implemented
- ✅ Documentation clarified
- 🟢 Ready for primal integration

### For ToadStool Team
- ✅ Workflow executor complete
- ❌ Need: BiomeOSClient module
- ❌ Need: Capability-based discovery
- **Estimated**: 3-4 hours

### For Application Developers
- ✅ Clear separation: `tower.toml` vs `biome.yaml`
- ✅ Know where to define infrastructure vs applications
- ✅ Understand two-level orchestration model

---

**Status**: Documentation complete. Architecture clarified. Ready for integration!

