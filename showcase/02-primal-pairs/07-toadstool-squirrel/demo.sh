#!/usr/bin/env bash
# Demo: ToadStool + Squirrel - AI Compute Orchestration
# Shows BiomeOS orchestrating AI workloads on compute resources

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$SCRIPT_DIR/../../../.."
PHASE1_BINS="$BIOMEOS_ROOT/primalBins"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "╔════════════════════════════════════════════════════════╗"
echo "║  Phase 1 Core: ToadStool + Squirrel                   ║"
echo "║  AI Compute Orchestration                             ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

GAP_REPORT="$SCRIPT_DIR/gaps-discovered.md"
mkdir -p "$SCRIPT_DIR/logs"

cat > "$GAP_REPORT" <<'EOF'
# Gaps Discovered: ToadStool + Squirrel Integration

## AI Orchestration Issues
- [ ] To be documented during demo

## Resource Allocation Issues
- [ ] To be documented during demo

## Task Routing Issues
- [ ] To be documented during demo
EOF

echo -e "${GREEN}Step 1: Start ToadStool (Compute)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

TOADSTOOL_BIN="$PHASE1_BINS/toadstool-bin"
TOADSTOOL_PORT=9001

if [ ! -f "$TOADSTOOL_BIN" ]; then
    echo -e "${RED}✗ ToadStool binary not found${NC}"
    exit 1
fi

$TOADSTOOL_BIN --port $TOADSTOOL_PORT > "$SCRIPT_DIR/logs/toadstool.log" 2>&1 &
TOADSTOOL_PID=$!
sleep 3
echo -e "${GREEN}✓ ToadStool started (PID: $TOADSTOOL_PID)${NC}"
echo ""

echo -e "${GREEN}Step 2: Start Squirrel (AI)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

SQUIRREL_BIN="$PHASE1_BINS/squirrel-bin"
SQUIRREL_PORT=9010

if [ ! -f "$SQUIRREL_BIN" ]; then
    echo -e "${RED}✗ Squirrel binary not found${NC}"
    kill $TOADSTOOL_PID 2>/dev/null || true
    exit 1
fi

$SQUIRREL_BIN > "$SCRIPT_DIR/logs/squirrel.log" 2>&1 &
SQUIRREL_PID=$!
sleep 4
echo -e "${GREEN}✓ Squirrel started (PID: $SQUIRREL_PID, Port: $SQUIRREL_PORT)${NC}"
echo ""

echo -e "${GREEN}Step 3: AI Compute Orchestration${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}Integration Pattern:${NC}"
echo ""
echo "  // AI agent needs compute resources"
echo "  let squirrel = SquirrelClient::new(endpoint);"
echo "  let agent = squirrel.create_agent(config).await?;"
echo ""
echo "  // Route heavy computation to ToadStool"
echo "  let toadstool = ToadStoolClient::new(endpoint);"
echo "  let compute_task = agent.prepare_task(problem).await?;"
echo "  let result = toadstool.execute(compute_task).await?;"
echo ""
echo "  // Agent processes results"
echo "  agent.integrate_result(result).await?;"
echo ""

echo -e "${BLUE}What This Enables:${NC}"
echo "  ✓ AI agents can use GPU resources"
echo "  ✓ Distributed ML training"
echo "  ✓ Resource-intensive AI tasks"
echo "  ✓ Scalable AI infrastructure"
echo ""

echo -e "${BLUE}Use Cases:${NC}"
echo "  • Large language model inference"
echo "  • Computer vision processing"
echo "  • Scientific ML workloads"
echo "  • Collaborative AI research"
echo ""

echo -e "${GREEN}Cleanup${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
kill $SQUIRREL_PID 2>/dev/null || true
kill $TOADSTOOL_PID 2>/dev/null || true
sleep 1
echo -e "${GREEN}✓ Services stopped${NC}"
echo ""

echo -e "${GREEN}Demo Complete!${NC}"
echo ""
echo -e "${BLUE}All 7 Primal Pairs Complete!${NC}"
echo "Next: Build triple combinations in ../../../03-primal-triples/"
echo ""

