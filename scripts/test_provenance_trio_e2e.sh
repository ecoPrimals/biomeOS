#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-only
# Provenance Trio E2E Test Runner
#
# Starts the full provenance stack (Tower + NestGate + Trio) and runs
# the E2E tests, then tears everything down.
#
# Usage:
#   ./scripts/test_provenance_trio_e2e.sh [family-id]

set -euo pipefail

FAMILY_ID="${1:-e2e-trio}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
SOCKET_DIR="${XDG_RUNTIME_DIR:-/tmp}/biomeos"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m'

PIDS=()

cleanup() {
    echo -e "\n${YELLOW}Cleaning up...${NC}"
    for pid in "${PIDS[@]}"; do
        kill "$pid" 2>/dev/null || true
    done
    rm -f "$SOCKET_DIR"/*-"$FAMILY_ID".sock
    echo -e "${GREEN}Cleanup complete${NC}"
}
trap cleanup EXIT

log() { echo -e "${GREEN}[trio-e2e]${NC} $*"; }
err() { echo -e "${RED}[trio-e2e]${NC} $*"; }

wait_for_socket() {
    local socket="$1"
    local name="$2"
    local timeout="${3:-15}"
    local elapsed=0

    while [ ! -S "$socket" ] && [ "$elapsed" -lt "$timeout" ]; do
        sleep 0.5
        elapsed=$((elapsed + 1))
    done

    if [ ! -S "$socket" ]; then
        err "$name socket not found after ${timeout}s: $socket"
        return 1
    fi
    log "$name ready"
}

mkdir -p "$SOCKET_DIR"

# ═══════════════════════════════════════════════════════════════════════════════
# Phase 1: Start Tower (BearDog + Songbird)
# ═══════════════════════════════════════════════════════════════════════════════

log "Starting Tower Atomic (BearDog + Songbird)..."

if command -v biomeos &>/dev/null; then
    log "Using biomeos nucleus start for Tower bootstrap"
    (cd "$PROJECT_DIR" && FAMILY_ID="$FAMILY_ID" biomeos nucleus start --mode tower --node-id tower1 --family-id "$FAMILY_ID") &
    PIDS+=($!)
    wait_for_socket "$SOCKET_DIR/beardog-$FAMILY_ID.sock" "BearDog"
    wait_for_socket "$SOCKET_DIR/songbird-$FAMILY_ID.sock" "Songbird"
else
    log "biomeos not found — assuming Tower is already running"
fi

# ═══════════════════════════════════════════════════════════════════════════════
# Phase 2: Start NestGate
# ═══════════════════════════════════════════════════════════════════════════════

log "Checking NestGate..."
if [ -S "$SOCKET_DIR/nestgate-$FAMILY_ID.sock" ]; then
    log "NestGate already running"
else
    log "NestGate not found — tests requiring storage may be skipped"
fi

# ═══════════════════════════════════════════════════════════════════════════════
# Phase 3: Start Provenance Trio
# ═══════════════════════════════════════════════════════════════════════════════

log "Checking Provenance Trio..."

for primal in rhizocrypt loamspine sweetgrass; do
    if [ -S "$SOCKET_DIR/$primal-$FAMILY_ID.sock" ]; then
        log "$primal already running"
    else
        log "$primal not found — will attempt graph deployment"
    fi
done

# Try graph-based deployment if Neural API is available
if [ -S "$SOCKET_DIR/neural-api-$FAMILY_ID.sock" ]; then
    log "Deploying trio via graph..."
    echo '{"jsonrpc":"2.0","method":"graph.execute","params":{"graph_id":"provenance_trio_deploy","params":{"FAMILY_ID":"'"$FAMILY_ID"'"}},"id":1}' | \
        socat - UNIX-CONNECT:"$SOCKET_DIR/neural-api-$FAMILY_ID.sock" 2>/dev/null || \
        log "Graph deployment returned (check output above)"
    sleep 3
fi

# ═══════════════════════════════════════════════════════════════════════════════
# Phase 4: Run E2E Tests
# ═══════════════════════════════════════════════════════════════════════════════

log "Running Provenance Trio E2E tests..."

cd "$PROJECT_DIR"
FAMILY_ID="$FAMILY_ID" cargo test \
    --test provenance_trio_e2e \
    -- --test-threads=1 --ignored --nocapture 2>&1

TEST_EXIT=$?

if [ "$TEST_EXIT" -eq 0 ]; then
    log "${GREEN}All Provenance Trio E2E tests PASSED${NC}"
else
    err "Some tests FAILED (exit code: $TEST_EXIT)"
fi

exit $TEST_EXIT
