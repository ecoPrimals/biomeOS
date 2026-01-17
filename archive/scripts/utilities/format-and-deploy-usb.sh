#!/usr/bin/env bash
#
# Format and Prepare USB for biomeOS Deployment
# This script formats the USB with ext4 (Linux native) for proper execute permissions
#

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}   BiomeOS USB Spore - Format & Deploy${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo -e "${RED}❌ This script must be run as root (for formatting)${NC}"
    echo -e "${YELLOW}   Usage: sudo $0 /dev/sdX [label]${NC}"
    exit 1
fi

# Get device and optional label
DEVICE="${1:-}"
LABEL="${2:-biomeOS}"

if [ -z "$DEVICE" ]; then
    echo -e "${RED}❌ Error: No device specified${NC}"
    echo ""
    echo -e "${YELLOW}Available devices:${NC}"
    lsblk -o NAME,SIZE,TYPE,MOUNTPOINT,FSTYPE | grep -E "disk|part"
    echo ""
    echo -e "${YELLOW}Usage: sudo $0 /dev/sdX [label]${NC}"
    echo -e "${YELLOW}Example: sudo $0 /dev/sda biomeOS${NC}"
    exit 1
fi

# Verify device exists
if [ ! -b "$DEVICE" ]; then
    echo -e "${RED}❌ Error: Device $DEVICE does not exist${NC}"
    exit 1
fi

# Safety check - don't format system disk
if [[ "$DEVICE" =~ nvme0n1 ]] || [[ "$DEVICE" =~ sda ]] && [ ! "$(lsblk -no TYPE "$DEVICE")" = "part" ]; then
    echo -e "${RED}❌ SAFETY CHECK FAILED${NC}"
    echo -e "${RED}   Device $DEVICE appears to be a system disk!${NC}"
    echo -e "${YELLOW}   Please specify the partition (e.g., /dev/sda1, not /dev/sda)${NC}"
    exit 1
fi

# Show device info
echo -e "${BLUE}📊 Device Information:${NC}"
lsblk -o NAME,SIZE,TYPE,MOUNTPOINT,FSTYPE "$DEVICE"
echo ""

# Confirm
echo -e "${YELLOW}⚠️  WARNING: This will ERASE ALL DATA on $DEVICE${NC}"
echo -e "${YELLOW}   Device: $DEVICE${NC}"
echo -e "${YELLOW}   New label: $LABEL${NC}"
echo -e "${YELLOW}   Filesystem: ext4 (Linux native, supports execute permissions)${NC}"
echo ""
read -p "Are you ABSOLUTELY SURE you want to continue? (type 'yes' to confirm): " -r
echo
if [ "$REPLY" != "yes" ]; then
    echo -e "${BLUE}❌ Aborted by user${NC}"
    exit 1
fi

# Unmount if mounted
echo -e "${BLUE}🔓 Unmounting if mounted...${NC}"
umount "$DEVICE" 2>/dev/null || true
echo -e "${GREEN}✅ Unmounted${NC}"

# Format with ext4
echo -e "${BLUE}💾 Formatting $DEVICE with ext4...${NC}"
mkfs.ext4 -L "$LABEL" -F "$DEVICE"
echo -e "${GREEN}✅ Formatted with ext4${NC}"

# Wait for udev
sleep 2

# Create mount point
MOUNT_POINT="/media/$SUDO_USER/$LABEL"
mkdir -p "$MOUNT_POINT"
echo -e "${GREEN}✅ Created mount point: $MOUNT_POINT${NC}"

# Mount
echo -e "${BLUE}📂 Mounting...${NC}"
mount "$DEVICE" "$MOUNT_POINT"
chown "$SUDO_USER:$SUDO_USER" "$MOUNT_POINT"
echo -e "${GREEN}✅ Mounted at $MOUNT_POINT${NC}"

# Prepare biomeOS deployment
echo ""
echo -e "${BLUE}📦 Preparing biomeOS deployment package...${NC}"

# Switch to non-root user for deployment prep
sudo -u "$SUDO_USER" bash << EOF
cd /home/$SUDO_USER/Development/ecoPrimals/phase2/biomeOS
./scripts/prepare-usb-proper.sh "$MOUNT_POINT"
EOF

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ USB Spore Preparation Complete!${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo -e "${BLUE}📊 Summary:${NC}"
echo -e "   Device:     ${GREEN}$DEVICE${NC}"
echo -e "   Filesystem: ${GREEN}ext4${NC}"
echo -e "   Label:      ${GREEN}$LABEL${NC}"
echo -e "   Mount:      ${GREEN}$MOUNT_POINT${NC}"
echo -e "   Package:    ${GREEN}~36M (Tower + BearDog + Songbird)${NC}"
echo ""
echo -e "${BLUE}✨ Advantages of ext4:${NC}"
echo -e "   ✅ Execute permissions (binaries run directly from USB)"
echo -e "   ✅ Linux native filesystem (optimal for tower-to-tower)"
echo -e "   ✅ File ownership and permissions preserved"
echo -e "   ✅ Symbolic links supported"
echo -e "   ✅ Better performance than FAT32"
echo ""
echo -e "${YELLOW}📝 Note: ext4 is Linux-only (not readable on Windows/Mac)${NC}"
echo -e "${YELLOW}   For cross-platform, use FAT32 but copy to /tmp on deployment${NC}"
echo ""
echo -e "${GREEN}🚀 Ready to deploy to Tower 2!${NC}"
echo ""
echo -e "${BLUE}To unmount before removal:${NC}"
echo -e "   sudo umount $MOUNT_POINT"
echo ""

