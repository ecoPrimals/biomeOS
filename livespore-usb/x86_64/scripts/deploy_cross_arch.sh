#!/bin/bash
# =============================================================================
# LiveSpore Cross-Architecture Deployment Script
# =============================================================================
#
# This script automatically detects the system architecture and deploys
# the appropriate primal binaries from the LiveSpore into the plasmidBin
# directory (the canonical binary source for biomeOS NUCLEUS).
#
# Usage: ./scripts/deploy_cross_arch.sh [target_dir]
#
# Default target: ./plasmidBin/primals/ (relative to biomeOS repo root)
# Override: BIOMEOS_PLASMID_DIR or explicit target_dir argument
#
# AGPL-3.0-or-later License
# =============================================================================

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIVESPORE_ROOT="$(dirname "$SCRIPT_DIR")"

echo -e "${BLUE}=========================================${NC}"
echo -e "${BLUE}   LiveSpore Cross-Arch Deployment${NC}"
echo -e "${BLUE}=========================================${NC}"
echo ""

# Detect architecture
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)
        ARCH_DIR="x86_64"
        echo -e "${GREEN}✓ Detected architecture: x86_64 (Intel/AMD)${NC}"
        ;;
    aarch64|arm64)
        ARCH_DIR="aarch64"
        echo -e "${GREEN}✓ Detected architecture: aarch64 (ARM64)${NC}"
        ;;
    armv7l|armhf)
        echo -e "${RED}✗ ARM32 not yet supported${NC}"
        exit 1
        ;;
    *)
        echo -e "${RED}✗ Unsupported architecture: $ARCH${NC}"
        exit 1
        ;;
esac

# Check if primals exist for this architecture
PRIMAL_DIR="$LIVESPORE_ROOT/primals/$ARCH_DIR"
if [[ ! -d "$PRIMAL_DIR" ]]; then
    echo -e "${RED}✗ No primals found for $ARCH_DIR${NC}"
    echo "  Expected: $PRIMAL_DIR"
    exit 1
fi

# Determine target directory: plasmidBin/primals/ is the post-primordial canonical path.
# biomeOS binary resolution: plasmidBin/primals/ → target/release/ → $PATH
TARGET_DIR="${1:-}"
if [[ -z "$TARGET_DIR" ]]; then
    if [[ -n "${BIOMEOS_PLASMID_DIR:-}" ]]; then
        TARGET_DIR="$BIOMEOS_PLASMID_DIR/primals"
    else
        # Walk up from livespore-usb/x86_64/scripts/ to repo root
        REPO_ROOT="$(cd "$LIVESPORE_ROOT/../.." && pwd)"
        TARGET_DIR="$REPO_ROOT/plasmidBin/primals"
    fi
    mkdir -p "$TARGET_DIR"
fi

echo -e "${BLUE}Target directory: $TARGET_DIR${NC}"
echo ""

# Create target directory if it doesn't exist
if [[ ! -d "$TARGET_DIR" ]]; then
    mkdir -p "$TARGET_DIR" 2>/dev/null || true
fi

# Check if target is writable
if [[ ! -w "$TARGET_DIR" ]]; then
    echo -e "${YELLOW}⚠ Target directory not writable. Trying with sudo...${NC}"
    SUDO="sudo"
    $SUDO mkdir -p "$TARGET_DIR"
else
    SUDO=""
fi

# Deploy primals
echo -e "${BLUE}Deploying primals...${NC}"
DEPLOYED=0
for primal in "$PRIMAL_DIR"/*; do
    if [[ -f "$primal" && -x "$primal" ]]; then
        name=$(basename "$primal")
        
        # Backup existing binary if present
        if [[ -f "$TARGET_DIR/$name" ]]; then
            $SUDO mv "$TARGET_DIR/$name" "$TARGET_DIR/$name.bak.$(date +%s)"
            echo -e "  ${YELLOW}↻ Backed up existing: $name${NC}"
        fi
        
        # Copy and set permissions
        $SUDO cp "$primal" "$TARGET_DIR/$name"
        $SUDO chmod 755 "$TARGET_DIR/$name"
        
        # Verify
        if [[ -x "$TARGET_DIR/$name" ]]; then
            version=$("$TARGET_DIR/$name" --version 2>/dev/null | head -1) || version="installed"
            echo -e "  ${GREEN}✓ $name: $version${NC}"
            DEPLOYED=$((DEPLOYED + 1))
        else
            echo -e "  ${RED}✗ Failed to deploy: $name${NC}"
        fi
    fi
done

echo ""
echo -e "${BLUE}=========================================${NC}"
echo -e "${GREEN}✓ Deployed $DEPLOYED primals to $TARGET_DIR${NC}"
echo -e "${BLUE}=========================================${NC}"

# Note: plasmidBin/primals/ is discovered by biomeOS directly — no $PATH entry needed.
echo -e "${GREEN}✓ biomeOS discovers primals from plasmidBin/primals/ automatically.${NC}"

# Copy configuration files
if [[ -d "$LIVESPORE_ROOT/config" ]]; then
    CONFIG_TARGET="${XDG_CONFIG_HOME:-$HOME/.config}/biomeos"
    mkdir -p "$CONFIG_TARGET"
    cp -r "$LIVESPORE_ROOT/config"/* "$CONFIG_TARGET/" 2>/dev/null || true
    echo ""
    echo -e "${GREEN}✓ Configuration copied to: $CONFIG_TARGET${NC}"
fi

# Copy graphs
if [[ -d "$LIVESPORE_ROOT/graphs" ]]; then
    GRAPHS_TARGET="${XDG_DATA_HOME:-$HOME/.local/share}/biomeos/graphs"
    mkdir -p "$GRAPHS_TARGET"
    cp -r "$LIVESPORE_ROOT/graphs"/* "$GRAPHS_TARGET/" 2>/dev/null || true
    echo -e "${GREEN}✓ Graphs copied to: $GRAPHS_TARGET${NC}"
fi

echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "  1. Start NUCLEUS: biomeos nucleus start --mode tower --node-id tower1"
echo "  2. Or full stack: biomeos nucleus start --mode full --node-id tower1"
echo ""

