#!/bin/bash
# Launch biomeOS Stack with Dynamic Family Discovery
#
# This script reads family ID from .family.seed file or environment
# and starts the Tower Atomic stack with proper identity.
#
# Usage:
#   ./scripts/launch_with_family.sh [seed_path]
#   ./scripts/launch_with_family.sh /media/user/USB/biomeOS/.family.seed
#
# Environment Variables:
#   FAMILY_ID - Override family ID (highest priority)
#   NODE_ID   - Node identifier (default: hostname)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$(dirname "$SCRIPT_DIR")"
SEED_PATH="${1:-$BIOMEOS_ROOT/.family.seed}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}     biomeOS Dynamic Family Launch                          ${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
echo ""

# =============================================================================
# Family ID Discovery
# =============================================================================
discover_family_id() {
    # Priority 1: FAMILY_ID env var (explicit override)
    if [ -n "$FAMILY_ID" ] && [ "$FAMILY_ID" != "1894e909e454" ]; then
        echo -e "${GREEN}✅ Using FAMILY_ID from environment: $FAMILY_ID${NC}"
        return 0
    fi
    
    # Priority 2: Read from .family.seed file
    if [ -f "$SEED_PATH" ]; then
        # Family ID = hex of first 8 bytes
        FAMILY_ID=$(xxd -p "$SEED_PATH" | head -c 16)
        if [ -n "$FAMILY_ID" ]; then
            echo -e "${GREEN}✅ Family ID from seed file: $FAMILY_ID${NC}"
            export FAMILY_ID
            return 0
        fi
    fi
    
    # Priority 3: Search common USB mount points
    for mount_point in /media/$USER/*/biomeOS/.family.seed /media/$USER/*/*/biomeOS/.family.seed; do
        if [ -f "$mount_point" ]; then
            FAMILY_ID=$(xxd -p "$mount_point" | head -c 16)
            if [ -n "$FAMILY_ID" ]; then
                echo -e "${GREEN}✅ Family ID from USB: $mount_point${NC}"
                echo -e "   Family: ${YELLOW}$FAMILY_ID${NC}"
                export FAMILY_ID
                export SEED_PATH="$mount_point"
                return 0
            fi
        fi
    done
    
    # Warn about 1894e909e454 deprecation
    if [ "$FAMILY_ID" = "1894e909e454" ]; then
        echo -e "${YELLOW}⚠️ WARNING: 1894e909e454 is deprecated prototype family ID${NC}"
        echo -e "${YELLOW}   Consider using a proper .family.seed file${NC}"
        export FAMILY_ID="1894e909e454"
        return 0
    fi
    
    echo -e "${RED}❌ No family ID found!${NC}"
    echo "   Set FAMILY_ID environment variable or provide .family.seed path"
    return 1
}

# =============================================================================
# Setup Environment
# =============================================================================
setup_environment() {
    # XDG Runtime Directory
    export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"
    
    # Node ID (for federation)
    export NODE_ID="${NODE_ID:-node-$(hostname)}"
    
    # Socket directory
    SOCKET_DIR="$XDG_RUNTIME_DIR/biomeos"
    mkdir -p "$SOCKET_DIR"
    
    # Dynamic socket paths
    export BEARDOG_SOCKET="$SOCKET_DIR/beardog-$FAMILY_ID.sock"
    export SONGBIRD_SOCKET="$SOCKET_DIR/songbird-$FAMILY_ID.sock"
    export NEURAL_API_SOCKET="$SOCKET_DIR/neural-api-$FAMILY_ID.sock"
    export SQUIRREL_SOCKET="$SOCKET_DIR/squirrel-$FAMILY_ID.sock"
    export TOADSTOOL_SOCKET="$SOCKET_DIR/toadstool-$FAMILY_ID.jsonrpc.sock"
    
    # Security provider config
    export SONGBIRD_SECURITY_PROVIDER="$BEARDOG_SOCKET"
    export HTTP_REQUEST_PROVIDER_SOCKET="$SONGBIRD_SOCKET"
    
    # Logging
    export RUST_LOG="${RUST_LOG:-info}"
    
    echo ""
    echo -e "${BLUE}Environment:${NC}"
    echo "  XDG_RUNTIME_DIR: $XDG_RUNTIME_DIR"
    echo "  FAMILY_ID:       $FAMILY_ID"
    echo "  NODE_ID:         $NODE_ID"
    echo ""
    echo -e "${BLUE}Socket Paths:${NC}"
    echo "  BearDog:    $BEARDOG_SOCKET"
    echo "  Songbird:   $SONGBIRD_SOCKET"
    echo "  Neural API: $NEURAL_API_SOCKET"
}

# =============================================================================
# Start Primals
# =============================================================================
start_beardog() {
    echo -e "${YELLOW}Starting BearDog (Security Foundation)...${NC}"
    
    local BEARDOG_BIN="${BIOMEOS_ROOT}/plasmidBin/primals/beardog/beardog"
    if [ ! -f "$BEARDOG_BIN" ]; then
        echo -e "${RED}❌ BearDog binary not found at $BEARDOG_BIN${NC}"
        return 1
    fi
    
    "$BEARDOG_BIN" server --socket "$BEARDOG_SOCKET" &
    BEARDOG_PID=$!
    echo "  PID: $BEARDOG_PID"
    
    # Wait for socket
    for i in {1..30}; do
        if [ -S "$BEARDOG_SOCKET" ]; then
            echo -e "  ${GREEN}✅ BearDog socket ready${NC}"
            return 0
        fi
        sleep 0.1
    done
    
    echo -e "${RED}❌ BearDog socket timeout${NC}"
    return 1
}

start_neural_api() {
    echo -e "${YELLOW}Starting Neural API (Orchestration Layer)...${NC}"
    
    local NEURAL_API_BIN="${BIOMEOS_ROOT}/plasmidBin/neural-api-server"
    if [ ! -f "$NEURAL_API_BIN" ]; then
        echo -e "${RED}❌ Neural API binary not found at $NEURAL_API_BIN${NC}"
        return 1
    fi
    
    "$NEURAL_API_BIN" \
        --socket "$NEURAL_API_SOCKET" \
        --graphs "$BIOMEOS_ROOT/graphs" \
        --family-id "$FAMILY_ID" &
    NEURAL_API_PID=$!
    echo "  PID: $NEURAL_API_PID"
    
    # Wait for socket
    for i in {1..30}; do
        if [ -S "$NEURAL_API_SOCKET" ]; then
            echo -e "  ${GREEN}✅ Neural API socket ready${NC}"
            return 0
        fi
        sleep 0.1
    done
    
    echo -e "${RED}❌ Neural API socket timeout${NC}"
    return 1
}

start_songbird() {
    echo -e "${YELLOW}Starting Songbird (Network Layer)...${NC}"
    
    local SONGBIRD_BIN="${BIOMEOS_ROOT}/plasmidBin/primals/songbird/songbird"
    if [ ! -f "$SONGBIRD_BIN" ]; then
        echo -e "${RED}❌ Songbird binary not found at $SONGBIRD_BIN${NC}"
        return 1
    fi
    
    "$SONGBIRD_BIN" server \
        --socket "$SONGBIRD_SOCKET" \
        --beardog-socket "$BEARDOG_SOCKET" \
        --port 8080 &
    SONGBIRD_PID=$!
    echo "  PID: $SONGBIRD_PID"
    
    # Wait for socket
    for i in {1..30}; do
        if [ -S "$SONGBIRD_SOCKET" ]; then
            echo -e "  ${GREEN}✅ Songbird socket ready${NC}"
            return 0
        fi
        sleep 0.1
    done
    
    echo -e "${RED}❌ Songbird socket timeout${NC}"
    return 1
}

# =============================================================================
# Health Check
# =============================================================================
check_health() {
    echo ""
    echo -e "${BLUE}Health Check:${NC}"
    
    # BearDog
    if echo '{"jsonrpc":"2.0","method":"health.check","id":1}' | timeout 2 nc -U "$BEARDOG_SOCKET" &>/dev/null; then
        echo -e "  BearDog:    ${GREEN}✅${NC}"
    else
        echo -e "  BearDog:    ${RED}❌${NC}"
    fi
    
    # Neural API
    if echo '{"jsonrpc":"2.0","method":"health.check","id":1}' | timeout 2 nc -U "$NEURAL_API_SOCKET" &>/dev/null; then
        echo -e "  Neural API: ${GREEN}✅${NC}"
    else
        echo -e "  Neural API: ${RED}❌${NC}"
    fi
    
    # Songbird
    if echo '{"jsonrpc":"2.0","method":"health.check","id":1}' | timeout 2 nc -U "$SONGBIRD_SOCKET" &>/dev/null; then
        echo -e "  Songbird:   ${GREEN}✅${NC}"
    else
        echo -e "  Songbird:   ${RED}❌${NC}"
    fi
}

# =============================================================================
# Main
# =============================================================================
main() {
    # Discover family ID
    if ! discover_family_id; then
        exit 1
    fi
    
    # Setup environment with family ID
    setup_environment
    
    # Start primals in order
    echo ""
    echo -e "${BLUE}Starting Tower Atomic Stack...${NC}"
    echo ""
    
    start_beardog || exit 1
    start_neural_api || exit 1
    start_songbird || exit 1
    
    # Health check
    sleep 1
    check_health
    
    echo ""
    echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}     Tower Atomic Ready!                                    ${NC}"
    echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "Family ID: $FAMILY_ID"
    echo "Node ID:   $NODE_ID"
    echo ""
    echo "Test commands:"
    echo "  # Health check"
    echo "  echo '{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"id\":1}' | nc -U $NEURAL_API_SOCKET"
    echo ""
    echo "  # Federation verify"
    echo "  echo '{\"jsonrpc\":\"2.0\",\"method\":\"federation.verify_family_member\",\"params\":{\"family_id\":\"$FAMILY_ID\",\"node_id\":\"$NODE_ID\"},\"id\":1}' | nc -U $BEARDOG_SOCKET"
    echo ""
    
    # Keep running
    echo "Press Ctrl+C to stop all primals"
    wait
}

main "$@"

