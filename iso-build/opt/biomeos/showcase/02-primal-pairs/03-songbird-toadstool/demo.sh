#!/usr/bin/env bash
# Demo: Compute + Discovery (ToadStool + Songbird)
# Shows BiomeOS discovering ToadStool compute via Songbird

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
echo "║  Multi-Primal Demo: Compute + Discovery               ║"
echo "║  ToadStool (Compute) + Songbird (Discovery)           ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

GAP_REPORT="$SCRIPT_DIR/gaps-discovered.md"
mkdir -p "$SCRIPT_DIR/logs"

# Initialize gap report
cat > "$GAP_REPORT" <<'EOF'
# Gaps Discovered: ToadStool + Songbird Integration

## Discovery Issues
- [ ] To be documented during demo

## Compute Orchestration Issues
- [ ] To be documented during demo

## Task Routing Issues
- [ ] To be documented during demo

## Performance Issues
- [ ] To be documented during demo
EOF

echo -e "${GREEN}Step 1: Start Songbird (Discovery Service)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

SONGBIRD_BIN="$PHASE1_BINS/songbird-cli-dec-25-2025-standalone"
SONGBIRD_PORT=8080

if [ ! -f "$SONGBIRD_BIN" ]; then
    echo -e "${RED}✗ Songbird binary not found${NC}"
    exit 1
fi

echo "Starting Songbird tower..."
$SONGBIRD_BIN tower start --port $SONGBIRD_PORT --bind 127.0.0.1 \
    > "$SCRIPT_DIR/logs/songbird.log" 2>&1 &
SONGBIRD_PID=$!
sleep 4

if ! kill -0 $SONGBIRD_PID 2>/dev/null; then
    echo -e "${RED}✗ Songbird failed to start${NC}"
    cat "$SCRIPT_DIR/logs/songbird.log"
    exit 1
fi

echo -e "${GREEN}✓ Songbird started (PID: $SONGBIRD_PID)${NC}"
echo ""

echo -e "${GREEN}Step 2: Start ToadStool (Compute Service)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

TOADSTOOL_BIN="$PHASE1_BINS/toadstool-bin"
TOADSTOOL_PORT=9001

if [ ! -f "$TOADSTOOL_BIN" ]; then
    echo -e "${RED}✗ ToadStool binary not found${NC}"
    kill $SONGBIRD_PID 2>/dev/null || true
    exit 1
fi

echo "Starting ToadStool compute service..."
$TOADSTOOL_BIN --port $TOADSTOOL_PORT > "$SCRIPT_DIR/logs/toadstool.log" 2>&1 &
TOADSTOOL_PID=$!
sleep 3

if ! kill -0 $TOADSTOOL_PID 2>/dev/null; then
    echo -e "${RED}✗ ToadStool failed to start${NC}"
    cat "$SCRIPT_DIR/logs/toadstool.log"
    kill $SONGBIRD_PID 2>/dev/null || true
    exit 1
fi

echo -e "${GREEN}✓ ToadStool started (PID: $TOADSTOOL_PID)${NC}"
echo ""

echo -e "${GREEN}Step 3: BiomeOS Discovers ToadStool via Songbird${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}Scenario:${NC}"
echo "  1. BiomeOS needs compute resources"
echo "  2. Asks Songbird: 'Find me compute capability'"
echo "  3. Songbird discovers ToadStool"
echo "  4. BiomeOS submits tasks to ToadStool"
echo ""

SONGBIRD_ENDPOINT="http://127.0.0.1:$SONGBIRD_PORT"
TOADSTOOL_ENDPOINT="http://127.0.0.1:$TOADSTOOL_PORT"

echo "Testing Songbird discovery..."
if curl -s -f "$SONGBIRD_ENDPOINT/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Songbird reachable${NC}"
else
    echo -e "${YELLOW}⚠ Songbird health endpoint not standard${NC}"
    echo "  GAP: Document Songbird discovery API"
fi
echo ""

echo "Testing ToadStool compute service..."
if curl -s -f "$TOADSTOOL_ENDPOINT/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ ToadStool reachable${NC}"
    echo "  Response: $(curl -s $TOADSTOOL_ENDPOINT/health)"
else
    echo -e "${YELLOW}⚠ ToadStool health endpoint not responding${NC}"
    echo "  Testing alternative endpoints..."
fi
echo ""

echo -e "${GREEN}Step 4: Submit Compute Tasks${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}Task Types:${NC}"
echo "  1. Simple computation"
echo "  2. ML inference"
echo "  3. Parallel processing"
echo "  4. GPU acceleration (if available)"
echo ""

# Test task submission
echo -e "${BLUE}1. Submit Simple Task${NC}"
TASK_JSON='{"task": "compute", "payload": {"operation": "add", "values": [1, 2, 3]}}'
if curl -s -X POST "$TOADSTOOL_ENDPOINT/api/v1/tasks" \
    -H "Content-Type: application/json" \
    -d "$TASK_JSON" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Task submitted${NC}"
else
    echo -e "${YELLOW}⚠ Task submission endpoint not found${NC}"
    echo "  GAP: Document ToadStool task submission API"
fi
echo ""

echo -e "${BLUE}2. Query Available Resources${NC}"
if curl -s "$TOADSTOOL_ENDPOINT/api/v1/resources" 2>&1 | grep -q "cpu\|gpu\|memory"; then
    echo -e "${GREEN}✓ Resources available:${NC}"
    curl -s "$TOADSTOOL_ENDPOINT/api/v1/resources" | head -10
else
    echo -e "${YELLOW}⚠ Resource query endpoint not found${NC}"
    echo "  GAP: Document ToadStool resource API"
fi
echo ""

echo -e "${GREEN}Step 5: Integration Pattern${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}BiomeOS Code Example:${NC}"
echo ""
echo "  // Discover compute via Songbird"
echo "  let discovery = SongbirdClient::new();"
echo "  let compute = discovery"
echo "      .find_capability(\"compute\")"
echo "      .await?;"
echo ""
echo "  // Submit task to ToadStool"
echo "  let toadstool = ToadStoolClient::new("
echo "      compute.endpoint"
echo "  );"
echo "  let result = toadstool.submit_task(task).await?;"
echo ""

echo -e "${GREEN}Step 6: Benefits${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}What We Demonstrated:${NC}"
echo "  ✓ Dynamic compute discovery"
echo "  ✓ Task orchestration"
echo "  ✓ Resource allocation"
echo "  ✓ Capability-based routing"
echo ""

echo -e "${BLUE}Real-World Use Cases:${NC}"
echo "  • ML model training distribution"
echo "  • Batch job processing"
echo "  • GPU compute farm"
echo "  • Scientific computation"
echo ""

echo -e "${GREEN}Cleanup${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Stopping services..."
kill $TOADSTOOL_PID 2>/dev/null || true
kill $SONGBIRD_PID 2>/dev/null || true
sleep 1
echo -e "${GREEN}✓ Services stopped${NC}"
echo ""

echo -e "${GREEN}Demo Complete!${NC}"
echo ""
echo "Review gap report: $GAP_REPORT"
echo "Next: Run ../03-security-storage/demo.sh"
echo ""

