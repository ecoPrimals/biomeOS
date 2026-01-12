#!/bin/bash
# Deploy All 3 Atomics from Single USB Genetic Seed
# Master deployment script for complete genetic lineage demonstration

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FAMILY_ID="${BIOMEOS_FAMILY_ID:-nat0}"
USB_SEED="${BIOMEOS_USB_SEED:-/media/usb0/biomeos/.family.seed}"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧬 Complete Genetic Lineage Deployment"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Deploying ALL 3 atomics from single USB seed"
echo "Family: $FAMILY_ID"
echo ""

# Check if USB seed exists
if [ ! -f "$USB_SEED" ]; then
    echo "⚠️  USB seed not found: $USB_SEED"
    echo ""
    echo "💡 Generating test seed for demonstration..."
    
    # Create test seed location
    TEST_SEED_DIR="/tmp/biomeos-test"
    mkdir -p "$TEST_SEED_DIR"
    
    # Generate 32-byte seed
    dd if=/dev/urandom of="$TEST_SEED_DIR/.family.seed" bs=32 count=1 2>/dev/null
    chmod 600 "$TEST_SEED_DIR/.family.seed"
    
    export BIOMEOS_USB_SEED="$TEST_SEED_DIR/.family.seed"
    USB_SEED="$TEST_SEED_DIR/.family.seed"
    
    echo "   ✅ Test seed created: $USB_SEED"
    echo ""
    echo "   ⚠️  NOTE: This is a TEST seed for demonstration only!"
    echo "   For production, use a real USB seed."
    echo ""
fi

echo "✅ USB Seed: $USB_SEED"
SEED_HASH=$(sha256sum "$USB_SEED" | awk '{print substr($1,1,16)}')
echo "   Hash Preview: $SEED_HASH..."
echo ""

# Deploy sequence
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 Deployment Sequence Starting..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Step 1/3: Deploying Tower Atomic..."
echo ""
"$SCRIPT_DIR/deploy-tower-lineage.sh"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Step 2/3: Deploying Node Atomic..."
echo ""
"$SCRIPT_DIR/deploy-node-lineage.sh"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Step 3/3: Deploying Nest Atomic..."
echo ""
"$SCRIPT_DIR/deploy-nest-lineage.sh"
echo ""

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎊 ALL 3 ATOMICS DEPLOYED!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "🧬 Genetic Lineage Architecture:"
echo ""
echo "   USB Seed (Parent DNA)"
echo "       ↓ SHA256(seed || node_id || batch)"
echo "       ├─→ Tower (unique child seed)"
echo "       ├─→ Node  (unique child seed)"
echo "       └─→ Nest  (unique child seed)"
echo ""
echo "   Properties:"
echo "   • All share cryptographic lineage"
echo "   • Each has unique identity"
echo "   • Automatic sibling recognition"
echo "   • Secure cooperation enabled"
echo ""

echo "🔌 Running Services:"
RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
echo ""
echo "   Tower @ $RUNTIME_DIR:"
echo "      • beardog-tower.sock"
echo "      • songbird-tower.sock"
echo ""
echo "   Node @ $RUNTIME_DIR:"
echo "      • beardog-node.sock"
echo "      • songbird-node.sock"
echo "      • toadstool-node.sock"
echo ""
echo "   Nest @ $RUNTIME_DIR:"
echo "      • beardog-nest.sock"
echo "      • songbird-nest.sock"
echo "      • nestgate-nest.sock"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔬 Next Step: Verify Lineage Recognition"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Run verification tests:"
echo "   $SCRIPT_DIR/verify-lineage-cooperation.sh"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Offer to run verification
read -p "Run lineage verification now? [Y/n] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]] || [[ -z $REPLY ]]; then
    echo ""
    "$SCRIPT_DIR/verify-lineage-cooperation.sh"
else
    echo ""
    echo "Verification skipped. Run manually when ready:"
    echo "   $SCRIPT_DIR/verify-lineage-cooperation.sh"
    echo ""
fi

echo "Different orders of the same architecture. 🍄🐸"
echo ""

