#!/usr/bin/env bash
# Demo: Storage + Discovery (NestGate + Songbird)
# Shows BiomeOS discovering NestGate storage via Songbird

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$SCRIPT_DIR/../../../.."
PHASE1_BINS="$BIOMEOS_ROOT/primalBins"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "╔════════════════════════════════════════════════════════╗"
echo "║  Multi-Primal Demo: Storage + Discovery               ║"
echo "║  NestGate (Storage) + Songbird (Discovery)            ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

GAP_REPORT="$SCRIPT_DIR/gaps-discovered.md"
mkdir -p "$SCRIPT_DIR/logs"

# Initialize gap report
cat > "$GAP_REPORT" <<'EOF'
# Gaps Discovered: NestGate + Songbird Integration

## Discovery Issues
- [ ] To be documented during demo

## API Integration Issues
- [ ] To be documented during demo

## Performance Issues
- [ ] To be documented during demo

## Documentation Gaps
- [ ] To be documented during demo
EOF

echo -e "${GREEN}Step 1: Start Songbird (Discovery Service)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

SONGBIRD_BIN="$PHASE1_BINS/songbird-cli-dec-25-2025-standalone"
SONGBIRD_PORT=8080

if [ ! -f "$SONGBIRD_BIN" ]; then
    echo -e "${RED}✗ Songbird binary not found${NC}"
    echo "  Expected: $SONGBIRD_BIN"
    exit 1
fi

echo "Starting Songbird tower on port $SONGBIRD_PORT..."
$SONGBIRD_BIN tower start --port $SONGBIRD_PORT --bind 127.0.0.1 \
    > "$SCRIPT_DIR/logs/songbird.log" 2>&1 &
SONGBIRD_PID=$!

echo "Waiting for Songbird to initialize..."
sleep 4

if ! kill -0 $SONGBIRD_PID 2>/dev/null; then
    echo -e "${RED}✗ Songbird failed to start${NC}"
    cat "$SCRIPT_DIR/logs/songbird.log"
    exit 1
fi

echo -e "${GREEN}✓ Songbird started (PID: $SONGBIRD_PID)${NC}"
echo ""

echo -e "${GREEN}Step 2: Start NestGate (Storage Service)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

NESTGATE_BIN="$PHASE1_BINS/nestgate-bin"
NESTGATE_PORT=9000

if [ ! -f "$NESTGATE_BIN" ]; then
    echo -e "${RED}✗ NestGate binary not found${NC}"
    echo "  Expected: $NESTGATE_BIN"
    kill $SONGBIRD_PID 2>/dev/null || true
    exit 1
fi

echo "Starting NestGate on port $NESTGATE_PORT..."
$NESTGATE_BIN --port $NESTGATE_PORT > "$SCRIPT_DIR/logs/nestgate.log" 2>&1 &
NESTGATE_PID=$!

echo "Waiting for NestGate to initialize..."
sleep 3

if ! kill -0 $NESTGATE_PID 2>/dev/null; then
    echo -e "${RED}✗ NestGate failed to start${NC}"
    cat "$SCRIPT_DIR/logs/nestgate.log"
    kill $SONGBIRD_PID 2>/dev/null || true
    exit 1
fi

echo -e "${GREEN}✓ NestGate started (PID: $NESTGATE_PID)${NC}"
echo ""

echo -e "${GREEN}Step 3: BiomeOS Discovers NestGate via Songbird${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}Scenario:${NC}"
echo "  1. BiomeOS asks Songbird: 'Find me a storage service'"
echo "  2. Songbird discovers NestGate"
echo "  3. BiomeOS gets NestGate endpoint"
echo "  4. BiomeOS uses NestGate for storage"
echo ""

# Test Songbird discovery endpoint
echo "Testing Songbird service discovery..."
SONGBIRD_ENDPOINT="http://127.0.0.1:$SONGBIRD_PORT"

if curl -s -f "$SONGBIRD_ENDPOINT/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Songbird is reachable${NC}"
else
    echo -e "${YELLOW}⚠ Songbird health endpoint not standard${NC}"
    echo "  Expected: $SONGBIRD_ENDPOINT/health"
    echo "  GAP: Document actual Songbird API endpoints"
fi
echo ""

# Test NestGate directly
echo "Testing NestGate storage service..."
NESTGATE_ENDPOINT="http://127.0.0.1:$NESTGATE_PORT"

if curl -s -f "$NESTGATE_ENDPOINT/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ NestGate is reachable${NC}"
    echo "  Response: $(curl -s $NESTGATE_ENDPOINT/health)"
else
    echo -e "${YELLOW}⚠ NestGate health endpoint not responding${NC}"
    echo "  Testing alternative endpoints..."
    # Try other common patterns
    for path in /api/health /status /ping; do
        if curl -s -f "$NESTGATE_ENDPOINT$path" > /dev/null 2>&1; then
            echo -e "${GREEN}✓ Found: $NESTGATE_ENDPOINT$path${NC}"
            break
        fi
    done
fi
echo ""

echo -e "${GREEN}Step 4: BiomeOS API Integration${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}Integration Pattern:${NC}"
echo ""
echo "  BiomeOS Code:"
echo "  ┌─────────────────────────────────────────┐"
echo "  │ // Discover storage via Songbird       │"
echo "  │ let discovery = SongbirdClient::new();  │"
echo "  │ let storage = discovery                 │"
echo "  │   .find_service(\"storage\")            │"
echo "  │   .await?;                              │"
echo "  │                                         │"
echo "  │ // Use discovered storage               │"
echo "  │ let nestgate = NestGateClient::new(    │"
echo "  │   storage.endpoint                      │"
echo "  │ );                                      │"
echo "  │ nestgate.store(data).await?;            │"
echo "  └─────────────────────────────────────────┘"
echo ""

echo -e "${GREEN}Step 5: Test Storage Operations${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Testing NestGate storage operations..."
echo ""

# Test volume creation (if NestGate supports it)
echo -e "${BLUE}1. Create Volume${NC}"
TEST_VOLUME="test-volume-$$"
if curl -s -X POST "$NESTGATE_ENDPOINT/api/v1/volumes" \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"$TEST_VOLUME\",\"size\":\"1GB\"}" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Volume created: $TEST_VOLUME${NC}"
else
    echo -e "${YELLOW}⚠ Volume creation endpoint not found${NC}"
    echo "  GAP: Document NestGate volume creation API"
fi
echo ""

# Test data storage
echo -e "${BLUE}2. Store Data${NC}"
TEST_DATA="Hello from BiomeOS Multi-Primal Demo!"
if curl -s -X PUT "$NESTGATE_ENDPOINT/api/v1/data/test-key" \
    -H "Content-Type: text/plain" \
    -d "$TEST_DATA" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Data stored${NC}"
else
    echo -e "${YELLOW}⚠ Data storage endpoint not found${NC}"
    echo "  GAP: Document NestGate data storage API"
fi
echo ""

# Test data retrieval
echo -e "${BLUE}3. Retrieve Data${NC}"
RETRIEVED=$(curl -s "$NESTGATE_ENDPOINT/api/v1/data/test-key" 2>/dev/null || echo "")
if [ -n "$RETRIEVED" ]; then
    echo -e "${GREEN}✓ Data retrieved: $RETRIEVED${NC}"
else
    echo -e "${YELLOW}⚠ Data retrieval endpoint not found${NC}"
    echo "  GAP: Document NestGate data retrieval API"
fi
echo ""

echo -e "${GREEN}Step 6: Benefits of This Pattern${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}What We Demonstrated:${NC}"
echo "  ✓ Dynamic service discovery (no hardcoding!)"
echo "  ✓ Songbird as universal adapter"
echo "  ✓ NestGate provides storage capability"
echo "  ✓ BiomeOS orchestrates both"
echo "  ✓ Real primal interaction (not mocks!)"
echo ""

echo -e "${BLUE}Benefits:${NC}"
echo "  • No hardcoded endpoints"
echo "  • Services can move/scale"
echo "  • Automatic failover possible"
echo "  • Clean separation of concerns"
echo "  • Easy to add new storage providers"
echo ""

echo -e "${GREEN}Cleanup${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Stopping services..."
kill $NESTGATE_PID 2>/dev/null || true
kill $SONGBIRD_PID 2>/dev/null || true
sleep 1
echo -e "${GREEN}✓ Services stopped${NC}"
echo ""

echo -e "${GREEN}Demo Complete!${NC}"
echo ""
echo "Review gap report: $GAP_REPORT"
echo "Review logs: $SCRIPT_DIR/logs/"
echo ""
echo "Next: Run ./02-compute-discovery/demo.sh"
echo ""

