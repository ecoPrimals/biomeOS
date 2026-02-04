#!/bin/bash
# LiveSpore USB - Start Tower Atomic
# Architecture: aarch64 (ARM64)
# Components: BearDog + Songbird
# Standard: PRIMAL_DEPLOYMENT_STANDARD v1.0

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PRIMAL_DIR="$SCRIPT_DIR/../primals"
ARCH="$(uname -m)"

# Environment
export FAMILY_ID="${FAMILY_ID:-livespore}"
export NODE_ID="${NODE_ID:-$(hostname)}"
export RUST_LOG="${RUST_LOG:-info}"

# Socket directory (5-tier resolution per PRIMAL_DEPLOYMENT_STANDARD)
if [ -n "$XDG_RUNTIME_DIR" ]; then
    SOCKET_DIR="$XDG_RUNTIME_DIR/biomeos"
elif [ -d "/run/user/$(id -u)" ]; then
    SOCKET_DIR="/run/user/$(id -u)/biomeos"
elif [ -d "/data/local/tmp" ]; then
    SOCKET_DIR="/data/local/tmp/biomeos"  # Android
else
    SOCKET_DIR="/tmp/biomeos"
fi

echo "🚀 Starting Tower Atomic from LiveSpore USB"
echo "═══════════════════════════════════════════════════"
echo "Architecture: $ARCH (aarch64)"
echo "Family ID: $FAMILY_ID"
echo "Node ID: $NODE_ID"
echo "Socket Dir: $SOCKET_DIR"
echo ""

# Create socket directory
mkdir -p "$SOCKET_DIR"

# Start BearDog
echo "🐻 Starting BearDog..."
export BEARDOG_SOCKET="$SOCKET_DIR/beardog-$FAMILY_ID.sock"
nohup "$PRIMAL_DIR/beardog" server \
    --socket "$BEARDOG_SOCKET" \
    --family-id "$FAMILY_ID" > /tmp/beardog_livespore.log 2>&1 &
BEARDOG_PID=$!
echo "  PID: $BEARDOG_PID"

# Wait for BearDog socket
sleep 4
if [ ! -e "$BEARDOG_SOCKET" ]; then
  echo "❌ BearDog socket not created at $BEARDOG_SOCKET"
  tail -10 /tmp/beardog_livespore.log
  exit 1
fi
echo "  ✅ BearDog operational"

# Start Songbird
echo "🎵 Starting Songbird..."
export SONGBIRD_SOCKET="$SOCKET_DIR/songbird-$FAMILY_ID.sock"
export SONGBIRD_SECURITY_PROVIDER=beardog
export BEARDOG_MODE=neural
nohup "$PRIMAL_DIR/songbird" server \
    --socket "$SONGBIRD_SOCKET" > /tmp/songbird_livespore.log 2>&1 &
SONGBIRD_PID=$!
echo "  PID: $SONGBIRD_PID"

# Wait for Songbird socket
sleep 5
if [ ! -e "$SONGBIRD_SOCKET" ]; then
  echo "❌ Songbird socket not created at $SONGBIRD_SOCKET"
  tail -10 /tmp/songbird_livespore.log
  exit 1
fi
echo "  ✅ Songbird operational"

echo ""
echo "🎊 TOWER ATOMIC OPERATIONAL!"
echo ""
echo "Sockets:"
ls -lh "$SOCKET_DIR"/*.sock
echo ""
echo "Health checks:"
echo '{"jsonrpc":"2.0","method":"health","id":1}' | nc -U "$BEARDOG_SOCKET" -w 2 || echo "(nc not available)"
echo '{"jsonrpc":"2.0","method":"health","id":1}' | nc -U "$SONGBIRD_SOCKET" -w 2 || echo "(nc not available)"

echo ""
echo "✅ Tower Atomic ready for use!"
echo "PIDs: BearDog=$BEARDOG_PID, Songbird=$SONGBIRD_PID"
