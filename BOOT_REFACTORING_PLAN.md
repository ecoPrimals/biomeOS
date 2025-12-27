# BiomeOS Boot Refactoring Plan

**Goal:** Evolve to modern, idiomatic Rust while maintaining functionality

**Date:** December 27, 2025

---

## Audit Findings

### Critical Issues to Address

#### 1. **Error Handling & Result Propagation**
- [ ] Remove all `unwrap()` calls
- [ ] Remove `ok()` silent failures
- [ ] Use `?` operator consistently
- [ ] Provide context with `.context()` for all errors

#### 2. **Async Patterns**
- [ ] Use proper async/await instead of blocking
- [ ] Avoid unnecessary `.await` in sync contexts
- [ ] Use `tokio::fs` for filesystem operations
- [ ] Consider `tokio::process` for child processes

#### 3. **Ownership & Borrowing**
- [ ] Reduce unnecessary clones
- [ ] Use references where possible
- [ ] Improve lifetime annotations
- [ ] Use `Cow<'_, str>` for conditional ownership

#### 4. **Type Safety**
- [ ] Create newtype wrappers for primitives
- [ ] Use enums instead of multiple bools
- [ ] Implement proper Display/Debug traits
- [ ] Add #[non_exhaustive] to future-proof structs

#### 5. **Documentation**
- [ ] Add module-level documentation
- [ ] Document all public APIs
- [ ] Add examples in doc comments
- [ ] Document panic conditions
- [ ] Add safety documentation for unsafe code

#### 6. **Code Organization**
- [ ] Extract magic numbers to constants
- [ ] Group related functionality into modules
- [ ] Use builder patterns for complex initialization
- [ ] Separate concerns (IO, business logic, error handling)

---

## Refactoring Strategy

### Phase 1: Init System (`init.rs`)

**Current Issues:**
```rust
// ❌ Silent failure
let _ = std::io::stdout().write_all(b"...");

// ❌ Manual console writes
let console = std::fs::OpenOptions::new().write(true).open("/dev/console").ok();

// ❌ Blocking in async context
std::process::Command::new("/bin/busybox").status()
```

**Improvements:**
```rust
// ✅ Proper error handling
write_to_console("...")?;

// ✅ Dedicated console writer
struct ConsoleWriter { /* ... */ }

// ✅ Async process handling
tokio::process::Command::new("/bin/busybox")
    .status()
    .await?
```

### Phase 2: Error Types (`error.rs`)

**Current:**
```rust
// Generic anyhow::Error everywhere
pub type Result<T> = anyhow::Result<T>;
```

**Improved:**
```rust
// Specific error types with context
#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error("Failed to mount {target}: {source}")]
    MountFailed {
        target: String,
        source: nix::errno::Errno,
    },
    
    #[error("Not running as PID 1 (current: {0})")]
    NotPid1(i32),
    
    #[error("Hardware detection failed: {0}")]
    HardwareDetection(#[from] sysinfo::Error),
}
```

### Phase 3: Filesystem Module

**Extract mounting logic:**
```rust
// crates/biomeos-boot/src/init/filesystem.rs

pub struct FilesystemManager {
    mounted: HashSet<PathBuf>,
}

impl FilesystemManager {
    pub async fn mount_essential(&mut self) -> Result<()> {
        self.mount_if_needed("/proc", "proc", "proc").await?;
        self.mount_if_needed("/sys", "sysfs", "sysfs").await?;
        // ...
        Ok(())
    }
    
    async fn mount_if_needed(
        &mut self,
        target: impl AsRef<Path>,
        source: &str,
        fstype: &str,
    ) -> Result<()> {
        // Idiomatic implementation
    }
}
```

### Phase 4: Hardware Detection Module

**Current:** Inline in init  
**Improved:** Separate module with proper types

```rust
// crates/biomeos-boot/src/init/hardware.rs

#[derive(Debug, Clone)]
pub struct HardwareInfo {
    cpu_count: NonZeroUsize,
    total_memory: u64,
    architecture: Architecture,
}

#[derive(Debug, Clone, Copy)]
pub enum Architecture {
    X86_64,
    Aarch64,
    Riscv64,
}

pub async fn detect() -> Result<HardwareInfo> {
    // Proper error handling
}
```

### Phase 5: Boot Parameters Module

**Strongly typed boot modes:**
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BootMode {
    Standard { config_path: Option<PathBuf> },
    Discovery,
    Install { target_device: PathBuf },
    Network { server: String },
    Recovery,
}

impl BootMode {
    pub fn from_cmdline(cmdline: &str) -> Result<Self> {
        // Parse with proper error handling
    }
}
```

### Phase 6: Console Writer Abstraction

**Type-safe console output:**
```rust
pub struct ConsoleWriter {
    stdout: io::Stdout,
    stderr: io::Stderr,
    console: Option<File>,
}

impl ConsoleWriter {
    pub fn new() -> Result<Self> { /* ... */ }
    
    pub fn write_line(&mut self, msg: &str) -> Result<()> {
        // Write to all outputs with proper error handling
    }
    
    pub fn write_error(&mut self, msg: &str) -> Result<()> {
        // Write to stderr and console
    }
}
```

---

## Implementation Order

### Day 1: Foundation (Today)
1. ✅ Audit complete
2. [ ] Create improved error types
3. [ ] Extract console writer
4. [ ] Add constants for magic values
5. [ ] Run clippy --fix

### Day 2: Modules
6. [ ] Extract filesystem module
7. [ ] Extract hardware module
8. [ ] Extract boot parameters module
9. [ ] Update tests

### Day 3: Polish
10. [ ] Add comprehensive documentation
11. [ ] Remove all unwrap/expect
12. [ ] Add integration tests
13. [ ] Benchmark and optimize

---

## Clippy Rules to Enforce

```toml
[lints.clippy]
# Correctness
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "warn"

# Performance
large_enum_variant = "warn"
large_stack_arrays = "warn"

# Style
doc_markdown = "warn"
missing_errors_doc = "warn"
missing_panics_doc = "warn"

# Pedantic (enforced)
pedantic = "warn"
```

---

## Testing Strategy

### Unit Tests
- Test each module independently
- Mock filesystem operations
- Test error paths

### Integration Tests
- Test full boot sequence
- Test with QEMU harness
- Validate error recovery

### Property Tests
- Use proptest for edge cases
- Fuzz boot parameter parsing
- Test mount logic with random inputs

---

## Documentation Requirements

### Module Level
```rust
//! # Filesystem Management
//!
//! This module handles mounting and managing essential filesystems during
//! `BiomeOS` init. It provides:
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
//! let mut mgr = FilesystemManager::new()?;
//! mgr.mount_essential().await?;
//! ```
```

### Function Level
```rust
/// Mounts a filesystem at the specified target.
///
/// # Arguments
///
/// * `target` - The mount point (e.g., `/proc`)
/// * `source` - The source device or pseudo-filesystem
/// * `fstype` - The filesystem type (e.g., `"proc"`)
///
/// # Errors
///
/// Returns [`MountError::AlreadyMounted`] if the target is already mounted.
/// Returns [`MountError::PermissionDenied`] if not running as root.
///
/// # Panics
///
/// This function does not panic.
///
/// # Example
///
/// ```no_run
/// mount("/proc", "proc", "proc").await?;
/// ```
pub async fn mount(
    target: impl AsRef<Path>,
    source: &str,
    fstype: &str,
) -> Result<(), MountError> {
    // ...
}
```

---

## Performance Considerations

### Avoid
- [ ] Unnecessary allocations in hot paths
- [ ] Blocking operations in async contexts
- [ ] Repeated string parsing
- [ ] Large copies (use references)

### Optimize
- [ ] Use `Cow` for conditional allocation
- [ ] Cache parsed boot parameters
- [ ] Use `SmallVec` for fixed-size collections
- [ ] Profile with `cargo flamegraph`

---

## Compatibility

### Rust Version
- **MSRV:** 1.75.0 (or latest stable)
- Use edition 2021 features
- Avoid nightly-only features in production code

### Dependencies
- Keep dependency tree minimal
- Prefer well-maintained crates
- Document security considerations
- Regular `cargo audit`

---

## Success Criteria

- [ ] All clippy lints pass (pedantic enabled)
- [ ] 100% documentation coverage for public APIs
- [ ] Zero `unwrap()`/`expect()` in production code
- [ ] All tests pass
- [ ] Boot time < 2 seconds (maintained)
- [ ] Binary size < 3MB (maintained)
- [ ] Code review approved

---

**Status:** Planning Complete - Ready for Implementation

