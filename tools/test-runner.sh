#!/bin/bash
# biomeOS Comprehensive Test Runner
# 
# This script runs all test suites for biomeOS with focus on
# Songbird and NestGate eco-primals and recursive BYOB functionality.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEST_OUTPUT_DIR="${WORKSPACE_ROOT}/target/test-results"
COVERAGE_DIR="${WORKSPACE_ROOT}/target/coverage"

# Create output directories
mkdir -p "${TEST_OUTPUT_DIR}"
mkdir -p "${COVERAGE_DIR}"

echo -e "${BLUE}🚀 biomeOS Comprehensive Test Suite${NC}"
echo -e "${BLUE}====================================${NC}"
echo "Workspace: ${WORKSPACE_ROOT}"
echo "Test Output: ${TEST_OUTPUT_DIR}"
echo "Coverage: ${COVERAGE_DIR}"
echo ""

# Function to run tests with timing
run_test_suite() {
    local suite_name="$1"
    local test_command="$2"
    local start_time=$(date +%s)
    
    echo -e "${YELLOW}📋 Running ${suite_name}...${NC}"
    
    if eval "$test_command" > "${TEST_OUTPUT_DIR}/${suite_name}.log" 2>&1; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        echo -e "${GREEN}✅ ${suite_name} passed (${duration}s)${NC}"
        return 0
    else
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        echo -e "${RED}❌ ${suite_name} failed (${duration}s)${NC}"
        echo -e "${RED}   Check ${TEST_OUTPUT_DIR}/${suite_name}.log for details${NC}"
        return 1
    fi
}

# Function to check if cargo is available
check_cargo() {
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Cargo not found. Please install Rust and Cargo.${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Cargo found: $(cargo --version)${NC}"
}

# Function to build all crates
build_crates() {
    echo -e "${YELLOW}🔨 Building all crates...${NC}"
    
    cd "${WORKSPACE_ROOT}"
    
    if cargo build --all 2>&1 | tee "${TEST_OUTPUT_DIR}/build.log"; then
        echo -e "${GREEN}✅ Build successful${NC}"
    else
        echo -e "${RED}❌ Build failed${NC}"
        echo -e "${RED}   Check ${TEST_OUTPUT_DIR}/build.log for details${NC}"
        exit 1
    fi
}

# Function to run unit tests
run_unit_tests() {
    echo -e "${YELLOW}🧪 Running unit tests...${NC}"
    
    cd "${WORKSPACE_ROOT}"
    
    # Core tests
    run_test_suite "biomeos-core-tests" "cargo test -p biomeos-core --lib"
    
    # Manifest tests
    run_test_suite "biomeos-manifest-tests" "cargo test -p biomeos-manifest --lib"
    
    # System tests
    run_test_suite "biomeos-system-tests" "cargo test -p biomeos-system --lib"
    
    # Tools tests
    run_test_suite "biomeos-tools-tests" "cargo test -p biomeos-tools"
}

# Function to run integration tests
run_integration_tests() {
    echo -e "${YELLOW}🔄 Running integration tests...${NC}"
    
    cd "${WORKSPACE_ROOT}"
    
    # Core integration tests
    run_test_suite "core-integration" "cargo test -p biomeos-core --test integration_tests"
    
    # Recursive BYOB tests
    run_test_suite "recursive-biome-tests" "cargo test -p biomeos-manifest --test recursive_biome_tests"
    
    # Songbird+NestGate integration
    run_test_suite "songbird-nestgate-integration" "cargo test -p biomeos-manifest --test songbird_nestgate_integration_tests"
    
    # Gaming tournament tests
    run_test_suite "gaming-tournament-integration" "cargo test -p biomeos-manifest --test gaming_tournament_integration_tests"
}

# Function to run performance tests
run_performance_tests() {
    echo -e "${YELLOW}⚡ Running performance tests...${NC}"
    
    cd "${WORKSPACE_ROOT}"
    
    # Recursive deployment performance
    run_test_suite "recursive-deployment-perf" "cargo test --release recursive_deployment_performance --test performance_tests"
    
    # Songbird orchestration performance
    run_test_suite "songbird-orchestration-perf" "cargo test --release songbird_orchestration_performance --test performance_tests"
    
    # NestGate storage performance
    run_test_suite "nestgate-storage-perf" "cargo test --release nestgate_storage_performance --test performance_tests"
    
    # Gaming tournament stress test
    run_test_suite "tournament-stress-test" "cargo test --release tournament_stress_test --test stress_tests"
}

# Function to run template validation tests
run_template_tests() {
    echo -e "${YELLOW}📋 Running template validation tests...${NC}"
    
    cd "${WORKSPACE_ROOT}"
    
    # Validate all template files
    for template in templates/*.biome.yaml templates/*.yaml; do
        if [ -f "$template" ]; then
            template_name=$(basename "$template" .biome.yaml)
            template_name=$(basename "$template_name" .yaml)
            
            echo -e "  🔍 Validating ${template_name}..."
            
            if cargo run --bin manifest-validator -- "$template" > "${TEST_OUTPUT_DIR}/template-${template_name}.log" 2>&1; then
                echo -e "    ${GREEN}✅ ${template_name} valid${NC}"
            else
                echo -e "    ${RED}❌ ${template_name} invalid${NC}"
                echo -e "    ${RED}     Check ${TEST_OUTPUT_DIR}/template-${template_name}.log${NC}"
            fi
        fi
    done
}

# Function to run BYOB scenario tests
run_byob_scenario_tests() {
    echo -e "${YELLOW}🎮 Running BYOB scenario tests...${NC}"
    
    cd "${WORKSPACE_ROOT}"
    
    # Gaming tournament scenario
    echo -e "  🎯 Testing gaming tournament scenario..."
    if cargo run --bin byob -- validate templates/gaming-tournament-recursive.biome.yaml > "${TEST_OUTPUT_DIR}/byob-gaming-tournament.log" 2>&1; then
        echo -e "    ${GREEN}✅ Gaming tournament scenario valid${NC}"
    else
        echo -e "    ${RED}❌ Gaming tournament scenario failed${NC}"
    fi
    
    # Multi-region deployment scenario
    echo -e "  🌍 Testing multi-region deployment..."
    cat > "${TEST_OUTPUT_DIR}/multi-region-test.yaml" << 'YAML'
apiVersion: v1
kind: Biome
metadata:
  name: "multi-region-test"
  specialization: "edge-computing"
topology:
  topology_type: "recursive"
  orchestration_ring:
    topology: "ring"
    instances: 5
    regions: ["us-east", "us-west", "eu-west", "eu-central", "ap-southeast"]
    template: "songbird-orchestrator"
iterative:
  ring-formation:
    pattern: "ring"
    instances: 5
    configuration:
      ring_size: 5
      redundancy: 3
YAML
    
    if cargo run --bin byob -- validate "${TEST_OUTPUT_DIR}/multi-region-test.yaml" > "${TEST_OUTPUT_DIR}/byob-multi-region.log" 2>&1; then
        echo -e "    ${GREEN}✅ Multi-region deployment valid${NC}"
    else
        echo -e "    ${RED}❌ Multi-region deployment failed${NC}"
    fi
}

# Function to generate test coverage
generate_coverage() {
    echo -e "${YELLOW}📊 Generating test coverage...${NC}"
    
    cd "${WORKSPACE_ROOT}"
    
    if command -v cargo-tarpaulin &> /dev/null; then
        echo -e "  🔍 Using cargo-tarpaulin for coverage..."
        
        cargo tarpaulin \
            --all-features \
            --workspace \
            --timeout 120 \
            --out Html \
            --output-dir "${COVERAGE_DIR}" \
            > "${TEST_OUTPUT_DIR}/coverage.log" 2>&1
        
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✅ Coverage report generated in ${COVERAGE_DIR}/tarpaulin-report.html${NC}"
        else
            echo -e "${YELLOW}⚠️  Coverage generation failed, but continuing...${NC}"
        fi
    else
        echo -e "${YELLOW}⚠️  cargo-tarpaulin not found, skipping coverage...${NC}"
        echo -e "    Install with: cargo install cargo-tarpaulin"
    fi
}

# Function to run ecosystem health check
run_ecosystem_health_check() {
    echo -e "${YELLOW}🌱 Running ecosystem health check...${NC}"
    
    cd "${WORKSPACE_ROOT}"
    
    # Check if all eco-primals can be instantiated
    echo -e "  🐻 Testing BearDog instantiation..."
    echo -e "  🐦 Testing Songbird instantiation..."
    echo -e "  🏠 Testing NestGate instantiation..."
    echo -e "  🍄 Testing Toadstool instantiation..."
    echo -e "  🐿️  Testing Squirrel instantiation..."
    
    if cargo test --release ecosystem_health_check > "${TEST_OUTPUT_DIR}/ecosystem-health.log" 2>&1; then
        echo -e "${GREEN}✅ Ecosystem health check passed${NC}"
    else
        echo -e "${RED}❌ Ecosystem health check failed${NC}"
        echo -e "${RED}   Check ${TEST_OUTPUT_DIR}/ecosystem-health.log for details${NC}"
    fi
}

# Function to run compatibility tests
run_compatibility_tests() {
    echo -e "${YELLOW}�� Running compatibility tests...${NC}"
    
    cd "${WORKSPACE_ROOT}"
    
    # Test YAML serialization compatibility
    run_test_suite "yaml-compatibility" "cargo test yaml_compatibility"
    
    # Test version compatibility
    run_test_suite "version-compatibility" "cargo test version_compatibility"
    
    # Test cross-platform compatibility
    run_test_suite "cross-platform-compatibility" "cargo test cross_platform"
}

# Function to print test summary
print_test_summary() {
    echo ""
    echo -e "${BLUE}📊 Test Summary${NC}"
    echo -e "${BLUE}===============${NC}"
    
    local total_tests=0
    local passed_tests=0
    local failed_tests=0
    
    for log_file in "${TEST_OUTPUT_DIR}"/*.log; do
        if [ -f "$log_file" ]; then
            total_tests=$((total_tests + 1))
            
            if grep -q "test result: ok" "$log_file" 2>/dev/null || 
               grep -q "✅" "$log_file" 2>/dev/null; then
                passed_tests=$((passed_tests + 1))
            else
                failed_tests=$((failed_tests + 1))
                echo -e "${RED}❌ $(basename "$log_file" .log)${NC}"
            fi
        fi
    done
    
    echo -e "Total test suites: ${total_tests}"
    echo -e "${GREEN}Passed: ${passed_tests}${NC}"
    echo -e "${RED}Failed: ${failed_tests}${NC}"
    
    if [ $failed_tests -eq 0 ]; then
        echo -e "${GREEN}🎉 All tests passed!${NC}"
        echo -e "${GREEN}   biomeOS ecosystem is healthy and ready for experimentation${NC}"
        return 0
    else
        echo -e "${RED}💥 Some tests failed${NC}"
        echo -e "${RED}   Check individual log files in ${TEST_OUTPUT_DIR}${NC}"
        return 1
    fi
}

# Main execution
main() {
    local start_time=$(date +%s)
    
    # Parse command line arguments
    RUN_UNIT=true
    RUN_INTEGRATION=true
    RUN_PERFORMANCE=false
    RUN_COVERAGE=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --unit-only)
                RUN_INTEGRATION=false
                RUN_PERFORMANCE=false
                shift
                ;;
            --integration-only)
                RUN_UNIT=false
                RUN_PERFORMANCE=false
                shift
                ;;
            --performance)
                RUN_PERFORMANCE=true
                shift
                ;;
            --coverage)
                RUN_COVERAGE=true
                shift
                ;;
            --all)
                RUN_UNIT=true
                RUN_INTEGRATION=true
                RUN_PERFORMANCE=true
                RUN_COVERAGE=true
                shift
                ;;
            --help)
                echo "Usage: $0 [options]"
                echo "  --unit-only      Run only unit tests"
                echo "  --integration-only Run only integration tests"
                echo "  --performance    Include performance tests"
                echo "  --coverage       Generate coverage report"
                echo "  --all           Run all tests including performance and coverage"
                echo "  --help          Show this help"
                exit 0
                ;;
            *)
                echo "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done
    
    # Pre-flight checks
    check_cargo
    
    # Build phase
    build_crates
    
    # Test phases
    if [ "$RUN_UNIT" = true ]; then
        run_unit_tests
    fi
    
    if [ "$RUN_INTEGRATION" = true ]; then
        run_integration_tests
        run_template_tests
        run_byob_scenario_tests
        run_compatibility_tests
    fi
    
    if [ "$RUN_PERFORMANCE" = true ]; then
        run_performance_tests
    fi
    
    # Health checks
    run_ecosystem_health_check
    
    # Coverage generation
    if [ "$RUN_COVERAGE" = true ]; then
        generate_coverage
    fi
    
    # Summary
    local end_time=$(date +%s)
    local total_duration=$((end_time - start_time))
    
    echo ""
    echo -e "${BLUE}⏱️  Total execution time: ${total_duration}s${NC}"
    
    print_test_summary
}

# Execute main function with all arguments
main "$@"
