#!/bin/bash
set -e

# BiomeOS Alpine USB Builder
# Fast path to bootable USB for real-world validation
# Uses Alpine Linux as base + BiomeOS on top

ALPINE_VERSION="3.19"
ALPINE_ARCH="x86_64"
ALPINE_ISO="alpine-standard-${ALPINE_VERSION}.0-${ALPINE_ARCH}.iso"
ALPINE_URL="https://dl-cdn.alpinelinux.org/alpine/v${ALPINE_VERSION}/releases/${ALPINE_ARCH}/${ALPINE_ISO}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/build/alpine-usb"
OUTPUT_DIR="${PROJECT_ROOT}/dist"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "BiomeOS Alpine USB Builder - Fast Track"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
log_info "Purpose: Get NUC bootable for real-world validation"
log_info "Strategy: Alpine Linux + BiomeOS primals"
echo ""

# Check dependencies
log_info "Checking dependencies..."
missing_deps=()
for dep in curl cargo rsync syslinux; do
    if ! command -v "$dep" &> /dev/null; then
        missing_deps+=("$dep")
    fi
done

if [ ${#missing_deps[@]} -ne 0 ]; then
    log_error "Missing dependencies: ${missing_deps[*]}"
    log_info "Install: sudo apt-get install ${missing_deps[*]}"
    exit 1
fi
log_success "All dependencies found"

# Create build directory
log_info "Creating build directory..."
rm -rf "${BUILD_DIR}"
mkdir -p "${BUILD_DIR}"
mkdir -p "${OUTPUT_DIR}"
mkdir -p "${BUILD_DIR}/usb"
mkdir -p "${BUILD_DIR}/alpine-root"

# Download Alpine ISO if needed
if [ ! -f "${BUILD_DIR}/${ALPINE_ISO}" ]; then
    log_info "Downloading Alpine Linux ${ALPINE_VERSION}..."
    curl -L "${ALPINE_URL}" -o "${BUILD_DIR}/${ALPINE_ISO}"
    log_success "Alpine ISO downloaded"
else
    log_info "Using cached Alpine ISO"
fi

# Mount Alpine ISO
log_info "Extracting Alpine ISO..."
MOUNT_POINT="${BUILD_DIR}/alpine-mount"
mkdir -p "${MOUNT_POINT}"
sudo mount -o loop "${BUILD_DIR}/${ALPINE_ISO}" "${MOUNT_POINT}" 2>/dev/null || true

# Copy Alpine to USB structure
log_info "Creating USB file structure..."
sudo rsync -a "${MOUNT_POINT}/" "${BUILD_DIR}/usb/"
sudo umount "${MOUNT_POINT}" 2>/dev/null || true
rmdir "${MOUNT_POINT}"

# Build BiomeOS binaries
log_info "Building BiomeOS binaries..."
cd "${PROJECT_ROOT}"
cargo build --release --bins

# Create BiomeOS directory on USB
log_info "Adding BiomeOS to USB..."
mkdir -p "${BUILD_DIR}/usb/biomeos/bin"
mkdir -p "${BUILD_DIR}/usb/biomeos/primals"
mkdir -p "${BUILD_DIR}/usb/biomeos/configs"
mkdir -p "${BUILD_DIR}/usb/biomeos/keys"
mkdir -p "${BUILD_DIR}/usb/biomeos/logs"

# Copy BiomeOS binaries
cp "${PROJECT_ROOT}/target/release/biome" "${BUILD_DIR}/usb/biomeos/bin/" 2>/dev/null || true
cp "${PROJECT_ROOT}/target/release/biomeos-cli" "${BUILD_DIR}/usb/biomeos/bin/" 2>/dev/null || true

# Copy Phase 1 binaries if available
if [ -d "${PROJECT_ROOT}/../phase1bins" ]; then
    log_info "Copying Phase 1 primal binaries..."
    cp -r "${PROJECT_ROOT}/../phase1bins/"* "${BUILD_DIR}/usb/biomeos/primals/" 2>/dev/null || true
fi

# Copy BYOB templates
if [ -d "${PROJECT_ROOT}/templates" ]; then
    log_info "Copying BYOB templates..."
    cp -r "${PROJECT_ROOT}/templates" "${BUILD_DIR}/usb/biomeos/"
fi

# Create BiomeOS init script
log_info "Creating BiomeOS init script..."
cat > "${BUILD_DIR}/usb/biomeos/biomeos-init.sh" << 'EOF'
#!/bin/sh
# BiomeOS Initialization Script

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "BiomeOS - Sovereignty-First Operating System"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Set up environment
export PATH="/media/usb/biomeos/bin:$PATH"
export BIOMEOS_ROOT="/media/usb/biomeos"
export BIOMEOS_PRIMALS="$BIOMEOS_ROOT/primals"
export BIOMEOS_CONFIGS="$BIOMEOS_ROOT/configs"

echo "🔍 Detecting network configuration..."
ip addr show

echo ""
echo "🔍 Discovering BiomeOS primals..."
ls -lh "$BIOMEOS_PRIMALS/" 2>/dev/null || echo "No primals found"

echo ""
echo "📡 Starting mDNS discovery..."
# avahi-daemon would run here

echo ""
echo "🚀 BiomeOS ready for deployment testing!"
echo ""
echo "Available commands:"
echo "  biomeos-cli discover    - Discover primals and nodes"
echo "  biomeos-cli p2p         - Test P2P coordination"
echo "  biomeos-cli test        - Run integration tests"
echo ""

# Drop to shell
exec /bin/sh
EOF

chmod +x "${BUILD_DIR}/usb/biomeos/biomeos-init.sh"

# Create Alpine boot configuration
log_info "Configuring Alpine boot..."
cat > "${BUILD_DIR}/usb/boot/grub/grub.cfg" << 'EOF'
set timeout=10
set default=0

menuentry "BiomeOS - Real Deployment Validation" {
    linux /boot/vmlinuz-lts modules=loop,squashfs,sd-mod,usb-storage quiet
    initrd /boot/initramfs-lts
}

menuentry "BiomeOS - Network Discovery Mode" {
    linux /boot/vmlinuz-lts modules=loop,squashfs,sd-mod,usb-storage,e1000,e1000e biomeos=discovery
    initrd /boot/initramfs-lts
}

menuentry "Alpine Linux (Standard)" {
    linux /boot/vmlinuz-lts modules=loop,squashfs,sd-mod,usb-storage quiet
    initrd /boot/initramfs-lts
}
EOF

# Create README
log_info "Creating README..."
cat > "${BUILD_DIR}/usb/biomeos/README.md" << EOF
# BiomeOS Alpine USB - Real Deployment Validation

## Purpose
This USB drive contains BiomeOS on top of Alpine Linux for real-world
deployment validation on physical hardware (NUC).

## What's Included
- Alpine Linux ${ALPINE_VERSION} (base OS)
- BiomeOS binaries (Rust)
- Phase 1 primal binaries
- BYOB templates
- Network discovery tools

## Usage

### 1. Boot NUC from USB
- Insert USB into NUC
- Boot (F12 or Del for boot menu)
- Select USB device

### 2. Network Configuration
\`\`\`bash
# Configure network (DHCP)
setup-interfaces

# Or static IP
ifconfig eth0 192.168.1.100 netmask 255.255.255.0
route add default gw 192.168.1.1
\`\`\`

### 3. Start BiomeOS
\`\`\`bash
cd /media/usb/biomeos
./biomeos-init.sh
\`\`\`

### 4. Discover VMs on Other Machines
\`\`\`bash
# Discover nodes via mDNS
biomeos-cli discover --network

# Test P2P coordination
biomeos-cli p2p test --remote-nodes
\`\`\`

## Validation Tests

### Test 1: Same LAN Discovery
- NUC: 192.168.1.100
- VM1 on Machine 2: 192.168.1.50
- VM2 on Machine 3: 192.168.1.75
- Expected: All nodes discover each other

### Test 2: BTSP Tunnels
- Establish BTSP tunnel: NUC → VM1
- Establish BTSP tunnel: NUC → VM2
- Expected: Encrypted P2P connections

### Test 3: Multi-Machine Coordination
- NUC coordinates with VMs on different machines
- Expected: Distributed primal mesh working

## Rebuilding on Different Network

Just reboot and configure different network:
\`\`\`bash
# Test on home network
ifconfig eth0 192.168.1.100 ...

# Reboot and test on lab network
ifconfig eth0 10.0.0.100 ...
\`\`\`

## Success Criteria

✅ NUC boots from USB
✅ Network configured automatically/manually
✅ Discovers VMs on different physical machines
✅ BTSP tunnels work across real network
✅ P2P coordination works (not just localhost)
✅ Can wipe/rebuild quickly (just reboot!)

## Next Steps

After validation:
1. Document what works
2. Document what fails
3. Identify real-world gaps
4. Plan pure Rust evolution

## Built
$(date)

## Version
Alpine: ${ALPINE_VERSION}
BiomeOS: $(cd "${PROJECT_ROOT}" && git describe --always 2>/dev/null || echo "dev")
EOF

# Create ISO from USB structure
log_info "Creating bootable USB image..."
OUTPUT_IMG="${OUTPUT_DIR}/biomeos-alpine-$(date +%Y%m%d-%H%M%S).img"

# Calculate size (Alpine ISO + 500MB for BiomeOS)
ALPINE_SIZE=$(du -sm "${BUILD_DIR}/${ALPINE_ISO}" | cut -f1)
USB_SIZE=$((ALPINE_SIZE + 500))

log_info "Creating ${USB_SIZE}MB USB image..."
dd if=/dev/zero of="${OUTPUT_IMG}" bs=1M count="${USB_SIZE}" status=progress

# Create partition table
log_info "Creating partition table..."
parted "${OUTPUT_IMG}" mklabel msdos
parted "${OUTPUT_IMG}" mkpart primary fat32 1MiB 100%
parted "${OUTPUT_IMG}" set 1 boot on

# Setup loop device
LOOP_DEVICE=$(sudo losetup -f)
sudo losetup "${LOOP_DEVICE}" "${OUTPUT_IMG}"
sudo partprobe "${LOOP_DEVICE}"

# Format partition
log_info "Formatting USB..."
sudo mkfs.vfat -F 32 "${LOOP_DEVICE}p1"

# Mount and copy files
FINAL_MOUNT="${BUILD_DIR}/final-usb"
mkdir -p "${FINAL_MOUNT}"
sudo mount "${LOOP_DEVICE}p1" "${FINAL_MOUNT}"
sudo cp -r "${BUILD_DIR}/usb/"* "${FINAL_MOUNT}/"

# Install bootloader
log_info "Installing bootloader..."
sudo extlinux --install "${FINAL_MOUNT}/boot"
sudo dd if=/usr/lib/syslinux/mbr/mbr.bin of="${LOOP_DEVICE}" bs=440 count=1 conv=notrunc

# Cleanup
sudo umount "${FINAL_MOUNT}"
sudo losetup -d "${LOOP_DEVICE}"

# Generate checksums
log_info "Generating checksums..."
cd "${OUTPUT_DIR}"
sha256sum "$(basename "${OUTPUT_IMG}")" > "$(basename "${OUTPUT_IMG}").sha256"

log_success "USB image created: ${OUTPUT_IMG}"
log_info "Size: $(du -h "${OUTPUT_IMG}" | cut -f1)"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ BiomeOS Alpine USB Ready!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
log_info "To write to USB drive:"
echo "  sudo dd if=${OUTPUT_IMG} of=/dev/sdX bs=4M status=progress"
echo "  (Replace /dev/sdX with your USB device)"
echo ""
log_info "To test in QEMU first:"
echo "  qemu-system-x86_64 -m 2048 -enable-kvm -hda ${OUTPUT_IMG}"
echo ""
log_warning "Remember: This is for REAL DEPLOYMENT validation!"
log_info "Test on NUC with VMs on other machines to prove it's real!"
echo ""

