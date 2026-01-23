#!/bin/bash
# HTTPS Test Suite - Test against multiple real-world sites
# Date: January 23, 2026

echo "╔══════════════════════════════════════════════════════════════════════╗"
echo "║                                                                      ║"
echo "║           🧪 100% Pure Rust HTTPS - Test Suite 🧪                   ║"
echo "║                                                                      ║"
echo "╚══════════════════════════════════════════════════════════════════════╝"
echo ""

# Test endpoints (various TLS implementations)
ENDPOINTS=(
    "https://api.github.com/zen"                    # GitHub (Cloudflare)
    "https://www.google.com"                        # Google
    "https://www.cloudflare.com"                    # Cloudflare
    "https://www.mozilla.org"                       # Mozilla
    "https://httpbin.org/get"                       # HTTPBin (simple echo)
    "https://api.ipify.org?format=json"             # IP service
    "https://jsonplaceholder.typicode.com/posts/1"  # JSON placeholder
    "https://www.rust-lang.org"                     # Rust site
)

SOCKET="/tmp/songbird-nat0.sock"
LOG_DIR="/tmp/https-test-logs"
mkdir -p "$LOG_DIR"

echo "📊 Testing ${#ENDPOINTS[@]} endpoints..."
echo ""

PASSED=0
FAILED=0

for url in "${ENDPOINTS[@]}"; do
    echo "────────────────────────────────────────────────────────────────────"
    echo "🔗 Testing: $url"
    
    # Extract domain for logging
    DOMAIN=$(echo "$url" | sed -E 's|https?://([^/]+).*|\1|')
    LOG_FILE="$LOG_DIR/${DOMAIN//./_}.log"
    
    # Send request
    RESPONSE=$(echo "{\"jsonrpc\":\"2.0\",\"method\":\"http.request\",\"params\":{\"method\":\"GET\",\"url\":\"$url\"},\"id\":1}" | \
               timeout 30 nc -N -U "$SOCKET" 2>&1)
    
    EXIT_CODE=$?
    
    # Save response
    echo "$RESPONSE" > "$LOG_FILE"
    
    # Check result
    if echo "$RESPONSE" | grep -q '"result"'; then
        STATUS=$(echo "$RESPONSE" | jq -r '.result.status' 2>/dev/null || echo "unknown")
        echo "✅ SUCCESS (HTTP $STATUS)"
        ((PASSED++))
        
        # Show response preview
        BODY=$(echo "$RESPONSE" | jq -r '.result.body' 2>/dev/null | head -c 100)
        echo "   Preview: $BODY..."
    else
        ERROR=$(echo "$RESPONSE" | jq -r '.error.message' 2>/dev/null || echo "$RESPONSE")
        echo "❌ FAILED: $ERROR" | head -c 150
        echo ""
        ((FAILED++))
    fi
    
    echo ""
    sleep 1  # Rate limiting
done

echo "════════════════════════════════════════════════════════════════════"
echo ""
echo "📊 RESULTS:"
echo "   ✅ Passed: $PASSED / ${#ENDPOINTS[@]}"
echo "   ❌ Failed: $FAILED / ${#ENDPOINTS[@]}"
echo ""
echo "📁 Logs saved to: $LOG_DIR"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "🎉 ALL TESTS PASSED! 100% Pure Rust HTTPS is WORKING! 🎉"
    exit 0
else
    echo "🔍 Some tests failed. Check logs for details."
    exit 1
fi

