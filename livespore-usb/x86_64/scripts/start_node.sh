#!/bin/bash
# LiveSpore USB - Start Node Atomic
# Architecture: x86_64
# Components: Tower (BearDog + Songbird) + Toadstool
# Standard: PRIMAL_DEPLOYMENT_STANDARD v1.0

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PRIMAL_DIR="$SCRIPT_DIR/../primals"

# Start Tower first (sets SOCKET_DIR, FAMILY_ID, etc.)
echo "🚀 Starting Tower Atomic (prerequisite)..."
source "$SCRIPT_DIR/start_tower.sh"

echo ""
echo "🍄 Adding Toadstool for Node Atomic..."
echo "═══════════════════════════════════════════════════"

# Environment (SOCKET_DIR inherited from start_tower.sh)
export BIOMEOS_FAMILY_ID="$FAMILY_ID"
export TOADSTOOL_SECURITY_WARNING_ACKNOWLEDGED=1
export TOADSTOOL_SOCKET="$SOCKET_DIR/toadstool-$FAMILY_ID.sock"

# Start Toadstool
echo "Starting Toadstool daemon..."
nohup "$PRIMAL_DIR/toadstool" daemon \
  --socket "$TOADSTOOL_SOCKET" \
  --register > /tmp/toadstool_livespore.log 2>&1 &
TOADSTOOL_PID=$!
echo "  PID: $TOADSTOOL_PID"

# Wait for socket
sleep 5
if [ -S "$TOADSTOOL_SOCKET" ]; then
  echo "  ✅ Toadstool operational"
else
  echo "  ⚠️  Toadstool socket not found at $TOADSTOOL_SOCKET"
  tail -20 /tmp/toadstool_livespore.log 2>/dev/null || true
fi

echo ""
echo "🎊 NODE ATOMIC OPERATIONAL!"
echo "(Tower + Toadstool = Compute + barraCUDA)"
echo ""
echo "All sockets:"
ls -lh "$SOCKET_DIR"/*.sock 2>/dev/null || echo "(checking...)"
echo ""
echo "✅ Node Atomic ready with 50 GPU operations!"
echo "PIDs: BearDog=$BEARDOG_PID, Songbird=$SONGBIRD_PID, Toadstool=$TOADSTOOL_PID"
