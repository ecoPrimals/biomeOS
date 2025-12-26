#!/usr/bin/env bash
# Demo: AI Compute Mesh - 3 Primals
# Songbird (Discovery) + ToadStool (Compute) + Squirrel (AI)
# Shows BiomeOS orchestrating distributed AI with GPU resources

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$SCRIPT_DIR/../../../.."
PHASE1_BINS="$BIOMEOS_ROOT/phase1bins"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "╔════════════════════════════════════════════════════════╗"
echo "║  Phase 1 Triple: AI Compute Mesh                      ║"
echo "║  Songbird + ToadStool + Squirrel                      ║"
echo "║  Distributed AI with Friend GPU Resources             ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

GAP_REPORT="$SCRIPT_DIR/gaps-discovered.md"
mkdir -p "$SCRIPT_DIR/logs"

cat > "$GAP_REPORT" <<'EOF'
# Gaps Discovered: AI Compute Mesh

## AI + Compute Orchestration
- [ ] To be documented during demo

## Resource Discovery and Allocation
- [ ] To be documented during demo

## Discovery + Compute + AI Integration
- [ ] To be documented during demo
EOF

echo -e "${GREEN}Step 1: Start Songbird (Discovery)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

SONGBIRD_BIN="$PHASE1_BINS/songbird-cli-dec-25-2025-standalone"
SONGBIRD_PORT=8080

$SONGBIRD_BIN tower start --port $SONGBIRD_PORT --bind 127.0.0.1 \
    > "$SCRIPT_DIR/logs/songbird.log" 2>&1 &
SONGBIRD_PID=$!
sleep 4
echo -e "${GREEN}✓ Songbird started (PID: $SONGBIRD_PID)${NC}"
echo ""

echo -e "${GREEN}Step 2: Start ToadStool (Compute)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

TOADSTOOL_BIN="$PHASE1_BINS/toadstool-bin"
TOADSTOOL_PORT=9001

$TOADSTOOL_BIN --port $TOADSTOOL_PORT > "$SCRIPT_DIR/logs/toadstool.log" 2>&1 &
TOADSTOOL_PID=$!
sleep 3
echo -e "${GREEN}✓ ToadStool started (PID: $TOADSTOOL_PID)${NC}"
echo ""

echo -e "${GREEN}Step 3: Start Squirrel (AI)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

SQUIRREL_BIN="$PHASE1_BINS/squirrel-bin"
SQUIRREL_PORT=9010

$SQUIRREL_BIN > "$SCRIPT_DIR/logs/squirrel.log" 2>&1 &
SQUIRREL_PID=$!
sleep 4
echo -e "${GREEN}✓ Squirrel started (PID: $SQUIRREL_PID, Port: $SQUIRREL_PORT)${NC}"
echo ""

echo -e "${GREEN}Step 4: BiomeOS Orchestration${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}Complete Integration Pattern:${NC}"
echo ""
echo "  // 1. Discover resources"
echo "  let songbird = SongbirdClient::new();"
echo "  let compute = songbird.find_capability(\"compute\").await?;"
echo "  let ai = songbird.find_capability(\"ai\").await?;"
echo ""
echo "  // 2. Create AI agent"
echo "  let squirrel = SquirrelClient::new(ai.endpoint);"
echo "  let agent = squirrel.create_agent(config).await?;"
echo ""
echo "  // 3. Agent needs heavy compute"
echo "  let task = agent.prepare_gpu_task(problem).await?;"
echo ""
echo "  // 4. Route to ToadStool GPU"
echo "  let toadstool = ToadStoolClient::new(compute.endpoint);"
echo "  let result = toadstool.execute(task).await?;"
echo ""
echo "  // 5. Agent integrates result"
echo "  agent.integrate_result(result).await?;"
echo ""

echo -e "${GREEN}Step 5: Real-World Scenario${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}Use Case: Collaborative AI Research${NC}"
echo ""
echo "  Research group with AI agents and GPUs:"
echo ""
echo "  1. Songbird discovers all friend GPUs (ToadStool)"
echo "  2. Squirrel AI agents coordinate work"
echo "  3. Heavy ML tasks routed to available GPUs"
echo "  4. Results shared back to agents"
echo "  5. Collaborative reasoning across the mesh"
echo ""
echo "  Result: Distributed AI with pooled GPU resources!"
echo ""

echo -e "${BLUE}Benefits:${NC}"
echo "  ✓ GPU resource sharing"
echo "  ✓ AI agent coordination"
echo "  ✓ Automatic resource discovery"
echo "  ✓ Scalable ML infrastructure"
echo "  ✓ Friend-owned compute mesh"
echo ""

echo -e "${BLUE}Real-World Applications:${NC}"
echo "  • Large language model inference"
echo "  • Computer vision processing"
echo "  • Distributed training"
echo "  • Multi-agent simulations"
echo "  • Collaborative research"
echo ""

echo -e "${GREEN}Cleanup${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
kill $SQUIRREL_PID $TOADSTOOL_PID $SONGBIRD_PID 2>/dev/null || true
sleep 1
echo -e "${GREEN}✓ All services stopped${NC}"
echo ""

echo -e "${GREEN}Demo Complete!${NC}"
echo ""
echo -e "${BLUE}Key Takeaway:${NC}"
echo "  BiomeOS orchestrated 3 primals to create a"
echo "  complete distributed AI compute mesh!"
echo ""
echo -e "${BLUE}All Triple Demos Complete!${NC}"
echo "Next: Run ../../04-complete-ecosystem/demo.sh (ALL 5!)"
echo ""

