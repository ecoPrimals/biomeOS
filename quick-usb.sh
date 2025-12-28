#!/bin/bash
# Quick USB Creation - Build ISO and write to /dev/sda1
# Run with: ./quick-usb.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
USB_DEVICE="${USB_DEVICE:-/dev/sda1}"

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  💾 BiomeOS Quick USB Creation 💾                       ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

echo "Target: $USB_DEVICE"
echo ""

if [ "$AUTO_CONFIRM" = "1" ]; then
    echo "✅ Auto-confirmed (AUTO_CONFIRM=1)"
    CONFIRM="yes"
else
    echo "⚠️  This will format $USB_DEVICE"
    echo "Type 'yes' to continue (or set AUTO_CONFIRM=1):"
    read -r CONFIRM
fi

if [ "$CONFIRM" != "yes" ]; then
    echo "Aborted"
    exit 1
fi

echo ""
echo "🔨 Building BiomeOS..."
cd "$SCRIPT_DIR"
cargo build --release --workspace --quiet
echo "✅ Build complete"
echo ""

echo "📦 Creating deployment package..."
ISO_DIR="iso-build"
rm -rf "$ISO_DIR"
mkdir -p "$ISO_DIR"/{opt/biomeos,install}

# Copy everything
echo "   Copying files..."
cp -r target/release/biomeos* "$ISO_DIR/opt/biomeos/" 2>/dev/null || true
cp -r primals "$ISO_DIR/opt/biomeos/"
cp -r niches "$ISO_DIR/opt/biomeos/"
cp -r showcase "$ISO_DIR/opt/biomeos/"
cp *.sh "$ISO_DIR/opt/biomeos/" 2>/dev/null || true
chmod +x "$ISO_DIR/opt/biomeos/"*.sh 2>/dev/null || true

# Create install script
cat > "$ISO_DIR/install/install-biomeos.sh" << 'EOF'
#!/bin/bash
set -e
echo "🚀 Installing BiomeOS..."
sudo cp -r ./opt/biomeos /opt/
sudo chown -R $USER:$USER /opt/biomeos
echo "✅ Installed to /opt/biomeos"
echo ""
echo "To start:"
echo "  cd /opt/biomeos"
echo "  ./deploy-real-primals.sh"
EOF

chmod +x "$ISO_DIR/install/install-biomeos.sh"

# Create README
cat > "$ISO_DIR/README.txt" << 'EOF'
BiomeOS USB Package

To Install:
  cd install
  ./install-biomeos.sh

To Run:
  cd /opt/biomeos
  ./deploy-real-primals.sh

To Test:
  cd /opt/biomeos
  ./run-e2e-tests.sh

Contents:
- BiomeOS core
- 5 Primals
- RootPulse niche
- 20 Showcases
EOF

echo "✅ Package ready"
echo ""

echo "💾 Creating tar archive..."
TAR_FILE="biomeos-$(date +%Y%m%d-%H%M%S).tar.gz"
tar -czf "$TAR_FILE" -C "$ISO_DIR" .
TAR_SIZE=$(du -h "$TAR_FILE" | cut -f1)
echo "✅ Created: $TAR_FILE ($TAR_SIZE)"
echo ""

echo "🔄 Formatting USB..."
sudo mkfs.ext4 -F -L "BiomeOS" "$USB_DEVICE"
echo "✅ Formatted"
echo ""

echo "📁 Mounting USB..."
MOUNT_POINT="/mnt/biomeos-usb"
sudo mkdir -p "$MOUNT_POINT"
sudo mount "$USB_DEVICE" "$MOUNT_POINT"
echo "✅ Mounted at $MOUNT_POINT"
echo ""

echo "📤 Copying to USB..."
sudo tar -xzf "$TAR_FILE" -C "$MOUNT_POINT"
sudo sync
echo "✅ Copied"
echo ""

echo "🔓 Unmounting..."
sudo umount "$MOUNT_POINT"
echo "✅ Done"
echo ""

echo "═══════════════════════════════════════════════════════════"
echo "✅ USB READY"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "USB Device: $USB_DEVICE"
echo "Archive: $TAR_FILE ($TAR_SIZE)"
echo ""
echo "🧪 Test in VM:"
echo "1. Attach USB to VM"
echo "2. Mount: sudo mkdir /mnt/usb && sudo mount /dev/sdb1 /mnt/usb"
echo "3. Install: cd /mnt/usb/install && ./install-biomeos.sh"
echo "4. Run: cd /opt/biomeos && ./deploy-real-primals.sh"
echo "5. Test: ./run-e2e-tests.sh"
echo ""

