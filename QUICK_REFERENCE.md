# BiomeOS - Quick Reference

**Version**: 0.1.0  
**Status**: Production Validated + Pure Rust Evolution (64.3% - TWO-THIRDS!)  
**Last Updated**: December 27, 2025 (Phase 6 - Architectural Clarity)

---

## 🚀 Quick Commands

### Pure Rust Tools 🦀 (TWO-THIRDS RUST!)

#### Deploy Federation
```bash
# Pure Rust deployment orchestrator
biomeos-deploy deploy -t topologies/rust-federation.yaml --kvm --wait

# Check health
biomeos-deploy health -t topologies/rust-federation.yaml

# Shutdown
biomeos-deploy shutdown -t topologies/rust-federation.yaml
```

#### Verify VM
```bash
# Pure Rust VM verification
biomeos-verify --serial-log /tmp/vm1-serial.log --rootfs biomeos-root/

# With custom timeout
biomeos-verify -s /tmp/vm1-serial.log -t 60 --verbose
```

#### Build Root Filesystem
```bash
# Pure Rust filesystem builder (replaces ALL bash variants!)
biomeos-rootfs --output root.qcow2 \
  --primals primals/ \
  --services services/ \
  --size 8G

# Batch create multiple VMs
for i in 1 2 3; do
  biomeos-rootfs --output vm${i}.qcow2 --hostname vm${i} --size 2G
done
```

#### Test VM
```bash
# Pure Rust VM testing
cargo run --release --example test_vm_primal
```

### Boot System
```bash
# Build bootable ISO
cargo run --release -p biomeos-boot --bin biomeos-mkboot -- iso

# Test in VM (using pure Rust tools)
biomeos-deploy deploy -t topologies/test-single.yaml
biomeos-verify --serial-log /tmp/test-vm-serial.log
```

---

## 📊 Pure Rust Evolution Status

**Progress**: 53.5% (15/28 bash scripts) - **MAJORITY PURE RUST!** 🎉  
**Velocity**: MAXIMUM (instant wins!)  
**Target**: 100% Pure Rust sovereignty

**Categories 100% Complete** (3/5):
- ✅ Deployment (6/6 scripts)
- ✅ Filesystem (7/7 scripts) - COMPLETE!
- ✅ Network (integrated)

**Scripts Eliminated**:
1-13. *(Previous scripts)*
14. ✅ build-rootfs.sh → biomeos-rootfs 🆕
15. ✅ build-rootfs-simple.sh → biomeos-rootfs 🆕

**Remaining**: 13 scripts (Testing, Build/Boot, Demo)  
**Next Milestone**: 75% (7 more scripts)
1. ✅ `deploy-federation.sh` → `biomeos-deploy`
2. ✅ `build-rootfs-robust.sh` → enhanced `rootfs.rs`
3. ✅ `test-primals-vm.sh` → `examples/test_vm_primal.rs`
4. ✅ `setup-vm-network.sh` → `biomeos-deploy/network.rs`
5. ✅ `install-services.sh` → integrated into `rootfs.rs`
6. ✅ `biomeos-vm-wrapper.sh` → `biomeos-deploy` CLI

**Remaining**: 22 scripts  
**Next Target**: 30% milestone

---

## 🎯 Core Principles

1. **Deep Debt Solutions** - Fix root causes, not symptoms
2. **Modern Idiomatic Rust** - Best practices, zero-cost abstractions
3. **Smart Refactoring** - Reuse over duplication
4. **Safe Rust** - Zero unsafe blocks
5. **Capability-Based Discovery** - No hardcoding
6. **Primal Self-Knowledge** - Self-awareness only
7. **Mocks Only in Tests** - Real production code

---

## 📁 Key Directories

```
biomeOS/
├── crates/
│   ├── biomeos-deploy/      🆕 Pure Rust deployment
│   ├── biomeos-boot/         ⚡ Pure Rust boot system
│   ├── biomeos-core/         💚 Core logic
│   └── biomeos-types/        🧬 Shared types
├── examples/
│   └── test_vm_primal.rs    🆕 Pure Rust VM testing
├── archive/
│   └── bash-scripts/        📦 Replaced bash scripts (6)
├── scripts/                 📜 Remaining bash (22 scripts)
└── docs/                    📚 Documentation
```

---

## 🔧 Development Workflow

### 1. Build All
```bash
cargo build --release
```

### 2. Run Tests
```bash
cargo test
```

### 3. Deploy Federation (Pure Rust)
```bash
biomeos-deploy deploy -t topologies/rust-federation.yaml
```

### 4. Check Logs
```bash
tail -f vm-testing/*.log
```

---

## 📚 Documentation

**Start Here**:
- `START_HERE.md` - Entry point
- `README.md` - Project overview
- `QUICK_REFERENCE.md` - This file

**Pure Rust Evolution**:
- `PURE_RUST_SOVEREIGNTY.md` - Vision & strategy
- `COMPREHENSIVE_SESSION_FINAL.md` - Latest progress
- `MILESTONE_20_PERCENT.md` - 20% achievement
- `archive/bash-scripts/README.md` - Eliminated scripts

**Architecture**:
- `docs/architecture/` - System design
- `specs/` - Technical specifications
- `showcase/` - Demos & examples

---

## 🐛 Troubleshooting

### Common Issues

**Issue**: Network bridge already exists  
**Solution**: Tool will use existing bridge automatically

**Issue**: VM won't start  
**Solution**: Check serial logs in `vm-testing/*.log`

**Issue**: Permission denied  
**Solution**: Use `sudo -E` for commands needing elevated privileges

---

## 🎯 Next Steps

### For Users
1. Try pure Rust deployment: `biomeos-deploy deploy ...`
2. Build root filesystem: `biomeos-rootfs ...`
3. Test VM: `cargo run --example test_vm_primal`

### For Developers
1. Review `PURE_RUST_SOVEREIGNTY.md` for vision
2. Check `archive/bash-scripts/` for migration examples
3. Follow execution principles for new code
4. Submit PRs for bash script elimination

---

## ✅ Status Summary

**Production**: ✅ Validated and ready  
**Boot System**: ✅ BootLogger Phase 1 complete  
**P2P Coordination**: ✅ 5 demos validated  
**Pure Rust Evolution**: ⏳ 21.4% complete (on track)

**Code Quality**: A+  
**Type Safety**: 100% in new code  
**Unsafe Blocks**: 0 in new code  
**Documentation**: Comprehensive

---

**BiomeOS**: From bash to pure Rust sovereignty! 🦀✨
