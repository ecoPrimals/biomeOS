#!/usr/bin/env bash
#
# Tower Atomic Manual Deployment
# BearDog + Songbird with Genetic Bonding
#
# PINNED DEPLOYMENT - Use while DAG system evolves
#
# Usage: ./scripts/deploy_tower_atomic_manual.sh [family_id]
#
set -euo pipefail

FAMILY_ID="${1:-nat0}"
RUNTIME_DIR="${RUNTIME_DIR:-/tmp}"
PLASMID_BIN="${PLASMID_BIN:-./plasmidBin}"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🧬 Deploying Tower Atomic${NC}"
echo -e "${BLUE}Family ID: $FAMILY_ID${NC}"
echo ""

# Socket paths
BEARDOG_SOCKET="$RUNTIME_DIR/beardog-$FAMILY_ID.sock"
SONGBIRD_SOCKET="$RUNTIME_DIR/songbird-$FAMILY_ID.sock"

# Clean previous deployment
echo "Cleaning previous deployment..."
rm -f "$BEARDOG_SOCKET" "$SONGBIRD_SOCKET"
pkill -f "beardog.*$FAMILY_ID" 2>/dev/null || true
pkill -f "songbird.*$FAMILY_ID" 2>/dev/null || true
sleep 1

echo ""
echo -e "${GREEN}Phase 1/2: Starting BearDog (security primal)...${NC}"
"$PLASMID_BIN/primals/beardog/beardog-x86_64-musl" server \
  --socket "$BEARDOG_SOCKET" \
  --family-id "$FAMILY_ID" \
  > "$RUNTIME_DIR/beardog-$FAMILY_ID.log" 2>&1 &

BEARDOG_PID=$!
echo "  ✅ BearDog started (PID: $BEARDOG_PID)"

# Wait for BearDog socket
echo -n "  Waiting for BearDog socket"
for i in {1..30}; do
  if [ -S "$BEARDOG_SOCKET" ]; then
    echo ""
    echo "  ✅ BearDog socket ready: $BEARDOG_SOCKET"
    break
  fi
  echo -n "."
  sleep 0.1
done

if [ ! -S "$BEARDOG_SOCKET" ]; then
  echo ""
  echo "  ❌ BearDog socket not found after 3s"
  echo "  Check logs: $RUNTIME_DIR/beardog-$FAMILY_ID.log"
  exit 1
fi

echo ""
echo -e "${GREEN}Phase 2/2: Starting Songbird (bonded to BearDog)...${NC}"

# 🧬 GENETIC BONDING - Songbird inherits from BearDog
SONGBIRD_SOCKET="$SONGBIRD_SOCKET" \
SONGBIRD_SECURITY_PROVIDER="$BEARDOG_SOCKET" \
SECURITY_ENDPOINT="$BEARDOG_SOCKET" \
SONGBIRD_ORCHESTRATOR_FAMILY_ID="$FAMILY_ID" \
"$PLASMID_BIN/primals/songbird/songbird-x86_64-musl" server \
  > "$RUNTIME_DIR/songbird-$FAMILY_ID.log" 2>&1 &

SONGBIRD_PID=$!
echo "  ✅ Songbird started (PID: $SONGBIRD_PID)"
echo "  🧬 Bonded to BearDog: $BEARDOG_SOCKET"

# Wait for Songbird socket
echo -n "  Waiting for Songbird socket"
for i in {1..30}; do
  if [ -S "$SONGBIRD_SOCKET" ]; then
    echo ""
    echo "  ✅ Songbird socket ready: $SONGBIRD_SOCKET"
    break
  fi
  echo -n "."
  sleep 0.1
done

if [ ! -S "$SONGBIRD_SOCKET" ]; then
  echo ""
  echo "  ❌ Songbird socket not found after 3s"
  echo "  Check logs: $RUNTIME_DIR/songbird-$FAMILY_ID.log"
  exit 1
fi

echo ""
echo "=========================================="
echo -e "${GREEN}Tower Atomic Deployed! 🎉${NC}"
echo "=========================================="
echo ""
echo "BearDog:  $BEARDOG_SOCKET (PID: $BEARDOG_PID)"
echo "Songbird: $SONGBIRD_SOCKET (PID: $SONGBIRD_PID)"
echo "Family:   $FAMILY_ID"
echo ""
echo "Logs:"
echo "  - $RUNTIME_DIR/beardog-$FAMILY_ID.log"
echo "  - $RUNTIME_DIR/songbird-$FAMILY_ID.log"
echo ""
echo "Bonding:  Covalent (BearDog + Songbird)"
echo "Status:   Tower Atomic Operational ✅"
echo ""
echo "To stop:"
echo "  pkill -f 'beardog.*$FAMILY_ID'"
echo "  pkill -f 'songbird.*$FAMILY_ID'"
echo ""

