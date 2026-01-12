#!/bin/bash
# Start All Primals for Full E2E Testing
#
# This script starts all available primals with proper configuration
# for live biomeOS + petalTongue integration testing

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$(dirname "$SCRIPT_DIR")"
PRIMAL_BIN="$BIOMEOS_ROOT/plasmidBin"

echo "═══════════════════════════════════════════════════════════════"
echo "    🌸 STARTING ALL PRIMALS FOR E2E TESTING 🌸"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Kill any existing primal processes (clean start)
echo "🧹 Cleaning up old processes..."
pkill -f 'beardog|toadstool|nestgate|squirrel' 2>/dev/null || true
sleep 1

# Start BearDog (Security Primal)
if [ -f "$PRIMAL_BIN/beardog" ]; then
    echo "🐻 Starting BearDog (security)..."
    "$PRIMAL_BIN/beardog" > /tmp/beardog.log 2>&1 &
    BEARDOG_PID=$!
    echo "   PID: $BEARDOG_PID"
    sleep 1
else
    echo "⚠️  BearDog binary not found"
fi

# Start ToadStool (Compute Primal with CI)
if [ -f "$PRIMAL_BIN/toadstool" ]; then
    echo "🍄 Starting ToadStool (compute + CI)..."
    "$PRIMAL_BIN/toadstool" > /tmp/toadstool.log 2>&1 &
    TOADSTOOL_PID=$!
    echo "   PID: $TOADSTOOL_PID"
    sleep 1
else
    echo "⚠️  ToadStool binary not found"
fi

# Start NestGate (Storage Primal)
if [ -f "$PRIMAL_BIN/nestgate" ]; then
    echo "🏠 Starting NestGate (storage)..."
    "$PRIMAL_BIN/nestgate" > /tmp/nestgate.log 2>&1 &
    NESTGATE_PID=$!
    echo "   PID: $NESTGATE_PID"
    sleep 1
else
    echo "⚠️  NestGate binary not found"
fi

# Start Squirrel (AI Primal)
if [ -f "$PRIMAL_BIN/squirrel" ]; then
    echo "🐿️  Starting Squirrel (AI)..."
    "$PRIMAL_BIN/squirrel" > /tmp/squirrel.log 2>&1 &
    SQUIRREL_PID=$!
    echo "   PID: $SQUIRREL_PID"
    sleep 1
else
    echo "⚠️  Squirrel binary not found"
fi

echo ""
echo "⏳ Waiting for primals to initialize..."
sleep 3

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "    ✅ PRIMALS STARTED"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Check which primals are running
echo "📊 Running Primals:"
ps aux | grep -E '(beardog|toadstool|nestgate|squirrel)' | grep -v grep | grep -v '.log' || echo "   (none detected via ps)"

echo ""
echo "🔍 Unix Sockets:"
ls -la /run/user/$(id -u)/ 2>/dev/null | grep -E '(beardog|toadstool|nestgate|squirrel)' || echo "   (checking...)"

echo ""
echo "📝 Log files:"
echo "   BearDog: /tmp/beardog.log"
echo "   ToadStool: /tmp/toadstool.log"
echo "   NestGate: /tmp/nestgate.log"
echo "   Squirrel: /tmp/squirrel.log"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "    🎉 READY FOR E2E TESTING!"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Next steps:"
echo "  1. Device management server should auto-detect new primals"
echo "  2. petalTongue GUI will show all primals"
echo "  3. Try deploying a niche from the GUI!"
echo ""

