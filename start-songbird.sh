#!/usr/bin/env bash
# Start Songbird Tower for biomeOS
# Uses Songbird's built-in mDNS/UDP discovery - no hardcoded ports!

set -e
cd "$(dirname "$0")"

echo "🎵 Starting Songbird Tower for BiomeOS"
echo "========================================"
echo ""

# Check if Songbird binary exists
SONGBIRD_PHASE1="/home/eastgate/Development/ecoPrimals/phase1/songbird"
SONGBIRD_BIN="$SONGBIRD_PHASE1/target/release/songbird-orchestrator"

if [ ! -f "$SONGBIRD_BIN" ]; then
    echo "❌ Songbird binary not found at: $SONGBIRD_BIN"
    echo ""
    echo "Building Songbird..."
    cd "$SONGBIRD_PHASE1"
    cargo build --release
    echo "✅ Build complete"
    echo ""
fi

# Kill any existing songbird
pkill -f songbird-orchestrator 2>/dev/null && echo "  Stopped existing Songbird" || true
sleep 2

# Create logs directory
mkdir -p logs/primals logs/pids

# Start Songbird with biomeOS configuration
echo "🚀 Starting Songbird Orchestrator..."
echo "  Configuration:"
echo "    • TLS: Enabled"
echo "    • Discovery: Anonymous UDP broadcast (port 2300)"
echo "    • Federation: Enabled with zero-trust"
echo "    • Network: Auto-detected (mDNS)"
echo ""

LOG_FILE="logs/primals/songbird.log"

# Start with biomeOS-specific settings
SONGBIRD_TLS_ENABLED=true \
SONGBIRD_FEDERATION_ENABLED=true \
SONGBIRD_ANONYMOUS_DISCOVERY=true \
SONGBIRD_NODE_NAME="biomeOS-$(hostname)" \
SONGBIRD_TOWER_NAME="biomeOS-tower" \
RUST_LOG="info" \
nohup "$SONGBIRD_BIN" > "$LOG_FILE" 2>&1 &

SONGBIRD_PID=$!
echo $SONGBIRD_PID > logs/pids/songbird.pid

echo "✅ Songbird started (PID: $SONGBIRD_PID)"
echo ""

# Wait for startup
echo "⏳ Waiting for Songbird to initialize..."
sleep 5

# Check if still running
if ! ps -p $SONGBIRD_PID > /dev/null 2>&1; then
    echo "❌ Songbird failed to start"
    echo "Check logs: $LOG_FILE"
    tail -20 "$LOG_FILE"
    exit 1
fi

# Detect HTTPS port
echo "🔍 Detecting Songbird services..."
HTTPS_PORT=$(sudo lsof -i -P -n 2>/dev/null | grep "$SONGBIRD_PID" | grep TCP | grep LISTEN | head -1 | awk '{print $9}' | cut -d':' -f2 || echo "")

if [ -n "$HTTPS_PORT" ]; then
    echo "  ✅ HTTPS Server: Port $HTTPS_PORT"
    echo "  ✅ Discovery: UDP port 2300"
    echo ""
    echo "📡 Songbird Endpoints:"
    echo "  Local: https://localhost:$HTTPS_PORT"
    LOCAL_IP=$(hostname -I | awk '{print $1}')
    echo "  Network: https://$LOCAL_IP:$HTTPS_PORT"
    echo "  Discovery: UDP broadcast on $LOCAL_IP:2300"
else
    echo "  ⏳ Services still initializing..."
    echo "  Check: tail -f $LOG_FILE"
fi

echo ""
echo "✅ Songbird Tower Ready!"
echo ""
echo "🌐 Discovery Mode:"
echo "  • Songbird broadcasts its capabilities via UDP (port 2300)"
echo "  • Other towers auto-discover this node"
echo "  • BiomeOS can discover Songbird via mDNS"
echo "  • NO HARDCODED PORTS - fully dynamic!"
echo ""
echo "📋 Monitoring:"
echo "  Logs: tail -f $LOG_FILE"
echo "  Filter: tail -f $LOG_FILE | grep discovery"
if [ -n "$HTTPS_PORT" ]; then
    echo "  API: curl -k https://localhost:$HTTPS_PORT/api/info"
fi
echo ""
echo "🛑 Stop: pkill -f songbird-orchestrator"
echo ""
echo "🎵 Songbird is singing! Other primals will discover it automatically."

