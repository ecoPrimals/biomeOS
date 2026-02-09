#!/bin/bash
# NUCLEUS Quick Stack Validation
# Run: ./scripts/validate_nucleus_quick.sh

set -e
cd "$(dirname "$0")/.."

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║          NUCLEUS QUICK VALIDATION                        ║"
echo "╚═══════════════════════════════════════════════════════════╝"

SOCKET_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/biomeos"
PASSED=0
FAILED=0

test_primal() {
    local name=$1
    local socket=$2
    local method=$3
    local expect=$4
    
    printf "  %-20s " "$name:"
    if [ ! -S "$socket" ]; then
        echo "❌ Socket missing"
        FAILED=$((FAILED + 1))
        return
    fi
    
    result=$(echo "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":{},\"id\":1}" | \
             timeout 3 nc -U "$socket" 2>/dev/null | head -1)
    
    if echo "$result" | grep -q "$expect"; then
        echo "✅"
        PASSED=$((PASSED + 1))
    else
        echo "❌"
        FAILED=$((FAILED + 1))
    fi
}

echo ""
echo "=== Tower Atomic (Security + Network) ==="
test_primal "BearDog" "$SOCKET_DIR/beardog-node-alpha.sock" "health" "healthy"
test_primal "Songbird" "$SOCKET_DIR/songbird-node-alpha.sock" "version" "jsonrpc"

echo ""
echo "=== Node Atomic (Compute) ==="
test_primal "Toadstool" "$SOCKET_DIR/toadstool-1894e909e454.jsonrpc.sock" "toadstool.health" "healthy"

echo ""
echo "=== Nest Atomic (Storage) ==="
test_primal "NestGate" "$SOCKET_DIR/nestgate-1894e909e454.sock" "storage.store" "jsonrpc"

echo ""
echo "=== AI Router ==="
test_primal "Squirrel" "$SOCKET_DIR/squirrel-1894e909e454.sock" "list_providers" "providers"

echo ""
echo "=== Neural API ==="
test_primal "Neural API" "$SOCKET_DIR/neural-api-node-alpha.sock" "protocol.status" "family_id"

echo ""
echo "══════════════════════════════════════════════════════════════"
printf "Results: %d PASSED, %d FAILED\n" $PASSED $FAILED

if [ $FAILED -eq 0 ]; then
    echo "🎉 NUCLEUS OPERATIONAL"
else
    echo "⚠️  CHECK FAILED COMPONENTS"
fi

exit $FAILED

