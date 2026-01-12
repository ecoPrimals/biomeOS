#!/usr/bin/env bash
# Clean launch script for full biomeOS UI + NUCLEUS visualization

set -euo pipefail

PROJECT_ROOT="/home/eastgate/Development/ecoPrimals/phase2/biomeOS"
cd "$PROJECT_ROOT"

UID_VAR=$(id -u)
SOCKET_PATH="/run/user/${UID_VAR}/biomeos-device-management.sock"
PETALTONGUE_BIN="$PROJECT_ROOT/plasmidBin/petaltongue"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧬 biomeOS NUCLEUS + UI Launch System"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Step 1: Check if device_management_server is running
if pgrep -f "device_management_server" > /dev/null; then
    echo "✅ device_management_server is running"
else
    echo "🚀 Starting device_management_server..."
    cargo run --bin device_management_server > /tmp/device_management_server.log 2>&1 &
    SERVER_PID=$!
    echo "   PID: $SERVER_PID"
    
    # Wait for socket
    echo "   Waiting for socket..."
    for i in {1..10}; do
        if [ -S "$SOCKET_PATH" ]; then
            echo "   ✅ Socket ready: $SOCKET_PATH"
            break
        fi
        sleep 0.5
    done
fi

echo ""

# Step 2: Launch petalTongue GUI
if [ ! -f "$PETALTONGUE_BIN" ]; then
    echo "❌ petalTongue binary not found at: $PETALTONGUE_BIN"
    exit 1
fi

echo "🌸 Launching petalTongue GUI..."
echo "   Binary: $PETALTONGUE_BIN"
echo "   Socket: $SOCKET_PATH"
echo ""

BIOMEOS_URL="unix://${SOCKET_PATH}" RUST_LOG=info "$PETALTONGUE_BIN"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Session complete"


