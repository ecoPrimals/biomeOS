#!/bin/bash
#═══════════════════════════════════════════════════════════════════════════════
# LiveSpore USB - Sovereign Tower Atomic
# Architecture: x86_64 (auto-detects, works on any arch)
# Standard: Evolved Genetic Standard v2.0 — Sovereign Multi-Path
#
# Seeds loaded from livespore-usb root:
#   .family.seed  - mitochondrial DNA (shared across family)
#   .beacon.seed  - beacon identity (derived from family)
#   .lineage.seed - nuclear DNA (unique to THIS device)
#   .lineage.json - identity record
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

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# USB_ROOT: use BIOMEOS_ROOT if set, otherwise resolve from script location
# Original LiveSpore USB: scripts is at x86_64/scripts (2 levels deep)
# Flat deployment: scripts is at scripts/ (1 level deep)
if [ -n "$BIOMEOS_ROOT" ] && [ -f "$BIOMEOS_ROOT/.family.seed" ]; then
    USB_ROOT="$BIOMEOS_ROOT"
elif [ -f "$SCRIPT_DIR/../.family.seed" ]; then
    USB_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
elif [ -f "$SCRIPT_DIR/../../.family.seed" ]; then
    USB_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
else
    USB_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
fi
# Primals: check flat layout first, then USB layout
if [ -d "$USB_ROOT/primals" ]; then
    PRIMAL_DIR="$USB_ROOT/primals"
else
    PRIMAL_DIR="$SCRIPT_DIR/../primals"
fi
ARCH="$(uname -m)"

# Evolved standard environment
export NODE_ID="${NODE_ID:-usb}"
export RUST_LOG="${RUST_LOG:-info}"
export BIOMEOS_ROOT="$USB_ROOT"

#───────────────────────────────────────────────────────────────────────────────
# Family ID Derivation from Mitochondrial Seed
# Standard: hex(family.seed[0..8]) = 16 hex chars
#───────────────────────────────────────────────────────────────────────────────
derive_family_id() {
    local seed_file=""
    for candidate in "$USB_ROOT/.family.seed" "$SCRIPT_DIR/../../.family.seed"; do
        if [ -f "$candidate" ]; then
            seed_file="$candidate"
            break
        fi
    done
    if [ -n "$seed_file" ]; then
        xxd -p -l 8 "$seed_file" | tr -d '\n'
    fi
}

if [ -n "$FAMILY_ID" ] && [ ${#FAMILY_ID} -ge 16 ]; then
    export FAMILY_ID
elif SEED_ID=$(derive_family_id) && [ -n "$SEED_ID" ]; then
    export FAMILY_ID="$SEED_ID"
    [ -n "$FAMILY_ID" ] && [ ${#FAMILY_ID} -lt 16 ] && echo "⚠️  Short FAMILY_ID override: $FAMILY_ID"
else
    echo "❌ No .family.seed found and FAMILY_ID not set"
    exit 1
fi

# Socket directory (5-tier resolution)
if [ -n "$BIOMEOS_SOCKET_DIR" ]; then
    SOCKET_DIR="$BIOMEOS_SOCKET_DIR"
elif [ -n "$XDG_RUNTIME_DIR" ]; then
    SOCKET_DIR="$XDG_RUNTIME_DIR/biomeos"
elif [ -d "/run/user/$(id -u)" ]; then
    SOCKET_DIR="/run/user/$(id -u)/biomeos"
elif [ -d "/data/local/tmp" ]; then
    SOCKET_DIR="/data/local/tmp/biomeos"
else
    SOCKET_DIR="/tmp/biomeos"
fi

echo "═══════════════════════════════════════════════════════════════"
echo "  LiveSpore USB - Sovereign Multi-Path Protocol"
echo "  Pure Rust ecoPrimals (no external dependencies)"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Architecture:  $ARCH"
echo "Family ID:     $FAMILY_ID"
echo "Node ID:       $NODE_ID"
echo "BIOMEOS_ROOT:  $BIOMEOS_ROOT"
echo "Socket Dir:    $SOCKET_DIR"
echo "Primal Dir:    $PRIMAL_DIR"
echo ""

#───────────────────────────────────────────────────────────────────────────────
# Seeds Verification
#───────────────────────────────────────────────────────────────────────────────
echo "Seeds:"
[ -f "$USB_ROOT/.family.seed" ] && echo "  .family.seed    OK (mitochondrial)" || { echo "  .family.seed    MISSING"; exit 1; }
[ -f "$USB_ROOT/.beacon.seed" ] && echo "  .beacon.seed    OK (beacon)" || echo "  .beacon.seed    -- (will use family seed)"
[ -f "$USB_ROOT/.lineage.seed" ] && echo "  .lineage.seed   OK (nuclear)" || echo "  .lineage.seed   -- (will derive on first run)"
[ -f "$USB_ROOT/.lineage.json" ] && echo "  .lineage.json   OK (identity)" || echo "  .lineage.json   -- (will derive on first run)"
[ -f "$USB_ROOT/.known_beacons.json" ] && echo "  .known_beacons  OK (address book)" || echo "  .known_beacons  -- (will discover)"
echo ""

# Create socket directory
mkdir -p "$SOCKET_DIR"

# Kill stale processes
pkill -f "beardog server" 2>/dev/null || true
pkill -f "songbird server" 2>/dev/null || true
rm -f "$SOCKET_DIR"/*.sock 2>/dev/null
sleep 1

#───────────────────────────────────────────────────────────────────────────────
# Phase 1: BearDog (Crypto Foundation)
#───────────────────────────────────────────────────────────────────────────────
echo "Phase 1: BearDog (Security)..."
export BEARDOG_SOCKET="$SOCKET_DIR/beardog.sock"

"$PRIMAL_DIR/beardog" server \
    --socket "$BEARDOG_SOCKET" > /tmp/beardog_usb.log 2>&1 &
BEARDOG_PID=$!
echo "  PID: $BEARDOG_PID"

tries=0
while [ ! -S "$BEARDOG_SOCKET" ] && [ $tries -lt 12 ]; do
    sleep 1
    tries=$((tries + 1))
done

if [ -S "$BEARDOG_SOCKET" ]; then
    echo "  BearDog operational"
else
    echo "  BearDog socket not created after ${tries}s"
    tail -10 /tmp/beardog_usb.log
    exit 1
fi

#───────────────────────────────────────────────────────────────────────────────
# Phase 2: Songbird (Network + Discovery)
#───────────────────────────────────────────────────────────────────────────────
echo "Phase 2: Songbird (Network)..."
SONGBIRD_PORT="${SONGBIRD_PORT:-3492}"
export SONGBIRD_SOCKET="$SOCKET_DIR/songbird.sock"
export SONGBIRD_SECURITY_PROVIDER="$BEARDOG_SOCKET"
export CRYPTO_PROVIDER_SOCKET="$BEARDOG_SOCKET"
export BIOMEOS_BIND_ALL=true
export BIND_ADDRESS="::"
export BIOMEOS_SOVEREIGN=true
export BIOMEOS_DARK_FOREST=true
export BIOMEOS_NO_EXTERNAL_DEPS=true
export SONGBIRD_ONION_ENABLED=true
export SONGBIRD_MESH_ENABLED=true

"$PRIMAL_DIR/songbird" server \
    --port "$SONGBIRD_PORT" \
    --socket "$SONGBIRD_SOCKET" \
    --verbose > /tmp/songbird_usb.log 2>&1 &
SONGBIRD_PID=$!
echo "  PID: $SONGBIRD_PID"

tries=0
while [ ! -S "$SONGBIRD_SOCKET" ] && [ $tries -lt 12 ]; do
    sleep 1
    tries=$((tries + 1))
done

if [ -S "$SONGBIRD_SOCKET" ]; then
    echo "  Songbird operational (TCP :$SONGBIRD_PORT + IPC)"
else
    echo "  Songbird socket pending after ${tries}s"
fi

#───────────────────────────────────────────────────────────────────────────────
# Phase 2.1: LAN JSON-RPC Bridge (TCP :3493 -> Unix socket)
# Exposes full Songbird API to LAN peers for mesh linking
#───────────────────────────────────────────────────────────────────────────────
SONGBIRD_RPC_PORT="${SONGBIRD_RPC_PORT:-3493}"
if command -v socat >/dev/null 2>&1; then
    # Kill stale socat bridges
    pkill -f "socat TCP-LISTEN:${SONGBIRD_RPC_PORT}" 2>/dev/null || true
    sleep 0.5
    
    socat "TCP-LISTEN:${SONGBIRD_RPC_PORT},bind=0.0.0.0,reuseaddr,fork" \
        "UNIX-CONNECT:${SONGBIRD_SOCKET}" > /tmp/socat_rpc_bridge.log 2>&1 &
    SOCAT_PID=$!
    echo "  LAN RPC bridge: TCP :${SONGBIRD_RPC_PORT} -> songbird.sock (PID: $SOCAT_PID)"
else
    echo "  LAN RPC bridge: socat not installed (apt install socat)"
    echo "  LAN peers will need socat or direct socket access for full RPC"
    SOCAT_PID=""
fi

#───────────────────────────────────────────────────────────────────────────────
# RPC Helper
#───────────────────────────────────────────────────────────────────────────────
rpc_songbird() {
    local method=$1 params=$2
    echo "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":$params,\"id\":1}" | \
        nc -U "$SONGBIRD_SOCKET" -w 8 -q 2 2>/dev/null
}

#───────────────────────────────────────────────────────────────────────────────
# Phase 2a: Sovereign Onion Service
#───────────────────────────────────────────────────────────────────────────────
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

# Check seeds
[ -f "$USB_ROOT/.family.seed" ] && echo "  Family seed: $USB_ROOT/.family.seed"
[ -f "$USB_ROOT/.beacon.seed" ] && echo "  Beacon seed: $USB_ROOT/.beacon.seed"
[ -f "$USB_ROOT/.lineage.seed" ] && echo "  Lineage seed: $USB_ROOT/.lineage.seed"

# Verify BearDog crypto pipeline
BD_HEALTH=$(echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' | \
    nc -U "$BEARDOG_SOCKET" -w 3 -q 1 2>/dev/null || true)
if echo "$BD_HEALTH" | grep -q '"healthy"'; then
    echo "  Crypto pipeline: verified"
else
    echo "  Crypto pipeline: pending"
fi

# Generate encrypted beacon
BEACON_RESP=$(rpc_songbird "birdsong.generate_encrypted_beacon" "{\"node_id\":\"$NODE_ID\",\"capabilities\":[\"http\",\"discovery\",\"security\"]}")
if echo "$BEACON_RESP" | grep -q '"encrypted_beacon"'; then
    echo "  Dark Forest: beacon ready"
else
    echo "  Dark Forest: beacon pending"
fi

export BIOMEOS_FAMILY_SEED="$USB_ROOT/.family.seed"
export BIOMEOS_BEACON_MODE="dark_forest"

#───────────────────────────────────────────────────────────────────────────────
# Phase 3b: LAN Beacon Discovery (Dark Forest)
#───────────────────────────────────────────────────────────────────────────────
echo ""
echo "Phase 3b: LAN Dark Forest Link..."

# Read known beacons for family gate endpoints
KNOWN_BEACONS="$USB_ROOT/.known_beacons.json"
if [ -f "$KNOWN_BEACONS" ]; then
    # Extract known gate LAN endpoints
    GATE_ENDPOINTS=$(python3 -c "
import json
with open('$KNOWN_BEACONS') as f:
    d = json.load(f)
for name, member in d.get('family_members', {}).items():
    ep = member.get('endpoints', {})
    lan = ep.get('lan', '')
    if lan and name != '$NODE_ID':
        print(f'{name}={lan}')
" 2>/dev/null)

    if [ -n "$GATE_ENDPOINTS" ]; then
        echo "  Known family gates:"
        MY_LAN_IP=$(hostname -I | awk '{print $1}')
        echo "$GATE_ENDPOINTS" | while IFS='=' read -r gate_name gate_addr; do
            echo -n "    $gate_name ($gate_addr): "
            gate_host="${gate_addr%:*}"
            gate_port="${gate_addr##*:}"
            # RPC port is gate_port + 1 (3492 -> 3493 convention)
            gate_rpc_port=$((gate_port + 1))
            
            # Try TCP connection to RPC bridge first, then HTTP port
            if nc -z -w 2 "$gate_host" "$gate_rpc_port" 2>/dev/null; then
                echo -n "RPC reachable → "
                
                # Get remote gate's lineage to verify family match
                REMOTE_LINEAGE=$(echo "{\"jsonrpc\":\"2.0\",\"method\":\"birdsong.get_lineage\",\"params\":{},\"id\":1}" | \
                    nc -w 5 "$gate_host" "$gate_rpc_port" 2>/dev/null || true)
                REMOTE_FAMILY=$(echo "$REMOTE_LINEAGE" | python3 -c "import sys,json; print(json.load(sys.stdin)['result']['family_id'])" 2>/dev/null || echo "?")
                
                if [ "$REMOTE_FAMILY" = "$FAMILY_ID" ]; then
                    echo -n "family match → "
                    
                    # Announce ourselves to the remote mesh
                    rpc_songbird "mesh.announce" "{\"node_id\":\"$NODE_ID\",\"address\":\"$MY_LAN_IP:$SONGBIRD_PORT\",\"capabilities\":[\"compute\",\"discovery\"]}" >/dev/null 2>&1
                    
                    # Generate Dark Forest verification proof
                    SEED_B64=$(base64 -w0 "$USB_ROOT/.family.seed")
                    VERIFY_KEY=$(echo "{\"jsonrpc\":\"2.0\",\"method\":\"crypto.hmac_sha256\",\"params\":{\"key\":\"$SEED_B64\",\"data\":\"$(echo -n 'dark-forest-verify' | base64)\"},\"id\":1}" | \
                        nc -U "$BEARDOG_SOCKET" -w 3 -q 1 2>/dev/null)
                    
                    if echo "$VERIFY_KEY" | grep -q '"mac"'; then
                        LOCAL_MAC=$(echo "$VERIFY_KEY" | python3 -c "import sys,json; print(json.load(sys.stdin)['result']['mac'])" 2>/dev/null)
                        echo "linked (Dark Forest proof: ${LOCAL_MAC:0:12}...)"
                    else
                        echo "linked (crypto pending)"
                    fi
                else
                    echo "family mismatch ($REMOTE_FAMILY != $FAMILY_ID)"
                fi
            elif nc -z -w 2 "$gate_host" "$gate_port" 2>/dev/null; then
                echo "HTTP reachable (no RPC bridge — link on next startup)"
            else
                echo "offline"
            fi
        done
    else
        echo "  No known family gates (will discover via broadcast)"
    fi
else
    echo "  No .known_beacons.json (standalone mode)"
fi

# Also try LAN broadcast discovery via Songbird
DISCOVER_RESP=$(rpc_songbird "discovery.scan" "{\"timeout_ms\":3000}")
if echo "$DISCOVER_RESP" | grep -q '"peers"\|"result"'; then
    PEER_COUNT=$(echo "$DISCOVER_RESP" | python3 -c "import sys,json; r=json.load(sys.stdin).get('result',{}); print(len(r.get('peers',r.get('discovered',[]))))" 2>/dev/null || echo "0")
    echo "  LAN scan: found $PEER_COUNT peer(s)"
else
    echo "  LAN scan: no response (may need time)"
fi

#───────────────────────────────────────────────────────────────────────────────
# Phase 4: Health Validation
#───────────────────────────────────────────────────────────────────────────────
echo ""
echo "Phase 4: Validation..."
BD_OK=false; SB_OK=false

echo "$BD_HEALTH" | grep -q '"healthy"' && BD_OK=true
$BD_OK && echo "  BearDog:  OK" || echo "  BearDog:  FAIL"

SB_RESP=$(rpc_songbird "health" "{}")
echo "$SB_RESP" | grep -q '"healthy"' && SB_OK=true
$SB_OK && echo "  Songbird: OK" || echo "  Songbird: FAIL"

echo ""
echo "═══════════════════════════════════════════════════════════════"
if $BD_OK && $SB_OK; then
    echo "  LiveSpore USB - SOVEREIGN TOWER OPERATIONAL"
else
    echo "  LiveSpore USB - PARTIAL (some services pending)"
fi
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Sockets:"
ls -lh "$SOCKET_DIR"/*.sock 2>/dev/null || echo "  (checking...)"
echo ""
echo "Network:"
echo "  Songbird HTTP: :$SONGBIRD_PORT"
echo "  Songbird RPC:  :$SONGBIRD_RPC_PORT (LAN JSON-RPC bridge)"
echo "  BearDog:       $BEARDOG_SOCKET"
[ -n "$ONION_ADDR" ] && echo "  Onion:         $ONION_ADDR:$SONGBIRD_PORT"
[ -n "$PUB_ADDR" ] && echo "  Public STUN:   $PUB_ADDR"
echo ""
echo "PIDs: BearDog=$BEARDOG_PID, Songbird=$SONGBIRD_PID"
echo "Logs: /tmp/beardog_usb.log, /tmp/songbird_usb.log"
echo ""
echo "Mesh:"
MESH_STATUS=$(rpc_songbird "mesh.peers" "{}" 2>/dev/null)
if echo "$MESH_STATUS" | grep -q '"result"'; then
    echo "$MESH_STATUS" | python3 -c "
import sys,json
r = json.load(sys.stdin).get('result',{})
peers = r.get('peers', [])
if peers:
    for p in peers:
        print(f'  → {p.get(\"node_id\",\"?\")} at {p.get(\"address\",\"?\")}')
else:
    print('  No peers yet (waiting for family gates)')
" 2>/dev/null
else
    echo "  Pending (start another gate to form mesh)"
fi
echo ""
