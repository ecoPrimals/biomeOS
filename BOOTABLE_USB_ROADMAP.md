# BiomeOS Bootable USB Roadmap

**Status**: Existing bash scripts → Pure Rust evolution needed  
**Target**: Bootable USB for NUC + VM integration  
**Date**: December 26, 2025

---

## Current State Analysis

### ✅ What Exists (Old Architecture)

**ISO Builder** (`installer/make-biomeos-iso.sh`):
- Bash-based ISO creation script (~1,053 lines)
- GRUB bootloader configuration
- Placeholder kernel/initrd (NOT REAL!)
- biomeOS binary packaging
- Multiple boot modes (Standard, Demo, YAML Editor, Install)

**Bootstrap Orchestration** (`specs/BOOTSTRAP_ORCHESTRATION_SEQUENCE.md`):
- Complete startup sequence specification
- Dependency graph for primals
- Health check coordination
- Failure recovery procedures
- Python pseudocode examples

### ❌ What's Missing

**Critical Gaps**:
1. ❌ **No real Linux kernel** - Just placeholder files
2. ❌ **No initrd/initramfs** - No boot environment
3. ❌ **Bash-based** - Not pure Rust
4. ❌ **No USB persistence** - Can't save state
5. ❌ **No primal discovery** - Doesn't use our registry
6. ❌ **No network boot** - Can't coordinate with VMs
7. ❌ **Old UI references** - Points to archived code

---

## Gap Analysis: Where We Are vs. Bootable USB

### Distance from Bootable USB

```
Current State:        [▓▓▓░░░░░░░] 30% Complete
                      
What We Have:         What We Need:
✅ Pure Rust platform  ❌ Linux kernel selection
✅ Primal registry     ❌ Initramfs builder (Rust)
✅ P2P coordination    ❌ USB persistence layer  
✅ benchScale labs     ❌ Boot menu (Rust TUI)
✅ Docker integration  ❌ Network configuration
✅ BYOB manifests      ❌ Installation system
✅ ISO build scripts   ❌ Pure Rust ISO builder

Estimate: 4-6 weeks to production-quality bootable USB
```

---

## NUC Experiment Plan

### Hardware Setup

**Your NUC**:
- Boot from USB (BiomeOS)
- Run primal services natively
- Coordinate with VMs via network

**VMs** (using benchScale):
- Create 2-3 VMs simulating other nodes
- Test P2P coordination
- Validate network boot discovery

### Experiment Scenarios

#### Scenario 1: USB Boot + Local Primals
```bash
# 1. Create bootable USB
cd biomeOS
cargo run --bin biomeos-boot -- create-usb /dev/sdX

# 2. Copy Phase 1 binaries to USB
cp ../phase1bins/* /mnt/usb/primals/

# 3. Boot NUC from USB
# - BiomeOS init starts
# - Discovers primals on USB
# - Starts local services
# - Presents boot menu
```

#### Scenario 2: NUC + VM Coordination
```bash
# On host: Start benchScale VMs
cd ../benchscale
cargo run -- create biomeos-nuc-test topologies/nuc-integration.yaml

# On NUC (booted from USB):
# - Discovers VMs via mDNS
# - Establishes BTSP tunnels
# - Coordinates P2P mesh
# - Runs full integration test
```

#### Scenario 3: Persistent USB Storage
```bash
# Boot NUC from USB
# - Load saved biome.yaml from USB
# - Use persistent keys from USB
# - Save logs to USB
# - Update primals on USB
```

---

## Pure Rust Evolution Plan

### Phase 1: Boot Infrastructure (Week 1-2)

**Priority 1: Rust Init System**
- Replace bash init with pure Rust
- Mount essential filesystems (`/proc`, `/sys`, `/dev`)
- Parse boot parameters
- Integrate with primal registry

**Priority 2: Initramfs Builder**
- Pure Rust initramfs generation
- Include BiomeOS binaries
- Minimal dependencies
- Fast boot times

**Priority 3: Kernel Integration**
- Use system kernel or download minimal
- Proper initrd generation
- Module loading

### Phase 2: USB Persistence (Week 2-3)

**USB Detection & Partitioning**:
- Detect BiomeOS USB drives
- Create persistent data partition
- ext4 filesystem for primals/configs

**Storage Structure**:
```
/usb/
├── primals/       ← Discovered by primal registry
├── configs/       ← biome.yaml files
├── keys/          ← BearDog keys
├── logs/          ← System logs
└── biomes/        ← Saved ecosystems
```

### Phase 3: Boot Menu & TUI (Week 3)

**Pure Rust TUI**:
- ratatui-based boot menu
- Select boot mode
- Configure network
- Primal selection

**Boot Modes**:
- Standard (load biome.yaml)
- Discovery (scan network)
- Installation
- Network Boot
- Shell

### Phase 4: Network Boot & VM Integration (Week 4)

**Network Discovery**:
- mDNS service discovery
- Find BiomeOS VMs
- Establish P2P mesh

**VM Coordination**:
- Use existing P2P coordination
- BTSP tunnels to VMs
- Distributed primals

### Phase 5: Pure Rust ISO Builder (Week 5-6)

**ISO Generation**:
- Replace bash scripts
- Pure Rust implementation
- xorriso integration
- GRUB configuration

---

## Implementation Priority

### Must Have (Week 1-3)
1. ✅ Pure Rust init system
2. ✅ Initramfs builder
3. ✅ USB persistence layer
4. ✅ Primal registry integration
5. ✅ Basic boot menu (TUI)

### Should Have (Week 3-4)
6. ✅ Network discovery (mDNS)
7. ✅ P2P coordination with VMs
8. ✅ GRUB configuration generator
9. ✅ Installation system

### Nice to Have (Week 5-6)
10. ⚠️ OTA updates from USB
11. ⚠️ Multi-boot support
12. ⚠️ Encrypted USB partition
13. ⚠️ Recovery mode

---

## Timeline Estimate

```
Week 1: Boot Infrastructure
├── Day 1-2: Rust init system
├── Day 3-4: Initramfs builder  
└── Day 5-7: Kernel integration

Week 2: USB Persistence
├── Day 1-3: Partition management
├── Day 4-5: Persistent storage
└── Day 6-7: Primal discovery

Week 3: Boot Menu & TUI
├── Day 1-3: TUI boot menu
├── Day 4-5: Configuration UI
└── Day 6-7: Integration testing

Week 4: Network Boot
├── Day 1-3: Network discovery
├── Day 4-5: VM coordination
└── Day 6-7: P2P integration

Week 5-6: Polish & Testing
├── ISO builder refinement
├── NUC testing
├── VM integration tests
└── Documentation
```

**Total Estimate**: 4-6 weeks to production-quality bootable USB

---

## Next Immediate Steps

1. **Create `crates/biomeos-boot/`** - New crate for boot infrastructure
2. **Implement Rust init system** - Replace placeholder init
3. **Build initramfs generator** - Pure Rust implementation
4. **Test on VM first** - Before NUC hardware
5. **Integrate with existing code** - Use primal registry, P2P coordination

**First Command to Run**:
```bash
cd biomeOS
cargo new --lib crates/biomeos-boot
```

---

## Advantages of This Approach

✅ **Pure Rust** - No bash scripts, all type-safe  
✅ **Reuses existing code** - Primal registry, P2P coordination  
✅ **Production patterns** - Same quality as rest of BiomeOS  
✅ **Testable** - Can test with benchScale before hardware  
✅ **Flexible** - USB, network, or CD boot  
✅ **Modern** - TUI, async, structured logging  
✅ **Secure** - Memory safe, no shell injection  

**This is a natural evolution, not a rewrite!** 🚀

