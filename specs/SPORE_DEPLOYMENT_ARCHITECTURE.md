# Spore Deployment Architecture

**Version**: 2.0.0  
**Date**: January 16, 2026  
**Status**: Production Architecture  
**Evolution**: From HPC-Anchored to HSM-Anchored

---

## 🎯 Architectural Evolution

### Traditional Model (HPC-Anchored)
```
Basement HPC (Anchor)
  ├─> Stores family_seed
  ├─> Always-on infrastructure
  └─> Portable HSM for authentication

Compute: Local (covalent)
Security: External device (ionic)
```

### Evolved Model (HSM-Anchored) ⭐
```
Pixel HSM (Anchor)
  ├─> Stores family_seed (hardware-backed)
  ├─> Always with you
  ├─> Multiple spores per project
  └─> Compute resources are services (ionic!)

Compute: HPC/Cloud (ionic services)
Security: Always-present HSM (anchor)
```

**Key Insight**: Your **security root travels with you**, compute is consumed as needed!

---

## 🌱 Spore Types on HSM Device

### 1. ColdSpores (Archive/Storage)
**Purpose**: Genetic material preserved for storage

**Characteristics**:
- Read-only archives
- Backup copies
- Historical snapshots
- Project archives

**Storage Location**: `/storage/emulated/0/coldSpores/`

**Examples**:
```
/coldSpores/
  ├─ project-alpha-v1.0/     # Archived project state
  ├─ family-backup-2026-01/  # Family seed backup
  ├─ deployment-snapshot/    # Deployment configuration
  └─ primal-binaries-arm64/  # Binary archives
```

### 2. LiveSpores (Active Projects)
**Purpose**: Ready-to-deploy project environments

**Characteristics**:
- Executable binaries
- Active configuration
- Ready for immediate deployment
- Project-specific families

**Storage Location**: `/storage/emulated/0/liveSpores/`

**Examples**:
```
/liveSpores/
  ├─ biomeOS-main/          # Main biomeOS project
  ├─ research-project-A/    # Research subproject
  ├─ client-deployment/     # Client-specific deployment
  └─ experimental-feature/  # Experimental branch
```

### 3. HSM Anchor (Security Root)
**Purpose**: Hardware-backed security foundation

**Characteristics**:
- Stores master family_seed (hardware keystore)
- Biometric-protected
- Always available
- Derives child seeds for projects

**Storage Location**: Android Keystore (hardware-backed)

**Hierarchy**:
```
Master Family Seed (Hardware Keystore)
  ├─> Project Alpha Seed (derived)
  ├─> Research Project Seed (derived)
  ├─> Client Deployment Seed (derived)
  └─> Experimental Seed (derived)
```

---

## 🏗️ Multi-Spore Architecture

### Pixel HSM as Genetic Anchor

```
Pixel 8a (HSM Anchor)
  │
  ├─ Master Seed (Titan M2 Hardware Keystore)
  │   ├─> Biometric-protected
  │   ├─> Never exported
  │   └─> Derives all project seeds
  │
  ├─ ColdSpores (Storage)
  │   ├─ Archives and backups
  │   ├─ Read-only genetic material
  │   └─ Historical snapshots
  │
  └─ LiveSpores (Active Projects)
      ├─ biomeOS Main
      │   ├─> Family: biomeos_main
      │   ├─> Binaries: ARM64 native
      │   └─> Status: Active development
      │
      ├─ Research Project A
      │   ├─> Family: research_a
      │   ├─> Binaries: Cross-compiled
      │   └─> Status: Experimental
      │
      └─ Client Deployment
          ├─> Family: client_x
          ├─> Binaries: Production
          └─> Status: Production-ready

External Compute (Ionic Services)
  ├─ Basement HPC (covalent if joined, ionic if service)
  ├─ Cloud GPU (ionic - contract-based)
  └─ University Cluster (metallic - shared resources)
```

---

## 🚀 Deployment Modes

### Mode 1: HSM-Only (Anchor + Light Services)

**Use Case**: Portable security root, minimal compute

**Deployed**:
- BearDog HSM (hardware-backed)
- Minimal Songbird (discovery only)
- Storage for spores

**Bonding**:
- External compute via ionic (contract-based)
- HSM provides security services to any device
- Portable genetic anchor

**Command**:
```bash
# Deploy HSM anchor on Pixel
./spore-deploy pixel-hsm \
  --mode hsm_anchor \
  --hardware-backed \
  --multi-spore
```

### Mode 2: Full NUCLEUS (Complete Ecosystem)

**Use Case**: Independent mobile ecosystem

**Deployed**:
- BearDog HSM (hardware-backed)
- Songbird (full mesh)
- ToadStool (mobile compute)
- NestGate (storage)

**Bonding**:
- Internal covalent (shared electrons)
- Can join external meshes
- Full primal capabilities

**Command**:
```bash
# Deploy full NUCLEUS on Pixel
./spore-deploy pixel-nucleus \
  --mode full_nucleus \
  --family pixel_main \
  --hardware-backed
```

### Mode 3: Hybrid (HSM + Selective Services)

**Use Case**: Anchor + specific capabilities

**Deployed**:
- BearDog HSM (always)
- Selected primals per project
- Project-specific configurations

**Bonding**:
- Mix of covalent and ionic
- Flexible based on project needs

**Command**:
```bash
# Deploy hybrid configuration
./spore-deploy pixel-hybrid \
  --mode hybrid \
  --services beardog,songbird \
  --family project_alpha
```

---

## 📦 Spore Deployment Framework

### Unified Deployment Tool

**Purpose**: Support all deployment targets (USB, Pixel, HPC, Cloud)

**Capabilities**:
1. **Cross-Compilation**: Build for any target architecture
2. **Native Build**: Build on target device
3. **Multi-Spore**: Manage multiple spores per device
4. **HSM Integration**: Hardware security module support
5. **Bonding Configuration**: Specify interaction patterns

### Directory Structure

```
Spore Root/
  ├─ meta/
  │   ├─ spore.toml          # Spore metadata
  │   ├─ bonding.toml        # Bonding configuration
  │   └─ manifest.json       # Deployment manifest
  │
  ├─ primals/
  │   ├─ beardog-server      # Primal binaries
  │   ├─ songbird-orchestrator
  │   ├─ toadstool
  │   └─ nestgate
  │
  ├─ graphs/
  │   ├─ 01_nucleus.toml     # Deployment graphs
  │   ├─ hsm_anchor.toml
  │   └─ hybrid_mode.toml
  │
  ├─ config/
  │   ├─ family.toml         # Family configuration
  │   ├─ security.toml       # Security settings
  │   └─ network.toml        # Network configuration
  │
  └─ seeds/
      └─ .family.seed.enc    # Encrypted family seed
```

### Spore Metadata Format

```toml
# spore.toml
[spore]
id = "biomeOS-main-v1.0"
type = "live"  # live, cold, hybrid
version = "1.0.0"
created = "2026-01-16T14:00:00Z"
architecture = "aarch64-linux-android"

[deployment]
mode = "hsm_anchor"  # hsm_anchor, full_nucleus, hybrid
target = "pixel_8a"
hardware_backed = true

[bonding]
internal = "covalent"  # Internal bonding type
default_external = "ionic"  # Default external bonding

[family]
id = "biomeos_main"
master_seed_source = "hardware_keystore"  # or "file", "usb"
derived_from = "master_seed"

[primals]
beardog = { enabled = true, hsm_mode = true }
songbird = { enabled = true, mode = "light" }
toadstool = { enabled = false }
nestgate = { enabled = false }

[storage]
coldspores = "/storage/emulated/0/coldSpores"
livespores = "/storage/emulated/0/liveSpores"
```

---

## 🔐 HSM Anchor Pattern

### Master Seed Hierarchy

```
Master Seed (Titan M2 Keystore)
  │
  ├─ Biometric Protection
  │   ├─> Fingerprint unlock
  │   ├─> Face unlock
  │   └─> PIN fallback
  │
  ├─ Derived Seeds (BIP-32-like)
  │   ├─> biomeos_main     (m/0')
  │   ├─> research_a       (m/1')
  │   ├─> client_x         (m/2')
  │   └─> experimental     (m/3')
  │
  └─ Capabilities
      ├─> JWT generation (hardware-backed)
      ├─> Key derivation (secure enclave)
      ├─> Encryption (hardware accelerated)
      └─> Signing (tamper-resistant)
```

### BearDog HSM Configuration

```toml
# config/beardog-hsm.toml
[hsm]
enabled = true
hardware_backend = "android_keystore"

[hsm.keystore]
use_strongbox = true  # Use Titan M2 if available
biometric_auth = true
key_attestation = true
user_authentication_validity = 3600  # 1 hour

[hsm.capabilities]
jwt_generation = true
key_derivation = true
encryption = true
signing = true

[hsm.master_seed]
storage = "hardware_keystore"
key_alias = "biomeos_master_seed"
derivation_path = "m/44'/0'/0'"

[network]
# HSM can provide services over network (ionic bonding)
listen_address = "0.0.0.0:8080"
allow_network = true
require_tls = true
```

---

## 🚀 Deployment Commands

### Deploy HSM Anchor to Pixel

```bash
# Cross-compile for Android
cargo build --release \
  --target aarch64-linux-android \
  --bin beardog-server

# Create spore
./spore-create pixel-hsm \
  --type live \
  --mode hsm_anchor \
  --target pixel_8a \
  --hardware-backed

# Deploy to Pixel
./spore-deploy pixel-hsm \
  --device pixel_8a \
  --via adb

# Or native build on Pixel
./spore-deploy pixel-hsm \
  --device pixel_8a \
  --build native
```

### Deploy Additional LiveSpore

```bash
# Create project-specific spore
./spore-create research-project-a \
  --type live \
  --mode hybrid \
  --family research_a \
  --derive-from pixel-hsm

# Deploy alongside existing spores
./spore-deploy research-project-a \
  --device pixel_8a \
  --coexist \
  --port-offset 100  # Avoid port conflicts
```

### Archive to ColdSpore

```bash
# Create cold archive
./spore-freeze biomeos-main-v1.0 \
  --source livespores/biomeos-main \
  --destination coldspores/ \
  --compress

# Later: thaw and deploy
./spore-thaw biomeos-main-v1.0 \
  --destination livespores/ \
  --activate
```

---

## 🧪 Bonding Patterns with HSM Anchor

### Pattern 1: HSM Anchor + HPC Compute (Ionic)

```
Pixel HSM (Anchor)
  ├─> Stores master seed
  ├─> Provides security services (ionic)
  └─> Always available

Basement HPC
  ├─> Requests JWT from Pixel (ionic)
  ├─> Heavy compute workloads
  └─> No security secrets stored

Bonding: Ionic (contract-based, no electron sharing)
```

### Pattern 2: HSM Anchor + Mobile NUCLEUS (Covalent Internal)

```
Pixel Full NUCLEUS
  ├─> HSM (BearDog) - hardware-backed
  ├─> Tower (Songbird) - mobile mesh
  ├─> Node (ToadStool) - mobile compute
  └─> Nest (NestGate) - mobile storage

Bonding: Covalent internal, Ionic to external services
```

### Pattern 3: HSM Anchor + Multi-Project (Hybrid)

```
Pixel HSM
  │
  ├─ Project Alpha (Covalent)
  │   └─> biomeOS main development
  │
  ├─ Project Beta (Ionic)
  │   └─> Client deployment (isolated)
  │
  └─ Research (Covalent + Ionic)
      └─> University collaboration
```

---

## 🎯 Implementation Phases

### Phase 1: Simple Cross-Compile (Immediate)

**Goal**: Get BearDog HSM on Pixel quickly

**Steps**:
1. Cross-compile BearDog for Android
2. Push via ADB
3. Launch in HSM mode
4. Test hardware-backed JWT
5. Validate ionic bonding from desktop

**Timeline**: 1-2 hours

### Phase 2: Robust Spore Framework (Next Session)

**Goal**: Support both cross-compile and native build

**Components**:
- `spore-create`: Create spore packages
- `spore-deploy`: Deploy to any target
- `spore-manage`: Manage multiple spores
- Metadata format standardization

**Timeline**: 1 session

### Phase 3: Multi-Spore Management (Future)

**Goal**: Full multi-project support on single device

**Features**:
- Multiple LiveSpores coexisting
- ColdSpore archival
- Automated seed derivation
- Project isolation
- Resource management

**Timeline**: 2-3 sessions

---

## 📊 Architecture Validation

### Traditional HPC-Anchored
**Pros**: High compute power, always-on
**Cons**: Location-bound, requires infrastructure

### HSM-Anchored (New) ⭐
**Pros**: 
- Security always with you
- Portable genetic root
- Hardware-backed trust
- Multi-project flexibility
- Compute as service (ionic!)

**Cons**:
- Mobile power constraints
- Network dependency for heavy compute

**Solution**: Hybrid approach - HSM anchor + ionic compute services!

---

## 🚀 Recommended Path Forward

### This Session (Simple Start)

```bash
# 1. Cross-compile BearDog
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
cargo build --release \
  --target aarch64-linux-android \
  --bin beardog-server

# 2. Push to Pixel
adb push target/aarch64-linux-android/release/beardog-server \
  /data/local/tmp/biomeos/primals/

# 3. Launch HSM
adb shell "cd /data/local/tmp/biomeos && \
  ./primals/beardog-server --hsm-mode hardware"

# 4. Test from desktop
# Desktop families request JWT from Pixel HSM (ionic!)
```

### Next Session (Robust Framework)

```bash
# Implement spore-create tool
# Implement spore-deploy tool
# Support both cross-compile and native
# Multi-spore management
# ColdSpore archival
```

---

**Status**: 🎯 Architecture Defined  
**Insight**: HSM-Anchored > HPC-Anchored  
**Implementation**: Start simple, evolve robust  
**Next**: Cross-compile BearDog for Pixel! 📱🚀

---

*This architecture evolution validates TRUE PRIMAL principles:*
*Your security root travels with you, compute is discovered at runtime!* ⚛️

