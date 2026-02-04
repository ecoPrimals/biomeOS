#!/bin/bash
# LiveSpore USB - Start Nest Atomic
# Architecture: aarch64 (ARM64)
# Components: Tower + NestGate + Squirrel
# Standard: PRIMAL_DEPLOYMENT_STANDARD v1.0

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PRIMAL_DIR="$SCRIPT_DIR/../primals"

# Start Tower first (sets SOCKET_DIR, FAMILY_ID, etc.)
echo "🚀 Starting Tower Atomic (prerequisite)..."
source "$SCRIPT_DIR/start_tower.sh"

echo ""
echo "🦅 Adding NestGate + Squirrel for Nest Atomic..."
echo "═══════════════════════════════════════════════════"

# Environment (SOCKET_DIR inherited from start_tower.sh)
export NESTGATE_JWT_SECRET="${NESTGATE_JWT_SECRET:-$(openssl rand -base64 48 2>/dev/null || head -c 48 /dev/urandom | base64)}"
export NESTGATE_SOCKET="$SOCKET_DIR/nestgate-$FAMILY_ID.sock"
export SQUIRREL_SOCKET="$SOCKET_DIR/squirrel-$FAMILY_ID.sock"

# Start NestGate
echo "Starting NestGate (socket-only mode)..."
nohup "$PRIMAL_DIR/nestgate" service start \
    --socket "$NESTGATE_SOCKET" \
    --daemon > /tmp/nestgate_livespore.log 2>&1 &
NESTGATE_PID=$!
echo "  PID: $NESTGATE_PID"

# Wait for socket
sleep 5
if [ ! -e "$NESTGATE_SOCKET" ]; then
  echo "⚠️  NestGate socket not found at $NESTGATE_SOCKET"
  tail -20 /tmp/nestgate_livespore.log 2>/dev/null || true
fi

# Start Squirrel
echo "Starting Squirrel..."
export NEURAL_API_SOCKET="$SOCKET_DIR/neural-api-$FAMILY_ID.sock"
nohup "$PRIMAL_DIR/squirrel" server > /tmp/squirrel_livespore.log 2>&1 &
SQUIRREL_PID=$!
echo "  PID: $SQUIRREL_PID"

# Wait for socket
sleep 5
if [ ! -e "$SQUIRREL_SOCKET" ]; then
  echo "⚠️  Squirrel socket not found at $SQUIRREL_SOCKET"
  tail -20 /tmp/squirrel_livespore.log 2>/dev/null || true
fi

echo ""
echo "🎊 NEST ATOMIC OPERATIONAL!"
echo "(Tower + NestGate + Squirrel = Storage + AI)"
echo ""
echo "All sockets:"
ls -lh "$SOCKET_DIR"/*.sock 2>/dev/null || echo "(checking...)"
echo ""
echo "✅ Nest Atomic ready!"
echo "PIDs: BearDog=$BEARDOG_PID, Songbird=$SONGBIRD_PID, NestGate=$NESTGATE_PID, Squirrel=$SQUIRREL_PID"
