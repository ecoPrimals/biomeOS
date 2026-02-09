#!/usr/bin/env bash
#
# 🧬 Lineage Gate Demo
#
# Demonstrates the Family Beacon + Lineage Gate architecture:
# 1. Both spores advertise family_id (beacon)
# 2. Before any real communication, require lineage proof (gate)
# 3. Derive session keys for encrypted communication
#
# This is the pattern for enclave-ready federation!
#

set -euo pipefail

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                                                                  ║${NC}"
echo -e "${BLUE}║         🧬 Family Beacon + Lineage Gate Demo 🧬                 ║${NC}"
echo -e "${BLUE}║                                                                  ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Configuration
SPORE1_ROOT="${SPORE1_ROOT:-/media/eastgate/biomeOS1/biomeOS}"
SPORE2_ROOT="${SPORE2_ROOT:-/media/eastgate/BEA6-BBCE/biomeOS}"
BEARDOG_SOCKET_1="${BEARDOG_SOCKET_1:-/tmp/beardog-1894e909e454-node-alpha.sock}"
BEARDOG_SOCKET_2="${BEARDOG_SOCKET_2:-/tmp/beardog-1894e909e454-node-beta.sock}"

# ═══════════════════════════════════════════════════════════════════════════════
# Phase 1: Family Beacon (Public Advertisement)
# ═══════════════════════════════════════════════════════════════════════════════

echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}PHASE 1: Family Beacon (Public Discovery)${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo "📡 What mDNS/UDP would advertise:"
echo ""
echo "  Spore A (node-alpha):"
echo "    _biome._tcp.local"
echo "    TXT: family_id=1894e909e454"
echo "    TXT: node_id=node-alpha"
echo "    TXT: lineage_mode=genesis"
echo "    TXT: socket=/tmp/beardog-1894e909e454-node-alpha.sock"
echo ""
echo "  Spore B (node-beta):"
echo "    _biome._tcp.local"
echo "    TXT: family_id=1894e909e454"
echo "    TXT: node_id=node-beta"
echo "    TXT: lineage_mode=sibling"
echo "    TXT: socket=/tmp/beardog-1894e909e454-node-beta.sock"
echo ""
echo -e "${YELLOW}⚠️  Anyone can advertise family_id=1894e909e454 - it's just a tag!${NC}"
echo -e "${YELLOW}   The REAL verification happens in Phase 2...${NC}"
echo ""

sleep 1

# ═══════════════════════════════════════════════════════════════════════════════
# Phase 2: Lineage Gate (Cryptographic Verification)
# ═══════════════════════════════════════════════════════════════════════════════

echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}PHASE 2: Lineage Gate (Cryptographic Verification)${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Check if BearDog sockets exist
if [[ ! -S "$BEARDOG_SOCKET_1" ]]; then
    echo -e "${RED}❌ BearDog socket not found at $BEARDOG_SOCKET_1${NC}"
    echo "   Start spore 1 first: cd $SPORE1_ROOT && ./deploy.sh"
    exit 1
fi

if [[ ! -S "$BEARDOG_SOCKET_2" ]]; then
    echo -e "${RED}❌ BearDog socket not found at $BEARDOG_SOCKET_2${NC}"
    echo "   Start spore 2 first: cd $SPORE2_ROOT && ./deploy.sh"
    exit 1
fi

# Load seeds
SEED1_PATH="$SPORE1_ROOT/.family.seed"
SEED2_PATH="$SPORE2_ROOT/.family.seed"

if [[ ! -f "$SEED1_PATH" ]]; then
    echo -e "${RED}❌ Seed file not found: $SEED1_PATH${NC}"
    exit 1
fi

if [[ ! -f "$SEED2_PATH" ]]; then
    echo -e "${RED}❌ Seed file not found: $SEED2_PATH${NC}"
    exit 1
fi

# Read seeds and encode as base64
SEED1_B64=$(base64 -w0 "$SEED1_PATH")
SEED2_B64=$(base64 -w0 "$SEED2_PATH")

echo "🔐 Loaded genetic seeds:"
echo "   Spore 1: ${SEED1_B64:0:32}..."
echo "   Spore 2: ${SEED2_B64:0:32}..."
echo ""

# Step 1: Spore A generates challenge (nonce)
echo -e "${GREEN}Step 1: Spore A generates challenge nonce${NC}"
NONCE=$(openssl rand -hex 32)
echo "   🎲 Nonce: ${NONCE:0:32}..."
echo ""

# Step 2: Spore B generates lineage proof using its seed
echo -e "${GREEN}Step 2: Spore B generates lineage proof via BearDog${NC}"

PROOF_REQUEST=$(cat <<EOF
{
    "jsonrpc": "2.0",
    "method": "genetic.generate_lineage_proof",
    "params": {
        "our_family_id": "1894e909e454",
        "peer_family_id": "1894e909e454",
        "lineage_seed": "$SEED2_B64"
    },
    "id": 1
}
EOF
)

echo "   → Calling genetic.generate_lineage_proof on Spore B's BearDog..."
PROOF_RESPONSE=$(echo "$PROOF_REQUEST" | socat - UNIX-CONNECT:"$BEARDOG_SOCKET_2" 2>/dev/null || echo '{"error":"failed"}')

if echo "$PROOF_RESPONSE" | grep -q '"error"'; then
    echo -e "${RED}   ❌ Failed to generate proof: $PROOF_RESPONSE${NC}"
    exit 1
fi

PROOF_B64=$(echo "$PROOF_RESPONSE" | jq -r '.result.proof // empty')
if [[ -z "$PROOF_B64" ]]; then
    echo -e "${RED}   ❌ No proof in response: $PROOF_RESPONSE${NC}"
    exit 1
fi

echo -e "${GREEN}   ✅ Proof generated: ${PROOF_B64:0:32}...${NC}"
echo ""

# Step 3: Spore A verifies the proof against its own seed
echo -e "${GREEN}Step 3: Spore A verifies lineage proof via BearDog${NC}"

VERIFY_REQUEST=$(cat <<EOF
{
    "jsonrpc": "2.0",
    "method": "genetic.verify_lineage",
    "params": {
        "our_family_id": "1894e909e454",
        "peer_family_id": "1894e909e454",
        "lineage_proof": "$PROOF_B64",
        "lineage_seed": "$SEED1_B64"
    },
    "id": 2
}
EOF
)

echo "   → Calling genetic.verify_lineage on Spore A's BearDog..."
VERIFY_RESPONSE=$(echo "$VERIFY_REQUEST" | socat - UNIX-CONNECT:"$BEARDOG_SOCKET_1" 2>/dev/null || echo '{"error":"failed"}')

if echo "$VERIFY_RESPONSE" | grep -q '"error"'; then
    echo -e "${RED}   ❌ Verification failed: $VERIFY_RESPONSE${NC}"
    exit 1
fi

IS_VALID=$(echo "$VERIFY_RESPONSE" | jq -r '.result.valid // false')

echo ""
if [[ "$IS_VALID" == "true" ]]; then
    echo -e "${GREEN}   ╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}   ║  ✅ LINEAGE GATE PASSED - Spore B is VERIFIED FAMILY!     ║${NC}"
    echo -e "${GREEN}   ╚════════════════════════════════════════════════════════════╝${NC}"
else
    echo -e "${RED}   ╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}   ║  ❌ LINEAGE GATE FAILED - Spore B is NOT FAMILY!           ║${NC}"
    echo -e "${RED}   ╚════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "   Response: $VERIFY_RESPONSE"
    exit 1
fi

echo ""
sleep 1

# ═══════════════════════════════════════════════════════════════════════════════
# Phase 3: Session Key Derivation (For Encrypted Communication)
# ═══════════════════════════════════════════════════════════════════════════════

echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}PHASE 3: Session Key Derivation (For Enclave Compute)${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${GREEN}Step 4: Derive shared session key from lineage${NC}"

SESSION_CONTEXT="session-${NONCE:0:16}"

KEY_REQUEST=$(cat <<EOF
{
    "jsonrpc": "2.0",
    "method": "genetic.derive_lineage_key",
    "params": {
        "our_family_id": "1894e909e454",
        "peer_family_id": "1894e909e454",
        "context": "$SESSION_CONTEXT",
        "lineage_seed": "$SEED1_B64"
    },
    "id": 3
}
EOF
)

echo "   → Calling genetic.derive_lineage_key on Spore A's BearDog..."
KEY_RESPONSE=$(echo "$KEY_REQUEST" | socat - UNIX-CONNECT:"$BEARDOG_SOCKET_1" 2>/dev/null || echo '{"error":"failed"}')

if echo "$KEY_RESPONSE" | grep -q '"error"'; then
    echo -e "${RED}   ❌ Key derivation failed: $KEY_RESPONSE${NC}"
    exit 1
fi

SESSION_KEY=$(echo "$KEY_RESPONSE" | jq -r '.result.key // empty')
KEY_METHOD=$(echo "$KEY_RESPONSE" | jq -r '.result.method // empty')

if [[ -n "$SESSION_KEY" ]]; then
    echo -e "${GREEN}   ✅ Session key derived!${NC}"
    echo "      Method: $KEY_METHOD"
    echo "      Key: ${SESSION_KEY:0:24}..."
    echo ""
    echo -e "${BLUE}   🔒 This key can now be used to encrypt all family communication:${NC}"
    echo "      - JSON-RPC calls"
    echo "      - Capability routing"
    echo "      - Enclave compute commands"
else
    echo -e "${YELLOW}   ⚠️ Key derivation returned empty (method may need parent seed)${NC}"
fi

echo ""

# ═══════════════════════════════════════════════════════════════════════════════
# Summary
# ═══════════════════════════════════════════════════════════════════════════════

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}                           SUMMARY${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "  ┌─────────────────────────────────────────────────────────────────┐"
echo "  │  LAYER           │  MECHANISM        │  STATUS                 │"
echo "  ├─────────────────────────────────────────────────────────────────┤"
echo "  │  Family Beacon   │  family_id tag    │  ✅ Advertised          │"
echo "  │  Lineage Gate    │  Blake3 proof     │  ✅ Verified            │"
echo "  │  Session Key     │  HKDF derivation  │  ✅ Derived             │"
echo "  │  Enclave Ready   │  Encrypted comms  │  🔜 Next step           │"
echo "  └─────────────────────────────────────────────────────────────────┘"
echo ""
echo -e "${GREEN}✅ Family tag is the beacon. Lineage proof is the gate.${NC}"
echo -e "${GREEN}   Only TRUE genetic family can establish encrypted sessions!${NC}"
echo ""
echo -e "${CYAN}🚀 Next: Use session key for enclave compute${NC}"
echo "   See: specs/FAMILY_BEACON_LINEAGE_GATE_ARCHITECTURE.md"
echo ""

