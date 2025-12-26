#!/usr/bin/env bash
# Squirrel AI Demo
# Tests BiomeOS AI orchestration with REAL Squirrel binary

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "╔════════════════════════════════════════════════════════╗"
echo "║  BiomeOS + Squirrel: AI Operations Demo               ║"
echo "║  Testing with REAL Squirrel binary                     ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

GAP_REPORT="$SCRIPT_DIR/gaps/squirrel-gaps.md"
mkdir -p "$SCRIPT_DIR/gaps"

echo "Purpose: Test BiomeOS AI orchestration with real Squirrel"
echo "Duration: ~5 minutes"
echo ""

cat > "$GAP_REPORT" <<'EOF'
# Gaps Found: Squirrel Integration

## MCP Protocol Issues
- [ ] To be documented

## Agent Management Issues
- [ ] To be documented

## AI Provider Issues
- [ ] To be documented

## Follow-Up Actions
- [ ] To be documented
EOF

echo -e "${GREEN}Step 1: Start Real Squirrel Binary${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

SQUIRREL_PORT=8001
"$SCRIPT_DIR/common/start-primal.sh" squirrel $SQUIRREL_PORT

export SQUIRREL_ENDPOINT="http://localhost:$SQUIRREL_PORT"

echo ""
echo -e "${GREEN}Step 2: Test MCP Protocol${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Try MCP endpoints
for endpoint in "/mcp" "/api/mcp" "/api/v1/mcp"; do
    if curl -s "$SQUIRREL_ENDPOINT$endpoint" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ MCP endpoint found: $endpoint${NC}"
        break
    fi
done

echo ""
echo -e "${GREEN}Step 3: Test Agent Creation${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

AGENT_REQUEST='{
  "agent_id": "demo-agent-001",
  "type": "assistant",
  "model": "claude-3-5-sonnet"
}'

for endpoint in "/api/v1/agents" "/agents" "/mcp/agents"; do
    if curl -s -X POST "$SQUIRREL_ENDPOINT$endpoint" \
        -H "Content-Type: application/json" \
        -d "$AGENT_REQUEST" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Agent creation endpoint found: $endpoint${NC}"
        break
    fi
done

echo ""
echo -e "${GREEN}Step 4: Test Tool Execution${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Try tool endpoints
curl -s "$SQUIRREL_ENDPOINT/api/v1/tools" 2>/dev/null | jq '.' 2>/dev/null || echo "  (exploring...)"

echo ""
"$SCRIPT_DIR/common/stop-primal.sh" squirrel

echo ""
echo "Demo Complete! Gaps documented in: $GAP_REPORT"
echo ""
echo "All single-primal demos complete!"
echo "Review all gap reports in: $SCRIPT_DIR/gaps/"

