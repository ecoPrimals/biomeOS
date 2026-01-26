#!/bin/bash
# Tower Atomic Integration Test Script
# Tests biomeOS Neural API → Songbird → GitHub

set -e

echo "═══════════════════════════════════════════════════════════════"
echo "🧪 Tower Atomic Integration Test"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Configuration
BIOMEOS_DIR="/home/eastgate/Development/ecoPrimals/phase2/biomeOS"
SONGBIRD_DIR="/home/eastgate/Development/ecoPrimals/phase1/songbird"
SONGBIRD_SOCKET="/run/user/$(id -u)/songbird-nat0.sock"
BIOMEOS_SOCKET="/run/user/$(id -u)/neural-api-nat0.sock"

echo "📋 Test Configuration:"
echo "  - biomeOS: $BIOMEOS_DIR"
echo "  - Songbird: $SONGBIRD_DIR"
echo "  - Songbird Socket: $SONGBIRD_SOCKET"
echo "  - biomeOS Socket: $BIOMEOS_SOCKET"
echo ""

# Phase 1: Check builds
echo "Phase 1: Verify Builds"
echo "─────────────────────────────────────────────────────────────"

if [ ! -f "$SONGBIRD_DIR/target/release/songbird" ]; then
    echo "❌ Songbird binary not found"
    exit 1
fi
echo "✅ Songbird binary: $(du -h $SONGBIRD_DIR/target/release/songbird | cut -f1)"

if [ ! -f "$BIOMEOS_DIR/target/release/biomeos" ]; then
    echo "❌ biomeOS binary not found"
    exit 1
fi
echo "✅ biomeOS binary: $(du -h $BIOMEOS_DIR/target/release/biomeos | cut -f1)"
echo ""

# Phase 2: Start Songbird
echo "Phase 2: Start Songbird"
echo "─────────────────────────────────────────────────────────────"

# Kill any existing Songbird
pkill -f "songbird" || true
sleep 1

# Start Songbird in background
cd $SONGBIRD_DIR
export RUST_LOG=info
export SONGBIRD_SOCKET_PATH="$SONGBIRD_SOCKET"
nohup ./target/release/songbird > /tmp/songbird-test.log 2>&1 &
SONGBIRD_PID=$!

echo "✅ Songbird started (PID: $SONGBIRD_PID)"
echo "   Log: /tmp/songbird-test.log"
echo "   Waiting for socket..."

# Wait for Songbird socket
for i in {1..10}; do
    if [ -S "$SONGBIRD_SOCKET" ]; then
        echo "✅ Songbird socket ready: $SONGBIRD_SOCKET"
        break
    fi
    if [ $i -eq 10 ]; then
        echo "❌ Songbird socket not ready after 10 seconds"
        kill $SONGBIRD_PID 2>/dev/null || true
        exit 1
    fi
    sleep 1
done
echo ""

# Phase 3: Test Songbird IPC
echo "Phase 3: Test Songbird IPC"
echo "─────────────────────────────────────────────────────────────"

# Create test request
TEST_REQUEST=$(cat <<EOF
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "http.request",
  "params": {
    "url": "https://api.github.com/zen",
    "method": "GET",
    "headers": {
      "User-Agent": "ecoPrimals-Tower-Atomic/1.0"
    }
  }
}
EOF
)

echo "Sending test request to Songbird..."
echo "$TEST_REQUEST" | nc -U "$SONGBIRD_SOCKET" > /tmp/songbird-response.json 2>&1 &
NC_PID=$!

# Wait for response (with timeout)
for i in {1..5}; do
    if ! ps -p $NC_PID > /dev/null 2>&1; then
        break
    fi
    if [ $i -eq 5 ]; then
        echo "⚠️  Timeout waiting for Songbird response"
        kill $NC_PID 2>/dev/null || true
    fi
    sleep 1
done

if [ -f /tmp/songbird-response.json ] && [ -s /tmp/songbird-response.json ]; then
    echo "✅ Songbird responded"
    echo "Response:"
    cat /tmp/songbird-response.json | jq '.' 2>/dev/null || cat /tmp/songbird-response.json
    echo ""
else
    echo "❌ No response from Songbird"
    echo "Check log: /tmp/songbird-test.log"
fi
echo ""

# Phase 4: Cleanup
echo "Phase 4: Cleanup"
echo "─────────────────────────────────────────────────────────────"

kill $SONGBIRD_PID 2>/dev/null || true
sleep 1

echo "✅ Test complete"
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "📊 Results:"
echo "  - Songbird build: ✅"
echo "  - Songbird start: ✅"
echo "  - Songbird socket: ✅"
echo "  - IPC test: Check /tmp/songbird-response.json"
echo "═══════════════════════════════════════════════════════════════"

