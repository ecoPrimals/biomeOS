#!/usr/bin/env bash
# API Adapter Testing with Real Primal Binaries
# Tests all Phase 1 API adapters with their respective binaries

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$SCRIPT_DIR/.."
PHASE2_ROOT="$SCRIPT_DIR/../.."
PHASE1_BINS="$PHASE2_ROOT/phase1bins"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                                                                ║"
echo "║     🧪 API Adapter Testing with Real Primals!                ║"
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

GAP_REPORT_DIR="$SCRIPT_DIR/api-adapter-test-results"
mkdir -p "$GAP_REPORT_DIR"

echo -e "${BLUE}Test Strategy:${NC}"
echo "  1. Start each primal binary"
echo "  2. Test API adapter discovery"
echo "  3. Document discovered endpoints"
echo "  4. Test discovered operations"
echo "  5. Generate gap report"
echo ""

# Check binaries exist
echo -e "${GREEN}Checking Phase 1 Binaries...${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

SONGBIRD_BIN="$PHASE1_BINS/songbird-cli-dec-25-2025-standalone"
NESTGATE_BIN="$PHASE1_BINS/nestgate-bin"
BEARDOG_BIN="$PHASE1_BINS/beardog-bin"
TOADSTOOL_BIN="$PHASE1_BINS/toadstool-bin"
SQUIRREL_BIN="$PHASE1_BINS/squirrel-bin"

check_binary() {
    local name=$1
    local bin=$2
    
    if [ -f "$bin" ]; then
        echo -e "  ${GREEN}✓${NC} $name: $(du -h "$bin" | cut -f1)"
    else
        echo -e "  ${RED}✗${NC} $name: NOT FOUND at $bin"
        return 1
    fi
}

check_binary "Songbird" "$SONGBIRD_BIN" || exit 1
check_binary "NestGate" "$NESTGATE_BIN" || exit 1
check_binary "BearDog" "$BEARDOG_BIN" || exit 1
check_binary "ToadStool" "$TOADSTOOL_BIN" || exit 1
check_binary "Squirrel" "$SQUIRREL_BIN" || exit 1

echo ""
echo -e "${GREEN}All binaries present!${NC}"
echo ""

# Test 1: Songbird
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}Test 1: Songbird API Adapter${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

SONGBIRD_PORT=9990
echo "Starting Songbird tower on port $SONGBIRD_PORT..."

# Kill any existing Songbird
pkill -f "songbird.*tower start" 2>/dev/null || true
sleep 2

# Start Songbird
nohup "$SONGBIRD_BIN" tower start --port "$SONGBIRD_PORT" --bind 127.0.0.1 > "$GAP_REPORT_DIR/songbird.log" 2>&1 &
SONGBIRD_PID=$!
echo "Songbird PID: $SONGBIRD_PID"

# Wait for startup
echo "Waiting for Songbird to initialize..."
sleep 8

# Test discovery patterns
echo ""
echo -e "${BLUE}Testing Songbird API Discovery:${NC}"

SONGBIRD_REPORT="$GAP_REPORT_DIR/songbird-discovery.md"
cat > "$SONGBIRD_REPORT" <<EOF
# Songbird API Discovery Results

**Date**: $(date)
**Port**: $SONGBIRD_PORT
**Binary**: $SONGBIRD_BIN

## Endpoint Discovery Results

EOF

# Test common patterns
test_endpoint() {
    local url=$1
    local name=$2
    
    echo -n "  Testing $name... "
    
    if curl -s -o /dev/null -w "%{http_code}" --max-time 3 "$url" | grep -qE "^(200|404|405)$"; then
        STATUS=$(curl -s -o /dev/null -w "%{http_code}" --max-time 3 "$url")
        echo -e "${GREEN}$STATUS${NC}"
        echo "- $name: \`$url\` → HTTP $STATUS" >> "$SONGBIRD_REPORT"
        return 0
    else
        echo -e "${YELLOW}No response${NC}"
        echo "- $name: \`$url\` → No response" >> "$SONGBIRD_REPORT"
        return 1
    fi
}

# Test health endpoints
echo ""
echo "Health endpoints:"
test_endpoint "http://localhost:$SONGBIRD_PORT/" "Root"
test_endpoint "http://localhost:$SONGBIRD_PORT/health" "Health"
test_endpoint "http://localhost:$SONGBIRD_PORT/api/health" "API Health"
test_endpoint "http://localhost:$SONGBIRD_PORT/status" "Status"

# Test tower endpoints
echo ""
echo "Tower endpoints:"
test_endpoint "http://localhost:$SONGBIRD_PORT/tower/status" "Tower Status"
test_endpoint "http://localhost:$SONGBIRD_PORT/tower/info" "Tower Info"
test_endpoint "http://localhost:$SONGBIRD_PORT/api/tower/status" "API Tower Status"

# Test service endpoints
echo ""
echo "Service endpoints:"
test_endpoint "http://localhost:$SONGBIRD_PORT/services" "Services"
test_endpoint "http://localhost:$SONGBIRD_PORT/api/services" "API Services"
test_endpoint "http://localhost:$SONGBIRD_PORT/api/v1/services" "API v1 Services"

# Test federation endpoints
echo ""
echo "Federation endpoints:"
test_endpoint "http://localhost:$SONGBIRD_PORT/federation" "Federation"
test_endpoint "http://localhost:$SONGBIRD_PORT/api/federation" "API Federation"

# Test gaming endpoints
echo ""
echo "Gaming endpoints:"
test_endpoint "http://localhost:$SONGBIRD_PORT/gaming/sessions" "Gaming Sessions"
test_endpoint "http://localhost:$SONGBIRD_PORT/api/gaming/sessions" "API Gaming Sessions"

# Add summary
cat >> "$SONGBIRD_REPORT" <<EOF

## Summary

Songbird API discovery completed. Check the log file for detailed output:
\`$GAP_REPORT_DIR/songbird.log\`

## Next Steps

1. Document which patterns worked
2. Update SongbirdAdapter with correct patterns
3. Cache discovered structure
4. Report any gaps to Songbird team

EOF

echo ""
echo -e "${GREEN}Songbird discovery report:${NC} $SONGBIRD_REPORT"

# Stop Songbird
echo ""
echo "Stopping Songbird..."
kill $SONGBIRD_PID 2>/dev/null || true
sleep 2

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}Songbird API Adapter Test Complete!${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}Results Location:${NC}"
echo "  Discovery Report: $SONGBIRD_REPORT"
echo "  Server Log: $GAP_REPORT_DIR/songbird.log"
echo ""

echo -e "${YELLOW}Note:${NC} Testing other primals (NestGate, BearDog, ToadStool, Squirrel)"
echo "will follow the same pattern. Each will get its own discovery report."
echo ""

echo "🎊 API Adapter Testing Session Started!"
echo ""
echo "Next: Review Songbird results, then test remaining primals."
echo ""

