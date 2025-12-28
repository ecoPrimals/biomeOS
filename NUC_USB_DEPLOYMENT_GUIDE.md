# NUC USB Bootable BiomeOS - Complete Guide

**Deploy BiomeOS on NUC hardware, connect to federated VM**

---

## 🎯 Goal

Create a **bootable USB drive** that:
1. Boots on Intel NUC hardware
2. Runs BiomeOS with all primals
3. Connects to a federated VM node
4. Demonstrates complete 3-tier deployment pipeline

---

## 🏗️ Architecture

```
┌──────────────────────────────────────────────────────┐
│  Tier 1: Development (Local Machine)                 │
│  - Build BiomeOS                                     │
│  - Run tests                                         │
│  - Create showcases                                  │
└────────────────────┬─────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────┐
│  Tier 2: Validation (benchScale VM)                  │
│  - Multi-VM testing                                  │
│  - Federation validation                             │
│  - Performance benchmarking                          │
└────────────────────┬─────────────────────────────────┘
                     │
                     ▼
┌──────────────────────────────────────────────────────┐
│  Tier 3: Production (NUC USB) ← WE ARE HERE          │
│  - Bootable USB drive                                │
│  - Real hardware deployment                          │
│  - Connect to federated VM                           │
└──────────────────────────────────────────────────────┘
```

---

## 📦 What Gets Packaged

### BiomeOS Core
- `biomeos-core` - Runtime coordination
- `biomeos-types` - Type definitions
- `biomeos-manifest` - Niche definitions
- `biomeos-boot` - Boot sequence
- `biomeos-system` - System integration

### Primals (Binaries)
- `nestgate` - Storage & sovereignty
- `beardog` - Cryptography & lineage
- `songbird` - Federation & discovery
- `toadstool` - Compute orchestration
- `petal-tongue` - UI (optional)

### Niches (Configurations)
- `rootpulse-niche.yaml` - Emergent version control
- Custom niches (user-defined)

### Scripts
- `deploy-real-primals.sh` - Primal startup
- `start-songbird.sh` - Federation discovery
- `showcase/common/discovery.sh` - Runtime discovery

---

## 🛠️ Prerequisites

### Development Machine
```bash
# Ubuntu/Debian
sudo apt install \
  debootstrap \
  squashfs-tools \
  xorriso \
  isolinux \
  syslinux-efi \
  grub-pc-bin \
  grub-efi-amd64-bin \
  mtools

# Or Arch/Manjaro
sudo pacman -S \
  debootstrap \
  squashfs-tools \
  libisoburn \
  syslinux \
  grub \
  mtools
```

### NUC Hardware
- Intel NUC (any generation)
- USB 3.0 drive (16GB+ recommended)
- Network connection (for federation)

### VM Node (for federation testing)
- Ubuntu 22.04 VM
- BiomeOS installed
- Accessible IP address

---

## 🚀 Quick Start (5 Steps)

### Step 1: Build BiomeOS
```bash
cd biomeOS/
cargo build --release --workspace
```

### Step 2: Collect Primal Binaries
```bash
# Copy primals to local directory
cp ../../../primalBins/nestgate primals/
cp ../../../primalBins/beardog primals/
cp ../../../primalBins/songbird primals/
cp ../../../primalBins/toadstool primals/
cp ../../../primalBins/petal-tongue primals/

# Verify
ls -lh primals/
```

### Step 3: Create Bootable USB
```bash
# Run USB creation script
sudo ./create-nuc-usb.sh /dev/sdX  # Replace sdX with your USB drive

# WARNING: This will ERASE the USB drive!
```

### Step 4: Boot NUC from USB
```bash
# 1. Insert USB into NUC
# 2. Power on NUC
# 3. Press F10 (or F2/Del) for boot menu
# 4. Select USB drive
# 5. BiomeOS will boot automatically
```

### Step 5: Connect to Federated VM
```bash
# On NUC (after boot):
# BiomeOS discovers federated nodes automatically via Songbird

# Check federation status
curl http://localhost:2300/federation/peers

# Should show your VM node!
```

---

## 📝 Detailed Instructions

### A. Prepare Base System

```bash
#!/bin/bash
# Create minimal Ubuntu base system

WORK_DIR="/tmp/biomeos-usb"
mkdir -p "$WORK_DIR"

# Debootstrap Ubuntu 22.04 minimal
sudo debootstrap \
  --variant=minbase \
  --include=systemd,network-manager,curl,ca-certificates \
  jammy \
  "$WORK_DIR/rootfs" \
  http://archive.ubuntu.com/ubuntu/

# Configure system
sudo chroot "$WORK_DIR/rootfs" /bin/bash <<EOF
# Set hostname
echo "biomeos-nuc" > /etc/hostname

# Configure network (DHCP)
cat > /etc/netplan/01-dhcp.yaml <<NETPLAN
network:
  version: 2
  ethernets:
    all:
      match:
        name: en*
      dhcp4: true
NETPLAN

# Enable services
systemctl enable systemd-networkd
systemctl enable systemd-resolved
EOF
```

### B. Install BiomeOS

```bash
# Copy BiomeOS files
sudo mkdir -p "$WORK_DIR/rootfs/opt/biomeos"

# Binaries
sudo cp -r target/release/biomeos* "$WORK_DIR/rootfs/opt/biomeos/"
sudo cp -r primals/ "$WORK_DIR/rootfs/opt/biomeos/"

# Configurations
sudo cp -r niches/ "$WORK_DIR/rootfs/opt/biomeos/"
sudo cp -r showcase/ "$WORK_DIR/rootfs/opt/biomeos/"

# Scripts
sudo cp deploy-real-primals.sh "$WORK_DIR/rootfs/opt/biomeos/"
sudo cp start-songbird.sh "$WORK_DIR/rootfs/opt/biomeos/"
sudo chmod +x "$WORK_DIR/rootfs/opt/biomeos/"*.sh
```

### C. Configure Auto-Start

```bash
# Create systemd service
sudo cat > "$WORK_DIR/rootfs/etc/systemd/system/biomeos.service" <<EOF
[Unit]
Description=BiomeOS Runtime
After=network.target

[Service]
Type=forking
ExecStart=/opt/biomeos/deploy-real-primals.sh
WorkingDirectory=/opt/biomeos
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Enable service
sudo chroot "$WORK_DIR/rootfs" systemctl enable biomeos.service
```

### D. Create Bootable Image

```bash
# Create squashfs
sudo mksquashfs \
  "$WORK_DIR/rootfs" \
  "$WORK_DIR/filesystem.squashfs" \
  -comp xz -b 1M

# Create ISO structure
mkdir -p "$WORK_DIR/iso"/{casper,boot/grub}

# Copy kernel and initrd
sudo cp "$WORK_DIR/rootfs/boot/vmlinuz-*" "$WORK_DIR/iso/casper/vmlinuz"
sudo cp "$WORK_DIR/rootfs/boot/initrd.img-*" "$WORK_DIR/iso/casper/initrd"
sudo cp "$WORK_DIR/filesystem.squashfs" "$WORK_DIR/iso/casper/"

# Create GRUB config
cat > "$WORK_DIR/iso/boot/grub/grub.cfg" <<EOF
set timeout=5
set default=0

menuentry "BiomeOS - Production" {
    linux /casper/vmlinuz boot=casper quiet splash ---
    initrd /casper/initrd
}

menuentry "BiomeOS - Debug" {
    linux /casper/vmlinuz boot=casper debug ---
    initrd /casper/initrd
}
EOF

# Create ISO
sudo grub-mkrescue -o "$WORK_DIR/biomeos.iso" "$WORK_DIR/iso/"
```

### E. Write to USB

```bash
# Write ISO to USB drive
sudo dd if="$WORK_DIR/biomeos.iso" of=/dev/sdX bs=4M status=progress

# Sync
sudo sync

echo "✅ Bootable USB created!"
```

---

## 🔬 Federation Testing

### Setup VM Node

```bash
# On your VM (benchScale or cloud)
cd biomeOS/
./deploy-real-primals.sh

# Note the VM's IP address
ip addr show | grep "inet "
# Example: 192.168.1.100
```

### Boot NUC and Connect

```bash
# NUC boots and starts BiomeOS automatically

# Check Songbird discovery
curl http://localhost:2300/health

# Check discovered peers
curl http://localhost:2300/federation/peers

# Should show your VM!
{
  "peers": [
    {
      "id": "tower-vm-1",
      "address": "192.168.1.100:2300",
      "status": "healthy"
    }
  ]
}
```

### Test Federation

```bash
# On NUC: Deploy RootPulse niche
cd /opt/biomeos
biomeos niche deploy niches/rootpulse/rootpulse-niche.yaml

# Create a repository
cd /tmp/test-repo
rootpulse init
echo "Hello from NUC!" > README.md
rootpulse commit -m "First commit from hardware"

# Push to federation
rootpulse push federation://tower-vm-1/test-repo

# On VM: Clone and verify
cd /tmp
rootpulse clone federation://tower-nuc/test-repo
cat test-repo/README.md
# Output: "Hello from NUC!"
```

---

## 🎯 Success Criteria

### Boot Sequence
- ✅ USB boots on NUC
- ✅ BiomeOS starts automatically
- ✅ Network configured (DHCP)
- ✅ All primals start

### Primal Discovery
- ✅ NestGate discovered
- ✅ BearDog discovered
- ✅ Songbird discovered
- ✅ Toadstool discovered

### Federation
- ✅ Songbird discovers VM node
- ✅ P2P connection established
- ✅ Data can be pushed/pulled
- ✅ Lineage verified

---

## 🐛 Troubleshooting

### USB Won't Boot
```bash
# Check UEFI/Legacy BIOS settings
# Ensure "USB Boot" is enabled
# Try both UEFI and Legacy modes
```

### Network Not Working
```bash
# Boot into debug mode (GRUB menu)
# Check network interfaces
ip link show

# Manually configure if needed
sudo dhclient enp0s3
```

### Primals Not Starting
```bash
# Check logs
journalctl -u biomeos.service -f

# Manual start
cd /opt/biomeos
./deploy-real-primals.sh
```

### Can't Find Federation Peers
```bash
# Check Songbird
curl http://localhost:2300/health

# Check network connectivity to VM
ping 192.168.1.100

# Check firewall
sudo ufw status
```

---

## 📊 Status

### Complete ✅
- [x] Architecture designed
- [x] Prerequisites documented
- [x] Base system creation
- [x] BiomeOS installation
- [x] Auto-start configuration
- [x] Federation testing plan

### In Progress 🔄
- [ ] USB creation script (`create-nuc-usb.sh`)
- [ ] ISO build automation
- [ ] Hardware testing
- [ ] Federation validation

### Next Steps 📋
1. Enhance `create-nuc-usb.sh` with full automation
2. Test on real NUC hardware
3. Validate federation with VM
4. Document hardware compatibility
5. Create video walkthrough

---

## 🎓 Key Insights

### Why USB Boot?
- **Portable**: BiomeOS on any NUC
- **Immutable**: USB is read-only (security)
- **Reproducible**: Same image everywhere
- **Hardware**: Real production testing

### Why Federation?
- **Realistic**: Multi-node scenarios
- **P2P**: True decentralization
- **Validation**: Complete stack testing
- **Production**: What users will deploy

---

**Status**: Guide complete, automation next  
**Next**: Enhance USB creation script  
**Goal**: One-command USB creation  

💾 **BiomeOS: From Code to Hardware to Federation!**

