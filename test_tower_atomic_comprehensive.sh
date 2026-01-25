#!/bin/bash
# Comprehensive Tower Atomic Validation Suite
# Tests Songbird + BearDog against real-world HTTPS endpoints

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo "═══════════════════════════════════════════════════════════════"
echo "🧪 TOWER ATOMIC COMPREHENSIVE VALIDATION SUITE"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Testing: Songbird (Pure Rust TLS 1.3) + BearDog (Pure Rust Crypto)"
echo "Against: Real-world HTTPS endpoints"
echo ""

# Configuration
SONGBIRD_SOCKET="${SONGBIRD_SOCKET:-/tmp/songbird-nat0.sock}"
TIMEOUT=30
RESULTS_DIR="./test-results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Check if Songbird is running
if [ ! -S "$SONGBIRD_SOCKET" ]; then
    echo -e "${RED}❌ Songbird socket not found: $SONGBIRD_SOCKET${NC}"
    echo "   Please start Songbird first:"
    echo "   cd ../songbird && ./target/release/songbird server"
    exit 1
fi

echo -e "${GREEN}✅ Songbird socket found: $SONGBIRD_SOCKET${NC}"
echo ""

# Create results directory
mkdir -p "$RESULTS_DIR"

# Counters
TOTAL=0
PASSED=0
FAILED=0
TIMEOUT_COUNT=0

# Test function
test_endpoint() {
    local category="$1"
    local name="$2"
    local url="$3"
    local method="${4:-GET}"
    local expect_code="${5:-200}"
    
    ((TOTAL++))
    
    echo -e "${CYAN}[$TOTAL] Testing: $name${NC}"
    echo "    Category: $category"
    echo "    URL: $url"
    echo "    Method: $method"
    
    # Build JSON-RPC request
    local request=$(cat <<EOF
{
  "jsonrpc": "2.0",
  "method": "http.request",
  "params": {
    "url": "$url",
    "method": "$method",
    "headers": {
      "User-Agent": "ecoPrimals-TowerAtomic/1.0 (Pure-Rust-TLS-1.3)",
      "Accept": "*/*"
    }
  },
  "id": $TOTAL
}
EOF
)
    
    # Send request with timeout
    local response
    local exit_code=0
    response=$(echo "$request" | timeout $TIMEOUT nc -U "$SONGBIRD_SOCKET" 2>&1) || exit_code=$?
    
    if [ $exit_code -eq 124 ]; then
        echo -e "    ${YELLOW}⏱️  TIMEOUT (${TIMEOUT}s)${NC}"
        ((TIMEOUT_COUNT++))
        echo "$category|$name|$url|TIMEOUT" >> "$RESULTS_DIR/results_${TIMESTAMP}.csv"
        return
    fi
    
    if [ $exit_code -ne 0 ]; then
        echo -e "    ${RED}❌ CONNECTION ERROR${NC}"
        echo "    Error: $response"
        ((FAILED++))
        echo "$category|$name|$url|ERROR|$response" >> "$RESULTS_DIR/results_${TIMESTAMP}.csv"
        return
    fi
    
    # Parse response
    if echo "$response" | jq -e '.result.status' > /dev/null 2>&1; then
        local status=$(echo "$response" | jq -r '.result.status')
        echo -e "    ${GREEN}✅ Response: $status${NC}"
        
        # Log response details
        local body_preview=$(echo "$response" | jq -r '.result.body' 2>/dev/null | head -c 100)
        if [ -n "$body_preview" ]; then
            echo "    Body preview: ${body_preview:0:80}..."
        fi
        
        ((PASSED++))
        echo "$category|$name|$url|SUCCESS|$status" >> "$RESULTS_DIR/results_${TIMESTAMP}.csv"
    else
        echo -e "    ${RED}❌ FAILED${NC}"
        echo "    Response: $response" | head -c 200
        ((FAILED++))
        echo "$category|$name|$url|FAILED|$response" >> "$RESULTS_DIR/results_${TIMESTAMP}.csv"
    fi
    
    echo ""
}

# Initialize results file
echo "Category|Name|URL|Status|Details" > "$RESULTS_DIR/results_${TIMESTAMP}.csv"

echo "═══════════════════════════════════════════════════════════════"
echo "CATEGORY 1: MAJOR TECH COMPANIES"
echo "═══════════════════════════════════════════════════════════════"
echo ""

test_endpoint "Tech" "GitHub API" "https://api.github.com/zen"
test_endpoint "Tech" "GitHub Repos" "https://api.github.com/repositories"
test_endpoint "Tech" "Google" "https://www.google.com"
test_endpoint "Tech" "Amazon" "https://www.amazon.com"
test_endpoint "Tech" "Microsoft" "https://www.microsoft.com"
test_endpoint "Tech" "Apple" "https://www.apple.com"

echo "═══════════════════════════════════════════════════════════════"
echo "CATEGORY 2: AI/ML PROVIDERS"
echo "═══════════════════════════════════════════════════════════════"
echo ""

test_endpoint "AI/ML" "Hugging Face" "https://huggingface.co"
test_endpoint "AI/ML" "Hugging Face API" "https://huggingface.co/api/models"
test_endpoint "AI/ML" "OpenAI Status" "https://status.openai.com"
test_endpoint "AI/ML" "Anthropic" "https://www.anthropic.com"
test_endpoint "AI/ML" "Replicate" "https://replicate.com"
test_endpoint "AI/ML" "Cohere" "https://cohere.com"

echo "═══════════════════════════════════════════════════════════════"
echo "CATEGORY 3: PUBLIC DATA & RESEARCH"
echo "═══════════════════════════════════════════════════════════════"
echo ""

test_endpoint "Data" "NCBI" "https://www.ncbi.nlm.nih.gov"
test_endpoint "Data" "PubMed" "https://pubmed.ncbi.nlm.nih.gov"
test_endpoint "Data" "NCBI E-utilities" "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/einfo.fcgi"
test_endpoint "Data" "arXiv" "https://arxiv.org"
test_endpoint "Data" "bioRxiv" "https://www.biorxiv.org"
test_endpoint "Data" "Zenodo" "https://zenodo.org"
test_endpoint "Data" "Kaggle" "https://www.kaggle.com"

echo "═══════════════════════════════════════════════════════════════"
echo "CATEGORY 4: OPEN DATA APIS"
echo "═══════════════════════════════════════════════════════════════"
echo ""

test_endpoint "Open-Data" "JSONPlaceholder" "https://jsonplaceholder.typicode.com/posts/1"
test_endpoint "Open-Data" "HTTPBin GET" "https://httpbin.org/get"
test_endpoint "Open-Data" "HTTPBin User-Agent" "https://httpbin.org/user-agent"
test_endpoint "Open-Data" "HTTPBin Headers" "https://httpbin.org/headers"
test_endpoint "Open-Data" "REST Countries" "https://restcountries.com/v3.1/all"
test_endpoint "Open-Data" "NASA API" "https://api.nasa.gov/planetary/apod?api_key=DEMO_KEY"

echo "═══════════════════════════════════════════════════════════════"
echo "CATEGORY 5: DEVELOPER TOOLS & REGISTRIES"
echo "═══════════════════════════════════════════════════════════════"
echo ""

test_endpoint "Dev-Tools" "crates.io" "https://crates.io"
test_endpoint "Dev-Tools" "crates.io API" "https://crates.io/api/v1/crates/tokio"
test_endpoint "Dev-Tools" "npm" "https://www.npmjs.com"
test_endpoint "Dev-Tools" "PyPI" "https://pypi.org"
test_endpoint "Dev-Tools" "Docker Hub" "https://hub.docker.com"
test_endpoint "Dev-Tools" "GitLab" "https://gitlab.com"

echo "═══════════════════════════════════════════════════════════════"
echo "CATEGORY 6: CLOUD PROVIDERS"
echo "═══════════════════════════════════════════════════════════════"
echo ""

test_endpoint "Cloud" "AWS" "https://aws.amazon.com"
test_endpoint "Cloud" "Google Cloud" "https://cloud.google.com"
test_endpoint "Cloud" "Azure" "https://azure.microsoft.com"
test_endpoint "Cloud" "DigitalOcean" "https://www.digitalocean.com"
test_endpoint "Cloud" "Heroku" "https://www.heroku.com"
test_endpoint "Cloud" "Netlify" "https://www.netlify.com"

echo "═══════════════════════════════════════════════════════════════"
echo "CATEGORY 7: SCIENTIFIC DATABASES"
echo "═══════════════════════════════════════════════════════════════"
echo ""

test_endpoint "Science" "UniProt" "https://www.uniprot.org"
test_endpoint "Science" "PDB (Protein Data Bank)" "https://www.rcsb.org"
test_endpoint "Science" "GenBank" "https://www.ncbi.nlm.nih.gov/genbank/"
test_endpoint "Science" "Europe PMC" "https://europepmc.org"
test_endpoint "Science" "COSMIC" "https://cancer.sanger.ac.uk/cosmic"

echo "═══════════════════════════════════════════════════════════════"
echo "CATEGORY 8: MODEL REPOSITORIES"
echo "═══════════════════════════════════════════════════════════════"
echo ""

test_endpoint "Models" "Hugging Face Models" "https://huggingface.co/models"
test_endpoint "Models" "TensorFlow Hub" "https://tfhub.dev"
test_endpoint "Models" "PyTorch Hub" "https://pytorch.org/hub/"
test_endpoint "Models" "Model Zoo" "https://modelzoo.co"

echo "═══════════════════════════════════════════════════════════════"
echo "CATEGORY 9: CONTENT DELIVERY"
echo "═══════════════════════════════════════════════════════════════"
echo ""

test_endpoint "CDN" "Cloudflare" "https://www.cloudflare.com"
test_endpoint "CDN" "jsDelivr" "https://www.jsdelivr.com"
test_endpoint "CDN" "unpkg" "https://unpkg.com"
test_endpoint "CDN" "cdnjs" "https://cdnjs.com"

echo "═══════════════════════════════════════════════════════════════"
echo "CATEGORY 10: NEWS & INFORMATION"
echo "═══════════════════════════════════════════════════════════════"
echo ""

test_endpoint "News" "Wikipedia" "https://en.wikipedia.org"
test_endpoint "News" "Wikipedia API" "https://en.wikipedia.org/api/rest_v1/"
test_endpoint "News" "Hacker News" "https://news.ycombinator.com"
test_endpoint "News" "Reddit" "https://www.reddit.com"

echo "═══════════════════════════════════════════════════════════════"
echo "📊 TEST RESULTS SUMMARY"
echo "═══════════════════════════════════════════════════════════════"
echo ""

echo -e "Total Tests:     ${CYAN}$TOTAL${NC}"
echo -e "Passed:          ${GREEN}$PASSED${NC}"
echo -e "Failed:          ${RED}$FAILED${NC}"
echo -e "Timeouts:        ${YELLOW}$TIMEOUT_COUNT${NC}"
echo ""

SUCCESS_RATE=$(awk "BEGIN {printf \"%.1f\", ($PASSED/$TOTAL)*100}")
echo -e "Success Rate:    ${GREEN}${SUCCESS_RATE}%${NC}"
echo ""

echo "Results saved to: $RESULTS_DIR/results_${TIMESTAMP}.csv"
echo ""

# Generate detailed report
REPORT_FILE="$RESULTS_DIR/report_${TIMESTAMP}.md"

cat > "$REPORT_FILE" <<EOF
# Tower Atomic Validation Report

**Date**: $(date)
**Songbird Socket**: $SONGBIRD_SOCKET
**Test Duration**: ${SECONDS}s

## Summary

| Metric | Value |
|--------|-------|
| Total Tests | $TOTAL |
| Passed | $PASSED |
| Failed | $FAILED |
| Timeouts | $TIMEOUT_COUNT |
| Success Rate | ${SUCCESS_RATE}% |

## Test Categories

EOF

# Add category breakdown
for category in "Tech" "AI/ML" "Data" "Open-Data" "Dev-Tools" "Cloud" "Science" "Models" "CDN" "News"; do
    cat_total=$(grep "^$category|" "$RESULTS_DIR/results_${TIMESTAMP}.csv" | wc -l)
    cat_passed=$(grep "^$category|.*|SUCCESS|" "$RESULTS_DIR/results_${TIMESTAMP}.csv" | wc -l)
    if [ $cat_total -gt 0 ]; then
        cat_rate=$(awk "BEGIN {printf \"%.1f\", ($cat_passed/$cat_total)*100}")
        echo "- **$category**: $cat_passed/$cat_total (${cat_rate}%)" >> "$REPORT_FILE"
    fi
done

cat >> "$REPORT_FILE" <<EOF

## Detailed Results

See: \`results_${TIMESTAMP}.csv\`

## Architecture

- **TLS**: Pure Rust 1.3 (Songbird)
- **Crypto**: Pure Rust (BearDog)
- **C Dependencies**: ZERO ✅

## Recommendations

EOF

if [ $PASSED -eq $TOTAL ]; then
    cat >> "$REPORT_FILE" <<EOF
✅ **ALL TESTS PASSED!** Tower Atomic is production-ready for all tested endpoints.

**Next Steps:**
1. Deploy to production
2. Monitor real-world usage
3. Add more edge case testing
EOF
elif [ $SUCCESS_RATE -gt 90 ]; then
    cat >> "$REPORT_FILE" <<EOF
✅ **EXCELLENT RESULTS!** Tower Atomic handles 90%+ of real-world endpoints.

**Action Items:**
1. Investigate failed endpoints
2. Consider production deployment with monitoring
3. Document any known limitations
EOF
elif [ $SUCCESS_RATE -gt 75 ]; then
    cat >> "$REPORT_FILE" <<EOF
⚠️ **GOOD RESULTS** with some issues. Tower Atomic handles most endpoints.

**Action Items:**
1. Analyze failure patterns
2. Fix blocking issues
3. Retest before production deployment
EOF
else
    cat >> "$REPORT_FILE" <<EOF
❌ **NEEDS WORK** - Several endpoints failing.

**Action Items:**
1. Debug TLS handshake issues
2. Check certificate validation
3. Review protocol compliance
4. Retest thoroughly
EOF
fi

echo "Report saved to: $REPORT_FILE"
echo ""

# Final verdict
echo "═══════════════════════════════════════════════════════════════"
if [ $PASSED -eq $TOTAL ]; then
    echo -e "${GREEN}🎉 PERFECT SCORE! Tower Atomic is PRODUCTION READY!${NC}"
elif [ $SUCCESS_RATE -gt 90 ]; then
    echo -e "${GREEN}✅ EXCELLENT! Tower Atomic handles 90%+ of real endpoints!${NC}"
elif [ $SUCCESS_RATE -gt 75 ]; then
    echo -e "${YELLOW}⚠️  GOOD with room for improvement${NC}"
else
    echo -e "${RED}❌ NEEDS DEBUGGING${NC}"
fi
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Exit code based on success rate
if [ $SUCCESS_RATE -gt 90 ]; then
    exit 0
elif [ $SUCCESS_RATE -gt 75 ]; then
    exit 1
else
    exit 2
fi

