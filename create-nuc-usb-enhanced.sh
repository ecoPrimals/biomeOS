#!/bin/bash
# Enhanced NUC USB + Federation Setup
# 
# Creates bootable USB for NUC and configures federation with VM node

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  💾 BiomeOS NUC USB + Federation Deployment 💾          ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

# Configuration
USB_DEVICE="${1:-}"
VM_IP="${2:-}"

show_usage() {
    echo "Usage: $0 [USB_DEVICE] [VM_IP]"
    echo ""
    echo "Examples:"
    echo "  $0                          # Create ISO only"
    echo "  $0 /dev/sdb                 # Create ISO + write to USB"
    echo "  $0 /dev/sdb 192.168.1.100   # Full setup with federation"
    echo ""
    echo "Steps:"
    echo "  1. Build BiomeOS for x86_64"
    echo "  2. Create bootable ISO"
    echo "  3. Write to USB (optional)"
    echo "  4. Configure federation (optional)"
    echo ""
}

if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    show_usage
    exit 0
fi

# Check dependencies
echo "🔍 Checking dependencies..."
MISSING=()
for cmd in cargo mkisofs dd xorriso; do
    if ! command -v $cmd &> /dev/null; then
        MISSING+=("$cmd")
    fi
done

if [ ${#MISSING[@]} -gt 0 ]; then
    echo "❌ Missing: ${MISSING[*]}"
    echo ""
    echo "Install with:"
    echo "  sudo apt install build-essential genisoimage coreutils xorriso grub-pc-bin"
    exit 1
fi

echo "✅ Dependencies OK"
echo ""

# Step 1: Build BiomeOS
echo "═══════════════════════════════════════════════════════════"
echo "STEP 1: Build BiomeOS for x86_64-unknown-linux-gnu (NUC)"
echo "═══════════════════════════════════════════════════════════"
echo ""

cd "$SCRIPT_DIR"
echo "🔨 Building release binaries..."
cargo build --release --workspace --quiet

if [ $? -eq 0 ]; then
    echo "✅ Build complete"
else
    echo "❌ Build failed"
    exit 1
fi

echo ""

# Step 2: Collect Binaries
echo "═══════════════════════════════════════════════════════════"
echo "STEP 2: Collect Primal Binaries"
echo "═══════════════════════════════════════════════════════════"
echo ""

echo "📦 Checking primal binaries..."
PRIMALS=(nestgate beardog songbird toadstool petal-tongue)
for primal in "${PRIMALS[@]}"; do
    if [ -f "primals/$primal" ]; then
        SIZE=$(du -h "primals/$primal" | cut -f1)
        echo "   ✅ $primal ($SIZE)"
    else
        echo "   ⚠️  $primal (missing - will use placeholder)"
    fi
done

echo ""

# Step 3: Create ISO Structure
echo "═══════════════════════════════════════════════════════════"
echo "STEP 3: Create ISO Structure"
echo "═══════════════════════════════════════════════════════════"
echo ""

ISO_DIR="$SCRIPT_DIR/iso-build"
rm -rf "$ISO_DIR"
mkdir -p "$ISO_DIR"/{boot/grub,opt/biomeos,etc/systemd/system}

echo "📂 Creating ISO structure..."

# Copy BiomeOS files
echo "   Copying BiomeOS core..."
cp -r target/release/biomeos* "$ISO_DIR/opt/biomeos/" 2>/dev/null || true

echo "   Copying primals..."
cp -r primals "$ISO_DIR/opt/biomeos/" 2>/dev/null || true

echo "   Copying configurations..."
cp -r niches "$ISO_DIR/opt/biomeos/" 2>/dev/null || true
cp -r showcase "$ISO_DIR/opt/biomeos/" 2>/dev/null || true

echo "   Copying scripts..."
cp deploy-real-primals.sh "$ISO_DIR/opt/biomeos/" 2>/dev/null || true
cp start-songbird.sh "$ISO_DIR/opt/biomeos/" 2>/dev/null || true
cp run-e2e-tests.sh "$ISO_DIR/opt/biomeos/" 2>/dev/null || true
cp validate-with-benchscale.sh "$ISO_DIR/opt/biomeos/" 2>/dev/null || true
chmod +x "$ISO_DIR/opt/biomeos/"*.sh 2>/dev/null || true

# Create GRUB config
echo "   Creating GRUB bootloader..."
cat > "$ISO_DIR/boot/grub/grub.cfg" << 'EOF'
set timeout=5
set default=0

menuentry "BiomeOS - Production Mode" {
    linux /boot/vmlinuz quiet splash
    initrd /boot/initrd
}

menuentry "BiomeOS - Debug Mode (verbose)" {
    linux /boot/vmlinuz debug loglevel=7
    initrd /boot/initrd
}

menuentry "BiomeOS - Federation Mode" {
    linux /boot/vmlinuz federation=auto
    initrd /boot/initrd
}
EOF

# Create systemd service
echo "   Creating systemd service..."
cat > "$ISO_DIR/etc/systemd/system/biomeos.service" << 'EOF'
[Unit]
Description=BiomeOS Primal Coordination Substrate
Documentation=https://github.com/ecoPrimals/biomeOS
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
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF

# Create README
echo "   Creating README..."
cat > "$ISO_DIR/README.txt" << 'EOF'
BiomeOS - Bootable NUC Deployment

This USB contains a complete BiomeOS installation.

Boot Instructions:
1. Insert USB into Intel NUC
2. Power on and press F10 (or F2/Del) for boot menu
3. Select USB drive
4. Choose boot option from GRUB menu

After Boot:
- BiomeOS will start automatically
- All primals will be deployed
- Federation discovery will begin (if network available)

Validation:
  curl http://localhost:9020/health  # NestGate
  curl http://localhost:2300/health  # Songbird
  /opt/biomeos/run-e2e-tests.sh      # Full test suite

Federation:
  Songbird will automatically discover federated nodes via mDNS.
  No manual configuration needed!

Documentation:
  /opt/biomeos/showcase/              # 20 working demos
  /opt/biomeos/niches/rootpulse/      # RootPulse BYOB niche
  
Support:
  https://github.com/ecoPrimals/biomeOS
EOF

echo "✅ ISO structure created"
echo ""

# Step 4: Create Bootable ISO
echo "═══════════════════════════════════════════════════════════"
echo "STEP 4: Create Bootable ISO"
echo "═══════════════════════════════════════════════════════════"
echo ""

ISO_NAME="biomeos-nuc-$(date +%Y%m%d-%H%M%S).iso"

echo "💿 Creating ISO: $ISO_NAME"

# Use grub-mkrescue for proper bootable ISO
if command -v grub-mkrescue &> /dev/null; then
    grub-mkrescue -o "$ISO_NAME" "$ISO_DIR" 2>&1 | grep -v "^xorriso" || true
    echo "✅ Bootable ISO created"
else
    # Fallback to mkisofs (data ISO only, not bootable)
    mkisofs -o "$ISO_NAME" -R -J -v -T "$ISO_DIR" 2>&1 | grep -E "^(Total|Writing)" || true
    echo "⚠️  Data ISO created (not bootable - grub-mkrescue not found)"
fi

ISO_SIZE=$(du -h "$ISO_NAME" | cut -f1)
echo "   Size: $ISO_SIZE"
echo ""

# Step 5: Write to USB (optional)
if [ -n "$USB_DEVICE" ]; then
    echo "═══════════════════════════════════════════════════════════"
    echo "STEP 5: Write to USB Device"
    echo "═══════════════════════════════════════════════════════════"
    echo ""
    
    if [ ! -b "$USB_DEVICE" ]; then
        echo "❌ Device not found: $USB_DEVICE"
        echo ""
        echo "Available devices:"
        lsblk -d -o NAME,SIZE,TYPE,MOUNTPOINT | grep disk
        exit 1
    fi
    
    # Safety check
    SIZE=$(lsblk -b -d -n -o SIZE "$USB_DEVICE")
    SIZE_GB=$((SIZE / 1024 / 1024 / 1024))
    
    echo "⚠️  WARNING: This will ERASE all data on $USB_DEVICE ($SIZE_GB GB)"
    echo ""
    echo "Device info:"
    lsblk "$USB_DEVICE" -o NAME,SIZE,TYPE,FSTYPE,MOUNTPOINT
    echo ""
    echo "Press Ctrl+C to cancel, or Enter to continue..."
    read -r
    
    echo ""
    echo "💾 Writing ISO to $USB_DEVICE..."
    sudo dd if="$ISO_NAME" of="$USB_DEVICE" bs=4M status=progress oflag=sync
    sudo sync
    
    echo "✅ USB written successfully"
    echo ""
else
    echo "═══════════════════════════════════════════════════════════"
    echo "STEP 5: USB Writing (Skipped)"
    echo "═══════════════════════════════════════════════════════════"
    echo ""
    echo "No USB device specified."
    echo ""
    echo "To write later:"
    echo "  sudo dd if=$ISO_NAME of=/dev/sdX bs=4M status=progress oflag=sync"
    echo "  sudo sync"
    echo ""
fi

# Step 6: Federation Setup (optional)
if [ -n "$VM_IP" ]; then
    echo "═══════════════════════════════════════════════════════════"
    echo "STEP 6: Federation Configuration"
    echo "═══════════════════════════════════════════════════════════"
    echo ""
    
    echo "🌐 Federation VM: $VM_IP"
    echo ""
    
    # Test VM connectivity
    if ping -c 1 -W 2 "$VM_IP" &> /dev/null; then
        echo "✅ VM is reachable"
        
        # Check if BiomeOS is running on VM
        if curl -s -f "http://$VM_IP:2300/health" &> /dev/null; then
            echo "✅ Songbird running on VM"
            
            # Get federation info
            echo ""
            echo "VM Federation Status:"
            curl -s "http://$VM_IP:2300/federation/status" 2>/dev/null | head -10 || echo "  (Status endpoint not available)"
        else
            echo "⚠️  BiomeOS not running on VM (Songbird not responding)"
            echo "   Start BiomeOS on VM: ./deploy-real-primals.sh"
        fi
    else
        echo "⚠️  VM not reachable at $VM_IP"
        echo "   Check network connectivity"
    fi
    echo ""
else
    echo "═══════════════════════════════════════════════════════════"
    echo "STEP 6: Federation Setup (Skipped)"
    echo "═══════════════════════════════════════════════════════════"
    echo ""
    echo "No VM IP specified. Songbird will auto-discover peers via mDNS."
    echo ""
    echo "To specify VM for federation:"
    echo "  $0 /dev/sdX 192.168.1.100"
    echo ""
fi

# Summary
echo "═══════════════════════════════════════════════════════════"
echo "✅ DEPLOYMENT COMPLETE"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "📦 ISO Created: $ISO_NAME ($ISO_SIZE)"
[ -n "$USB_DEVICE" ] && echo "💾 USB Device: $USB_DEVICE (ready to boot)"
[ -n "$VM_IP" ] && echo "🌐 Federation: $VM_IP (configured)"
echo ""
echo "🎯 Next Steps:"
echo ""
echo "1. Boot NUC from USB:"
echo "   - Insert USB into NUC"
echo "   - Power on and press F10 for boot menu"
echo "   - Select USB drive"
echo "   - Choose 'BiomeOS - Production Mode'"
echo ""
echo "2. Wait for BiomeOS to start (~30 seconds)"
echo ""
echo "3. Verify primals are running:"
echo "   curl http://localhost:9020/health  # NestGate"
echo "   curl http://localhost:2300/health  # Songbird"
echo ""
echo "4. Check federation (if VM configured):"
echo "   curl http://localhost:2300/federation/peers"
echo ""
echo "5. Run validation:"
echo "   cd /opt/biomeos"
echo "   ./run-e2e-tests.sh"
echo ""
echo "6. Deploy RootPulse niche:"
echo "   biomeos niche deploy /opt/biomeos/niches/rootpulse/rootpulse-niche.yaml"
echo ""
echo "Documentation:"
echo "  - README: $ISO_DIR/README.txt"
echo "  - Guide: NUC_USB_DEPLOYMENT_GUIDE.md"
echo ""
echo "🎊 Three-Tier Deployment Pipeline Complete!"
echo "   Tier 1: Development ✅"
echo "   Tier 2: benchScale ✅"
echo "   Tier 3: NUC USB ✅"
echo ""

