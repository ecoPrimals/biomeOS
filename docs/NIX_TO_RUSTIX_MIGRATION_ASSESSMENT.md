# nix to rustix Migration Assessment

**Date:** 2025-03-13  
**Status:** Assessment only — DO NOT migrate without careful testing.  
**Scope:** Crates using `nix` in the biomeOS workspace.

---

## Summary

| Crate | nix Features | Primary Usage | rustix Equivalent | Custom Needed |
|-------|-------------|---------------|-------------------|---------------|
| biomeos-boot | mount, fs, process, net, ioctl, reboot, hostname, user | mount, mknod, makedev, errno | Partial | Yes (mount API, mknod) |
| biomeos-core | fs | statvfs | Yes | No |
| biomeos-spore | signal, process, fs | kill, Pid, statvfs | Yes | No |
| biomeos-graph | signal | kill, Pid |

| biomeos | signal | (via deps) | Yes | No |
| biomeos-atomic-deploy | signal | kill, Pid | Yes | No |
| biomeos-deploy | process, net, ioctl, signal | kill, Pid, errno | Yes | Yes (ioctl) |
| biomeos-ui | fs | statvfs | Yes | No |

---

## Per-Crate Analysis

### 1. biomeos-boot

**Cargo.toml:** `nix = { version = "0.29", features = ["mount", "fs", "process", "net", "ioctl", "reboot", "hostname", "user"] }`

**Usage in .rs files:**

| File | Import | Operation | rustix Equivalent |
|------|--------|-----------|-------------------|
| `init_filesystem.rs` | `nix::mount::{mount, MsFlags}` | mount | `rustix::mount` — Linux mount API |
| `init_filesystem.rs` | `nix::errno::Errno::EBUSY` | errno check | `rustix::io::Errno` or `std::io::Error::raw_os_error()` |
| `boot_logger/device_mgr.rs` | `nix::sys::stat::{makedev, mknod, Mode, SFlag}` | mknod, makedev | **Custom** — rustix has `mknod` in `rustix::fs` but API differs; `makedev` may need `libc` or manual |
| `init_error.rs` | `nix::errno::Errno` | Error type | `rustix::io::Errno` |

**Notes:**
- rustix has `rustix::mount` for Linux mount API.
- rustix `fs::mknod` exists but may have different signature; `makedev` equivalent may need `libc` or manual `(major << 8) | minor` for dev_t.
- `reboot`, `hostname`, `user` features — no direct usage found in grep; may be transitive or unused.

---

### 2. biomeos-core

**Cargo.toml:** `nix = { version = "0.29", features = ["fs"] }`

**Usage:**

| File | Import | Operation | rustix Equivalent |
|------|--------|-----------|-------------------|
| `integration/live_service.rs` | `nix::sys::statvfs::statvfs` | statvfs | `rustix::fs::statvfs` |

**Migration:** Straightforward. `rustix::fs::statvfs` returns `StatVfs` with equivalent fields.

---

### 3. biomeos-spore

**Cargo.toml:** `nix = { version = "0.29", features = ["signal", "process", "fs"] }`

**Usage:**

| File | Import | Operation | rustix Equivalent |
|------|--------|-----------|-------------------|
| `neural_spore.rs` | `nix::sys::signal::{kill, Signal}`, `nix::unistd::Pid` | kill(pid, sig) | `rustix::process::kill_process` |
| `usb.rs` | `nix::sys::statvfs::statvfs` | statvfs | `rustix::fs::statvfs` |

**Migration:** Straightforward. `rustix::process::kill_process` and `rustix::process::Signal`; `rustix::fs::statvfs`.

---

### 4. biomeos-graph

**Cargo.toml:** `nix = { version = "0.29", features = ["signal"] }`

**Usage:**

| File | Import | Operation | rustix Equivalent |
|------|--------|-----------|-------------------|
| `executor/rollback.rs` | `nix::sys::signal::{kill, Signal}`, `nix::unistd::Pid` | kill | `rustix::process::kill_process` |
| `neural_executor.rs` | same | kill | same |

**Migration:** Straightforward.

---

### 5. biomeos (UniBin)

**Cargo.toml:** `nix = { version = "0.29", features = ["signal"] }`

**Usage:** No direct `use nix::` in biomeos crate. Likely transitively via biomeos-graph, biomeos-atomic-deploy, or biomeos-spore. Remove from biomeos Cargo.toml if no direct usage after migration of dependent crates.

---

### 6. biomeos-atomic-deploy

**Cargo.toml:** `nix = { version = "0.29", features = ["signal"] }`

**Usage:**

| File | Import | Operation | rustix Equivalent |
|------|--------|-----------|-------------------|
| `primal_launcher.rs` | `nix::sys::signal::kill`, `nix::unistd::Pid` | kill | `rustix::process::kill_process` |
| `lifecycle_manager/resurrection.rs` | same | kill | same |

**Migration:** Straightforward.

---

### 7. biomeos-deploy

**Cargo.toml:** `nix = { version = "0.29", features = ["process", "net", "ioctl", "signal"] }`

**Usage:**

| File | Import | Operation | rustix Equivalent |
|------|--------|-----------|-------------------|
| `qemu.rs` | `nix::sys::signal::{kill, Signal}`, `nix::unistd::Pid` | kill | `rustix::process::kill_process` |
| `error.rs` | `nix::errno::Errno` | From impl | `rustix::io::Errno` |

**Notes:** `process`, `net`, `ioctl` features — no direct usage found in grep. May be used elsewhere or for error types. Check `ioctl` usage if any QEMU or TTY control is needed.

---

### 8. biomeos-ui

**Cargo.toml:** `nix = { version = "0.29", features = ["fs"] }`

**Usage:**

| File | Import | Operation | rustix Equivalent |
|------|--------|-----------|-------------------|
| `capabilities/device_management/provider.rs` | `nix::sys::statvfs::statvfs` | statvfs | `rustix::fs::statvfs` |

**Migration:** Straightforward.

---

## Test / Helper Code (Not Production)

| Location | Usage | Notes |
|----------|-------|-------|
| `tests/atomics/common/helpers.rs` | `nix::unistd::Uid` | `rustix::process::getuid` or `std::env::var("EUID")` |
| `tests/atomics/common/chaos_engine.rs` | `nix::sys::signal`, `nix::unistd::Pid` | Same as production |
| `tests/atomics/common/fixtures.rs` | `nix::unistd::getuid` | `rustix::process::getuid` |
| `tests/atomics/tower_chaos.rs` | `nix::sys::signal::Signal` | `rustix::process::Signal` |

---

## rustix API Mapping Summary

| nix | rustix |
|-----|--------|
| `nix::sys::statvfs::statvfs` | `rustix::fs::statvfs` |
| `nix::sys::signal::kill` + `nix::unistd::Pid` | `rustix::process::kill_process` |
| `nix::sys::signal::Signal` | `rustix::process::Signal` |
| `nix::errno::Errno` | `rustix::io::Errno` |
| `nix::mount::{mount, MsFlags}` | `rustix::mount` (Linux mount API) |
| `nix::sys::stat::{makedev, mknod, Mode, SFlag}` | `rustix::fs::mknod` (verify makedev/dev_t) |
| `nix::unistd::getuid` | `rustix::process::getuid` |
| `nix::unistd::Uid` | `rustix::process::Uid` |

---

## Operations Requiring Custom Implementation

1. **makedev / dev_t** — rustix may use different types. Check `rustix::fs::mknod` signature for device number encoding.
2. **Mount flags** — rustix mount API may use different flag types; verify `MsFlags` mapping.
3. **ioctl** — If biomeos-deploy uses ioctl for QEMU/TTY, rustix has `rustix::ioctl` but API is lower-level; may need wrapper.

---

## Recommended Migration Order

1. **Low risk:** biomeos-core, biomeos-ui, biomeos-spore (statvfs + signal)
2. **Medium:** biomeos-graph, biomeos-atomic-deploy (signal only)
3. **Higher risk:** biomeos-boot (mount, mknod), biomeos-deploy (errno, ioctl)

---

## Dependencies

Add to Cargo.toml:
```toml
rustix = { version = "0.38", features = ["fs", "process", "mount", "ioctl"] }
```

Use `rustix` feature flags to match what each crate needs. Avoid pulling in unused features.
