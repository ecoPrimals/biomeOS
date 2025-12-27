# BiomeOS Deployment Strategy - benchScale First, NUC Last

**Date**: December 27, 2025  
**Strategy**: Validate everything in benchScale VMs before NUC deployment  
**Assumption**: NUC deployment will be remote/hands-off (no troubleshooting access)

---

## 🎯 Strategy Overview

### **Phase 1: benchScale VM Federation** (4-5 days)
Validate complete BiomeOS deployment using QEMU VMs orchestrated by benchScale

### **Phase 2: NUC Final Validation** (1-2 days)
Deploy validated artifacts to physical NUCs as final production test

---

## 📋 Phase 1: benchScale VM Validation

### **Goal**: Get primals running inside BiomeOS VMs, fully validated

### **Available Resources**:
✅ Primal binaries (Phase 1 bins available, can request fresh ones)  
✅ benchScale infrastructure (validated)  
✅ Boot infrastructure (working)  
✅ Storage options (segregate drive or USB NVMe)

### **What We'll Build**:

#### **1. Root Filesystem with Primals** (Day 1-2)
```bash
# Create base root filesystem
./scripts/create-biomeos-rootfs.sh \
  --size 8G \
  --primals ../phase1bins/ \
  --output vm-testing/biomeos-root.img

# What it includes:
# - BiomeOS init system
# - All 5 primal binaries
# - Service management (custom or systemd)
# - Configuration files
# - Network tools
# - Persistent /var, /home, /etc
```

**Deliverables**:
- Root filesystem image builder
- Primal installation script
- Service configuration templates
- Network configuration

**Testing**:
- Single VM boots with primals
- Primals start automatically
- Services are healthy
- Logs are accessible

---

#### **2. benchScale VM Backend** (Day 2-3)
Extend benchScale to support "BiomeOS VM" as a backend (not just Docker)

```yaml
# topologies/biomeos-federation.yaml
topology:
  name: biomeos-3node
  backend: biomeos_vm  # New backend type
  
nodes:
  - name: node1
    image: biomeos-root.img
    memory: 2G
    vcpus: 2
    network: biome-net
    primals:
      - songbird
      - beardog
    
  - name: node2
    image: biomeos-root.img
    memory: 2G
    vcpus: 2
    network: biome-net
    primals:
      - nestgate
      - toadstool
    
  - name: node3
    image: biomeos-root.img
    memory: 2G
    vcpus: 2
    network: biome-net
    primals:
      - squirrel

network:
  type: bridge
  name: biome-net
  subnet: 10.42.0.0/24
```

**Deliverables**:
- benchScale BiomeOS VM backend
- VM lifecycle management (start, stop, monitor)
- Network bridge setup
- Serial console capture
- Health monitoring integration

**Testing**:
- benchScale creates VMs
- VMs boot and join network
- Serial logs captured
- Health checks pass

---

#### **3. Primal Service Orchestration** (Day 3-4)
Get primals running and coordinating inside VMs

**Service Manager Options**:

**Option A: Custom Rust Service Manager** (sovereignty-first)
```rust
// crates/biomeos-service-manager/
// - Pure Rust
// - Integrated with BootLogger
// - Primal-aware (capabilities)
// - Dependency management
```

**Option B: systemd** (pragmatic)
```ini
# /etc/systemd/system/songbird.service
[Unit]
Description=Songbird Service Mesh
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/songbird tower --port 9999
Restart=always
RestartSec=5s

[Install]
WantedBy=multi-user.target
```

**Recommendation**: Start with systemd (faster), evolve to custom later

**Deliverables**:
- Service unit files for all primals
- Startup order management
- Health check integration
- Log aggregation
- Failure recovery

**Testing**:
- All primals start on boot
- Dependencies respected (Songbird first)
- Services register with each other
- P2P coordination works
- Failures trigger restarts

---

#### **4. Inter-VM Networking** (Day 4)
Bridge networking for real P2P communication

```bash
# Setup script
./scripts/setup-biomeos-network.sh \
  --bridge biome-br0 \
  --subnet 10.42.0.0/24 \
  --vms 3

# Each VM gets:
# - Static IP (10.42.0.10, .11, .12)
# - DNS configuration
# - Route to other VMs
# - Bridge to host
```

**Deliverables**:
- Bridge network setup
- IP address management
- DNS/hosts configuration
- Network validation tests

**Testing**:
- VMs can ping each other
- Songbird discovers services across VMs
- BTSP tunnels establish
- BirdSong encryption works
- NAT traversal functions

---

#### **5. Full Integration Test** (Day 5)
Complete validation using benchScale

```bash
# Run full test suite
cargo run --example biomeos_vm_federation_test

# Test sequence:
# 1. Create 3-node BiomeOS VM topology
# 2. Boot all VMs
# 3. Wait for primals to start
# 4. Test service discovery
# 5. Test P2P coordination
# 6. Test failure scenarios
# 7. Verify recovery
# 8. Validate logs
# 9. Clean shutdown
```

**Success Criteria**:
- ✅ All VMs boot (< 200ms each)
- ✅ All primals running (5/5)
- ✅ Service mesh established
- ✅ P2P coordination working
- ✅ Logs captured and structured
- ✅ Failure recovery validated
- ✅ Clean shutdown successful

---

## 📋 Phase 2: NUC Deployment

### **Goal**: Deploy validated BiomeOS to physical NUCs (hands-off)

### **Pre-requisites from Phase 1**:
- ✅ Validated root filesystem image
- ✅ Working primal configuration
- ✅ Network configuration tested
- ✅ Service orchestration proven
- ✅ Failure recovery validated

### **NUC Deployment Process**:

#### **1. Create Deployment Artifacts** (Day 6)
```bash
# Build final ISO with validated components
cargo run --release -p biomeos-boot --bin biomeos-mkboot -- \
  --root-fs vm-testing/biomeos-root.img \
  --output dist/biomeos-production.iso

# Create USB installer
sudo dd if=dist/biomeos-production.iso \
  of=/dev/sdX \
  bs=4M \
  status=progress
```

#### **2. NUC Storage Strategy**

**Option A: Partition Internal Drive**
```bash
# Reserve 20GB for BiomeOS
# - /boot: 1GB (kernel, initramfs)
# - /: 10GB (root filesystem)
# - /var: 9GB (primal data, logs)
```

**Option B: USB NVMe (Recommended)**
```bash
# Dedicated BiomeOS drive
# - Clean separation from host OS
# - Easy to swap/upgrade
# - Better for testing
# - 32GB+ recommended
```

**Recommendation**: USB NVMe for initial deployment (safer, easier to recover)

#### **3. NUC Deployment Checklist**

**Pre-deployment**:
- [ ] Create recovery USB (in case of issues)
- [ ] Document NUC network setup
- [ ] Test UEFI boot order
- [ ] Backup any existing data
- [ ] Document serial numbers

**Deployment**:
- [ ] Insert USB NVMe (if using)
- [ ] Insert BiomeOS USB installer
- [ ] Boot from USB
- [ ] Automated installation runs
- [ ] First boot completes
- [ ] Services start
- [ ] Register with network

**Validation**:
- [ ] Can reach NUC via network
- [ ] Serial logs accessible (if serial console)
- [ ] All primals running
- [ ] Service mesh formed
- [ ] P2P coordination working
- [ ] Logs being captured

**Remote Troubleshooting** (if needed):
```bash
# SSH into NUC (if network is up)
ssh biomeos@nuc-01.local

# Check service status
systemctl status songbird beardog nestgate

# View logs
journalctl -u songbird -f

# Check networking
ip addr show
ping 10.42.0.11

# Emergency recovery
# (reboot from USB installer)
```

---

## 🗓️ Detailed Timeline

### **Week 1: benchScale Validation**

**Day 1-2: Root Filesystem**
- Create root FS builder script
- Install primals and dependencies
- Configure services
- Test single VM boot

**Day 3: benchScale VM Backend**
- Implement BiomeOS VM backend
- VM lifecycle management
- Network bridge setup
- Integration with existing benchScale

**Day 4: Service Orchestration**
- Service manager setup (systemd initially)
- Primal startup scripts
- Health check integration
- Log aggregation

**Day 5: Networking & Integration**
- Bridge networking
- IP management
- Full 3-node test
- P2P validation

**End of Week 1**: 
✅ Complete BiomeOS federation running in benchScale VMs
✅ All primals coordinating
✅ Network validated
✅ Ready for NUC deployment

### **Week 2: NUC Deployment**

**Day 6: Artifacts & Preparation**
- Build production ISO
- Create USB installer
- USB NVMe preparation (if using)
- Documentation

**Day 7: NUC Deployment**
- Deploy to first NUC
- Validate functionality
- Deploy to remaining NUCs
- Form federation

**End of Week 2**:
✅ BiomeOS running on physical NUCs
✅ Federation established
✅ Production validation complete

---

## 🎯 Success Metrics

### **Phase 1 (benchScale) Success**:
- [ ] 3 VMs running BiomeOS
- [ ] All 5 primals operational
- [ ] < 200ms boot time
- [ ] Service mesh established
- [ ] P2P coordination validated
- [ ] Failure recovery tested
- [ ] Logs captured via BootLogger

### **Phase 2 (NUC) Success**:
- [ ] BiomeOS boots on NUC hardware
- [ ] All drivers functional
- [ ] Network connectivity established
- [ ] Primals running
- [ ] Remote monitoring working
- [ ] No hands-on intervention needed

---

## 🚀 Next Actions (Priority Order)

### **Immediate (Today/Tomorrow)**:

1. **Create Root Filesystem Builder**
   ```bash
   # New script: scripts/create-biomeos-rootfs.sh
   # - Format ext4 filesystem
   # - Install base system
   # - Copy primal binaries
   # - Configure services
   # - Set up networking
   ```

2. **Install Primals into Root FS**
   ```bash
   # Use existing phase1bins
   # Copy to /usr/local/bin/
   # Create service files
   # Configure startup
   ```

3. **Test Single VM Boot**
   ```bash
   # Boot VM with new root FS
   # Verify primals present
   # Test manual startup
   ```

### **This Week**:

4. **benchScale VM Backend**
5. **Service Orchestration**
6. **Network Bridge Setup**
7. **Full Integration Test**

### **Next Week**:

8. **NUC Preparation**
9. **Deployment**
10. **Validation**

---

## 💾 Storage Recommendation

### **For benchScale Testing**:
- Use existing disk space
- 8GB per VM (24GB total for 3 VMs)
- Can use sparse files (only use what's needed)

### **For NUC Deployment**:
**Recommended: USB NVMe (32GB+)**

**Advantages**:
- ✅ Clean separation from host OS
- ✅ Easy to swap/upgrade
- ✅ Safe for testing (won't break host)
- ✅ Portable between NUCs
- ✅ Fast (NVMe speeds)

**Partition Layout**:
```
/dev/nvme0n1
├── nvme0n1p1: 1GB   /boot (EFI)
├── nvme0n1p2: 15GB  /     (root)
└── nvme0n1p3: 16GB  /var  (data/logs)
```

---

## 🎯 The Path Forward

### **Our Advantages**:
1. ✅ Boot infrastructure proven (BootLogger Phase 1)
2. ✅ benchScale validated (integration testing)
3. ✅ Primal binaries available
4. ✅ NUCs on hand for final validation
5. ✅ Storage flexibility (USB NVMe option)

### **Our Strategy**:
1. **Validate in benchScale first** (low risk, fast iteration)
2. **Treat NUC deployment as production** (hands-off)
3. **Use USB NVMe** (safer, easier recovery)
4. **Full testing before hardware** (no surprises)

### **Timeline**:
- **Week 1**: benchScale validation (5 days)
- **Week 2**: NUC deployment (2 days)
- **Total**: 7 days to production-ready NUC federation

---

## 🎊 What This Achieves

**End State**:
- ✅ 3 physical NUCs running BiomeOS
- ✅ All 5 Phase 1 primals operational
- ✅ P2P federation established
- ✅ Remote monitoring working
- ✅ Validated, reproducible deployment
- ✅ Ready for production workloads

**Bonus**:
- ✅ Deployment pipeline proven
- ✅ Can replicate to more NUCs
- ✅ Can deploy to cloud (same artifacts)
- ✅ Full observability maintained

---

**Summary**: We use benchScale to de-risk everything, then deploy to NUCs with confidence. USB NVMe gives us safety and flexibility. 7-day timeline to complete BiomeOS federation on physical hardware.

**Next Step**: Create root filesystem builder script and start Phase 1.

---

**BiomeOS: benchScale First. NUC Last. Production Ready.** 🦀✨

