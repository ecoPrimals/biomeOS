#!/usr/bin/env bash
#
# Tower Atomic Bootstrap Script
# 
# Deploys the foundational security layer: BearDog + Songbird + Neural API
# with proper XDG socket paths and genetic lineage.
#
# Usage:
#   ./scripts/tower_atomic_bootstrap.sh [--family-id ID] [--node-id ID]
#
# Environment variables:
#   FAMILY_ID        - Family identifier (default: nat0)
#   NODE_ID          - Node identifier (default: tower0)
#   XDG_RUNTIME_DIR  - Runtime directory (default: /run/user/$(id -u))
#   BIOMEOS_DIR      - biomeOS root directory (default: script location)
#
# Author: biomeOS Team
# Date: January 28, 2026

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $*"; }
log_success() { echo -e "${GREEN}[✓]${NC} $*"; }
log_warn() { echo -e "${YELLOW}[⚠]${NC} $*"; }
log_error() { echo -e "${RED}[✗]${NC} $*" >&2; }

# Determine script directory (biomeOS root)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_DIR="${BIOMEOS_DIR:-$(dirname "$SCRIPT_DIR")}"

# Default configuration
FAMILY_ID="${FAMILY_ID:-nat0}"
NODE_ID="${NODE_ID:-tower0}"
XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
BIOMEOS_RUNTIME_DIR="${XDG_RUNTIME_DIR}/biomeos"

# Primal binaries location
PLASMID_BIN="${BIOMEOS_DIR}/plasmidBin"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --family-id)
            FAMILY_ID="$2"
            shift 2
            ;;
        --node-id)
            NODE_ID="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [--family-id ID] [--node-id ID]"
            echo ""
            echo "Options:"
            echo "  --family-id ID   Set family identifier (default: nat0)"
            echo "  --node-id ID     Set node identifier (default: tower0)"
            echo ""
            echo "Environment:"
            echo "  FAMILY_ID        Family identifier"
            echo "  NODE_ID          Node identifier"
            echo "  XDG_RUNTIME_DIR  Runtime directory for sockets"
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Display banner
echo ""
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║            🏗️  TOWER ATOMIC BOOTSTRAP                      ║"
echo "║                                                           ║"
echo "║  Deploying: BearDog + Songbird + Neural API              ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

log_info "Configuration:"
log_info "  Family ID:     $FAMILY_ID"
log_info "  Node ID:       $NODE_ID"
log_info "  Runtime Dir:   $BIOMEOS_RUNTIME_DIR"
log_info "  Primal Bins:   $PLASMID_BIN"
echo ""

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check binaries exist
    local missing=()
    for bin in beardog songbird; do
        if [[ ! -x "$PLASMID_BIN/$bin" ]]; then
            missing+=("$bin")
        fi
    done
    
    if [[ ${#missing[@]} -gt 0 ]]; then
        log_error "Missing binaries: ${missing[*]}"
        log_error "Run 'cargo build --release' in the primal directories first"
        exit 1
    fi
    
    # Check Neural API binary
    local neural_api_bin="$BIOMEOS_DIR/target/release/neural-api-server"
    if [[ ! -x "$neural_api_bin" ]]; then
        log_warn "Neural API binary not found at $neural_api_bin"
        log_info "Building Neural API..."
        (cd "$BIOMEOS_DIR" && cargo build --release --package biomeos-atomic-deploy --bin neural-api-server) || {
            log_error "Failed to build Neural API"
            exit 1
        }
    fi
    
    log_success "Prerequisites satisfied"
}

# Create runtime directory
setup_runtime_dir() {
    log_info "Setting up runtime directory..."
    
    mkdir -p "$BIOMEOS_RUNTIME_DIR"
    
    log_success "Runtime directory ready: $BIOMEOS_RUNTIME_DIR"
}

# Stop any existing primals
cleanup_existing() {
    log_info "Cleaning up existing processes..."
    
    # Kill existing primals by socket
    for socket in "$BIOMEOS_RUNTIME_DIR"/*.sock; do
        if [[ -e "$socket" ]]; then
            rm -f "$socket"
            log_info "  Removed stale socket: $(basename "$socket")"
        fi
    done
    
    # Kill processes by name (gentle)
    for primal in beardog songbird neural-api-server; do
        if pkill -0 "$primal" 2>/dev/null; then
            log_info "  Stopping existing $primal..."
            pkill "$primal" || true
            sleep 1
        fi
    done
    
    log_success "Cleanup complete"
}

# Start BearDog (crypto/identity foundation)
start_beardog() {
    log_info "Starting BearDog..."
    
    local socket="$BIOMEOS_RUNTIME_DIR/beardog-${FAMILY_ID}.sock"
    
    FAMILY_ID="$FAMILY_ID" \
    NODE_ID="$NODE_ID" \
    BEARDOG_SOCKET="$socket" \
    "$PLASMID_BIN/beardog" \
        --socket "$socket" \
        --family-id "$FAMILY_ID" \
        --node-id "$NODE_ID" \
        > "$BIOMEOS_RUNTIME_DIR/beardog.log" 2>&1 &
    
    local pid=$!
    
    # Wait for socket to appear
    local timeout=10
    while [[ ! -e "$socket" && $timeout -gt 0 ]]; do
        sleep 1
        ((timeout--))
    done
    
    if [[ -e "$socket" ]]; then
        log_success "BearDog started (PID: $pid, Socket: $socket)"
        echo "$pid" > "$BIOMEOS_RUNTIME_DIR/beardog.pid"
    else
        log_error "BearDog failed to start (check $BIOMEOS_RUNTIME_DIR/beardog.log)"
        exit 1
    fi
}

# Start Neural API
start_neural_api() {
    log_info "Starting Neural API..."
    
    local socket="$BIOMEOS_RUNTIME_DIR/neural-api-${FAMILY_ID}.sock"
    local neural_api_bin="$BIOMEOS_DIR/target/release/neural-api-server"
    
    FAMILY_ID="$FAMILY_ID" \
    NODE_ID="$NODE_ID" \
    BEARDOG_SOCKET="$BIOMEOS_RUNTIME_DIR/beardog-${FAMILY_ID}.sock" \
    "$neural_api_bin" \
        --socket "$socket" \
        --graphs-dir "$BIOMEOS_DIR/graphs" \
        --family-id "$FAMILY_ID" \
        > "$BIOMEOS_RUNTIME_DIR/neural-api.log" 2>&1 &
    
    local pid=$!
    
    # Wait for socket to appear
    local timeout=10
    while [[ ! -e "$socket" && $timeout -gt 0 ]]; do
        sleep 1
        ((timeout--))
    done
    
    if [[ -e "$socket" ]]; then
        log_success "Neural API started (PID: $pid, Socket: $socket)"
        echo "$pid" > "$BIOMEOS_RUNTIME_DIR/neural-api.pid"
    else
        log_error "Neural API failed to start (check $BIOMEOS_RUNTIME_DIR/neural-api.log)"
        exit 1
    fi
}

# Start Songbird (HTTP/TLS layer)
start_songbird() {
    log_info "Starting Songbird..."
    
    local socket="$BIOMEOS_RUNTIME_DIR/songbird-${FAMILY_ID}.sock"
    local beardog_socket="$BIOMEOS_RUNTIME_DIR/beardog-${FAMILY_ID}.sock"
    local neural_api_socket="$BIOMEOS_RUNTIME_DIR/neural-api-${FAMILY_ID}.sock"
    
    FAMILY_ID="$FAMILY_ID" \
    NODE_ID="${FAMILY_ID}-${NODE_ID}" \
    BEARDOG_SOCKET="$beardog_socket" \
    NEURAL_API_SOCKET="$neural_api_socket" \
    BEARDOG_MODE="neural" \
    SONGBIRD_SECURITY_PROVIDER="beardog" \
    "$PLASMID_BIN/songbird" server \
        --socket "$socket" \
        --beardog-socket "$beardog_socket" \
        --federation-port 8080 \
        > "$BIOMEOS_RUNTIME_DIR/songbird.log" 2>&1 &
    
    local pid=$!
    
    # Wait for socket to appear
    local timeout=15
    while [[ ! -e "$socket" && $timeout -gt 0 ]]; do
        sleep 1
        ((timeout--))
    done
    
    if [[ -e "$socket" ]]; then
        log_success "Songbird started (PID: $pid, Socket: $socket)"
        echo "$pid" > "$BIOMEOS_RUNTIME_DIR/songbird.pid"
    else
        log_error "Songbird failed to start (check $BIOMEOS_RUNTIME_DIR/songbird.log)"
        cat "$BIOMEOS_RUNTIME_DIR/songbird.log" | tail -20
        exit 1
    fi
}

# Verify deployment
verify_deployment() {
    log_info "Verifying deployment..."
    
    local all_ok=true
    
    # Check BearDog
    if echo '{"jsonrpc":"2.0","method":"crypto.sha256","params":{"message":"dGVzdA=="},"id":1}' \
        | nc -U "$BIOMEOS_RUNTIME_DIR/beardog-${FAMILY_ID}.sock" 2>/dev/null | grep -q "result"; then
        log_success "BearDog: crypto.sha256 ✓"
    else
        log_error "BearDog: crypto.sha256 ✗"
        all_ok=false
    fi
    
    # Check Neural API
    if echo '{"jsonrpc":"2.0","method":"capability.list","params":{},"id":1}' \
        | nc -U "$BIOMEOS_RUNTIME_DIR/neural-api-${FAMILY_ID}.sock" 2>/dev/null | grep -q "result"; then
        log_success "Neural API: capability.list ✓"
    else
        log_error "Neural API: capability.list ✗"
        all_ok=false
    fi
    
    # Check Songbird (may take a moment for TLS to initialize)
    sleep 2
    if echo '{"jsonrpc":"2.0","method":"health.check","params":{},"id":1}' \
        | nc -U "$BIOMEOS_RUNTIME_DIR/songbird-${FAMILY_ID}.sock" 2>/dev/null | grep -q "result\|error"; then
        log_success "Songbird: health.check ✓"
    else
        log_warn "Songbird: health.check (may still be initializing)"
    fi
    
    if $all_ok; then
        echo ""
        log_success "Tower Atomic deployment SUCCESSFUL!"
    else
        echo ""
        log_error "Tower Atomic deployment INCOMPLETE"
        exit 1
    fi
}

# Display status
show_status() {
    echo ""
    echo "╔═══════════════════════════════════════════════════════════╗"
    echo "║                  🏗️  TOWER ATOMIC STATUS                   ║"
    echo "╚═══════════════════════════════════════════════════════════╝"
    echo ""
    echo "Sockets:"
    ls -la "$BIOMEOS_RUNTIME_DIR"/*.sock 2>/dev/null || echo "  (no sockets)"
    echo ""
    echo "PIDs:"
    for pid_file in "$BIOMEOS_RUNTIME_DIR"/*.pid; do
        if [[ -f "$pid_file" ]]; then
            local name=$(basename "$pid_file" .pid)
            local pid=$(cat "$pid_file")
            if kill -0 "$pid" 2>/dev/null; then
                echo "  $name: $pid (running)"
            else
                echo "  $name: $pid (not running)"
            fi
        fi
    done
    echo ""
    echo "Logs:"
    echo "  BearDog:    $BIOMEOS_RUNTIME_DIR/beardog.log"
    echo "  Neural API: $BIOMEOS_RUNTIME_DIR/neural-api.log"
    echo "  Songbird:   $BIOMEOS_RUNTIME_DIR/songbird.log"
    echo ""
    echo "Quick test:"
    echo "  echo '{\"jsonrpc\":\"2.0\",\"method\":\"capability.list\",\"id\":1}' | nc -U $BIOMEOS_RUNTIME_DIR/neural-api-${FAMILY_ID}.sock"
}

# Main execution
main() {
    check_prerequisites
    setup_runtime_dir
    cleanup_existing
    
    echo ""
    log_info "Starting Tower Atomic stack..."
    echo ""
    
    start_beardog
    start_neural_api
    start_songbird
    
    echo ""
    verify_deployment
    show_status
}

# Run main
main "$@"

