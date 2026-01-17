#!/bin/bash
# Deploy Nest Atomic from USB Genetic Seed
# Part of genetic lineage deployment system

set -euo pipefail

# Configuration
FAMILY_ID="${BIOMEOS_FAMILY_ID:-nat0}"
NODE_ID="nest"
USB_SEED="${BIOMEOS_USB_SEED:-/media/usb0/biomeos/.family.seed}"
RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
PLASMID_BIN="/home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🏠 Nest Atomic - Genetic Lineage Deployment"
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

# Derive Nest-specific seed
echo ""
echo "🧬 Deriving Nest-Specific Seed..."
NEST_SEED="/tmp/biomeos-nest-${FAMILY_ID}.seed"
DEPLOYMENT_BATCH=$(date +%Y%m%d)

echo "parent_seed: $USB_SEED, node_id: $NODE_ID, batch: $DEPLOYMENT_BATCH" | \
    sha256sum | \
    xxd -r -p | \
    head -c 32 > "$NEST_SEED"

chmod 600 "$NEST_SEED"
echo "   ✅ Nest seed derived: $NEST_SEED"

# Deploy Nest primals
echo ""
echo "🚀 Deploying Nest Primals..."
echo ""

# 1. BearDog
echo "   🐻 Starting BearDog..."
BEARDOG_SOCKET="$RUNTIME_DIR/beardog-nest.sock"
BEARDOG_FAMILY_SEED_FILE="$NEST_SEED" \
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
SONGBIRD_SOCKET="$RUNTIME_DIR/songbird-nest.sock"
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

# 3. NestGate
echo ""
echo "   🚪 Starting NestGate..."
NESTGATE_SOCKET="$RUNTIME_DIR/nestgate-nest.sock"
NESTGATE_SOCKET="$NESTGATE_SOCKET" \
NESTGATE_FAMILY_ID="$FAMILY_ID" \
"$PLASMID_BIN/primals/nestgate" &

NESTGATE_PID=$!
echo "      PID: $NESTGATE_PID"
echo "      Socket: $NESTGATE_SOCKET"

sleep 2

if [ ! -S "$NESTGATE_SOCKET" ]; then
    echo "      ⚠️  NestGate socket not created (may not be critical)"
    NESTGATE_PID=""
else
    echo "      ✅ NestGate operational"
fi

# Summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Nest Atomic Deployed Successfully!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🧬 Genetic Lineage:"
echo "   Family ID: $FAMILY_ID"
echo "   Node ID: $NODE_ID"
echo "   Parent Seed: $USB_SEED"
echo "   Child Seed: $NEST_SEED"
echo "   Deployment Batch: $DEPLOYMENT_BATCH"
echo ""
echo "🔌 Running Primals:"
echo "   • BearDog:  PID $BEARDOG_PID @ $BEARDOG_SOCKET"
echo "   • Songbird: PID $SONGBIRD_PID @ $SONGBIRD_SOCKET"
if [ -n "$NESTGATE_PID" ]; then
    echo "   • NestGate: PID $NESTGATE_PID @ $NESTGATE_SOCKET"
fi
echo ""
echo "🎯 All 3 Atomics Should Now Be Running!"
echo ""
echo "🔬 Verify Lineage Recognition:"
echo "   ./scripts/verify-lineage-cooperation.sh"
echo ""
echo "🛑 To stop:"
if [ -n "$NESTGATE_PID" ]; then
    echo "   kill $BEARDOG_PID $SONGBIRD_PID $NESTGATE_PID"
else
    echo "   kill $BEARDOG_PID $SONGBIRD_PID"
fi
echo ""
echo "Different orders of the same architecture. 🍄🐸"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

