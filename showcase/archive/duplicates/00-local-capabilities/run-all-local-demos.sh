#!/usr/bin/env bash
# Run all local capability demos in sequence

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "╔════════════════════════════════════════════════════════╗"
echo "║                                                        ║"
echo "║  BiomeOS Local Capabilities Showcase                  ║"
echo "║  Demonstrating BiomeOS core without primals           ║"
echo "║                                                        ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""
echo "This will run all 5 local capability demos:"
echo "  01 - Manifest Parsing"
echo "  02 - Capability Matching"
echo "  03 - Configuration Management"
echo "  04 - Sovereignty Guardian"
echo "  05 - Client Registry"
echo ""
echo "Total duration: ~10-15 minutes"
echo ""
read -p "Press Enter to start..."
echo ""

# Track success/failure
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
        echo "✓ $demo_name completed successfully"
    else
        DEMOS_FAILED=$((DEMOS_FAILED + 1))
        echo ""
        echo "✗ $demo_name failed"
    fi
    
    echo ""
    read -p "Press Enter to continue to next demo..."
}

# Run all demos
run_demo "01 - Manifest Parsing" "01-manifest-parsing.sh"
run_demo "02 - Capability Matching" "02-capability-matching.sh"
run_demo "04 - Sovereignty Guardian" "04-sovereignty-guardian.sh"
run_demo "05 - Client Registry" "05-client-registry.sh"

# Final summary
echo ""
echo "╔════════════════════════════════════════════════════════╗"
echo "║                                                        ║"
echo "║  Local Capabilities Showcase Complete!                ║"
echo "║                                                        ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""
echo "Results:"
echo "  Demos run: $DEMOS_RUN"
echo "  Successful: $DEMOS_SUCCESS"
echo "  Failed: $DEMOS_FAILED"
echo ""

if [ $DEMOS_FAILED -eq 0 ]; then
    echo "✓ All demos completed successfully!"
    echo ""
    echo "You've seen BiomeOS's core capabilities:"
    echo "  ✓ Manifest parsing and validation"
    echo "  ✓ Capability-based matching"
    echo "  ✓ Sovereignty protections"
    echo "  ✓ Client registry management"
    echo ""
    echo "Next Steps:"
    echo "  1. Review gap documentation in each demo"
    echo "  2. Move to showcase/01-single-primal/ for real primal integration"
    echo "  3. Test with actual Phase 1 binaries"
    echo ""
else
    echo "⚠ Some demos failed. Please review output above."
    echo ""
fi

echo "Thank you for exploring BiomeOS local capabilities!"
echo ""

