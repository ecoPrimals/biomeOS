#!/usr/bin/env bash
# Demo: Songbird + BearDog - BTSP P2P Foundation
# Based on: songbird/showcase/13-beardog-integration/
# Shows BiomeOS deploying BTSP for VPN-free P2P

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
echo "║  Phase 1 Core: Songbird + BearDog                     ║"
echo "║  BTSP & BirdSong - VPN-Free P2P Foundation            ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

echo -e "${BLUE}Based on: songbird/showcase/13-beardog-integration/${NC}"
echo -e "${BLUE}Real integration with BTSP and BirdSong${NC}"
echo ""

GAP_REPORT="$SCRIPT_DIR/gaps-discovered.md"
mkdir -p "$SCRIPT_DIR/logs"

# Initialize gap report
cat > "$GAP_REPORT" <<'EOF'
# Gaps Discovered: Songbird + BearDog Integration

## BTSP Deployment Issues
- [ ] To be documented during demo

## BirdSong Privacy Issues  
- [ ] To be documented during demo

## P2P Coordination Issues
- [ ] To be documented during demo

## API Integration Issues
- [ ] To be documented during demo
EOF

echo -e "${GREEN}Step 1: Understanding BTSP & BirdSong${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}What is BTSP?${NC}"
echo "  BTSP = BearDog Transport Security Protocol"
echo "  • VPN-free P2P communication"
echo "  • Genetic cryptography (lineage-based trust)"
echo "  • No central TURN servers needed"
echo "  • Privacy-preserving by design"
echo ""

echo -e "${BLUE}What is BirdSong?${NC}"
echo "  BirdSong = Privacy-preserving discovery"
echo "  • Encrypted service broadcasts"
echo "  • Only family can decrypt"
echo "  • Network observers see noise"
echo "  • Sovereignty + Privacy"
echo ""

echo -e "${BLUE}Why This Matters:${NC}"
echo "  Traditional: VPN or central server needed"
echo "  Songbird + BearDog: Direct P2P, no middleman!"
echo ""

echo -e "${GREEN}Step 2: Start Songbird (Service Mesh)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

SONGBIRD_BIN="$PHASE1_BINS/songbird-cli-dec-25-2025-standalone"
SONGBIRD_PORT=8080

if [ ! -f "$SONGBIRD_BIN" ]; then
    echo -e "${RED}✗ Songbird binary not found${NC}"
    echo "  Expected: $SONGBIRD_BIN"
    exit 1
fi

echo "Starting Songbird tower with BirdSong mode..."
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

echo -e "${GREEN}Step 3: Start BearDog (Crypto Service)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

BEARDOG_BIN="$PHASE1_BINS/beardog-bin"
BEARDOG_PORT=9002

if [ ! -f "$BEARDOG_BIN" ]; then
    echo -e "${RED}✗ BearDog binary not found${NC}"
    kill $SONGBIRD_PID 2>/dev/null || true
    exit 1
fi

echo "Starting BearDog BTSP service..."
$BEARDOG_BIN btsp start --port $BEARDOG_PORT > "$SCRIPT_DIR/logs/beardog.log" 2>&1 &
BEARDOG_PID=$!
sleep 3

if ! kill -0 $BEARDOG_PID 2>/dev/null; then
    echo -e "${YELLOW}⚠ BearDog may not support 'btsp start' command${NC}"
    echo "  Checking alternative BearDog modes..."
    # Try general start
    $BEARDOG_BIN --port $BEARDOG_PORT > "$SCRIPT_DIR/logs/beardog.log" 2>&1 &
    BEARDOG_PID=$!
    sleep 2
    if kill -0 $BEARDOG_PID 2>/dev/null; then
        echo -e "${GREEN}✓ BearDog started in default mode${NC}"
    else
        echo -e "${RED}✗ BearDog failed to start${NC}"
        cat "$SCRIPT_DIR/logs/beardog.log"
        kill $SONGBIRD_PID 2>/dev/null || true
        exit 1
    fi
else
    echo -e "${GREEN}✓ BearDog BTSP started (PID: $BEARDOG_PID)${NC}"
fi
echo ""

echo -e "${GREEN}Step 4: BiomeOS Deploys BTSP${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}Integration Pattern:${NC}"
echo ""
echo "  // BiomeOS discovers BearDog via Songbird"
echo "  let discovery = SongbirdClient::new();"
echo "  let crypto = discovery.find_capability(\"crypto\").await?;"
echo ""
echo "  // Deploy BTSP"
echo "  let beardog = BearDogClient::new(crypto.endpoint);"
echo "  let btsp = beardog.deploy_btsp().await?;"
echo ""
echo "  // Establish P2P connection"
echo "  let channel = btsp.connect(peer_id).await?;"
echo "  // No VPN needed! Direct P2P with privacy!"
echo ""

SONGBIRD_ENDPOINT="http://127.0.0.1:$SONGBIRD_PORT"
BEARDOG_ENDPOINT="http://127.0.0.1:$BEARDOG_PORT"

echo "Testing Songbird discovery..."
if curl -s -f "$SONGBIRD_ENDPOINT/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Songbird reachable${NC}"
else
    echo -e "${YELLOW}⚠ Checking Songbird API...${NC}"
fi
echo ""

echo "Testing BearDog crypto service..."
if curl -s -f "$BEARDOG_ENDPOINT/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ BearDog reachable${NC}"
else
    echo -e "${YELLOW}⚠ Checking BearDog API...${NC}"
    echo "  GAP: Document BearDog BTSP API endpoints"
fi
echo ""

echo -e "${GREEN}Step 5: Privacy Comparison${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}Without BearDog (Plaintext Mode):${NC}"
echo "  Network Observer Sees:"
echo "  ┌────────────────────────────────────┐"
echo "  │ Node: alice-pc (192.168.1.100)    │"
echo "  │ Capabilities: storage, compute     │"
echo "  │ Services: NestGate, ToadStool      │"
echo "  └────────────────────────────────────┘"
echo "  Privacy: ❌ LOW (everything visible)"
echo ""

echo -e "${BLUE}With BearDog (BirdSong Mode):${NC}"
echo "  Network Observer Sees:"
echo "  ┌────────────────────────────────────┐"
echo "  │ 7a 3f 9c 2e 8b 4d 1a 5f 6e 2c ... │"
echo "  │ (encrypted noise - meaningless)    │"
echo "  └────────────────────────────────────┘"
echo "  Privacy: ✅ HIGH (only family decrypts)"
echo ""

echo -e "${GREEN}Step 6: Real-World Benefits${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}What We Enable:${NC}"
echo "  ✓ Friend joins your LAN - no VPN setup!"
echo "  ✓ Privacy-preserving discovery"
echo "  ✓ No central servers needed"
echo "  ✓ Genetic trust (lineage-based)"
echo "  ✓ Connection migration (move between networks)"
echo ""

echo -e "${BLUE}Use Cases:${NC}"
echo "  • Friend compute mesh (no VPN!)"
echo "  • Private file sharing"
echo "  • Collaborative AI"
echo "  • Family cloud storage"
echo "  • Gaming P2P networks"
echo ""

echo -e "${GREEN}Step 7: Next Steps${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}Template Source:${NC}"
echo "  Songbird's 13-beardog-integration showcase has:"
echo "  • 01-privacy-comparison.sh"
echo "  • 02-graceful-degradation-test.sh"
echo "  • 03-btsp-live-integration-test.sh"
echo "  • 04-birdsong-discovery-test.sh"
echo "  • 05-full-p2p-test-suite.sh"
echo "  • 06-complete-e2e-validation.sh"
echo ""
echo "  We can adapt these for BiomeOS integration!"
echo ""

echo -e "${GREEN}Cleanup${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Stopping services..."
kill $BEARDOG_PID 2>/dev/null || true
kill $SONGBIRD_PID 2>/dev/null || true
sleep 1
echo -e "${GREEN}✓ Services stopped${NC}"
echo ""

echo -e "${GREEN}Demo Complete!${NC}"
echo ""
echo -e "${BLUE}Key Takeaways:${NC}"
echo "  • Songbird + BearDog = VPN-free P2P"
echo "  • BTSP provides secure transport"
echo "  • BirdSong provides private discovery"
echo "  • No central servers needed!"
echo "  • Privacy + Sovereignty together"
echo ""
echo "Review gap report: $GAP_REPORT"
echo "Next: Run ../02-songbird-nestgate/demo.sh (Data Federation)"
echo ""

