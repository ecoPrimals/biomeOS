# genomeBin Bare-Metal UEFI Boot Specification
**Version**: 1.0.0  
**Status**: Design Phase  
**Type**: Standalone OS Deployment via UEFI  
**Date**: January 31, 2026

---

## Overview

This specification extends genomeBin v3.0 to support **bare-metal deployment** as a **standalone operating system** via UEFI boot. This evolution transforms genomeBin from an application deployment format into a **complete bootable OS image** that can run directly on hardware, making biomeOS a true **orchestrating OS** for the ecoPrimals ecosystem.

---

## Vision

**Goal**: Single genomeBin file that can:
1. **Deploy as application** (current: Linux, Android, Windows, macOS)
2. **Boot as standalone OS** (new: UEFI bare-metal)
3. **Orchestrate primals as OS services** (biomeOS as kernel-level orchestrator)

**Use Case**: Flash `biomeos-os.genome` to USB → Boot on any UEFI system → Complete primal ecosystem runs as native OS.

---

## Key Features

### 1. **UEFI Bootable Format**
- EFI System Partition (ESP) structure
- UEFI boot stub embedded in genomeBin
- Multi-architecture UEFI support (x64, AA64, ARM, RISC-V)
- Secure Boot compatible (signed EFI binaries)

### 2. **Minimal OS Kernel**
- Linux kernel (embedded, minimal config)
- initramfs with biomeOS runtime
- systemd or custom init system
- Essential drivers (network, storage, graphics)

### 3. **biomeOS as OS Orchestrator**
- Primals run as native OS services
- systemd units for each primal
- Genetic trust framework at OS level
- neuralAPI as system bus

### 4. **Hardware Abstraction**
- GPU detection and initialization
- Network interface management
- Storage management (NestGate)
- Hardware discovery at boot

### 5. **Deployment Modes**
- **Live USB**: Boot from USB, run in RAM
- **Install Mode**: Install to disk (GPT partition)
- **Dual Boot**: Coexist with other OS
- **Standalone**: Complete OS replacement

---

## Architecture

### genomeBin OS Structure

```
biomeos-os.genome (bootable + executable)
├── UEFI Boot Stub (~1 MB)
│   ├── bootx64.efi (x86_64 UEFI)
│   ├── bootaa64.efi (ARM64 UEFI)
│   └── grub.cfg (boot configuration)
├── Linux Kernel (~10 MB compressed)
│   ├── vmlinuz-x86_64
│   ├── vmlinuz-aarch64
│   └── kernel modules (essential only)
├── initramfs (~50 MB compressed)
│   ├── biomeOS runtime
│   ├── Primal binaries (all 4)
│   ├── Essential libraries
│   └── Boot scripts
├── Root Filesystem (squashfs, ~200 MB)
│   ├── /usr/bin/
│   │   ├── beardog, songbird, toadstool, nestgate
│   │   └── biomeos-api, biomeos-cli
│   ├── /etc/systemd/system/
│   │   ├── beardog.service
│   │   ├── songbird.service
│   │   ├── toadstool.service
│   │   └── nestgate.service
│   ├── /opt/biomeos/
│   │   ├── graphs/
│   │   └── configs/
│   └── /lib/
│       └── firmware/ (GPU, network drivers)
└── Genome Metadata
    ├── manifest.json
    └── checksums.sha256
```

**Total Size**: ~250-300 MB (compressed, bootable USB image)

---

## UEFI Boot Process

### 1. **UEFI Firmware Phase**
```
Power On → UEFI Firmware
  ↓
Detect EFI System Partition (ESP)
  ↓
Load bootx64.efi (or bootaa64.efi)
  ↓
UEFI boot stub initializes
```

### 2. **Boot Stub Phase**
```
Boot stub (Rust UEFI)
  ↓
Detect hardware (CPU, RAM, GPU, storage)
  ↓
Load Linux kernel (vmlinuz)
  ↓
Load initramfs (biomeOS runtime)
  ↓
Transfer control to kernel
```

### 3. **Kernel Boot Phase**
```
Linux kernel starts
  ↓
Mount initramfs as root
  ↓
Run biomeOS init script
  ↓
Initialize hardware drivers
  ↓
Mount squashfs root filesystem
  ↓
Pivot to real root
```

### 4. **biomeOS OS Phase**
```
biomeOS init system
  ↓
Start systemd (or custom init)
  ↓
Launch primal services:
  1. BearDog (authentication, HSM)
  2. Songbird (discovery, P2P)
  3. Toadstool (GPU compute, optional)
  4. NestGate (storage, optional)
  ↓
neuralAPI starts
  ↓
System ready (login or GUI)
```

---

## Implementation Components

### 1. **UEFI Boot Stub (Rust)**

**Crate**: `biomeos-uefi-boot`

```rust
// biomeos-uefi-boot/src/lib.rs
#![no_std]
#![no_main]

use uefi::prelude::*;
use uefi::proto::console::text::Output;
use uefi::proto::media::file::*;
use uefi::proto::loaded_image::LoadedImage;

#[entry]
fn efi_main(image: Handle, mut system_table: SystemTable<Boot>) -> Status {
    // 1. Initialize UEFI services
    uefi_services::init(&mut system_table).unwrap();
    
    // 2. Detect hardware
    let hardware = detect_hardware(&system_table);
    
    // 3. Load kernel and initramfs
    let kernel = load_kernel(&system_table, hardware.arch)?;
    let initramfs = load_initramfs(&system_table)?;
    
    // 4. Set kernel command line
    let cmdline = format!(
        "root=/dev/ram0 init=/init console=tty0 biosys.arch={} biosys.mode=boot",
        hardware.arch
    );
    
    // 5. Boot kernel
    boot_linux_kernel(kernel, initramfs, &cmdline)?;
    
    // Never returns
    Status::SUCCESS
}

fn detect_hardware(st: &SystemTable<Boot>) -> Hardware {
    Hardware {
        arch: detect_arch(),
        ram_mb: detect_ram(st),
        gpu: detect_gpu(st),
        storage: detect_storage(st),
        network: detect_network(st),
    }
}
```

### 2. **Minimal Kernel Config**

**File**: `configs/kernel.config`

```bash
# Minimal Linux kernel for biomeOS
CONFIG_64BIT=y
CONFIG_SMP=y
CONFIG_EFI=y
CONFIG_EFI_STUB=y
CONFIG_X86_64=y (or ARM64)

# Essential drivers
CONFIG_BLK_DEV_LOOP=y
CONFIG_SQUASHFS=y
CONFIG_OVERLAY_FS=y
CONFIG_EXT4_FS=y
CONFIG_VFAT_FS=y

# Network
CONFIG_NETDEVICES=y
CONFIG_NET_CORE=y
CONFIG_ETHERNET=y
CONFIG_E1000=y
CONFIG_VIRTIO_NET=y

# GPU (essential only)
CONFIG_DRM=y
CONFIG_DRM_NOUVEAU=y (NVIDIA open)
CONFIG_DRM_AMDGPU=y
CONFIG_DRM_I915=y (Intel)

# USB
CONFIG_USB=y
CONFIG_USB_XHCI_HCD=y
CONFIG_USB_STORAGE=y

# Security
CONFIG_SECURITY=y
CONFIG_SECCOMP=y
CONFIG_HARDENED_USERCOPY=y

# Total: ~10 MB compressed
```

### 3. **initramfs Structure**

**File**: `initramfs/init` (Rust binary)

```rust
// initramfs-init/src/main.rs
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 1. Mount essential filesystems
    mount_proc();
    mount_sys();
    mount_dev();
    
    // 2. Detect boot device
    let boot_device = detect_boot_device();
    
    // 3. Mount squashfs root
    mount_squashfs(&boot_device, "/newroot");
    
    // 4. Pivot to real root
    pivot_root("/newroot", "/newroot/mnt");
    
    // 5. Execute biomeOS init
    exec("/sbin/biomeos-init");
    
    // Never returns
    loop {}
}
```

### 4. **biomeOS Init System**

**Crate**: `biomeos-init`

```rust
// biomeos-init/src/main.rs

use std::process::Command;
use anyhow::Result;

fn main() -> Result<()> {
    println!("🧬 biomeOS - Primal Operating System");
    println!("Initializing ecosystem...");
    
    // 1. Detect hardware
    let hardware = Hardware::detect()?;
    println!("✅ Hardware: {} CPU, {} GPU, {} GB RAM", 
        hardware.cpu_count, hardware.gpu_count, hardware.ram_gb);
    
    // 2. Initialize genetic framework
    let family_id = initialize_family()?;
    println!("✅ Family ID: {}", family_id);
    
    // 3. Start core primals
    start_primal("beardog")?;
    start_primal("songbird")?;
    
    // 4. Start optional primals (if hardware available)
    if hardware.has_gpu() {
        start_primal("toadstool")?;
    }
    if hardware.has_storage() {
        start_primal("nestgate")?;
    }
    
    // 5. Start neuralAPI
    start_neural_api()?;
    
    // 6. Display login or start GUI
    if let Some(display) = hardware.display {
        start_gui()?;
    } else {
        start_login_shell()?;
    }
    
    println!("✅ biomeOS ready!");
    
    // 7. Wait for signals
    wait_for_shutdown();
    
    Ok(())
}

fn start_primal(name: &str) -> Result<()> {
    println!("Starting {}...", name);
    Command::new(format!("/usr/bin/{}", name))
        .env("BIOMEOS_MODE", "os")
        .env("BIOMEOS_FAMILY_SEED_PATH", "/etc/biomeos/.family.seed")
        .spawn()?;
    Ok(())
}
```

### 5. **Systemd Service Units**

**File**: `rootfs/etc/systemd/system/beardog.service`

```ini
[Unit]
Description=BearDog - Genetic Trust & Authentication
After=network.target
Wants=network.target

[Service]
Type=notify
ExecStart=/usr/bin/beardog
Environment="BIOMEOS_MODE=os"
Environment="BEARDOG_FAMILY_SEED_PATH=/etc/biomeos/.family.seed"
Restart=always
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

**Similar units for**: songbird.service, toadstool.service, nestgate.service

---

## Deployment Modes

### Mode 1: Live USB (Pop!_OS Style)

**Use Case**: Boot from USB, test without installation

```bash
# Create bootable USB
biomeos genome create-usb biomeos-os.genome --device /dev/sdb

# Result: Bootable USB with:
# - EFI partition (bootx64.efi)
# - biomeOS partition (squashfs)
# - Persistent storage partition (optional)

# Boot process:
# 1. Insert USB → Boot
# 2. biomeOS runs entirely from RAM
# 3. Changes discarded on reboot (or saved to persistent partition)
```

### Mode 2: Install to Disk

**Use Case**: Permanent installation

```bash
# Boot from USB, then install
biomeos-installer --target /dev/nvme0n1 --mode install

# Partitioning:
# /dev/nvme0n1p1: EFI System Partition (512 MB)
# /dev/nvme0n1p2: biomeOS root (20 GB, ext4)
# /dev/nvme0n1p3: NestGate storage (remaining, ext4)

# Installation:
# 1. Create GPT partitions
# 2. Format filesystems
# 3. Extract genomeBin to /dev/nvme0n1p2
# 4. Install bootloader (GRUB or systemd-boot)
# 5. Generate family seed
# 6. Reboot
```

### Mode 3: Dual Boot

**Use Case**: Coexist with existing OS

```bash
# Detect existing OS
biomeos-installer --detect-os

# Found: Windows 11 on /dev/nvme0n1p1-p4
# Available space: 100 GB unallocated

# Install biomeOS to free space
biomeos-installer --target /dev/nvme0n1 --mode dual-boot --size 50GB

# GRUB configuration:
# - Windows Boot Manager
# - biomeOS
# - Advanced options
```

### Mode 4: VM/Cloud Image

**Use Case**: Deploy to VM or cloud

```bash
# Create VM image
biomeos genome create-vm biomeos-os.genome --format qcow2 --size 20GB

# Result: biomeos-os.qcow2 (QEMU/KVM)
# Or: biomeos-os.vhdx (Hyper-V)
# Or: biomeos-os.vmdk (VMware)

# Deploy to cloud
biomeos genome create-ami biomeos-os.genome --region us-east-1
# Result: AMI for AWS EC2
```

---

## Hardware Requirements

### Minimum

| Component | Requirement |
|-----------|-------------|
| **CPU** | x86_64 or ARM64, 2 cores, 1 GHz |
| **RAM** | 2 GB (4 GB recommended) |
| **Storage** | 10 GB (20 GB recommended) |
| **GPU** | None (Toadstool optional) |
| **Network** | Ethernet or WiFi |
| **Boot** | UEFI 2.0+ (Secure Boot optional) |

### Recommended

| Component | Requirement |
|-----------|-------------|
| **CPU** | x86_64 or ARM64, 4+ cores, 2+ GHz |
| **RAM** | 8 GB+ |
| **Storage** | 50 GB+ NVMe SSD |
| **GPU** | NVIDIA RTX 3000+, AMD RX 6000+, or Intel Arc |
| **Network** | Gigabit Ethernet |
| **Boot** | UEFI 2.7+ with Secure Boot |

### Optimal (Full NUCLEUS)

| Component | Requirement |
|-----------|-------------|
| **CPU** | x86_64, 8+ cores, 3+ GHz |
| **RAM** | 16 GB+ |
| **Storage** | 100 GB+ NVMe SSD |
| **GPU** | NVIDIA RTX 4090 or AMD RX 7900 XTX |
| **Network** | 10 Gigabit Ethernet |
| **Boot** | UEFI 2.8+ with Secure Boot + TPM 2.0 |

---

## Security Considerations

### 1. **Secure Boot**
- Sign UEFI boot stub with trusted key
- Kernel signature verification
- Chain of trust from firmware → kernel → init

### 2. **Measured Boot**
- TPM 2.0 integration
- PCR measurements at each stage
- Remote attestation support

### 3. **Genetic Trust**
- Family seed derived from hardware (TPM, CPU ID)
- BearDog HSM integration
- Lineage verification at boot

### 4. **Encrypted Storage**
- LUKS2 full disk encryption
- TPM-sealed decryption keys
- NestGate encrypted volumes

### 5. **Network Security**
- Firewall by default (nftables)
- Only Songbird P2P ports open
- BearDog authentication for all services

---

## Implementation Phases

### Phase A: UEFI Boot Stub (6-8 weeks)

**Goal**: Bootable UEFI stub that loads Linux kernel

**Deliverables**:
- ✅ `biomeos-uefi-boot` crate
- ✅ Hardware detection (CPU, RAM, GPU)
- ✅ Kernel loading (vmlinuz)
- ✅ initramfs loading
- ✅ Kernel command line setup
- ✅ Boot on QEMU/KVM
- ✅ Boot on real hardware (x86_64)

**Validation**:
- Boot on 3+ different hardware configs
- Verify all architectures (x64, AA64)

### Phase B: Minimal Kernel + initramfs (4-6 weeks)

**Goal**: Linux kernel boots to biomeOS init

**Deliverables**:
- ✅ Minimal kernel config (~10 MB)
- ✅ initramfs with Rust init (~50 MB)
- ✅ squashfs root filesystem
- ✅ Pivot root working
- ✅ biomeOS init starts

**Validation**:
- Boot to shell
- Mount filesystems
- Network working

### Phase C: biomeOS OS Orchestration (8-10 weeks)

**Goal**: Primals run as OS services

**Deliverables**:
- ✅ `biomeos-init` system
- ✅ Primal systemd units
- ✅ neuralAPI as system bus
- ✅ Hardware initialization
- ✅ Genetic trust at OS level

**Validation**:
- All 4 primals start
- Discovery working
- neuralAPI accessible
- Handshake validated

### Phase D: Installation & Deployment (4-6 weeks)

**Goal**: Full installation modes

**Deliverables**:
- ✅ USB creator tool
- ✅ Disk installer
- ✅ GRUB/systemd-boot integration
- ✅ Dual-boot support
- ✅ VM image generation

**Validation**:
- Install on 3+ machines
- Dual-boot with Windows/Linux
- VM deployment (QEMU, VirtualBox, VMware)

### Phase E: Hardware Support Expansion (Ongoing)

**Goal**: Support more hardware

**Deliverables**:
- ✅ Additional GPU drivers (AMD, Intel)
- ✅ WiFi support
- ✅ Bluetooth support
- ✅ ARM SBC support (Raspberry Pi, etc.)

**Validation**:
- Test on 10+ hardware configs
- Document compatibility matrix

---

## Testing Strategy

### 1. **QEMU/KVM Testing**
```bash
# Test boot in VM
qemu-system-x86_64 \
  -enable-kvm \
  -m 4G \
  -bios /usr/share/ovmf/OVMF.fd \
  -drive file=biomeos-os.genome,format=raw \
  -net nic -net user
```

### 2. **Hardware Testing**
- **Minimum**: Old laptop (2016+)
- **Recommended**: Modern desktop (2020+)
- **Optimal**: Workstation (2024+)

### 3. **Compatibility Testing**
- UEFI firmware versions (2.0 → 2.8)
- Secure Boot enabled/disabled
- Legacy CSM disabled
- Various GPU vendors (NVIDIA, AMD, Intel)

### 4. **Performance Testing**
- Boot time (target: <10 seconds to ready)
- Primal startup (target: <5 seconds)
- Memory usage (target: <2 GB idle)
- Network latency (target: <1ms local)

---

## Integration with genomeBin v3.0

### Unified Format

**Same genomeBin, multiple modes**:

```bash
# Mode 1: Application deployment (current)
./biomeos.genome --extract-to /opt/biomeos

# Mode 2: Boot as OS (new)
dd if=biomeos-os.genome of=/dev/sdb bs=1M status=progress

# Mode 3: Install to disk (new)
./biomeos-os.genome --install --target /dev/nvme0n1

# Mode 4: Create VM image (new)
./biomeos-os.genome --create-vm --format qcow2 --output biomeos.qcow2
```

### Architecture Extension

```rust
// genomeBin v3.0 extension
pub enum GenomeMode {
    Application,  // Current: Extract and run
    BootableUSB,  // New: Write to USB, boot
    InstallDisk,  // New: Install to disk
    VMImage,      // New: Create VM image
}

pub struct GenomeBin {
    manifest: GenomeManifest,
    binaries: HashMap<Arch, CompressedBinary>,
    embedded_genomes: Vec<GenomeBin>,
    
    // NEW: OS boot components
    boot_stub: Option<UEFIBootStub>,
    kernel: Option<LinuxKernel>,
    initramfs: Option<Initramfs>,
    rootfs: Option<SquashFS>,
}
```

---

## Validation Goals (from specs/VALIDATION_GOALS.md)

### H7: Bare-Metal UEFI Boot **NEW**

**Hypothesis**: biomeOS can function as a standalone OS, orchestrating primals as native services on UEFI hardware.

**Validation Criteria**:
- [ ] Boots on QEMU/KVM (x86_64)
- [ ] Boots on real hardware (x86_64, 3+ machines)
- [ ] Boots on ARM64 (Raspberry Pi 4/5)
- [ ] UEFI Secure Boot working
- [ ] All 4 primals start as services
- [ ] neuralAPI accessible at boot
- [ ] Hardware detection working (GPU, network, storage)
- [ ] Installation modes working (USB, disk, dual-boot)
- [ ] Performance meets targets (boot time, memory, latency)
- [ ] Independent replication by external party

**Current Status**: Design Phase (0% implemented)

**Timeline**: 22-30 weeks (Phases A-E)

**Priority**: Medium (validate core functionality first, then bare-metal)

---

## Success Criteria

### Functional
- [ ] Boots on UEFI hardware (x86_64, ARM64)
- [ ] All primals start automatically
- [ ] neuralAPI accessible
- [ ] Discovery working
- [ ] Storage persists (NestGate)
- [ ] GPU detected (Toadstool)

### Performance
- [ ] Boot time: <10 seconds to ready
- [ ] Primal startup: <5 seconds each
- [ ] Memory usage: <2 GB idle
- [ ] Network latency: <1ms local

### Compatibility
- [ ] Works on 10+ hardware configurations
- [ ] UEFI 2.0+ support
- [ ] Secure Boot compatible
- [ ] Dual-boot with Windows/Linux

### Security
- [ ] Signed UEFI boot stub
- [ ] TPM integration
- [ ] Encrypted storage (LUKS2)
- [ ] Genetic trust at OS level

---

## Documentation

### User Documentation
- [ ] Installation guide
- [ ] Hardware compatibility list
- [ ] Troubleshooting guide
- [ ] FAQ

### Developer Documentation
- [ ] Boot process architecture
- [ ] Kernel configuration guide
- [ ] Driver development guide
- [ ] OS customization guide

---

## Open Questions

1. **Init System**: systemd vs custom biomeOS init?
   - **Recommendation**: Start with systemd (battle-tested), migrate to custom later if needed

2. **Kernel**: Upstream Linux vs custom?
   - **Recommendation**: Upstream Linux (5.15 LTS or 6.1 LTS), minimal config

3. **Package Manager**: Use existing (apt, dnf) or custom?
   - **Recommendation**: Custom genomeBin-based (consistency)

4. **GUI**: Wayland compositor or headless?
   - **Recommendation**: Optional Wayland (Weston), default headless

5. **Persistence**: Full disk or overlay?
   - **Recommendation**: Full disk for install mode, overlay for live USB

---

## Related Specifications

- [`GENOMEBIN_V3_SPECIFICATION.md`](GENOMEBIN_V3_SPECIFICATION.md) - Core genomeBin v3.0 architecture
- [`BIOMEOS_GENOME_FACTORY_SPEC.md`](BIOMEOS_GENOME_FACTORY_SPEC.md) - Genome creation API
- [`VALIDATION_GOALS.md`](VALIDATION_GOALS.md) - Validation framework (H7 added)
- [`NUCLEUS_DEPLOYMENT_SPEC.md`](NUCLEUS_DEPLOYMENT_SPEC.md) - Atomic deployment patterns

---

## Status

**Phase**: Design Complete  
**Implementation**: Not Started (0%)  
**Priority**: Medium (after core validation)  
**Timeline**: 22-30 weeks  
**Impact**: **MASSIVE** - biomeOS becomes standalone OS platform

---

**Vision**: "Write once, run anywhere" → "Boot anywhere, orchestrate everywhere" 🧬🚀

---

*Last Updated: January 31, 2026*
