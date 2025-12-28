#!/usr/bin/env bash
# Deploy real primal binaries for biomeOS showcase
# NO MOCKS - Real primals only

set -e
cd "$(dirname "$0")"

echo "🚀 BiomeOS Real Primal Deployment"
echo "========================================"
echo ""

# Configuration
PRIMALS_DIR="./primals"
LOGS_DIR="./logs/primals"
PIDS_DIR="./logs/pids"

# Generate secure JWT secret for NestGate if not set
if [ -z "$NESTGATE_JWT_SECRET" ]; then
    NESTGATE_JWT_SECRET=$(openssl rand -base64 48)
    echo "🔐 Generated secure JWT secret for NestGate"
fi

# Create log directories
mkdir -p "$LOGS_DIR" "$PIDS_DIR"

# Primal configurations (port, binary name)
declare -A PRIMALS=(
    ["nestgate"]="9020"
    ["songbird"]="9000"
    ["beardog"]="9040"
    ["toadstool"]="9030"
    ["squirrel"]="9050"
)

# Check binaries exist
echo "📋 Checking primal binaries..."
missing=0
for primal in "${!PRIMALS[@]}"; do
    if [ -f "$PRIMALS_DIR/$primal" ]; then
        size=$(du -h "$PRIMALS_DIR/$primal" | cut -f1)
        echo "  ✓ $primal ($size)"
    else
        echo "  ✗ $primal - MISSING!"
        ((missing++))
    fi
done

if [ $missing -gt 0 ]; then
    echo ""
    echo "❌ Missing $missing primal binaries!"
    echo "   Check $PRIMALS_DIR directory"
    exit 1
fi

echo ""
echo "🧹 Cleaning up old processes..."
# Kill any existing primals
for primal in "${!PRIMALS[@]}"; do
    if [ -f "$PIDS_DIR/$primal.pid" ]; then
        old_pid=$(cat "$PIDS_DIR/$primal.pid")
        if ps -p "$old_pid" > /dev/null 2>&1; then
            echo "  Stopping old $primal (PID: $old_pid)..."
            kill "$old_pid" 2>/dev/null || true
            sleep 1
        fi
        rm "$PIDS_DIR/$primal.pid"
    fi
done

# Also check for any rogue processes
pkill -f "primals/nestgate" 2>/dev/null || true
pkill -f "primals/songbird" 2>/dev/null || true
pkill -f "primals/beardog" 2>/dev/null || true
pkill -f "primals/toadstool" 2>/dev/null || true
pkill -f "primals/squirrel" 2>/dev/null || true

sleep 2

echo ""
echo "🚀 Starting primals..."
started=0
for primal in "${!PRIMALS[@]}"; do
    port="${PRIMALS[@]/$primal}"
    port="${PRIMALS[$primal]}"
    
    echo "  Starting $primal on port $port..."
    
    # Different start commands per primal
    case "$primal" in
        nestgate)
            # NestGate requires 'service start' subcommand and JWT secret
            PORT="$port" \
            HOST="0.0.0.0" \
            NESTGATE_API_PORT="$port" \
            NESTGATE_API_ENDPOINT="http://0.0.0.0:$port" \
            NESTGATE_JWT_SECRET="$NESTGATE_JWT_SECRET" \
            RUST_LOG="info" \
            "$PRIMALS_DIR/$primal" service start --port "$port" \
                > "$LOGS_DIR/$primal.log" 2>&1 &
            ;;
        songbird)
            # Songbird runs as-is
            PORT="$port" \
            HOST="0.0.0.0" \
            SONGBIRD_PORT="$port" \
            RUST_LOG="info" \
            "$PRIMALS_DIR/$primal" \
                > "$LOGS_DIR/$primal.log" 2>&1 &
            ;;
        beardog)
            # BearDog (Wireguard P2P)
            PORT="$port" \
            HOST="0.0.0.0" \
            BEARDOG_PORT="$port" \
            RUST_LOG="info" \
            "$PRIMALS_DIR/$primal" \
                > "$LOGS_DIR/$primal.log" 2>&1 &
            ;;
        toadstool)
            # Toadstool (compute orchestration)
            PORT="$port" \
            HOST="0.0.0.0" \
            TOADSTOOL_PORT="$port" \
            RUST_LOG="info" \
            "$PRIMALS_DIR/$primal" \
                > "$LOGS_DIR/$primal.log" 2>&1 &
            ;;
        squirrel)
            # Squirrel (configuration management)
            PORT="$port" \
            HOST="0.0.0.0" \
            SQUIRREL_PORT="$port" \
            RUST_LOG="info" \
            "$PRIMALS_DIR/$primal" \
                > "$LOGS_DIR/$primal.log" 2>&1 &
            ;;
        *)
            # Generic fallback
            PORT="$port" \
            HOST="0.0.0.0" \
            RUST_LOG="info" \
            "$PRIMALS_DIR/$primal" \
                > "$LOGS_DIR/$primal.log" 2>&1 &
            ;;
    esac
    
    pid=$!
    echo $pid > "$PIDS_DIR/$primal.pid"
    
    # Quick check if it started
    sleep 1
    if ps -p "$pid" > /dev/null 2>&1; then
        echo "    ✓ Started (PID: $pid)"
        ((started++))
    else
        echo "    ✗ Failed to start!"
        echo "    Check logs: $LOGS_DIR/$primal.log"
    fi
done

echo ""
echo "⏳ Waiting for primals to initialize (5s)..."
sleep 5

echo ""
echo "🔍 Verifying primal health..."
healthy=0
for primal in "${!PRIMALS[@]}"; do
    port="${PRIMALS[$primal]}"
    
    # Try health check
    if curl -s -f "http://localhost:$port/health" > /dev/null 2>&1; then
        echo "  ✓ $primal healthy (http://localhost:$port)"
        ((healthy++))
    elif curl -s -f "http://localhost:$port/api/health" > /dev/null 2>&1; then
        echo "  ✓ $primal healthy (http://localhost:$port/api/health)"
        ((healthy++))
    else
        pid=$(cat "$PIDS_DIR/$primal.pid" 2>/dev/null || echo "unknown")
        if ps -p "$pid" > /dev/null 2>&1; then
            echo "  ⚠ $primal running (PID: $pid) but no health endpoint"
            echo "    Check logs: tail -f $LOGS_DIR/$primal.log"
            ((healthy++))
        else
            echo "  ✗ $primal not responding"
            echo "    Check logs: cat $LOGS_DIR/$primal.log"
        fi
    fi
done

echo ""
echo "✅ Deployment Summary"
echo "========================================"
echo "  Started: $started/${#PRIMALS[@]} primals"
echo "  Healthy: $healthy/${#PRIMALS[@]} primals"
echo ""
echo "📁 Logs: $LOGS_DIR/"
echo "📁 PIDs: $PIDS_DIR/"
echo ""

if [ $healthy -eq ${#PRIMALS[@]} ]; then
    echo "🎉 All primals deployed successfully!"
    echo ""
    echo "🔗 Endpoints:"
    for primal in nestgate songbird beardog toadstool squirrel; do
        port="${PRIMALS[$primal]}"
        echo "  $primal: http://localhost:$port"
    done
    echo ""
    echo "📊 Monitor logs:"
    echo "  tail -f $LOGS_DIR/*.log"
    echo ""
    echo "🛑 Stop primals:"
    echo "  ./stop-primals.sh"
    echo ""
else
    echo "⚠ Some primals failed to deploy"
    echo ""
    echo "🔍 Debug:"
    echo "  Check logs: ls -lh $LOGS_DIR/"
    echo "  View specific log: cat $LOGS_DIR/[primal].log"
    echo ""
fi

# Create stop script
cat > stop-primals.sh << 'STOP_EOF'
#!/usr/bin/env bash
# Stop all running primals

PIDS_DIR="./logs/pids"

echo "🛑 Stopping primals..."
for pidfile in "$PIDS_DIR"/*.pid; do
    if [ -f "$pidfile" ]; then
        primal=$(basename "$pidfile" .pid)
        pid=$(cat "$pidfile")
        if ps -p "$pid" > /dev/null 2>&1; then
            echo "  Stopping $primal (PID: $pid)..."
            kill "$pid"
        fi
        rm "$pidfile"
    fi
done

echo "✓ All primals stopped"
STOP_EOF

chmod +x stop-primals.sh

echo "✅ Ready for showcase demos!"
