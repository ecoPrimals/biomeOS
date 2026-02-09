#!/usr/bin/env bash
# Bootstrap Sequence Validation Test
# Tests biomeOS bootstrap mode and Tower Atomic genesis

set -e

echo "╔════════════════════════════════════════════════════════════════════╗"
echo "║           Bootstrap Sequence Validation Test                      ║"
echo "╚════════════════════════════════════════════════════════════════════╝"
echo

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test results
TESTS_PASSED=0
TESTS_FAILED=0

# Helper functions
pass() {
    echo -e "${GREEN}✅ PASS${NC}: $1"
    ((TESTS_PASSED++))
}

fail() {
    echo -e "${RED}❌ FAIL${NC}: $1"
    ((TESTS_FAILED++))
}

info() {
    echo -e "${YELLOW}ℹ️  INFO${NC}: $1"
}

check_file_exists() {
    if [ -f "$1" ]; then
        pass "File exists: $1"
        return 0
    else
        fail "File missing: $1"
        return 1
    fi
}

check_socket_exists() {
    if [ -S "$1" ]; then
        pass "Socket exists: $1"
        return 0
    else
        fail "Socket missing: $1"
        return 1
    fi
}

# Test 1: Verify build artifacts exist
echo "🔍 Test 1: Verify Build Artifacts"
echo "=================================="

check_file_exists "target/debug/biomeos" || \
check_file_exists "target/release/biomeos" || \
    info "Building biomeOS binary..."

check_file_exists "graphs/tower_atomic_bootstrap.toml"
check_file_exists "crates/biomeos-atomic-deploy/src/mode.rs"
check_file_exists "crates/biomeos-atomic-deploy/src/nucleation.rs"

echo

# Test 2: Clean environment (no existing ecosystem)
echo "🧹 Test 2: Clean Environment"
echo "============================"

info "Stopping any running primals..."
pkill -f "beardog|songbird|biomeos|nucleus" 2>/dev/null || true
sleep 1

info "Removing existing sockets..."
rm -f /tmp/beardog-*.sock /tmp/songbird-*.sock /tmp/biomeos-*.sock /tmp/neural-api-*.sock 2>/dev/null || true

if [ ! -S "/tmp/beardog-1894e909e454.sock" ] && [ ! -S "/tmp/songbird-1894e909e454.sock" ]; then
    pass "Environment clean (no Tower Atomic sockets)"
else
    fail "Environment not clean (sockets still exist)"
fi

echo

# Test 3: Verify mode detection code exists
echo "🔍 Test 3: Mode Detection Implementation"
echo "========================================"

if grep -q "pub enum BiomeOsMode" crates/biomeos-atomic-deploy/src/mode.rs; then
    pass "BiomeOsMode enum defined"
else
    fail "BiomeOsMode enum not found"
fi

if grep -q "pub async fn detect" crates/biomeos-atomic-deploy/src/mode.rs; then
    pass "Mode detection method exists"
else
    fail "Mode detection method not found"
fi

echo

# Test 4: Verify socket nucleation code exists
echo "🔍 Test 4: Socket Nucleation Implementation"
echo "==========================================="

if grep -q "pub struct SocketNucleation" crates/biomeos-atomic-deploy/src/nucleation.rs; then
    pass "SocketNucleation struct defined"
else
    fail "SocketNucleation struct not found"
fi

if grep -q "pub fn assign_socket" crates/biomeos-atomic-deploy/src/nucleation.rs; then
    pass "Socket assignment method exists"
else
    fail "Socket assignment method not found"
fi

echo

# Test 5: Verify bootstrap sequence code exists
echo "🔍 Test 5: Bootstrap Sequence Implementation"
echo "============================================"

if grep -q "execute_bootstrap_sequence" crates/biomeos-atomic-deploy/src/neural_api_server.rs; then
    pass "Bootstrap sequence method exists"
else
    fail "Bootstrap sequence method not found"
fi

if grep -q "transition_to_coordinated" crates/biomeos-atomic-deploy/src/neural_api_server.rs; then
    pass "Mode transition method exists"
else
    fail "Mode transition method not found"
fi

echo

# Test 6: Verify bootstrap graph structure
echo "🔍 Test 6: Bootstrap Graph Validation"
echo "====================================="

if grep -q "germinate_beardog" graphs/tower_atomic_bootstrap.toml; then
    pass "BearDog germination node exists"
else
    fail "BearDog germination node missing"
fi

if grep -q "germinate_songbird" graphs/tower_atomic_bootstrap.toml; then
    pass "Songbird germination node exists"
else
    fail "Songbird germination node missing"
fi

if grep -q 'depends_on.*germinate_beardog' graphs/tower_atomic_bootstrap.toml; then
    pass "Songbird depends on BearDog (genetic bonding)"
else
    fail "Songbird dependency on BearDog missing"
fi

if grep -q "validate_tower" graphs/tower_atomic_bootstrap.toml; then
    pass "Tower validation node exists"
else
    fail "Tower validation node missing"
fi

echo

# Test 7: Verify primal binaries exist (for actual execution test)
echo "🔍 Test 7: Primal Binary Availability"
echo "====================================="

PLASMID_BIN="../../../plasmidBin"

if [ -d "$PLASMID_BIN" ]; then
    info "Checking plasmidBin for primal binaries..."
    
    if find "$PLASMID_BIN" -name "beardog*" -type f | grep -q .; then
        pass "BearDog binary available in plasmidBin"
    else
        fail "BearDog binary not found in plasmidBin"
    fi
    
    if find "$PLASMID_BIN" -name "songbird*" -type f | grep -q .; then
        pass "Songbird binary available in plasmidBin"
    else
        fail "Songbird binary not found in plasmidBin"
    fi
else
    info "plasmidBin not found - primals would need to be built"
fi

echo

# Test 8: Code compilation test
echo "🔍 Test 8: Code Compilation"
echo "==========================="

info "Compiling biomeos-atomic-deploy..."
if cargo build --package biomeos-atomic-deploy 2>&1 | grep -q "Finished"; then
    pass "biomeos-atomic-deploy compiles successfully"
else
    fail "biomeos-atomic-deploy compilation failed"
fi

echo

# Summary
echo "╔════════════════════════════════════════════════════════════════════╗"
echo "║                        Test Summary                                ║"
echo "╚════════════════════════════════════════════════════════════════════╝"
echo
echo "Tests Passed: ${TESTS_PASSED}"
echo "Tests Failed: ${TESTS_FAILED}"
echo

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ All validation tests passed!${NC}"
    echo
    echo "🎯 READY FOR LIVE BOOTSTRAP TEST"
    echo
    echo "To test bootstrap sequence:"
    echo "  1. Ensure environment is clean (no Tower Atomic)"
    echo "  2. Start biomeOS: cargo run --bin nucleus"
    echo "  3. Watch for bootstrap sequence execution"
    echo "  4. Verify Tower Atomic sockets created"
    echo "  5. Verify mode transition to Coordinated"
    exit 0
else
    echo -e "${RED}❌ Some validation tests failed${NC}"
    echo
    echo "Fix the failures above before attempting live bootstrap test"
    exit 1
fi

