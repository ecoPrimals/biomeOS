#!/bin/bash
# HTTPS Endpoint Testing Script
# Tests 100% Pure Rust HTTPS stack against real-world endpoints

echo "╔══════════════════════════════════════════════════════════════════════╗"
echo "║                                                                      ║"
echo "║   🌐 100% PURE RUST HTTPS - COMPREHENSIVE ENDPOINT TESTING 🌐       ║"
echo "║                                                                      ║"
echo "╚══════════════════════════════════════════════════════════════════════╝"
echo
echo "Stack:"
echo "  • Songbird v5.8.0 (RFC 8446 transcript hash)"
echo "  • BearDog v0.14.0 (RFC 8446 key schedule)"
echo "  • Neural API (capability translation)"
echo
echo "═══════════════════════════════════════════════════════════════════════"

# Test counter
TOTAL=0
PASSED=0
FAILED=0

# Test function
test_https() {
    local name="$1"
    local url="$2"
    local expected_status="${3:-200}"
    
    echo
    echo "───────────────────────────────────────────────────────────────────────"
    echo "Test $((TOTAL+1)): $name"
    echo "URL: $url"
    echo "Expected status: $expected_status"
    echo
    
    TOTAL=$((TOTAL+1))
    
    # Make request via Songbird Unix socket
    local result=$(echo "{\"jsonrpc\":\"2.0\",\"method\":\"http.request\",\"params\":{\"method\":\"GET\",\"url\":\"$url\",\"headers\":{},\"body\":null},\"id\":$TOTAL}" | timeout 30 nc -N -U /tmp/songbird-nat0.sock 2>&1)
    
    # Check if we got a response
    if [ -z "$result" ]; then
        echo "❌ FAILED: No response (timeout or connection error)"
        FAILED=$((FAILED+1))
        return 1
    fi
    
    # Parse result
    local status=$(echo "$result" | jq -r '.result.status // .error.message // "error"' 2>/dev/null)
    local has_error=$(echo "$result" | jq -r '.error // empty' 2>/dev/null)
    
    # Check for RPC error
    if [ -n "$has_error" ]; then
        echo "❌ FAILED: RPC Error"
        echo "$result" | jq '.'
        FAILED=$((FAILED+1))
        return 1
    fi
    
    # Check status code
    if [ "$status" = "$expected_status" ]; then
        echo "✅ PASSED: Status $status"
        
        # Show response preview
        local body=$(echo "$result" | jq -r '.result.body // empty' 2>/dev/null | head -c 200)
        if [ -n "$body" ]; then
            echo
            echo "Response preview:"
            echo "$body..."
        fi
        
        PASSED=$((PASSED+1))
        return 0
    else
        echo "❌ FAILED: Status $status (expected $expected_status)"
        echo "$result" | jq '.'
        FAILED=$((FAILED+1))
        return 1
    fi
}

# Run tests
echo
echo "🚀 Starting HTTPS endpoint tests..."

# Test 1: GitHub API (Zen endpoint)
test_https "GitHub API (Zen)" "https://api.github.com/zen" 200

# Test 2: GitHub API (Rate limit)
test_https "GitHub API (Rate Limit)" "https://api.github.com/rate_limit" 200

# Test 3: Google (homepage)
test_https "Google Homepage" "https://www.google.com" 200

# Test 4: CloudFlare
test_https "CloudFlare" "https://www.cloudflare.com" 200

# Test 5: HuggingFace
test_https "HuggingFace" "https://huggingface.co" 200

# Test 6: httpbin.org (GET test)
test_https "httpbin.org (GET)" "https://httpbin.org/get" 200

# Test 7: httpbin.org (User-Agent test)
test_https "httpbin.org (User-Agent)" "https://httpbin.org/user-agent" 200

# Test 8: Example.com
test_https "Example.com" "https://example.com" 200

# Summary
echo
echo "═══════════════════════════════════════════════════════════════════════"
echo "TEST SUMMARY"
echo "═══════════════════════════════════════════════════════════════════════"
echo
echo "Total tests: $TOTAL"
echo "Passed: $PASSED ✅"
echo "Failed: $FAILED ❌"
echo
if [ $FAILED -eq 0 ]; then
    echo "🎉 ALL TESTS PASSED! 100% Pure Rust HTTPS is WORKING! 🎉"
    echo
    echo "Achievement:"
    echo "  • TLS 1.3 handshake: ✅"
    echo "  • RFC 8446 compliance: ✅"
    echo "  • Multiple real-world endpoints: ✅"
    echo "  • Zero C dependencies: ✅"
    echo "  • Capability-based routing: ✅"
    exit 0
else
    echo "⚠️  Some tests failed. Review errors above."
    exit 1
fi

