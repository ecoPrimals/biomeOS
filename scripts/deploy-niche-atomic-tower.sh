#!/usr/bin/env bash
# 🌳 TRUE PRIMAL Tower Niche Deployment
# 
# Deploys Tower atomic (BearDog + Songbird) with LiveSpore genetic lineage
# Uses environment-based discovery, NO hardcoding!
#
# Usage:
#   FAMILY_ID=nat0 ./scripts/deploy-niche-atomic-tower.sh
#   FAMILY_ID=custom USB_SEED=/path/to/usb/.family.seed ./scripts/deploy-niche-atomic-tower.sh

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

FAMILY_ID="${FAMILY_ID:-nat0}"
USB_SEED="${USB_SEED:-}"
BIOMEOS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🌳 TRUE PRIMAL Tower Niche Deployment"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Configuration:"
echo "  Family ID: $FAMILY_ID"
echo "  USB Seed: ${USB_SEED:-<none - will generate>}"
echo "  BiomeOS Dir: $BIOMEOS_DIR"
echo ""

# ============================================================================
# Step 1: LiveSpore - Create USB Seed (if not provided)
# ============================================================================

if [ -z "$USB_SEED" ]; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🌱 Step 1: Generate LiveSpore USB Seed"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    USB_SEED="/tmp/livespore-${FAMILY_ID}/.family.seed"
    mkdir -p "/tmp/livespore-${FAMILY_ID}"
    
    # Generate unique genetic seed
    GENETIC_SEED=$(openssl rand -hex 32)
    echo "$GENETIC_SEED" > "$USB_SEED"
    
    echo "✅ Created USB seed: $USB_SEED"
    echo "   Genetic lineage: ${GENETIC_SEED:0:16}..."
    echo ""
else
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🌱 Step 1: Using Existing LiveSpore USB Seed"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    if [ ! -f "$USB_SEED" ]; then
        echo "❌ Error: USB seed not found at $USB_SEED"
        exit 1
    fi
    
    GENETIC_SEED=$(cat "$USB_SEED")
    echo "✅ Loaded USB seed: $USB_SEED"
    echo "   Genetic lineage: ${GENETIC_SEED:0:16}..."
    echo ""
fi

# ============================================================================
# Step 2: Tower Atomic - Start BearDog
# ============================================================================

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🐻🐕 Step 2: Start BearDog (Security Primal)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if BearDog already running
BEARDOG_SOCKET="/run/user/$(id -u)/beardog-${FAMILY_ID}.sock"
if [ -S "$BEARDOG_SOCKET" ]; then
    echo "⚠️  BearDog already running (socket exists)"
    echo "   Socket: $BEARDOG_SOCKET"
else
    echo "Starting BearDog..."
    
    # TRUE PRIMAL way: Just run with env vars!
    FAMILY_ID="$FAMILY_ID" \
    NODE_ID="tower-beardog" \
    USB_SEED="$USB_SEED" \
    nohup "$BIOMEOS_DIR/plasmidBin/beardog" \
        > "/tmp/beardog-${FAMILY_ID}.log" 2>&1 &
    
    BEARDOG_PID=$!
    echo "   PID: $BEARDOG_PID"
    echo "   Log: /tmp/beardog-${FAMILY_ID}.log"
    
    # Wait for socket creation
    echo "   Waiting for socket creation..."
    for i in {1..10}; do
        if [ -S "$BEARDOG_SOCKET" ]; then
            echo "   ✅ Socket created: $BEARDOG_SOCKET"
            break
        fi
        sleep 0.5
    done
fi

echo ""

# ============================================================================
# Step 3: Tower Atomic - Verify Songbird (should be running)
# ============================================================================

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🐦 Step 3: Verify Songbird (Discovery Primal)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

SONGBIRD_SOCKET="/run/user/$(id -u)/songbird-${FAMILY_ID}.sock"
if [ -S "$SONGBIRD_SOCKET" ]; then
    echo "✅ Songbird already running"
    echo "   Socket: $SONGBIRD_SOCKET"
else
    echo "⚠️  Songbird not found, starting..."
    
    # Check if we have songbird binary
    if [ ! -f "$BIOMEOS_DIR/plasmidBin/songbird" ]; then
        echo "❌ Songbird binary not found in plasmidBin/"
        echo "   Continuing with available primals..."
    else
        FAMILY_ID="$FAMILY_ID" \
        NODE_ID="tower-songbird" \
        USB_SEED="$USB_SEED" \
        nohup "$BIOMEOS_DIR/plasmidBin/songbird" \
            > "/tmp/songbird-${FAMILY_ID}.log" 2>&1 &
        
        SONGBIRD_PID=$!
        echo "   PID: $SONGBIRD_PID"
        echo "   Log: /tmp/songbird-${FAMILY_ID}.log"
        
        # Wait for socket creation
        echo "   Waiting for socket creation..."
        for i in {1..10}; do
            if [ -S "$SONGBIRD_SOCKET" ]; then
                echo "   ✅ Socket created: $SONGBIRD_SOCKET"
                break
            fi
            sleep 0.5
        done
    fi
fi

echo ""

# ============================================================================
# Step 4: Discovery - List All Sockets
# ============================================================================

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 Step 4: Discover All Primals (via Unix Sockets)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "BiomeOS primals discovered:"
ls -lh /run/user/$(id -u)/*.sock 2>/dev/null | \
    grep -E "(beardog|songbird|toadstool|nestgate|squirrel|biomeos)" | \
    awk '{print "  •", $9}' || echo "  (None yet)"

echo ""

# ============================================================================
# Step 5: BiomeOS API - Discovery & Coordination
# ============================================================================

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🌍 Step 5: Start biomeOS API (Discovery Engine)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if already running
if pgrep -f "biomeos-api" > /dev/null; then
    echo "⚠️  biomeOS API already running"
    API_PID=$(pgrep -f "biomeos-api")
    echo "   PID: $API_PID"
else
    echo "Starting biomeOS API..."
    echo "  (This will discover all primals via socket scanning)"
    echo ""
    
    cd "$BIOMEOS_DIR"
    FAMILY_ID="$FAMILY_ID" \
    USB_SEED="$USB_SEED" \
    nohup cargo run --quiet -p biomeos-api \
        > "/tmp/biomeos-api-${FAMILY_ID}.log" 2>&1 &
    
    API_PID=$!
    echo "   PID: $API_PID"
    echo "   Log: /tmp/biomeos-api-${FAMILY_ID}.log"
    echo "   Endpoint: http://localhost:3000"
    
    # Wait for API to start
    echo "   Waiting for API startup..."
    for i in {1..20}; do
        if curl -s http://localhost:3000/api/v1/health > /dev/null 2>&1; then
            echo "   ✅ API ready!"
            break
        fi
        sleep 0.5
    done
fi

echo ""

# ============================================================================
# Step 6: PetalTongue - Visualization
# ============================================================================

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🌸 Step 6: Launch PetalTongue (Visualization UI)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Ready to launch PetalTongue!"
echo ""
echo "Choose mode:"
echo "  1. GUI (graphical):     BIOMEOS_URL=http://localhost:3000 $BIOMEOS_DIR/plasmidBin/petal-tongue"
echo "  2. TUI (terminal):      BIOMEOS_URL=http://localhost:3000 $BIOMEOS_DIR/plasmidBin/petal-tongue-headless --mode terminal"
echo "  3. Headless (SVG):      BIOMEOS_URL=http://localhost:3000 $BIOMEOS_DIR/plasmidBin/petal-tongue-headless --mode svg --output /tmp/tower.svg"
echo ""

# Auto-launch TUI by default
echo "Auto-launching TUI mode in 3 seconds... (Ctrl+C to skip)"
sleep 3 || true

echo ""
echo "🌸 Launching PetalTongue TUI..."
BIOMEOS_URL=http://localhost:3000 \
PETALTONGUE_REFRESH_INTERVAL=2.0 \
RUST_LOG=info \
exec "$BIOMEOS_DIR/plasmidBin/petal-tongue-headless" --mode terminal

