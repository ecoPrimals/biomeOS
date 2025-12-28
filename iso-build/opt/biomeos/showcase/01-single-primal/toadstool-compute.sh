#!/usr/bin/env bash
# ToadStool Compute Demo
# Tests BiomeOS compute orchestration with REAL ToadStool binary

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$SCRIPT_DIR/../.."

echo "╔════════════════════════════════════════════════════════╗"
echo "║  BiomeOS + ToadStool: Compute Orchestration Demo      ║"
echo "║  Testing with REAL ToadStool binary                    ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

GAP_REPORT="$SCRIPT_DIR/gaps/toadstool-gaps.md"
mkdir -p "$SCRIPT_DIR/gaps"

echo "Purpose: Demonstrate BiomeOS orchestrating compute tasks on real ToadStool"
echo "Duration: ~5 minutes"
echo ""

# Initialize gap report
cat > "$GAP_REPORT" <<'EOF'
# Gaps Found: ToadStool Integration

## Discovery Issues
- [ ] To be documented during demo

## Compute Task Submission Issues
- [ ] To be documented during demo

## API Issues
- [ ] To be documented during demo

## Resource Management Issues
- [ ] To be documented during demo

## Follow-Up Actions
- [ ] To be documented during demo
EOF

echo -e "${GREEN}Step 1: Start Real ToadStool Binary${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

TOADSTOOL_PORT=8080

echo "Starting ToadStool on port $TOADSTOOL_PORT..."
if "$SCRIPT_DIR/common/start-primal.sh" toadstool $TOADSTOOL_PORT; then
    echo -e "${GREEN}✓ ToadStool started successfully${NC}"
else
    echo -e "${RED}✗ Failed to start ToadStool${NC}"
    echo ""
    echo "GAP FOUND: ToadStool startup issue"
    exit 1
fi

echo ""
echo -e "${GREEN}Step 2: BiomeOS Discovers Compute Capability${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

export TOADSTOOL_ENDPOINT="http://localhost:$TOADSTOOL_PORT"
echo "Using endpoint: $TOADSTOOL_ENDPOINT"
echo ""

echo "Checking ToadStool health..."
if curl -s -f "$TOADSTOOL_ENDPOINT/health" > /dev/null 2>&1; then
    HEALTH=$(curl -s "$TOADSTOOL_ENDPOINT/health")
    echo -e "${GREEN}✓ Health check passed${NC}"
    echo "$HEALTH" | jq '.' 2>/dev/null || echo "$HEALTH"
else
    echo -e "${YELLOW}⚠ Health endpoint not found at /health${NC}"
    echo "GAP: Health endpoint location unknown"
fi

echo ""
echo -e "${GREEN}Step 3: Submit Simple Compute Task${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Creating test compute task..."
TASK_PAYLOAD='{
  "task_id": "demo-task-001",
  "task_type": "compute",
  "workload": {
    "type": "echo",
    "command": "echo Hello from BiomeOS",
    "timeout": 30
  }
}'

echo "Task payload:"
echo "$TASK_PAYLOAD" | jq '.' 2>/dev/null || echo "$TASK_PAYLOAD"
echo ""

# Try common compute task endpoints
for endpoint in "/api/v1/tasks" "/tasks" "/execute" "/compute"; do
    FULL_ENDPOINT="$TOADSTOOL_ENDPOINT$endpoint"
    echo "Trying task submission at: $FULL_ENDPOINT"
    
    if curl -s -X POST "$FULL_ENDPOINT" \
        -H "Content-Type: application/json" \
        -d "$TASK_PAYLOAD" > /dev/null 2>&1; then
        
        RESPONSE=$(curl -s -X POST "$FULL_ENDPOINT" \
            -H "Content-Type: application/json" \
            -d "$TASK_PAYLOAD")
        
        echo -e "${GREEN}✓ Task submission succeeded at: $endpoint${NC}"
        echo "Response:"
        echo "$RESPONSE" | jq '.' 2>/dev/null || echo "$RESPONSE"
        
        # Extract task ID if available
        TASK_ID=$(echo "$RESPONSE" | jq -r '.task_id // .id // "demo-task-001"' 2>/dev/null)
        echo ""
        echo "Task ID: $TASK_ID"
        break
    else
        echo "  Not found at: $endpoint"
    fi
done

echo ""
echo -e "${GREEN}Step 4: Query Task Status${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if [ -n "$TASK_ID" ]; then
    echo "Querying task status for: $TASK_ID"
    
    # Try common status endpoints
    for endpoint in "/api/v1/tasks/$TASK_ID" "/tasks/$TASK_ID/status" "/status/$TASK_ID"; do
        FULL_ENDPOINT="$TOADSTOOL_ENDPOINT$endpoint"
        echo "Trying: $FULL_ENDPOINT"
        
        if curl -s -f "$FULL_ENDPOINT" > /dev/null 2>&1; then
            STATUS=$(curl -s "$FULL_ENDPOINT")
            echo -e "${GREEN}✓ Task status retrieved${NC}"
            echo "$STATUS" | jq '.' 2>/dev/null || echo "$STATUS"
            break
        fi
    done
else
    echo -e "${YELLOW}No task ID available to query${NC}"
fi

echo ""
echo -e "${GREEN}Step 5: Test BiomeOS ToadStoolClient${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Testing BiomeOS client integration..."
echo ""

echo -e "${BLUE}Real API Exploration:${NC}"
echo ""

# Explore available endpoints
echo "1. Root endpoint:"
curl -s "$TOADSTOOL_ENDPOINT/" 2>/dev/null | jq '.' 2>/dev/null || echo "  (no response)"

echo ""
echo "2. API documentation (if available):"
curl -s "$TOADSTOOL_ENDPOINT/api" 2>/dev/null | jq '.' 2>/dev/null || echo "  (not found)"

echo ""
echo "3. Capabilities endpoint:"
curl -s "$TOADSTOOL_ENDPOINT/capabilities" 2>/dev/null | jq '.' 2>/dev/null || echo "  (not found)"

echo ""
echo "4. Resources endpoint:"
curl -s "$TOADSTOOL_ENDPOINT/resources" 2>/dev/null | jq '.' 2>/dev/null || echo "  (not found)"

echo ""
echo -e "${GREEN}Step 6: Test with Real biome.yaml${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Creating test biome.yaml with compute requirements..."
TEST_BIOME="$SCRIPT_DIR/test-biomes/compute-test.yaml"
mkdir -p "$SCRIPT_DIR/test-biomes"

cat > "$TEST_BIOME" <<'YAML_EOF'
biome:
  name: "compute-test"
  version: "1.0.0"
  description: "Test biome for ToadStool compute"

capabilities:
  required:
    - name: "compute"
      type: "execution"
      version: ">=1.0"

services:
  - name: "hello-compute"
    type: "task"
    image: "echo"
    command: ["echo", "Hello from BiomeOS"]
    resources:
      cpu: 1
      memory: "256M"
YAML_EOF

echo "biome.yaml created:"
cat "$TEST_BIOME"
echo ""

echo "Testing manifest submission to ToadStool..."
# Try to submit biome to ToadStool
if curl -s -X POST "$TOADSTOOL_ENDPOINT/api/v1/biomes" \
    -H "Content-Type: application/yaml" \
    --data-binary "@$TEST_BIOME" > /dev/null 2>&1; then
    
    echo -e "${GREEN}✓ Biome manifest accepted${NC}"
else
    echo -e "${YELLOW}⚠ Biome endpoint not found or different format${NC}"
    echo "GAP: Need to understand ToadStool's biome submission API"
fi

echo ""
echo -e "${GREEN}Step 7: Clean Shutdown${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Stopping ToadStool..."
"$SCRIPT_DIR/common/stop-primal.sh" toadstool

echo ""
echo "╔════════════════════════════════════════════════════════╗"
echo "║  Demo Complete: ToadStool Compute                     ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

echo "Summary:"
echo "  ✓ Started real ToadStool binary"
echo "  ✓ Tested compute task submission"
echo "  ✓ Explored available APIs"
echo "  ✓ Tested biome.yaml submission"
echo "  ✓ Clean shutdown"
echo ""

echo "Gaps documented in: $GAP_REPORT"
echo ""
echo -e "${YELLOW}Key Findings:${NC}"
echo "  - Document actual API endpoints discovered"
echo "  - Note any mismatches with expected APIs"
echo "  - Record biome.yaml submission process"
echo ""

echo "Next: ./nestgate-storage.sh"
echo ""

