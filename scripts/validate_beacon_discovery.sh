#!/bin/bash
# Beacon Genetics Discovery Validation Script
# Validates USB ↔ Pixel beacon discovery via Neural API capability.call
#
# Date: February 4, 2026
# Purpose: Validate beacon genetics and create Songbird evolution handoff

set -e

FAMILY_ID="${FAMILY_ID:-1894e909e454}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOME_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}       BEACON GENETICS DISCOVERY VALIDATION                     ${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo "Family ID: $FAMILY_ID"
echo "Date: $(date -Iseconds)"
echo ""

# ═══════════════════════════════════════════════════════════════════════════
# Step 1: Check if primals are running locally
# ═══════════════════════════════════════════════════════════════════════════

echo -e "${YELLOW}Step 1: Check local primal status${NC}"

SOCKET_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/biomeos"
BEARDOG_SOCKET="$SOCKET_DIR/beardog-$FAMILY_ID.sock"
SONGBIRD_SOCKET="$SOCKET_DIR/songbird-$FAMILY_ID.sock"

check_socket() {
    local socket=$1
    local name=$2
    if [ -S "$socket" ]; then
        echo -e "  ${GREEN}✓${NC} $name socket exists: $socket"
        return 0
    else
        echo -e "  ${RED}✗${NC} $name socket missing: $socket"
        return 1
    fi
}

BEARDOG_OK=false
SONGBIRD_OK=false

if check_socket "$BEARDOG_SOCKET" "BearDog"; then
    BEARDOG_OK=true
fi

if check_socket "$SONGBIRD_SOCKET" "Songbird"; then
    SONGBIRD_OK=true
fi

echo ""

# ═══════════════════════════════════════════════════════════════════════════
# Step 2: Test BearDog beacon capabilities
# ═══════════════════════════════════════════════════════════════════════════

echo -e "${YELLOW}Step 2: Test BearDog beacon capabilities${NC}"

if [ "$BEARDOG_OK" = true ]; then
    echo "  Testing beacon.get_id..."
    BEACON_ID_RESPONSE=$(echo '{"jsonrpc":"2.0","method":"beacon.get_id","params":{},"id":1}' | \
        timeout 5 nc -U "$BEARDOG_SOCKET" 2>/dev/null || echo '{"error":"timeout"}')
    
    if echo "$BEACON_ID_RESPONSE" | grep -q '"result"'; then
        echo -e "  ${GREEN}✓${NC} beacon.get_id works"
        BEACON_ID=$(echo "$BEACON_ID_RESPONSE" | grep -o '"beacon_id":"[^"]*"' | cut -d'"' -f4 || echo "unknown")
        echo "    Beacon ID: ${BEACON_ID:0:16}..."
    else
        echo -e "  ${RED}✗${NC} beacon.get_id failed: $BEACON_ID_RESPONSE"
    fi
    
    echo "  Testing health.check..."
    HEALTH_RESPONSE=$(echo '{"jsonrpc":"2.0","method":"health.check","params":{},"id":2}' | \
        timeout 5 nc -U "$BEARDOG_SOCKET" 2>/dev/null || echo '{"error":"timeout"}')
    
    if echo "$HEALTH_RESPONSE" | grep -q '"healthy"'; then
        echo -e "  ${GREEN}✓${NC} BearDog healthy"
    else
        echo -e "  ${YELLOW}⚠${NC} BearDog health unknown: $HEALTH_RESPONSE"
    fi
else
    echo -e "  ${RED}✗${NC} BearDog not running - skipping tests"
fi

echo ""

# ═══════════════════════════════════════════════════════════════════════════
# Step 3: Test Songbird capabilities
# ═══════════════════════════════════════════════════════════════════════════

echo -e "${YELLOW}Step 3: Test Songbird discovery capabilities${NC}"

if [ "$SONGBIRD_OK" = true ]; then
    echo "  Testing rpc.discover (list methods)..."
    RPC_DISCOVER=$(echo '{"jsonrpc":"2.0","method":"rpc.discover","params":{},"id":1}' | \
        timeout 5 nc -U "$SONGBIRD_SOCKET" 2>/dev/null || echo '{"error":"timeout"}')
    
    if echo "$RPC_DISCOVER" | grep -q 'beacon_exchange\|beacon\|birdsong'; then
        echo -e "  ${GREEN}✓${NC} Songbird has beacon/discovery methods"
    elif echo "$RPC_DISCOVER" | grep -q '"result"'; then
        echo -e "  ${YELLOW}⚠${NC} Songbird responded but beacon_exchange not found"
        echo "    Available methods may not include beacon exchange yet"
    else
        echo -e "  ${YELLOW}⚠${NC} rpc.discover failed: ${RPC_DISCOVER:0:100}..."
    fi
    
    echo "  Testing health.check..."
    HEALTH_RESPONSE=$(echo '{"jsonrpc":"2.0","method":"health.check","params":{},"id":2}' | \
        timeout 5 nc -U "$SONGBIRD_SOCKET" 2>/dev/null || echo '{"error":"timeout"}')
    
    if echo "$HEALTH_RESPONSE" | grep -q '"healthy"\|"status"'; then
        echo -e "  ${GREEN}✓${NC} Songbird healthy"
    else
        echo -e "  ${YELLOW}⚠${NC} Songbird health unknown"
    fi
    
    echo "  Testing http capability (proxy test)..."
    HTTP_TEST=$(echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://httpbin.org/get"},"id":3}' | \
        timeout 10 nc -U "$SONGBIRD_SOCKET" 2>/dev/null || echo '{"error":"timeout"}')
    
    if echo "$HTTP_TEST" | grep -q '"result"\|"body"'; then
        echo -e "  ${GREEN}✓${NC} Songbird HTTP proxy works"
    else
        echo -e "  ${YELLOW}⚠${NC} HTTP test: ${HTTP_TEST:0:80}..."
    fi
else
    echo -e "  ${RED}✗${NC} Songbird not running - skipping tests"
fi

echo ""

# ═══════════════════════════════════════════════════════════════════════════
# Step 4: Check beacon genetics files
# ═══════════════════════════════════════════════════════════════════════════

echo -e "${YELLOW}Step 4: Check beacon genetics files${NC}"

LIVESPORE="$BIOME_ROOT/livespore-usb"
KNOWN_BEACONS="$LIVESPORE/.known_beacons.json"
BEACON_SEED="$LIVESPORE/.beacon.seed"
FAMILY_SEED="$LIVESPORE/.family.seed"

if [ -f "$KNOWN_BEACONS" ]; then
    echo -e "  ${GREEN}✓${NC} .known_beacons.json exists"
    MEETING_COUNT=$(grep -o '"meetings"' "$KNOWN_BEACONS" | wc -l || echo "0")
    echo "    Meetings configured: $MEETING_COUNT"
else
    echo -e "  ${YELLOW}⚠${NC} .known_beacons.json not found"
fi

if [ -f "$BEACON_SEED" ]; then
    echo -e "  ${GREEN}✓${NC} .beacon.seed exists"
else
    echo -e "  ${YELLOW}⚠${NC} .beacon.seed not found"
fi

if [ -f "$FAMILY_SEED" ]; then
    echo -e "  ${GREEN}✓${NC} .family.seed exists"
else
    echo -e "  ${YELLOW}⚠${NC} .family.seed not found"
fi

echo ""

# ═══════════════════════════════════════════════════════════════════════════
# Step 5: Check ADB/Pixel connectivity
# ═══════════════════════════════════════════════════════════════════════════

echo -e "${YELLOW}Step 5: Check Pixel 8a connectivity${NC}"

if command -v adb &> /dev/null; then
    DEVICES=$(adb devices 2>/dev/null | grep -v "List" | grep -v "^$" | wc -l)
    if [ "$DEVICES" -gt 0 ]; then
        echo -e "  ${GREEN}✓${NC} ADB connected ($DEVICES device(s))"
        
        # Check if TCP forwarding is set up
        FORWARDS=$(adb forward --list 2>/dev/null || echo "")
        if [ -n "$FORWARDS" ]; then
            echo "    Forwards active:"
            echo "$FORWARDS" | while read line; do echo "      $line"; done
        else
            echo "    No TCP forwards active"
        fi
        
        # Check if primals running on Pixel
        echo "  Checking Pixel primal status..."
        PIXEL_BEARDOG=$(adb shell "ls /data/local/tmp/biomeos/beardog*.sock 2>/dev/null" || echo "")
        if [ -n "$PIXEL_BEARDOG" ]; then
            echo -e "  ${GREEN}✓${NC} BearDog socket found on Pixel"
        else
            echo -e "  ${YELLOW}⚠${NC} No BearDog socket on Pixel (not deployed yet)"
        fi
    else
        echo -e "  ${YELLOW}⚠${NC} No ADB devices connected"
    fi
else
    echo -e "  ${YELLOW}⚠${NC} ADB not installed"
fi

echo ""

# ═══════════════════════════════════════════════════════════════════════════
# Summary
# ═══════════════════════════════════════════════════════════════════════════

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}                         SUMMARY                                ${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""

if [ "$BEARDOG_OK" = true ] && [ "$SONGBIRD_OK" = true ]; then
    echo -e "${GREEN}✓ Tower Atomic running locally${NC}"
    echo ""
    echo "Next steps to test USB ↔ Pixel beacon discovery:"
    echo ""
    echo "  1. Deploy Tower on Pixel:"
    echo "     adb push livespore-usb/aarch64 /data/local/tmp/biomeos/"
    echo "     adb shell 'cd /data/local/tmp/biomeos/aarch64/scripts && FAMILY_ID=$FAMILY_ID ./start_tower.sh'"
    echo ""
    echo "  2. Set up TCP forwarding for cross-device:"
    echo "     adb reverse tcp:9100 tcp:9100  # BearDog"
    echo "     adb reverse tcp:9101 tcp:9101  # Songbird"
    echo ""
    echo "  3. Test beacon discovery via Neural API:"
    echo "     (See handoff document for capability.call examples)"
else
    echo -e "${YELLOW}⚠ Tower Atomic not fully running${NC}"
    echo ""
    echo "To start Tower Atomic locally:"
    echo "  cd livespore-usb/x86_64/scripts/"
    echo "  FAMILY_ID=$FAMILY_ID ./start_tower.sh"
fi

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo "Validation complete: $(date -Iseconds)"
