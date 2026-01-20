#!/usr/bin/env bash
#
# Tower Atomic + Squirrel Manual Deployment
# Full AI Routing Stack with Genetic Lineage
#
# PINNED DEPLOYMENT - Use while DAG system evolves
#
# Usage: export ANTHROPIC_API_KEY=sk-ant-... && ./scripts/deploy_tower_squirrel_manual.sh [family_id]
#
set -euo pipefail

FAMILY_ID="${1:-nat0}"
RUNTIME_DIR="${RUNTIME_DIR:-/tmp}"
PLASMID_BIN="${PLASMID_BIN:-./plasmidBin}"
ANTHROPIC_API_KEY="${ANTHROPIC_API_KEY:-}"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

if [ -z "$ANTHROPIC_API_KEY" ]; then
  echo -e "${RED}❌ ANTHROPIC_API_KEY not set${NC}"
  echo "Usage: export ANTHROPIC_API_KEY=sk-ant-... && $0 [family_id]"
  exit 1
fi

echo -e "${BLUE}🧬 Deploying Tower Atomic + Squirrel${NC}"
echo -e "${BLUE}Family ID: $FAMILY_ID${NC}"
echo ""

# Socket paths
BEARDOG_SOCKET="$RUNTIME_DIR/beardog-$FAMILY_ID.sock"
SONGBIRD_SOCKET="$RUNTIME_DIR/songbird-$FAMILY_ID.sock"
SQUIRREL_SOCKET="$RUNTIME_DIR/squirrel-$FAMILY_ID.sock"
NEURAL_API_SOCKET="$RUNTIME_DIR/neural-api-$FAMILY_ID.sock"

# Clean previous deployment
echo "Cleaning previous deployment..."
rm -f "$BEARDOG_SOCKET" "$SONGBIRD_SOCKET" "$SQUIRREL_SOCKET"
pkill -f "beardog.*$FAMILY_ID" 2>/dev/null || true
pkill -f "songbird.*$FAMILY_ID" 2>/dev/null || true
pkill -f "squirrel.*$FAMILY_ID" 2>/dev/null || true
sleep 1

# ============================================================================
# Phase 1: BearDog (Security Foundation)
# ============================================================================
echo ""
echo -e "${GREEN}Phase 1/3: Starting BearDog (security)...${NC}"
"$PLASMID_BIN/primals/beardog/beardog-x86_64-musl" server \
  --socket "$BEARDOG_SOCKET" \
  --family-id "$FAMILY_ID" \
  > "$RUNTIME_DIR/beardog-$FAMILY_ID.log" 2>&1 &

BEARDOG_PID=$!
echo "  ✅ BearDog started (PID: $BEARDOG_PID)"

# Wait for socket
echo -n "  Waiting for socket"
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
  echo -e "${RED}  ❌ BearDog socket not found${NC}"
  echo "  Check logs: $RUNTIME_DIR/beardog-$FAMILY_ID.log"
  exit 1
fi

# ============================================================================
# Phase 2: Songbird (Bonded to BearDog)
# ============================================================================
echo ""
echo -e "${GREEN}Phase 2/3: Starting Songbird (bonded to BearDog)...${NC}"

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

# Wait for socket
echo -n "  Waiting for socket"
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
  echo -e "${RED}  ❌ Songbird socket not found${NC}"
  echo "  Check logs: $RUNTIME_DIR/songbird-$FAMILY_ID.log"
  exit 1
fi

# ============================================================================
# Phase 3: Squirrel (Inherits from Tower)
# ============================================================================
echo ""
echo -e "${GREEN}Phase 3/3: Starting Squirrel (inherits from Tower)...${NC}"

# 🧬 GENETIC LINEAGE - Squirrel inherits family_id from Tower
SERVICE_MESH_ENDPOINT="$NEURAL_API_SOCKET" \
ANTHROPIC_API_KEY="$ANTHROPIC_API_KEY" \
"$PLASMID_BIN/primals/squirrel/squirrel-x86_64-musl" server \
  --socket "$SQUIRREL_SOCKET" \
  > "$RUNTIME_DIR/squirrel-$FAMILY_ID.log" 2>&1 &

SQUIRREL_PID=$!
echo "  ✅ Squirrel started (PID: $SQUIRREL_PID)"
echo "  🧬 Inherits from Tower Atomic (family: $FAMILY_ID)"

# Wait for socket
echo -n "  Waiting for socket"
for i in {1..30}; do
  if [ -S "$SQUIRREL_SOCKET" ]; then
    echo ""
    echo "  ✅ Squirrel socket ready: $SQUIRREL_SOCKET"
    break
  fi
  echo -n "."
  sleep 0.1
done

if [ ! -S "$SQUIRREL_SOCKET" ]; then
  echo ""
  echo -e "${YELLOW}  ⚠️  Squirrel socket not found (may still be starting)${NC}"
  echo "  Check logs: $RUNTIME_DIR/squirrel-$FAMILY_ID.log"
fi

# ============================================================================
# Deployment Summary
# ============================================================================
echo ""
echo "=========================================="
echo -e "${GREEN}Tower Atomic + Squirrel Deployed! 🎉${NC}"
echo "=========================================="
echo ""
echo -e "${BLUE}Tower Atomic (Covalent Bonding):${NC}"
echo "  BearDog:  $BEARDOG_SOCKET (PID: $BEARDOG_PID)"
echo "  Songbird: $SONGBIRD_SOCKET (PID: $SONGBIRD_PID)"
echo ""
echo -e "${BLUE}AI Orchestration (Genetic Lineage):${NC}"
echo "  Squirrel: $SQUIRREL_SOCKET (PID: $SQUIRREL_PID)"
echo ""
echo "Family ID: $FAMILY_ID"
echo ""
echo -e "${BLUE}Communication Flow:${NC}"
echo "  Squirrel → Tower (Songbird) → Anthropic API"
echo "  (Secure by default, genetic bonding model)"
echo ""
echo -e "${BLUE}Logs:${NC}"
echo "  - $RUNTIME_DIR/beardog-$FAMILY_ID.log"
echo "  - $RUNTIME_DIR/songbird-$FAMILY_ID.log"
echo "  - $RUNTIME_DIR/squirrel-$FAMILY_ID.log"
echo ""
echo -e "${BLUE}Test AI call:${NC}"
echo "  echo '{\"jsonrpc\":\"2.0\",\"method\":\"ai.chat\","
echo "    \"params\":{\"messages\":[{\"role\":\"user\",\"content\":\"Hello!\"}]},"
echo "    \"id\":1}' | nc -U $SQUIRREL_SOCKET"
echo ""
echo -e "${BLUE}To stop:${NC}"
echo "  pkill -f 'beardog.*$FAMILY_ID'"
echo "  pkill -f 'songbird.*$FAMILY_ID'"
echo "  pkill -f 'squirrel.*$FAMILY_ID'"
echo ""

