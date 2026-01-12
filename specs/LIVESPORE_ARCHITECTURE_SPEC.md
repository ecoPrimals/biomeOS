# 🌱 LiveSpore Architecture Specification

**Version**: 1.0.0  
**Date**: January 12, 2026  
**Status**: 🔬 RESEARCH & DESIGN  
**Grade**: Planning Phase

---

## 🎯 Core Philosophy

**LiveSpore is NOT a traditional live USB system.**

LiveSpore is a **portable, self-bootstrapping NUCLEUS deployment** that can:
1. Run from removable media (USB/SD) without installation
2. Install itself onto bare metal
3. Run as a sibling layer on existing operating systems
4. Discover and federate with other LiveSpore instances

**Pure Rust. JSON-RPC & tarpc. Capability-based. Self-sovereign.**

---

## 🧬 What is a LiveSpore?

A LiveSpore is a **self-extracting, self-aware primal ecosystem** that:

```
LiveSpore Package (USB/Archive)
├── Primal Binaries (all harvested primals)
│   ├── beardog (encryption & tunneling)
│   ├── songbird-orchestrator (service registry)
│   ├── toadstool (distributed compute)
│   ├── nestgate (federated storage)
│   ├── squirrel (AI orchestration)
│   └── petal-tongue (UI)
│
├── Bootstrap Runtime (pure Rust)
│   ├── spore-detector (hardware & environment detection)
│   ├── spore-deployer (atomic deployment)
│   ├── spore-installer (optional bare metal install)
│   └── spore-bridge (cross-OS communication)
│
├── Neural Graphs (deployment orchestration)
│   ├── tower.toml
│   ├── node.toml
│   ├── nest.toml
│   └── nucleus.toml
│
└── Persistence Layer (optional)
    ├── plasmidBin/ (primal binaries)
    ├── graphs/ (neural API graphs)
    └── data/ (optional persistent storage)
```

---

## 🔬 Three Deployment Modes

### 1. **COLD SPORE** (Live/Portable Mode)

**Definition**: Runs entirely from removable media (USB/SD card) without touching host system.

**Characteristics**:
- ✅ No installation required
- ✅ Host system unmodified
- ✅ Self-contained execution
- ✅ Optional persistence on USB
- ✅ Can run on ANY OS with USB support

**Use Cases**:
- Demonstrations and showcases
- Secure, ephemeral environments
- Testing on new hardware
- Portable workspaces

**Technical Approach**:
```rust
// spore-detector identifies Cold Spore mode
pub enum DeploymentMode {
    ColdSpore {
        media_path: PathBuf,        // e.g., /media/usb0
        persistence: bool,           // Use USB for storage?
        host_os: HostOS,            // Linux/Mac/Windows/Unknown
    },
    // ... other modes
}
```

**Socket Strategy**:
- Use USB-based socket paths: `/media/usb0/runtime/beardog-nat0.sock`
- Fallback to `/tmp/biomeos-cold-*/` if USB not writable
- All primals use `PRIMAL_SOCKET` override to point to USB

**Persistence**:
- Optional: Use USB for NestGate storage
- Ephemeral: All data in RAM, cleared on shutdown

---

### 2. **LIVE SPORE** (Bare Metal Install)

**Definition**: Full installation onto physical hardware, replacing or dual-booting with existing OS.

**Characteristics**:
- ✅ Full hardware access
- ✅ Maximum performance
- ✅ Persistent storage on disk
- ✅ Boot without external media
- ✅ Traditional "installed OS" experience

**Use Cases**:
- Production deployments
- Dedicated biomeOS nodes
- High-performance compute nodes
- Long-term installations

**Technical Approach**:
```rust
pub enum DeploymentMode {
    LiveSpore {
        root_partition: PathBuf,    // e.g., /dev/nvme0n1p2
        boot_partition: PathBuf,    // e.g., /dev/nvme0n1p1
        installed_version: String,  // biomeOS version
    },
    // ... other modes
}
```

**Socket Strategy**:
- Standard XDG paths: `/run/user/<uid>/beardog-nat0.sock`
- System-wide: `/var/run/biomeos/`
- Full systemd integration (optional)

**Installation Process**:
1. Boot from LiveSpore USB (Cold Spore mode)
2. Run `spore-installer` binary
3. Partition disk, extract primals
4. Configure bootloader (optional, host OS can boot biomeOS)
5. Reboot into Live Spore

---

### 3. **SIBLING SPORE** (On-Top-of-OS)

**Definition**: Runs as a layer on top of existing OS (Linux/Mac/Windows), coexisting peacefully.

**Characteristics**:
- ✅ No installation, no repartitioning
- ✅ Runs alongside existing OS
- ✅ Can discover bare metal biomeOS nodes
- ✅ Uses host OS resources
- ✅ Sandboxed execution

**Use Cases**:
- Development on existing machines
- Gradual migration to biomeOS
- Multi-OS environments
- Testing and experimentation

**Technical Approach**:
```rust
pub enum DeploymentMode {
    SiblingSpore {
        host_os: HostOS,            // Linux/Mac/Windows
        install_dir: PathBuf,       // e.g., ~/biomeOS/
        isolation: IsolationLevel,  // Sandboxed/Shared
    },
    // ... other modes
}

pub enum HostOS {
    Linux { distro: String },
    MacOS { version: String },
    Windows { version: String },
    BSD { variant: String },
    Unknown,
}
```

**Socket Strategy**:
- User-space paths: `~/.local/share/biomeos/runtime/beardog-nat0.sock`
- Fallback to `/tmp/biomeos-sibling-<uid>/`
- Cross-mode discovery via JSON-RPC over localhost

**Cross-OS Communication**:
- JSON-RPC over TCP (localhost only)
- tarpc for high-performance local IPC
- HTTP fallback for legacy systems

---

## 🧪 Pure Rust Implementation Strategy

### ✅ Easy in Pure Rust (Existing Crates)

| Component | Pure Rust Crate | Status |
|-----------|-----------------|--------|
| **Filesystem Detection** | `sysinfo`, `sys-mount` | ✅ Production-ready |
| **Partitioning** | `gptman` | ✅ Production-ready |
| **Archive Extraction** | `tar`, `flate2` | ✅ Production-ready |
| **File Compression** | `zstd`, `lz4`, `bzip2` | ✅ Production-ready |
| **JSON-RPC Client/Server** | `jsonrpsee` (we'll evolve) | ✅ Already using |
| **tarpc** | `tarpc` | ✅ Already using |
| **Hardware Detection** | `sysinfo`, `udev` bindings | ✅ Production-ready |
| **Process Management** | `tokio::process` | ✅ Already using |
| **Socket Management** | `tokio::net` | ✅ Already using |

---

### 🔧 Challenging but Solvable (Full Evolution Required)

| Component | Challenge | biomeOS Evolution Strategy |
|-----------|-----------|----------------------------|
| **Bootloader** | Traditional bootloaders (GRUB/systemd-boot) are C | **Evolution**: LiveSpore doesn't need a traditional bootloader! Host OS boots biomeOS as a "program". For bare metal, we use Rust `bootloader` crate or leverage existing UEFI. |
| **Initramfs** | Traditional initramfs is bash scripts + C tools | **Evolution**: `spore-detector` is our initramfs! Pure Rust binary that detects hardware and launches NUCLEUS. |
| **SquashFS Creation** | `mksquashfs` is C-based | **Evolution**: We don't need SquashFS! Use tarball + zstd compression (pure Rust). Optionally, `squashfs-tools` Rust bindings if needed. |
| **ISO Creation** | `xorriso` is C-based | **Evolution**: We don't need ISOs! LiveSpore is a directory tree + bootstrap binary. For legacy boot, use `fatfs` crate to create FAT32 bootable USB. |
| **OverlayFS** | Kernel module (C) | **Evolution**: We don't need OverlayFS! Use NestGate for persistent storage overlay (pure Rust, federated). |
| **mDNS Discovery** | Traditional `avahi` is C | **Evolution**: Use `mdns` or `libmdns` Rust crates. JSON-RPC over mDNS for service discovery. |

---

### 🚀 biomeOS-Native Solutions (New Implementations)

#### 1. **spore-detector** (Deployment Mode Detection)

**Purpose**: Detect deployment mode and coordinate hardware detection with primals.

**Implementation**:
```rust
// crates/biomeos-spore/src/detector.rs

pub struct SporeDetector {
    system_info: SystemInfo,
    deployment_mode: DeploymentMode,
    toadstool: Option<ToadStoolClient>,  // Hardware detection by ToadStool!
}

impl SporeDetector {
    pub async fn detect() -> Result<Self> {
        // 1. Detect host OS (biomeOS owns this - minimal OS detection)
        let host_os = Self::detect_host_os()?;
        
        // 2. Detect execution context (biomeOS owns this - deployment mode)
        let deployment_mode = Self::detect_deployment_mode(&host_os)?;
        
        // 3. Try to connect to ToadStool for hardware scanning
        let toadstool = ToadStoolClient::discover_by_capability("hardware.detect").await.ok();
        
        // 4. If ToadStool available, delegate hardware detection
        let system_info = if let Some(ref toadstool) = toadstool {
            toadstool.get_system_info().await?
        } else {
            // Fallback: minimal detection (only for bootstrap)
            Self::minimal_system_info()?
        };
        
        Ok(Self {
            system_info,
            deployment_mode,
            toadstool,
        })
    }
    
    fn detect_deployment_mode(host_os: &HostOS) -> Result<DeploymentMode> {
        // Check if running from USB (biomeOS logic)
        if Self::is_running_from_removable_media()? {
            return Ok(DeploymentMode::ColdSpore { /* ... */ });
        }
        
        // Check if installed to root filesystem (biomeOS logic)
        if Self::is_installed_to_root()? {
            return Ok(DeploymentMode::LiveSpore { /* ... */ });
        }
        
        // Otherwise, running on top of existing OS
        Ok(DeploymentMode::SiblingSpore { /* ... */ })
    }
    
    fn minimal_system_info() -> Result<SystemInfo> {
        // Only detect what's needed to bootstrap primals
        // Full hardware detection is ToadStool's job
        Ok(SystemInfo {
            host_os: Self::detect_host_os()?,
            is_removable_media: Self::is_running_from_removable_media()?,
        })
    }
}
```

**Capability Delegation**:
- **ToadStool**: `hardware.detect` - Full hardware scanning (CPU, GPU, disks, network)
- **biomeOS**: Deployment mode detection, OS detection, bootstrap only

**Dependencies** (minimal):
- `sys-mount` (mount detection for deployment mode)
- `tokio` (async runtime)
- `ToadStoolClient` (optional, for hardware detection)

---

#### 2. **spore-deployer** (Atomic Deployment)

**Purpose**: Replace bash deployment scripts with pure Rust orchestration.

**Implementation**:
```rust
// crates/biomeos-spore/src/deployer.rs

pub struct SporeDeployer {
    detector: SporeDetector,
    graph_executor: GraphExecutor,
}

impl SporeDeployer {
    pub async fn deploy_nucleus(&self) -> Result<NucleusHandle> {
        // 1. Load NUCLEUS graph (already have this!)
        let graph = PrimalGraph::from_file("graphs/nucleus.toml")?;
        
        // 2. Adapt socket paths for deployment mode
        let adapted_graph = self.adapt_graph_for_mode(&graph)?;
        
        // 3. Execute deployment using Neural API
        let handle = self.graph_executor.execute(adapted_graph).await?;
        
        // 4. Verify all atomics are healthy
        self.verify_deployment(&handle).await?;
        
        Ok(handle)
    }
    
    fn adapt_graph_for_mode(&self, graph: &PrimalGraph) -> Result<PrimalGraph> {
        match &self.detector.deployment_mode {
            DeploymentMode::ColdSpore { media_path, .. } => {
                // Override all socket paths to USB
                graph.with_socket_prefix(media_path.join("runtime"))
            }
            DeploymentMode::LiveSpore { .. } => {
                // Use standard XDG paths
                graph.with_socket_prefix("/run/user")
            }
            DeploymentMode::SiblingSpore { install_dir, .. } => {
                // Use user-space paths
                graph.with_socket_prefix(install_dir.join("runtime"))
            }
        }
    }
}
```

**Key Feature**: Reuses existing `biomeos-graph` crate! No new orchestration logic needed.

---

#### 3. **spore-installer** (Bare Metal Installation Orchestrator)

**Purpose**: Coordinate installation by delegating to primals.

**Implementation**:
```rust
// crates/biomeos-spore/src/installer.rs

pub struct SporeInstaller {
    detector: SporeDetector,
    petaltongue: PetalTongueClient,  // TUI handled by petalTongue!
    toadstool: ToadStoolClient,      // Hardware/compute by ToadStool!
    nestgate: NestGateClient,        // Storage by NestGate!
}

impl SporeInstaller {
    pub async fn run_interactive_install(&mut self) -> Result<()> {
        // 1. Request petalTongue to show installer UI
        let ui_session = self.petaltongue.start_installer_ui().await?;
        
        // 2. Get hardware info from ToadStool
        let available_disks = self.toadstool.list_block_devices().await?;
        
        // 3. User selects disk via petalTongue UI
        let selection = ui_session.wait_for_disk_selection(available_disks).await?;
        
        // 4. Ask Squirrel for optimal partitioning strategy (optional)
        let strategy = self.suggest_partition_strategy(&selection).await?;
        
        // 5. Confirmation via petalTongue
        ui_session.confirm_install(&selection, &strategy).await?;
        
        // 6. Execute installation (biomeOS coordinates)
        self.execute_install(&selection, &strategy).await?;
        
        // 7. Success via petalTongue
        ui_session.show_success().await?;
        
        Ok(())
    }
    
    async fn execute_install(&self, disk: &DiskSelection, strategy: &PartitionStrategy) -> Result<()> {
        // 1. Ask NestGate to prepare storage
        let partitions = self.nestgate.prepare_storage(disk, strategy).await?;
        
        // 2. Copy primal binaries (biomeOS owns this - it's the orchestrator)
        self.copy_primals(&partitions.root).await?;
        
        // 3. Generate deployment graphs (biomeOS owns this)
        self.generate_graphs(&partitions.root)?;
        
        // 4. Ask NestGate to finalize storage (bootloader, etc.)
        self.nestgate.finalize_install(&partitions).await?;
        
        Ok(())
    }
}
```

**Capability Delegation**:
- **petalTongue**: `installer.ui` - All TUI rendering
- **ToadStool**: `hardware.detect` - Block device enumeration
- **NestGate**: `storage.prepare` - Partitioning, formatting, bootloader
- **Squirrel**: `installer.suggest` - AI-assisted partitioning decisions
- **biomeOS**: Orchestration only

**Dependencies** (all JSON-RPC/tarpc clients):
- `PetalTongueClient` (UI coordination)
- `ToadStoolClient` (hardware detection)
- `NestGateClient` (storage management)
- `SquirrelClient` (AI suggestions)

---

---

#### 4. **spore-bridge** (Cross-Mode Discovery)

**Purpose**: Enable LiveSpore instances in different modes to discover each other.

**Implementation**:
```rust
// crates/biomeos-spore/src/bridge.rs

pub struct SporeBridge {
    local_mode: DeploymentMode,
    discovery_server: DiscoveryServer,
    federation_client: FederationClient,
}

impl SporeBridge {
    pub async fn start(&mut self) -> Result<()> {
        // 1. Start JSON-RPC discovery server
        self.discovery_server.start().await?;
        
        // 2. Announce presence via mDNS (if local network)
        self.announce_via_mdns().await?;
        
        // 3. Discover other LiveSpore instances
        let peers = self.discover_peers().await?;
        
        // 4. Federate with discovered peers
        for peer in peers {
            self.federate_with(peer).await?;
        }
        
        Ok(())
    }
    
    async fn announce_via_mdns(&self) -> Result<()> {
        use mdns_sd::{ServiceDaemon, ServiceInfo};
        
        let daemon = ServiceDaemon::new()?;
        
        let service = ServiceInfo::new(
            "_biomeos._tcp.local.",
            "biomeOS-LiveSpore",
            "local.",
            "", // IP auto-detected
            self.discovery_server.port,
            &[
                ("mode", self.local_mode.as_str()),
                ("version", env!("CARGO_PKG_VERSION")),
                ("protocol", "jsonrpc"),
            ],
        )?;
        
        daemon.register(service)?;
        Ok(())
    }
    
    async fn discover_peers(&self) -> Result<Vec<PeerInfo>> {
        use mdns_sd::ServiceDaemon;
        
        let daemon = ServiceDaemon::new()?;
        let receiver = daemon.browse("_biomeos._tcp.local.")?;
        
        let mut peers = Vec::new();
        
        // Collect peers for 3 seconds
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(3)) => {}
            event = receiver.recv() => {
                if let Ok(event) = event {
                    peers.push(PeerInfo::from_mdns_event(event));
                }
            }
        }
        
        Ok(peers)
    }
    
    async fn federate_with(&mut self, peer: PeerInfo) -> Result<()> {
        // Connect via JSON-RPC
        let client = JsonRpcClient::connect(&peer.endpoint).await?;
        
        // Exchange capabilities
        let their_caps = client.request("get_capabilities", ()).await?;
        let our_caps = self.get_local_capabilities();
        
        // Register with Songbird (if available)
        if let Some(songbird) = self.find_songbird().await? {
            songbird.register_peer(&peer, &their_caps).await?;
        }
        
        Ok(())
    }
}
```

**Discovery Stack**:
1. **mDNS** (local network): `mdns-sd` crate (pure Rust)
2. **JSON-RPC** (inter-primal): `jsonrpsee` (evolving to pure)
3. **tarpc** (high-performance IPC): `tarpc` crate
4. **HTTP** (fallback only): `hyper` crate

---

## 📦 LiveSpore Package Format

**NOT an ISO. NOT a SquashFS. Pure biomeOS.**

### Format: Compressed Tarball

```
livespore-v1.0.0.tar.zst
├── bin/
│   ├── beardog
│   ├── songbird-orchestrator
│   ├── toadstool
│   ├── nestgate
│   ├── squirrel
│   ├── petal-tongue
│   ├── spore-detector      (NEW)
│   ├── spore-deployer      (NEW)
│   ├── spore-installer     (NEW)
│   └── spore-bridge        (NEW)
│
├── graphs/
│   ├── tower.toml
│   ├── node.toml
│   ├── nest.toml
│   └── nucleus.toml
│
├── bootstrap.sh            (Minimal shell script for host OS)
└── README.md
```

**bootstrap.sh** (Minimal, delegates to Rust):
```bash
#!/bin/bash
# LiveSpore Bootstrap (delegates to pure Rust)
set -euo pipefail

SPORE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export LIVESPORE_ROOT="$SPORE_DIR"

# Detect architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *) echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

# Launch spore-detector (pure Rust takes over)
exec "$SPORE_DIR/bin/spore-detector"
```

**Creation** (pure Rust):
```rust
// crates/biomeos-spore/src/packager.rs

pub struct SporePackager;

impl SporePackager {
    pub async fn create_livespore(output_path: &Path) -> Result<()> {
        use async_tar::Builder;
        use async_compression::tokio::write::ZstdEncoder;
        
        // 1. Create tarball builder
        let file = File::create(output_path).await?;
        let encoder = ZstdEncoder::new(file);
        let mut tar = Builder::new(encoder);
        
        // 2. Add primal binaries
        tar.append_dir("bin", ".").await?;
        for primal in PRIMALS {
            let bin_path = format!("plasmidBin/{primal}");
            tar.append_path_with_name(&bin_path, format!("bin/{primal}")).await?;
        }
        
        // 3. Add graphs
        tar.append_dir_all("graphs", "graphs/").await?;
        
        // 4. Add bootstrap script
        tar.append_path("bootstrap.sh").await?;
        
        // 5. Finalize
        tar.finish().await?;
        
        Ok(())
    }
}
```

**Extraction** (pure Rust):
```rust
// In spore-detector
pub async fn extract_livespore(archive: &Path, target: &Path) -> Result<()> {
    use async_tar::Archive;
    use async_compression::tokio::bufread::ZstdDecoder;
    
    let file = File::open(archive).await?;
    let decoder = ZstdDecoder::new(BufReader::new(file));
    let mut archive = Archive::new(decoder);
    
    archive.unpack(target).await?;
    
    Ok(())
}
```

**Dependencies** (all pure Rust):
- `async-tar` (tarball creation/extraction)
- `async-compression` (zstd compression)
- `tokio::fs` (async file I/O)

---

## 🌐 Cross-Mode Communication Architecture

### Scenario: 3 LiveSpore Instances Discovering Each Other

```
┌─────────────────────────────────────────────────────────────┐
│  Mac (Sibling Spore)                                        │
│  ~/biomeOS/runtime/                                         │
│  - beardog-nat0.sock                                        │
│  - songbird-nat0.sock                                       │
│                                                             │
│  Announces via mDNS: _biomeos._tcp.local.                  │
│  Protocol: JSON-RPC over TCP (localhost + LAN)             │
└─────────────────────────────────────────────────────────────┘
                        ↕ (mDNS discovery)
┌─────────────────────────────────────────────────────────────┐
│  Linux Server (Live Spore - bare metal install)             │
│  /run/user/1000/                                            │
│  - beardog-nat0.sock                                        │
│  - songbird-nat0.sock                                       │
│  - toadstool-nat0.sock                                      │
│  - nestgate-nat0.sock                                       │
│                                                             │
│  Announces via mDNS + Songbird federation                  │
│  Protocol: JSON-RPC + tarpc (Unix sockets + TCP)           │
└─────────────────────────────────────────────────────────────┘
                        ↕ (BearDog tunnels)
┌─────────────────────────────────────────────────────────────┐
│  Windows Laptop (Sibling Spore)                             │
│  C:\Users\user\.local\share\biomeos\runtime\               │
│  - beardog-nat0.sock (via WSL or named pipes)              │
│  - songbird-nat0.sock                                       │
│                                                             │
│  Discovers via mDNS (if on LAN)                            │
│  Protocol: JSON-RPC over TCP (HTTP fallback)               │
└─────────────────────────────────────────────────────────────┘
```

### Communication Protocol Stack

**Layer 1: Discovery**
- **Local**: Unix domain sockets (same machine)
- **LAN**: mDNS + JSON-RPC over TCP
- **WAN**: Songbird federation + BearDog tunnels

**Layer 2: IPC**
- **Primary**: tarpc (high-performance, type-safe)
- **Secondary**: JSON-RPC (language-agnostic, flexible)
- **Fallback**: HTTP/REST (legacy compatibility)

**Layer 3: Security**
- **All communication** encrypted via BearDog
- **Genetic lineage** for trust verification
- **Capability-based** authorization

**Implementation**:
```rust
// crates/biomeos-spore/src/comms.rs

pub struct CrossModeComms {
    local: UnixSocketTransport,    // tarpc + JSON-RPC
    lan: TcpTransport,              // JSON-RPC + mDNS
    wan: TunnelTransport,           // BearDog tunnels
}

impl CrossModeComms {
    pub async fn connect_to_peer(&self, peer: &PeerInfo) -> Result<PeerConnection> {
        match peer.location {
            PeerLocation::SameMachine => {
                // Use Unix sockets (fastest)
                self.local.connect(&peer.socket_path).await
            }
            PeerLocation::LocalNetwork => {
                // Use JSON-RPC over TCP (discovered via mDNS)
                self.lan.connect(&peer.tcp_endpoint).await
            }
            PeerLocation::Remote => {
                // Use BearDog tunnel (encrypted, federated)
                self.wan.connect_via_beardog(&peer.family_id).await
            }
        }
    }
}
```

---

## 📊 Implementation Phases

### **Phase 1: Runtime Adaptation (2 weeks)**

**Goal**: Make biomeOS deployment-mode aware.

**Deliverables**:
1. ✅ `DeploymentMode` enum and detection logic
2. ✅ Adaptive socket path configuration
3. ✅ Environment variable overrides for all primals
4. ✅ Tests for all 3 modes

**Implementation**:
```rust
// In existing crates/biomeos-core/src/deployment.rs

pub enum DeploymentMode {
    ColdSpore { /* ... */ },
    LiveSpore { /* ... */ },
    SiblingSpore { /* ... */ },
}

impl DeploymentMode {
    pub fn detect() -> Result<Self> {
        // Detection logic here
    }
    
    pub fn socket_prefix(&self) -> PathBuf {
        match self {
            Self::ColdSpore { media_path, .. } => media_path.join("runtime"),
            Self::LiveSpore { .. } => PathBuf::from("/run/user").join(uid().to_string()),
            Self::SiblingSpore { install_dir, .. } => install_dir.join("runtime"),
        }
    }
}
```

**Testing**:
- Unit tests for mode detection
- E2E tests for socket path adaptation
- Integration tests with live primals

---

### **Phase 2: Spore Tooling (3 weeks)**

**Goal**: Create LiveSpore packaging and deployment tools.

**Deliverables**:
1. ✅ `spore-detector` binary (hardware detection)
2. ✅ `spore-deployer` binary (atomic deployment)
3. ✅ `spore-packager` binary (create LiveSpore packages)
4. ✅ Tests for packaging/extraction

**New Crate**:
```
crates/biomeos-spore/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── detector.rs       (hardware detection)
│   ├── deployer.rs       (atomic deployment)
│   ├── packager.rs       (package creation)
│   ├── extractor.rs      (package extraction)
│   └── comms.rs          (cross-mode communication)
├── bin/
│   ├── spore-detector.rs
│   ├── spore-deployer.rs
│   └── spore-packager.rs
└── tests/
    ├── detector_tests.rs
    ├── deployer_tests.rs
    └── packager_tests.rs
```

---

### **Phase 3: Cross-Mode Discovery (2 weeks)**

**Goal**: Enable LiveSpore instances to discover each other.

**Deliverables**:
1. ✅ `spore-bridge` binary (mDNS discovery)
2. ✅ JSON-RPC over TCP for LAN communication
3. ✅ BearDog tunnel integration for WAN
4. ✅ Songbird federation support

**Implementation**:
```rust
// crates/biomeos-spore/src/bridge.rs (see above for full impl)
```

**Dependencies** (all pure Rust):
- `mdns-sd` (mDNS service discovery)
- `jsonrpsee` (JSON-RPC client/server)
- `tarpc` (high-performance RPC)
- `tokio` (async runtime)

---

### **Phase 4: Installer (2 weeks)**

**Goal**: Create interactive installer for bare metal installations.

**Deliverables**:
1. ✅ `spore-installer` binary (TUI installer)
2. ✅ Disk partitioning (GPT)
3. ✅ Filesystem creation (ext4/btrfs)
4. ✅ Bootloader integration (optional)

**TUI Design** (using `ratatui`):
```
┌─────────────────────────────────────────────────────────────┐
│ 🌱 LiveSpore Installer v1.0.0                               │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Welcome to the biomeOS LiveSpore Installer!               │
│                                                             │
│  This will install biomeOS to physical hardware.           │
│                                                             │
│  Available Disks:                                          │
│  ┌───────────────────────────────────────────────────────┐ │
│  │ ● /dev/nvme0n1  (500 GB NVMe SSD)                     │ │
│  │   /dev/sda      (1 TB HDD)                            │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                             │
│  [Next]  [Cancel]                                          │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

### **Phase 5: Integration & Testing (3 weeks)**

**Goal**: End-to-end testing of all 3 deployment modes.

**Test Matrix**:
```
┌─────────────┬──────────┬──────────┬──────────────┐
│ Test        │ Cold     │ Live     │ Sibling      │
├─────────────┼──────────┼──────────┼──────────────┤
│ Boot        │ ✅ USB   │ ✅ Disk  │ ✅ Launch    │
│ Deploy      │ ✅       │ ✅       │ ✅           │
│ Discover    │ ✅       │ ✅       │ ✅           │
│ Federate    │ ✅       │ ✅       │ ✅           │
│ UI          │ ✅       │ ✅       │ ✅           │
│ Persist     │ ✅ USB   │ ✅ Disk  │ ✅ User home │
│ Cross-mode  │ ✅       │ ✅       │ ✅           │
└─────────────┴──────────┴──────────┴──────────────┘
```

**Deliverables**:
1. ✅ E2E tests for each mode
2. ✅ Cross-mode federation tests
3. ✅ Performance benchmarks
4. ✅ Documentation

---

## 🔍 Deep Debt Compliance

### ✅ Modern Idiomatic Rust
- All new code uses `async`/`await`
- Strong typing, no string soup
- `anyhow::Result` for errors
- Zero `unwrap()` in production

### ✅ Zero Hardcoding
- All paths derived from `DeploymentMode`
- Environment variable overrides
- Runtime detection, not compile-time

### ✅ Capability-Based Discovery
- mDNS announces capabilities
- Songbird registers discovered primals
- No hardcoded endpoints

### ✅ Mock Isolation
- No mocks in production
- All detection is real hardware
- Tests use dependency injection

### ✅ Smart Refactoring
- Reuses `biomeos-graph` for orchestration
- Extends existing socket configuration
- No duplication of logic

---

## 📈 Success Metrics

### Technical Metrics
- ✅ 100% pure Rust (no C dependencies for core functionality)
- ✅ < 50 MB LiveSpore package size (compressed)
- ✅ < 5 seconds boot time (Cold Spore to NUCLEUS deployed)
- ✅ < 1 second mode detection time
- ✅ 100% test coverage (unit + E2E + chaos)

### User Metrics
- ✅ Run biomeOS from USB without installation
- ✅ Install to bare metal in < 5 minutes
- ✅ Run on Mac/Linux/Windows without conflicts
- ✅ Auto-discover other biomeOS nodes
- ✅ Seamless cross-mode federation

---

## 🎯 Total Timeline

**12 weeks for complete LiveSpore system**

```
Weeks 1-2:   Runtime Adaptation
Weeks 3-5:   Spore Tooling
Weeks 6-7:   Cross-Mode Discovery
Weeks 8-9:   Installer
Weeks 10-12: Integration & Testing
```

---

## 🔗 Related Specifications

- `ATOMIC_DEPLOYMENT_SYSTEM_SPEC.md` - Atomic deployment (reused!)
- `NUCLEUS_ORCHESTRATION_SPEC.md` - NUCLEUS graphs (reused!)
- `PRIMAL_SOCKET_CONFIGURATION_SPEC.md` - Socket standards (extended!)
- `NEURAL_API_SPEC.md` - Graph execution (reused!)

---

## 🎉 Summary

**LiveSpore is NOT a traditional live USB.**

LiveSpore is a **self-bootstrapping, deployment-mode-aware NUCLEUS** that can:
- ✅ Run from USB (Cold Spore)
- ✅ Install to bare metal (Live Spore)
- ✅ Run on top of existing OS (Sibling Spore)
- ✅ Discover and federate across all modes
- ✅ 100% pure Rust
- ✅ JSON-RPC + tarpc (no gRPC)
- ✅ Capability-based discovery
- ✅ Zero hardcoding

**Different orders of the same architecture.** 🍄🐸

---

**Status**: 🔬 Ready for Phase 1 Implementation  
**Next Step**: Implement `DeploymentMode` detection  
**Timeline**: 12 weeks to complete system

---

*biomeOS: A pure Rust, self-sovereign, federated operating system.*

