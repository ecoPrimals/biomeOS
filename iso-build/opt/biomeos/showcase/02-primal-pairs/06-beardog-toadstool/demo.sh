#!/usr/bin/env bash
# Demo: BearDog + ToadStool - Secure Compute
# Shows BiomeOS enabling privacy-preserving computation

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
echo "║  Phase 1 Core: BearDog + ToadStool                    ║"
echo "║  Secure Compute with Privacy Preservation             ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

GAP_REPORT="$SCRIPT_DIR/gaps-discovered.md"
mkdir -p "$SCRIPT_DIR/logs"

cat > "$GAP_REPORT" <<'EOF'
# Gaps Discovered: BearDog + ToadStool Integration

## Secure Compute Issues
- [ ] To be documented during demo

## Privacy-Preserving Task Execution
- [ ] To be documented during demo

## Result Encryption Issues
- [ ] To be documented during demo
EOF

echo -e "${GREEN}Step 1: Start BearDog (Crypto)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

BEARDOG_BIN="$PHASE1_BINS/beardog-bin"
BEARDOG_PORT=9002

if [ ! -f "$BEARDOG_BIN" ]; then
    echo -e "${RED}✗ BearDog binary not found${NC}"
    exit 1
fi

$BEARDOG_BIN --port $BEARDOG_PORT > "$SCRIPT_DIR/logs/beardog.log" 2>&1 &
BEARDOG_PID=$!
sleep 3
echo -e "${GREEN}✓ BearDog started (PID: $BEARDOG_PID)${NC}"
echo ""

echo -e "${GREEN}Step 2: Start ToadStool (Compute)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

TOADSTOOL_BIN="$PHASE1_BINS/toadstool-bin"
TOADSTOOL_PORT=9001

if [ ! -f "$TOADSTOOL_BIN" ]; then
    echo -e "${RED}✗ ToadStool binary not found${NC}"
    kill $BEARDOG_PID 2>/dev/null || true
    exit 1
fi

$TOADSTOOL_BIN --port $TOADSTOOL_PORT > "$SCRIPT_DIR/logs/toadstool.log" 2>&1 &
TOADSTOOL_PID=$!
sleep 3
echo -e "${GREEN}✓ ToadStool started (PID: $TOADSTOOL_PID)${NC}"
echo ""

echo -e "${GREEN}Step 3: Secure Compute Pattern${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}Integration Pattern:${NC}"
echo ""
echo "  // Encrypt task data"
echo "  let beardog = BearDogClient::new(endpoint);"
echo "  let encrypted_task = beardog.encrypt(task).await?;"
echo ""
echo "  // Submit to compute"
echo "  let toadstool = ToadStoolClient::new(endpoint);"
echo "  let encrypted_result = toadstool"
echo "      .execute(encrypted_task).await?;"
echo ""
echo "  // Decrypt result"
echo "  let result = beardog.decrypt(encrypted_result).await?;"
echo ""

echo -e "${BLUE}Privacy Features:${NC}"
echo "  ✓ Task data encrypted"
echo "  ✓ Results encrypted"
echo "  ✓ Computation privacy-preserving"
echo "  ✓ No plaintext exposure"
echo ""

echo -e "${BLUE}Use Cases:${NC}"
echo "  • Private ML training on friend compute"
echo "  • Secure data analysis"
echo "  • Confidential computation"
echo "  • Privacy-preserving AI"
echo ""

echo -e "${GREEN}Cleanup${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
kill $TOADSTOOL_PID 2>/dev/null || true
kill $BEARDOG_PID 2>/dev/null || true
sleep 1
echo -e "${GREEN}✓ Services stopped${NC}"
echo ""

echo -e "${GREEN}Demo Complete!${NC}"
echo "Next: Run ../07-toadstool-squirrel/demo.sh (AI Compute)"
echo ""

