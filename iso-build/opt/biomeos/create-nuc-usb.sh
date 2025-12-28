#!/bin/bash
# NUC USB Deployment Pipeline - Create bootable BiomeOS USB

set -e

echo "💾 NUC USB Deployment Pipeline"
echo "==============================="
echo ""

# Configuration
BIOMEOS_DIR="${BIOMEOS_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}"
ISO_OUTPUT="${ISO_OUTPUT:-biomeos-nuc-$(date +%Y%m%d).iso}"
USB_DEVICE="${USB_DEVICE:-}"

echo "📋 Configuration:"
echo "   BiomeOS: $BIOMEOS_DIR"
echo "   ISO Output: $ISO_OUTPUT"
[ -n "$USB_DEVICE" ] && echo "   USB Device: $USB_DEVICE"
echo ""

# Check dependencies
echo "🔍 Checking dependencies..."
MISSING_DEPS=()

for cmd in cargo mkisofs dd; do
    if ! command -v $cmd &> /dev/null; then
        MISSING_DEPS+=("$cmd")
    fi
done

if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
    echo "❌ Missing dependencies: ${MISSING_DEPS[*]}"
    echo ""
    echo "💡 Install with:"
    echo "   sudo apt install build-essential genisoimage coreutils"
    exit 1
fi

echo "✅ All dependencies available"
echo ""

# Build BiomeOS for x86_64 (NUC target)
echo "🔨 Building BiomeOS for x86_64-unknown-linux-gnu..."
cd "$BIOMEOS_DIR"

if cargo build --release --target x86_64-unknown-linux-gnu --quiet; then
    echo "✅ BiomeOS built for NUC"
else
    echo "❌ Build failed"
    exit 1
fi

echo ""

# Create ISO structure
echo "📦 Creating ISO structure..."
ISO_DIR="$BIOMEOS_DIR/iso-build"
rm -rf "$ISO_DIR"
mkdir -p "$ISO_DIR"/{boot,opt/biomeos,etc/systemd/system}

# Copy binaries
echo "   Copying binaries..."
cp -r target/x86_64-unknown-linux-gnu/release/biomeos-* "$ISO_DIR/opt/biomeos/" 2>/dev/null || true
cp -r primals "$ISO_DIR/opt/biomeos/" 2>/dev/null || true
cp -r showcase "$ISO_DIR/opt/biomeos/"
cp deploy-real-primals.sh "$ISO_DIR/opt/biomeos/" 2>/dev/null || true
cp run-e2e-tests.sh "$ISO_DIR/opt/biomeos/"

# Create systemd service
echo "   Creating systemd service..."
cat > "$ISO_DIR/etc/systemd/system/biomeos.service" << 'EOF'
[Unit]
Description=BiomeOS Primal Coordination Substrate
After=network.target

[Service]
Type=simple
User=biomeos
WorkingDirectory=/opt/biomeos
ExecStart=/opt/biomeos/deploy-real-primals.sh
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Create install script
echo "   Creating install script..."
cat > "$ISO_DIR/install-biomeos.sh" << 'EOF'
#!/bin/bash
set -e

echo "🚀 Installing BiomeOS..."

# Create user
useradd -m -s /bin/bash biomeos || true

# Copy files
cp -r /mnt/opt/biomeos /opt/
chown -R biomeos:biomeos /opt/biomeos

# Install systemd service
cp /mnt/etc/systemd/system/biomeos.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable biomeos.service

echo "✅ BiomeOS installed"
echo ""
echo "Next steps:"
echo "  sudo systemctl start biomeos"
echo "  sudo journalctl -u biomeos -f"
EOF

chmod +x "$ISO_DIR/install-biomeos.sh"

echo "✅ ISO structure created"
echo ""

# Create ISO
echo "💿 Creating bootable ISO..."
if mkisofs -o "$ISO_OUTPUT" \
    -b boot/grub/stage2_eltorito \
    -no-emul-boot \
    -boot-load-size 4 \
    -boot-info-table \
    -R -J -v -T \
    "$ISO_DIR" 2>/dev/null; then
    echo "✅ ISO created: $ISO_OUTPUT"
else
    echo "⚠️  mkisofs failed (grub not included)"
    echo "   Creating data ISO instead..."
    mkisofs -o "$ISO_OUTPUT" -R -J -v -T "$ISO_DIR"
    echo "✅ Data ISO created: $ISO_OUTPUT"
fi

ISO_SIZE=$(du -h "$ISO_OUTPUT" | cut -f1)
echo "   Size: $ISO_SIZE"
echo ""

# Optionally write to USB
if [ -n "$USB_DEVICE" ]; then
    if [ ! -b "$USB_DEVICE" ]; then
        echo "❌ USB device not found: $USB_DEVICE"
        echo "   Available devices:"
        lsblk -d -o NAME,SIZE,TYPE | grep disk
        exit 1
    fi
    
    echo "⚠️  WARNING: This will ERASE $USB_DEVICE"
    echo "   Press Ctrl+C to cancel, Enter to continue..."
    read
    
    echo "💾 Writing to USB device..."
    sudo dd if="$ISO_OUTPUT" of="$USB_DEVICE" bs=4M status=progress
    sync
    
    echo "✅ USB device ready: $USB_DEVICE"
else
    echo "📋 USB writing skipped (no device specified)"
    echo ""
    echo "💡 To write to USB:"
    echo "   USB_DEVICE=/dev/sdX $0"
    echo ""
    echo "⚠️  Find device with: lsblk"
fi

echo ""
echo "═══════════════════════════════════════════"
echo "✅ NUC USB Deployment Package Ready"
echo "═══════════════════════════════════════════"
echo ""
echo "ISO File: $ISO_OUTPUT"
echo "Size: $ISO_SIZE"
echo ""
echo "Deployment Instructions:"
echo "  1. Boot NUC from USB"
echo "  2. Run: sudo /mnt/install-biomeos.sh"
echo "  3. Reboot"
echo "  4. BiomeOS starts automatically"
echo ""
echo "Validation:"
echo "  ssh biomeos@nuc-device"
echo "  cd /opt/biomeos"
echo "  ./run-e2e-tests.sh"
echo ""

