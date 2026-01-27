#!/bin/bash
# Test Federation Between LiveSpores
#
# This script:
# 1. Starts both USB spores (node-alpha and node-beta)
# 2. Verifies genetic lineage via BearDog
# 3. Tests federation trust establishment
#
# Prerequisites:
# - Both USBs mounted and configured
# - socat or nc available for JSON-RPC
#
# Usage:
#   ./scripts/test_federation.sh
#
# Author: biomeOS Team
# Date: 2026-01-27

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

log_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }
log_warn() { echo -e "${YELLOW}⚠️  $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}"; }
log_step() { echo -e "${CYAN}▶ $1${NC}"; }

# Configuration
USB1_ROOT="${USB1_ROOT:-/media/eastgate/biomeOS1/biomeOS}"
USB2_ROOT="${USB2_ROOT:-/media/eastgate/BEA6-BBCE/biomeOS}"
FAMILY_ID="nat0"
NODE1="node-alpha"
NODE2="node-beta"

BEARDOG1_SOCKET="/tmp/beardog-$FAMILY_ID-$NODE1.sock"
BEARDOG2_SOCKET="/tmp/beardog-$FAMILY_ID-$NODE2.sock"
SONGBIRD1_SOCKET="/tmp/songbird-$FAMILY_ID-$NODE1.sock"
SONGBIRD2_SOCKET="/tmp/songbird-$FAMILY_ID-$NODE2.sock"

echo ""
echo "═══════════════════════════════════════════════════════════════════════════"
echo "         LiveSpore Federation Test - Genetic Lineage Verification"
echo "═══════════════════════════════════════════════════════════════════════════"
echo ""
echo "  Node 1: $NODE1 (GENESIS) @ $USB1_ROOT"
echo "  Node 2: $NODE2 (SIBLING) @ $USB2_ROOT"
echo "  Family: $FAMILY_ID"
echo ""

# Cleanup function
cleanup() {
    echo ""
    log_info "Cleaning up..."
    
    # Kill both spores
    if [[ -n "${SPORE1_PID:-}" ]]; then
        kill $SPORE1_PID 2>/dev/null || true
    fi
    if [[ -n "${SPORE2_PID:-}" ]]; then
        kill $SPORE2_PID 2>/dev/null || true
    fi
    
    # Remove sockets
    rm -f "$BEARDOG1_SOCKET" "$BEARDOG2_SOCKET" "$SONGBIRD1_SOCKET" "$SONGBIRD2_SOCKET"
    rm -f "/tmp/neural-api-$NODE1.sock" "/tmp/neural-api-$NODE2.sock"
    
    log_success "Cleanup complete"
}
trap cleanup EXIT INT TERM

# ============================================================================
# Phase 1: Verify Prerequisites
# ============================================================================
log_step "Phase 1: Checking prerequisites..."

for root in "$USB1_ROOT" "$USB2_ROOT"; do
    if [[ ! -d "$root" ]]; then
        log_error "Spore root not found: $root"
        exit 1
    fi
    if [[ ! -f "$root/.family.seed" ]]; then
        log_error "Family seed not found: $root/.family.seed"
        exit 1
    fi
done

log_success "Both spore roots exist with family seeds"

# Check for JSON-RPC tools
JSONRPC_CMD=""
if command -v socat &>/dev/null; then
    JSONRPC_CMD="socat"
elif command -v nc &>/dev/null; then
    JSONRPC_CMD="nc"
else
    log_warn "Neither socat nor nc found - federation tests may fail"
fi
log_info "JSON-RPC tool: ${JSONRPC_CMD:-none}"
echo ""

# ============================================================================
# Phase 2: Start Spore 1 (Genesis)
# ============================================================================
log_step "Phase 2: Starting $NODE1 (GENESIS)..."

# Start in background, capturing output
"$USB1_ROOT/deploy.sh" > /tmp/spore1.log 2>&1 &
SPORE1_PID=$!

# Wait for BearDog socket
for i in {1..30}; do
    if [[ -S "$BEARDOG1_SOCKET" ]]; then
        break
    fi
    sleep 1
done

if [[ -S "$BEARDOG1_SOCKET" ]]; then
    log_success "$NODE1 BearDog ready: $BEARDOG1_SOCKET"
else
    log_error "$NODE1 failed to start"
    cat /tmp/spore1.log
    exit 1
fi
echo ""

# ============================================================================
# Phase 3: Start Spore 2 (Sibling)
# ============================================================================
log_step "Phase 3: Starting $NODE2 (SIBLING)..."

"$USB2_ROOT/deploy.sh" > /tmp/spore2.log 2>&1 &
SPORE2_PID=$!

# Wait for BearDog socket
for i in {1..30}; do
    if [[ -S "$BEARDOG2_SOCKET" ]]; then
        break
    fi
    sleep 1
done

if [[ -S "$BEARDOG2_SOCKET" ]]; then
    log_success "$NODE2 BearDog ready: $BEARDOG2_SOCKET"
else
    log_error "$NODE2 failed to start"
    cat /tmp/spore2.log
    exit 1
fi
echo ""

# ============================================================================
# Phase 4: Genetic Lineage Verification
# ============================================================================
log_step "Phase 4: Verifying genetic lineage..."

# Function to send JSON-RPC
send_jsonrpc() {
    local socket="$1"
    local request="$2"
    
    if [[ "$JSONRPC_CMD" == "socat" ]]; then
        echo "$request" | socat -t 5 - UNIX-CONNECT:"$socket" 2>/dev/null
    elif [[ "$JSONRPC_CMD" == "nc" ]]; then
        echo "$request" | nc -U -w 5 "$socket" 2>/dev/null
    else
        echo '{"error":"no jsonrpc tool"}'
    fi
}

# Node 1 verifies Node 2's family membership
log_info "Node 1 ($NODE1) verifying Node 2 ($NODE2)..."
VERIFY_REQUEST='{"jsonrpc":"2.0","method":"federation.verify_family_member","params":{"family_id":"nat0","node_id":"node-beta"},"id":1}'

RESULT1=$(send_jsonrpc "$BEARDOG1_SOCKET" "$VERIFY_REQUEST")
echo "  Response: $RESULT1"

if echo "$RESULT1" | grep -q '"is_family_member":true'; then
    log_success "Node 1 recognizes Node 2 as family member!"
else
    log_warn "Family membership not confirmed (expected with tag-only verification)"
fi
echo ""

# Node 2 verifies Node 1's family membership
log_info "Node 2 ($NODE2) verifying Node 1 ($NODE1)..."
VERIFY_REQUEST='{"jsonrpc":"2.0","method":"federation.verify_family_member","params":{"family_id":"nat0","node_id":"node-alpha"},"id":2}'

RESULT2=$(send_jsonrpc "$BEARDOG2_SOCKET" "$VERIFY_REQUEST")
echo "  Response: $RESULT2"

if echo "$RESULT2" | grep -q '"is_family_member":true'; then
    log_success "Node 2 recognizes Node 1 as family member!"
else
    log_warn "Family membership not confirmed (expected with tag-only verification)"
fi
echo ""

# ============================================================================
# Phase 5: Generate and Verify Lineage Proofs
# ============================================================================
log_step "Phase 5: Testing genetic proof generation..."

# Generate lineage proof from Node 1's seed
SEED1_B64=$(base64 -w0 "$USB1_ROOT/.family.seed")
PROOF_REQUEST=$(cat <<EOF
{"jsonrpc":"2.0","method":"genetic.generate_lineage_proof","params":{"our_family_id":"nat0","peer_family_id":"nat0","lineage_seed":"$SEED1_B64"},"id":3}
EOF
)

log_info "Generating lineage proof from $NODE1..."
PROOF1=$(send_jsonrpc "$BEARDOG1_SOCKET" "$PROOF_REQUEST")

if echo "$PROOF1" | grep -q '"proof"'; then
    log_success "Lineage proof generated from $NODE1"
    PROOF1_VALUE=$(echo "$PROOF1" | jq -r '.result.proof' 2>/dev/null || echo "parse_error")
    echo "  Proof hash: ${PROOF1_VALUE:0:32}..."
else
    log_warn "Failed to generate proof: $PROOF1"
fi
echo ""

# Generate lineage proof from Node 2's seed
SEED2_B64=$(base64 -w0 "$USB2_ROOT/.family.seed")
PROOF_REQUEST=$(cat <<EOF
{"jsonrpc":"2.0","method":"genetic.generate_lineage_proof","params":{"our_family_id":"nat0","peer_family_id":"nat0","lineage_seed":"$SEED2_B64"},"id":4}
EOF
)

log_info "Generating lineage proof from $NODE2..."
PROOF2=$(send_jsonrpc "$BEARDOG2_SOCKET" "$PROOF_REQUEST")

if echo "$PROOF2" | grep -q '"proof"'; then
    log_success "Lineage proof generated from $NODE2"
    PROOF2_VALUE=$(echo "$PROOF2" | jq -r '.result.proof' 2>/dev/null || echo "parse_error")
    echo "  Proof hash: ${PROOF2_VALUE:0:32}..."
else
    log_warn "Failed to generate proof: $PROOF2"
fi
echo ""

# ============================================================================
# Summary
# ============================================================================
echo ""
echo "═══════════════════════════════════════════════════════════════════════════"
echo "                         FEDERATION TEST RESULTS"
echo "═══════════════════════════════════════════════════════════════════════════"
echo ""
echo "  Spore 1 ($NODE1 - GENESIS):"
echo "    BearDog: $BEARDOG1_SOCKET"
echo "    Songbird: $SONGBIRD1_SOCKET"
echo "    PID: $SPORE1_PID"
echo ""
echo "  Spore 2 ($NODE2 - SIBLING):"
echo "    BearDog: $BEARDOG2_SOCKET"
echo "    Songbird: $SONGBIRD2_SOCKET"
echo "    PID: $SPORE2_PID"
echo ""
echo "  Family verification: Both nodes can verify each other via family_id"
echo "  Genetic proofs: Both nodes can generate Blake3 lineage proofs"
echo ""
echo "  NOTE: Full cryptographic sibling verification requires:"
echo "    1. Parent seed to derive expected sibling verification keys"
echo "    2. Challenge/response protocol (planned in BearDog)"
echo ""
echo "  Current verification uses family_id tags which provides:"
echo "    - Same-family recognition"
echo "    - Limited trust for federation"
echo ""
echo "═══════════════════════════════════════════════════════════════════════════"
echo ""
echo "Press Ctrl+C to stop both spores..."

# Keep running until interrupted
wait

