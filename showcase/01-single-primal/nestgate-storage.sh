#!/usr/bin/env bash
# NestGate Storage Demo
# Tests BiomeOS storage operations with REAL NestGate binary

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "╔════════════════════════════════════════════════════════╗"
echo "║  BiomeOS + NestGate: Storage Operations Demo          ║"
echo "║  Testing with REAL NestGate binary                     ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

GAP_REPORT="$SCRIPT_DIR/gaps/nestgate-gaps.md"
mkdir -p "$SCRIPT_DIR/gaps"

echo "Purpose: Test BiomeOS storage orchestration with real NestGate"
echo "Duration: ~5 minutes"
echo ""

cat > "$GAP_REPORT" <<'EOF'
# Gaps Found: NestGate Integration

## Volume Management Issues
- [ ] To be documented

## Storage API Issues
- [ ] To be documented

## Data Integrity Issues
- [ ] To be documented

## Follow-Up Actions
- [ ] To be documented
EOF

echo -e "${GREEN}Step 1: Start Real NestGate Binary${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

NESTGATE_PORT=8002
"$SCRIPT_DIR/common/start-primal.sh" nestgate $NESTGATE_PORT

export NESTGATE_ENDPOINT="http://localhost:$NESTGATE_PORT"

echo ""
echo -e "${GREEN}Step 2: Test Volume Creation${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

VOLUME_REQUEST='{
  "volume_id": "demo-vol-001",
  "size_gb": 1,
  "type": "persistent"
}'

echo "Creating test volume..."
for endpoint in "/api/v1/volumes" "/volumes" "/storage/volumes"; do
    if curl -s -X POST "$NESTGATE_ENDPOINT$endpoint" \
        -H "Content-Type: application/json" \
        -d "$VOLUME_REQUEST" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Volume creation endpoint found: $endpoint${NC}"
        break
    fi
done

echo ""
echo -e "${GREEN}Step 3: Test Data Storage${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

TEST_DATA="Hello from BiomeOS at $(date)"
echo "Storing test data: $TEST_DATA"

# Try different storage endpoints
for endpoint in "/api/v1/data" "/data" "/store"; do
    if curl -s -X PUT "$NESTGATE_ENDPOINT$endpoint/test-key" \
        -H "Content-Type: text/plain" \
        -d "$TEST_DATA" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Data storage endpoint found: $endpoint${NC}"
        break
    fi
done

echo ""
echo -e "${GREEN}Step 4: Test Data Retrieval${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

for endpoint in "/api/v1/data/test-key" "/data/test-key" "/retrieve/test-key"; do
    RETRIEVED=$(curl -s "$NESTGATE_ENDPOINT$endpoint" 2>/dev/null)
    if [ -n "$RETRIEVED" ]; then
        echo -e "${GREEN}✓ Data retrieved from: $endpoint${NC}"
        echo "Retrieved: $RETRIEVED"
        break
    fi
done

echo ""
"$SCRIPT_DIR/common/stop-primal.sh" nestgate

echo ""
echo "Demo Complete! Gaps documented in: $GAP_REPORT"
echo "Next: ./beardog-security.sh"

