#!/bin/bash
#═══════════════════════════════════════════════════════════════════════════════
# LiveSpore USB - Start Tower Atomic
# Architecture: aarch64 (ARM64)
# Standard: Evolved Genetic Standard v2.0
#
# Seeds loaded from livespore-usb root:
#   .family.seed  - mitochondrial DNA (shared across family)
#   .beacon.seed  - beacon identity (derived from family)
#   .lineage.seed - nuclear DNA (unique to THIS device)
#═══════════════════════════════════════════════════════════════════════════════

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
USB_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
PRIMAL_DIR="$SCRIPT_DIR/../primals"
ARCH="$(uname -m)"

# Evolved standard environment
export FAMILY_ID="${FAMILY_ID:-1894e909e454}"
export NODE_ID="${NODE_ID:-usb}"
export RUST_LOG="${RUST_LOG:-info}"
export BIOMEOS_ROOT="$USB_ROOT"

# Socket directory (5-tier resolution)
if [ -n "$XDG_RUNTIME_DIR" ]; then
    SOCKET_DIR="$XDG_RUNTIME_DIR/biomeos"
elif [ -d "/run/user/$(id -u)" ]; then
    SOCKET_DIR="/run/user/$(id -u)/biomeos"
elif [ -d "/data/local/tmp" ]; then
    SOCKET_DIR="/data/local/tmp/biomeos"
else
    SOCKET_DIR="/tmp/biomeos"
fi

echo "═══════════════════════════════════════════════════════════════"
echo "🧬 LiveSpore USB - Evolved Genetic Standard v2.0"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Architecture:  $ARCH (aarch64)"
echo "Family ID:     $FAMILY_ID"
echo "Node ID:       $NODE_ID"
echo "BIOMEOS_ROOT:  $BIOMEOS_ROOT"
echo "Socket Dir:    $SOCKET_DIR"
echo "Primal Dir:    $PRIMAL_DIR"
echo ""

# Verify seeds
echo "Seeds:"
[ -f "$USB_ROOT/.family.seed" ] && echo "  .family.seed    ✅ (mitochondrial)" || { echo "  .family.seed    ❌ MISSING"; exit 1; }
[ -f "$USB_ROOT/.beacon.seed" ] && echo "  .beacon.seed    ✅ (beacon)" || echo "  .beacon.seed    ⚠️  (will use family seed)"
[ -f "$USB_ROOT/.lineage.seed" ] && echo "  .lineage.seed   ✅ (nuclear)" || echo "  .lineage.seed   ⚠️  (will derive on first run)"
echo ""

# Create socket directory
mkdir -p "$SOCKET_DIR"

# Kill stale processes
pkill -f "beardog server" 2>/dev/null || true
pkill -f "songbird server" 2>/dev/null || true
rm -f "$SOCKET_DIR"/*.sock 2>/dev/null
sleep 1

# Start BearDog
echo "🐻 Starting BearDog..."
export BEARDOG_SOCKET="$SOCKET_DIR/beardog.sock"

"$PRIMAL_DIR/beardog" server \
    --socket "$BEARDOG_SOCKET" > /tmp/beardog_usb.log 2>&1 &
BEARDOG_PID=$!
echo "  PID: $BEARDOG_PID"

# Wait for BearDog
tries=0
while [ ! -S "$BEARDOG_SOCKET" ] && [ $tries -lt 10 ]; do
    sleep 1
    tries=$((tries + 1))
done

if [ -S "$BEARDOG_SOCKET" ]; then
    echo "  ✅ BearDog operational"
else
    echo "  ❌ BearDog socket not created after ${tries}s"
    tail -10 /tmp/beardog_usb.log
    exit 1
fi

# Start Songbird
echo "🐦 Starting Songbird..."
SONGBIRD_PORT="${SONGBIRD_PORT:-3492}"
export SONGBIRD_SOCKET="$SOCKET_DIR/songbird.sock"
export SONGBIRD_SECURITY_PROVIDER="$BEARDOG_SOCKET"
export BIOMEOS_BIND_ALL=true
export BIND_ADDRESS="::"

"$PRIMAL_DIR/songbird" server \
    --port "$SONGBIRD_PORT" \
    --socket "$SONGBIRD_SOCKET" > /tmp/songbird_usb.log 2>&1 &
SONGBIRD_PID=$!
echo "  PID: $SONGBIRD_PID"

# Wait for Songbird
tries=0
while [ ! -S "$SONGBIRD_SOCKET" ] && [ $tries -lt 10 ]; do
    sleep 1
    tries=$((tries + 1))
done

if [ -S "$SONGBIRD_SOCKET" ]; then
    echo "  ✅ Songbird operational (TCP :$SONGBIRD_PORT + IPC)"
else
    echo "  ⚠️  Songbird socket pending after ${tries}s"
fi

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "✅ LiveSpore USB - TOWER ATOMIC OPERATIONAL (aarch64)"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Sockets:"
ls -lh "$SOCKET_DIR"/*.sock 2>/dev/null || echo "  (checking...)"
echo ""
echo "Health:"
echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' | nc -U "$BEARDOG_SOCKET" -w 3 -q 1 2>/dev/null | head -1 || echo "  (nc not available)"
echo ""
echo "PIDs: BearDog=$BEARDOG_PID, Songbird=$SONGBIRD_PID"
