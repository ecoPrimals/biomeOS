#!/bin/bash
# Deploy Node Atomic from USB Genetic Seed
# Part of genetic lineage deployment system

set -euo pipefail

# Configuration
FAMILY_ID="${BIOMEOS_FAMILY_ID:-nat0}"
NODE_ID="node"
USB_SEED="${BIOMEOS_USB_SEED:-/media/usb0/biomeos/.family.seed}"
RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
PLASMID_BIN="/home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🖥️  Node Atomic - Genetic Lineage Deployment"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Verify USB seed exists
if [ ! -f "$USB_SEED" ]; then
    echo "❌ USB seed not found: $USB_SEED"
    echo ""
    echo "💡 Generate test seed:"
    echo "   mkdir -p /tmp/biomeos-test"
    echo "   dd if=/dev/urandom of=/tmp/biomeos-test/.family.seed bs=32 count=1"
    echo "   export BIOMEOS_USB_SEED=/tmp/biomeos-test/.family.seed"
    exit 1
fi

echo "✅ USB Seed Found: $USB_SEED"
SEED_SIZE=$(stat -c %s "$USB_SEED")
echo "   Size: $SEED_SIZE bytes"

if [ "$SEED_SIZE" != "32" ]; then
    echo "❌ Invalid seed size (must be 32 bytes)"
    exit 1
fi

# Derive Node-specific seed
echo ""
echo "🧬 Deriving Node-Specific Seed..."
NODE_SEED="/tmp/biomeos-node-${FAMILY_ID}.seed"
DEPLOYMENT_BATCH=$(date +%Y%m%d)

echo "parent_seed: $USB_SEED, node_id: $NODE_ID, batch: $DEPLOYMENT_BATCH" | \
    sha256sum | \
    xxd -r -p | \
    head -c 32 > "$NODE_SEED"

chmod 600 "$NODE_SEED"
echo "   ✅ Node seed derived: $NODE_SEED"

# Deploy Node primals
echo ""
echo "🚀 Deploying Node Primals..."
echo ""

# 1. BearDog
echo "   🐻 Starting BearDog..."
BEARDOG_SOCKET="$RUNTIME_DIR/beardog-node.sock"
BEARDOG_FAMILY_SEED_FILE="$NODE_SEED" \
BEARDOG_FAMILY_ID="$FAMILY_ID" \
BEARDOG_NODE_ID="$NODE_ID" \
BEARDOG_SOCKET="$BEARDOG_SOCKET" \
"$PLASMID_BIN/primals/beardog-server" &

BEARDOG_PID=$!
echo "      PID: $BEARDOG_PID"
echo "      Socket: $BEARDOG_SOCKET"

sleep 2

if [ ! -S "$BEARDOG_SOCKET" ]; then
    echo "      ❌ BearDog socket not created"
    kill $BEARDOG_PID 2>/dev/null || true
    exit 1
fi
echo "      ✅ BearDog operational"

# 2. Songbird
echo ""
echo "   🐦 Starting Songbird..."
SONGBIRD_SOCKET="$RUNTIME_DIR/songbird-node.sock"
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

# 3. ToadStool
echo ""
echo "   🍄 Starting ToadStool..."
TOADSTOOL_SOCKET="$RUNTIME_DIR/toadstool-node.sock"
TOADSTOOL_SOCKET="$TOADSTOOL_SOCKET" \
TOADSTOOL_FAMILY_ID="$FAMILY_ID" \
"$PLASMID_BIN/toadstool" &

TOADSTOOL_PID=$!
echo "      PID: $TOADSTOOL_PID"
echo "      Socket: $TOADSTOOL_SOCKET"

sleep 2

if [ ! -S "$TOADSTOOL_SOCKET" ]; then
    echo "      ⚠️  ToadStool socket not created (may not be critical)"
    TOADSTOOL_PID=""
else
    echo "      ✅ ToadStool operational"
fi

# Summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Node Atomic Deployed Successfully!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🧬 Genetic Lineage:"
echo "   Family ID: $FAMILY_ID"
echo "   Node ID: $NODE_ID"
echo "   Parent Seed: $USB_SEED"
echo "   Child Seed: $NODE_SEED"
echo "   Deployment Batch: $DEPLOYMENT_BATCH"
echo ""
echo "🔌 Running Primals:"
echo "   • BearDog:  PID $BEARDOG_PID @ $BEARDOG_SOCKET"
echo "   • Songbird: PID $SONGBIRD_PID @ $SONGBIRD_SOCKET"
if [ -n "$TOADSTOOL_PID" ]; then
    echo "   • ToadStool: PID $TOADSTOOL_PID @ $TOADSTOOL_SOCKET"
fi
echo ""
echo "🎯 Next Steps:"
echo "   • Deploy Nest atomic (same USB seed)"
echo "   • Run lineage verification tests"
echo "   • Test cross-atomic communication"
echo ""
echo "🛑 To stop:"
if [ -n "$TOADSTOOL_PID" ]; then
    echo "   kill $BEARDOG_PID $SONGBIRD_PID $TOADSTOOL_PID"
else
    echo "   kill $BEARDOG_PID $SONGBIRD_PID"
fi
echo ""
echo "Different orders of the same architecture. 🍄🐸"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

