#!/usr/bin/env bash
# Demo: Multiplex Pattern - Multiple Songbird Towers (Albatross)
# Based on: songbird/showcase/15-albatross/
# Shows BiomeOS coordinating multiple instances of the same primal

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$SCRIPT_DIR/../../../.."
PHASE1_BINS="$BIOMEOS_ROOT/phase1bins"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

echo "╔════════════════════════════════════════════════════════╗"
echo "║  Multiplex Pattern: Albatross (3 Songbird Towers)     ║"
echo "║  Based on: songbird/showcase/15-albatross/            ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

GAP_REPORT="$SCRIPT_DIR/gaps-discovered.md"
mkdir -p "$SCRIPT_DIR/logs"

cat > "$GAP_REPORT" <<'EOF'
# Gaps Discovered: Multiplex Pattern (Albatross)

## Multi-Instance Coordination
- [ ] To be documented during demo

## Load Balancing Issues
- [ ] To be documented during demo

## Federation Between Instances
- [ ] To be documented during demo
EOF

echo -e "${GREEN}Overview: Multiplex Pattern${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}What is Multiplexing?${NC}"
echo "  Running multiple instances of the SAME primal type"
echo "  for scaling, redundancy, or geographic distribution."
echo ""

echo -e "${BLUE}Why Multiplex?${NC}"
echo "  ✓ Horizontal scaling"
echo "  ✓ Fault tolerance"
echo "  ✓ Geographic distribution"
echo "  ✓ Load balancing"
echo "  ✓ High availability"
echo ""

echo -e "${GREEN}Step 1: Start Tower 1 (West Coast)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

SONGBIRD_BIN="$PHASE1_BINS/songbird-cli-dec-25-2025-standalone"
TOWER1_PORT=8081

if [ ! -f "$SONGBIRD_BIN" ]; then
    echo -e "${RED}✗ Songbird binary not found${NC}"
    exit 1
fi

echo "Starting Songbird Tower 1 (West Coast)..."
$SONGBIRD_BIN tower start --port $TOWER1_PORT --bind 127.0.0.1 \
    --name "tower-west" \
    > "$SCRIPT_DIR/logs/tower1.log" 2>&1 &
TOWER1_PID=$!
sleep 4
echo -e "${GREEN}✓ Tower 1 started (PID: $TOWER1_PID, Port: $TOWER1_PORT)${NC}"
echo ""

echo -e "${GREEN}Step 2: Start Tower 2 (East Coast)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

TOWER2_PORT=8082
echo "Starting Songbird Tower 2 (East Coast)..."
$SONGBIRD_BIN tower start --port $TOWER2_PORT --bind 127.0.0.1 \
    --name "tower-east" \
    > "$SCRIPT_DIR/logs/tower2.log" 2>&1 &
TOWER2_PID=$!
sleep 4
echo -e "${GREEN}✓ Tower 2 started (PID: $TOWER2_PID, Port: $TOWER2_PORT)${NC}"
echo ""

echo -e "${GREEN}Step 3: Start Tower 3 (Europe)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

TOWER3_PORT=8083
echo "Starting Songbird Tower 3 (Europe)..."
$SONGBIRD_BIN tower start --port $TOWER3_PORT --bind 127.0.0.1 \
    --name "tower-europe" \
    > "$SCRIPT_DIR/logs/tower3.log" 2>&1 &
TOWER3_PID=$!
sleep 4
echo -e "${GREEN}✓ Tower 3 started (PID: $TOWER3_PID, Port: $TOWER3_PORT)${NC}"
echo ""

echo -e "${PURPLE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${PURPLE}   Albatross Federation Active! 3 Towers Running!          ${NC}"
echo -e "${PURPLE}═══════════════════════════════════════════════════════════${NC}"
echo ""

echo -e "${BLUE}Tower Status:${NC}"
echo "  • Tower 1 (West):   Port $TOWER1_PORT ✓"
echo "  • Tower 2 (East):   Port $TOWER2_PORT ✓"
echo "  • Tower 3 (Europe): Port $TOWER3_PORT ✓"
echo ""

echo -e "${GREEN}Step 4: BiomeOS Multiplex Orchestration${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}How BiomeOS Uses Multiple Towers:${NC}"
echo ""
cat <<'CODE'
// BiomeOS discovers all available towers
let discovery = BiomeOSDiscovery::new();
let towers = discovery.find_all("songbird").await?;

// Result:
// towers = [
//   Tower { name: "west",   endpoint: "localhost:8081" },
//   Tower { name: "east",   endpoint: "localhost:8082" },
//   Tower { name: "europe", endpoint: "localhost:8083" },
// ]

// Strategy 1: Load Balancing
let tower = towers.select_least_loaded().await?;
tower.register_service(service).await?;

// Strategy 2: Geographic Routing
let tower = towers.select_closest_to(user_location).await?;
tower.discover_services().await?;

// Strategy 3: Redundancy
for tower in towers {
    tower.replicate_data(data).await?;
}
CODE
echo ""

echo -e "${GREEN}Step 5: Use Cases${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}1. Geographic Distribution${NC}"
echo "   Users connect to nearest tower for low latency"
echo ""

echo -e "${BLUE}2. Load Balancing${NC}"
echo "   Distribute service registrations across towers"
echo ""

echo -e "${BLUE}3. High Availability${NC}"
echo "   If one tower fails, others continue"
echo ""

echo -e "${BLUE}4. Federation${NC}"
echo "   Towers can share service discovery"
echo ""

echo -e "${BLUE}5. Privacy Zones${NC}"
echo "   Different towers for different trust domains"
echo ""

echo -e "${GREEN}Step 6: Other Primals Can Multiplex Too!${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}Examples:${NC}"
echo ""
echo "  • Multiple NestGate instances:"
echo "    - Different storage pools"
echo "    - Geographic redundancy"
echo "    - Privacy-based separation"
echo ""
echo "  • Multiple ToadStool instances:"
echo "    - Different GPU capabilities"
echo "    - Workload-specific optimization"
echo "    - Compute pool distribution"
echo ""
echo "  • Multiple Squirrel instances:"
echo "    - Agent specialization"
echo "    - Resource isolation"
echo "    - Fault tolerance"
echo ""

echo -e "${GREEN}Benefits of Multiplex Pattern${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "  ✓ Horizontal scaling (add more instances)"
echo "  ✓ Fault tolerance (redundancy)"
echo "  ✓ Geographic distribution (low latency)"
echo "  ✓ Load balancing (spread the work)"
echo "  ✓ Privacy zones (trust domains)"
echo "  ✓ Resource specialization (different configs)"
echo ""

echo "Press Enter to shut down Albatross federation..."
read

echo ""
echo -e "${GREEN}Cleanup${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
kill $TOWER3_PID $TOWER2_PID $TOWER1_PID 2>/dev/null || true
sleep 2
echo -e "${GREEN}✓ All 3 towers stopped${NC}"
echo ""

echo -e "${GREEN}Demo Complete!${NC}"
echo ""
echo -e "${BLUE}Key Takeaway:${NC}"
echo "  BiomeOS can orchestrate multiple instances of"
echo "  the same primal for scaling and reliability!"
echo ""
echo "Review: $GAP_REPORT"
echo ""

