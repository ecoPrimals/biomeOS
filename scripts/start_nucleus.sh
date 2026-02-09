#!/bin/bash
#═══════════════════════════════════════════════════════════════════════════════
# NUCLEUS Unified Startup Script
# Standard: PRIMAL_DEPLOYMENT_STANDARD v1.0
#
# Works identically on all architectures:
#   - x86_64 (Intel/AMD Linux)
#   - aarch64 (ARM64 Linux, Raspberry Pi, Pixel 8a)
#
# Usage:
#   ./start_nucleus.sh [atomic]
#
# Atomics:
#   tower   - BearDog + Songbird (default)
#   node    - Tower + Toadstool
#   nest    - Tower + NestGate + Squirrel
#   full    - All primals
#
#═══════════════════════════════════════════════════════════════════════════════

set -e

# Configuration
ATOMIC="${1:-sovereign}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ARCH="$(uname -m)"

# Environment - evolved standard defaults
export NODE_ID="${NODE_ID:-gate}"
export RUST_LOG="${RUST_LOG:-info}"
export BIOMEOS_ROOT="${BIOMEOS_ROOT:-$PROJECT_ROOT}"

#───────────────────────────────────────────────────────────────────────────────
# Family ID Derivation from Mitochondrial Seed
# Standard: hex(family.seed[0..8]) = 16 hex chars
# This is the canonical derivation per family_discovery.rs
# NO hardcoded tags — identity flows from the seed file
#───────────────────────────────────────────────────────────────────────────────
derive_family_id() {
    local seed_file=""
    for candidate in \
        "$BIOMEOS_ROOT/.family.seed" \
        "$PROJECT_ROOT/.family.seed" \
        "$PROJECT_ROOT/livespore-usb/.family.seed"; do
        if [ -f "$candidate" ]; then
            seed_file="$candidate"
            break
        fi
    done

    if [ -n "$seed_file" ]; then
        # Canonical derivation: hex of first 8 bytes of seed
        local derived
        derived=$(xxd -p -l 8 "$seed_file" | tr -d '\n')
        if [ ${#derived} -eq 16 ]; then
            echo "$derived"
            return 0
        fi
    fi
    echo ""
    return 1
}

if [ -n "$FAMILY_ID" ]; then
    # Caller provided FAMILY_ID — validate it's not the old tag format
    if [ ${#FAMILY_ID} -lt 16 ]; then
        echo "⚠️  FAMILY_ID '$FAMILY_ID' looks like an old tag (${#FAMILY_ID} chars)"
        echo "   Evolved standard derives from .family.seed (16 hex chars)"
        SEED_DERIVED=$(derive_family_id)
        if [ -n "$SEED_DERIVED" ]; then
            echo "   Overriding with seed-derived: $SEED_DERIVED"
            export FAMILY_ID="$SEED_DERIVED"
        fi
    fi
else
    SEED_DERIVED=$(derive_family_id)
    if [ -n "$SEED_DERIVED" ]; then
        export FAMILY_ID="$SEED_DERIVED"
    else
        echo "❌ No .family.seed found and FAMILY_ID not set"
        exit 1
    fi
fi

#───────────────────────────────────────────────────────────────────────────────
# Socket Directory Resolution (5-Tier per PRIMAL_DEPLOYMENT_STANDARD)
#───────────────────────────────────────────────────────────────────────────────
resolve_socket_dir() {
    if [ -n "$BIOMEOS_SOCKET_DIR" ]; then
        echo "$BIOMEOS_SOCKET_DIR"
    elif [ -n "$XDG_RUNTIME_DIR" ]; then
        echo "$XDG_RUNTIME_DIR/biomeos"
    elif [ -d "/run/user/$(id -u)" ]; then
        echo "/run/user/$(id -u)/biomeos"
    elif [ -d "/data/local/tmp" ]; then
        echo "/data/local/tmp/biomeos"  # Android
    else
        echo "/tmp/biomeos"
    fi
}

#───────────────────────────────────────────────────────────────────────────────
# Primal Directory Resolution
#───────────────────────────────────────────────────────────────────────────────
resolve_primal_dir() {
    # Check livespore-usb first (architecture-specific)
    local livespore="$SCRIPT_DIR/../livespore-usb/$ARCH/primals"
    if [ -d "$livespore" ]; then
        echo "$livespore"
        return
    fi
    
    # Check generic primals directory
    local generic="$SCRIPT_DIR/../primals"
    if [ -d "$generic" ]; then
        echo "$generic"
        return
    fi
    
    # Check relative to pixel8a-deploy
    local pixel="$SCRIPT_DIR/../pixel8a-deploy/primals"
    if [ -d "$pixel" ]; then
        echo "$pixel"
        return
    fi
    
    echo "ERROR: Could not find primals directory" >&2
    exit 1
}

SOCKET_DIR="$(resolve_socket_dir)"
PRIMAL_DIR="$(resolve_primal_dir)"

#───────────────────────────────────────────────────────────────────────────────
# Banner
#───────────────────────────────────────────────────────────────────────────────
echo "═══════════════════════════════════════════════════════════════"
echo "🧬 NUCLEUS Startup - Evolved Genetic Standard v2.0"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Architecture:  $ARCH"
echo "Atomic:        $ATOMIC"
echo "Family ID:     $FAMILY_ID"
echo "Node ID:       $NODE_ID"
echo "BIOMEOS_ROOT:  $BIOMEOS_ROOT"
echo "Socket Dir:    $SOCKET_DIR"
echo "Primal Dir:    $PRIMAL_DIR"
echo ""

# Create socket directory
mkdir -p "$SOCKET_DIR"

#───────────────────────────────────────────────────────────────────────────────
# Start BearDog (all atomics need this)
#───────────────────────────────────────────────────────────────────────────────
start_beardog() {
    echo "🐻 Starting BearDog (Security)..."
    export BEARDOG_SOCKET="$SOCKET_DIR/beardog.sock"
    
    # Kill stale
    pkill -f "beardog server" 2>/dev/null || true
    rm -f "$BEARDOG_SOCKET" 2>/dev/null
    sleep 1
    
    "$PRIMAL_DIR/beardog" server \
        --socket "$BEARDOG_SOCKET" > /tmp/beardog_nucleus.log 2>&1 &
    BEARDOG_PID=$!
    echo "  PID: $BEARDOG_PID"
    
    # Wait for socket (BearDog needs ~5s to init crypto)
    local tries=0
    while [ ! -S "$BEARDOG_SOCKET" ] && [ $tries -lt 10 ]; do
        sleep 1
        tries=$((tries + 1))
    done
    
    if [ -S "$BEARDOG_SOCKET" ]; then
        echo "  ✅ BearDog ready"
    else
        echo "  ❌ BearDog socket not found after ${tries}s"
        tail -5 /tmp/beardog_nucleus.log
        exit 1
    fi
}

#───────────────────────────────────────────────────────────────────────────────
# Start Songbird (all atomics need this)
#───────────────────────────────────────────────────────────────────────────────
start_songbird() {
    local PORT="${SONGBIRD_PORT:-3492}"
    echo "🐦 Starting Songbird (Network, port $PORT)..."
    export SONGBIRD_SOCKET="$SOCKET_DIR/songbird.sock"
    export SONGBIRD_SECURITY_PROVIDER="$BEARDOG_SOCKET"
    export BIOMEOS_BIND_ALL=true
    export BIND_ADDRESS="::"
    
    # Kill stale
    pkill -f "songbird server" 2>/dev/null || true
    rm -f "$SONGBIRD_SOCKET" 2>/dev/null
    sleep 1
    
    "$PRIMAL_DIR/songbird" server \
        --port "$PORT" \
        --socket "$SONGBIRD_SOCKET" \
        --verbose > /tmp/songbird_nucleus.log 2>&1 &
    SONGBIRD_PID=$!
    echo "  PID: $SONGBIRD_PID"
    
    # Wait for socket
    local tries=0
    while [ ! -S "$SONGBIRD_SOCKET" ] && [ $tries -lt 10 ]; do
        sleep 1
        tries=$((tries + 1))
    done
    
    if [ -S "$SONGBIRD_SOCKET" ]; then
        echo "  ✅ Songbird ready (TCP :$PORT + IPC)"
    else
        echo "  ⚠️  Songbird socket pending after ${tries}s"
        tail -5 /tmp/songbird_nucleus.log
    fi
}

#───────────────────────────────────────────────────────────────────────────────
# Start Toadstool (Node Atomic)
#───────────────────────────────────────────────────────────────────────────────
start_toadstool() {
    echo "🍄 Starting Toadstool (Compute)..."
    export TOADSTOOL_SOCKET="$SOCKET_DIR/toadstool.sock"
    
    pkill -f "toadstool daemon" 2>/dev/null || true
    rm -f "$TOADSTOOL_SOCKET" 2>/dev/null
    
    "$PRIMAL_DIR/toadstool" daemon \
        --socket "$TOADSTOOL_SOCKET" \
        --register > /tmp/toadstool_nucleus.log 2>&1 &
    TOADSTOOL_PID=$!
    echo "  PID: $TOADSTOOL_PID"
    
    local tries=0
    while [ ! -S "$TOADSTOOL_SOCKET" ] && [ $tries -lt 10 ]; do
        sleep 1
        tries=$((tries + 1))
    done
    
    if [ -S "$TOADSTOOL_SOCKET" ]; then
        echo "  ✅ Toadstool ready"
    else
        echo "  ⚠️  Toadstool socket pending"
    fi
}

#───────────────────────────────────────────────────────────────────────────────
# Start NestGate (Nest Atomic)
#───────────────────────────────────────────────────────────────────────────────
start_nestgate() {
    echo "🦅 Starting NestGate (Storage)..."
    export NESTGATE_SOCKET="$SOCKET_DIR/nestgate.sock"
    export NESTGATE_JWT_SECRET="${NESTGATE_JWT_SECRET:-$(head -c 48 /dev/urandom | base64)}"
    
    pkill -f "nestgate service" 2>/dev/null || true
    rm -f "$NESTGATE_SOCKET" 2>/dev/null
    
    "$PRIMAL_DIR/nestgate" service start \
        --socket "$NESTGATE_SOCKET" \
        --daemon > /tmp/nestgate_nucleus.log 2>&1 &
    NESTGATE_PID=$!
    echo "  PID: $NESTGATE_PID"
    
    local tries=0
    while [ ! -S "$NESTGATE_SOCKET" ] && [ $tries -lt 10 ]; do
        sleep 1
        tries=$((tries + 1))
    done
    
    if [ -S "$NESTGATE_SOCKET" ]; then
        echo "  ✅ NestGate ready"
    else
        echo "  ⚠️  NestGate socket pending"
    fi
}

#───────────────────────────────────────────────────────────────────────────────
# Start Squirrel (Nest Atomic)
#───────────────────────────────────────────────────────────────────────────────
start_squirrel() {
    echo "🐿️ Starting Squirrel (AI)..."
    export SQUIRREL_SOCKET="$SOCKET_DIR/squirrel.sock"
    export NEURAL_API_SOCKET="$SOCKET_DIR/neural-api.sock"
    
    pkill -f "squirrel server" 2>/dev/null || true
    rm -f "$SQUIRREL_SOCKET" "$NEURAL_API_SOCKET" 2>/dev/null
    
    "$PRIMAL_DIR/squirrel" server > /tmp/squirrel_nucleus.log 2>&1 &
    SQUIRREL_PID=$!
    echo "  PID: $SQUIRREL_PID"
    
    sleep 3
    echo "  ✅ Squirrel started"
}

#───────────────────────────────────────────────────────────────────────────────
# Verify Dark Forest Beacon (Sovereign mode)
#───────────────────────────────────────────────────────────────────────────────
verify_dark_forest() {
    echo "🌲 Verifying Dark Forest beacon..."
    
    # Locate seeds
    local SEED_FILE=""
    for candidate in \
        "$BIOMEOS_ROOT/.family.seed" \
        "$PROJECT_ROOT/.family.seed" \
        "$PROJECT_ROOT/livespore-usb/.family.seed"; do
        if [ -f "$candidate" ]; then
            SEED_FILE="$candidate"
            break
        fi
    done
    
    if [ -z "$SEED_FILE" ]; then
        echo "  ❌ No .family.seed found — Dark Forest beacon disabled"
        return 1
    fi
    echo "  Family seed: $SEED_FILE"
    
    # Check beacon seed
    local BEACON_SEED=""
    for candidate in "$BIOMEOS_ROOT/.beacon.seed" "$PROJECT_ROOT/.beacon.seed"; do
        if [ -f "$candidate" ]; then
            BEACON_SEED="$candidate"
            break
        fi
    done
    [ -n "$BEACON_SEED" ] && echo "  Beacon seed: $BEACON_SEED" || echo "  ⚠️  No .beacon.seed (using family seed)"
    
    # Check lineage seed
    local LINEAGE_SEED=""
    for candidate in "$BIOMEOS_ROOT/.lineage.seed" "$PROJECT_ROOT/.lineage.seed"; do
        if [ -f "$candidate" ]; then
            LINEAGE_SEED="$candidate"
            break
        fi
    done
    [ -n "$LINEAGE_SEED" ] && echo "  Lineage seed: $LINEAGE_SEED" || echo "  ⚠️  No .lineage.seed (derive with BearDog)"
    
    # Verify BearDog crypto via nc (universally available)
    local RESPONSE
    RESPONSE=$(echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' | \
        nc -U "$BEARDOG_SOCKET" -w 3 -q 1 2>/dev/null || true)
    
    if echo "$RESPONSE" | grep -q '"healthy"'; then
        echo "  ✅ BearDog crypto pipeline verified"
    else
        echo "  ⚠️  BearDog health check pending"
    fi
    
    # Export beacon environment
    export BIOMEOS_FAMILY_SEED="$SEED_FILE"
    export BIOMEOS_BEACON_MODE="dark_forest"
    
    echo "  ✅ Dark Forest beacon environment ready"
}

#───────────────────────────────────────────────────────────────────────────────
# Atomic Deployment Patterns
#───────────────────────────────────────────────────────────────────────────────
case "$ATOMIC" in
    tower)
        echo "═══════════════════════════════════════════════════════════════"
        echo "🏰 TOWER ATOMIC (BearDog + Songbird)"
        echo "═══════════════════════════════════════════════════════════════"
        echo ""
        start_beardog
        start_songbird
        ;;
    node)
        echo "═══════════════════════════════════════════════════════════════"
        echo "💻 NODE ATOMIC (Tower + Toadstool)"
        echo "═══════════════════════════════════════════════════════════════"
        echo ""
        start_beardog
        start_songbird
        start_toadstool
        ;;
    nest)
        echo "═══════════════════════════════════════════════════════════════"
        echo "🪺 NEST ATOMIC (Tower + NestGate + Squirrel)"
        echo "═══════════════════════════════════════════════════════════════"
        echo ""
        start_beardog
        start_songbird
        start_nestgate
        start_squirrel
        ;;
    sovereign)
        echo "═══════════════════════════════════════════════════════════════"
        echo "🏰 SOVEREIGN TOWER (Dark Forest Beacon + IPv6 Dual-Stack)"
        echo "═══════════════════════════════════════════════════════════════"
        echo ""
        echo "Mode: Pure Rust ecoPrimals (no coturn, no tor daemon)"
        echo "Beacon: Dark Forest encrypted, family-only"
        echo "Bind: [::] dual-stack (IPv4 + IPv6)"
        echo ""
        # Sovereign-specific environment
        export BIOMEOS_SOVEREIGN=true
        export BIOMEOS_DARK_FOREST=true
        export BIOMEOS_NO_EXTERNAL_DEPS=true
        start_beardog
        start_songbird
        verify_dark_forest
        echo ""
        echo "🌲 Dark Forest beacon active"
        echo "   Only family members can decrypt and connect"
        ;;
    full)
        echo "═══════════════════════════════════════════════════════════════"
        echo "🧬 FULL NUCLEUS (All Primals)"
        echo "═══════════════════════════════════════════════════════════════"
        echo ""
        start_beardog
        start_songbird
        start_toadstool
        start_nestgate
        start_squirrel
        ;;
    *)
        echo "Unknown atomic: $ATOMIC"
        echo "Usage: $0 [tower|node|nest|full|sovereign]"
        exit 1
        ;;
esac

#───────────────────────────────────────────────────────────────────────────────
# Status Report
#───────────────────────────────────────────────────────────────────────────────
echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "✅ NUCLEUS $ATOMIC ATOMIC OPERATIONAL"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Sockets:"
ls -lh "$SOCKET_DIR"/*.sock 2>/dev/null || echo "  (checking...)"
echo ""
echo "Network:"
ss -tlnp 2>/dev/null | grep -E '3492|9900' || echo "  (no TCP listeners)"
echo ""
echo "Seeds:"
[ -f "$BIOMEOS_ROOT/.family.seed" ] && echo "  .family.seed    ✅" || echo "  .family.seed    ❌"
[ -f "$BIOMEOS_ROOT/.beacon.seed" ] && echo "  .beacon.seed    ✅" || echo "  .beacon.seed    ❌"
[ -f "$BIOMEOS_ROOT/.lineage.seed" ] && echo "  .lineage.seed   ✅" || echo "  .lineage.seed   ❌"
[ -f "$BIOMEOS_ROOT/.lineage.json" ] && echo "  .lineage.json   ✅" || echo "  .lineage.json   ❌"
echo ""
echo "Logs: /tmp/*_nucleus.log"
echo ""
