#!/bin/bash
# Launch Full UI Integration with petalTongue GUI
#
# This script:
# 1. Starts required primals (if not running)
# 2. Registers biomeOS device.management capability
# 3. Launches petalTongue GUI with proper discovery

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$(dirname "$SCRIPT_DIR")"
PETALTONGUE_BIN="$BIOMEOS_ROOT/plasmidBin/petaltongue"

echo "═══════════════════════════════════════════════════════════════"
echo "    🌸 BIOMEOS + PETALTONGUE FULL UI INTEGRATION 🌸"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Check if petalTongue binary exists
if [ ! -f "$PETALTONGUE_BIN" ]; then
    echo "❌ ERROR: petalTongue binary not found at $PETALTONGUE_BIN"
    echo "   Please run: cargo build --release in ../petalTongue"
    echo "   Then: cp ../petalTongue/target/release/petal-tongue $PETALTONGUE_BIN"
    exit 1
fi

echo "✅ Found petalTongue binary"

# Check for Songbird (required for discovery)
if ! pgrep -x songbird > /dev/null; then
    echo "⚠️  Songbird not running - attempting to start..."
    
    if [ -f "$BIOMEOS_ROOT/plasmidBin/songbird" ]; then
        echo "🎵 Starting Songbird..."
        "$BIOMEOS_ROOT/plasmidBin/songbird" &
        SONGBIRD_PID=$!
        sleep 2
        
        if ps -p $SONGBIRD_PID > /dev/null; then
            echo "✅ Songbird started (PID: $SONGBIRD_PID)"
        else
            echo "❌ Failed to start Songbird"
        fi
    else
        echo "⚠️  Songbird binary not found"
        echo "   petalTongue will use mock mode"
    fi
else
    echo "✅ Songbird is running"
fi

# Get UID for socket path
UID_VAL=$(id -u)
SOCKET_PATH="/run/user/${UID_VAL}/biomeos-ui.sock"

echo ""
echo "📡 Configuration:"
echo "   Socket: $SOCKET_PATH"
echo "   Display: ${DISPLAY:-:0}"
echo ""

# Option 1: Start with mock mode (no biomeOS backend)
echo "🎨 Launching petalTongue GUI..."
echo "   (Using mock mode - biomeOS backend not yet wired)"
echo ""

# Set environment for petalTongue
export RUST_LOG=info
export RUST_BACKTRACE=1

# Launch petalTongue
"$PETALTONGUE_BIN" 2>&1 | tee /tmp/petaltongue-ui.log &
PETALTONGUE_PID=$!

echo "✅ petalTongue GUI launched (PID: $PETALTONGUE_PID)"
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "    🌸 GUI RUNNING - CHECK YOUR DISPLAY! 🌸"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Status:"
echo "  • petalTongue GUI: Running in mock mode"
echo "  • You should see the UI window with:"
echo "    - DevicePanel (device management)"
echo "    - PrimalPanel (primal status)"
echo "    - NicheDesigner (niche templates)"
echo ""
echo "  • Demo data includes:"
echo "    - 7 devices (GPUs, CPUs, storage, network)"
echo "    - 6 primals (Songbird, ToadStool, BearDog, etc.)"
echo "    - 3 niche templates (Tower, Node, Nest)"
echo ""
echo "Log file: /tmp/petaltongue-ui.log"
echo ""
echo "Press Ctrl+C to stop"
echo ""

# Wait for user interrupt
wait $PETALTONGUE_PID

echo ""
echo "👋 Shutting down..."

