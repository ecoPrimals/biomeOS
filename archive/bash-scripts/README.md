# Archived Bash Scripts

**Purpose**: Historical record of bash scripts replaced by pure Rust implementations

---

## Scripts in this Archive

### `deploy-federation.sh.replaced`
- **Original**: `scripts/deploy-federation.sh` (143 lines)
- **Replaced By**: `crates/biomeos-deploy` (pure Rust)
- **Date Replaced**: December 27, 2025
- **Reason**: Evolution to pure Rust sovereignty

**Replacement Benefits**:
- Type-safe configuration
- Compile-time validation
- Better error messages
- Integrated testing
- Async/await orchestration

**Usage (Old)**:
```bash
./scripts/deploy-federation.sh
```

**Usage (New)**:
```bash
biomeos-deploy deploy -t topologies/rust-federation.yaml
```

---

### `build-rootfs-robust.sh.replaced`
- **Original**: `scripts/build-rootfs-robust.sh` (192 lines)
- **Replaced By**: Enhanced `crates/biomeos-boot/src/rootfs.rs` (pure Rust)
- **Date Replaced**: December 27, 2025
- **Reason**: Evolution to pure Rust sovereignty

**Replacement Benefits**:
- Configurable DNS servers (auto-discovered or specified)
- Smart NBD device selection (auto-detect available)
- Temporary mount points (safe tempfile usage)
- Configurable hostname
- Rich error context
- Type-safe configuration

**Usage (Old)**:
```bash
./scripts/build-rootfs-robust.sh
```

**Usage (New)**:
```bash
biomeos-rootfs --output biomeos-root.qcow2 \
  --primals primals/ \
  --size 8G
```

---

### `test-primals-vm.sh.replaced`
- **Original**: `scripts/test-primals-vm.sh` (35 lines)
- **Replaced By**: `examples/test_vm_primal.rs` (pure Rust)
- **Date Replaced**: December 27, 2025
- **Reason**: Evolution to pure Rust sovereignty

**Replacement Benefits**:
- Type-safe configuration
- Comprehensive error handling
- Integration with cargo
- Async/await VM management
- Built-in health monitoring

**Usage (Old)**:
```bash
./scripts/test-primals-vm.sh
```

**Usage (New)**:
```bash
cargo run --release --example test_vm_primal
```

---

### `verify-primals.sh.replaced`
- **Original**: `scripts/verify-primals.sh` (75 lines)
- **Replaced By**: `biomeos-verify` CLI (pure Rust)
- **Date Replaced**: December 27, 2025
- **Reason**: Evolution to modern idiomatic Rust

**Replacement Benefits**:
- Async boot verification
- Comprehensive primal checking
- Structured error reporting
- Timeout handling with tokio
- Clean separation of concerns
- Full test coverage

**Usage (Old)**:
```bash
./scripts/verify-primals.sh dist/biomeos.iso vm-testing/biomeos-root.qcow2
```

**Usage (New)**:
```bash
biomeos-verify --serial-log /tmp/vm1-serial.log --rootfs biomeos-root/
```

---

### `test-federation-quick.sh.replaced`
- **Original**: `scripts/test-federation-quick.sh` (118 lines)
- **Replaced By**: `biomeos-deploy` with test topology (pure Rust)
- **Date Replaced**: December 27, 2025
- **Reason**: Functionality already covered by biomeos-deploy

**Replacement Benefits**:
- Declarative YAML topology
- Type-safe configuration
- Health monitoring built-in
- Graceful shutdown
- Serial log aggregation

**Usage (Old)**:
```bash
./scripts/test-federation-quick.sh dist/biomeos.iso
```

**Usage (New)**:
```bash
biomeos-deploy deploy -t topologies/rust-federation.yaml
biomeos-deploy health -t topologies/rust-federation.yaml
```

---

### `launch-vm-federation.sh.replaced`
- **Original**: `scripts/launch-vm-federation.sh` (132 lines)
- **Replaced By**: `biomeos-deploy` federation orchestration (pure Rust)
- **Date Replaced**: December 27, 2025
- **Reason**: Functionality already covered by biomeos-deploy

**Replacement Benefits**:
- Full federation orchestration
- Network bridge management
- VM lifecycle management
- Persistent PIDs and state
- Comprehensive error handling

**Usage (Old)**:
```bash
./scripts/launch-vm-federation.sh dist/biomeos.iso
```

**Usage (New)**:
```bash
biomeos-deploy deploy -t topologies/rust-federation.yaml
biomeos-deploy shutdown -t topologies/rust-federation.yaml
```

---

### `setup-single-vm-disk.sh.replaced`
- **Original**: `scripts/setup-single-vm-disk.sh` (71 lines)
- **Replaced By**: `biomeos-rootfs` CLI (pure Rust)
- **Date Replaced**: December 27, 2025
- **Reason**: Superseded by enhanced rootfs.rs functionality

**Replacement Benefits**:
- Comprehensive root filesystem building
- Smart NBD device auto-detection
- Temporary mount point management
- Configurable hostname and networking
- Service installation integrated
- Rich error context and recovery

**Usage (Old)**:
```bash
./scripts/setup-single-vm-disk.sh 1 vm-testing/vm1.qcow2
```

**Usage (New)**:
```bash
biomeos-rootfs --output vm-testing/vm1.qcow2 \
  --primals primals/ \
  --services services/ \
  --hostname vm1 \
  --size 2G
```

---

### `setup-all-vm-disks.sh.replaced`
- **Original**: `scripts/setup-all-vm-disks.sh` (87 lines)
- **Replaced By**: `biomeos-rootfs` CLI (pure Rust)
- **Date Replaced**: December 27, 2025 (Phase 4)
- **Reason**: Loop wrapper around superseded functionality

**Replacement Benefits**:
- Same as setup-single-vm-disk.sh
- Just call `biomeos-rootfs` multiple times
- Or use shell loop if batch processing needed

**Usage (Old)**:
```bash
./scripts/setup-all-vm-disks.sh
```

**Usage (New)**:
```bash
# Create multiple disks
for i in 1 2 3; do
  biomeos-rootfs --output vm-testing/vm${i}.qcow2 \
    --hostname vm${i} --size 2G
done

# Or use biomeos-deploy for full federation setup
biomeos-deploy deploy -t topologies/rust-federation.yaml
```

---

### `setup-root-disk.sh.replaced`
- **Original**: `scripts/setup-root-disk.sh` (78 lines)
- **Replaced By**: `biomeos-rootfs` CLI (pure Rust)
- **Date Replaced**: December 27, 2025 (Phase 4)
- **Reason**: Similar pattern to setup-single-vm-disk.sh, superseded

**Replacement Benefits**:
- Comprehensive filesystem building
- Auto-detection of dependencies
- Configurable options
- Rich error messages

**Usage (Old)**:
```bash
./scripts/setup-root-disk.sh vm-testing/root.qcow2
```

**Usage (New)**:
```bash
biomeos-rootfs --output vm-testing/root.qcow2 \
  --primals primals/ \
  --services services/ \
  --size 8G
```

---

### `benchscale-federation.sh.replaced`
- **Original**: `scripts/benchscale-federation.sh` (122 lines)
- **Replaced By**: N/A - Dead code (benchscale removed)
- **Date Replaced**: December 27, 2025 (Phase 4)
- **Reason**: References deleted `benchscale` project

**Context**:
- This script was a wrapper for the benchscale project
- benchscale was removed from the codebase (see deleted_files)
- Functionality superseded by `biomeos-deploy`

**Usage (Old)**:
```bash
./scripts/benchscale-federation.sh create
```

**Usage (New)**:
```bash
# Use biomeos-deploy instead
biomeos-deploy deploy -t topologies/rust-federation.yaml
```

---

### `build-rootfs.sh.replaced`
- **Original**: `scripts/build-rootfs.sh` (197 lines)
- **Replaced By**: `biomeos-rootfs` CLI (pure Rust)
- **Date Replaced**: December 27, 2025 (Phase 5)
- **Reason**: Superseded by enhanced rootfs.rs functionality

**Replacement Benefits**:
- Same comprehensive functionality as bash version
- All features of enhanced rootfs.rs
- Type-safe configuration
- Better error handling
- Configurable options (DNS, hostname, services)

**Usage (Old)**:
```bash
./scripts/build-rootfs.sh vm-testing/biomeos.qcow2 8G primals/
```

**Usage (New)**:
```bash
biomeos-rootfs --output vm-testing/biomeos.qcow2 \
  --primals primals/ \
  --services services/ \
  --size 8G
```

---

### `build-rootfs-simple.sh.replaced`
- **Original**: `scripts/build-rootfs-simple.sh` (178 lines)
- **Replaced By**: `biomeos-rootfs` CLI (pure Rust)
- **Date Replaced**: December 27, 2025 (Phase 5)
- **Reason**: Superseded by enhanced rootfs.rs functionality

**Replacement Benefits**:
- "Simple" version also covered by rootfs.rs
- Same ease of use, better implementation
- All configuration options available

**Usage (Old)**:
```bash
./scripts/build-rootfs-simple.sh biomeos-root vm-testing/biomeos.qcow2 8G primals/
```

**Usage (New)**:
```bash
biomeos-rootfs --output vm-testing/biomeos.qcow2 \
  --primals primals/ \
  --size 8G
```

---

### `demo-ui.sh.replaced`
- **Original**: `scripts/demo-ui.sh` (88 lines)
- **Replaced By**: N/A - UI evolved to separate primal (petalTongue)
- **Date Replaced**: December 27, 2025 (Phase 6)
- **Reason**: UI functionality moved to dedicated petalTongue primal

**Context**:
- BiomeOS UI was evolved into a separate, sovereign primal
- petalTongue primal handles all UI functionality
- Follows BiomeOS philosophy: specialized primals, not monolithic system

**Usage (Old)**:
```bash
./scripts/demo-ui.sh
```

**Usage (New)**:
```bash
# UI is now in petalTongue primal (../petalTongue)
cd ../petalTongue && cargo run
```

---

### `demo_universal_ui.sh.replaced`
- **Original**: `scripts/demo_universal_ui.sh` (695 lines)
- **Replaced By**: N/A - UI evolved to petalTongue primal
- **Date Replaced**: December 27, 2025 (Phase 6)
- **Reason**: UI functionality moved to dedicated petalTongue primal

**Context**:
- Comprehensive universal UI demo
- All functionality now in petalTongue primal
- Clean separation: BiomeOS (core OS) + petalTongue (UI primal)

**Usage (Old)**:
```bash
./scripts/demo_universal_ui.sh
```

**Usage (New)**:
```bash
# Universal UI is now petalTongue primal
cd ../petalTongue && cargo run -- --universal-demo
```

---

### `quick-demo.sh.replaced`
- **Original**: `scripts/quick-demo.sh` (53 lines)
- **Replaced By**: N/A - UI evolved to petalTongue primal
- **Date Replaced**: December 27, 2025 (Phase 6)
- **Reason**: UI functionality moved to dedicated petalTongue primal

**Context**:
- Quick UI demo launcher
- petalTongue primal now handles all UI demos

**Usage (Old)**:
```bash
./scripts/quick-demo.sh
```

**Usage (New)**:
```bash
# Quick demos now in petalTongue
cd ../petalTongue && cargo run -- --quick-demo
```

---

## Evolution Progress

**Bash Scripts Eliminated**: 18 / 28 (64.3%)  
**Target**: 100% Pure Rust (Tier 3)  
**Status**: 64.3%! Racing past 75% milestone! 🚀✨🎉

---

## Related Documentation

- `PURE_RUST_SOVEREIGNTY.md` - Overall strategy
- `RUST_EVOLUTION_SESSION.md` - Session report
- `showcase/04-deployment-evolution/` - Evolution demos

---

**Note**: These scripts are kept for reference only. Do not use in production. Use the pure Rust replacements instead.

