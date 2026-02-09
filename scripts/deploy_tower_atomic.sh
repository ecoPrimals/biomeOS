#!/bin/bash
#
# Tower Atomic Deployment Script
# Deploys Neural API + BearDog + Songbird from plasmidBin
#
# This is the standard deployment for liveSpore/USB self-propagation.
# All binaries are sourced from plasmidBin/ for portability.
#
# Usage:
#   ./deploy_tower_atomic.sh [family_id] [node_id]
#
# Environment:
#   FAMILY_ID - Family identifier (default: 1894e909e454)
#   NODE_ID   - Node identifier (default: tower1)
#

set -e

# ═══════════════════════════════════════════════════════════════════════════════
# Configuration
# ═══════════════════════════════════════════════════════════════════════════════

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PLASMID_BIN="${SCRIPT_DIR}/plasmidBin"

# Family/Node identity
export FAMILY_ID="${1:-${FAMILY_ID:-1894e909e454}}"
export NODE_ID="${2:-${NODE_ID:-tower1}}"

# Socket paths
export NEURAL_API_SOCKET="/tmp/neural-api.sock"
export BEARDOG_SOCKET="/tmp/beardog-${FAMILY_ID}.sock"
export SONGBIRD_SOCKET="/tmp/songbird-${FAMILY_ID}.sock"

# For Songbird to find security provider
export SECURITY_ENDPOINT="unix://${NEURAL_API_SOCKET}"
export CAPABILITY_SECURITY_ENDPOINT="unix://${NEURAL_API_SOCKET}"
export BEARDOG_MODE="neural"

# Logging
export RUST_LOG="${RUST_LOG:-info}"

# ═══════════════════════════════════════════════════════════════════════════════
# Helper Functions
# ═══════════════════════════════════════════════════════════════════════════════

log() {
    echo "[$(date '+%H:%M:%S')] $*"
}

cleanup() {
    log "🧹 Cleaning up..."
    pkill -f "biomeos neural-api" 2>/dev/null || true
    pkill -f "beardog server" 2>/dev/null || true
    pkill -f "songbird server" 2>/dev/null || true
    rm -f /tmp/neural-api.sock /tmp/beardog-*.sock /tmp/songbird-*.sock 2>/dev/null || true
    sleep 1
}

wait_for_socket() {
    local socket="$1"
    local name="$2"
    local timeout="${3:-10}"
    
    for i in $(seq 1 $timeout); do
        if [ -S "$socket" ]; then
            return 0
        fi
        sleep 1
    done
    
    log "❌ Timeout waiting for $name socket: $socket"
    return 1
}

# ═══════════════════════════════════════════════════════════════════════════════
# Main Deployment
# ═══════════════════════════════════════════════════════════════════════════════

main() {
    log "╔══════════════════════════════════════════════════════════════════════╗"
    log "║   🏰 TOWER ATOMIC DEPLOYMENT - biomeOS                               ║"
    log "╚══════════════════════════════════════════════════════════════════════╝"
    log ""
    log "Configuration:"
    log "  Family ID: ${FAMILY_ID}"
    log "  Node ID:   ${NODE_ID}"
    log "  plasmidBin: ${PLASMID_BIN}"
    log ""

    # Cleanup previous deployment
    cleanup

    # Verify binaries exist in plasmidBin
    local BIOMEOS_BIN="${SCRIPT_DIR}/target/release/biomeos"
    local BEARDOG_BIN="${PLASMID_BIN}/primals/beardog/beardog"
    local SONGBIRD_BIN="${PLASMID_BIN}/primals/songbird/songbird"

    if [ ! -x "$BIOMEOS_BIN" ]; then
        log "❌ biomeOS not found at: $BIOMEOS_BIN"
        log "   Run: cargo build --release -p biomeos-unibin"
        exit 1
    fi

    if [ ! -x "$BEARDOG_BIN" ]; then
        log "❌ BearDog not found at: $BEARDOG_BIN"
        log "   Harvest from phase1/beardog/target/release/beardog"
        exit 1
    fi

    if [ ! -x "$SONGBIRD_BIN" ]; then
        log "❌ Songbird not found at: $SONGBIRD_BIN"
        log "   Harvest from phase1/songbird/target/release/songbird"
        exit 1
    fi

    log "✅ All binaries found in plasmidBin"
    log ""

    # ─────────────────────────────────────────────────────────────────────────
    # Step 1: Start Neural API (Coordinated Mode)
    # ─────────────────────────────────────────────────────────────────────────
    log "1️⃣  Starting Neural API (coordinated mode)..."
    
    export BIOMEOS_MODE="coordinated"
    "$BIOMEOS_BIN" neural-api --socket "$NEURAL_API_SOCKET" > /tmp/neural-api.log 2>&1 &
    NEURAL_PID=$!
    
    if wait_for_socket "$NEURAL_API_SOCKET" "Neural API"; then
        log "   ✅ Neural API started (PID: $NEURAL_PID)"
    else
        log "   ❌ Neural API failed to start"
        cat /tmp/neural-api.log | tail -20
        exit 1
    fi

    # ─────────────────────────────────────────────────────────────────────────
    # Step 2: Start BearDog
    # ─────────────────────────────────────────────────────────────────────────
    log ""
    log "2️⃣  Starting BearDog..."
    
    "$BEARDOG_BIN" server --socket "$BEARDOG_SOCKET" > /tmp/beardog.log 2>&1 &
    BEARDOG_PID=$!
    
    if wait_for_socket "$BEARDOG_SOCKET" "BearDog"; then
        log "   ✅ BearDog started (PID: $BEARDOG_PID)"
    else
        log "   ❌ BearDog failed to start"
        cat /tmp/beardog.log | tail -20
        exit 1
    fi

    # ─────────────────────────────────────────────────────────────────────────
    # Step 3: Start Songbird
    # ─────────────────────────────────────────────────────────────────────────
    log ""
    log "3️⃣  Starting Songbird..."
    
    "$SONGBIRD_BIN" server > /tmp/songbird.log 2>&1 &
    SONGBIRD_PID=$!
    
    sleep 5  # Songbird takes longer to initialize
    
    if pgrep -f "songbird server" > /dev/null; then
        log "   ✅ Songbird started (PID: $SONGBIRD_PID)"
    else
        log "   ⚠️  Songbird may not have started cleanly"
        cat /tmp/songbird.log | tail -10
    fi

    # ─────────────────────────────────────────────────────────────────────────
    # Step 4: Validate deployment
    # ─────────────────────────────────────────────────────────────────────────
    log ""
    log "4️⃣  Validating deployment..."
    
    # Test capability.call
    RESULT=$(echo '{"jsonrpc":"2.0","method":"capability.call","params":{"capability":"crypto","operation":"sha256","args":{"data":"dGVzdA=="}},"id":1}' | timeout 5 nc -U "$NEURAL_API_SOCKET" 2>/dev/null)
    
    if echo "$RESULT" | grep -q "hash"; then
        log "   ✅ capability.call → BearDog: Working"
    else
        log "   ⚠️  capability.call test: $RESULT"
    fi

    # ─────────────────────────────────────────────────────────────────────────
    # Summary
    # ─────────────────────────────────────────────────────────────────────────
    log ""
    log "╔══════════════════════════════════════════════════════════════════════╗"
    log "║   🏰 TOWER ATOMIC READY                                              ║"
    log "╚══════════════════════════════════════════════════════════════════════╝"
    log ""
    log "Sockets:"
    log "  Neural API: $NEURAL_API_SOCKET"
    log "  BearDog:    $BEARDOG_SOCKET"
    log "  Songbird:   /tmp/songbird-*.sock"
    log ""
    log "Usage:"
    log "  # Call crypto operations via Neural API"
    log '  echo '"'"'{"jsonrpc":"2.0","method":"capability.call","params":{"capability":"crypto","operation":"sha256","args":{"data":"dGVzdA=="}},"id":1}'"'"' | nc -U /tmp/neural-api.sock'
    log ""
    log "Logs:"
    log "  Neural API: /tmp/neural-api.log"
    log "  BearDog:    /tmp/beardog.log"
    log "  Songbird:   /tmp/songbird.log"
    log ""
    log "To stop: pkill -f biomeos; pkill -f beardog; pkill -f songbird"
}

# Handle cleanup on exit
trap cleanup EXIT INT TERM

# Run if executed directly (not sourced)
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
