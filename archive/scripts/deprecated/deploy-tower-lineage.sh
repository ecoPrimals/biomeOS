#!/bin/bash
# Deploy Tower Atomic from USB Genetic Seed
# Part of genetic lineage deployment system

set -euo pipefail

# Configuration
FAMILY_ID="${BIOMEOS_FAMILY_ID:-nat0}"
NODE_ID="tower"
USB_SEED="${BIOMEOS_USB_SEED:-/media/usb0/biomeos/.family.seed}"
RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
PLASMID_BIN="/home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🏰 Tower Atomic - Genetic Lineage Deployment"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Verify USB seed exists
if [ ! -f "$USB_SEED" ]; then
    echo "❌ USB seed not found: $USB_SEED"
    echo ""
    echo "💡 Options:"
    echo "   1. Insert USB with family seed"
    echo "   2. Set BIOMEOS_USB_SEED env var"
    echo "   3. Generate test seed:"
    echo "      mkdir -p /tmp/biomeos-test"
    echo "      dd if=/dev/urandom of=/tmp/biomeos-test/.family.seed bs=32 count=1"
    echo "      export BIOMEOS_USB_SEED=/tmp/biomeos-test/.family.seed"
    exit 1
fi

echo "✅ USB Seed Found: $USB_SEED"
SEED_SIZE=$(stat -c %s "$USB_SEED")
echo "   Size: $SEED_SIZE bytes (expected: 32)"

if [ "$SEED_SIZE" != "32" ]; then
    echo "❌ Invalid seed size (must be 32 bytes)"
    exit 1
fi

# Derive Tower-specific seed
echo ""
echo "🧬 Deriving Tower-Specific Seed..."
TOWER_SEED="/tmp/biomeos-tower-${FAMILY_ID}.seed"

# Use biomeOS seed derivation (SHA256-based genetic mixing)
# Formula: child_seed = SHA256(parent_seed || node_id || batch)
DEPLOYMENT_BATCH=$(date +%Y%m%d)
echo "parent_seed: $USB_SEED, node_id: $NODE_ID, batch: $DEPLOYMENT_BATCH" | \
    sha256sum | \
    xxd -r -p | \
    head -c 32 > "$TOWER_SEED"

chmod 600 "$TOWER_SEED"
echo "   ✅ Tower seed derived: $TOWER_SEED"
echo "   🔒 Permissions: 600 (owner only)"

# Deploy Tower primals
echo ""
echo "🚀 Deploying Tower Primals..."
echo ""

# 1. BearDog (encryption foundation)
echo "   🐻 Starting BearDog..."
BEARDOG_SOCKET="$RUNTIME_DIR/beardog-tower.sock"
BEARDOG_FAMILY_SEED_FILE="$TOWER_SEED" \
BEARDOG_FAMILY_ID="$FAMILY_ID" \
BEARDOG_NODE_ID="$NODE_ID" \
BEARDOG_SOCKET="$BEARDOG_SOCKET" \
"$PLASMID_BIN/primals/beardog-server" &

BEARDOG_PID=$!
echo "      PID: $BEARDOG_PID"
echo "      Socket: $BEARDOG_SOCKET"

sleep 2

# Verify BearDog started
if [ ! -S "$BEARDOG_SOCKET" ]; then
    echo "      ❌ BearDog socket not created"
    kill $BEARDOG_PID 2>/dev/null || true
    exit 1
fi
echo "      ✅ BearDog operational"

# 2. Songbird (discovery)
echo ""
echo "   🐦 Starting Songbird..."
SONGBIRD_SOCKET="$RUNTIME_DIR/songbird-tower.sock"
SONGBIRD_SOCKET="$SONGBIRD_SOCKET" \
SONGBIRD_FAMILY_ID="$FAMILY_ID" \
SONGBIRD_SECURITY_PROVIDER="$BEARDOG_SOCKET" \
"$PLASMID_BIN/primals/songbird-orchestrator" &

SONGBIRD_PID=$!
echo "      PID: $SONGBIRD_PID"
echo "      Socket: $SONGBIRD_SOCKET"

sleep 2

if [ ! -S "$SONGBIRD_SOCKET" ]; then
    echo "      ❌ Songbird socket not created"
    kill $BEARDOG_PID $SONGBIRD_PID 2>/dev/null || true
    exit 1
fi
echo "      ✅ Songbird operational"

# Summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Tower Atomic Deployed Successfully!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🧬 Genetic Lineage:"
echo "   Family ID: $FAMILY_ID"
echo "   Node ID: $NODE_ID"
echo "   Parent Seed: $USB_SEED"
echo "   Child Seed: $TOWER_SEED"
echo "   Deployment Batch: $DEPLOYMENT_BATCH"
echo ""
echo "🔌 Running Primals:"
echo "   • BearDog:  PID $BEARDOG_PID @ $BEARDOG_SOCKET"
echo "   • Songbird: PID $SONGBIRD_PID @ $SONGBIRD_SOCKET"
echo ""
echo "🎯 Next Steps:"
echo "   1. Deploy Node atomic (same USB seed)"
echo "   2. Deploy Nest atomic (same USB seed)"
echo "   3. Verify cross-atomic lineage recognition"
echo "   4. Test secure cooperation"
echo ""
echo "🛑 To stop:"
echo "   kill $BEARDOG_PID $SONGBIRD_PID"
echo ""
echo "Different orders of the same architecture. 🍄🐸"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

