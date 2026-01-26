#!/usr/bin/env bash
#
# Deploy Tower Atomic + Squirrel via Neural API
# Complete stack deployment for AI routing mesh
#
# Usage: ./scripts/deploy_tower_squirrel.sh [family_id]
#
# Environment Variables:
#   ANTHROPIC_API_KEY - Required for AI calls
#   PLASMID_BIN_DIR   - Path to plasmidBin (default: ./plasmidBin)
#   LOG_LEVEL         - Log level for Neural API (default: debug)

set -euo pipefail

# Configuration
FAMILY_ID="${1:-nat0}"
PLASMID_BIN_DIR="${PLASMID_BIN_DIR:-./plasmidBin}"
LOG_LEVEL="${LOG_LEVEL:-debug}"

# Socket paths
BEARDOG_SOCKET="/tmp/security-${FAMILY_ID}.sock"
SONGBIRD_SOCKET="/tmp/discovery-${FAMILY_ID}.sock"
NEURAL_API_SOCKET="/tmp/neural-api-${FAMILY_ID}.sock"
SQUIRREL_SOCKET="/tmp/ai-${FAMILY_ID}.sock"

# PID file directory
PID_DIR="/tmp/biomeos-${FAMILY_ID}"
mkdir -p "$PID_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

cleanup() {
    log_info "Cleaning up previous deployment..."
    
    # Remove old sockets
    rm -f "$BEARDOG_SOCKET" "$SONGBIRD_SOCKET" "$NEURAL_API_SOCKET" "$SQUIRREL_SOCKET"
    
    # Kill previous processes
    if [ -f "$PID_DIR/beardog.pid" ]; then
        local pid=$(cat "$PID_DIR/beardog.pid")
        if kill -0 "$pid" 2>/dev/null; then
            log_info "Stopping BearDog (PID: $pid)"
            kill "$pid" 2>/dev/null || true
        fi
        rm -f "$PID_DIR/beardog.pid"
    fi
    
    if [ -f "$PID_DIR/songbird.pid" ]; then
        local pid=$(cat "$PID_DIR/songbird.pid")
        if kill -0 "$pid" 2>/dev/null; then
            log_info "Stopping Songbird (PID: $pid)"
            kill "$pid" 2>/dev/null || true
        fi
        rm -f "$PID_DIR/songbird.pid"
    fi
    
    if [ -f "$PID_DIR/neural-api.pid" ]; then
        local pid=$(cat "$PID_DIR/neural-api.pid")
        if kill -0 "$pid" 2>/dev/null; then
            log_info "Stopping Neural API (PID: $pid)"
            kill "$pid" 2>/dev/null || true
        fi
        rm -f "$PID_DIR/neural-api.pid"
    fi
    
    if [ -f "$PID_DIR/squirrel.pid" ]; then
        local pid=$(cat "$PID_DIR/squirrel.pid")
        if kill -0 "$pid" 2>/dev/null; then
            log_info "Stopping Squirrel (PID: $pid)"
            kill "$pid" 2>/dev/null || true
        fi
        rm -f "$PID_DIR/squirrel.pid"
    fi
    
    log_success "Cleanup complete"
}

wait_for_socket() {
    local socket_path=$1
    local service_name=$2
    local max_wait=10
    local count=0
    
    log_info "Waiting for $service_name socket: $socket_path"
    
    while [ ! -S "$socket_path" ] && [ $count -lt $max_wait ]; do
        sleep 1
        count=$((count + 1))
        echo -n "."
    done
    echo ""
    
    if [ -S "$socket_path" ]; then
        log_success "$service_name socket ready!"
        return 0
    else
        log_error "$service_name socket not found after ${max_wait}s"
        return 1
    fi
}

start_beardog() {
    log_info "Starting BearDog..."
    
    # Try multiple possible binary locations (universal discovery)
    local beardog_bin=""
    for path in \
        "$PLASMID_BIN_DIR/primals/beardog/beardog-x86_64-musl" \
        "$PLASMID_BIN_DIR/primals/beardog/beardog-x86_64" \
        "$PLASMID_BIN_DIR/beardog_x86_64_linux_musl/beardog" \
        "./target/release/beardog" \
        "./target/debug/beardog"; do
        if [ -f "$path" ]; then
            beardog_bin="$path"
            break
        fi
    done
    
    if [ -z "$beardog_bin" ]; then
        log_error "BearDog binary not found in any expected location"
        return 1
    fi
    
    log_info "Using BearDog binary: $beardog_bin"
    
    "$beardog_bin" server \
        --socket "$BEARDOG_SOCKET" \
        --family-id "$FAMILY_ID" \
        > "$PID_DIR/beardog.log" 2>&1 &
    
    local pid=$!
    echo "$pid" > "$PID_DIR/beardog.pid"
    
    log_info "BearDog started (PID: $pid)"
    
    wait_for_socket "$BEARDOG_SOCKET" "BearDog"
}

start_songbird() {
    log_info "Starting Songbird..."
    
    # Try multiple possible binary locations (universal discovery)
    local songbird_bin=""
    for path in \
        "$PLASMID_BIN_DIR/primals/songbird/songbird-x86_64-musl" \
        "$PLASMID_BIN_DIR/primals/songbird/songbird-x86_64" \
        "$PLASMID_BIN_DIR/primals/songbird" \
        "$PLASMID_BIN_DIR/songbird_x86_64_linux/songbird" \
        "./target/release/songbird" \
        "./target/debug/songbird"; do
        if [ -f "$path" ]; then
            songbird_bin="$path"
            break
        fi
    done
    
    if [ -z "$songbird_bin" ]; then
        log_error "Songbird binary not found in any expected location"
        return 1
    fi
    
    log_info "Using Songbird binary: $songbird_bin"
    
    SONGBIRD_ORCHESTRATOR_SOCKET="$SONGBIRD_SOCKET" \
    SONGBIRD_ORCHESTRATOR_FAMILY_ID="$FAMILY_ID" \
    "$songbird_bin" server \
        > "$PID_DIR/songbird.log" 2>&1 &
    
    local pid=$!
    echo "$pid" > "$PID_DIR/songbird.pid"
    
    log_info "Songbird started (PID: $pid)"
    
    wait_for_socket "$SONGBIRD_SOCKET" "Songbird"
}

start_neural_api() {
    log_info "Starting Neural API..."
    
    local biomeos_bin="./target/release/biomeos"
    
    if [ ! -f "$biomeos_bin" ]; then
        log_warning "biomeOS binary not found at $biomeos_bin, trying debug build..."
        biomeos_bin="./target/debug/biomeos"
        
        if [ ! -f "$biomeos_bin" ]; then
            log_error "biomeOS binary not found"
            return 1
        fi
    fi
    
    "$biomeos_bin" neural-api \
        --graphs-dir graphs \
        --log-level "$LOG_LEVEL" \
        > "$PID_DIR/neural-api.log" 2>&1 &
    
    local pid=$!
    echo "$pid" > "$PID_DIR/neural-api.pid"
    
    log_info "Neural API started (PID: $pid)"
    
    # Neural API socket is created dynamically, wait a bit
    sleep 2
    
    # Try to find the neural API socket
    if ls /tmp/neural-api-*.sock 1> /dev/null 2>&1; then
        NEURAL_API_SOCKET=$(ls /tmp/neural-api-*.sock | head -n 1)
        log_success "Neural API socket found: $NEURAL_API_SOCKET"
    else
        log_warning "Neural API socket not found yet (may take longer to start)"
    fi
}

start_squirrel() {
    log_info "Starting Squirrel..."
    
    # Try multiple possible binary locations (universal discovery)
    local squirrel_bin=""
    for path in \
        "$PLASMID_BIN_DIR/primals/squirrel/squirrel-x86_64-musl" \
        "$PLASMID_BIN_DIR/primals/squirrel/squirrel-x86_64" \
        "$PLASMID_BIN_DIR/primals/squirrel" \
        "$PLASMID_BIN_DIR/squirrel_x86_64_linux/squirrel" \
        "./target/release/squirrel" \
        "./target/debug/squirrel"; do
        if [ -f "$path" ]; then
            squirrel_bin="$path"
            break
        fi
    done
    
    if [ -z "$squirrel_bin" ]; then
        log_error "Squirrel binary not found in any expected location"
        return 1
    fi
    
    log_info "Using Squirrel binary: $squirrel_bin"
    
    # Check if ANTHROPIC_API_KEY is set
    if [ -z "${ANTHROPIC_API_KEY:-}" ]; then
        log_warning "ANTHROPIC_API_KEY not set - AI calls will fail"
        log_info "  Set it with: export ANTHROPIC_API_KEY=sk-ant-xxxxx"
    fi
    
    AI_CAPABILITY_SOCKET="$NEURAL_API_SOCKET" \
    HTTP_CAPABILITY_SOCKET="$NEURAL_API_SOCKET" \
    ANTHROPIC_API_KEY="${ANTHROPIC_API_KEY:-}" \
    "$squirrel_bin" server \
        --socket "$SQUIRREL_SOCKET" \
        > "$PID_DIR/squirrel.log" 2>&1 &
    
    local pid=$!
    echo "$pid" > "$PID_DIR/squirrel.pid"
    
    log_info "Squirrel started (PID: $pid)"
    
    wait_for_socket "$SQUIRREL_SOCKET" "Squirrel"
}

show_status() {
    echo ""
    echo "=========================================="
    echo "Deployment Status"
    echo "=========================================="
    echo ""
    
    echo "Service Sockets:"
    ls -lh "$BEARDOG_SOCKET" "$SONGBIRD_SOCKET" "$NEURAL_API_SOCKET" "$SQUIRREL_SOCKET" 2>/dev/null || log_warning "Some sockets not found"
    echo ""
    
    echo "Service PIDs:"
    if [ -f "$PID_DIR/beardog.pid" ]; then
        echo "  BearDog:    $(cat $PID_DIR/beardog.pid)"
    fi
    if [ -f "$PID_DIR/songbird.pid" ]; then
        echo "  Songbird:   $(cat $PID_DIR/songbird.pid)"
    fi
    if [ -f "$PID_DIR/neural-api.pid" ]; then
        echo "  Neural API: $(cat $PID_DIR/neural-api.pid)"
    fi
    if [ -f "$PID_DIR/squirrel.pid" ]; then
        echo "  Squirrel:   $(cat $PID_DIR/squirrel.pid)"
    fi
    echo ""
    
    echo "Logs:"
    echo "  BearDog:    tail -f $PID_DIR/beardog.log"
    echo "  Songbird:   tail -f $PID_DIR/songbird.log"
    echo "  Neural API: tail -f $PID_DIR/neural-api.log"
    echo "  Squirrel:   tail -f $PID_DIR/squirrel.log"
    echo ""
    
    echo "Test the deployment:"
    echo "  ./scripts/test_neural_api_routing.sh $FAMILY_ID"
    echo ""
    
    echo "Stop the deployment:"
    echo "  ./scripts/stop_tower_squirrel.sh $FAMILY_ID"
    echo ""
}

# Main deployment
main() {
    echo "=========================================="
    echo "Tower Atomic + Squirrel Deployment"
    echo "Family ID: $FAMILY_ID"
    echo "=========================================="
    echo ""
    
    # Cleanup previous deployment
    cleanup
    
    # Start services in order
    start_beardog || {
        log_error "BearDog failed to start"
        exit 1
    }
    
    start_songbird || {
        log_error "Songbird failed to start"
        exit 1
    }
    
    start_neural_api || {
        log_error "Neural API failed to start"
        exit 1
    }
    
    start_squirrel || {
        log_error "Squirrel failed to start"
        exit 1
    }
    
    # Show status
    show_status
    
    log_success "Tower Atomic + Squirrel deployment COMPLETE!"
    echo ""
    echo "🎉 Neural API Routing Mesh is READY!"
}

# Run main
main "$@"

