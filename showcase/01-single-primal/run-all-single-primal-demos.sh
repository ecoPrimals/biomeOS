#!/usr/bin/env bash
# Run all single-primal demos in sequence

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "╔════════════════════════════════════════════════════════╗"
echo "║                                                        ║"
echo "║  BiomeOS Single-Primal Integration Showcase           ║"
echo "║  Testing with REAL Phase 1 binaries                   ║"
echo "║                                                        ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

echo "This will test BiomeOS integration with each Phase 1 primal:"
echo "  1. Songbird - Service discovery"
echo "  2. ToadStool - Compute orchestration"
echo "  3. NestGate - Storage operations"
echo "  4. BearDog - Security operations"
echo "  5. Squirrel - AI operations"
echo ""
echo "Total duration: ~25 minutes"
echo ""
echo "Prerequisites:"
echo "  - Phase 1 binaries in ../../phase1bins/"
echo "  - Ports 3000, 8080, 8002, 9000, 8001 available"
echo ""

read -p "Press Enter to start..."

DEMOS_RUN=0
DEMOS_SUCCESS=0
DEMOS_FAILED=0

run_demo() {
    local demo_name="$1"
    local demo_script="$2"
    
    DEMOS_RUN=$((DEMOS_RUN + 1))
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Running: $demo_name"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    if "$SCRIPT_DIR/$demo_script"; then
        DEMOS_SUCCESS=$((DEMOS_SUCCESS + 1))
        echo ""
        echo "✓ $demo_name completed"
    else
        DEMOS_FAILED=$((DEMOS_FAILED + 1))
        echo ""
        echo "✗ $demo_name failed"
    fi
    
    echo ""
    read -p "Press Enter to continue..."
}

# Run all demos
run_demo "Songbird Discovery" "songbird-discovery.sh"
run_demo "ToadStool Compute" "toadstool-compute.sh"
run_demo "NestGate Storage" "nestgate-storage.sh"
run_demo "BearDog Security" "beardog-security.sh"
run_demo "Squirrel AI" "squirrel-ai.sh"

echo ""
echo "╔════════════════════════════════════════════════════════╗"
echo "║                                                        ║"
echo "║  Single-Primal Integration Complete!                  ║"
echo "║                                                        ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

echo "Results:"
echo "  Demos run: $DEMOS_RUN"
echo "  Successful: $DEMOS_SUCCESS"
echo "  Failed: $DEMOS_FAILED"
echo ""

echo "Gap Reports:"
echo "  Review all findings in: $SCRIPT_DIR/gaps/"
ls -1 "$SCRIPT_DIR/gaps/"*.md 2>/dev/null || echo "  (no gap reports generated yet)"
echo ""

echo "Next Steps:"
echo "  1. Review all gap reports"
echo "  2. Update BiomeOS adapters based on findings"
echo "  3. Coordinate with primal teams on API issues"
echo "  4. Move to showcase/02-multi-primal/ for cross-primal testing"
echo ""

echo "Thank you for testing BiomeOS integration!"

