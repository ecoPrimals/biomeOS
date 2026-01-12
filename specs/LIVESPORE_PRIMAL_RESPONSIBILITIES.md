# 🌱 LiveSpore: Primal Responsibilities & Coordination

**Version**: 1.0.0  
**Date**: January 12, 2026  
**Status**: 🎯 ARCHITECTURAL GUIDELINE  
**Purpose**: Define what biomeOS orchestrates vs what primals implement

---

## 🎯 Core Principle

**biomeOS is the ORCHESTRATOR, not the IMPLEMENTER.**

biomeOS coordinates primals via capability-based discovery. Each primal owns its domain expertise. biomeOS NEVER reimplements primal capabilities - it discovers and delegates to them.

---

## 🧬 Responsibility Matrix

### biomeOS Responsibilities (Orchestration Only)

| Responsibility | Description | Implementation |
|----------------|-------------|----------------|
| **Deployment Mode Detection** | Detect if running as Cold/Live/Sibling Spore | `spore-detector` using `sys-mount` |
| **Primal Coordination** | Launch primals, manage lifecycle | `spore-deployer` using Neural API graphs |
| **Socket Path Strategy** | Determine socket paths based on deployment mode | `DeploymentMode::socket_prefix()` |
| **Capability Discovery** | Discover primals by capability | Via Songbird registry |
| **Graph Execution** | Execute deployment graphs | `biomeos-graph` crate |
| **Binary Management** | Store/load primal binaries | `plasmidBin/` directory |

### Primal Responsibilities (Implementation)

| Primal | Capabilities | LiveSpore Responsibilities |
|--------|-------------|----------------------------|
| **petalTongue** | `installer.ui`, `system.ui`, `visualization` | ✅ All TUI rendering<br>✅ Installer interface<br>✅ Real-time visualization<br>✅ User interaction |
| **ToadStool** | `hardware.detect`, `compute.execute`, `resources.estimate` | ✅ Hardware scanning (CPU/GPU/disk/network)<br>✅ Compute orchestration<br>✅ Resource estimation |
| **NestGate** | `storage.prepare`, `storage.manage`, `persistence` | ✅ Disk partitioning<br>✅ Filesystem creation<br>✅ Bootloader installation<br>✅ Persistent storage |
| **BearDog** | `encryption`, `tunneling`, `security` | ✅ All communication encryption<br>✅ Cross-network tunnels<br>✅ Genetic lineage verification |
| **Songbird** | `service.registry`, `discovery`, `federation` | ✅ Primal registration<br>✅ Capability discovery<br>✅ Cross-node federation |
| **Squirrel** | `ai.suggest`, `ai.optimize`, `ai.learn` | ✅ Installation suggestions<br>✅ Partition optimization<br>✅ Deployment strategy |

---

## 🔀 LiveSpore Workflows

### Workflow 1: Cold Spore Boot (USB)

```
┌─────────────────────────────────────────────────────────────┐
│ 1. spore-detector (biomeOS)                                 │
│    - Detects: Running from USB                              │
│    - Sets: DeploymentMode::ColdSpore                        │
│    - Determines: Socket paths → /media/usb0/runtime/        │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. spore-deployer (biomeOS)                                 │
│    - Loads: graphs/nucleus.toml                             │
│    - Adapts: Socket paths for Cold Spore mode               │
│    - Launches: Primals in correct order                     │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. Primals Start (in order)                                 │
│    ├─ BearDog → /media/usb0/runtime/beardog-nat0.sock      │
│    ├─ Songbird → /media/usb0/runtime/songbird-nat0.sock    │
│    ├─ ToadStool → /media/usb0/runtime/toadstool-nat0.sock  │
│    ├─ NestGate → /media/usb0/runtime/nestgate-nat0.sock    │
│    ├─ Squirrel → /media/usb0/runtime/squirrel-nat0.sock    │
│    └─ petalTongue → Launches GUI                           │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 4. ToadStool (delegated)                                    │
│    - Scans hardware: CPU, GPU, disks, network               │
│    - Reports to biomeOS via JSON-RPC                        │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 5. petalTongue (delegated)                                  │
│    - Renders UI: "Welcome to biomeOS LiveSpore!"            │
│    - Shows: Hardware status, deployment options             │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 6. User Interaction                                         │
│    - Option 1: Run from USB (already running!)              │
│    - Option 2: Install to disk → Workflow 2                 │
│    - Option 3: Shutdown                                     │
└─────────────────────────────────────────────────────────────┘
```

**biomeOS Role**: Detect mode, set socket paths, launch primals  
**Primal Roles**: ToadStool (hardware), petalTongue (UI), NestGate (storage)

---

### Workflow 2: Installation to Bare Metal

```
┌─────────────────────────────────────────────────────────────┐
│ 1. User Clicks "Install to Disk" in petalTongue UI          │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. petalTongue → biomeOS (JSON-RPC)                         │
│    Method: installer.start()                                │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. biomeOS → ToadStool (JSON-RPC)                           │
│    Method: hardware.list_block_devices()                    │
│    Response: [/dev/nvme0n1 (500GB), /dev/sda (1TB)]        │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 4. biomeOS → petalTongue (JSON-RPC)                         │
│    Method: installer.show_disk_selection(disks)             │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 5. petalTongue (TUI rendering)                              │
│    - Shows: Disk selection UI                               │
│    - User selects: /dev/nvme0n1                             │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 6. biomeOS → Squirrel (JSON-RPC) [OPTIONAL]                 │
│    Method: installer.suggest_partition_strategy(disk)       │
│    Response: { strategy: "dual_boot", sizes: {...} }        │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 7. biomeOS → petalTongue (JSON-RPC)                         │
│    Method: installer.confirm(disk, strategy)                │
│    petalTongue shows: "Install to /dev/nvme0n1? [Yes/No]"  │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 8. biomeOS → NestGate (JSON-RPC)                            │
│    Method: storage.prepare(disk, strategy)                  │
│    NestGate:                                                │
│      - Partitions disk (gptman crate)                       │
│      - Formats partitions (ext4/btrfs)                      │
│      - Mounts partitions                                    │
│    Response: { root: /mnt/biomeos, boot: /mnt/biomeos/boot }│
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 9. biomeOS (orchestration)                                  │
│    - Copies primal binaries to /mnt/biomeos/plasmidBin/     │
│    - Copies graphs to /mnt/biomeos/graphs/                  │
│    - Generates deployment config                            │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 10. biomeOS → NestGate (JSON-RPC)                           │
│     Method: storage.finalize(partitions)                    │
│     NestGate:                                               │
│       - Installs bootloader (via existing OS or UEFI)       │
│       - Unmounts partitions                                 │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 11. biomeOS → petalTongue (JSON-RPC)                        │
│     Method: installer.show_success()                        │
│     petalTongue shows: "Installation complete! Reboot?"     │
└─────────────────────────────────────────────────────────────┘
```

**biomeOS Role**: Coordinate workflow, copy binaries, generate config  
**petalTongue Role**: All UI rendering and user interaction  
**ToadStool Role**: Hardware detection  
**NestGate Role**: All disk operations (partition, format, bootloader)  
**Squirrel Role**: AI-assisted suggestions (optional)

---

### Workflow 3: Sibling Spore on Mac/Linux/Windows

```
┌─────────────────────────────────────────────────────────────┐
│ 1. User runs: ./spore-detector                              │
│    (on existing OS - Mac/Linux/Windows)                     │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. spore-detector (biomeOS)                                 │
│    - Detects: Not on removable media, not in root           │
│    - Sets: DeploymentMode::SiblingSpore                     │
│    - Determines: Socket paths → ~/.local/share/biomeos/     │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. spore-deployer (biomeOS)                                 │
│    - Loads: graphs/nucleus.toml                             │
│    - Adapts: Socket paths for Sibling Spore mode            │
│    - Launches: Primals with user-space paths                │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 4. Primals Start (user-space)                               │
│    ├─ BearDog → ~/.local/share/biomeos/beardog-nat0.sock   │
│    ├─ Songbird → ~/.local/share/biomeos/songbird-nat0.sock │
│    └─ ... (all other primals)                               │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 5. spore-bridge (biomeOS)                                   │
│    - Announces via mDNS: "_biomeos._tcp.local."             │
│    - Discovers other LiveSpore instances on LAN             │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 6. Discovers: Linux Server (Live Spore - bare metal)        │
│    - Connects via JSON-RPC over TCP                         │
│    - Federates via Songbird                                 │
└─────────────────────────────────────────────────────────────┘
```

**biomeOS Role**: Detect mode, adapt paths, coordinate federation  
**Songbird Role**: Federation and cross-mode discovery  
**BearDog Role**: Secure tunnels for cross-OS communication

---

## 📋 Primal Handoff Documents

### For petalTongue Team

**New Capabilities Needed for LiveSpore**:

1. **`installer.ui` capability**:
   - `installer.show_welcome()` → Show welcome screen
   - `installer.show_disk_selection(disks: Vec<Disk>)` → Let user select disk
   - `installer.confirm(disk: Disk, strategy: PartitionStrategy)` → Confirmation dialog
   - `installer.show_progress(status: String, percent: u8)` → Progress bar
   - `installer.show_success()` → Success screen

2. **`deployment.ui` capability**:
   - `deployment.show_mode_selection()` → Let user choose Cold/Live/Sibling
   - `deployment.show_status(mode: DeploymentMode, primals: Vec<PrimalStatus>)` → Live status

**Implementation Timeline**: Phase 4 (Weeks 8-9)

**JSON-RPC Methods**:
```rust
// Example method signature
pub async fn show_disk_selection(&self, disks: Vec<DiskInfo>) -> Result<DiskSelection>
```

---

### For ToadStool Team

**New Capabilities Needed for LiveSpore**:

1. **`hardware.detect` capability**:
   - `hardware.list_block_devices()` → List all disks
   - `hardware.get_system_info()` → CPU, RAM, GPU, network
   - `hardware.detect_removable_media()` → Identify USB/SD cards

2. **`hardware.validate` capability**:
   - `hardware.validate_install_target(disk: Disk)` → Check if disk is suitable
   - `hardware.estimate_install_time(disk: Disk)` → Estimate time

**Implementation Timeline**: Phase 2 (Weeks 3-5)

**JSON-RPC Methods**:
```rust
// Example method signature
pub async fn list_block_devices(&self) -> Result<Vec<BlockDevice>>
```

---

### For NestGate Team

**New Capabilities Needed for LiveSpore**:

1. **`storage.prepare` capability**:
   - `storage.prepare(disk: Disk, strategy: PartitionStrategy)` → Partition & format
   - `storage.finalize(partitions: Partitions)` → Install bootloader, unmount

2. **`storage.bootloader` capability**:
   - `storage.install_bootloader(disk: Disk, os: HostOS)` → Install/update bootloader
   - `storage.add_boot_entry(name: String, path: PathBuf)` → Add boot menu entry

**Implementation Timeline**: Phase 4 (Weeks 8-9)

**JSON-RPC Methods**:
```rust
// Example method signature
pub async fn prepare(&self, disk: Disk, strategy: PartitionStrategy) -> Result<Partitions>
```

---

### For Squirrel Team

**New Capabilities Needed for LiveSpore**:

1. **`installer.suggest` capability**:
   - `installer.suggest_partition_strategy(disk: Disk)` → AI-suggested partitioning
   - `installer.suggest_dual_boot(existing_os: OS)` → Dual-boot recommendations

2. **`deployment.optimize` capability**:
   - `deployment.suggest_mode(hardware: HardwareInfo)` → Suggest Cold/Live/Sibling
   - `deployment.optimize_primal_placement(hardware: HardwareInfo)` → GPU assignment, etc.

**Implementation Timeline**: Phase 5 (Weeks 10-12) - Optional enhancement

**JSON-RPC Methods**:
```rust
// Example method signature
pub async fn suggest_partition_strategy(&self, disk: &Disk) -> Result<PartitionStrategy>
```

---

## 🚫 What biomeOS Should NEVER Implement

| ❌ DO NOT | ✅ DO INSTEAD |
|-----------|---------------|
| Direct TUI rendering | Delegate to petalTongue via `installer.ui` |
| Direct hardware scanning | Delegate to ToadStool via `hardware.detect` |
| Disk partitioning logic | Delegate to NestGate via `storage.prepare` |
| Bootloader installation | Delegate to NestGate via `storage.bootloader` |
| AI suggestions | Delegate to Squirrel via `installer.suggest` |

**biomeOS owns**: Mode detection, socket paths, primal lifecycle, graph execution  
**Primals own**: Everything else

---

## 🧪 Testing Strategy

### Unit Tests (biomeOS)
- ✅ `DeploymentMode` detection logic
- ✅ Socket path generation for each mode
- ✅ Graph adaptation for deployment modes

### Integration Tests (biomeOS + Primals)
- ✅ Cold Spore boot with mocked primals
- ✅ Installation workflow (biomeOS → petalTongue → ToadStool → NestGate)
- ✅ Sibling Spore federation

### E2E Tests (Full System)
- ✅ Cold Spore boot on real hardware
- ✅ Install to bare metal
- ✅ Sibling Spore on Mac/Linux/Windows
- ✅ Cross-mode discovery and federation

---

## 📊 Capability Coverage Matrix

| Capability | Primal | LiveSpore Phase | Status |
|------------|--------|-----------------|--------|
| `installer.ui` | petalTongue | Phase 4 | 🟡 Pending primal impl |
| `hardware.detect` | ToadStool | Phase 2 | 🟡 Pending primal impl |
| `storage.prepare` | NestGate | Phase 4 | 🟡 Pending primal impl |
| `storage.bootloader` | NestGate | Phase 4 | 🟡 Pending primal impl |
| `installer.suggest` | Squirrel | Phase 5 | 🟡 Optional |
| `deployment.mode` | biomeOS | Phase 1 | 🟢 Ready to implement |
| `deployment.graphs` | biomeOS | Phase 1 | ✅ Already exists |

---

## 🎯 Summary

**biomeOS is an orchestrator, NOT a primal.**

When implementing LiveSpore:
- ✅ Detect deployment mode (biomeOS owns this)
- ✅ Set socket paths (biomeOS owns this)
- ✅ Launch primals via graphs (biomeOS owns this)
- ✅ Coordinate workflows (biomeOS owns this)
- ❌ Never implement TUI (petalTongue owns this)
- ❌ Never implement hardware detection (ToadStool owns this)
- ❌ Never implement storage logic (NestGate owns this)

**Different orders of the same architecture.** 🍄🐸

---

**Status**: 🎯 Architectural Guideline Complete  
**Next**: Create primal handoff documents as we enter each phase

---

*biomeOS: The orchestrator of a self-sovereign ecosystem.*

