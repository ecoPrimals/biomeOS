#!/usr/bin/env bash
# Lineage Collaboration Demo
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../common/discovery.sh"

echo "╔══════════════════════════════════════════════════════════╗"
echo "║     🔗 Lineage Collaboration Demo                       ║"
echo "║  Trust-Based Sharing Without Third Parties              ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

echo "━━━ STEP 1: Lineage Setup ━━━"
echo ""
echo -e "${BLUE}🏛️  Genesis Device (You):${NC}"
echo "   ID: genesis-abc123"
echo "   Role: Root of trust"
echo ""
echo -e "${BLUE}📱 Child Devices:${NC}"
echo "   • Laptop: laptop-xyz789 (verified child)"
echo "   • Phone: phone-def456 (unverified)"
echo ""
sleep 2

echo "━━━ STEP 2: Create & Share ━━━"
echo ""
echo -e "${GREEN}✅ Document created on genesis${NC}"
echo "   Owner: genesis-abc123"
echo "   Lineage-gated: true"
echo ""
echo -e "${GREEN}✅ Laptop accesses (verified lineage)${NC}"
echo -e "${RED}❌ Phone denied (no lineage proof)${NC}"
echo ""
echo -e "${BLUE}🔑 Genesis grants phone lineage...${NC}"
echo -e "${GREEN}✅ Phone can now access!${NC}"
echo ""
sleep 2

echo "━━━ Demo Complete ━━━"
echo "🌱 Lineage: Trust inherited, not purchased"
echo ""

