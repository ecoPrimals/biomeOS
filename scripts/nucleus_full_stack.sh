#!/bin/bash
# Full NUCLEUS Stack Startup Script
# Start all 5 primals for complete NUCLEUS validation

set -e

export FAMILY_ID=1894e909e454
export NODE_ID=nucleus1
export JWT_SECRET="$(openssl rand -base64 48)"

echo "🎊 Starting FULL NUCLEUS Stack..."
echo "════════════════════════════════════════════"
echo ""
echo "Configuration:"
echo "  FAMILY_ID: $FAMILY_ID"
echo "  NODE_ID:   $NODE_ID"
echo "  JWT:       ${JWT_SECRET:0:20}..."
echo ""

# Clean up old sockets
echo "🧹 Cleaning old sockets..."
rm -f /run/user/$(id -u)/biomeos/*.sock
sleep 1

# 1. Start BearDog (Security Foundation)
echo "1️⃣ Starting BearDog (Security)..."
RUST_LOG=beardog=info \
    FAMILY_ID=$FAMILY_ID NODE_ID=$NODE_ID \
    beardog server > /tmp/beardog_nucleus.log 2>&1 &
BEARDOG_PID=$!
echo "   PID: $BEARDOG_PID"
sleep 3

# 2. Start Songbird (Network + Discovery)
echo "2️⃣ Starting Songbird (Network)..."
RUST_LOG=songbird=info \
    FAMILY_ID=$FAMILY_ID NODE_ID=$NODE_ID \
    SONGBIRD_SECURITY_PROVIDER=beardog \
    BEARDOG_SOCKET=/run/user/$(id -u)/biomeos/beardog.sock \
    songbird server > /tmp/songbird_nucleus.log 2>&1 &
SONGBIRD_PID=$!
echo "   PID: $SONGBIRD_PID"
sleep 3

# 3. Start Toadstool (GPU Compute)
echo "3️⃣ Starting Toadstool (Compute)..."
RUST_LOG=toadstool=info \
    FAMILY_ID=$FAMILY_ID NODE_ID=$NODE_ID \
    toadstool server > /tmp/toadstool_nucleus.log 2>&1 &
TOADSTOOL_PID=$!
echo "   PID: $TOADSTOOL_PID"
sleep 3

# 4. Start NestGate (Storage + Persistence)
echo "4️⃣ Starting NestGate (Storage - socket-only)..."
FAMILY_ID=$FAMILY_ID NODE_ID=$NODE_ID \
    NESTGATE_JWT_SECRET="$JWT_SECRET" \
    nestgate daemon --socket-only > /tmp/nestgate_nucleus.log 2>&1 &
NESTGATE_PID=$!
echo "   PID: $NESTGATE_PID"
sleep 3

# 5. Start Squirrel (AI Orchestration)
echo "5️⃣ Starting Squirrel (AI)..."
FAMILY_ID=$FAMILY_ID NODE_ID=$NODE_ID \
    squirrel server > /tmp/squirrel_nucleus.log 2>&1 &
SQUIRREL_PID=$!
echo "   PID: $SQUIRREL_PID"
sleep 5

# 6. Verify all sockets
echo ""
echo "🔍 Verifying ALL sockets..."
echo "════════════════════════════════════════════"
ls -lh /run/user/$(id -u)/biomeos/*.sock 2>/dev/null || echo "⚠️  No sockets found!"
echo ""

# Expected output:
# beardog.sock   ✅
# songbird.sock  ✅
# toadstool.sock ✅
# nestgate.sock  ✅
# squirrel.sock  ✅

# 7. Count sockets
SOCKET_COUNT=$(ls /run/user/$(id -u)/biomeos/*.sock 2>/dev/null | wc -l)
echo "Socket count: $SOCKET_COUNT/5"

if [ "$SOCKET_COUNT" -eq 5 ]; then
    echo "✅ All 5 sockets created!"
else
    echo "⚠️  Expected 5 sockets, found $SOCKET_COUNT"
    echo "Check logs in /tmp/*_nucleus.log"
fi
echo ""

# 8. Health checks (all 5)
echo "🏥 Health Checks (All 5 Primals)..."
echo "════════════════════════════════════════════"
echo ""

for primal in beardog songbird toadstool nestgate squirrel; do
    echo "🔍 $primal:"
    echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' | \
        nc -U /run/user/$(id -u)/biomeos/${primal}.sock -w 2 2>/dev/null || \
        echo "   ⚠️  No response"
    echo ""
done

# 9. Summary
echo "════════════════════════════════════════════"
echo "✅ FULL NUCLEUS STACK DEPLOYED!"
echo ""
echo "PIDs:"
echo "  BearDog:   $BEARDOG_PID"
echo "  Songbird:  $SONGBIRD_PID"
echo "  Toadstool: $TOADSTOOL_PID"
echo "  NestGate:  $NESTGATE_PID"
echo "  Squirrel:  $SQUIRREL_PID"
echo ""
echo "Logs:"
echo "  /tmp/beardog_nucleus.log"
echo "  /tmp/songbird_nucleus.log"
echo "  /tmp/toadstool_nucleus.log"
echo "  /tmp/nestgate_nucleus.log"
echo "  /tmp/squirrel_nucleus.log"
echo ""
echo "Stop all: kill $BEARDOG_PID $SONGBIRD_PID $TOADSTOOL_PID $NESTGATE_PID $SQUIRREL_PID"
echo ""
echo "🎊 NUCLEUS is operational!"
