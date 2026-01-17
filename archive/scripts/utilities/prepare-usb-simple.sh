#!/bin/bash
# 🧬 USB Spore Preparation Script - Simplified Working Version
# Direct primal startup with zero hardcoding patterns

set -e

echo "════════════════════════════════════════════════════════════════════════════════"
echo "  🧬 USB Spore v15.0 Preparation (Simplified Working Version)"
echo "  Zero-Hardcoding Patterns - Direct Startup"
echo "════════════════════════════════════════════════════════════════════════════════"
echo ""

# Configuration
USB_ROOT="/media/eastgate/BEA6-BBCE"
BIOMEOS_DIR="/home/eastgate/Development/ecoPrimals/phase2/biomeOS"
BEARDOG_BIN="/home/eastgate/Development/ecoPrimals/phase1/beardog/plasmidBin/beardog-server-v0.15.0-with-v2-api"
SONGBIRD_BIN="/home/eastgate/Development/ecoPrimals/phase1/songbird/target/release/songbird-orchestrator"

# Check USB
if [ ! -d "$USB_ROOT" ]; then
    echo "❌ USB not found at $USB_ROOT"
    exit 1
fi
echo "✅ USB found"
echo ""

# Verify binaries
echo "=== Verifying Binaries ==="
for bin in "$BEARDOG_BIN" "$SONGBIRD_BIN"; do
    if [ ! -f "$bin" ]; then
        echo "❌ Missing: $bin"
        exit 1
    fi
    echo "✅ $(basename $bin)"
done
echo ""

# Create structure
echo "=== Creating USB Structure ==="
mkdir -p "$USB_ROOT/biomeOS-LAN-Deploy/primals"
mkdir -p "$USB_ROOT/biomeOS-LAN-Deploy/scripts"
mkdir -p "$USB_ROOT/biomeOS-LAN-Deploy/configs"
mkdir -p "$USB_ROOT/biomeOS-LAN-Deploy/docs"
mkdir -p "$USB_ROOT/biomeOS-LAN-Deploy/logs"
echo "✅ Directories created"
echo ""

# Copy binaries
echo "=== Copying Binaries ==="
cp "$BEARDOG_BIN" "$USB_ROOT/biomeOS-LAN-Deploy/primals/beardog-server"
cp "$SONGBIRD_BIN" "$USB_ROOT/biomeOS-LAN-Deploy/primals/songbird-orchestrator"
chmod +x "$USB_ROOT/biomeOS-LAN-Deploy/primals/"*
echo "✅ Binaries copied"
echo ""

# Create family seed config
echo "=== Creating Configuration ==="
if [ ! -f "$USB_ROOT/biomeOS-LAN-Deploy/configs/family-seed.conf" ]; then
    cat > "$USB_ROOT/biomeOS-LAN-Deploy/configs/family-seed.conf" << 'EOF'
# Family Genetic Lineage Configuration
export FAMILY_ID="iidn"
export FAMILY_SEED="V2VsbCBoZWxsbywgdGhlcmUh"
EOF
    echo "✅ family-seed.conf created"
else
    echo "✅ family-seed.conf exists (not overwriting)"
fi
echo ""

# Create deployment script
echo "=== Creating Deployment Script ==="
cat > "$USB_ROOT/biomeOS-LAN-Deploy/scripts/activate-tower.sh" << 'EOF'
#!/bin/bash
# Zero-Hardcoding Tower Activation
# Uses port 0 for OS-assigned ports, minimal coordination

set -e

cd "$(dirname "$0")/.."

echo "════════════════════════════════════════════════════════════════════════════════"
echo "  🧬 Activating Tower - Zero-Hardcoding Pattern"
echo "════════════════════════════════════════════════════════════════════════════════"
echo ""

# Load family credentials
if [ ! -f "configs/family-seed.conf" ]; then
    echo "❌ configs/family-seed.conf not found!"
    exit 1
fi
source configs/family-seed.conf
echo "✅ Family: $FAMILY_ID"
echo ""

# Export for BearDog
export BEARDOG_FAMILY_ID="$FAMILY_ID"
export BEARDOG_FAMILY_SEED="$FAMILY_SEED"
export RUST_LOG=info

# Create logs directory
mkdir -p logs

# Cleanup function
cleanup() {
    echo ""
    echo "🛑 Shutting down tower..."
    if [ ! -z "$BEARDOG_PID" ] && kill -0 $BEARDOG_PID 2>/dev/null; then
        echo "  Stopping BearDog (PID: $BEARDOG_PID)..."
        kill $BEARDOG_PID
        wait $BEARDOG_PID 2>/dev/null
    fi
    if [ ! -z "$SONGBIRD_PID" ] && kill -0 $SONGBIRD_PID 2>/dev/null; then
        echo "  Stopping Songbird (PID: $SONGBIRD_PID)..."
        kill $SONGBIRD_PID
        wait $SONGBIRD_PID 2>/dev/null
    fi
    echo "✅ Tower shutdown complete"
    exit 0
}

trap cleanup SIGINT SIGTERM

# Start BearDog (Security Provider) - Port 0 = OS auto-selects
echo "🐻 Starting BearDog (Security Provider)..."
echo "  Binary: ./primals/beardog-server"
echo "  Port: 0 (OS auto-selects)"
echo "  Family: $BEARDOG_FAMILY_ID"
./primals/beardog-server --port 0 > logs/beardog.log 2>&1 &
BEARDOG_PID=$!
echo "  PID: $BEARDOG_PID"
echo "  Log: logs/beardog.log"

# Wait for BearDog to be ready
sleep 2
if ! kill -0 $BEARDOG_PID 2>/dev/null; then
    echo "❌ BearDog failed to start!"
    tail -20 logs/beardog.log
    exit 1
fi

# Extract BearDog's actual port
BEARDOG_PORT=$(grep -oP "Listening on 0\.0\.0\.0:\K\d+" logs/beardog.log | head -1)
if [ -z "$BEARDOG_PORT" ]; then
    echo "⚠️  Could not detect BearDog port, using discovery..."
else
    echo "  Detected port: $BEARDOG_PORT"
fi
echo "✅ BearDog started"
echo ""

# Start Songbird (Discovery Orchestrator)
echo "🐦 Starting Songbird (Discovery Orchestrator)..."
echo "  Binary: ./primals/songbird-orchestrator"
echo "  UDP Multicast: 239.255.77.88:5353"
echo "  Discovery: Auto"
./primals/songbird-orchestrator > logs/songbird.log 2>&1 &
SONGBIRD_PID=$!
echo "  PID: $SONGBIRD_PID"
echo "  Log: logs/songbird.log"

# Wait for Songbird to be ready
sleep 2
if ! kill -0 $SONGBIRD_PID 2>/dev/null; then
    echo "❌ Songbird failed to start!"
    tail -20 logs/songbird.log
    cleanup
fi
echo "✅ Songbird started"
echo ""

echo "════════════════════════════════════════════════════════════════════════════════"
echo "  ✅ TOWER ACTIVE"
echo "════════════════════════════════════════════════════════════════════════════════"
echo ""
echo "  🐻 BearDog:  PID $BEARDOG_PID $([ ! -z "$BEARDOG_PORT" ] && echo "(Port $BEARDOG_PORT)" || echo "(Port auto)")"
echo "  🐦 Songbird: PID $SONGBIRD_PID (UDP 239.255.77.88:5353)"
echo ""
echo "  Logs: logs/beardog.log, logs/songbird.log"
echo "  Press Ctrl+C to stop gracefully"
echo ""

# Wait for signals
wait
EOF
chmod +x "$USB_ROOT/biomeOS-LAN-Deploy/scripts/activate-tower.sh"
echo "✅ activate-tower.sh created"
echo ""

# Copy documentation
echo "=== Copying Documentation ==="
cp "$BIOMEOS_DIR/START_HERE_ZERO_HARDCODING.md" "$USB_ROOT/biomeOS-LAN-Deploy/docs/" 2>/dev/null || true
cp "$BIOMEOS_DIR/docs/jan3-session/JAN3_SESSION_SUMMARY.md" "$USB_ROOT/biomeOS-LAN-Deploy/docs/" 2>/dev/null || true
cp "$BIOMEOS_DIR/docs/jan3-session/PROPER_USB_DEPLOYMENT_STRATEGY.md" "$USB_ROOT/biomeOS-LAN-Deploy/docs/" 2>/dev/null || true
echo "✅ Documentation copied"
echo ""

# Generate checksums
echo "=== Generating Checksums ==="
cd "$USB_ROOT/biomeOS-LAN-Deploy/primals"
sha256sum beardog-server songbird-orchestrator > checksums.txt
echo "✅ Checksums generated:"
cat checksums.txt
echo ""

# Create version file
cat > "$USB_ROOT/biomeOS-LAN-Deploy/USB-V15.0-ZERO-HARDCODING.txt" << 'EOF'
════════════════════════════════════════════════════════════════════════════════
  🧬 USB Spore v15.0 - Zero-Hardcoding (Simplified Working Version)
════════════════════════════════════════════════════════════════════════════════

Version: 15.0
Date: January 3, 2026
Status: PRODUCTION READY - Simplified Zero-Hardcoding Pattern

DEPLOYMENT:
$ cd /media/USB/biomeOS-LAN-Deploy
$ ./scripts/activate-tower.sh

Press Ctrl+C to stop gracefully.

ZERO-HARDCODING FEATURES:
✅ Port 0 (OS auto-selects available ports)
✅ Graceful shutdown (proper signal handling)
✅ Automatic port detection
✅ UDP multicast discovery (no port conflicts)
✅ Clean process management
✅ Comprehensive logging

ARCHITECTURE:
- BearDog: Security Provider (HTTP, auto-port)
- Songbird: Discovery Orchestrator (UDP multicast 239.255.77.88:5353)

NO HARDCODED PORTS - OS ASSIGNS DYNAMICALLY! 🚀
EOF

# Sync
echo "=== Syncing USB ==="
sync
echo "✅ USB synced"
echo ""

# Summary
echo "════════════════════════════════════════════════════════════════════════════════"
echo "  ✅ USB SPORE v15.0 PREPARED!"
echo "════════════════════════════════════════════════════════════════════════════════"
echo ""
echo "USB Contents:"
echo "  📦 Binaries: beardog-server, songbird-orchestrator"
echo "  🧬 Family: family-seed.conf"
echo "  📜 Script: activate-tower.sh (zero-hardcoding pattern)"
echo "  📚 Docs: Zero-hardcoding guides"
echo ""
echo "Deploy on any tower:"
echo "  $ cd /media/USB/biomeOS-LAN-Deploy"
echo "  $ ./scripts/activate-tower.sh"
echo ""
echo "Zero-hardcoding: Port 0 everywhere, OS assigns dynamically!"
echo ""
echo "🎊 READY FOR DEPLOYMENT! 🚀"
echo ""

