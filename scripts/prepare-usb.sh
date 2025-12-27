#!/usr/bin/env bash
# BiomeOS USB Preparation - Clean Slate
# Wipes USB and prepares for bootable ISO

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔧 BiomeOS USB Preparation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Detect sudo method
if command -v pkexec &> /dev/null; then
    SUDO="pkexec"
else
    SUDO="sudo"
fi

# Show available devices
echo "Available devices:"
lsblk -o NAME,SIZE,TYPE,MOUNTPOINT | grep -E "NAME|sd"
echo ""

# Get USB device
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
echo "⚠️  WARNING: This will COMPLETELY WIPE $USB_PATH!"
echo "⚠️  All data will be PERMANENTLY DELETED!"
echo ""
read -p "Are you absolutely sure? Type 'WIPE' to continue: " CONFIRM

if [ "$CONFIRM" != "WIPE" ]; then
    echo "Cancelled (you typed: $CONFIRM)"
    exit 0
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Preparing $USB_PATH for BiomeOS..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Step 1: Unmount any mounted partitions
echo "Step 1: Unmounting any mounted partitions..."
$SUDO umount ${USB_PATH}* 2>/dev/null || true
echo "  ✅ Unmounted"
echo ""

# Step 2: Wipe the beginning of the drive (partition table, etc)
echo "Step 2: Wiping partition table..."
$SUDO dd if=/dev/zero of=$USB_PATH bs=1M count=10 status=progress
$SUDO sync
echo "  ✅ Wiped"
echo ""

# Step 3: Create new partition table
echo "Step 3: Creating fresh partition table..."
$SUDO parted -s $USB_PATH mklabel gpt
echo "  ✅ GPT partition table created"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ USB Prepared!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Device: $USB_PATH"
echo "Status: Clean slate, ready for bootable ISO"
echo ""
echo "Next: Run the USB creation script"
echo "  ./scripts/create-bootable-usb.sh"
echo ""
echo "Or write ISO directly:"
echo "  ISO_FILE=dist/biomeos-20251226-231626.iso"
echo "  $SUDO dd if=\$ISO_FILE of=$USB_PATH bs=4M status=progress oflag=sync"
echo ""

