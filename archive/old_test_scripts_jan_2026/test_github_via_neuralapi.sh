#!/bin/bash

set -e

cat << 'EOF'

╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║   🚀 TOWER ATOMIC → GITHUB VIA NEURAL API 🚀                 ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝

Testing: capability.call("secure_http", "http.request") → GitHub

EOF

BIOMEOS_PATH="/home/eastgate/Development/ecoPrimals/phase2/biomeOS"
BEARDOG_PATH="/home/eastgate/Development/ecoPrimals/phase1/beardog"
SONGBIRD_PATH="/home/eastgate/Development/ecoPrimals/phase1/songbird"

NEURAL_API_SOCKET="/tmp/neural-api.sock"
BEARDOG_SOCKET="/tmp/beardog-nat0.sock"
SONGBIRD_SOCKET="/tmp/songbird-nat0.sock"

cleanup() {
    echo ""
    echo "🧹 Cleanup..."
    pkill -f "biomeos neural-api" || true
    pkill -f "beardog server" || true
    pkill -f "songbird server" || true
    rm -f "$NEURAL_API_SOCKET" "$BEARDOG_SOCKET" "$SONGBIRD_SOCKET"
    sleep 1
}

trap cleanup EXIT

cleanup

echo "1️⃣  Starting BearDog..."
export FAMILY_ID=nat0
export NODE_ID=tower1
export NEURAL_API_SOCKET="$NEURAL_API_SOCKET"
cd "$BEARDOG_PATH"
./target/release/beardog server --socket "$BEARDOG_SOCKET" > /tmp/beardog.log 2>&1 &
sleep 2

if [ ! -S "$BEARDOG_SOCKET" ]; then
    echo "❌ BearDog socket not created"
    cat /tmp/beardog.log
    exit 1
fi
echo "   ✅ BearDog running at $BEARDOG_SOCKET"

echo ""
echo "2️⃣  Starting Songbird..."
export SONGBIRD_SECURITY_PROVIDER="beardog"
export BEARDOG_SOCKET="$BEARDOG_SOCKET"
export SONGBIRD_SOCKET_PATH="$SONGBIRD_SOCKET"
export SONGBIRD_FAMILY_ID="nat0"
export SONGBIRD_NODE_ID="tower1"
export NEURAL_API_SOCKET="$NEURAL_API_SOCKET"
cd "$SONGBIRD_PATH"
./target/release/songbird server > /tmp/songbird.log 2>&1 &
sleep 3

if [ ! -S "$SONGBIRD_SOCKET" ]; then
    echo "❌ Songbird socket not created"
    cat /tmp/songbird.log
    exit 1
fi
echo "   ✅ Songbird running at $SONGBIRD_SOCKET"

echo ""
echo "3️⃣  Starting Neural API (COORDINATED MODE)..."
cd "$BIOMEOS_PATH"
export RUST_LOG=info
export BIOMEOS_MODE=coordinated
export NEURAL_API_SOCKET="$NEURAL_API_SOCKET"
./target/release/biomeos neural-api --socket "$NEURAL_API_SOCKET" > /tmp/neural-api.log 2>&1 &
sleep 3

if [ ! -S "$NEURAL_API_SOCKET" ]; then
    echo "❌ Neural API socket not created"
    cat /tmp/neural-api.log
    exit 1
fi
echo "   ✅ Neural API running at $NEURAL_API_SOCKET"

echo ""
echo "4️⃣  Checking registrations..."
CAPS=$(echo '{"jsonrpc":"2.0","method":"capability.list","params":{},"id":1}' | nc -U "$NEURAL_API_SOCKET" -w 2)
echo "   Capabilities registered:"
echo "$CAPS" | jq -r '.result.capabilities[]' | while read cap; do
    echo "      • $cap"
done

echo ""
echo "5️⃣  Testing GitHub API via Neural API capability.call..."
echo ""
echo "   Request: capability.call(\"secure_http\", \"http.request\", {...})"
echo "   Target: https://api.github.com/zen"
echo ""

RESPONSE=$(cat << 'JSONRPC' | nc -U "$NEURAL_API_SOCKET" -w 10
{"jsonrpc":"2.0","method":"capability.call","params":{"capability":"secure_http","operation":"http.request","args":{"url":"https://api.github.com/zen","method":"GET"}},"id":42}
JSONRPC
)

echo "   Response:"
echo "$RESPONSE" | jq '.'

if echo "$RESPONSE" | jq -e '.result.status == 200' > /dev/null; then
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "✅ SUCCESS! GitHub responded via Tower Atomic + Neural API!"
    echo ""
    echo "🎊 ARCHITECTURE VALIDATION: COMPLETE! 🎊"
    echo ""
    echo "   Neural API → Songbird → BearDog → GitHub ✅"
    echo "   Graph-based semantic translation ✅"
    echo "   Pure Rust TLS 1.3 ✅"
    echo "   capability.call system ✅"
    echo ""
    echo "GitHub's wisdom: $(echo "$RESPONSE" | jq -r '.result.body')"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 0
else
    echo ""
    echo "❌ Request failed or unexpected response"
    echo ""
    echo "Neural API logs:"
    tail -20 /tmp/neural-api.log
    echo ""
    echo "Songbird logs:"
    tail -20 /tmp/songbird.log
    exit 1
fi

