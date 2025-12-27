#!/usr/bin/env bash
# BiomeOS USB Bootable Creator
# Creates a bootable USB drive with BiomeOS
# Uses pkexec for GUI password prompts

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 BiomeOS USB Bootable Creator"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Detect sudo method (prefer pkexec for GUI)
if command -v pkexec &> /dev/null; then
    SUDO="pkexec"
    echo "Using pkexec for GUI password prompts"
else
    SUDO="sudo"
    echo "Using sudo for terminal password prompts"
fi
echo ""

# Step 1: Install xorriso
echo "Step 1: Installing dependencies..."
if ! command -v xorriso &> /dev/null; then
    echo "  Installing xorriso (GUI password prompt will appear)..."
    $SUDO apt update && $SUDO apt install -y xorriso
else
    echo "  ✅ xorriso already installed"
fi
echo ""

# Step 2: Prepare kernel
echo "Step 2: Preparing kernel..."
if [ ! -f "/tmp/vmlinuz-biomeos" ]; then
    echo "  Copying system kernel to accessible location (GUI password prompt)..."
    $SUDO cp /boot/vmlinuz /tmp/vmlinuz-biomeos
    $SUDO chmod 644 /tmp/vmlinuz-biomeos
    echo "  ✅ Kernel ready at /tmp/vmlinuz-biomeos"
else
    echo "  ✅ Kernel already prepared at /tmp/vmlinuz-biomeos"
fi
echo ""

# Step 3: Build BiomeOS ISO
echo "Step 3: Building BiomeOS bootable ISO..."
export BIOMEOS_KERNEL=/tmp/vmlinuz-biomeos
cargo run --release -p biomeos-boot --bin biomeos-mkboot -- iso
echo ""

# Step 4: Find the ISO
ISO_FILE=$(ls -t dist/biomeos-*.iso 2>/dev/null | head -1)
if [ -z "$ISO_FILE" ]; then
    ISO_FILE=$(ls -t dist/biomeos-*.tar.gz 2>/dev/null | head -1)
    if [ -z "$ISO_FILE" ]; then
        echo "❌ No ISO or tar.gz found in dist/"
        exit 1
    fi
    echo "⚠️  Found tar.gz instead of ISO (xorriso might have failed)"
    echo "   File: $ISO_FILE"
    echo ""
    echo "To create ISO from tar.gz:"
    echo "  mkdir -p /tmp/boot-content"
    echo "  tar -xzf $ISO_FILE -C /tmp/boot-content"
    echo "  grub-mkrescue -o dist/biomeos-final.iso /tmp/boot-content"
    exit 1
fi

echo "✅ ISO created: $ISO_FILE"
ISO_SIZE=$(du -h "$ISO_FILE" | cut -f1)
echo "   Size: $ISO_SIZE"
echo ""

# Step 5: Identify USB device
echo "Step 4: USB Device"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
lsblk -o NAME,SIZE,TYPE,MOUNTPOINT | grep -E "NAME|sd"
echo ""
echo "⚠️  WARNING: Writing to USB will DESTROY ALL DATA on the device!"
echo ""
read -p "Enter USB device (e.g., sda): " USB_DEVICE

if [ -z "$USB_DEVICE" ]; then
    echo "❌ No device specified"
    exit 1
fi

USB_PATH="/dev/$USB_DEVICE"
if [ ! -b "$USB_PATH" ]; then
    echo "❌ Device $USB_PATH not found"
    exit 1
fi

echo ""
echo "Ready to write to: $USB_PATH"
echo "ISO: $ISO_FILE ($ISO_SIZE)"
echo ""
read -p "Proceed? (yes/no): " CONFIRM

if [ "$CONFIRM" != "yes" ]; then
    echo "Cancelled"
    exit 0
fi

# Step 6: Write to USB
echo ""
echo "Step 5: Writing to USB (GUI password prompt)..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  This may take a few minutes..."
echo ""

$SUDO dd if="$ISO_FILE" of="$USB_PATH" bs=4M status=progress oflag=sync

echo ""
echo "✅ Write complete!"
echo ""
$SUDO sync
echo "✅ Sync complete!"
echo ""

# Step 7: Success
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎉 BiomeOS USB Created Successfully!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "USB Device: $USB_PATH"
echo "ISO: $ISO_FILE"
echo "Size: $ISO_SIZE"
echo ""
echo "Next Steps:"
echo "  1. Safely eject USB: $SUDO eject $USB_PATH"
echo "  2. Insert into NUC"
echo "  3. Boot from USB (F10/F12 for boot menu)"
echo "  4. Select 'BiomeOS - Sovereignty-First Operating System'"
echo ""
echo "Boot Options in GRUB Menu:"
echo "  • BiomeOS - Standard boot"
echo "  • BiomeOS - Discovery Mode (network discovery)"
echo "  • BiomeOS - Network Boot (network coordination)"
echo ""

