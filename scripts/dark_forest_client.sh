#!/usr/bin/env bash
#
# 🌲 Dark Forest Client
#
# Copy this script and your .family.seed to another computer to discover
# family members broadcasting on the LAN.
#
# USAGE:
#   ./dark_forest_client.sh <spore-ip> [port]
#
# REQUIREMENTS:
#   - .family.seed file (same as the spore you want to connect to)
#   - BearDog binary (or any machine with BearDog running)
#   - nc (netcat)
#

set -euo pipefail

REMOTE_IP="${1:-}"
BEACON_PORT="${2:-7777}"

if [[ -z "$REMOTE_IP" ]]; then
    echo "Usage: $0 <spore-ip> [port]"
    echo ""
    echo "Example: $0 192.168.1.100"
    exit 1
fi

# Find family seed
FAMILY_SEED="${FAMILY_SEED:-}"
for P in "./.family.seed" "../.family.seed" "$HOME/.family.seed"; do
    if [[ -f "$P" ]]; then
        FAMILY_SEED="$P"
        break
    fi
done

if [[ -z "$FAMILY_SEED" || ! -f "$FAMILY_SEED" ]]; then
    echo "❌ Cannot find .family.seed"
    echo "   Copy your family seed to current directory or set FAMILY_SEED env var"
    exit 1
fi

# Find BearDog socket or start one
BEARDOG_SOCKET="${BEARDOG_SOCKET:-/tmp/beardog-client.sock}"

if [[ ! -S "$BEARDOG_SOCKET" ]]; then
    echo "⚠️  No BearDog socket found at $BEARDOG_SOCKET"
    echo "   Looking for any running BearDog..."
    
    # Check for any beardog socket
    for SOCK in /tmp/beardog-*.sock; do
        if [[ -S "$SOCK" ]]; then
            BEARDOG_SOCKET="$SOCK"
            echo "   Found: $BEARDOG_SOCKET"
            break
        fi
    done
fi

if [[ ! -S "$BEARDOG_SOCKET" ]]; then
    echo "❌ No BearDog running. Start BearDog first or set BEARDOG_SOCKET"
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🌲 DARK FOREST CLIENT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  Target:  $REMOTE_IP:$BEACON_PORT"
echo "  Seed:    $FAMILY_SEED"
echo "  BearDog: $BEARDOG_SOCKET"
echo ""

# Receive beacon
echo "📡 Listening for beacon from $REMOTE_IP..."
BEACON=$(timeout 10 nc -u "$REMOTE_IP" "$BEACON_PORT" -w 5 2>/dev/null || echo "")

if [[ -z "$BEACON" ]]; then
    echo "❌ No beacon received (timeout)"
    exit 1
fi

echo "✅ Received beacon (${#BEACON} bytes)"
echo ""

# Parse beacon
CIPHERTEXT=$(echo "$BEACON" | grep -oP '"c":"\K[^"]+' || echo "")
NONCE=$(echo "$BEACON" | grep -oP '"n":"\K[^"]+' || echo "")
TAG=$(echo "$BEACON" | grep -oP '"t":"\K[^"]+' || echo "")

if [[ -z "$CIPHERTEXT" || -z "$NONCE" || -z "$TAG" ]]; then
    echo "❌ Invalid beacon format"
    exit 1
fi

# Derive broadcast key
SEED_B64=$(base64 -w0 "$FAMILY_SEED")
FAMILY_ID="${FAMILY_ID:-1894e909e454}"

echo "🔑 Deriving broadcast key..."
KEY_RESP=$(echo '{"jsonrpc":"2.0","method":"genetic.derive_lineage_key","params":{"our_family_id":"'"$FAMILY_ID"'","peer_family_id":"broadcast","context":"birdsong-dark-forest-v1","lineage_seed":"'"$SEED_B64"'"},"id":1}' | nc -N -U "$BEARDOG_SOCKET")
BROADCAST_KEY=$(echo "$KEY_RESP" | grep -oP '"key":"\K[^"]+' || echo "")

if [[ -z "$BROADCAST_KEY" ]]; then
    echo "❌ Failed to derive broadcast key"
    exit 1
fi

# Try to decrypt
echo "🔓 Attempting decryption..."
DEC_RESP=$(echo '{"jsonrpc":"2.0","method":"crypto.chacha20_poly1305_decrypt","params":{"key":"'"$BROADCAST_KEY"'","ciphertext":"'"$CIPHERTEXT"'","nonce":"'"$NONCE"'","tag":"'"$TAG"'"},"id":2}' | nc -N -U "$BEARDOG_SOCKET")

if echo "$DEC_RESP" | grep -q '"error"'; then
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "❌ DECRYPTION FAILED - NOT FAMILY"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "🌲 Dark Forest: The beacon remains opaque"
    echo "   Your seed does not match the broadcaster's lineage"
    exit 1
fi

# Success!
PLAINTEXT_B64=$(echo "$DEC_RESP" | grep -oP '"plaintext":"\K[^"]+')
PLAINTEXT=$(echo "$PLAINTEXT_B64" | base64 -d)

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ FAMILY MEMBER DISCOVERED!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "$PLAINTEXT" | jq . 2>/dev/null || echo "$PLAINTEXT"
echo ""

# Extract peer info
PEER_NODE=$(echo "$PLAINTEXT" | grep -oP '"node":"\K[^"]+' || echo "unknown")
PEER_SOCKET=$(echo "$PLAINTEXT" | grep -oP '"socket":"\K[^"]+' || echo "unknown")

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "NEXT STEPS:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "1. Generate lineage proof for peer verification:"
echo "   PROOF=\$(echo '{\"jsonrpc\":\"2.0\",\"method\":\"genetic.generate_lineage_proof\",\"params\":{\"our_family_id\":\"$FAMILY_ID\",\"peer_family_id\":\"$PEER_NODE\",\"lineage_seed\":\"'\"$SEED_B64\"'\"},\"id\":3}' | nc -N -U $BEARDOG_SOCKET)"
echo ""
echo "2. Send proof to peer for verification"
echo ""
echo "3. Derive session key for encrypted communication:"
echo "   SESSION_KEY=\$(echo '{\"jsonrpc\":\"2.0\",\"method\":\"genetic.derive_lineage_key\",\"params\":{\"our_family_id\":\"$FAMILY_ID\",\"peer_family_id\":\"$PEER_NODE\",\"context\":\"session-\$(date +%s)\",\"lineage_seed\":\"'\"$SEED_B64\"'\"},\"id\":4}' | nc -N -U $BEARDOG_SOCKET)"
echo ""

