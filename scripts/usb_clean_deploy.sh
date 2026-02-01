#!/bin/bash
# USB Live Spore Clean Deployment Script
# Date: January 31, 2026
# Purpose: Deploy biomeOS + NUCLEUS using genomeBin v3.0 standard

set -euo pipefail

echo "═══════════════════════════════════════════════════════════════════"
echo "🧬 USB LIVE SPORE CLEAN DEPLOYMENT"
echo "═══════════════════════════════════════════════════════════════════"
echo ""

# Configuration
USB_MOUNT="/media/eastgate/biomeOS1"
USB_BIOMEOS="${USB_MOUNT}/biomeOS"
WORKSPACE="$HOME/Development/ecoPrimals/phase2/biomeOS"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}✅${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}⚠️${NC} $1"
}

log_error() {
    echo -e "${RED}❌${NC} $1"
}

# Validate USB mount
if [[ ! -d "$USB_MOUNT" ]]; then
    log_error "USB not mounted at $USB_MOUNT"
    echo "Available mounts:"
    mount | grep /media/eastgate
    exit 1
fi

log_info "USB mounted at: $USB_MOUNT"

# Step 1: Backup current deployment
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📦 Step 1: Backup Current Deployment"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [[ -d "$USB_BIOMEOS" ]]; then
    BACKUP_DIR="/tmp/usb_backup_$(date +%Y%m%d_%H%M%S)"
    log_info "Backing up to: $BACKUP_DIR"
    cp -r "$USB_BIOMEOS" "$BACKUP_DIR"
    log_info "Backup complete"
else
    log_warn "No existing deployment found (fresh USB)"
fi

# Step 2: Clean old binaries
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧹 Step 2: Clean Old Binaries"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Remove old bin directory (old binaries)
if [[ -d "$USB_BIOMEOS/bin" ]]; then
    log_info "Removing old bin/ directory..."
    rm -rf "$USB_BIOMEOS/bin"
fi

# Remove old plasmidBin (old genomeBins)
if [[ -d "$USB_BIOMEOS/plasmidBin" ]]; then
    log_info "Removing old plasmidBin/ directory..."
    rm -rf "$USB_BIOMEOS/plasmidBin"
fi

# Remove old primals directory
if [[ -d "$USB_BIOMEOS/primals" ]]; then
    log_info "Removing old primals/ directory..."
    rm -rf "$USB_BIOMEOS/primals"
fi

# Keep config, graphs, logs, certs
log_info "Preserving: config/, graphs/, logs/, certs/, .family.seed"

# Step 3: Create new directory structure
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📁 Step 3: Create New Directory Structure"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

mkdir -p "$USB_BIOMEOS/genomeBins"
mkdir -p "$USB_BIOMEOS/extracted"
mkdir -p "$USB_BIOMEOS/primals"

log_info "Created: genomeBins/ (new genomeBin storage)"
log_info "Created: extracted/ (extraction target)"
log_info "Created: primals/ (primal binaries)"

# Step 4: Copy genomeBins
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧬 Step 4: Deploy genomeBins"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# biomeOS system
log_info "Copying biomeos-complete.genome (3.8 MB)..."
cp "$WORKSPACE/plasmidBin/biomeos-complete.genome" "$USB_BIOMEOS/genomeBins/"

# NUCLEUS ecosystem
log_info "Copying nucleus.genome (31 MB)..."
cp "$WORKSPACE/plasmidBin/nucleus.genome" "$USB_BIOMEOS/genomeBins/"

# Individual primals (optional, for flexibility)
log_info "Copying individual primal genomeBins..."
cp "$WORKSPACE/plasmidBin/beardog-linux-multi.genome" "$USB_BIOMEOS/genomeBins/"
cp "$WORKSPACE/plasmidBin/songbird-linux.genome" "$USB_BIOMEOS/genomeBins/"
cp "$WORKSPACE/plasmidBin/toadstool-linux.genome" "$USB_BIOMEOS/genomeBins/"
cp "$WORKSPACE/plasmidBin/nestgate-linux.genome" "$USB_BIOMEOS/genomeBins/"

# Make executable
chmod +x "$USB_BIOMEOS/genomeBins"/*.genome

log_info "All genomeBins deployed!"

# Step 5: Extract biomeOS system
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📦 Step 5: Extract biomeOS System"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cd "$USB_BIOMEOS/genomeBins"

# Note: Current genomeBin v3.0 requires biomeos CLI to extract
# Copy biomeos CLI first (from local build)
log_info "Installing biomeos CLI..."
cp "$WORKSPACE/target/x86_64-unknown-linux-musl/release/biomeos" "$USB_BIOMEOS/extracted/"
chmod +x "$USB_BIOMEOS/extracted/biomeos"

log_info "biomeOS CLI installed: $USB_BIOMEOS/extracted/biomeos"

# Use CLI to extract biomeOS system components
# (Future: genomeBins will be self-extracting)

# Step 6: Display summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ DEPLOYMENT COMPLETE"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo ""
echo "USB Structure:"
ls -lh "$USB_BIOMEOS/genomeBins/"

echo ""
echo "Deployed genomeBins:"
echo "  ✅ biomeos-complete.genome   (3.8 MB - biomeOS system)"
echo "  ✅ nucleus.genome            (31 MB - all 4 primals)"
echo "  ✅ beardog-linux-multi.genome (3.2 MB)"
echo "  ✅ songbird-linux.genome      (7.5 MB)"
echo "  ✅ toadstool-linux.genome     (3.4 MB)"
echo "  ✅ nestgate-linux.genome      (3.6 MB)"

echo ""
echo "Next Steps:"
echo "  1. cd /media/eastgate/biomeOS1/biomeOS/extracted"
echo "  2. ./biomeos genome list (verify genomeBins)"
echo "  3. ./biomeos genome extract ../genomeBins/nucleus.genome --output ../primals/"
echo "  4. Start primals and neuralAPI"
echo "  5. Validate full system"

echo ""
log_info "USB Live Spore clean deployment complete! ✅"
echo ""
echo "═══════════════════════════════════════════════════════════════════"
