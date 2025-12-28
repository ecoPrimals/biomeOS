#!/usr/bin/env bash
# Master script to run all primal pair demos in sequence

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "╔════════════════════════════════════════════════════════╗"
echo "║                                                        ║"
echo "║  BiomeOS Phase 1 Core: All Primal Pairs              ║"
echo "║  7 Key Integration Demos                              ║"
echo "║                                                        ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

echo -e "${BLUE}This will run all 7 primal pair demos:${NC}"
echo "  1. Songbird + BearDog  (BTSP & BirdSong P2P)"
echo "  2. Songbird + NestGate (Data Federation)"
echo "  3. Songbird + ToadStool (Compute Mesh)"
echo "  4. Songbird + Squirrel (AI Coordination)"
echo "  5. BearDog + NestGate  (Encrypted Storage)"
echo "  6. BearDog + ToadStool (Secure Compute)"
echo "  7. ToadStool + Squirrel (AI Compute)"
echo ""
echo "Total duration: ~30-40 minutes"
echo ""
read -p "Press Enter to start (or Ctrl+C to cancel)..."
echo ""

DEMOS_RUN=0
DEMOS_SUCCESS=0
DEMOS_FAILED=0

run_demo() {
    local name="$1"
    local dir="$2"
    
    DEMOS_RUN=$((DEMOS_RUN + 1))
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Demo $DEMOS_RUN/7: $name"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    if cd "$SCRIPT_DIR/$dir" && ./demo.sh; then
        DEMOS_SUCCESS=$((DEMOS_SUCCESS + 1))
        echo ""
        echo -e "${GREEN}✓ $name completed successfully${NC}"
    else
        DEMOS_FAILED=$((DEMOS_FAILED + 1))
        echo ""
        echo "✗ $name failed"
    fi
    
    echo ""
    read -p "Press Enter to continue to next demo..."
}

# Run all primal pair demos
run_demo "Songbird + BearDog" "01-songbird-beardog"
run_demo "Songbird + NestGate" "02-songbird-nestgate"
run_demo "Songbird + ToadStool" "03-songbird-toadstool"
run_demo "Songbird + Squirrel" "04-songbird-squirrel"
run_demo "BearDog + NestGate" "05-beardog-nestgate"
run_demo "BearDog + ToadStool" "06-beardog-toadstool"
run_demo "ToadStool + Squirrel" "07-toadstool-squirrel"

# Final summary
echo ""
echo "╔════════════════════════════════════════════════════════╗"
echo "║                                                        ║"
echo "║  All Primal Pair Demos Complete!                      ║"
echo "║                                                        ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""
echo "Results:"
echo "  Demos run: $DEMOS_RUN"
echo "  Successful: $DEMOS_SUCCESS"
echo "  Failed: $DEMOS_FAILED"
echo ""

if [ $DEMOS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All demos completed successfully!${NC}"
    echo ""
    echo "You've seen all 7 key Phase 1 Core integrations:"
    echo "  ✓ P2P foundation (BTSP & BirdSong)"
    echo "  ✓ Data federation"
    echo "  ✓ Compute orchestration"
    echo "  ✓ AI coordination"
    echo "  ✓ Encrypted storage"
    echo "  ✓ Secure compute"
    echo "  ✓ AI compute"
    echo ""
    echo "Next Steps:"
    echo "  1. Review gap reports in each demo directory"
    echo "  2. Move to ../03-primal-triples/ for 3-primal demos"
    echo "  3. Eventually: ../04-complete-ecosystem/ for all 5!"
    echo ""
else
    echo "⚠ Some demos failed. Review output above."
    echo ""
fi

echo "Thank you for exploring BiomeOS Phase 1 Core integration!"
echo ""

