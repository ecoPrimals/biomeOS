#!/bin/bash
# =============================================================================
# Quick Start NUCLEUS AI Integration Test
# =============================================================================
#
# Fast startup script using pre-built primal binaries
#
# Hardware: RTX 4070 12GB VRAM
# Primals: beardog, songbird, nestgate, toadstool, squirrel
#
# Created: January 29, 2026
# =============================================================================

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[✓]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[!]${NC} $1"; }

# Configuration
PRIMALS_BASE="/home/eastgate/Development/ecoPrimals/phase1"
PRIMAL_BINS="/home/eastgate/Development/ecoPrimals/primalBins"
FAMILY_ID="${FAMILY_ID:-nucleus-test-$(date +%s)}"
USER_ID=$(id -u)
SOCKET_DIR="/run/user/$USER_ID/biomeos"
API_KEYS="/home/eastgate/Development/ecoPrimals/testing-secrets/api-keys.toml"

# Cleanup function
cleanup() {
    echo ""
    log_info "Cleaning up..."
    pkill -f "beardog.*server" || true
    pkill -f "songbird.*server" || true
    pkill -f "nestgate.*server" || true
    pkill -f "toadstool.*server" || true
    pkill -f "squirrel.*server" || true
    sleep 2
    rm -f "$SOCKET_DIR"/*.sock || true
    log_success "Cleanup complete"
}

trap cleanup EXIT INT TERM

# Create socket directory
mkdir -p "$SOCKET_DIR"

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║       NUCLEUS AI Integration Test - Quick Start              ║"
echo "╠══════════════════════════════════════════════════════════════╣"
echo "║  Hardware: RTX 4070 (12GB VRAM)                              ║"
echo "║  Family ID: $FAMILY_ID" | head -c 66
echo "║"
echo "║  Socket Dir: $SOCKET_DIR" | head -c 66
echo "║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Phase 1: Tower Atomic
log_info "🏗️  Phase 1: Deploying Tower Atomic (BearDog + Songbird)..."

# Start BearDog
log_info "Starting BearDog..."
BEARDOG_BIN="$PRIMALS_BASE/beardog/target/release/beardog"
if [ ! -f "$BEARDOG_BIN" ]; then
    BEARDOG_BIN="$PRIMAL_BINS/beardog"
fi

if [ -f "$BEARDOG_BIN" ]; then
    cd "$(dirname "$BEARDOG_BIN")"
    BEARDOG_SOCKET="$SOCKET_DIR/beardog.sock" \
    FAMILY_ID="$FAMILY_ID" \
    RUST_LOG=beardog=info \
    "$BEARDOG_BIN" server > /tmp/beardog.log 2>&1 &
    BEARDOG_PID=$!
    log_success "BearDog started (PID: $BEARDOG_PID)"
    cd - > /dev/null
else
    log_warning "BearDog binary not found"
fi

# Wait for BearDog socket
for i in {1..15}; do
    if [ -S "$SOCKET_DIR/beardog.sock" ]; then
        log_success "BearDog socket ready"
        break
    fi
    sleep 1
done

# Start Songbird
log_info "Starting Songbird..."
SONGBIRD_BIN="$PRIMALS_BASE/songbird/target/release/songbird"
if [ ! -f "$SONGBIRD_BIN" ]; then
    SONGBIRD_BIN="$PRIMAL_BINS/songbird"
fi

if [ -f "$SONGBIRD_BIN" ]; then
    cd "$(dirname "$SONGBIRD_BIN")"
    SONGBIRD_SOCKET="$SOCKET_DIR/songbird.sock" \
    BEARDOG_SOCKET="$SOCKET_DIR/beardog.sock" \
    SONGBIRD_SECURITY_PROVIDER="beardog" \
    FAMILY_ID="$FAMILY_ID" \
    RUST_LOG=songbird=info \
    "$SONGBIRD_BIN" server > /tmp/songbird.log 2>&1 &
    SONGBIRD_PID=$!
    log_success "Songbird started (PID: $SONGBIRD_PID)"
    cd - > /dev/null
else
    log_warning "Songbird binary not found"
fi

# Wait for Songbird socket
for i in {1..15}; do
    if [ -S "$SOCKET_DIR/songbird.sock" ]; then
        log_success "Songbird socket ready"
        break
    fi
    sleep 1
done

log_success "✅ Tower Atomic deployed!"

# Phase 2: Node Atomic (+ Toadstool)
log_info "🖥️  Phase 2: Deploying Node Atomic (+ Toadstool with 4070 GPU)..."

TOADSTOOL_BIN="$PRIMALS_BASE/toadstool/target/release/toadstool"
if [ -f "$TOADSTOOL_BIN" ]; then
    cd "$(dirname "$TOADSTOOL_BIN")"
    TOADSTOOL_SOCKET="$SOCKET_DIR/toadstool.sock" \
    CUDA_VISIBLE_DEVICES="0" \
    GPU_MEMORY_FRACTION="0.9" \
    FAMILY_ID="$FAMILY_ID" \
    RUST_LOG=toadstool=info \
    "$TOADSTOOL_BIN" server > /tmp/toadstool.log 2>&1 &
    TOADSTOOL_PID=$!
    log_success "Toadstool started (PID: $TOADSTOOL_PID)"
    cd - > /dev/null
    
    # Wait for socket
    for i in {1..20}; do
        if [ -S "$SOCKET_DIR/toadstool.sock" ]; then
            log_success "Toadstool socket ready"
            break
        fi
        sleep 1
    done
    
    log_success "✅ Node Atomic deployed!"
else
    log_warning "Toadstool binary not found at $TOADSTOOL_BIN"
    log_info "Skipping Node Atomic deployment"
fi

# Phase 3: Nest Atomic (+ NestGate)
log_info "💾 Phase 3: Deploying Nest Atomic (+ NestGate for model persistence)..."

NESTGATE_BIN="$PRIMALS_BASE/nestgate/target/release/nestgate"
STORAGE_PATH="/var/tmp/biomeos/nestgate/models"
mkdir -p "$STORAGE_PATH"

if [ -f "$NESTGATE_BIN" ]; then
    cd "$(dirname "$NESTGATE_BIN")"
    NESTGATE_SOCKET="$SOCKET_DIR/nestgate.sock" \
    STORAGE_PATH="$STORAGE_PATH" \
    FAMILY_ID="$FAMILY_ID" \
    RUST_LOG=nestgate=info \
    "$NESTGATE_BIN" server > /tmp/nestgate.log 2>&1 &
    NESTGATE_PID=$!
    log_success "NestGate started (PID: $NESTGATE_PID)"
    cd - > /dev/null
    
    # Wait for socket
    for i in {1..15}; do
        if [ -S "$SOCKET_DIR/nestgate.sock" ]; then
            log_success "NestGate socket ready"
            break
        fi
        sleep 1
    done
    
    log_success "✅ Nest Atomic deployed!"
else
    log_warning "NestGate binary not found at $NESTGATE_BIN"
    log_info "Skipping Nest Atomic deployment"
fi

# Phase 4: Squirrel AI
log_info "🐿️  Phase 4: Deploying Squirrel AI (multi-provider coordinator)..."

SQUIRREL_BIN="$PRIMALS_BASE/squirrel/target/release/squirrel"
if [ -f "$SQUIRREL_BIN" ]; then
    cd "$(dirname "$SQUIRREL_BIN")"
    SQUIRREL_SOCKET="$SOCKET_DIR/squirrel.sock" \
    ANTHROPIC_API_KEY_FILE="$API_KEYS" \
    OPENAI_API_KEY_FILE="$API_KEYS" \
    HUGGINGFACE_TOKEN_FILE="$API_KEYS" \
    LOCAL_MODEL_PROVIDER="toadstool" \
    MODEL_CACHE_PROVIDER="nestgate" \
    FAMILY_ID="$FAMILY_ID" \
    RUST_LOG=squirrel=info \
    "$SQUIRREL_BIN" server > /tmp/squirrel.log 2>&1 &
    SQUIRREL_PID=$!
    log_success "Squirrel started (PID: $SQUIRREL_PID)"
    cd - > /dev/null
    
    # Wait for socket
    for i in {1..15}; do
        if [ -S "$SOCKET_DIR/squirrel.sock" ]; then
            log_success "Squirrel socket ready"
            break
        fi
        sleep 1
    done
    
    log_success "✅ Squirrel AI deployed!"
else
    log_warning "Squirrel binary not found at $SQUIRREL_BIN"
fi

# Give everything time to initialize
sleep 3

# Health checks
echo ""
log_info "🏥 Running Health Checks..."

for primal in beardog songbird toadstool nestgate squirrel; do
    SOCK="$SOCKET_DIR/$primal.sock"
    if [ -S "$SOCK" ]; then
        RESPONSE=$(echo '{"jsonrpc":"2.0","method":"health","id":1}' | nc -U "$SOCK" -w 2 2>/dev/null || echo "{}")
        if echo "$RESPONSE" | grep -q "healthy\|result"; then
            log_success "  ✓ $primal is healthy"
        else
            log_warning "  ! $primal health check failed"
        fi
    else
        log_warning "  ! $primal socket not found"
    fi
done

# GPU Check
if [ -S "$SOCKET_DIR/toadstool.sock" ]; then
    echo ""
    log_info "🎮 Checking GPU (RTX 4070)..."
    GPU_RESPONSE=$(echo '{"jsonrpc":"2.0","method":"gpu.query_status","id":2}' | nc -U "$SOCKET_DIR/toadstool.sock" -w 3 2>/dev/null || echo "{}")
    if echo "$GPU_RESPONSE" | grep -q "4070\|12282"; then
        log_success "  ✓ RTX 4070 detected"
    else
        log_warning "  ! GPU detection result: $GPU_RESPONSE"
    fi
fi

# Summary
echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║              NUCLEUS Deployment Complete                     ║"
echo "╠══════════════════════════════════════════════════════════════╣"
echo "║  Atomics Deployed:                                           ║"
echo "║    ✓ Tower (BearDog + Songbird)                              ║"
[ -S "$SOCKET_DIR/toadstool.sock" ] && echo "║    ✓ Node (Tower + Toadstool + 4070 GPU)                    ║"
[ -S "$SOCKET_DIR/nestgate.sock" ] && echo "║    ✓ Nest (Tower + NestGate)                                ║"
[ -S "$SOCKET_DIR/squirrel.sock" ] && echo "║    ✓ Squirrel AI (Multi-provider)                           ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "📁 Sockets:"
ls -la "$SOCKET_DIR"/*.sock 2>/dev/null | awk '{print "   ", $9}' || echo "   (none found)"
echo ""

echo "📊 Running Processes:"
ps aux | grep -E "beardog.*server|songbird.*server|nestgate.*server|toadstool.*server|squirrel.*server" | grep -v grep | awk '{print "   PID", $2, "-", $11, $12, $13}' || echo "   (none)"
echo ""

echo "🧪 Test Commands:"
echo ""
echo "  # Test Anthropic AI"
echo "  echo '{\"jsonrpc\":\"2.0\",\"method\":\"ai.query\",\"params\":{\"provider\":\"anthropic\",\"prompt\":\"Hello!\"},\"id\":1}' | nc -U $SOCKET_DIR/squirrel.sock"
echo ""
echo "  # Test local AI (4070)"
echo "  echo '{\"jsonrpc\":\"2.0\",\"method\":\"ai.local\",\"params\":{\"prompt\":\"What is 2+2?\"},\"id\":2}' | nc -U $SOCKET_DIR/squirrel.sock"
echo ""
echo "  # Check GPU status"
echo "  echo '{\"jsonrpc\":\"2.0\",\"method\":\"gpu.query_status\",\"id\":3}' | nc -U $SOCKET_DIR/toadstool.sock"
echo ""
echo "  # Test NestGate storage"
echo "  echo '{\"jsonrpc\":\"2.0\",\"method\":\"storage.list\",\"id\":4}' | nc -U $SOCKET_DIR/nestgate.sock"
echo ""

log_info "NUCLEUS is running. Test manually or press Ctrl+C to stop."
log_info "Logs available in /tmp/{primal}.log"

# Keep running
sleep infinity
