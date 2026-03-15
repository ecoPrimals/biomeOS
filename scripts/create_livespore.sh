#!/bin/bash
# create_livespore.sh - Convert a USB to a bootable LiveSpore
#
# This script creates a bootable USB with:
# - Minimal Alpine Linux base (~50MB)
# - biomeOS genome binaries
# - Genetic inheritance from Tower parent
# - Auto-start NUCLEUS on boot
#
# IMPORTANT: This will DESTROY all data on the target USB!
#
# Usage: ./create_livespore.sh /dev/sdX spore-name
#
# Example: ./create_livespore.sh /dev/sdc livespore-alpha

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$(dirname "$SCRIPT_DIR")"
ALPINE_VERSION="3.19"
ALPINE_URL="https://dl-cdn.alpinelinux.org/alpine/v${ALPINE_VERSION}/releases/x86_64/alpine-standard-${ALPINE_VERSION}.0-x86_64.iso"
ALPINE_ISO="/tmp/alpine-livespore.iso"
FAMILY_ID="8ff3b864a4bc589a"
PARENT_MITO_SEED="${BIOMEOS_MITO_SEED:-/media/${USER}/BEA6-BBCE/biomeOS/.family.seed}"

# Functions
log_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }
log_warn() { echo -e "${YELLOW}⚠️  $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}" >&2; }

usage() {
    echo "Usage: $0 <device> <spore-name>"
    echo ""
    echo "Arguments:"
    echo "  device      - USB device (e.g., /dev/sdc) - NOT the partition!"
    echo "  spore-name  - Unique name for this spore (e.g., livespore-alpha)"
    echo ""
    echo "Example:"
    echo "  $0 /dev/sdc livespore-alpha"
    echo ""
    echo "WARNING: This will ERASE ALL DATA on the device!"
    exit 1
}

check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "This script must be run as root (use sudo)"
        exit 1
    fi
}

check_device() {
    local device="$1"
    
    if [[ ! -b "$device" ]]; then
        log_error "Device $device does not exist or is not a block device"
        exit 1
    fi
    
    # Safety check - don't allow nvme or main disk
    if [[ "$device" == *"nvme0n1"* ]] || [[ "$device" == "/dev/sda" ]]; then
        log_error "Refusing to format system disk: $device"
        exit 1
    fi
    
    # Check if mounted
    if mount | grep -q "^$device"; then
        log_warn "Device $device has mounted partitions. Unmounting..."
        umount "${device}"* 2>/dev/null || true
    fi
}

download_alpine() {
    if [[ -f "$ALPINE_ISO" ]]; then
        log_info "Alpine ISO already exists: $ALPINE_ISO"
        return 0
    fi
    
    log_info "Downloading Alpine Linux ${ALPINE_VERSION}..."
    wget -q --show-progress -O "$ALPINE_ISO" "$ALPINE_URL"
    log_success "Downloaded Alpine Linux"
}

create_partitions() {
    local device="$1"
    
    log_info "Creating partition table on $device..."
    
    # Create GPT partition table with:
    # 1. EFI System Partition (512MB, FAT32)
    # 2. Boot partition (512MB, ext4)
    # 3. Root partition (remaining, ext4)
    
    parted -s "$device" mklabel gpt
    parted -s "$device" mkpart "EFI" fat32 1MiB 513MiB
    parted -s "$device" set 1 esp on
    parted -s "$device" mkpart "boot" ext4 513MiB 1025MiB
    parted -s "$device" mkpart "root" ext4 1025MiB 100%
    
    # Wait for kernel to update partition table
    sleep 2
    partprobe "$device"
    sleep 1
    
    log_success "Partitions created"
}

format_partitions() {
    local device="$1"
    local spore_name="$2"
    
    log_info "Formatting partitions..."
    
    mkfs.vfat -F 32 -n "EFI" "${device}1"
    mkfs.ext4 -L "boot" "${device}2"
    mkfs.ext4 -L "biomeOS-${spore_name}" "${device}3"
    
    log_success "Partitions formatted"
}

mount_partitions() {
    local device="$1"
    local mount_root="/tmp/livespore-mount"
    
    log_info "Mounting partitions..."
    
    mkdir -p "$mount_root"/{efi,boot,root}
    mount "${device}3" "$mount_root/root"
    mkdir -p "$mount_root/root/boot"
    mount "${device}2" "$mount_root/root/boot"
    mkdir -p "$mount_root/root/boot/efi"
    mount "${device}1" "$mount_root/root/boot/efi"
    
    echo "$mount_root"
}

install_alpine() {
    local mount_root="$1"
    
    log_info "Installing Alpine Linux base system..."
    
    # Extract Alpine ISO to root
    local iso_mount="/tmp/alpine-iso-mount"
    mkdir -p "$iso_mount"
    mount -o loop "$ALPINE_ISO" "$iso_mount"
    
    # Copy Alpine system
    cp -a "$iso_mount"/* "$mount_root/root/" || true
    
    umount "$iso_mount"
    
    log_success "Alpine base installed"
}

install_genomes() {
    local mount_root="$1"
    local genome_dir="$mount_root/root/biomeOS"
    
    log_info "Installing biomeOS genomes..."
    
    mkdir -p "$genome_dir"/{primals,config,graphs,scripts}
    
    # Copy genome binaries
    cp "$BIOMEOS_ROOT/livespore-usb/x86_64/primals/x86_64"/* "$genome_dir/primals/"
    chmod +x "$genome_dir/primals"/*
    
    # Copy configuration
    cp "$BIOMEOS_ROOT/livespore-usb/x86_64/manifest.toml" "$genome_dir/"
    cp "$BIOMEOS_ROOT/livespore-usb/x86_64/checksums.toml" "$genome_dir/"
    cp -r "$BIOMEOS_ROOT/livespore-usb/x86_64/config"/* "$genome_dir/config/" 2>/dev/null || true
    cp -r "$BIOMEOS_ROOT/livespore-usb/x86_64/graphs"/* "$genome_dir/graphs/" 2>/dev/null || true
    
    # Copy deploy script
    cp "$BIOMEOS_ROOT/livespore-usb/x86_64/scripts/deploy_cross_arch.sh" "$genome_dir/scripts/"
    
    log_success "Genomes installed (6 components)"
}

derive_genetics() {
    local mount_root="$1"
    local spore_name="$2"
    local genome_dir="$mount_root/root/biomeOS"
    
    log_info "Deriving genetics for $spore_name..."
    
    # Pass mito beacon unchanged (maternal inheritance)
    cp "$PARENT_MITO_SEED" "$genome_dir/.family.seed"
    chmod 600 "$genome_dir/.family.seed"
    
    MITO_HASH=$(sha256sum "$genome_dir/.family.seed" | cut -c1-16)
    log_info "  Mito beacon: $MITO_HASH (passed unchanged)"
    
    # Derive new lineage seed (nuclear DNA mixing)
    BATCH=$(date -u +%Y-%m-%d)
    PARENT_SEED=$(xxd -p "$PARENT_MITO_SEED" | tr -d '\n')
    LINEAGE_SEED=$(echo -n "${PARENT_SEED}${spore_name}${BATCH}" | sha256sum | cut -d' ' -f1)
    
    echo "$LINEAGE_SEED" | xxd -r -p > "$genome_dir/.lineage.seed"
    chmod 600 "$genome_dir/.lineage.seed"
    
    LINEAGE_HASH="${LINEAGE_SEED:0:16}"
    log_info "  Lineage seed: $LINEAGE_HASH (mixed/derived)"
    
    # Create spore identity
    cat > "$genome_dir/.spore.json" << EOF
{
    "spore_id": "$spore_name",
    "spore_type": "LiveSpore",
    "spore_format": "bootable",
    "bootable": true,
    "generation": 1,
    "parent_id": "tower-pop-os",
    "family_id": "$FAMILY_ID",
    "deployment_batch": "$BATCH",
    "birth_timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "temporal_note": "Younger temporal sibling of ColdSpores",
    
    "genetics": {
        "model": "eukaryotic",
        "mito_beacon": {
            "hash": "$MITO_HASH",
            "inheritance": "PASSED"
        },
        "lineage_seed": {
            "hash": "$LINEAGE_HASH",
            "inheritance": "MIXED",
            "derivation": "SHA256(parent_mito || spore_id || batch)"
        }
    },
    
    "capabilities": ["deploy", "federate", "reproduce", "boot"],
    "requires_host_os": false
}
EOF
    
    log_success "Genetics derived"
}

install_bootloader() {
    local device="$1"
    local mount_root="$2"
    
    log_info "Installing GRUB bootloader..."
    
    # Install GRUB for UEFI
    grub-install --target=x86_64-efi \
        --efi-directory="$mount_root/root/boot/efi" \
        --boot-directory="$mount_root/root/boot" \
        --removable \
        --no-nvram \
        "$device"
    
    # Create GRUB config
    cat > "$mount_root/root/boot/grub/grub.cfg" << 'EOF'
set timeout=5
set default=0

menuentry "biomeOS LiveSpore" {
    linux /boot/vmlinuz-lts modules=loop,squashfs,sd-mod,usb-storage quiet
    initrd /boot/initramfs-lts
}

menuentry "biomeOS LiveSpore (Safe Mode)" {
    linux /boot/vmlinuz-lts modules=loop,squashfs,sd-mod,usb-storage single
    initrd /boot/initramfs-lts
}
EOF
    
    log_success "Bootloader installed"
}

create_autostart() {
    local mount_root="$1"
    local spore_name="$2"
    
    log_info "Creating auto-start script..."
    
    # Create init script to start NUCLEUS on boot
    cat > "$mount_root/root/etc/local.d/biomeos.start" << 'EOF'
#!/bin/sh
# biomeOS LiveSpore Auto-Start

BIOMEOS_DIR="/biomeOS"

# Export environment
export FAMILY_ID="8ff3b864a4bc589a"
export NODE_ID="$(cat /biomeOS/.spore.json | grep spore_id | cut -d'"' -f4)"
export FAMILY_SEED_PATH="$BIOMEOS_DIR/.family.seed"

# Start Tower (BearDog + Songbird)
$BIOMEOS_DIR/primals/beardog server --daemon &
sleep 2
$BIOMEOS_DIR/primals/songbird server --daemon &

echo "biomeOS LiveSpore started: $NODE_ID"
EOF
    
    chmod +x "$mount_root/root/etc/local.d/biomeos.start"
    
    log_success "Auto-start configured"
}

cleanup() {
    local mount_root="/tmp/livespore-mount"
    
    log_info "Cleaning up..."
    
    umount "$mount_root/root/boot/efi" 2>/dev/null || true
    umount "$mount_root/root/boot" 2>/dev/null || true
    umount "$mount_root/root" 2>/dev/null || true
    
    rm -rf "$mount_root"
}

main() {
    if [[ $# -lt 2 ]]; then
        usage
    fi
    
    local device="$1"
    local spore_name="$2"
    
    echo ""
    echo "=============================================="
    echo "   🧬 LiveSpore Creation Tool"
    echo "=============================================="
    echo ""
    echo "Device:     $device"
    echo "Spore Name: $spore_name"
    echo "Family ID:  $FAMILY_ID"
    echo ""
    echo -e "${RED}⚠️  WARNING: This will ERASE ALL DATA on $device!${NC}"
    echo ""
    read -p "Type 'YES' to continue: " confirm
    
    if [[ "$confirm" != "YES" ]]; then
        log_error "Aborted by user"
        exit 1
    fi
    
    check_root
    check_device "$device"
    
    trap cleanup EXIT
    
    download_alpine
    create_partitions "$device"
    format_partitions "$device" "$spore_name"
    
    local mount_root
    mount_root=$(mount_partitions "$device")
    
    install_alpine "$mount_root"
    install_genomes "$mount_root"
    derive_genetics "$mount_root" "$spore_name"
    install_bootloader "$device" "$mount_root"
    create_autostart "$mount_root" "$spore_name"
    
    echo ""
    log_success "=============================================="
    log_success "   LiveSpore '$spore_name' Created!"
    log_success "=============================================="
    echo ""
    echo "Genetics:"
    echo "  Mito beacon:  $(sha256sum $mount_root/root/biomeOS/.family.seed | cut -c1-16) (passed)"
    echo "  Lineage seed: $(xxd -p $mount_root/root/biomeOS/.lineage.seed | head -c 16) (mixed)"
    echo ""
    echo "This LiveSpore is a TEMPORAL SIBLING of existing ColdSpores."
    echo "Same generation (Gen 1), but born later."
    echo ""
    echo "To boot: Insert USB and select from BIOS boot menu"
    echo ""
}

main "$@"

