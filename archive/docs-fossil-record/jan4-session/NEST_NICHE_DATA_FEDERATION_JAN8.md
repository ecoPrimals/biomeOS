# 🗄️ Nest Niche - Data Federation Architecture

**Date:** January 8, 2026  
**Status:** 🎯 **DESIGN PHASE**  
**Philosophy:** "Data is physical, compute is energy - paradigm inversion"

---

## 🎯 Vision: Inverting the Traditional Paradigm

### **Traditional Paradigm (WRONG)**
```
Data: Cheap, easy to copy, replicate freely
Compute: Expensive, scarce, valuable

Result:
• Data replicated everywhere (waste)
• No provenance or ownership
• Compute bottleneck
• Energy inefficiency
```

### **ecoPrimals Paradigm (CORRECT)**
```
Data: Physical object with origin, ownership, location
Compute: Energy that moves to data, does work

Result:
• Data stays put (efficiency)
• Clear provenance & ownership
• Compute as energy (flows to data)
• Energy efficiency
```

---

## 🏗️ Complete Architecture: The 4 Niches

```
┌─────────────────────────────────────────────────────────────┐
│                  ECOPRIMALS ECOSYSTEM                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  🗼 TOWERS (Communication Niche)                             │
│     Songbird + BearDog + biomeOS                            │
│     Purpose: Discovery, federation, P2P mesh                │
│     Vertical: Between physical machines                     │
│                                                              │
│  🖥️ NODES (Compute Niche)                                   │
│     Toadstool + (optional BearDog)                          │
│     Purpose: Workload execution, fractal compute            │
│     Horizontal: Within/across machines                      │
│                                                              │
│  🗄️ NESTS (Data Niche) ← NEW!                               │
│     NestGate + BearDog + Songbird                           │
│     Purpose: Data federation, provenance, ownership         │
│     Physical: Data as objects with location                 │
│                                                              │
│  🖧 GATES (Physical Metal)                                   │
│     The actual hardware                                     │
│     Purpose: Physical resources (CPU, GPU, Storage, Net)    │
│     Foundation: Where everything runs                       │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## 🗄️ Nest Architecture: Data as Physical Objects

### **Core Principles**

#### **1. Data Has Provenance**
```rust
struct DataObject {
    id: DataId,
    origin: Origin {
        creator: DID,           // Who created it
        created_at: Timestamp,  // When
        location: PhysicalNest, // Where (original)
    },
    ownership: Ownership {
        owner: DID,             // Current owner
        permissions: Vec<Permission>,
        lineage: Vec<Transfer>, // Chain of custody
    },
    physical: PhysicalInfo {
        size_bytes: u64,
        checksum: Hash,
        current_location: NestId,
        shards: Vec<ShardLocation>, // If sharded
    },
}
```

#### **2. Data Stays Put (Minimize Copy)**
```
BEFORE (Traditional):
User requests data → Copy to compute → Process → Copy back

AFTER (ecoPrimals):
User requests compute → Compute moves to data → Process in-place → Return result only
```

#### **3. Compute as Energy (Flows to Data)**
```
DataNest (100GB file)
    ↓
ComputeNode sent to nest
    ↓
Work done in-place
    ↓
Result (1KB) returned
    
Energy: Compute moved (lightweight)
NOT Data moved (heavy)
```

#### **4. Sharding for Federation**
```
Large Data (1TB)
    ├── Shard 0 (256GB) → Nest-Alpha
    ├── Shard 1 (256GB) → Nest-Beta
    ├── Shard 2 (256GB) → Nest-Gamma
    └── Shard 3 (256GB) → Nest-Delta

Provenance:
  ShardManifest {
      parent_id: original_data_id,
      shards: [
          {id, location, size, checksum},
          ...
      ],
      reconstruction_metadata
  }
```

---

## 🎨 Nest Niche Components

### **Core Stack**
```
┌─────────────────────────────────────────────┐
│            Data Nest                         │
├─────────────────────────────────────────────┤
│  NestGate                                    │
│  • Storage gateway                           │
│  • Data provenance tracking                  │
│  • Adaptive compression (8:1 ratio)          │
│  • Zero-copy operations                      │
├─────────────────────────────────────────────┤
│  BearDog (Mandatory)                         │
│  • Encryption at rest                        │
│  • Encryption in transit                     │
│  • Access control (genetic lineage)          │
│  • Data ownership verification               │
├─────────────────────────────────────────────┤
│  Songbird (Federation)                       │
│  • Data discovery across nests               │
│  • Shard location tracking                   │
│  • Encrypted data transfer                   │
│  • Load balancing for data access            │
└─────────────────────────────────────────────┘
```

### **Physical Layer**
```
Gate (Physical Hardware)
├── Storage: NVMe, SSD, HDD, ZFS
├── Network: 10Gb, 40Gb, InfiniBand
└── Compute: CPU for in-place operations
```

---

## 📦 BYOB Manifest: `nest.toml`

```toml
# =============================================================================
# NEST NICHE - Data Federation
# =============================================================================
#
# Purpose: Data storage, provenance, ownership, federation
# Philosophy: Data is physical, compute is energy
#
# Components:
#   - NestGate: Storage gateway (adaptive compression, provenance)
#   - BearDog: Encryption & access control (mandatory)
#   - Songbird: Federation & discovery
#
# Created: January 8, 2026
# Status: Design Phase
#
# =============================================================================

[niche]
name = "nest"
version = "1.0.0"
type = "data"
description = "Data federation with provenance and ownership"
architecture = "physical"  # Data bound to physical location

# Genetic lineage (from parent spore or tower)
family_seed_file = "./.family.seed"

# =============================================================================
# PRIMALS CONFIGURATION
# =============================================================================

# Core: NestGate (Storage Gateway)
[[primals]]
binary = "./primals/nestgate"
provides = [
    "storage",
    "data-provenance",
    "adaptive-compression",
    "zero-copy-operations",
    "data-federation",
    "shard-management"
]
requires = ["security", "federation"]  # Needs BearDog + Songbird

[primals.env]
# Node identity
NESTGATE_NODE_ID = "${NODE_ID}"
NESTGATE_FAMILY_ID = "${FAMILY_ID}"

# Storage configuration
NESTGATE_STORAGE_PATH = "${STORAGE_PATH:-/data/nest}"
NESTGATE_CACHE_PATH = "${CACHE_PATH:-/cache/nest}"
NESTGATE_ZFS_ENABLED = "${ZFS_ENABLED:-false}"
NESTGATE_ZFS_POOL = "${ZFS_POOL:-nestpool}"

# Adaptive compression
NESTGATE_COMPRESSION_ENABLED = "${COMPRESSION_ENABLED:-true}"
NESTGATE_COMPRESSION_LEVEL = "${COMPRESSION_LEVEL:-adaptive}"  # adaptive|fast|max
NESTGATE_ENTROPY_THRESHOLD = "${ENTROPY_THRESHOLD:-6.0}"  # Skip compression if entropy > 6.0

# Data provenance
NESTGATE_PROVENANCE_ENABLED = "true"  # Always track
NESTGATE_OWNERSHIP_VERIFICATION = "true"  # Verify with BearDog

# Sharding
NESTGATE_SHARDING_ENABLED = "${SHARDING_ENABLED:-false}"
NESTGATE_SHARD_SIZE_GB = "${SHARD_SIZE_GB:-256}"
NESTGATE_REPLICATION_FACTOR = "${REPLICATION_FACTOR:-3}"

# Communication
NESTGATE_UNIX_SOCKET = "/tmp/nestgate-${NODE_ID}.sock"
NESTGATE_HTTP_ENABLED = "${HTTP_ENABLED:-false}"
NESTGATE_HTTP_PORT = "${HTTP_PORT:-9093}"

# Logging
RUST_LOG = "${LOG_LEVEL:-info}"

# Mandatory: BearDog (Encryption & Security)
[[primals]]
binary = "./primals/beardog-server"
provides = [
    "encryption",
    "decryption",
    "access-control",
    "ownership-verification",
    "genetic-lineage"
]
requires = []

[primals.env]
# Identity
BEARDOG_NODE_ID = "${NODE_ID}"
BEARDOG_FAMILY_ID = "${FAMILY_ID}"
BEARDOG_FAMILY_SEED_FILE = "./.family.seed"

# Mode
BEARDOG_MODE = "data-vault"  # Specialized for data encryption
BEARDOG_HSM_MODE = "software"  # Or "hardware" if available

# Communication
BEARDOG_HTTP_ENABLED = "false"  # Unix socket only
# Socket auto-created at: /tmp/beardog-${FAMILY_ID}-${NODE_ID}.sock

# Logging
RUST_LOG = "${LOG_LEVEL:-info}"

# Federation: Songbird (Discovery & Transfer)
[[primals]]
binary = "./primals/songbird-orchestrator"
provides = [
    "discovery",
    "federation",
    "p2p",
    "encrypted-transfer"
]
requires = ["security"]

[primals.env]
# Identity
SONGBIRD_NODE_ID = "${NODE_ID}"
SONGBIRD_FAMILY_ID = "${FAMILY_ID}"

# Discovery
SONGBIRD_UDP_MULTICAST = "${UDP_MULTICAST:-true}"
SONGBIRD_MDNS_ENABLED = "${MDNS_ENABLED:-true}"

# P2P
SONGBIRD_BTSP_ENABLED = "true"  # For encrypted data transfer

# Communication
# Socket auto-created at: /tmp/songbird-${NODE_ID}.sock

# Logging
RUST_LOG = "${LOG_LEVEL:-info}"

# =============================================================================
# NEST CONFIGURATION
# =============================================================================

[nest]
# Storage backend
storage_type = "${STORAGE_TYPE:-zfs}"  # zfs|ext4|btrfs|xfs
storage_path = "${STORAGE_PATH:-/data/nest}"

# Capacity
total_capacity_gb = "${TOTAL_CAPACITY_GB:-1000}"
reserved_capacity_gb = "${RESERVED_CAPACITY_GB:-100}"
max_file_size_gb = "${MAX_FILE_SIZE_GB:-1000}"

# Data policies
immutable_by_default = "${IMMUTABLE:-false}"
versioning_enabled = "${VERSIONING:-true}"
deduplication_enabled = "${DEDUP:-true}"

# =============================================================================
# DATA PROVENANCE
# =============================================================================

[provenance]
# Track everything
enabled = true
track_creator = true
track_creation_time = true
track_modifications = true
track_access = true
track_transfers = true

# Lineage
genetic_lineage_required = true  # Only family members can access
ownership_verification = true  # Verify with BearDog

# Metadata storage
metadata_backend = "sqlite"  # sqlite|postgres
metadata_path = "${STORAGE_PATH}/provenance.db"

# =============================================================================
# SHARDING & FEDERATION
# =============================================================================

[sharding]
# Enable sharding for large files
enabled = "${SHARDING_ENABLED:-false}"
threshold_gb = "${SHARD_THRESHOLD_GB:-100}"  # Shard files > 100GB
shard_size_gb = "${SHARD_SIZE_GB:-256}"

# Shard placement strategy
placement_strategy = "${PLACEMENT_STRATEGY:-round-robin}"  # round-robin|load-balanced|affinity

# Replication
replication_enabled = "${REPLICATION_ENABLED:-true}"
replication_factor = "${REPLICATION_FACTOR:-3}"  # 3 copies
replication_strategy = "${REPLICATION_STRATEGY:-genetic-affinity}"  # same family preferred

# =============================================================================
# COMPRESSION & OPTIMIZATION
# =============================================================================

[compression]
# Adaptive compression
enabled = true
mode = "adaptive"  # adaptive|always|never

# Entropy-based decisions
entropy_threshold = 6.0  # Skip compression if entropy > 6.0
format_detection = true  # Detect FASTA, FASTQ, etc.

# Compression algorithms
algorithm_genomic = "zstd"  # For genomic data (FASTA, FASTQ)
algorithm_text = "lz4"  # For text/logs
algorithm_binary = "none"  # Skip for binary/encrypted

# Performance
compression_threads = "${COMPRESSION_THREADS:-4}"
compression_level = "${COMPRESSION_LEVEL:-3}"  # 1-9

# =============================================================================
# ENCRYPTION
# =============================================================================

[encryption]
# Always encrypt at rest
enabled = true
algorithm = "AES-256-GCM"
key_derivation = "HKDF-SHA256"

# Encryption strategy
encrypt_all = true
encrypt_metadata = true
encrypt_shards = true

# Key management (via BearDog)
key_rotation_days = 90
key_backup_enabled = true

# =============================================================================
# FEDERATION
# =============================================================================

[federation]
# Data discovery
discover_other_nests = true
advertise_capacity = true
advertise_shards = true

# Genetic lineage
family_seed_file = "./.family.seed"
family_id = "${FAMILY_ID}"

# Sub-federations (granular access)
sub_federations = []  # e.g., ["genomics", "medical", "public"]

# Trust model
require_genetic_verification = true  # Only family members
allow_cross_family_read = false  # Strict isolation

# =============================================================================
# MONITORING
# =============================================================================

[monitoring]
# Metrics
prometheus_enabled = "${PROMETHEUS_ENABLED:-true}"
prometheus_port = "${PROMETHEUS_PORT:-9093}"

# Track data metrics
track_storage_utilization = true
track_io_operations = true
track_compression_ratio = true
track_access_patterns = true

# Logging
log_level = "${LOG_LEVEL:-info}"
log_file = "/var/log/nest-${NODE_ID}.log"
log_fossil_enabled = true

# Health checks
health_check_interval = 60
health_check_storage = true
health_check_beardog = true
health_check_songbird = true

# =============================================================================
# ADVANCED: COMPUTE-TO-DATA
# =============================================================================

[compute_to_data]
# Allow compute nodes to execute on data in-place
enabled = "${COMPUTE_TO_DATA:-true}"

# Sandboxing
sandbox_enabled = true
isolation_mode = "container"  # container|vm|process

# Resource limits
max_cpu_percent = 50  # Don't starve storage operations
max_memory_gb = 32
max_execution_time_minutes = 60

# Trusted compute
require_genetic_lineage = true  # Only family compute nodes
verify_workload_signature = true  # Sign with BearDog

# =============================================================================
# EXAMPLE: Westgate (76TB ZFS NAS)
# =============================================================================

# export NODE_ID=nest-westgate
# export FAMILY_ID=nat0
# export STORAGE_PATH=/mnt/zfs/nestpool
# export TOTAL_CAPACITY_GB=76000
# export ZFS_ENABLED=true
# export ZFS_POOL=nestpool
# biomeos deploy --niche nest --config deployments/basement-hpc/westgate-nest.toml

