#!/system/bin/sh
#═══════════════════════════════════════════════════════════════════════════════
# NUCLEUS Mobile Deployment - Pixel 8a (Sovereign Mode)
# Architecture: aarch64 (ARM64)
# Standard: Evolved Genetic Standard v2.0 — Sovereign Multi-Path
#
# Seeds loaded from BIOMEOS_ROOT:
#   .family.seed  - mitochondrial DNA (shared across family)
#   .beacon.seed  - beacon identity (derived from family)
#   .lineage.seed - nuclear DNA (unique to THIS device)
#   .lineage.json - identity record
#
# On Android, BearDog falls back to TCP 127.0.0.1:9900 for IPC
# when Unix domain sockets fail on the filesystem.
#
# Deployment Phases (matching tower_atomic_bootstrap.toml):
#   1. BearDog (crypto foundation)
#   2. Songbird (network + discovery)
#   2a. Sovereign Onion Service
#   2b. Mesh Network init
#   2c. STUN NAT discovery
#   3. Dark Forest beacon verification
#   4. Health validation
#═══════════════════════════════════════════════════════════════════════════════

set -e

# Environment
export BIOMEOS_ROOT="/data/local/tmp/biomeos"
export NODE_ID="${NODE_ID:-pixel8a}"
export RUST_LOG="${RUST_LOG:-info}"
export XDG_RUNTIME_DIR="/data/local/tmp"

#───────────────────────────────────────────────────────────────────────────────
# Family ID Derivation from Mitochondrial Seed
# Standard: hex(family.seed[0..8]) = 16 hex chars
# On Android, xxd may not be available — use od as fallback
#───────────────────────────────────────────────────────────────────────────────
derive_family_id() {
    local seed_file="$BIOMEOS_ROOT/.family.seed"
    if [ ! -f "$seed_file" ]; then
        return 1
    fi
    # Try xxd first, fall back to od + printf
    if command -v xxd >/dev/null 2>&1; then
        xxd -p -l 8 "$seed_file" | tr -d '\n'
    else
        # od fallback for Android (toybox)
        od -A n -t x1 -N 8 "$seed_file" | tr -d ' \n'
    fi
}

if [ -n "$FAMILY_ID" ] && [ ${#FAMILY_ID} -ge 16 ]; then
    export FAMILY_ID
elif SEED_ID=$(derive_family_id) && [ -n "$SEED_ID" ]; then
    export FAMILY_ID="$SEED_ID"
else
    echo "No .family.seed found and FAMILY_ID not set"
    exit 1
fi

PRIMAL_DIR="$BIOMEOS_ROOT/primals"
SOCKET_DIR="$XDG_RUNTIME_DIR/biomeos"

echo "═══════════════════════════════════════════════════════════════"
echo "  NUCLEUS Mobile - Sovereign Multi-Path Protocol"
echo "  Pure Rust ecoPrimals (no external dependencies)"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Device:        $(getprop ro.product.model 2>/dev/null || echo 'mobile')"
echo "Android:       $(getprop ro.build.version.release 2>/dev/null || echo 'unknown')"
echo "Architecture:  $(uname -m)"
echo "Family ID:     $FAMILY_ID"
echo "Node ID:       $NODE_ID"
echo "BIOMEOS_ROOT:  $BIOMEOS_ROOT"
echo "Socket Dir:    $SOCKET_DIR"
echo ""

mkdir -p "$SOCKET_DIR"
mkdir -p "$PRIMAL_DIR"

#───────────────────────────────────────────────────────────────────────────────
# Seeds Verification
#───────────────────────────────────────────────────────────────────────────────
echo "Seeds:"
[ -f "$BIOMEOS_ROOT/.family.seed" ] && echo "  .family.seed    OK (mitochondrial)" || { echo "  .family.seed    MISSING"; exit 1; }
[ -f "$BIOMEOS_ROOT/.beacon.seed" ] && echo "  .beacon.seed    OK (beacon)" || echo "  .beacon.seed    -- (will use family seed)"
[ -f "$BIOMEOS_ROOT/.lineage.seed" ] && echo "  .lineage.seed   OK (nuclear)" || echo "  .lineage.seed   -- (will derive on first run)"
[ -f "$BIOMEOS_ROOT/.lineage.json" ] && echo "  .lineage.json   OK (identity)" || echo "  .lineage.json   -- (will derive on first run)"
[ -f "$BIOMEOS_ROOT/.known_beacons.json" ] && echo "  .known_beacons  OK (address book)" || echo "  .known_beacons  -- (will discover)"
echo ""

# Kill stale processes
pkill -f "beardog server" 2>/dev/null || true
pkill -f "songbird server" 2>/dev/null || true
rm -f "$SOCKET_DIR"/*.sock 2>/dev/null
sleep 1

#───────────────────────────────────────────────────────────────────────────────
# Phase 1: BearDog (Crypto Foundation)
#───────────────────────────────────────────────────────────────────────────────
echo "Phase 1: BearDog (Security)..."
cd "$BIOMEOS_ROOT"
export BEARDOG_SOCKET="$SOCKET_DIR/beardog.sock"

"$PRIMAL_DIR/beardog" server \
    --socket "$BEARDOG_SOCKET" > "$BIOMEOS_ROOT/beardog.log" 2>&1 &
BEARDOG_PID=$!
echo "  PID: $BEARDOG_PID"

# Wait for BearDog (may fall back to TCP on Android)
BEARDOG_IPC=""
tries=0
while [ $tries -lt 12 ]; do
    if [ -S "$BEARDOG_SOCKET" ]; then
        echo "  BearDog ready (Unix socket)"
        BEARDOG_IPC="$BEARDOG_SOCKET"
        break
    fi
    sleep 1
    tries=$((tries + 1))
done

if [ -z "$BEARDOG_IPC" ]; then
    echo "  Unix socket not available, checking TCP fallback..."
    # BearDog falls back to TCP 127.0.0.1:9900 on Android
    if nc -z 127.0.0.1 9900 2>/dev/null; then
        echo "  BearDog ready (TCP 127.0.0.1:9900)"
        BEARDOG_IPC="tcp://127.0.0.1:9900"
    else
        echo "  BearDog not responding after ${tries}s"
        tail -15 "$BIOMEOS_ROOT/beardog.log" 2>/dev/null
        exit 1
    fi
fi

#───────────────────────────────────────────────────────────────────────────────
# Phase 2: Songbird (Network + Discovery)
#───────────────────────────────────────────────────────────────────────────────
echo "Phase 2: Songbird (Network)..."
SONGBIRD_PORT="${SONGBIRD_PORT:-9901}"
export SONGBIRD_SOCKET="$SOCKET_DIR/songbird.sock"
export SONGBIRD_SECURITY_PROVIDER="$BEARDOG_IPC"
export CRYPTO_PROVIDER_SOCKET="$BEARDOG_IPC"
export BIOMEOS_BIND_ALL=true
export BIOMEOS_SOVEREIGN=true
export BIOMEOS_DARK_FOREST=true
export BIOMEOS_NO_EXTERNAL_DEPS=true
export SONGBIRD_ONION_ENABLED=true
export SONGBIRD_MESH_ENABLED=true

cd "$BIOMEOS_ROOT"
"$PRIMAL_DIR/songbird" server \
    --port "$SONGBIRD_PORT" \
    --socket "$SONGBIRD_SOCKET" \
    --verbose > "$BIOMEOS_ROOT/songbird.log" 2>&1 &
SONGBIRD_PID=$!
echo "  PID: $SONGBIRD_PID"

# Wait for Songbird
tries=0
while [ $tries -lt 12 ]; do
    if [ -S "$SONGBIRD_SOCKET" ]; then
        echo "  Songbird ready (TCP :$SONGBIRD_PORT + IPC)"
        break
    fi
    sleep 1
    tries=$((tries + 1))
done

if [ ! -S "$SONGBIRD_SOCKET" ]; then
    echo "  Songbird socket pending, checking TCP..."
    if nc -z 127.0.0.1 $SONGBIRD_PORT 2>/dev/null; then
        echo "  Songbird ready (TCP :$SONGBIRD_PORT)"
    else
        echo "  Songbird still starting..."
        tail -5 "$BIOMEOS_ROOT/songbird.log" 2>/dev/null
    fi
fi

#───────────────────────────────────────────────────────────────────────────────
# Phase 2a: Sovereign Onion Service
#───────────────────────────────────────────────────────────────────────────────
rpc_songbird() {
    local method=$1 params=$2
    if [ -S "$SONGBIRD_SOCKET" ]; then
        echo "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":$params,\"id\":1}" | \
            nc -U "$SONGBIRD_SOCKET" -w 8 -q 2 2>/dev/null
    else
        # TCP fallback
        echo "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":$params,\"id\":1}" | \
            nc 127.0.0.1 "$SONGBIRD_PORT" -w 8 -q 2 2>/dev/null
    fi
}

echo ""
echo "Phase 2a: Sovereign Onion Service..."
ONION_RESP=$(rpc_songbird "onion.start" "{\"port\":$SONGBIRD_PORT}")
if echo "$ONION_RESP" | grep -q '"started"'; then
    ONION_ADDR=$(echo "$ONION_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['result']['onion_address'])" 2>/dev/null || echo "?")
    echo "  Onion: $ONION_ADDR:$SONGBIRD_PORT"
else
    echo "  Onion: pending (will retry)"
fi

#───────────────────────────────────────────────────────────────────────────────
# Phase 2b: Mesh Network Init
#───────────────────────────────────────────────────────────────────────────────
echo "Phase 2b: Mesh Network..."
MESH_RESP=$(rpc_songbird "mesh.init" "{\"family_id\":\"$FAMILY_ID\",\"node_id\":\"$NODE_ID\"}")
if echo "$MESH_RESP" | grep -q '"initialized"'; then
    echo "  Mesh: initialized (node=$NODE_ID)"
else
    echo "  Mesh: pending"
fi

#───────────────────────────────────────────────────────────────────────────────
# Phase 2c: STUN NAT Discovery
#───────────────────────────────────────────────────────────────────────────────
echo "Phase 2c: STUN NAT Discovery..."
STUN_RESP=$(rpc_songbird "stun.get_public_address" "{}")
if echo "$STUN_RESP" | grep -q '"public_address"'; then
    PUB_ADDR=$(echo "$STUN_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['result']['full_address'])" 2>/dev/null || echo "?")
    echo "  Public: $PUB_ADDR"
else
    echo "  STUN: pending (may be behind NAT)"
fi

#───────────────────────────────────────────────────────────────────────────────
# Phase 3: Dark Forest Beacon Verification
#───────────────────────────────────────────────────────────────────────────────
echo ""
echo "Phase 3: Dark Forest Beacon..."
BEACON_RESP=$(rpc_songbird "birdsong.generate_encrypted_beacon" "{\"node_id\":\"$NODE_ID\",\"capabilities\":[\"http\",\"discovery\",\"security\"]}")
if echo "$BEACON_RESP" | grep -q '"encrypted_beacon"'; then
    echo "  Dark Forest: beacon ready"
else
    echo "  Dark Forest: beacon pending"
fi

#───────────────────────────────────────────────────────────────────────────────
# Phase 4: Health Validation
#───────────────────────────────────────────────────────────────────────────────
echo ""
echo "Phase 4: Validation..."
BD_OK=false; SB_OK=false

# BearDog health
if [ -S "$BEARDOG_SOCKET" ]; then
    BD_RESP=$(echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' | \
        nc -U "$BEARDOG_SOCKET" -w 3 -q 1 2>/dev/null)
    echo "$BD_RESP" | grep -q '"healthy"' && BD_OK=true
elif nc -z 127.0.0.1 9900 2>/dev/null; then
    BD_OK=true
fi
$BD_OK && echo "  BearDog:  OK" || echo "  BearDog:  FAIL"

# Songbird health
SB_RESP=$(rpc_songbird "health" "{}")
echo "$SB_RESP" | grep -q '"healthy"' && SB_OK=true
$SB_OK && echo "  Songbird: OK" || echo "  Songbird: FAIL"

echo ""
echo "═══════════════════════════════════════════════════════════════"
if $BD_OK && $SB_OK; then
    echo "  NUCLEUS Mobile OPERATIONAL - Pixel 8a (Sovereign)"
else
    echo "  NUCLEUS Mobile PARTIAL - some services pending"
fi
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Sockets:"
ls -lh "$SOCKET_DIR"/*.sock 2>/dev/null || echo "  (TCP mode - no sockets)"
echo ""
echo "PIDs: BearDog=$BEARDOG_PID, Songbird=$SONGBIRD_PID"
echo ""
echo "Network:"
echo "  Songbird TCP:  :$SONGBIRD_PORT"
echo "  BearDog:       :9900 (TCP fallback)"
[ -n "$ONION_ADDR" ] && echo "  Onion:         $ONION_ADDR:$SONGBIRD_PORT"
[ -n "$PUB_ADDR" ] && echo "  Public STUN:   $PUB_ADDR"
echo ""
echo "Tower beacon:"
echo "  IPv4: nestgate.io:3492"
echo "  IPv6: tower.nestgate.io:3492"
echo "  Onion: (see .known_beacons.json)"
echo ""
echo "Logs:"
echo "  BearDog:  $BIOMEOS_ROOT/beardog.log"
echo "  Songbird: $BIOMEOS_ROOT/songbird.log"
echo ""
echo "Deploy to Pixel:"
echo "  adb push $BIOMEOS_ROOT /data/local/tmp/biomeos"
echo "  adb shell sh /data/local/tmp/biomeos/start_nucleus_mobile.sh"
echo ""
