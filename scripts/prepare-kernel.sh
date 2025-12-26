#!/usr/bin/env bash
# Helper script to make kernel accessible for BiomeOS boot image creation

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔑 BiomeOS Kernel Preparation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Detect kernel
KERNEL_PATH=$(readlink -f /boot/vmlinuz)

if [ ! -f "$KERNEL_PATH" ]; then
    echo "❌ Kernel not found at $KERNEL_PATH"
    exit 1
fi

echo "📍 Found kernel: $KERNEL_PATH"
echo "📊 Size: $(du -h "$KERNEL_PATH" | cut -f1)"
echo ""

# Copy to accessible location
ACCESSIBLE_KERNEL="/tmp/vmlinuz-biomeos"

echo "📋 Copying kernel to accessible location..."
sudo cp "$KERNEL_PATH" "$ACCESSIBLE_KERNEL"
sudo chmod 644 "$ACCESSIBLE_KERNEL"

echo "✅ Kernel ready at: $ACCESSIBLE_KERNEL"
echo ""
echo "Now you can build without sudo:"
echo "  export BIOMEOS_KERNEL=$ACCESSIBLE_KERNEL"
echo "  cargo run --release -p biomeos-boot --bin biomeos-mkboot -- iso"
echo ""

