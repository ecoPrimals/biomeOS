#!/bin/bash
# LiveSpore USB - Start Nest Atomic
# Architecture: x86_64
# Components: Tower + NestGate + Squirrel

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PRIMAL_DIR="$SCRIPT_DIR/../primals"

# Start Tower first
echo "🚀 Starting Tower Atomic (prerequisite)..."
source "$SCRIPT_DIR/start_tower.sh"

echo ""
echo "🦅 Adding NestGate + Squirrel for Nest Atomic..."
echo "═══════════════════════════════════════════════════"

# Environment
export NESTGATE_JWT_SECRET="${NESTGATE_JWT_SECRET:-$(openssl rand -base64 48)}"

# Start NestGate
echo "Starting NestGate (socket-only mode)..."
nohup "$PRIMAL_DIR/nestgate" service start --daemon > /tmp/nestgate_livespore.log 2>&1
NESTGATE_PID=$!
echo "  PID: $NESTGATE_PID"

# Wait for socket
sleep 5
if [ ! -e "/run/user/$(id -u)/biomeos/nestgate.sock" ]; then
  echo "⚠️  NestGate socket not found, checking log..."
  tail -20 /tmp/nestgate_livespore.log
fi

# Start Squirrel
echo "Starting Squirrel..."
nohup "$PRIMAL_DIR/squirrel" > /tmp/squirrel_livespore.log 2>&1 &
SQUIRREL_PID=$!
echo "  PID: $SQUIRREL_PID"

# Wait for socket
sleep 5
if [ ! -e "/run/user/$(id -u)/biomeos/squirrel.sock" ]; then
  echo "⚠️  Squirrel socket not found, checking log..."
  tail -20 /tmp/squirrel_livespore.log
fi

echo ""
echo "🎊 NEST ATOMIC OPERATIONAL!"
echo "(Tower + NestGate + Squirrel = Storage + AI)"
echo ""
echo "All sockets:"
ls -lh /run/user/$(id -u)/biomeos/*.sock
echo ""
echo "✅ Nest Atomic ready!"
echo "PIDs: BearDog=$BEARDOG_PID, Songbird=$SONGBIRD_PID, NestGate=$NESTGATE_PID, Squirrel=$SQUIRREL_PID"
