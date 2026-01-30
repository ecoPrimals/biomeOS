#!/bin/bash
# NUCLEUS Comprehensive Test Runner
# Runs all Unit, E2E, Chaos, and Fault tests for all 3 atomic patterns

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                                                              ║"
echo "║       🧪 NUCLEUS COMPREHENSIVE TEST SUITE 🧪                ║"
echo "║                                                              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Configuration
TEST_RESULTS_DIR="test-results"
mkdir -p $TEST_RESULTS_DIR

# Track results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run test suite
run_test_suite() {
    local name=$1
    local command=$2
    local requires_sudo=$3
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo -e "${BLUE}🔷 Running: $name${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    local log_file="$TEST_RESULTS_DIR/${name//[ \/]/_}.log"
    
    if [ "$requires_sudo" = "true" ]; then
        if sudo -n true 2>/dev/null; then
            if eval "$command" > "$log_file" 2>&1; then
                echo -e "${GREEN}✅ PASSED: $name${NC}"
                PASSED_TESTS=$((PASSED_TESTS + 1))
            else
                echo -e "${RED}❌ FAILED: $name${NC}"
                echo "   Log: $log_file"
                FAILED_TESTS=$((FAILED_TESTS + 1))
            fi
        else
            echo -e "${YELLOW}⚠️  SKIPPED: $name (requires sudo)${NC}"
        fi
    else
        if eval "$command" > "$log_file" 2>&1; then
            echo -e "${GREEN}✅ PASSED: $name${NC}"
            PASSED_TESTS=$((PASSED_TESTS + 1))
        else
            echo -e "${RED}❌ FAILED: $name${NC}"
            echo "   Log: $log_file"
            FAILED_TESTS=$((FAILED_TESTS + 1))
        fi
    fi
}

# ═══════════════════════════════════════════════════════════════════
# PHASE 1: UNIT TESTS (Existing primal tests)
# ═══════════════════════════════════════════════════════════════════

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  PHASE 1: UNIT TESTS                                         ║"
echo "╚══════════════════════════════════════════════════════════════╝"

run_test_suite \
    "All Unit Tests" \
    "cargo test --all --lib" \
    false

# ═══════════════════════════════════════════════════════════════════
# PHASE 2: E2E TESTS (Atomic pattern tests)
# ═══════════════════════════════════════════════════════════════════

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  PHASE 2: E2E TESTS                                          ║"
echo "╚══════════════════════════════════════════════════════════════╝"

run_test_suite \
    "Tower Atomic E2E" \
    "cargo test --test tower_e2e -- --nocapture" \
    false

run_test_suite \
    "Node Atomic E2E" \
    "cargo test --test node_e2e -- --nocapture" \
    false

run_test_suite \
    "Nest Atomic E2E" \
    "cargo test --test nest_e2e -- --nocapture" \
    false

# ═══════════════════════════════════════════════════════════════════
# PHASE 3: CHAOS TESTS (Resilience tests)
# ═══════════════════════════════════════════════════════════════════

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  PHASE 3: CHAOS TESTS                                        ║"
echo "╚══════════════════════════════════════════════════════════════╝"

run_test_suite \
    "Tower Atomic Chaos" \
    "cargo test --test tower_chaos -- --nocapture" \
    true

run_test_suite \
    "Node Atomic Chaos" \
    "cargo test --test node_chaos -- --nocapture" \
    true

run_test_suite \
    "Nest Atomic Chaos" \
    "cargo test --test nest_chaos -- --nocapture" \
    true

# ═══════════════════════════════════════════════════════════════════
# PHASE 4: FAULT INJECTION TESTS
# ═══════════════════════════════════════════════════════════════════

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║  PHASE 4: FAULT INJECTION TESTS                              ║"
echo "╚══════════════════════════════════════════════════════════════╝"

run_test_suite \
    "Tower Atomic Fault" \
    "cargo test --test tower_fault -- --nocapture" \
    false

run_test_suite \
    "Node Atomic Fault" \
    "cargo test --test node_fault -- --nocapture" \
    false

run_test_suite \
    "Nest Atomic Fault" \
    "cargo test --test nest_fault -- --nocapture" \
    false

# ═══════════════════════════════════════════════════════════════════
# SUMMARY
# ═══════════════════════════════════════════════════════════════════

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                                                              ║"
echo "║                    TEST SUMMARY                              ║"
echo "║                                                              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "Total Tests:   $TOTAL_TESTS"
echo -e "${GREEN}Passed:        $PASSED_TESTS${NC}"
echo -e "${RED}Failed:        $FAILED_TESTS${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                                                              ║${NC}"
    echo -e "${GREEN}║               ✅ ALL TESTS PASSED! ✅                       ║${NC}"
    echo -e "${GREEN}║                                                              ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "🎊 NUCLEUS is PRODUCTION READY! 🎊"
    exit 0
else
    echo -e "${RED}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${RED}║                                                              ║${NC}"
    echo -e "${RED}║               ❌ SOME TESTS FAILED ❌                       ║${NC}"
    echo -e "${RED}║                                                              ║${NC}"
    echo -e "${RED}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo "⚠️  Review logs in $TEST_RESULTS_DIR/"
    exit 1
fi
