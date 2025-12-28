#!/bin/bash
# E2E Test Runner for BiomeOS Showcase Demos
# Tests all showcase demonstrations with real primals

set -e

echo "🧪 BiomeOS Showcase E2E Test Suite"
echo "===================================="
echo ""
echo "Philosophy: Test with REAL primals, expose REAL gaps"
echo ""

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_RESULTS_DIR="$SCRIPT_DIR/test-results"
mkdir -p "$TEST_RESULTS_DIR"

PASSED=0
FAILED=0
SKIPPED=0

# Color codes
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test result tracking
declare -a FAILED_TESTS
declare -a SKIPPED_TESTS

# Helper function to run a demo and check exit code
run_demo_test() {
    local demo_path=$1
    local demo_name=$2
    
    echo -n "Testing: $demo_name ... "
    
    if [ ! -f "$demo_path" ]; then
        echo -e "${YELLOW}SKIP${NC} (demo not found)"
        SKIPPED=$((SKIPPED + 1))
        SKIPPED_TESTS+=("$demo_name")
        return
    fi
    
    # Run demo and capture output
    local output_file="$TEST_RESULTS_DIR/$(basename $demo_name).log"
    if bash "$demo_path" > "$output_file" 2>&1; then
        echo -e "${GREEN}PASS${NC}"
        PASSED=$((PASSED + 1))
    else
        local exit_code=$?
        echo -e "${RED}FAIL${NC} (exit code: $exit_code)"
        FAILED=$((FAILED + 1))
        FAILED_TESTS+=("$demo_name")
    fi
}

# Start timer
START_TIME=$(date +%s)

echo "═══════════════════════════════════════════"
echo "Phase 1: Substrate Demos (00-substrate/)"
echo "═══════════════════════════════════════════"
echo ""

run_demo_test "$SCRIPT_DIR/showcase/00-substrate/01-hello-biomeos/demo.sh" "00-01-hello-biomeos"
run_demo_test "$SCRIPT_DIR/showcase/00-substrate/02-capability-composition/demo.sh" "00-02-capability-composition"
run_demo_test "$SCRIPT_DIR/showcase/00-substrate/03-niche-deployment/demo.sh" "00-03-niche-deployment"
run_demo_test "$SCRIPT_DIR/showcase/00-substrate/04-federation/demo.sh" "00-04-federation"
run_demo_test "$SCRIPT_DIR/showcase/00-substrate/05-custom-primals/demo.sh" "00-05-custom-primals"

echo ""
echo "═══════════════════════════════════════════"
echo "Phase 2: NestGate Demos (01-nestgate/)"
echo "═══════════════════════════════════════════"
echo ""

run_demo_test "$SCRIPT_DIR/showcase/01-nestgate/01-sovereign-storage/demo.sh" "01-01-sovereign-storage"
run_demo_test "$SCRIPT_DIR/showcase/01-nestgate/02-zfs-snapshots/demo.sh" "01-02-zfs-snapshots"
run_demo_test "$SCRIPT_DIR/showcase/01-nestgate/03-lineage-collaboration/demo.sh" "01-03-lineage-collaboration"
run_demo_test "$SCRIPT_DIR/showcase/01-nestgate/04-federation-replication/demo.sh" "01-04-federation-replication"
run_demo_test "$SCRIPT_DIR/showcase/01-nestgate/05-benchscale-validation/demo.sh" "01-05-benchscale-validation"

echo ""
echo "═══════════════════════════════════════════"
echo "Phase 3: BirdSong P2P Demos (02-birdsong-p2p/)"
echo "═══════════════════════════════════════════"
echo ""

run_demo_test "$SCRIPT_DIR/showcase/02-birdsong-p2p/01-encrypted-p2p/demo.sh" "02-01-encrypted-p2p"
run_demo_test "$SCRIPT_DIR/showcase/02-birdsong-p2p/02-peer-discovery/demo.sh" "02-02-peer-discovery"
run_demo_test "$SCRIPT_DIR/showcase/02-birdsong-p2p/03-multi-tower/demo.sh" "02-03-multi-tower"
run_demo_test "$SCRIPT_DIR/showcase/02-birdsong-p2p/04-secure-relay/demo.sh" "02-04-secure-relay"
run_demo_test "$SCRIPT_DIR/showcase/02-birdsong-p2p/05-full-ecosystem/demo.sh" "02-05-full-ecosystem"

# End timer
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

echo ""
echo "═══════════════════════════════════════════"
echo "Test Results Summary"
echo "═══════════════════════════════════════════"
echo ""
echo "Duration: ${DURATION}s"
echo ""
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo -e "${YELLOW}Skipped: $SKIPPED${NC}"
echo "Total: $((PASSED + FAILED + SKIPPED))"
echo ""

# Show failed tests details
if [ $FAILED -gt 0 ]; then
    echo "❌ Failed Tests:"
    for test in "${FAILED_TESTS[@]}"; do
        echo "   - $test"
        echo "     Log: $TEST_RESULTS_DIR/$test.log"
    done
    echo ""
fi

# Show skipped tests
if [ $SKIPPED -gt 0 ]; then
    echo "⏭️  Skipped Tests:"
    for test in "${SKIPPED_TESTS[@]}"; do
        echo "   - $test"
    done
    echo ""
fi

# Overall result
echo "═══════════════════════════════════════════"
if [ $FAILED -eq 0 ]; then
    if [ $SKIPPED -eq 0 ]; then
        echo -e "${GREEN}✅ ALL TESTS PASSED!${NC}"
    else
        echo -e "${GREEN}✅ ALL AVAILABLE TESTS PASSED${NC}"
        echo -e "${YELLOW}   ($SKIPPED tests skipped)${NC}"
    fi
    echo "═══════════════════════════════════════════"
    echo ""
    echo "🎉 BiomeOS showcase validated with REAL primals!"
    echo ""
    exit 0
else
    echo -e "${RED}❌ SOME TESTS FAILED${NC}"
    echo "═══════════════════════════════════════════"
    echo ""
    echo "📋 Gap Analysis:"
    echo "   Failed tests indicate real integration gaps"
    echo "   See: ../PRIMAL_GAPS.md for tracking"
    echo ""
    echo "💡 This is maturity: We expose real gaps!"
    echo ""
    exit 1
fi

