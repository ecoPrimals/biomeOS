#!/bin/bash
# NUCLEUS Full Stack Validation Script
# Validates Tower, Node, and Nest atomics with AI capabilities
# 
# Usage: ./scripts/validate_nucleus_stack.sh
# 
# Requirements:
#   - Primals running: BearDog, Songbird, Squirrel, Toadstool, NestGate
#   - API keys in environment (ANTHROPIC_API_KEY, OPENAI_API_KEY)

set -e

echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║            NUCLEUS FULL STACK VALIDATION                         ║"
echo "║  Tower (BearDog+Songbird) + Node (Toadstool) + Nest (NestGate)  ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""

# Configuration
BIOMEOS_SOCKET_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/biomeos"
BEARDOG_SOCKET="${BIOMEOS_SOCKET_DIR}/beardog-node-alpha.sock"
SONGBIRD_SOCKET="${BIOMEOS_SOCKET_DIR}/songbird-node-alpha.sock"
NEURAL_API_SOCKET="${BIOMEOS_SOCKET_DIR}/neural-api-node-alpha.sock"
SQUIRREL_SOCKET="${BIOMEOS_SOCKET_DIR}/squirrel-1894e909e454.sock"
TOADSTOOL_SOCKET="${BIOMEOS_SOCKET_DIR}/toadstool-1894e909e454.jsonrpc.sock"
NESTGATE_SOCKET="${BIOMEOS_SOCKET_DIR}/nestgate-1894e909e454.sock"

PASSED=0
FAILED=0
TOTAL=0

# Test function
test_socket() {
    local name=$1
    local socket=$2
    local method=$3
    local expected=$4
    
    TOTAL=$((TOTAL + 1))
    echo -n "  [$TOTAL] $name... "
    
    if [ ! -S "$socket" ]; then
        echo "❌ FAILED (socket not found: $socket)"
        FAILED=$((FAILED + 1))
        return 1
    fi
    
    result=$(echo "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":{},\"id\":1}" | timeout 5 nc -U "$socket" 2>/dev/null)
    
    if echo "$result" | grep -q "$expected"; then
        echo "✅ PASSED"
        PASSED=$((PASSED + 1))
        return 0
    else
        echo "❌ FAILED"
        echo "      Response: $result"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

# Test with response capture
test_socket_capture() {
    local name=$1
    local socket=$2
    local method=$3
    local params=$4
    
    TOTAL=$((TOTAL + 1))
    echo -n "  [$TOTAL] $name... "
    
    if [ ! -S "$socket" ]; then
        echo "❌ FAILED (socket not found)"
        FAILED=$((FAILED + 1))
        return 1
    fi
    
    START=$(date +%s%3N)
    result=$(echo "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":$params,\"id\":1}" | timeout 30 nc -U "$socket" 2>/dev/null)
    END=$(date +%s%3N)
    LATENCY=$((END - START))
    
    if echo "$result" | grep -q "result"; then
        echo "✅ PASSED (${LATENCY}ms)"
        PASSED=$((PASSED + 1))
        echo "      Response: $(echo "$result" | head -c 200)..."
        return 0
    else
        echo "❌ FAILED (${LATENCY}ms)"
        echo "      Response: $result"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

echo "═══════════════════════════════════════════════════════════════════"
echo "PHASE 1: Socket Availability"
echo "═══════════════════════════════════════════════════════════════════"
echo ""

for socket_info in \
    "BearDog:$BEARDOG_SOCKET" \
    "Songbird:$SONGBIRD_SOCKET" \
    "Neural API:$NEURAL_API_SOCKET" \
    "Squirrel:$SQUIRREL_SOCKET" \
    "Toadstool:$TOADSTOOL_SOCKET" \
    "NestGate:$NESTGATE_SOCKET"; do
    
    name=$(echo "$socket_info" | cut -d: -f1)
    socket=$(echo "$socket_info" | cut -d: -f2-)
    TOTAL=$((TOTAL + 1))
    echo -n "  [$TOTAL] $name socket... "
    if [ -S "$socket" ]; then
        echo "✅ EXISTS"
        PASSED=$((PASSED + 1))
    else
        echo "❌ NOT FOUND"
        FAILED=$((FAILED + 1))
    fi
done

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "PHASE 2: Tower Atomic (Security + Network)"
echo "═══════════════════════════════════════════════════════════════════"
echo ""

test_socket "BearDog health" "$BEARDOG_SOCKET" "health" "healthy"
test_socket "Songbird health" "$SONGBIRD_SOCKET" "health" "healthy"

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "PHASE 3: Node Atomic (Compute Coordination)"
echo "═══════════════════════════════════════════════════════════════════"
echo ""

test_socket "Toadstool health" "$TOADSTOOL_SOCKET" "toadstool.health" "healthy"
test_socket "Toadstool version" "$TOADSTOOL_SOCKET" "toadstool.version" "0.1.0"

TOTAL=$((TOTAL + 1))
echo -n "  [$TOTAL] Toadstool GPU capabilities... "
result=$(echo '{"jsonrpc":"2.0","method":"toadstool.query_capabilities","params":{},"id":1}' | timeout 5 nc -U "$TOADSTOOL_SOCKET" 2>/dev/null)
if echo "$result" | grep -q "cpu_compute"; then
    cores=$(echo "$result" | grep -o '"total_cpu_cores":[0-9]*' | cut -d: -f2)
    memory=$(echo "$result" | grep -o '"total_memory_bytes":[0-9]*' | cut -d: -f2)
    memory_gb=$((memory / 1073741824))
    echo "✅ PASSED (${cores} cores, ${memory_gb}GB RAM)"
    PASSED=$((PASSED + 1))
else
    echo "❌ FAILED"
    FAILED=$((FAILED + 1))
fi

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "PHASE 4: Nest Atomic (Storage)"
echo "═══════════════════════════════════════════════════════════════════"
echo ""

TOTAL=$((TOTAL + 1))
echo -n "  [$TOTAL] NestGate storage.store... "
result=$(echo '{"jsonrpc":"2.0","method":"storage.store","params":{"family_id":"1894e909e454","key":"nucleus:validation","value":{"test":"NUCLEUS validation","timestamp":"'"$(date -Iseconds)"'"}},"id":1}' | timeout 5 nc -U "$NESTGATE_SOCKET" 2>/dev/null)
if echo "$result" | grep -q "success"; then
    echo "✅ PASSED"
    PASSED=$((PASSED + 1))
else
    echo "❌ FAILED: $result"
    FAILED=$((FAILED + 1))
fi

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "PHASE 5: AI Providers (External APIs via Tower)"
echo "═══════════════════════════════════════════════════════════════════"
echo ""

TOTAL=$((TOTAL + 1))
echo -n "  [$TOTAL] Squirrel AI providers discovery... "
result=$(echo '{"jsonrpc":"2.0","method":"list_providers","params":{},"id":1}' | timeout 5 nc -U "$SQUIRREL_SOCKET" 2>/dev/null)
if echo "$result" | grep -q "providers"; then
    count=$(echo "$result" | grep -o '"total":[0-9]*' | cut -d: -f2)
    echo "✅ PASSED ($count providers)"
    PASSED=$((PASSED + 1))
else
    echo "❌ FAILED"
    FAILED=$((FAILED + 1))
fi

# Test Anthropic (small query to minimize cost)
test_socket_capture "Anthropic API (Claude)" "$SQUIRREL_SOCKET" "query_ai" '{"prompt":"Reply with just: OK","model":"claude-3-haiku-20240307","max_tokens":10}'

# Test OpenAI (small query to minimize cost)  
test_socket_capture "OpenAI API (GPT)" "$SQUIRREL_SOCKET" "query_ai" '{"prompt":"Reply with just: OK","model":"gpt-4o-mini","max_tokens":10}'

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "PHASE 6: TLS 1.3 Cryptography (BearDog)"
echo "═══════════════════════════════════════════════════════════════════"
echo ""

test_socket "X25519 key generation" "$BEARDOG_SOCKET" "x25519_generate_ephemeral" "public_key"
test_socket "AES-128-GCM availability" "$BEARDOG_SOCKET" "crypto.capabilities" "aes"

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "PHASE 7: Neural API Semantic Routing"
echo "═══════════════════════════════════════════════════════════════════"
echo ""

test_socket "Neural API health" "$NEURAL_API_SOCKET" "health" "healthy"
test_socket "Protocol status" "$NEURAL_API_SOCKET" "protocol.status" "family_id"

echo ""
echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║                      VALIDATION SUMMARY                          ║"
echo "╠═══════════════════════════════════════════════════════════════════╣"
printf "║  Total Tests: %-3d                                                ║\n" $TOTAL
printf "║  Passed:      %-3d ✅                                             ║\n" $PASSED
printf "║  Failed:      %-3d ❌                                             ║\n" $FAILED
echo "╠═══════════════════════════════════════════════════════════════════╣"

if [ $FAILED -eq 0 ]; then
    echo "║  🎉 ALL TESTS PASSED - NUCLEUS STACK OPERATIONAL 🎉             ║"
else
    echo "║  ⚠️  SOME TESTS FAILED - CHECK PRIMAL STATUS                    ║"
fi
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""
echo "Timestamp: $(date -Iseconds)"
echo "Host: $(hostname)"

exit $FAILED

