# BiomeOS Rust Evolution Plan

**Goal:** Replace bash "jelly strings" with robust, idiomatic Rust  
**Date:** December 27, 2025

---

## Current State Analysis

### Bash Scripts (To Be Replaced)
```
scripts/
├── create-alpine-biomeos-usb.sh    (deprecated)
├── create-bootable-usb.sh          → Rust: biomeos-mkusb
├── prepare-kernel.sh               → Rust: biomeos-mkboot (integrated)
├── prepare-usb.sh                  → Rust: biomeos-mkusb
├── setup-root-disk.sh              → Rust: biomeos-vm (new)
├── setup-vm-network.sh             → Rust: biomeos-vm
├── launch-vm-federation.sh         → Rust: biomeos-vm
├── setup-all-vm-disks.sh           → Rust: biomeos-vm
├── setup-single-vm-disk.sh         → Rust: biomeos-vm
└── test-iso-qemu.sh                → Rust: biomeos-vm (test subcommand)
```

### Rust Codebase
```
crates/biomeos-boot/
├── src/
│   ├── lib.rs                      ✅ Modern (needs docs)
│   ├── init_error.rs               ✅ Modern
│   ├── init_console.rs             ✅ Modern
│   ├── bootable.rs                 🔄 Good (24 warnings)
│   ├── initramfs.rs                🔄 Good (needs refactor)
│   └── bin/
│       ├── init.rs                 ⚠️  Needs major refactor
│       └── mkboot.rs               ✅ Good
└── tests/
    ├── boot_diagnostics.rs         ✅ Modern
    ├── qemu_harness.rs             ✅ Modern
    ├── integration_tests.rs        ⚠️  Minimal
    └── e2e_tests.rs                ⚠️  Minimal
```

---

## Phase 1: Fix init.rs (Priority 1)

### Current Issues
- **unwrap/expect:** ~15 instances
- **Error handling:** Generic panic on error
- **Module size:** 390 lines (should be split)
- **No module structure:** Everything in one file

### Refactoring Plan

#### 1.1: Extract Modules
```rust
crates/biomeos-boot/src/init/
├── mod.rs              // Main entry point
├── filesystem.rs       // Mount operations
├── hardware.rs         // Hardware detection
├── network.rs          // Network configuration
├── boot_params.rs      // Boot parameter parsing
├── shell.rs            // Shell spawning
└── emergency.rs        // Emergency mode
```

#### 1.2: Remove unwrap/expect
```rust
// ❌ Before
let console = std::fs::OpenOptions::new()
    .write(true)
    .open("/dev/console")
    .ok();

// ✅ After
let console = ConsoleWriter::new()
    .context("Failed to initialize console")?;
```

#### 1.3: Use New Error Types
```rust
// ❌ Before
if pid.as_raw() != 1 {
    error!("Must run as PID 1");
    return ExitCode::FAILURE;
}

// ✅ After
if pid.as_raw() != 1 {
    return Err(BootError::NotPid1(pid.as_raw()))?;
}
```

---

## Phase 2: Create biomeos-vm CLI (Priority 2)

### New Tool: `biomeos-vm`

**Purpose:** Replace all bash VM management scripts with single Rust CLI

```bash
# Network setup
biomeos-vm network setup
biomeos-vm network teardown
biomeos-vm network status

# Disk management
biomeos-vm disk create <name> <size>
biomeos-vm disk setup <name>
biomeos-vm disk list

# VM management
biomeos-vm launch <vm-name> [--gui|--headless]
biomeos-vm launch --federation [--count 3]
biomeos-vm stop <vm-name>
biomeos-vm stop --all

# Testing
biomeos-vm test iso <path>
biomeos-vm test federation
```

### Implementation
```
crates/biomeos-vm/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── network.rs      // Bridge setup
│   ├── disk.rs         // Disk image management
│   ├── vm.rs           // QEMU wrapper
│   ├── federation.rs   // Multi-VM orchestration
│   └── error.rs        // Error types
└── tests/
    └── integration.rs
```

---

## Phase 3: Fix Clippy Warnings (Priority 3)

### Remaining 24 Warnings

**Categories:**
1. Empty `writeln!` → Use explicit `""`
2. Unnecessary trim before split → Remove redundant trim
3. Identical if blocks → Consolidate
4. Missing documentation → Add docs
5. Unused parameters → Remove or mark with `_`

**Fix Strategy:**
```bash
# Auto-fix what we can
cargo clippy --fix --allow-dirty -p biomeos-boot

# Manual fixes for the rest
cargo clippy -p biomeos-boot --all-targets -- -W clippy::all
```

---

## Phase 4: Module Extraction (Priority 4)

### Extract from init.rs

#### filesystem.rs
```rust
//! Filesystem mounting and management

use crate::init_error::{BootError, Result};
use nix::mount::{mount, MsFlags};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::info;

pub struct FilesystemManager {
    mounted: HashSet<PathBuf>,
}

impl FilesystemManager {
    pub fn new() -> Self {
        Self {
            mounted: HashSet::new(),
        }
    }

    pub async fn mount_essential(&mut self) -> Result<()> {
        self.mount_if_needed("/proc", "proc", "proc", MsFlags::empty()).await?;
        self.mount_if_needed("/sys", "sysfs", "sysfs", MsFlags::empty()).await?;
        self.mount_if_needed("/dev", "devtmpfs", "devtmpfs", MsFlags::empty()).await?;
        // ... more mounts
        Ok(())
    }

    async fn mount_if_needed(
        &mut self,
        target: impl AsRef<Path>,
        source: &str,
        fstype: &str,
        flags: MsFlags,
    ) -> Result<()> {
        let target_path = target.as_ref().to_path_buf();
        
        if self.mounted.contains(&target_path) {
            return Ok(());
        }

        // Create mount point
        tokio::fs::create_dir_all(&target_path).await
            .map_err(|e| BootError::DirectoryCreation {
                path: target_path.clone(),
                error: e.to_string(),
            })?;

        // Try to mount
        match mount(Some(source), target.as_ref(), Some(fstype), flags, None::<&str>) {
            Ok(_) => {
                info!("  ✓ {}", target.as_ref().display());
                self.mounted.insert(target_path);
                Ok(())
            }
            Err(nix::errno::Errno::EBUSY) => {
                info!("  ✓ {} (already mounted)", target.as_ref().display());
                self.mounted.insert(target_path);
                Ok(())
            }
            Err(e) => Err(BootError::mount_failed(
                target.as_ref().display().to_string(),
                source,
                e,
            )),
        }
    }
}
```

#### hardware.rs
```rust
//! Hardware detection and configuration

use crate::init_error::{BootError, Result};
use sysinfo::{System, SystemExt};
use std::num::NonZeroUsize;
use tracing::info;

#[derive(Debug, Clone)]
pub struct HardwareInfo {
    pub cpu_count: NonZeroUsize,
    pub total_memory_gb: u64,
    pub architecture: Architecture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    X86_64,
    Aarch64,
    Riscv64,
}

impl Architecture {
    pub fn detect() -> Self {
        #[cfg(target_arch = "x86_64")]
        return Self::X86_64;
        
        #[cfg(target_arch = "aarch64")]
        return Self::Aarch64;
        
        #[cfg(target_arch = "riscv64")]
        return Self::Riscv64;
    }
}

pub async fn detect() -> Result<HardwareInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu_count = NonZeroUsize::new(sys.cpus().len())
        .ok_or_else(|| BootError::HardwareDetection(
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No CPUs detected"
            ))
        ))?;

    let total_memory = sys.total_memory();
    let total_memory_gb = total_memory / (1024 * 1024 * 1024);

    let architecture = Architecture::detect();

    info!("Hardware: {} cores, {} GB RAM, {:?}", 
          cpu_count, total_memory_gb, architecture);

    Ok(HardwareInfo {
        cpu_count,
        total_memory_gb,
        architecture,
    })
}
```

---

## Phase 5: Comprehensive Testing (Priority 5)

### Test Coverage Goals
- **Unit tests:** Every public function
- **Integration tests:** Module interaction
- **E2E tests:** Full boot sequence
- **Property tests:** Edge cases

### New Tests
```rust
// tests/init_modules.rs
#[tokio::test]
async fn test_filesystem_manager() {
    let mut mgr = FilesystemManager::new();
    // Mock mount operations
    assert!(mgr.mount_essential().await.is_ok());
}

#[tokio::test]
async fn test_hardware_detection() {
    let hw = hardware::detect().await.unwrap();
    assert!(hw.cpu_count.get() > 0);
    assert!(hw.total_memory_gb >= 0);
}

// tests/vm_integration.rs
#[test]
fn test_vm_cli_disk_create() {
    // Test biomeos-vm disk operations
}

#[test]
fn test_vm_cli_network_setup() {
    // Test biomeos-vm network operations
}
```

---

## Phase 6: Documentation (Priority 6)

### Documentation Standards

**Module Level:**
```rust
//! # Filesystem Management
//!
//! This module handles mounting and managing essential filesystems during
//! `BiomeOS` initialization.
//!
//! ## Features
//!
//! - Automatic detection of already-mounted filesystems
//! - Graceful handling of `EBUSY` errors
//! - Comprehensive error context
//!
//! ## Example
//!
//! ```no_run
//! use biomeos_boot::init::filesystem::FilesystemManager;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let mut mgr = FilesystemManager::new();
//! mgr.mount_essential().await?;
//! # Ok(())
//! # }
//! ```
```

**Function Level:**
```rust
/// Mounts a filesystem at the specified target.
///
/// This function will create the mount point if it doesn't exist and
/// gracefully handle the case where the filesystem is already mounted.
///
/// # Arguments
///
/// * `target` - The mount point (e.g., `/proc`)
/// * `source` - The source device or pseudo-filesystem
/// * `fstype` - The filesystem type (e.g., `"proc"`)
/// * `flags` - Mount flags
///
/// # Errors
///
/// Returns [`BootError::MountFailed`] if the mount operation fails.
/// Returns [`BootError::DirectoryCreation`] if the mount point cannot be created.
///
/// # Example
///
/// ```no_run
/// # use biomeos_boot::init::filesystem::FilesystemManager;
/// # async fn example() -> anyhow::Result<()> {
/// let mut mgr = FilesystemManager::new();
/// mgr.mount_if_needed("/proc", "proc", "proc", MsFlags::empty()).await?;
/// # Ok(())
/// # }
/// ```
pub async fn mount_if_needed(/*...*/) -> Result<()> {
    // ...
}
```

---

## Implementation Order

### Week 1 (Current)
1. ✅ Create error types (init_error.rs)
2. ✅ Create console abstraction (init_console.rs)
3. 🔄 Fix clippy warnings (24 remaining)
4. ⏳ Remove unwrap/expect from init.rs

### Week 2
5. ⏳ Extract filesystem module
6. ⏳ Extract hardware module
7. ⏳ Extract boot_params module
8. ⏳ Update init.rs to use new modules

### Week 3
9. ⏳ Create biomeos-vm CLI foundation
10. ⏳ Implement disk management commands
11. ⏳ Implement network management commands
12. ⏳ Implement VM launch commands

### Week 4
13. ⏳ Add comprehensive tests
14. ⏳ Add comprehensive documentation
15. ⏳ Performance profiling
16. ⏳ Final integration testing

---

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| **Clippy Warnings** | 24 | 0 |
| **Bash Scripts** | 10 | 0 |
| **unwrap/expect** | 15+ | 0 |
| **Test Coverage** | ~30% | 80% |
| **Documentation** | Partial | 100% |
| **Module Count** | 3 | 12+ |

---

## Benefits of Rust Evolution

### 1. Type Safety
- Compile-time guarantees
- No null pointer errors
- Exhaustive pattern matching

### 2. Error Handling
- Explicit error propagation
- Rich error context
- Recovery strategies

### 3. Maintainability
- Self-documenting code
- Refactoring confidence
- IDE support

### 4. Performance
- Zero-cost abstractions
- Compile-time optimizations
- Small binary sizes

### 5. Testability
- Unit test everything
- Integration tests
- Property-based testing

---

**Status:** Planning Complete - Ready for Execution  
**Next:** Start with init.rs refactoring and clippy fixes

