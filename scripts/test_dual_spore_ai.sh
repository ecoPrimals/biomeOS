#!/bin/bash
# Test Dual Spore AI Coordination
# Validates that both spores can work together

SPORE1_SOCKET="/run/user/$(id -u)/biomeos/spore1-neural-api.sock"
SPORE2_SOCKET="/run/user/$(id -u)/biomeos/spore2-neural-api.sock"

echo "═══════════════════════════════════════════════════════════"
echo "     Dual Spore AI Coordination Test                        "
echo "═══════════════════════════════════════════════════════════"
echo ""

# Test 1: Local compute via Spore 1
echo "Test 1: Local compute health (Spore 1 → Toadstool)"
echo '{"jsonrpc":"2.0","method":"capability.call","params":{"capability":"compute","operation":"health"},"id":1}' | \
    timeout 5 nc -U "$SPORE1_SOCKET" 2>/dev/null && echo "✅" || echo "❌"

# Test 2: External API via Spore 2
echo ""
echo "Test 2: External API health (Spore 2 → Squirrel)"
echo '{"jsonrpc":"2.0","method":"capability.call","params":{"capability":"ai","operation":"health"},"id":2}' | \
    timeout 5 nc -U "$SPORE2_SOCKET" 2>/dev/null && echo "✅" || echo "❌"

# Test 3: Cross-spore coordination
echo ""
echo "Test 3: Cross-spore AI query"
echo "  Sending text to Spore 2 (API) for analysis..."
RESULT=$(echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"What is 2+2?","model":"claude-3-haiku-20240307","max_tokens":50},"id":3}' | \
    timeout 30 nc -U "$SPORE2_SOCKET" 2>/dev/null)
if echo "$RESULT" | grep -q "result"; then
    echo "  ✅ API response received"
    echo "$RESULT" | jq -r '.result.response' 2>/dev/null | head -c 100
else
    echo "  ❌ No response"
fi

# Test 4: Federation trust
echo ""
echo "Test 4: Federation trust verification"
SPORE1_FAMILY=$(cat /media/$USER/*/biomeOS/.family.seed 2>/dev/null | head -1 | head -c 8 | xxd -p)
echo '{"jsonrpc":"2.0","method":"federation.verify_family_member","params":{"family_id":"'$SPORE1_FAMILY'"},"id":4}' | \
    timeout 5 nc -U "$SPORE2_SOCKET" 2>/dev/null && echo "✅ Federation verified" || echo "❌ Federation check failed"

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "     Test Complete                                          "
echo "═══════════════════════════════════════════════════════════"
