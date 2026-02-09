#!/bin/bash
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
#     🏗️  Tower Atomic Bootstrap
#     biomeOS Neural API Orchestration
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
#
# Usage: ./scripts/bootstrap_tower_atomic.sh [--stop]
#
# Environment:
#   FAMILY_ID    - Family identifier (default: 1894e909e454)
#   NODE_ID      - Node identifier (default: tower0)
#   BIOMEOS_ROOT - biomeOS root directory (default: script dir parent)

set -e

# Configuration
FAMILY_ID="${FAMILY_ID:-1894e909e454}"
NODE_ID="${NODE_ID:-tower0}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="${BIOMEOS_ROOT:-$(dirname "$SCRIPT_DIR")}"
XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
SOCKET_DIR="${XDG_RUNTIME_DIR}/biomeos"
PLASMID_BIN="${BIOMEOS_ROOT}/plasmidBin"

# Socket paths
BEARDOG_SOCKET="${SOCKET_DIR}/beardog-${FAMILY_ID}.sock"
SONGBIRD_SOCKET="${SOCKET_DIR}/songbird-${FAMILY_ID}.sock"
NEURAL_API_SOCKET="${SOCKET_DIR}/neural-api-${FAMILY_ID}.sock"

# PID files
PID_DIR="${SOCKET_DIR}/pids"
BEARDOG_PID="${PID_DIR}/beardog.pid"
SONGBIRD_PID="${PID_DIR}/songbird.pid"
NEURAL_API_PID="${PID_DIR}/neural-api.pid"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warn() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Cleanup function
cleanup() {
    log_info "Cleaning up..."
    
    if [ -f "$NEURAL_API_PID" ]; then
        kill "$(cat "$NEURAL_API_PID")" 2>/dev/null || true
        rm -f "$NEURAL_API_PID"
    fi
    
    if [ -f "$SONGBIRD_PID" ]; then
        kill "$(cat "$SONGBIRD_PID")" 2>/dev/null || true
        rm -f "$SONGBIRD_PID"
    fi
    
    if [ -f "$BEARDOG_PID" ]; then
        kill "$(cat "$BEARDOG_PID")" 2>/dev/null || true
        rm -f "$BEARDOG_PID"
    fi
    
    # Clean up sockets
    rm -f "$BEARDOG_SOCKET" "$SONGBIRD_SOCKET" "$NEURAL_API_SOCKET" 2>/dev/null || true
    
    log_success "Cleanup complete"
}

# Stop all processes
stop_all() {
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "     🛑 Stopping Tower Atomic"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cleanup
    exit 0
}

# Wait for socket to be available
wait_for_socket() {
    local socket=$1
    local timeout=$2
    local elapsed=0
    
    while [ ! -S "$socket" ] && [ $elapsed -lt $timeout ]; do
        sleep 0.5
        elapsed=$((elapsed + 1))
    done
    
    if [ -S "$socket" ]; then
        return 0
    else
        return 1
    fi
}

# JSON-RPC call
json_rpc() {
    local socket=$1
    local method=$2
    local params=$3
    
    echo "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":$params,\"id\":1}" \
        | timeout 5 nc -U "$socket" 2>/dev/null || echo '{"error":"timeout"}'
}

# Start BearDog
start_beardog() {
    log_info "Starting BearDog..."
    
    local binary="${PLASMID_BIN}/beardog"
    if [ ! -x "$binary" ]; then
        log_error "BearDog binary not found: $binary"
        return 1
    fi
    
    # Clean up old socket
    rm -f "$BEARDOG_SOCKET" 2>/dev/null || true
    
    FAMILY_ID="$FAMILY_ID" \
    NODE_ID="$NODE_ID" \
    XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR" \
    "$binary" server --socket "$BEARDOG_SOCKET" &
    
    local pid=$!
    echo "$pid" > "$BEARDOG_PID"
    
    if wait_for_socket "$BEARDOG_SOCKET" 20; then
        log_success "BearDog ready at $BEARDOG_SOCKET (PID: $pid)"
        return 0
    else
        log_error "BearDog failed to start"
        kill "$pid" 2>/dev/null || true
        return 1
    fi
}

# Start Songbird
start_songbird() {
    log_info "Starting Songbird..."
    
    local binary="${PLASMID_BIN}/songbird"
    if [ ! -x "$binary" ]; then
        log_error "Songbird binary not found: $binary"
        return 1
    fi
    
    # Clean up old socket
    rm -f "$SONGBIRD_SOCKET" 2>/dev/null || true
    
    FAMILY_ID="$FAMILY_ID" \
    NODE_ID="$NODE_ID" \
    XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR" \
    BEARDOG_SOCKET="$BEARDOG_SOCKET" \
    BEARDOG_MODE="direct" \
    SONGBIRD_SECURITY_PROVIDER="beardog" \
    "$binary" server \
        --socket "$SONGBIRD_SOCKET" \
        --beardog-socket "$BEARDOG_SOCKET" \
        --federation-port 8080 &
    
    local pid=$!
    echo "$pid" > "$SONGBIRD_PID"
    
    if wait_for_socket "$SONGBIRD_SOCKET" 30; then
        log_success "Songbird ready at $SONGBIRD_SOCKET (PID: $pid)"
        return 0
    else
        log_error "Songbird failed to start"
        kill "$pid" 2>/dev/null || true
        return 1
    fi
}

# Start Neural API
start_neural_api() {
    log_info "Starting Neural API..."
    
    local binary="${PLASMID_BIN}/neural-api-server"
    if [ ! -x "$binary" ]; then
        # Try alternative
        binary="${BIOMEOS_ROOT}/target/release/neural-api-server"
        if [ ! -x "$binary" ]; then
            log_error "Neural API binary not found"
            return 1
        fi
    fi
    
    # Clean up old socket
    rm -f "$NEURAL_API_SOCKET" 2>/dev/null || true
    
    FAMILY_ID="$FAMILY_ID" \
    NODE_ID="$NODE_ID" \
    XDG_RUNTIME_DIR="$XDG_RUNTIME_DIR" \
    "$binary" \
        --graphs-dir "${BIOMEOS_ROOT}/graphs" \
        --socket "$NEURAL_API_SOCKET" &
    
    local pid=$!
    echo "$pid" > "$NEURAL_API_PID"
    
    if wait_for_socket "$NEURAL_API_SOCKET" 20; then
        log_success "Neural API ready at $NEURAL_API_SOCKET (PID: $pid)"
        return 0
    else
        log_error "Neural API failed to start"
        kill "$pid" 2>/dev/null || true
        return 1
    fi
}

# Verify the stack
verify_stack() {
    echo ""
    log_info "Verifying Tower Atomic stack..."
    echo ""
    
    # Test BearDog
    printf "  BearDog crypto.sha256... "
    local response
    response=$(json_rpc "$BEARDOG_SOCKET" "crypto.sha256" '{"message":"dGVzdA=="}')
    if echo "$response" | grep -q "result"; then
        echo -e "${GREEN}✅${NC}"
    else
        echo -e "${RED}❌${NC}"
    fi
    
    # Test Neural API
    printf "  Neural API capability.list... "
    response=$(json_rpc "$NEURAL_API_SOCKET" "capability.list" '{}')
    if echo "$response" | grep -q "result"; then
        echo -e "${GREEN}✅${NC}"
    else
        echo -e "${RED}❌${NC}"
    fi
    
    # Test capability routing
    printf "  Neural API capability.call... "
    response=$(json_rpc "$NEURAL_API_SOCKET" "capability.call" \
        '{"capability":"crypto","operation":"sha256","args":{"message":"dGVzdA=="}}')
    if echo "$response" | grep -q "result"; then
        echo -e "${GREEN}✅${NC}"
    else
        echo -e "${RED}❌${NC}"
    fi
    
    echo ""
    log_success "Tower Atomic stack is operational!"
    echo ""
}

# Main
main() {
    # Handle --stop flag
    if [ "$1" = "--stop" ]; then
        stop_all
    fi
    
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "     🏗️  Tower Atomic Bootstrap"
    echo "     biomeOS Neural API Orchestration"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    echo "📋 Configuration:"
    echo "   Family ID: $FAMILY_ID"
    echo "   Node ID: $NODE_ID"
    echo "   biomeOS Root: $BIOMEOS_ROOT"
    echo "   Socket Dir: $SOCKET_DIR"
    echo ""
    
    # Create directories
    mkdir -p "$SOCKET_DIR"
    mkdir -p "$PID_DIR"
    
    # Set up trap for cleanup
    trap cleanup EXIT INT TERM
    
    # Start primals
    if ! start_beardog; then
        log_error "Failed to start BearDog"
        exit 1
    fi
    
    if ! start_songbird; then
        log_error "Failed to start Songbird"
        exit 1
    fi
    
    if ! start_neural_api; then
        log_error "Failed to start Neural API"
        exit 1
    fi
    
    # Verify
    verify_stack
    
    # Print status
    echo "📍 Socket Paths:"
    echo "   BearDog:    $BEARDOG_SOCKET"
    echo "   Songbird:   $SONGBIRD_SOCKET"
    echo "   Neural API: $NEURAL_API_SOCKET"
    echo ""
    echo "🔗 Quick Test Commands:"
    echo "   echo '{\"jsonrpc\":\"2.0\",\"method\":\"capability.list\",\"params\":{},\"id\":1}' | nc -U $NEURAL_API_SOCKET"
    echo ""
    echo "Press Ctrl+C to stop all primals..."
    echo ""
    
    # Wait
    wait
}

main "$@"

