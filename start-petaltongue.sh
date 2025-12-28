#!/bin/bash
# Start PetalTongue UI for BiomeOS
# Universal Interface for primal visualization and management

set -e

echo "🌸 Starting PetalTongue UI..."
echo "=============================="
echo ""

# Configuration
PETALTONGUE_PORT="${PETALTONGUE_PORT:-8090}"
PETALTONGUE_MOCK_MODE="${PETALTONGUE_MOCK_MODE:-false}"
BIOMEOS_URL="${BIOMEOS_URL:-http://localhost:3000}"

# BiomeOS primal endpoints (discovered dynamically)
export BIOMEOS_URL
export SONGBIRD_URL="${SONGBIRD_URL:-http://localhost:2300}"
export NESTGATE_URL="${NESTGATE_URL:-http://localhost:9020}"
export PETALTONGUE_PORT
export PETALTONGUE_MOCK_MODE

echo "Configuration:"
echo "  Port: $PETALTONGUE_PORT"
echo "  BiomeOS: $BIOMEOS_URL"
echo "  Songbird: $SONGBIRD_URL"
echo "  NestGate: $NESTGATE_URL"
echo "  Mock Mode: $PETALTONGUE_MOCK_MODE"
echo ""

# Check if already running
if pgrep -f petal-tongue > /dev/null; then
    echo "⚠️  PetalTongue already running"
    echo "   PID: $(pgrep -f petal-tongue)"
    echo "   Use 'pkill -f petal-tongue' to stop"
    exit 0
fi

# Start PetalTongue
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PETALTONGUE_BIN="$SCRIPT_DIR/primals/petal-tongue"

if [ ! -f "$PETALTONGUE_BIN" ]; then
    echo "❌ PetalTongue binary not found: $PETALTONGUE_BIN"
    exit 1
fi

echo "🚀 Starting PetalTongue..."
echo "   Binary: $PETALTONGUE_BIN"
echo ""

# Start in background with nohup
nohup "$PETALTONGUE_BIN" > petaltongue.log 2>&1 &
PETALTONGUE_PID=$!

echo "✅ PetalTongue started"
echo "   PID: $PETALTONGUE_PID"
echo "   Port: $PETALTONGUE_PORT"
echo "   Log: petaltongue.log"
echo ""
echo "🌐 Access UI at: http://localhost:$PETALTONGUE_PORT"
echo ""
echo "💡 To stop: pkill -f petal-tongue"

