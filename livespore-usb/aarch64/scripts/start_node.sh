#!/bin/bash
# LiveSpore USB - Start Node Atomic
# Architecture: x86_64
# Components: Tower (BearDog + Songbird) + Toadstool

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PRIMAL_DIR="$SCRIPT_DIR/../primals"

# Start Tower first
echo "🚀 Starting Tower Atomic (prerequisite)..."
source "$SCRIPT_DIR/start_tower.sh"

echo ""
echo "🍄 Adding Toadstool for Node Atomic..."
echo "═══════════════════════════════════════════════════"

# Environment
export BIOMEOS_FAMILY_ID="${FAMILY_ID:-livespore}"
export TOADSTOOL_SECURITY_WARNING_ACKNOWLEDGED=1

# Start Toadstool
echo "Starting Toadstool daemon..."
nohup "$PRIMAL_DIR/toadstool" daemon \
  --socket "/run/user/$(id -u)/biomeos/toadstool.sock" \
  --register > /tmp/toadstool_livespore.log 2>&1 &
TOADSTOOL_PID=$!
echo "  PID: $TOADSTOOL_PID"

# Wait for socket
sleep 5
if [ ! -e "/run/user/$(id -u)/biomeos/toadstool.sock" ]; then
  echo "❌ Toadstool socket not created"
  tail -20 /tmp/toadstool_livespore.log
  exit 1
fi
echo "  ✅ Toadstool operational"

echo ""
echo "🎊 NODE ATOMIC OPERATIONAL!"
echo "(Tower + Toadstool = Compute + barraCUDA)"
echo ""
echo "All sockets:"
ls -lh /run/user/$(id -u)/biomeos/*.sock
echo ""
echo "✅ Node Atomic ready with 50 GPU operations!"
echo "PIDs: BearDog=$BEARDOG_PID, Songbird=$SONGBIRD_PID, Toadstool=$TOADSTOOL_PID"
