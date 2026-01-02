#!/usr/bin/env bash
# Lifecycle Negotiation Demo

set -e

echo "═══════════════════════════════════════════════════════════"
echo "   🔄 Lifecycle Negotiation Demo - Cell Senescence Model"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Check for Python
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 required"
    exit 1
fi

# Make mocks executable
chmod +x lifecycle-mocks/*-lifecycle-mock

echo "🚀 Step 1: Starting all primals with lifecycle support..."
echo ""

# Start lifecycle-aware mocks
PORT=9010 ./lifecycle-mocks/squirrel-lifecycle-mock > /tmp/squirrel-lifecycle.log 2>&1 &
SQUIRREL_PID=$!
echo "  ✅ Squirrel started (PID: $SQUIRREL_PID)"

PORT=9020 ./lifecycle-mocks/nestgate-lifecycle-mock service > /tmp/nestgate-lifecycle.log 2>&1 &
NESTGATE_PID=$!
echo "  ✅ NestGate started (PID: $NESTGATE_PID)"

PORT=9030 ./lifecycle-mocks/toadstool-lifecycle-mock > /tmp/toadstool-lifecycle.log 2>&1 &
TOADSTOOL_PID=$!
echo "  ✅ ToadStool started (PID: $TOADSTOOL_PID)"

echo ""
echo "⏳ Waiting for primals to initialize..."
sleep 3
echo ""

# Verify health
echo "🏥 Step 2: Verifying all primals healthy..."
echo ""

ALL_HEALTHY=true
for primal in squirrel:9010 nestgate:9020 toadstool:9030; do
    NAME=${primal%:*}
    PORT=${primal#*:}
    
    if curl -s -f http://localhost:$PORT/health > /dev/null 2>&1; then
        echo "  ✅ $NAME healthy (port $PORT)"
    else
        echo "  ❌ $NAME unhealthy (port $PORT)"
        ALL_HEALTHY=false
    fi
done

echo ""

if [ "$ALL_HEALTHY" != "true" ]; then
    echo "❌ Some primals unhealthy. Check logs."
    kill $SQUIRREL_PID $NESTGATE_PID $TOADSTOOL_PID 2>/dev/null
    exit 1
fi

echo "═══════════════════════════════════════════════════════════"
echo "   🔄 Scenario: Request Graceful Stop from All Primals"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Request lifecycle transition from each primal
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📤 Requesting: Squirrel to gracefully stop"
echo "   Reason: Ecosystem maintenance"
echo "   Urgency: Normal"
echo ""

RESPONSE=$(curl -s http://localhost:9010/lifecycle/request)
STATUS=$(echo $RESPONSE | python3 -c "import sys, json; print(json.load(sys.stdin)['status'])")

echo "📥 Squirrel Response: $STATUS"
if [ "$STATUS" = "accepted" ]; then
    echo "   ✅ Squirrel accepted - stopping gracefully"
    kill -TERM $SQUIRREL_PID 2>/dev/null
    echo "   🛑 Squirrel stopped"
else
    echo "   ⚠️  Unexpected response: $STATUS"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📤 Requesting: NestGate to gracefully stop"
echo "   Reason: Ecosystem maintenance"
echo "   Urgency: Normal"
echo ""

RESPONSE=$(curl -s http://localhost:9020/lifecycle/request)
STATUS=$(echo $RESPONSE | python3 -c "import sys, json; print(json.load(sys.stdin)['status'])")

echo "📥 NestGate Response: $STATUS"
if [ "$STATUS" = "deferred" ]; then
    REASON=$(echo $RESPONSE | python3 -c "import sys, json; print(json.load(sys.stdin)['reason'])")
    DURATION=$(echo $RESPONSE | python3 -c "import sys, json; print(json.load(sys.stdin)['duration_secs'])")
    echo "   Reason: \"$REASON\""
    echo "   ⏳ BiomeOS waiting $DURATION seconds as requested..."
    
    # Countdown
    for i in $(seq $DURATION -1 1); do
        echo -ne "   ⏱️  $i seconds remaining...\r"
        sleep 1
    done
    echo "   ⏱️  Time elapsed                     "
    echo ""
    
    echo "📤 Retrying: NestGate graceful stop"
    RESPONSE=$(curl -s http://localhost:9020/lifecycle/request)
    STATUS=$(echo $RESPONSE | python3 -c "import sys, json; print(json.load(sys.stdin)['status'])")
    echo "📥 NestGate Response: $STATUS"
    
    if [ "$STATUS" = "accepted" ]; then
        echo "   ✅ Operations complete, stopping now"
        kill -TERM $NESTGATE_PID 2>/dev/null
        echo "   🛑 NestGate stopped"
    fi
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📤 Requesting: ToadStool to gracefully stop"
echo "   Reason: Ecosystem maintenance"
echo "   Urgency: Normal"
echo ""

RESPONSE=$(curl -s http://localhost:9030/lifecycle/request)
STATUS=$(echo $RESPONSE | python3 -c "import sys, json; print(json.load(sys.stdin)['status'])")

echo "📥 ToadStool Response: $STATUS"
if [ "$STATUS" = "refused" ]; then
    REASON=$(echo $RESPONSE | python3 -c "import sys, json; print(json.load(sys.stdin)['reason'])")
    echo "   Reason: \"$REASON\""
    echo "   ❌ BiomeOS respects decision (sovereignty preserved)"
    echo "   ℹ️  ToadStool continues running"
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "   📊 Summary"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Lifecycle Request Results:"
echo "  ✅ Accepted immediately: Squirrel"
echo "  ⏳ Deferred then accepted: NestGate"
echo "  ❌ Refused (sovereignty): ToadStool"
echo ""
echo "Final Status:"
echo "  🛑 Stopped: 2 primals (Squirrel, NestGate)"
echo "  🔄 Running: 1 primal (ToadStool - critical work)"
echo ""
echo "🌱 Key Principles Demonstrated:"
echo "  ✅ Request, not command"
echo "  ✅ Primals have full autonomy"
echo "  ✅ Deferred requests honored (BiomeOS waited)"
echo "  ✅ Refusals respected (ToadStool still running)"
echo "  ✅ Ecosystem coordination without coercion"
echo "  ✅ Cell senescence > overwatch"
echo ""
echo "🎉 Lifecycle Negotiation Complete!"
echo ""

# Cleanup remaining primals
echo "🧹 Cleaning up..."
kill $TOADSTOOL_PID 2>/dev/null && echo "  🛑 ToadStool stopped" || true

echo ""
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "This demo showed BiomeOS requesting lifecycle transitions"
echo "and respecting primal sovereignty. Each primal made its own"
echo "decision, and BiomeOS adapted accordingly."
echo ""
echo "Compare this to traditional orchestrators that force lifecycle"
echo "changes. Cell senescence > overwatch! 🌱"
echo ""

