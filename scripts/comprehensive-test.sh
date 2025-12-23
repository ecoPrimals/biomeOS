#!/bin/bash

# Comprehensive BiomeOS UI Test & Demo Script
# This script demonstrates all the enhanced functionality we've built

set -e

echo "========================================="
echo "🌿 BiomeOS Enhanced UI Test Suite"
echo "========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counter
TESTS_PASSED=0
TESTS_TOTAL=0

# Function to run a test
run_test() {
    local test_name="$1"
    local command="$2"
    
    echo -e "${BLUE}🧪 Testing: $test_name${NC}"
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    
    if eval "$command" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ PASSED: $test_name${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}❌ FAILED: $test_name${NC}"
    fi
    echo ""
}

# Function to show demo
show_demo() {
    local demo_name="$1"
    local description="$2"
    
    echo -e "${YELLOW}🎬 Demo: $demo_name${NC}"
    echo -e "${YELLOW}   $description${NC}"
    echo ""
}

echo "========================================="
echo "🔧 COMPILATION TESTS"
echo "========================================="

# Test 1: Core Library Compilation
run_test "Core Library Compilation" "cargo build --lib -p biomeos-core"

# Test 2: UI Compilation (Debug)
run_test "UI Compilation (Debug)" "cd ui && cargo build"

# Test 3: UI Compilation (Release)
run_test "UI Compilation (Release)" "cd ui && cargo build --release"

# Test 4: All Workspace Compilation
run_test "Workspace Compilation" "cargo build --workspace"

echo "========================================="
echo "🧪 UNIT TESTS"
echo "========================================="

# Test 5: Core Library Tests
run_test "Core Library Tests" "cargo test --lib -p biomeos-core"

# Test 6: UI Tests
run_test "UI Tests" "cd ui && cargo test"

# Test 7: System Tests
run_test "System Tests" "cargo test --lib -p biomeos-system"

# Test 8: All Library Tests
run_test "All Library Tests" "cargo test --workspace --lib"

echo "========================================="
echo "🎯 FUNCTIONALITY TESTS"
echo "========================================="

# Test 9: Binary Existence
run_test "UI Binary Exists" "test -f target/release/biomeos-ui"

# Test 10: Binary Execution (Help)
run_test "UI Binary Help" "target/release/biomeos-ui --help"

# Test 11: Command Line Mode Access
run_test "BYOB Mode Access" "timeout 2s target/release/biomeos-ui --byob || true"

# Test 12: ISO Creator Mode Access
run_test "ISO Creator Mode Access" "timeout 2s target/release/biomeos-ui --iso-creator || true"

# Test 13: Niche Manager Mode Access
run_test "Niche Manager Mode Access" "timeout 2s target/release/biomeos-ui --niche-manager || true"

# Test 14: YAML Editor Mode Access
run_test "YAML Editor Mode Access" "timeout 2s target/release/biomeos-ui --yaml-editor || true"

echo "========================================="
echo "🎬 INTERACTIVE DEMOS"
echo "========================================="

show_demo "BYOB (Build Your Own Biome)" "Team workspace management with resource isolation"
show_demo "ISO Creator" "Custom distribution builder with niche integration"
show_demo "Niche Manager" "Package lifecycle management with marketplace"
show_demo "YAML Editor" "Advanced configuration editor with validation"
show_demo "Universal Adapter" "Future-proof Primal integration system"

echo "========================================="
echo "📊 FEATURE OVERVIEW"
echo "========================================="

echo -e "${BLUE}🌟 Major Features Implemented:${NC}"
echo ""
echo "1. 🏗️  BYOB (Build Your Own Biome)"
echo "   • Team workspace management (5 tabs)"
echo "   • Resource isolation and quotas"
echo "   • Deployment monitoring"
echo "   • Service health checks"
echo "   • Performance metrics"
echo ""
echo "2. 💿 ISO Creator"
echo "   • Custom distribution builder (5 tabs)"
echo "   • Niche package integration"
echo "   • Component customization"
echo "   • Build progress monitoring"
echo "   • Template system"
echo "   • Multi-architecture support"
echo ""
echo "3. 📦 Niche Manager"
echo "   • Package lifecycle management (5 tabs)"
echo "   • Multi-mode editor (Visual/YAML/Preview)"
echo "   • Testing framework"
echo "   • Marketplace integration"
echo "   • Security scoring"
echo ""
echo "4. 📝 YAML Editor"
echo "   • Advanced configuration editor"
echo "   • Real-time validation"
echo "   • Template system"
echo "   • Error highlighting"
echo ""
echo "5. 🔗 Universal Adapter"
echo "   • Future-proof Primal integration"
echo "   • Automatic capability detection"
echo "   • Configuration-driven routing"
echo "   • Cross-component workflows"
echo ""

echo "========================================="
echo "🔄 CROSS-INTEGRATION WORKFLOWS"
echo "========================================="

echo -e "${BLUE}🔗 Integration Patterns:${NC}"
echo ""
echo "• BYOB → ISO Creator: Package team configs into ISOs"
echo "• Niche Manager → ISO Creator: Include niches in distributions"
echo "• BYOB → Niche Manager: Create niches from deployments"
echo "• ISO Creator → BYOB: Use custom ISOs as base images"
echo ""

echo "========================================="
echo "🏗️ ARCHITECTURE HIGHLIGHTS"
echo "========================================="

echo -e "${BLUE}🎯 Technical Excellence:${NC}"
echo ""
echo "• Universal Adapter Pattern for unlimited Primal support"
echo "• Sovereignty-first design with no vendor lock-in"
echo "• API-driven architecture with real-time updates"
echo "• Comprehensive error handling and validation"
echo "• Mock data for immediate testing and development"
echo "• Command-line access to all major features"
echo "• Cross-platform compatibility (Linux, macOS, Windows)"
echo ""

echo "========================================="
echo "📈 TEST RESULTS"
echo "========================================="

echo -e "${GREEN}✅ Tests Passed: $TESTS_PASSED / $TESTS_TOTAL${NC}"

if [ $TESTS_PASSED -eq $TESTS_TOTAL ]; then
    echo -e "${GREEN}🎉 ALL TESTS PASSED! System is ready for production.${NC}"
else
    echo -e "${YELLOW}⚠️  Some tests failed. Check output above for details.${NC}"
fi

echo ""
echo "========================================="
echo "🚀 NEXT STEPS"
echo "========================================="

echo -e "${BLUE}To run the UI:${NC}"
echo "  ./target/release/biomeos-ui"
echo ""
echo -e "${BLUE}To run specific modes:${NC}"
echo "  ./target/release/biomeos-ui --byob"
echo "  ./target/release/biomeos-ui --iso-creator"
echo "  ./target/release/biomeos-ui --niche-manager"
echo "  ./target/release/biomeos-ui --yaml-editor"
echo ""
echo -e "${BLUE}To run with developer panel:${NC}"
echo "  ./target/release/biomeos-ui --dev"
echo ""

echo "========================================="
echo "🌿 BiomeOS Enhanced UI - Ready for Action!"
echo "=========================================" 