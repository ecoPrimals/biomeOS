#!/bin/bash
# LiveSpore USB - Start Tower Atomic
# Architecture: x86_64
# Components: BearDog + Songbird

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PRIMAL_DIR="$SCRIPT_DIR/../primals"

# Environment
export FAMILY_ID="${FAMILY_ID:-livespore}"
export NODE_ID="${NODE_ID:-$(hostname)}"
export RUST_LOG="${RUST_LOG:-info}"

echo "🚀 Starting Tower Atomic from LiveSpore USB"
echo "═══════════════════════════════════════════════════"
echo "Architecture: x86_64"
echo "Family ID: $FAMILY_ID"
echo "Node ID: $NODE_ID"
echo ""

# Create socket directory
mkdir -p /run/user/$(id -u)/biomeos

# Start BearDog
echo "🐻 Starting BearDog..."
nohup "$PRIMAL_DIR/beardog" server > /tmp/beardog_livespore.log 2>&1 &
BEARDOG_PID=$!
echo "  PID: $BEARDOG_PID"

# Wait for BearDog socket
sleep 4
if [ ! -e "/run/user/$(id -u)/biomeos/beardog.sock" ]; then
  echo "❌ BearDog socket not created"
  exit 1
fi
echo "  ✅ BearDog operational"

# Start Songbird
echo "🎵 Starting Songbird..."
export SONGBIRD_SECURITY_PROVIDER=beardog
export BEARDOG_SOCKET="/run/user/$(id -u)/biomeos/beardog.sock"
nohup "$PRIMAL_DIR/songbird" server > /tmp/songbird_livespore.log 2>&1 &
SONGBIRD_PID=$!
echo "  PID: $SONGBIRD_PID"

# Wait for Songbird socket
sleep 5
if [ ! -e "/run/user/$(id -u)/biomeos/songbird.sock" ]; then
  echo "❌ Songbird socket not created"
  exit 1
fi
echo "  ✅ Songbird operational"

echo ""
echo "🎊 TOWER ATOMIC OPERATIONAL!"
echo ""
echo "Sockets:"
ls -lh /run/user/$(id -u)/biomeos/*.sock
echo ""
echo "Health checks:"
echo '{"jsonrpc":"2.0","method":"health","id":1}' | nc -U /run/user/$(id -u)/biomeos/beardog.sock -w 2
echo '{"jsonrpc":"2.0","method":"health","id":1}' | nc -U /run/user/$(id -u)/biomeos/songbird.sock -w 2

echo ""
echo "✅ Tower Atomic ready for use!"
echo "PIDs: BearDog=$BEARDOG_PID, Songbird=$SONGBIRD_PID"
