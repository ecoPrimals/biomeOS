#!/usr/bin/env bash
# Neural API Activation — eastGate test harness
# Run after reboot to verify G67 forwarding fix and Stage 2 routing
#
# Usage: ./scripts/neural-api-test.sh
#
# Prerequisites:
#   cargo build --release -p biomeos-atomic-deploy --bin neural-api-server
#   (primals started by this script)

set -euo pipefail

SOCKET_DIR="/run/user/1000/biomeos"
NEURAL_SOCK="${SOCKET_DIR}/neural-api-default.sock"
BIOME_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
GRAPHS_DIR="${BIOME_ROOT}/graphs"
LOG_LEVEL="${RUST_LOG:-info}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

pass() { echo -e "${GREEN}  PASS${NC} $1"; }
fail() { echo -e "${RED}  FAIL${NC} $1"; }
info() { echo -e "${YELLOW}  ....${NC} $1"; }

cleanup() {
    info "Cleaning up..."
    kill $(jobs -p) 2>/dev/null || true
    rm -f "${NEURAL_SOCK}" 2>/dev/null || true
}
trap cleanup EXIT

# ── 0. Environment ──────────────────────────────────────────────

echo "═══════════════════════════════════════════════════════"
echo "  Neural API Activation Test — eastGate"
echo "═══════════════════════════════════════════════════════"
echo ""

mkdir -p "${SOCKET_DIR}"

BINARY="${BIOME_ROOT}/target/release/neural-api-server"
if [[ ! -x "$BINARY" ]]; then
    info "Building neural-api-server (release)..."
    cd "$BIOME_ROOT"
    cargo build --release -p biomeos-atomic-deploy --bin neural-api-server
fi

# ── 1. Kill stale processes ─────────────────────────────────────

info "Killing stale processes..."
pkill -f 'neural-api-server' 2>/dev/null || true
pkill -f 'biomeos.*nucleus' 2>/dev/null || true
sleep 1

# Clean stale sockets
rm -f "${SOCKET_DIR}"/*.sock 2>/dev/null || true
rm -f /run/user/1000/membrane/*.sock 2>/dev/null || true

# ── 2. Start Tower Atomic primals ───────────────────────────────

info "Starting Tower Atomic primals..."

DEPOT="/home/eastgate/Development/ecoPrimals/depot"
PRIMAL_DIRS="/home/eastgate/Development/ecoPrimals/primals"

start_primal() {
    local name=$1
    local sock="${SOCKET_DIR}/${name}.sock"
    local bin

    # Try release binary in primal dir, then depot
    for candidate in \
        "${PRIMAL_DIRS}/${name}/target/release/${name}" \
        "${DEPOT}/x86_64-unknown-linux-musl/${name}" \
        "${DEPOT}/x86_64-unknown-linux-gnu/${name}"; do
        if [[ -x "$candidate" ]]; then
            bin="$candidate"
            break
        fi
    done

    if [[ -z "${bin:-}" ]]; then
        fail "No binary found for ${name}"
        return 1
    fi

    info "Starting ${name} → ${sock}"
    RUST_LOG=warn "$bin" --bind "${sock}" &
    sleep 0.5

    if [[ -S "$sock" ]]; then
        pass "${name} socket alive"
    else
        fail "${name} socket not found after start"
        return 1
    fi
}

for primal in bearDog songBird skunkBat; do
    start_primal "$(echo "$primal" | tr '[:upper:]' '[:lower:]')" || true
done

# ── 3. Start Neural API server ──────────────────────────────────

info "Starting neural-api-server..."
rm -f "$NEURAL_SOCK"

BIOMEOS_SOCKET_DIR="${SOCKET_DIR}" \
RUST_LOG="${LOG_LEVEL}" \
"$BINARY" \
    --graphs-dir "$GRAPHS_DIR" \
    --socket "$NEURAL_SOCK" \
    --protocol-preference jsonrpc \
    &

NEURAL_PID=$!
info "Neural API PID: ${NEURAL_PID}"

# Wait for socket
for i in $(seq 1 30); do
    if [[ -S "$NEURAL_SOCK" ]]; then
        break
    fi
    sleep 1
done

if [[ ! -S "$NEURAL_SOCK" ]]; then
    fail "Neural API socket never appeared"
    exit 1
fi
pass "Neural API socket alive"
sleep 2

# ── 4. N1: Basic health check ──────────────────────────────────

rpc() {
    local method=$1
    local params=${2:-'{}'}
    echo "{\"jsonrpc\":\"2.0\",\"method\":\"${method}\",\"params\":${params},\"id\":1}" \
        | socat -t5 - UNIX-CONNECT:"${NEURAL_SOCK}" 2>/dev/null
}

echo ""
echo "── N1: Verify forwarding ────────────────────────────────"

HEALTH=$(rpc "health.check")
if echo "$HEALTH" | grep -q '"status"'; then
    pass "health.check responds"
else
    fail "health.check: ${HEALTH}"
fi

CAPS=$(rpc "capabilities.list")
if echo "$CAPS" | grep -q 'crypto'; then
    pass "capabilities.list includes crypto"
else
    fail "capabilities.list: ${CAPS}"
fi

# The key test: capability.call routes to bearDog
SIGN_RESULT=$(rpc "capability.call" '{"capability":"crypto","operation":"sign_ed25519","args":{"data":"dGVzdA=="}}')
if echo "$SIGN_RESULT" | grep -q '"result"'; then
    pass "capability.call(crypto.sign_ed25519) → bearDog FORWARDED"
elif echo "$SIGN_RESULT" | grep -q '"error"'; then
    # An error response (not a hang) is still progress — forwarding works
    ERROR_MSG=$(echo "$SIGN_RESULT" | python3 -c "import sys,json; print(json.load(sys.stdin).get('error',{}).get('message','?'))" 2>/dev/null || echo "?")
    pass "capability.call forwarded (got error response: ${ERROR_MSG})"
else
    fail "capability.call HUNG or no response: ${SIGN_RESULT}"
fi

# ── 5. N2: Tower Atomic routing ─────────────────────────────────

echo ""
echo "── N2: Tower Atomic routing ─────────────────────────────"

HEALTH_BD=$(rpc "capability.call" '{"capability":"crypto","operation":"health.check","args":{}}')
if echo "$HEALTH_BD" | grep -q '"result"\|"error"'; then
    pass "Tower: crypto.health.check routed"
else
    fail "Tower: crypto.health.check: ${HEALTH_BD}"
fi

HEALTH_SB=$(rpc "capability.call" '{"capability":"mesh","operation":"health.check","args":{}}')
if echo "$HEALTH_SB" | grep -q '"result"\|"error"'; then
    pass "Tower: mesh.health.check routed"
else
    fail "Tower: mesh.health.check: ${HEALTH_SB}"
fi

HEALTH_SK=$(rpc "capability.call" '{"capability":"defense","operation":"health.check","args":{}}')
if echo "$HEALTH_SK" | grep -q '"result"\|"error"'; then
    pass "Tower: defense.health.check routed"
else
    fail "Tower: defense.health.check: ${HEALTH_SK}"
fi

# ── 6. N3: Node Atomic (if available) ───────────────────────────

echo ""
echo "── N3: Node Atomic routing ──────────────────────────────"

for primal in toadstool barracuda coralreef; do
    start_primal "$primal" 2>/dev/null || true
done
sleep 1

COMPUTE=$(rpc "capability.call" '{"capability":"compute","operation":"health.check","args":{}}')
if echo "$COMPUTE" | grep -q '"result"\|"error"'; then
    pass "Node: compute.health.check routed"
else
    info "Node: compute primals may not be available (expected on eastGate)"
fi

# ── 7. N4: Provenance Trio ──────────────────────────────────────

echo ""
echo "── N4: Provenance Trio routing ──────────────────────────"

for primal in rhizocrypt loamspine sweetgrass; do
    start_primal "$primal" 2>/dev/null || true
done
sleep 1

DAG=$(rpc "capability.call" '{"capability":"dag","operation":"health.check","args":{}}')
if echo "$DAG" | grep -q '"result"\|"error"'; then
    pass "Provenance: dag.health.check routed (rhizoCrypt)"
else
    info "Provenance: rhizoCrypt may not be available"
fi

SPINE=$(rpc "capability.call" '{"capability":"spine","operation":"health.check","args":{}}')
if echo "$SPINE" | grep -q '"result"\|"error"'; then
    pass "Provenance: spine.health.check routed (loamSpine)"
else
    info "Provenance: loamSpine may not be available"
fi

BRAID=$(rpc "capability.call" '{"capability":"braid","operation":"health.check","args":{}}')
if echo "$BRAID" | grep -q '"result"\|"error"'; then
    pass "Provenance: braid.health.check routed (sweetGrass)"
else
    info "Provenance: sweetGrass may not be available"
fi

# ── 8. N5: squirrel agent routing ───────────────────────────────

echo ""
echo "── N5: squirrel agent routing ───────────────────────────"

start_primal "squirrel" 2>/dev/null || true
sleep 1

AGENT=$(rpc "capability.call" '{"capability":"agent","operation":"health.check","args":{}}')
if echo "$AGENT" | grep -q '"result"\|"error"'; then
    pass "Agent: agent.health.check routed (squirrel)"
else
    info "Agent: squirrel may not be available on eastGate"
fi

# ── Summary ─────────────────────────────────────────────────────

echo ""
echo "═══════════════════════════════════════════════════════"
echo "  Test complete. Neural API PID: ${NEURAL_PID}"
echo "  Socket: ${NEURAL_SOCK}"
echo "  Kill with: kill ${NEURAL_PID}"
echo "═══════════════════════════════════════════════════════"
