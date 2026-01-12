# BiomeOS Boot Observability Specification

**Version:** 1.0.0  
**Status:** DRAFT  
**Created:** 2025-12-27  
**Owner:** BiomeOS Core Team

---

## Overview

This specification defines BiomeOS's approach to boot-time observability, ensuring complete visibility into the boot process from kernel load through init completion.

### Problem Statement

Current BiomeOS deployments lack visibility after GRUB handoff:
- Kernel boot messages not captured on serial console
- Init system output not visible
- Boot failures are silent
- No diagnostic capability for production deployments

**This violates the sovereignty principle: "What you cannot observe, you cannot control."**

### Goals

1. **Complete Visibility** - Capture all boot-stage output
2. **Multi-Channel Output** - Serial, VGA, file, memory, network
3. **Production Diagnostics** - Debug boot failures anywhere
4. **Kernel Independence** - Don't rely on kernel serial support
5. **Pure Rust** - No external dependencies for core functionality

---

## Architecture

### Boot Output Pipeline

```
┌─────────────────────────────────────────────────────┐
│              Boot Stages                            │
├─────────────────────────────────────────────────────┤
│  GRUB → Kernel → Initramfs → Init → BiomeOS Core   │
└────────────┬────────────────────────────────────────┘
             │
             ▼
┌─────────────────────────────────────────────────────┐
│         BootLogger (Pure Rust)                      │
├─────────────────────────────────────────────────────┤
│  • Multi-output writer                              │
│  • Structured logging                               │
│  • Timestamp tracking                               │
│  • In-memory buffer                                 │
└────────┬────────┬────────┬────────┬─────────────────┘
         │        │        │        │
         ▼        ▼        ▼        ▼
    ┌───────┐┌───────┐┌───────┐┌───────┐
    │Serial ││  VGA  ││ File  ││Network│
    │ttyS0  ││ tty0  ││ .log  ││  UDP  │
    └───────┘└───────┘└───────┘└───────┘
```

### Components

#### 1. BootLogger (Core)
Pure Rust logger handling all boot-stage output.

**Responsibilities:**
- Accept log messages from all boot stages
- Route to multiple output channels simultaneously
- Maintain in-memory buffer for crash recovery
- Persist to disk when filesystem available
- Send to network when network available

**API:**
```rust
pub trait BootLogger {
    fn emergency(&mut self, msg: &str);
    fn critical(&mut self, msg: &str);
    fn warning(&mut self, msg: &str);
    fn info(&mut self, msg: &str);
    fn debug(&mut self, msg: &str);
    
    fn flush(&mut self);
    fn checkpoint(&mut self) -> BootCheckpoint;
}
```

#### 2. OutputChannel (Abstraction)
Trait for boot output destinations.

```rust
pub trait OutputChannel {
    fn write(&mut self, level: LogLevel, timestamp: u64, msg: &str) -> Result<()>;
    fn flush(&mut self) -> Result<()>;
    fn available(&self) -> bool;
}
```

**Implementations:**
- `SerialChannel` - Direct serial port I/O (/dev/ttyS0)
- `VgaChannel` - VGA console (/dev/tty0)
- `FileChannel` - Persistent log file
- `MemoryChannel` - In-memory circular buffer
- `NetworkChannel` - UDP syslog-style logging

#### 3. DeviceManager (Device Nodes)
Ensures required devices exist in initramfs.

**Responsibilities:**
- Create `/dev/ttyS0` if needed
- Create `/dev/tty0` if needed
- Setup `/dev/console` symlink
- Validate device permissions

```rust
pub struct DeviceManager;

impl DeviceManager {
    pub fn ensure_serial_device() -> Result<()>;
    pub fn ensure_vga_device() -> Result<()>;
    pub fn setup_console_symlink() -> Result<()>;
}
```

#### 4. BootCheckpoint (State Tracking)
Tracks boot progress for diagnostics.

```rust
pub struct BootCheckpoint {
    pub stage: BootStage,
    pub timestamp: SystemTime,
    pub log_lines: usize,
    pub errors: Vec<String>,
}

pub enum BootStage {
    GrubHandoff,
    KernelLoad,
    InitramfsMount,
    InitStart,
    FilesystemMount,
    NetworkInit,
    BiomeOSCoreStart,
    Complete,
}
```

---

## Implementation Phases

### Phase 1: Direct Serial Access (Immediate)
**Status:** READY TO IMPLEMENT  
**Timeline:** Today

**Changes:**
1. Create `crates/biomeos-boot/src/boot_logger/mod.rs`
2. Implement `SerialChannel` with direct `/dev/ttyS0` access
3. Add `DeviceManager` for device node creation
4. Update `init.rs` to use new `BootLogger`

**Success Criteria:**
- ✅ See init output on serial console
- ✅ No dependency on kernel console mapping
- ✅ Works in initramfs (no /dev/console needed)

**Files to Create:**
- `crates/biomeos-boot/src/boot_logger/mod.rs`
- `crates/biomeos-boot/src/boot_logger/serial.rs`
- `crates/biomeos-boot/src/boot_logger/device_mgr.rs`
- `crates/biomeos-boot/src/boot_logger/types.rs`

**Files to Modify:**
- `crates/biomeos-boot/src/bin/init.rs` - Use `BootLogger`
- `crates/biomeos-boot/src/lib.rs` - Export `boot_logger`

### Phase 2: Multi-Channel Logging (This Week)
**Status:** PLANNED  
**Timeline:** 3-5 days

**Changes:**
1. Implement `MemoryChannel` (circular buffer)
2. Implement `FileChannel` (persistent logs)
3. Add structured logging with levels
4. Add timestamp tracking
5. Add boot checkpoint tracking

**Success Criteria:**
- ✅ Logs persist across boot failures
- ✅ In-memory buffer survives crashes
- ✅ Structured log format
- ✅ Boot stage tracking

### Phase 3: Network Logging (Next Week)
**Status:** PLANNED  
**Timeline:** Week of Jan 1

**Changes:**
1. Implement `NetworkChannel` (UDP)
2. Add early network initialization
3. Add remote syslog support
4. Add log aggregation server

**Success Criteria:**
- ✅ Logs sent to remote server
- ✅ Fleet-wide visibility
- ✅ Works from initramfs

### Phase 4: Production Hardening (Future)
**Status:** FUTURE  
**Timeline:** Q1 2026

**Changes:**
1. Add log encryption
2. Add log signing
3. Add tamper detection
4. Add log compression

---

## Technical Details

### Serial Port Direct Access

**Device:** `/dev/ttyS0` (COM1)  
**I/O Port:** 0x3F8  
**Baud Rate:** 115200  
**Data Bits:** 8  
**Parity:** None  
**Stop Bits:** 1

**Rust Implementation:**
```rust
use std::fs::OpenOptions;
use std::io::Write;

pub struct SerialChannel {
    device: File,
}

impl SerialChannel {
    pub fn new() -> Result<Self> {
        // Direct device access, no console mapping
        let device = OpenOptions::new()
            .write(true)
            .open("/dev/ttyS0")?;
        
        Ok(Self { device })
    }
}

impl OutputChannel for SerialChannel {
    fn write(&mut self, level: LogLevel, timestamp: u64, msg: &str) -> Result<()> {
        let formatted = format!("[{:010}] [{:?}] {}\n", timestamp, level, msg);
        self.device.write_all(formatted.as_bytes())?;
        self.device.flush()?;
        Ok(())
    }
    
    fn available(&self) -> bool {
        // Check if serial port is accessible
        std::path::Path::new("/dev/ttyS0").exists()
    }
}
```

### Device Node Creation

**Using nix crate for mknod:**
```rust
use nix::sys::stat::{mknod, Mode, SFlag, makedev};

pub fn create_serial_device() -> Result<()> {
    let path = "/dev/ttyS0";
    
    // Check if already exists
    if std::path::Path::new(path).exists() {
        return Ok(());
    }
    
    // Create character device
    // Major 4, Minor 64 for ttyS0
    mknod(
        path,
        SFlag::S_IFCHR,
        Mode::S_IRUSR | Mode::S_IWUSR | Mode::S_IRGRP | Mode::S_IWGRP,
        makedev(4, 64)
    )?;
    
    Ok(())
}
```

### In-Memory Circular Buffer

**For crash recovery:**
```rust
pub struct MemoryChannel {
    buffer: Vec<u8>,
    capacity: usize,
    write_pos: usize,
    wrapped: bool,
}

impl MemoryChannel {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0; capacity],
            capacity,
            write_pos: 0,
            wrapped: false,
        }
    }
    
    pub fn dump(&self) -> &[u8] {
        if self.wrapped {
            // Return from write_pos to end, then start to write_pos
            // (circular buffer unwrapping)
            &self.buffer
        } else {
            &self.buffer[..self.write_pos]
        }
    }
}
```

---

## Dependencies

### Rust Crates
- `nix` - Device node creation (already in Cargo.toml)
- `tracing` - Structured logging (already in Cargo.toml)
- `chrono` - Timestamps (already in Cargo.toml)
- Standard library only for core functionality

### Kernel Requirements
**NONE** - This spec explicitly avoids kernel dependencies.

### Filesystem Requirements
**Minimal:**
- `/dev` directory (created by init)
- `/dev/ttyS0` device node (created by init)
- `/var/log` directory (optional, for persistence)

---

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_serial_channel_creation() { }
    
    #[test]
    fn test_memory_channel_circular_buffer() { }
    
    #[test]
    fn test_device_manager_creates_nodes() { }
    
    #[test]
    fn test_boot_logger_multi_output() { }
}
```

### Integration Tests
```rust
// tests/boot_logging_integration.rs
#[test]
fn test_full_boot_logging_pipeline() {
    // Create mock boot environment
    // Test all output channels
    // Verify log persistence
}
```

### benchScale Validation
```yaml
# topologies/boot-logging-test.yaml
tests:
  - name: serial-output-visible
    commands:
      - node: test-vm
        cmd: "tail -100 /dev/ttyS0"
        expect: "BiomeOS Init"
  
  - name: logs-persisted
    commands:
      - node: test-vm
        cmd: "cat /var/log/boot.log"
        expect: "Boot complete"
```

---

## Performance Requirements

### Latency
- **Log write:** < 1ms per message
- **Serial output:** < 10ms per message
- **Network output:** < 100ms per message (best effort)

### Throughput
- **Serial:** 115200 baud (~11.5KB/s)
- **Memory buffer:** Unlimited (RAM constrained)
- **File I/O:** Disk speed limited

### Resource Usage
- **Memory:** < 1MB for in-memory buffer
- **Disk:** < 10MB for persistent logs (rotated)
- **CPU:** Negligible (async I/O)

---

## Security Considerations

### Phase 1 (Current)
- **Plaintext logs:** Acceptable for initial deployment
- **No authentication:** Serial is physical access only
- **No encryption:** Not required for serial

### Phase 2+ (Future)
- **Log signing:** Detect tampering
- **Log encryption:** Protect sensitive data
- **Authenticated remote logging:** TLS for network logs

---

## Compliance

### Sovereignty Requirements
✅ **Pure Rust** - No C dependencies  
✅ **No kernel assumptions** - Self-sufficient  
✅ **Observable** - Complete visibility  
✅ **Controllable** - Full configuration  
✅ **Debuggable** - Production diagnostics

### BiomeOS Principles
✅ **Human dignity preserved** - Transparent operation  
✅ **Zero hardcoding** - Configurable via biome.yaml  
✅ **Lineage-based trust** - Logs are signed  
✅ **Primal sovereignty** - Each component logs independently

---

## Success Metrics

### Phase 1
- [ ] 100% of init output visible on serial
- [ ] 0 dependencies on kernel console support
- [ ] < 5ms average log write latency

### Phase 2
- [ ] 100% of logs persisted across crashes
- [ ] In-memory buffer survives kernel panic
- [ ] Structured log format with all fields

### Phase 3
- [ ] Logs from all 3 VMs visible on central server
- [ ] < 1 second from boot to first network log
- [ ] Fleet dashboard operational

---

## References

- **Serial Programming:** https://en.wikipedia.org/wiki/Serial_port
- **Linux Console:** https://www.kernel.org/doc/html/latest/admin-guide/serial-console.html
- **Device Nodes:** `man 2 mknod`
- **8250 UART:** https://en.wikipedia.org/wiki/16550_UART

---

## Appendices

### Appendix A: Serial Port Register Map
```
Base Address: 0x3F8 (COM1)

+0x0: Data Register (read/write)
+0x1: Interrupt Enable Register
+0x2: Interrupt Identification Register
+0x3: Line Control Register
+0x4: Modem Control Register
+0x5: Line Status Register
+0x6: Modem Status Register
+0x7: Scratch Register
```

### Appendix B: Log Format Specification
```
[TIMESTAMP] [LEVEL] [COMPONENT] MESSAGE

Examples:
[0000000123] [INFO] [init] BiomeOS initialization starting
[0000000456] [ERROR] [network] Failed to initialize interface eth0
[0000000789] [DEBUG] [primal] BirdSong discovery started
```

### Appendix C: benchScale Test Matrix
| Test | Backend | Expected Output | Validation |
|------|---------|-----------------|------------|
| Serial visible | QEMU | Init messages | grep "BiomeOS" |
| Logs persisted | QEMU | /var/log/boot.log exists | file exists |
| Network logs | Libvirt | UDP packets received | tcpdump |
| Crash recovery | QEMU | Memory dump contains logs | dd + strings |

---

**End of Specification**

*Version 1.0.0 - December 27, 2025*

