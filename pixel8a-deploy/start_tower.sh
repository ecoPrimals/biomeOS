#!/bin/bash
# biomeOS Tower Atomic for Pixel 8a
# Run in Termux proot-distro (Debian/Ubuntu)
# Standard: PRIMAL_DEPLOYMENT_STANDARD v1.0

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ARCH="$(uname -m)"

# Environment
export FAMILY_ID="${FAMILY_ID:-pixel8a}"
export NODE_ID="${NODE_ID:-pixel}"
export RUST_LOG="${RUST_LOG:-info}"

# Socket directory (5-tier resolution per PRIMAL_DEPLOYMENT_STANDARD)
# Android/Termux typically doesn't have XDG_RUNTIME_DIR
if [ -n "$XDG_RUNTIME_DIR" ]; then
    SOCKET_DIR="$XDG_RUNTIME_DIR/biomeos"
elif [ -d "/data/local/tmp" ]; then
    SOCKET_DIR="/data/local/tmp/biomeos"  # Android standard
else
    SOCKET_DIR="/tmp/biomeos"  # Fallback
fi

echo "🚀 Starting Tower Atomic on Pixel 8a"
echo "═══════════════════════════════════════════════════"
echo "Architecture: $ARCH (aarch64)"
echo "Family ID: $FAMILY_ID"
echo "Socket Dir: $SOCKET_DIR"
echo ""

# Create socket directory
mkdir -p "$SOCKET_DIR"

# Export socket paths
export BEARDOG_SOCKET="$SOCKET_DIR/beardog-$FAMILY_ID.sock"
export SONGBIRD_SOCKET="$SOCKET_DIR/songbird-$FAMILY_ID.sock"
export NEURAL_API_SOCKET="$SOCKET_DIR/neural-api-$FAMILY_ID.sock"

echo "🐻🐕 Starting BearDog..."
$SCRIPT_DIR/primals/beardog server \
    --socket "$BEARDOG_SOCKET" \
    --family-id "$FAMILY_ID" > "$SOCKET_DIR/../beardog.log" 2>&1 &
BEARDOG_PID=$!
echo "  PID: $BEARDOG_PID"
sleep 3

if [ -S "$BEARDOG_SOCKET" ]; then
    echo "  ✅ BearDog ready!"
else
    echo "  ❌ BearDog socket not found"
    exit 1
fi

echo ""
echo "🧠 Starting Neural API..."
$SCRIPT_DIR/neural-api-server \
    --socket "$NEURAL_API_SOCKET" \
    --family-id "$FAMILY_ID" \
    --graphs-dir "$SCRIPT_DIR/graphs" > "$SOCKET_DIR/../neural-api.log" 2>&1 &
NEURAL_API_PID=$!
echo "  PID: $NEURAL_API_PID"
sleep 2

echo ""
echo "🐦 Starting Songbird..."
export BEARDOG_MODE=neural
$SCRIPT_DIR/primals/songbird server \
    --socket "$SONGBIRD_SOCKET" > "$SOCKET_DIR/../songbird.log" 2>&1 &
SONGBIRD_PID=$!
echo "  PID: $SONGBIRD_PID"
sleep 3

if [ -S "$SONGBIRD_SOCKET" ]; then
    echo "  ✅ Songbird ready!"
else
    echo "  ⚠️  Songbird socket not found (may take longer)"
fi

echo ""
echo "═══════════════════════════════════════════════════"
echo "✅ Tower Atomic running on Pixel 8a!"
echo "═══════════════════════════════════════════════════"
echo ""
echo "Sockets:"
ls -la "$SOCKET_DIR"/*.sock 2>/dev/null || echo "(waiting for sockets...)"
echo ""
echo "PIDs: BearDog=$BEARDOG_PID, NeuralAPI=$NEURAL_API_PID, Songbird=$SONGBIRD_PID"
echo ""
echo "Note: All primals use Unix sockets (no HTTP ports)"
