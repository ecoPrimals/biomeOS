#!/usr/bin/env bash
# BiomeOS ISO Testing - QEMU Boot Verification

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧪 BiomeOS ISO Boot Test - QEMU"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Detect sudo method
if command -v pkexec &> /dev/null; then
    SUDO="pkexec"
else
    SUDO="sudo"
fi

# Check for QEMU
if ! command -v qemu-system-x86_64 &> /dev/null; then
    echo "Installing QEMU..."
    $SUDO apt update && $SUDO apt install -y qemu-system-x86 qemu-utils
    echo ""
fi

# Find the ISO
ISO_FILE=$(ls -t /home/eastgate/Development/ecoPrimals/phase2/biomeOS/dist/biomeos-*.iso 2>/dev/null | head -1)

if [ -z "$ISO_FILE" ]; then
    echo "❌ No ISO found in dist/"
    exit 1
fi

echo "✅ ISO found: $ISO_FILE"
ISO_SIZE=$(du -h "$ISO_FILE" | cut -f1)
echo "   Size: $ISO_SIZE"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Booting BiomeOS in QEMU"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Configuration:"
echo "  • Memory: 2GB RAM"
echo "  • CPU: Host CPU with KVM acceleration"
echo "  • Display: GTK window"
echo "  • Boot: CD-ROM (ISO)"
echo "  • Network: User mode (NAT)"
echo ""
echo "What to expect:"
echo "  1. GRUB menu with 3 options"
echo "  2. Select 'BiomeOS - Sovereignty-First Operating System'"
echo "  3. Kernel boot messages"
echo "  4. BiomeOS init startup"
echo "  5. Shell prompt or interface"
echo ""
echo "Press Ctrl+Alt+G to release mouse from QEMU window"
echo "Press Ctrl+Alt+F to toggle fullscreen"
echo "Press Ctrl+C in terminal to stop QEMU"
echo ""
read -p "Press Enter to start QEMU boot test..."
echo ""

# Boot the ISO in QEMU
qemu-system-x86_64 \
    -cdrom "$ISO_FILE" \
    -m 2048 \
    -enable-kvm \
    -cpu host \
    -smp 2 \
    -display gtk \
    -serial stdio \
    -boot d \
    -name "BiomeOS Boot Test"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "QEMU session ended"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

