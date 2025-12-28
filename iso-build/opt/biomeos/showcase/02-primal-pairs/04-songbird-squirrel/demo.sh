#!/usr/bin/env bash
# Demo: Songbird + Squirrel - AI Agent Coordination
# Shows BiomeOS discovering and coordinating AI agents via Songbird

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$SCRIPT_DIR/../../../.."
PHASE1_BINS="$BIOMEOS_ROOT/phase1bins"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "╔════════════════════════════════════════════════════════╗"
echo "║  Phase 1 Core: Songbird + Squirrel                    ║"
echo "║  AI Agent Discovery & Coordination                    ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

GAP_REPORT="$SCRIPT_DIR/gaps-discovered.md"
mkdir -p "$SCRIPT_DIR/logs"

cat > "$GAP_REPORT" <<'EOF'
# Gaps Discovered: Songbird + Squirrel Integration

## AI Discovery Issues
- [ ] To be documented during demo

## Agent Coordination Issues
- [ ] To be documented during demo

## MCP Protocol Issues
- [ ] To be documented during demo
EOF

echo -e "${GREEN}Step 1: Start Songbird (Discovery)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

SONGBIRD_BIN="$PHASE1_BINS/songbird-cli-dec-25-2025-standalone"
SONGBIRD_PORT=8080

if [ ! -f "$SONGBIRD_BIN" ]; then
    echo -e "${RED}✗ Songbird binary not found${NC}"
    exit 1
fi

$SONGBIRD_BIN tower start --port $SONGBIRD_PORT --bind 127.0.0.1 \
    > "$SCRIPT_DIR/logs/songbird.log" 2>&1 &
SONGBIRD_PID=$!
sleep 4

if ! kill -0 $SONGBIRD_PID 2>/dev/null; then
    echo -e "${RED}✗ Songbird failed to start${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Songbird started (PID: $SONGBIRD_PID)${NC}"
echo ""

echo -e "${GREEN}Step 2: Start Squirrel (AI Service)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

SQUIRREL_BIN="$PHASE1_BINS/squirrel-bin"
SQUIRREL_PORT=9010

if [ ! -f "$SQUIRREL_BIN" ]; then
    echo -e "${RED}✗ Squirrel binary not found${NC}"
    kill $SONGBIRD_PID 2>/dev/null || true
    exit 1
fi

echo "Starting Squirrel AI service..."
$SQUIRREL_BIN > "$SCRIPT_DIR/logs/squirrel.log" 2>&1 &
SQUIRREL_PID=$!
sleep 4

if ! kill -0 $SQUIRREL_PID 2>/dev/null; then
    echo -e "${RED}✗ Squirrel failed to start${NC}"
    kill $SONGBIRD_PID 2>/dev/null || true
    exit 1
fi

echo -e "${GREEN}✓ Squirrel started (PID: $SQUIRREL_PID, Port: $SQUIRREL_PORT)${NC}"
echo ""

echo -e "${GREEN}Step 3: BiomeOS AI Coordination${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}Integration Pattern:${NC}"
echo ""
echo "  // Discover AI agents via Songbird"
echo "  let discovery = SongbirdClient::new();"
echo "  let ai = discovery.find_capability(\"ai\").await?;"
echo ""
echo "  // Create and coordinate agents"
echo "  let squirrel = SquirrelClient::new(ai.endpoint);"
echo "  let agent = squirrel.create_agent(config).await?;"
echo "  let result = agent.execute_task(task).await?;"
echo ""

SONGBIRD_ENDPOINT="http://127.0.0.1:$SONGBIRD_PORT"
SQUIRREL_ENDPOINT="http://127.0.0.1:$SQUIRREL_PORT"

echo "Testing Squirrel AI service..."
if curl -s -f "$SQUIRREL_ENDPOINT/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Squirrel reachable${NC}"
    echo "  Response: $(curl -s $SQUIRREL_ENDPOINT/health)"
else
    echo -e "${YELLOW}⚠ Squirrel health endpoint checking...${NC}"
fi
echo ""

echo -e "${GREEN}Step 4: AI Agent Use Cases${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}What BiomeOS Enables:${NC}"
echo "  ✓ Distributed AI agents"
echo "  ✓ MCP protocol support"
echo "  ✓ Tool use coordination"
echo "  ✓ Context management"
echo "  ✓ Collaborative reasoning"
echo ""

echo -e "${BLUE}Real-World Scenarios:${NC}"
echo "  • Code generation across friend network"
echo "  • Collaborative research"
echo "  • Distributed problem solving"
echo "  • Multi-agent simulations"
echo ""

echo -e "${GREEN}Cleanup${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
kill $SQUIRREL_PID 2>/dev/null || true
kill $SONGBIRD_PID 2>/dev/null || true
sleep 1
echo -e "${GREEN}✓ Services stopped${NC}"
echo ""

echo -e "${GREEN}Demo Complete!${NC}"
echo "Next: Run ../05-beardog-nestgate/demo.sh (Encrypted Storage)"
echo ""

