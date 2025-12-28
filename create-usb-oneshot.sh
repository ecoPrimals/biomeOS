#!/bin/bash
# One-shot USB creation with single sudo prompt
# Creates ISO and writes to USB in one go

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  💾 BiomeOS USB Creation - One-Shot 💾                  ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Detect USB device
echo "🔍 Detecting USB devices..."
echo ""
lsblk -d -o NAME,SIZE,TYPE,MOUNTPOINT | grep disk | nl -v 0
echo ""

# Find most likely USB device (removable, not mounted)
USB_DEVICE=$(lsblk -d -o NAME,RM,TYPE -n | awk '$2=="1" && $3=="disk" {print "/dev/"$1; exit}')

if [ -z "$USB_DEVICE" ]; then
    echo "❌ No removable USB device detected"
    echo ""
    echo "Please specify manually:"
    echo "  USB_DEVICE=/dev/sdX $0"
    exit 1
fi

USB_SIZE=$(lsblk -d -n -o SIZE "$USB_DEVICE")
echo "📍 Detected USB: $USB_DEVICE ($USB_SIZE)"
echo ""

# Confirmation
echo "⚠️  WARNING: This will ERASE all data on $USB_DEVICE"
echo ""
lsblk "$USB_DEVICE" -o NAME,SIZE,TYPE,FSTYPE,MOUNTPOINT
echo ""
echo "Type 'yes' to continue, or Ctrl+C to cancel:"
read -r CONFIRM

if [ "$CONFIRM" != "yes" ]; then
    echo "❌ Aborted"
    exit 1
fi

echo ""
echo "🔑 Requesting sudo access (you'll need to enter password once)..."
echo ""

# Get sudo access once
sudo -v

# Keep sudo alive in background
(while true; do sudo -n true; sleep 50; done 2>/dev/null) &
SUDO_KEEPER_PID=$!

# Cleanup function
cleanup() {
    kill $SUDO_KEEPER_PID 2>/dev/null || true
}
trap cleanup EXIT

echo "✅ Sudo access granted"
echo ""

# Now run the full deployment
echo "═══════════════════════════════════════════════════════════"
echo "Starting USB Creation Process"
echo "═══════════════════════════════════════════════════════════"
echo ""

cd "$SCRIPT_DIR"

# Step 1: Build
echo "STEP 1/4: Building BiomeOS..."
if cargo build --release --workspace --quiet 2>&1 | grep -i error; then
    echo "❌ Build failed"
    exit 1
fi
echo "✅ Build complete"
echo ""

# Step 2: Create ISO structure
echo "STEP 2/4: Creating ISO structure..."
ISO_DIR="$SCRIPT_DIR/iso-build"
sudo rm -rf "$ISO_DIR"
mkdir -p "$ISO_DIR"/{boot/grub,opt/biomeos,etc/systemd/system}

echo "   Copying files..."
cp -r target/release/biomeos* "$ISO_DIR/opt/biomeos/" 2>/dev/null || true
cp -r primals "$ISO_DIR/opt/biomeos/" 2>/dev/null || true
cp -r niches "$ISO_DIR/opt/biomeos/" 2>/dev/null || true
cp -r showcase "$ISO_DIR/opt/biomeos/" 2>/dev/null || true
cp deploy-real-primals.sh "$ISO_DIR/opt/biomeos/" 2>/dev/null || true
cp start-songbird.sh "$ISO_DIR/opt/biomeos/" 2>/dev/null || true
cp run-e2e-tests.sh "$ISO_DIR/opt/biomeos/" 2>/dev/null || true
chmod +x "$ISO_DIR/opt/biomeos/"*.sh 2>/dev/null || true

# GRUB config
cat > "$ISO_DIR/boot/grub/grub.cfg" << 'EOF'
set timeout=5
set default=0

menuentry "BiomeOS - Production Mode" {
    echo "Loading BiomeOS..."
    echo "This is a data ISO. Copy /opt/biomeos to target system."
    halt
}
EOF

# Systemd service
cat > "$ISO_DIR/etc/systemd/system/biomeos.service" << 'EOF'
[Unit]
Description=BiomeOS Primal Coordination
After=network-online.target
Wants=network-online.target

[Service]
Type=forking
User=root
WorkingDirectory=/opt/biomeos
ExecStartPre=/bin/sleep 5
ExecStart=/opt/biomeos/deploy-real-primals.sh
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Install script
cat > "$ISO_DIR/install-biomeos.sh" << 'INSTALL'
#!/bin/bash
set -e
echo "🚀 Installing BiomeOS to system..."
sudo cp -r /mnt/iso/opt/biomeos /opt/
sudo cp /mnt/iso/etc/systemd/system/biomeos.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable biomeos.service
echo "✅ BiomeOS installed"
echo "Start with: sudo systemctl start biomeos"
INSTALL

chmod +x "$ISO_DIR/install-biomeos.sh"

# README
cat > "$ISO_DIR/README.txt" << 'README'
BiomeOS USB Installation

This USB contains a complete BiomeOS deployment package.

To Install:
1. Mount this USB
2. Run: sudo ./install-biomeos.sh
3. Start: sudo systemctl start biomeos
4. Verify: curl http://localhost:9020/health

What's Included:
- BiomeOS core (all crates)
- 5 Primals (NestGate, BearDog, Songbird, Toadstool, PetalTongue)
- RootPulse BYOB niche
- 20 Showcase demos
- Auto-start systemd service

Support: https://github.com/ecoPrimals/biomeOS
README

echo "✅ ISO structure ready"
echo ""

# Step 3: Create ISO
echo "STEP 3/4: Creating ISO image..."
ISO_NAME="biomeos-usb-$(date +%Y%m%d-%H%M%S).iso"

if command -v xorriso &> /dev/null; then
    xorriso -as mkisofs \
        -o "$ISO_NAME" \
        -R -J -joliet-long \
        -V "BiomeOS" \
        "$ISO_DIR" 2>&1 | grep -E "(Writing|extents|done)" || true
    echo "✅ ISO created: $ISO_NAME"
else
    echo "⚠️  xorriso not found, creating tar.gz instead"
    tar -czf "$ISO_NAME.tar.gz" -C "$ISO_DIR" .
    ISO_NAME="$ISO_NAME.tar.gz"
    echo "✅ Archive created: $ISO_NAME"
fi

ISO_SIZE=$(du -h "$ISO_NAME" | cut -f1)
echo "   Size: $ISO_SIZE"
echo ""

# Step 4: Write to USB
echo "STEP 4/4: Writing to USB device..."
echo "   Target: $USB_DEVICE"
echo "   Size: $USB_SIZE"
echo ""

# Unmount if mounted
sudo umount "$USB_DEVICE"* 2>/dev/null || true

# Write ISO
echo "💾 Writing (this may take 2-5 minutes)..."
sudo dd if="$ISO_NAME" of="$USB_DEVICE" bs=4M status=progress oflag=sync 2>&1 | tail -5

# Sync
echo ""
echo "🔄 Syncing..."
sudo sync

echo "✅ USB written successfully"
echo ""

# Verification
echo "📊 Verification:"
sudo fdisk -l "$USB_DEVICE" | head -10

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "✅ USB CREATION COMPLETE"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "📦 Created: $ISO_NAME ($ISO_SIZE)"
echo "💾 USB Device: $USB_DEVICE (ready)"
echo ""
echo "🎯 Next Steps - Test in VM:"
echo ""
echo "1. Create test VM (Ubuntu 22.04)"
echo "2. Attach USB to VM"
echo "3. In VM, mount USB:"
echo "   sudo mkdir -p /mnt/iso"
echo "   sudo mount $USB_DEVICE /mnt/iso"
echo ""
echo "4. Install BiomeOS:"
echo "   cd /mnt/iso"
echo "   sudo ./install-biomeos.sh"
echo ""
echo "5. Start and verify:"
echo "   sudo systemctl start biomeos"
echo "   curl http://localhost:9020/health"
echo "   curl http://localhost:2300/health"
echo ""
echo "6. Run validation:"
echo "   cd /opt/biomeos"
echo "   ./run-e2e-tests.sh"
echo ""
echo "Once validated in VM, deploy to NUC!"
echo ""

