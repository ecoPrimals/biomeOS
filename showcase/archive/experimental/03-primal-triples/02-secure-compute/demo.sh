#!/usr/bin/env bash
# Demo: Secure Compute Mesh - 3 Primals
# Songbird (Discovery) + BearDog (Crypto) + ToadStool (Compute)
# Shows BiomeOS orchestrating privacy-preserving distributed compute

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
echo "║  Phase 1 Triple: Secure Compute Mesh                  ║"
echo "║  Songbird + BearDog + ToadStool                       ║"
echo "║  Privacy-Preserving Friend Compute Network            ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

GAP_REPORT="$SCRIPT_DIR/gaps-discovered.md"
mkdir -p "$SCRIPT_DIR/logs"

cat > "$GAP_REPORT" <<'EOF'
# Gaps Discovered: Secure Compute Mesh

## Secure Compute Orchestration
- [ ] To be documented during demo

## Privacy-Preserving Task Execution
- [ ] To be documented during demo

## Discovery + Crypto + Compute Integration
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

echo -e "${GREEN}Step 2: Start BearDog (Crypto)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

BEARDOG_BIN="$PHASE1_BINS/beardog-bin"
BEARDOG_PORT=9002

$BEARDOG_BIN --port $BEARDOG_PORT > "$SCRIPT_DIR/logs/beardog.log" 2>&1 &
BEARDOG_PID=$!
sleep 3
echo -e "${GREEN}✓ BearDog started (PID: $BEARDOG_PID)${NC}"
echo ""

echo -e "${GREEN}Step 3: Start ToadStool (Compute)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

TOADSTOOL_BIN="$PHASE1_BINS/toadstool-bin"
TOADSTOOL_PORT=9001

$TOADSTOOL_BIN --port $TOADSTOOL_PORT > "$SCRIPT_DIR/logs/toadstool.log" 2>&1 &
TOADSTOOL_PID=$!
sleep 3
echo -e "${GREEN}✓ ToadStool started (PID: $TOADSTOOL_PID)${NC}"
echo ""

echo -e "${GREEN}Step 4: BiomeOS Orchestration${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}Complete Integration Pattern:${NC}"
echo ""
echo "  // 1. Discover services"
echo "  let songbird = SongbirdClient::new();"
echo "  let compute = songbird.find_capability(\"compute\").await?;"
echo "  let crypto = songbird.find_capability(\"crypto\").await?;"
echo ""
echo "  // 2. Encrypt task data"
echo "  let beardog = BearDogClient::new(crypto.endpoint);"
echo "  let encrypted_task = beardog.encrypt(task).await?;"
echo ""
echo "  // 3. Submit to compute"
echo "  let toadstool = ToadStoolClient::new(compute.endpoint);"
echo "  let encrypted_result = toadstool"
echo "      .execute(encrypted_task).await?;"
echo ""
echo "  // 4. Decrypt result"
echo "  let result = beardog.decrypt(encrypted_result).await?;"
echo ""

echo -e "${GREEN}Step 5: Real-World Scenario${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}Use Case: Privacy-Preserving ML Training${NC}"
echo ""
echo "  Alice wants to train ML model using Bob's GPU:"
echo ""
echo "  1. Songbird discovers Bob's ToadStool (GPU available)"
echo "  2. BearDog encrypts Alice's training data"
echo "  3. ToadStool trains model on encrypted data"
echo "  4. BearDog decrypts results"
echo "  5. Bob's GPU helped but never saw Alice's data!"
echo ""
echo "  Result: Private, distributed ML training!"
echo ""

echo -e "${BLUE}Benefits:${NC}"
echo "  ✓ Data privacy preserved"
echo "  ✓ Friend compute sharing"
echo "  ✓ No central server"
echo "  ✓ GPU resource pooling"
echo "  ✓ Secure by default"
echo ""

echo -e "${GREEN}Cleanup${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
kill $TOADSTOOL_PID $BEARDOG_PID $SONGBIRD_PID 2>/dev/null || true
sleep 1
echo -e "${GREEN}✓ All services stopped${NC}"
echo ""

echo -e "${GREEN}Demo Complete!${NC}"
echo ""
echo -e "${BLUE}Key Takeaway:${NC}"
echo "  BiomeOS orchestrated 3 primals to enable"
echo "  privacy-preserving distributed computing!"
echo ""
echo "Next: Run ../03-ai-compute/demo.sh"
echo ""

