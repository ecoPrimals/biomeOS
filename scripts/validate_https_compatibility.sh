#!/usr/bin/env bash
# Comprehensive HTTPS Compatibility Validation
# Tests Tower Atomic (Songbird v5.12.0) against production sites
# Date: January 23, 2026

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TEST_HTTPS_BIN="/home/eastgate/Development/ecoPrimals/phase1/songbird/target/release/examples/test_https"
LOG_DIR="/tmp/songbird-validation-$(date +%Y%m%d-%H%M%S)"
TIMEOUT=15

# Create log directory
mkdir -p "$LOG_DIR"

# Test sites
declare -A TEST_SITES=(
    # AI/ML Platforms
    ["HuggingFace"]="https://huggingface.co"
    ["OpenAI"]="https://api.openai.com"
    ["Anthropic"]="https://api.anthropic.com"
    
    # Major Tech Companies
    ["Google"]="https://www.google.com"
    ["GitHub"]="https://github.com"
    ["Microsoft"]="https://www.microsoft.com"
    ["Amazon"]="https://www.amazon.com"
    ["Cloudflare"]="https://www.cloudflare.com"
    
    # Social/Content
    ["Reddit"]="https://www.reddit.com"
    ["Twitter"]="https://twitter.com"
    ["LinkedIn"]="https://www.linkedin.com"
    
    # Developer Tools
    ["NPM"]="https://www.npmjs.com"
    ["PyPI"]="https://pypi.org"
    ["Crates.io"]="https://crates.io"
    ["Docker Hub"]="https://hub.docker.com"
    
    # Cloud Providers
    ["AWS"]="https://aws.amazon.com"
    ["Azure"]="https://azure.microsoft.com"
    ["GCP"]="https://cloud.google.com"
    
    # CDN/Edge
    ["Fastly"]="https://www.fastly.com"
    ["Akamai"]="https://www.akamai.com"
    
    # Common Test Sites
    ["Example.com"]="https://example.com"
    ["HTTPBin"]="https://httpbin.org"
)

# Statistics
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
TLS13_SITES=0
TLS12_SITES=0
OTHER_FAILURES=0

# Results storage
declare -A RESULTS
declare -A TLS_VERSIONS
declare -A ERROR_MESSAGES

echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}🔒 Tower Atomic HTTPS Compatibility Validation${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""
echo -e "Test Binary: ${TEST_HTTPS_BIN}"
echo -e "Log Directory: ${LOG_DIR}"
echo -e "Timeout: ${TIMEOUT}s per test"
echo -e "Total Sites: ${#TEST_SITES[@]}"
echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

# Test function
test_site() {
    local name="$1"
    local url="$2"
    local log_file="$LOG_DIR/${name}.log"
    
    echo -n "Testing ${name} (${url})... "
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    # Run test with timeout
    if timeout "$TIMEOUT" env RUST_LOG=info "$TEST_HTTPS_BIN" "$url" > "$log_file" 2>&1; then
        # Check for TLS 1.3 handshake success
        if grep -q "TLS 1.3 handshake complete" "$log_file" || grep -q "Handshake complete" "$log_file"; then
            echo -e "${GREEN}✅ PASS (TLS 1.3)${NC}"
            PASSED_TESTS=$((PASSED_TESTS + 1))
            TLS13_SITES=$((TLS13_SITES + 1))
            RESULTS["$name"]="PASS"
            TLS_VERSIONS["$name"]="TLS 1.3"
            return 0
        elif grep -q "TLS 1.2" "$log_file"; then
            echo -e "${YELLOW}⚠️  TLS 1.2 only${NC}"
            TLS12_SITES=$((TLS12_SITES + 1))
            RESULTS["$name"]="TLS12"
            TLS_VERSIONS["$name"]="TLS 1.2"
            # Extract error message
            ERROR_MESSAGES["$name"]=$(grep -oP "error: \K.*" "$log_file" | head -1 || echo "TLS 1.2 not supported")
            return 1
        else
            # Check for specific errors
            if grep -q "early eof" "$log_file"; then
                echo -e "${YELLOW}⚠️  Early EOF${NC}"
                RESULTS["$name"]="EARLY_EOF"
                ERROR_MESSAGES["$name"]="Connection closed prematurely"
            elif grep -q "timeout" "$log_file"; then
                echo -e "${YELLOW}⚠️  Timeout${NC}"
                RESULTS["$name"]="TIMEOUT"
                ERROR_MESSAGES["$name"]="Connection timeout"
            elif grep -q "alert" "$log_file"; then
                local alert=$(grep -oP "alert: \K.*" "$log_file" | head -1)
                echo -e "${YELLOW}⚠️  Alert: $alert${NC}"
                RESULTS["$name"]="ALERT"
                ERROR_MESSAGES["$name"]="TLS Alert: $alert"
            else
                echo -e "${RED}❌ FAIL${NC}"
                RESULTS["$name"]="FAIL"
                ERROR_MESSAGES["$name"]=$(grep -oP "error: \K.*" "$log_file" | head -1 || echo "Unknown error")
            fi
            FAILED_TESTS=$((FAILED_TESTS + 1))
            OTHER_FAILURES=$((OTHER_FAILURES + 1))
            return 1
        fi
    else
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            echo -e "${YELLOW}⚠️  Timeout (${TIMEOUT}s)${NC}"
            RESULTS["$name"]="TIMEOUT"
            ERROR_MESSAGES["$name"]="Test timeout after ${TIMEOUT}s"
        else
            echo -e "${RED}❌ CRASH (exit ${exit_code})${NC}"
            RESULTS["$name"]="CRASH"
            ERROR_MESSAGES["$name"]="Process crashed with exit code $exit_code"
        fi
        FAILED_TESTS=$((FAILED_TESTS + 1))
        OTHER_FAILURES=$((OTHER_FAILURES + 1))
        return 1
    fi
}

# Run tests
echo -e "${BLUE}Running tests...${NC}"
echo ""

# Sort sites by name for consistent output
for site in $(printf '%s\n' "${!TEST_SITES[@]}" | sort); do
    url="${TEST_SITES[$site]}"
    test_site "$site" "$url" || true
done

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}📊 Test Results Summary${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

# Calculate percentages
PASS_PERCENT=$((PASSED_TESTS * 100 / TOTAL_TESTS))
FAIL_PERCENT=$((FAILED_TESTS * 100 / TOTAL_TESTS))
TLS13_PERCENT=$((TLS13_SITES * 100 / TOTAL_TESTS))

echo -e "Total Tests:     ${TOTAL_TESTS}"
echo -e "${GREEN}Passed:          ${PASSED_TESTS} (${PASS_PERCENT}%)${NC}"
echo -e "${RED}Failed:          ${FAILED_TESTS} (${FAIL_PERCENT}%)${NC}"
echo ""
echo -e "${GREEN}TLS 1.3 Sites:   ${TLS13_SITES} (${TLS13_PERCENT}%)${NC}"
echo -e "${YELLOW}TLS 1.2 Only:    ${TLS12_SITES}${NC}"
echo -e "${RED}Other Failures:  ${OTHER_FAILURES}${NC}"
echo ""

# Detailed results
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}📋 Detailed Results${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

echo -e "${GREEN}✅ PASSED (TLS 1.3):${NC}"
for site in $(echo "${!RESULTS[@]}" | tr ' ' '\n' | sort); do
    if [ "${RESULTS[$site]}" = "PASS" ]; then
        echo -e "  • ${site}: ${TEST_SITES[$site]}"
    fi
done
echo ""

if [ $TLS12_SITES -gt 0 ]; then
    echo -e "${YELLOW}⚠️  TLS 1.2 ONLY:${NC}"
    for site in $(echo "${!RESULTS[@]}" | tr ' ' '\n' | sort); do
        if [ "${RESULTS[$site]}" = "TLS12" ]; then
            echo -e "  • ${site}: ${TEST_SITES[$site]}"
            echo -e "    ${ERROR_MESSAGES[$site]}"
        fi
    done
    echo ""
fi

if [ $OTHER_FAILURES -gt 0 ]; then
    echo -e "${RED}❌ FAILED:${NC}"
    for site in $(echo "${!RESULTS[@]}" | tr ' ' '\n' | sort); do
        if [ "${RESULTS[$site]}" != "PASS" ] && [ "${RESULTS[$site]}" != "TLS12" ]; then
            echo -e "  • ${site}: ${TEST_SITES[$site]}"
            echo -e "    Status: ${RESULTS[$site]}"
            echo -e "    Error: ${ERROR_MESSAGES[$site]}"
        fi
    done
    echo ""
fi

# Recommendations
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}💡 Recommendations${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"
echo ""

if [ $PASS_PERCENT -ge 80 ]; then
    echo -e "${GREEN}✅ EXCELLENT!${NC} Tower Atomic has ${PASS_PERCENT}% compatibility!"
    echo ""
elif [ $PASS_PERCENT -ge 60 ]; then
    echo -e "${YELLOW}⚠️  GOOD${NC}, but room for improvement (${PASS_PERCENT}% passing)"
    echo ""
else
    echo -e "${RED}❌ NEEDS WORK${NC} - Only ${PASS_PERCENT}% passing"
    echo ""
fi

if [ $TLS12_SITES -gt 0 ]; then
    echo -e "📋 Next Steps:"
    echo -e "  1. Add TLS 1.2 support (${TLS12_SITES} sites need it)"
    echo -e "  2. Implement automatic fallback"
    echo -e "  3. See: Tower Atomic architecture document"
    echo ""
fi

if [ $OTHER_FAILURES -gt 0 ]; then
    echo -e "🔍 Debug:"
    echo -e "  • Check logs in: ${LOG_DIR}"
    echo -e "  • Focus on: $(echo "${!RESULTS[@]}" | tr ' ' '\n' | sort | while read site; do [ "${RESULTS[$site]}" != "PASS" ] && [ "${RESULTS[$site]}" != "TLS12" ] && echo -n "$site "; done)"
    echo ""
fi

# Log summary to file
SUMMARY_FILE="$LOG_DIR/SUMMARY.txt"
{
    echo "═══════════════════════════════════════════════════════"
    echo "Tower Atomic HTTPS Compatibility Validation"
    echo "Date: $(date)"
    echo "═══════════════════════════════════════════════════════"
    echo ""
    echo "Results:"
    echo "  Total: $TOTAL_TESTS"
    echo "  Passed: $PASSED_TESTS ($PASS_PERCENT%)"
    echo "  Failed: $FAILED_TESTS ($FAIL_PERCENT%)"
    echo ""
    echo "TLS Versions:"
    echo "  TLS 1.3: $TLS13_SITES ($TLS13_PERCENT%)"
    echo "  TLS 1.2 only: $TLS12_SITES"
    echo "  Other failures: $OTHER_FAILURES"
    echo ""
    echo "Passed Sites:"
    for site in $(echo "${!RESULTS[@]}" | tr ' ' '\n' | sort); do
        if [ "${RESULTS[$site]}" = "PASS" ]; then
            echo "  • $site: ${TEST_SITES[$site]}"
        fi
    done
    echo ""
    if [ $TLS12_SITES -gt 0 ]; then
        echo "TLS 1.2 Only Sites:"
        for site in $(echo "${!RESULTS[@]}" | tr ' ' '\n' | sort); do
            if [ "${RESULTS[$site]}" = "TLS12" ]; then
                echo "  • $site: ${TEST_SITES[$site]}"
            fi
        done
        echo ""
    fi
    if [ $OTHER_FAILURES -gt 0 ]; then
        echo "Failed Sites:"
        for site in $(echo "${!RESULTS[@]}" | tr ' ' '\n' | sort); do
            if [ "${RESULTS[$site]}" != "PASS" ] && [ "${RESULTS[$site]}" != "TLS12" ]; then
                echo "  • $site: ${TEST_SITES[$site]}"
                echo "    Status: ${RESULTS[$site]}"
                echo "    Error: ${ERROR_MESSAGES[$site]}"
            fi
        done
    fi
} > "$SUMMARY_FILE"

echo -e "📄 Summary saved to: ${SUMMARY_FILE}"
echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════${NC}"

# Exit code based on results
if [ $PASS_PERCENT -ge 80 ]; then
    exit 0  # Success
elif [ $PASS_PERCENT -ge 60 ]; then
    exit 1  # Warning
else
    exit 2  # Failure
fi

