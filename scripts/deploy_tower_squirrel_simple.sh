#!/usr/bin/env bash
#
# Tower Atomic + Squirrel Simple Deployment
# Based on verified working commands from testing
#
set -euo pipefail

FAMILY_ID="${1:-nat0}"
RUNTIME_DIR="/tmp"
ANTHROPIC_API_KEY="${ANTHROPIC_API_KEY:-}"

if [ -z "$ANTHROPIC_API_KEY" ]; then
  echo "❌ ANTHROPIC_API_KEY not set"
  exit 1
fi

echo "🧬 Deploying Tower Atomic + Squirrel (family: $FAMILY_ID)"
echo ""

# Clean up
echo "Cleaning previous deployment..."
pkill -f "beardog.*$FAMILY_ID" 2>/dev/null || true
pkill -f "songbird.*$FAMILY_ID" 2>/dev/null || true
pkill -f "squirrel.*$FAMILY_ID" 2>/dev/null || true
sleep 1
rm -f "$RUNTIME_DIR/*-$FAMILY_ID.sock"

# Socket paths
BEARDOG_SOCKET="$RUNTIME_DIR/beardog-$FAMILY_ID.sock"
SONGBIRD_SOCKET="$RUNTIME_DIR/songbird-$FAMILY_ID.sock"
SQUIRREL_SOCKET="$RUNTIME_DIR/squirrel-$FAMILY_ID.sock"

# Phase 1: BearDog
echo "Phase 1/3: Starting BearDog..."
./plasmidBin/primals/beardog/beardog-x86_64-musl server \
  --socket "$BEARDOG_SOCKET" \
  --family-id "$FAMILY_ID" \
  > "$RUNTIME_DIR/beardog-$FAMILY_ID.log" 2>&1 &

echo -n "  Waiting for BearDog socket"
for i in {1..30}; do
  [ -S "$BEARDOG_SOCKET" ] && break
  echo -n "."
  sleep 0.2
done
echo ""

if [ ! -S "$BEARDOG_SOCKET" ]; then
  echo "❌ BearDog failed to start"
  tail -20 "$RUNTIME_DIR/beardog-$FAMILY_ID.log"
  exit 1
fi
echo "  ✅ BearDog ready: $BEARDOG_SOCKET"

# Phase 2: Songbird
echo ""
echo "Phase 2/3: Starting Songbird (bonded to BearDog)..."
env SONGBIRD_SOCKET="$SONGBIRD_SOCKET" \
    SONGBIRD_SECURITY_PROVIDER="$BEARDOG_SOCKET" \
    SONGBIRD_ORCHESTRATOR_FAMILY_ID="$FAMILY_ID" \
  ./plasmidBin/primals/songbird/songbird-x86_64-musl server \
  > "$RUNTIME_DIR/songbird-$FAMILY_ID.log" 2>&1 &

echo -n "  Waiting for Songbird socket"
for i in {1..30}; do
  [ -S "$SONGBIRD_SOCKET" ] && break
  echo -n "."
  sleep 0.2
done
echo ""

if [ ! -S "$SONGBIRD_SOCKET" ]; then
  echo "❌ Songbird failed to start"
  tail -20 "$RUNTIME_DIR/songbird-$FAMILY_ID.log"
  exit 1
fi
echo "  ✅ Songbird ready: $SONGBIRD_SOCKET"

# Phase 3: Squirrel
echo ""
echo "Phase 3/3: Starting Squirrel..."
env SQUIRREL_SOCKET="$SQUIRREL_SOCKET" \
    SONGBIRD_ENDPOINT="$SONGBIRD_SOCKET" \
    ANTHROPIC_API_KEY="$ANTHROPIC_API_KEY" \
  ./plasmidBin/primals/squirrel/squirrel-x86_64-musl server \
  > "$RUNTIME_DIR/squirrel-$FAMILY_ID.log" 2>&1 &

echo -n "  Waiting for Squirrel socket"
for i in {1..30}; do
  [ -S "$SQUIRREL_SOCKET" ] && break
  echo -n "."
  sleep 0.2
done
echo ""

if [ ! -S "$SQUIRREL_SOCKET" ]; then
  echo "⚠️  Squirrel socket not found (may still be starting)"
  tail -20 "$RUNTIME_DIR/squirrel-$FAMILY_ID.log"
fi
echo "  ✅ Squirrel ready: $SQUIRREL_SOCKET"

# Summary
echo ""
echo "=========================================="
echo "✅ Tower Atomic + Squirrel Deployed!"
echo "=========================================="
echo ""
echo "Sockets:"
echo "  BearDog:  $BEARDOG_SOCKET"
echo "  Songbird: $SONGBIRD_SOCKET"
echo "  Squirrel: $SQUIRREL_SOCKET"
echo ""
echo "Test AI call:"
echo "  echo '{\"jsonrpc\":\"2.0\",\"method\":\"ai.chat\",\"params\":{\"messages\":[{\"role\":\"user\",\"content\":\"Hello!\"}]},\"id\":1}' | nc -U $SQUIRREL_SOCKET"
echo ""
echo "Logs:"
echo "  tail -f $RUNTIME_DIR/beardog-$FAMILY_ID.log"
echo "  tail -f $RUNTIME_DIR/songbird-$FAMILY_ID.log"
echo "  tail -f $RUNTIME_DIR/squirrel-$FAMILY_ID.log"
echo ""


