# BiomeOS USB Bootable Creation - Manual Steps

## Quick Reference Guide

Your USB: `/dev/sda` (116.1GB)

### Step 1: Install xorriso (one-time)

```bash
sudo apt update && sudo apt install -y xorriso
```

### Step 2: Prepare kernel (one-time)

```bash
sudo cp /boot/vmlinuz /tmp/vmlinuz-biomeos
sudo chmod 644 /tmp/vmlinuz-biomeos
```

### Step 3: Build BiomeOS ISO

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
export BIOMEOS_KERNEL=/tmp/vmlinuz-biomeos
cargo run --release -p biomeos-boot --bin biomeos-mkboot -- iso
```

This will create: `dist/biomeos-YYYYMMDD-HHMMSS.iso`

### Step 4: Write to USB

⚠️ **WARNING**: This will DESTROY ALL DATA on `/dev/sda`!

```bash
# Find the ISO
ISO_FILE=$(ls -t dist/biomeos-*.iso 2>/dev/null | head -1)
echo "ISO: $ISO_FILE"

# Write to USB (WILL ERASE /dev/sda!)
sudo dd if="$ISO_FILE" of=/dev/sda bs=4M status=progress oflag=sync

# Sync and eject
sudo sync
sudo eject /dev/sda
```

### Step 5: Boot on NUC

1. Insert USB into NUC
2. Power on and press F10 or F12 for boot menu
3. Select USB drive
4. Choose from GRUB menu:
   - BiomeOS - Sovereignty-First Operating System
   - BiomeOS - Discovery Mode
   - BiomeOS - Network Boot

---

## Troubleshooting

### If ISO creation fails (tar.gz instead)

If you get `biomeos-*.tar.gz` instead of `.iso`, xorriso failed. Convert it:

```bash
# Extract tar.gz
mkdir -p /tmp/boot-content
tar -xzf dist/biomeos-*.tar.gz -C /tmp/boot-content

# Create ISO with grub-mkrescue
grub-mkrescue -o dist/biomeos-final.iso /tmp/boot-content

# Then write to USB
sudo dd if=dist/biomeos-final.iso of=/dev/sda bs=4M status=progress oflag=sync
```

### Check USB after writing

```bash
# Verify it's bootable
sudo fdisk -l /dev/sda | head -20

# Should show hybrid MBR/GPT partitions
```

---

## Expected Timeline

- Step 1 (install xorriso): 1-2 minutes
- Step 2 (prepare kernel): 10 seconds
- Step 3 (build ISO): 30-60 seconds
- Step 4 (write to USB): 2-5 minutes
- **Total**: ~5-10 minutes

---

## What's in the ISO

- **Kernel**: Linux kernel (system or custom)
- **Initramfs**: BiomeOS init + binaries (1.73 MB)
- **GRUB**: Bootloader with 3 boot options
- **Phase 1 Primals**: All stable primal binaries
- **Templates**: BYOB YAML configs

Total size: ~50-100 MB (depending on phase1bins)

---

## After Boot

BiomeOS will:
1. Load kernel
2. Mount initramfs
3. Start `biomeos-init` (PID 1)
4. Detect hardware
5. Configure network
6. Start primal services
7. Present terminal or GUI

---

## Safety Notes

⚠️ **Double-check device name!**
```bash
lsblk  # Verify /dev/sda is your USB, not your main drive!
```

⚠️ **Backup important data first!**

✅ **After writing, safely eject before removing**:
```bash
sudo sync && sudo eject /dev/sda
```

