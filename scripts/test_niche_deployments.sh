#!/usr/bin/env bash
# Test live niche deployments with Neural API + NUCLEUS

set -euo pipefail

PROJECT_ROOT="/home/eastgate/Development/ecoPrimals/phase2/biomeOS"
cd "$PROJECT_ROOT"

UID_VAR=$(id -u)
SOCKET_DIR="/run/user/${UID_VAR}"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧬 biomeOS Niche Deployment Test System"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Step 1: Check available primals
echo "📊 Step 1: Checking available primals..."
echo ""

echo "Running Primals:"
pgrep -a "beardog|toadstool|nestgate|squirrel|songbird" | while read -r line; do
    echo "  ✅ $line"
done

echo ""
echo "Unix Sockets:"
ls "$SOCKET_DIR"/*.sock 2>/dev/null | while read -r sock; do
    echo "  🔌 $(basename "$sock")"
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Step 2: Start NestGate if not running
echo ""
echo "📦 Step 2: Ensuring NestGate is running..."

if pgrep -f "nestgate" > /dev/null; then
    echo "  ✅ NestGate already running"
else
    echo "  🚀 Starting NestGate..."
    SONGBIRD_FAMILY_ID=nat0 plasmidBin/nestgate > /tmp/nestgate.log 2>&1 &
    NESTGATE_PID=$!
    echo "  ✅ NestGate started (PID: $NESTGATE_PID)"
    sleep 2
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Step 3: Test primal interactions
echo ""
echo "🔗 Step 3: Testing primal interactions..."
echo ""

echo "Available Graphs:"
ls -1 graphs/*.toml | while read -r graph; do
    name=$(basename "$graph" .toml)
    echo "  📊 $name"
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Step 4: Manual interaction tests
echo ""
echo "🧪 Step 4: Manual Primal Interaction Tests"
echo ""

echo "Test 1: Check Songbird discovery..."
SONGBIRD_SOCK=$(ls "$SOCKET_DIR"/songbird-*.sock 2>/dev/null | head -1)
if [ -n "$SONGBIRD_SOCK" ]; then
    echo "  ✅ Songbird socket: $(basename "$SONGBIRD_SOCK")"
else
    echo "  ⚠️  Songbird not found"
fi

echo ""
echo "Test 2: Check BearDog security..."
BEARDOG_SOCK=$(ls "$SOCKET_DIR"/beardog-*.sock 2>/dev/null | head -1)
if [ -n "$BEARDOG_SOCK" ]; then
    echo "  ✅ BearDog socket: $(basename "$BEARDOG_SOCK")"
else
    echo "  ⚠️  BearDog not found"
fi

echo ""
echo "Test 3: Check ToadStool compute..."
if pgrep -f "toadstool" > /dev/null; then
    echo "  ✅ ToadStool running"
    # Note: ToadStool uses TCP endpoint (hardcoded issue)
    echo "  ⚠️  ToadStool uses TCP 127.0.0.1:9944 (not Unix socket yet)"
else
    echo "  ⚠️  ToadStool not running"
fi

echo ""
echo "Test 4: Check NestGate storage..."
NESTGATE_SOCK=$(ls "$SOCKET_DIR"/nestgate-*.sock 2>/dev/null | head -1)
if [ -n "$NESTGATE_SOCK" ]; then
    echo "  ✅ NestGate socket: $(basename "$NESTGATE_SOCK")"
else
    echo "  ⚠️  NestGate not found"
fi

echo ""
echo "Test 5: Check Squirrel AI..."
SQUIRREL_SOCK=$(ls "$SOCKET_DIR"/squirrel-*.sock 2>/dev/null | head -1)
if [ -n "$SQUIRREL_SOCK" ]; then
    echo "  ✅ Squirrel socket: $(basename "$SQUIRREL_SOCK")"
else
    echo "  ⚠️  Squirrel not found"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Step 5: Summary
echo ""
echo "📊 Step 5: Summary"
echo ""

PRIMAL_COUNT=$(pgrep -c "beardog|toadstool|nestgate|squirrel|songbird" || echo "0")
SOCKET_COUNT=$(ls "$SOCKET_DIR"/*.sock 2>/dev/null | wc -l || echo "0")

echo "Running Primals: $PRIMAL_COUNT"
echo "Available Sockets: $SOCKET_COUNT"
echo ""

if [ "$PRIMAL_COUNT" -ge 4 ]; then
    echo "✅ Ecosystem is healthy! Ready for niche deployment."
    echo ""
    echo "Next Steps:"
    echo "  1. graphs/tower_deploy.toml - Deploy Tower (security + discovery)"
    echo "  2. graphs/node_deploy.toml - Deploy Node (compute)"
    echo "  3. graphs/nest_deploy.toml - Deploy Nest (storage + persistence)"
    echo "  4. graphs/nucleus_deploy.toml - Deploy complete NUCLEUS"
    echo ""
    echo "Currently: Graphs exist but need Neural API executor integration"
else
    echo "⚠️  Some primals missing. Start them first:"
    [ -z "$SONGBIRD_SOCK" ] && echo "  - Songbird (discovery)"
    [ -z "$BEARDOG_SOCK" ] && echo "  - BearDog (security)"
    ! pgrep -f "toadstool" > /dev/null && echo "  - ToadStool (compute)"
    [ -z "$NESTGATE_SOCK" ] && echo "  - NestGate (storage)"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

