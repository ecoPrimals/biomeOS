#!/usr/bin/env bash
#
# 🌲 Dark Forest LAN Discovery
#
# This script enables encrypted discovery between family members on a LAN.
# Each node broadcasts an encrypted beacon that only family members can read.
#
# MODES:
#   broadcast - Broadcast encrypted beacons (server mode)
#   discover  - Listen for and decrypt family beacons (client mode)
#
# USAGE:
#   USB LiveSpore (node-alpha): ./dark_forest_discovery.sh broadcast
#   Another machine (client):   ./dark_forest_discovery.sh discover <IP>
#

set -euo pipefail

MODE="${1:-help}"
REMOTE_IP="${2:-}"

# Detect spore root
if [[ -f "/.family.seed" ]]; then
    SPORE_ROOT="/"
elif [[ -f "./.family.seed" ]]; then
    SPORE_ROOT="."
elif [[ -f "../.family.seed" ]]; then
    SPORE_ROOT=".."
else
    # Try common USB paths
    for P in /media/*/biomeOS1/biomeOS /media/*/*/biomeOS; do
        if [[ -f "$P/.family.seed" ]]; then
            SPORE_ROOT="$P"
            break
        fi
    done
fi

if [[ -z "${SPORE_ROOT:-}" || ! -f "${SPORE_ROOT}/.family.seed" ]]; then
    echo "❌ Cannot find .family.seed - run from spore root or mount USB"
    exit 1
fi

FAMILY_SEED="$SPORE_ROOT/.family.seed"
BEARDOG_BIN="$SPORE_ROOT/primals/beardog"
BEARDOG_SOCKET="/tmp/beardog-dark-forest.sock"
BEACON_PORT=7777
FAMILY_ID="${FAMILY_ID:-nat0}"
NODE_ID="${NODE_ID:-$(hostname)}"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m'

log() { echo -e "${CYAN}[$(date +%H:%M:%S)]${NC} $1"; }
ok()  { echo -e "${GREEN}✅ $1${NC}"; }
err() { echo -e "${RED}❌ $1${NC}"; }
warn(){ echo -e "${YELLOW}⚠️  $1${NC}"; }

# Ensure BearDog is running
ensure_beardog() {
    if [[ ! -S "$BEARDOG_SOCKET" ]]; then
        log "Starting BearDog..."
        export FAMILY_ID BEARDOG_FAMILY_ID="$FAMILY_ID" BEARDOG_NODE_ID="$NODE_ID"
        
        # Handle non-executable (FAT32)
        if [[ ! -x "$BEARDOG_BIN" ]]; then
            cp "$BEARDOG_BIN" /tmp/beardog_exec
            chmod +x /tmp/beardog_exec
            BEARDOG_BIN=/tmp/beardog_exec
        fi
        
        nohup "$BEARDOG_BIN" server --socket "$BEARDOG_SOCKET" --family-id "$FAMILY_ID" > /tmp/beardog-discovery.log 2>&1 &
        sleep 3
        
        if [[ ! -S "$BEARDOG_SOCKET" ]]; then
            err "Failed to start BearDog"
            exit 1
        fi
    fi
    ok "BearDog running at $BEARDOG_SOCKET"
}

# Call BearDog JSON-RPC
beardog() {
    echo "$1" | nc -N -U "$BEARDOG_SOCKET" 2>/dev/null
}

# Generate encrypted beacon
generate_beacon() {
    local SEED_B64=$(base64 -w0 "$FAMILY_SEED")
    
    # Derive broadcast key
    local KEY_RESP=$(beardog "{\"jsonrpc\":\"2.0\",\"method\":\"genetic.derive_lineage_key\",\"params\":{\"our_family_id\":\"$FAMILY_ID\",\"peer_family_id\":\"broadcast\",\"context\":\"birdsong-dark-forest-v1\",\"lineage_seed\":\"$SEED_B64\"},\"id\":1}")
    local BROADCAST_KEY=$(echo "$KEY_RESP" | grep -oP '"key":"\K[^"]+')
    
    if [[ -z "$BROADCAST_KEY" ]]; then
        err "Failed to derive broadcast key"
        return 1
    fi
    
    # Create beacon plaintext
    local TIMESTAMP=$(date +%s)
    local BEACON_DATA=$(cat << EOF
{
    "family_hash": "$(echo -n "$FAMILY_ID" | sha256sum | cut -c1-16)",
    "node_id": "$NODE_ID",
    "timestamp": $TIMESTAMP,
    "socket_path": "$BEARDOG_SOCKET",
    "capabilities": ["crypto", "lineage", "relay"]
}
EOF
)
    local BEACON_B64=$(echo -n "$BEACON_DATA" | base64 -w0)
    
    # Encrypt
    local ENC_RESP=$(beardog "{\"jsonrpc\":\"2.0\",\"method\":\"crypto.chacha20_poly1305_encrypt\",\"params\":{\"key\":\"$BROADCAST_KEY\",\"plaintext\":\"$BEACON_B64\"},\"id\":2}")
    
    local CIPHERTEXT=$(echo "$ENC_RESP" | grep -oP '"ciphertext":"\K[^"]+')
    local NONCE=$(echo "$ENC_RESP" | grep -oP '"nonce":"\K[^"]+')
    local TAG=$(echo "$ENC_RESP" | grep -oP '"tag":"\K[^"]+')
    
    if [[ -z "$CIPHERTEXT" ]]; then
        err "Failed to encrypt beacon"
        return 1
    fi
    
    # Output encrypted beacon (JSON format)
    echo "{\"v\":1,\"c\":\"$CIPHERTEXT\",\"n\":\"$NONCE\",\"t\":\"$TAG\"}"
}

# Try to decrypt beacon
decrypt_beacon() {
    local BEACON="$1"
    local SEED_B64=$(base64 -w0 "$FAMILY_SEED")
    
    # Parse beacon
    local CIPHERTEXT=$(echo "$BEACON" | grep -oP '"c":"\K[^"]+')
    local NONCE=$(echo "$BEACON" | grep -oP '"n":"\K[^"]+')
    local TAG=$(echo "$BEACON" | grep -oP '"t":"\K[^"]+')
    
    # Derive broadcast key
    local KEY_RESP=$(beardog "{\"jsonrpc\":\"2.0\",\"method\":\"genetic.derive_lineage_key\",\"params\":{\"our_family_id\":\"$FAMILY_ID\",\"peer_family_id\":\"broadcast\",\"context\":\"birdsong-dark-forest-v1\",\"lineage_seed\":\"$SEED_B64\"},\"id\":10}")
    local BROADCAST_KEY=$(echo "$KEY_RESP" | grep -oP '"key":"\K[^"]+')
    
    # Try to decrypt
    local DEC_RESP=$(beardog "{\"jsonrpc\":\"2.0\",\"method\":\"crypto.chacha20_poly1305_decrypt\",\"params\":{\"key\":\"$BROADCAST_KEY\",\"ciphertext\":\"$CIPHERTEXT\",\"nonce\":\"$NONCE\",\"tag\":\"$TAG\"},\"id\":11}")
    
    if echo "$DEC_RESP" | grep -q '"error"'; then
        return 1  # Not family
    fi
    
    # Decrypted!
    local PLAINTEXT_B64=$(echo "$DEC_RESP" | grep -oP '"plaintext":"\K[^"]+')
    echo "$PLAINTEXT_B64" | base64 -d
}

# BROADCAST MODE
mode_broadcast() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🌲 DARK FOREST BROADCAST MODE"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "  Spore Root: $SPORE_ROOT"
    echo "  Node ID:    $NODE_ID"
    echo "  Family:     $FAMILY_ID"
    echo "  Port:       $BEACON_PORT"
    echo ""
    
    ensure_beardog
    
    log "Generating encrypted beacon..."
    BEACON=$(generate_beacon)
    
    if [[ -z "$BEACON" ]]; then
        err "Failed to generate beacon"
        exit 1
    fi
    
    ok "Beacon ready (encrypted - reveals nothing to outsiders)"
    echo ""
    echo "  Beacon size: $(echo -n "$BEACON" | wc -c) bytes"
    echo ""
    
    # Start beacon server
    log "Broadcasting on port $BEACON_PORT..."
    log "Press Ctrl+C to stop"
    echo ""
    
    # Use netcat to serve beacon on UDP
    while true; do
        # Regenerate beacon each time (fresh timestamp)
        BEACON=$(generate_beacon)
        echo "$BEACON" | nc -u -l -p "$BEACON_PORT" -w 5 2>/dev/null || true
        sleep 1
    done
}

# DISCOVER MODE
mode_discover() {
    if [[ -z "$REMOTE_IP" ]]; then
        err "Usage: $0 discover <IP>"
        exit 1
    fi
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🌲 DARK FOREST DISCOVERY MODE"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "  Target: $REMOTE_IP:$BEACON_PORT"
    echo "  Family: $FAMILY_ID"
    echo ""
    
    ensure_beardog
    
    log "Listening for encrypted beacons..."
    
    # Try to receive beacon
    BEACON=$(timeout 10 nc -u "$REMOTE_IP" "$BEACON_PORT" -w 5 2>/dev/null || echo "")
    
    if [[ -z "$BEACON" ]]; then
        warn "No beacon received (timeout)"
        exit 1
    fi
    
    log "Received beacon, attempting decryption..."
    
    PLAINTEXT=$(decrypt_beacon "$BEACON" 2>/dev/null || echo "")
    
    if [[ -z "$PLAINTEXT" ]]; then
        err "DECRYPTION FAILED - Not family (or wrong seed)"
        echo ""
        echo "  🌲 Dark Forest: The beacon remains opaque"
        echo "     Only true family members can see inside"
        exit 1
    fi
    
    ok "DECRYPTION SUCCEEDED - Family member found!"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "DISCOVERED PEER:"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "$PLAINTEXT" | jq . 2>/dev/null || echo "$PLAINTEXT"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    # Extract node info
    PEER_NODE=$(echo "$PLAINTEXT" | grep -oP '"node_id":\s*"\K[^"]+' || echo "unknown")
    
    log "Proceeding to lineage verification with $PEER_NODE..."
    
    # Generate proof
    SEED_B64=$(base64 -w0 "$FAMILY_SEED")
    PROOF_RESP=$(beardog "{\"jsonrpc\":\"2.0\",\"method\":\"genetic.generate_lineage_proof\",\"params\":{\"our_family_id\":\"$FAMILY_ID\",\"peer_family_id\":\"$PEER_NODE\",\"lineage_seed\":\"$SEED_B64\"},\"id\":20}")
    PROOF=$(echo "$PROOF_RESP" | grep -oP '"proof":"\K[^"]+')
    
    ok "Lineage proof generated"
    echo ""
    echo "  Ready to establish secure session with $PEER_NODE"
    echo "  Proof: ${PROOF:0:32}..."
}

# TEST MODE - local validation
mode_test() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🌲 DARK FOREST LOCAL TEST"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    ensure_beardog
    
    log "Generating beacon..."
    BEACON=$(generate_beacon)
    ok "Beacon generated"
    echo ""
    
    log "Attempting family decryption..."
    PLAINTEXT=$(decrypt_beacon "$BEACON")
    
    if [[ -n "$PLAINTEXT" ]]; then
        ok "Family decryption succeeded!"
        echo ""
        echo "$PLAINTEXT" | jq . 2>/dev/null || echo "$PLAINTEXT"
    else
        err "Decryption failed"
        exit 1
    fi
    
    echo ""
    ok "Dark Forest protocol validated locally!"
}

# HELP
mode_help() {
    cat << EOF

🌲 Dark Forest LAN Discovery

USAGE:
    $0 <mode> [options]

MODES:
    broadcast     Broadcast encrypted beacon (USB spore server)
    discover <IP> Discover and connect to family member
    test          Local validation test

ENVIRONMENT:
    FAMILY_ID     Family identifier (default: nat0)
    NODE_ID       This node's identifier (default: hostname)
    BEACON_PORT   UDP port for beacons (default: 7777)

EXAMPLES:
    # On USB LiveSpore (node-alpha):
    $0 broadcast
    
    # On another machine with same .family.seed:
    $0 discover 192.168.1.100
    
    # Local test:
    $0 test

EOF
}

# Main
case "$MODE" in
    broadcast) mode_broadcast ;;
    discover)  mode_discover ;;
    test)      mode_test ;;
    *)         mode_help ;;
esac

