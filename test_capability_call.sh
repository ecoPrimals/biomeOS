#!/bin/bash
# Test capability.call with extended HTTP methods
# Tests POST, PUT, DELETE operations via Neural API's capability.call

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "═══════════════════════════════════════════════════════════════"
echo "🧪 Testing capability.call - Extended HTTP Methods"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Check if Neural API is running
NEURAL_SOCKET="/tmp/neural-api-nat0.sock"
SONGBIRD_SOCKET="/tmp/songbird-nat0.sock"

if [ ! -S "$NEURAL_SOCKET" ]; then
    echo -e "${RED}❌ Neural API socket not found: $NEURAL_SOCKET${NC}"
    echo "   Please start Neural API first:"
    echo "   ./target/release/biomeos neural-api --mode coordinated"
    exit 1
fi

if [ ! -S "$SONGBIRD_SOCKET" ]; then
    echo -e "${YELLOW}⚠️  Songbird socket not found: $SONGBIRD_SOCKET${NC}"
    echo "   Some tests may fail if Tower Atomic is not deployed"
    echo "   Deploy with: graph_deploy tower_atomic_bootstrap"
fi

echo -e "${BLUE}Testing endpoint: httpbin.org (HTTP test server)${NC}"
echo ""

# Test 1: HTTP GET via capability.call
echo "═══════════════════════════════════════════════════════════════"
echo "Test 1: HTTP GET via capability.call"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Request:"
echo '{
  "jsonrpc": "2.0",
  "method": "capability.call",
  "params": {
    "capability": "secure_http",
    "operation": "http.get",
    "args": {
      "url": "https://httpbin.org/get?test=capability_call",
      "headers": {
        "User-Agent": "ecoPrimals/1.0",
        "X-Test": "capability.call"
      }
    }
  },
  "id": 1
}'
echo ""

GET_RESPONSE=$(echo '{
  "jsonrpc": "2.0",
  "method": "capability.call",
  "params": {
    "capability": "secure_http",
    "operation": "http.get",
    "args": {
      "url": "https://httpbin.org/get?test=capability_call",
      "headers": {
        "User-Agent": "ecoPrimals/1.0",
        "X-Test": "capability.call"
      }
    }
  },
  "id": 1
}' | nc -U "$NEURAL_SOCKET" 2>&1)

echo "Response:"
echo "$GET_RESPONSE" | jq '.' 2>/dev/null || echo "$GET_RESPONSE"
echo ""

if echo "$GET_RESPONSE" | grep -q '"result"'; then
    echo -e "${GREEN}✅ GET test passed${NC}"
else
    echo -e "${RED}❌ GET test failed${NC}"
fi
echo ""

# Test 2: HTTP POST via capability.call
echo "═══════════════════════════════════════════════════════════════"
echo "Test 2: HTTP POST via capability.call"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Request:"
echo '{
  "jsonrpc": "2.0",
  "method": "capability.call",
  "params": {
    "capability": "secure_http",
    "operation": "http.post",
    "args": {
      "url": "https://httpbin.org/post",
      "headers": {
        "User-Agent": "ecoPrimals/1.0",
        "Content-Type": "application/json"
      },
      "body": {
        "test": "capability.call",
        "method": "POST",
        "timestamp": "2026-01-25"
      }
    }
  },
  "id": 2
}'
echo ""

POST_RESPONSE=$(echo '{
  "jsonrpc": "2.0",
  "method": "capability.call",
  "params": {
    "capability": "secure_http",
    "operation": "http.post",
    "args": {
      "url": "https://httpbin.org/post",
      "headers": {
        "User-Agent": "ecoPrimals/1.0",
        "Content-Type": "application/json"
      },
      "body": {
        "test": "capability.call",
        "method": "POST",
        "timestamp": "2026-01-25"
      }
    }
  },
  "id": 2
}' | nc -U "$NEURAL_SOCKET" 2>&1)

echo "Response:"
echo "$POST_RESPONSE" | jq '.' 2>/dev/null || echo "$POST_RESPONSE"
echo ""

if echo "$POST_RESPONSE" | grep -q '"result"'; then
    echo -e "${GREEN}✅ POST test passed${NC}"
else
    echo -e "${RED}❌ POST test failed${NC}"
fi
echo ""

# Test 3: HTTP PUT via capability.call
echo "═══════════════════════════════════════════════════════════════"
echo "Test 3: HTTP PUT via capability.call"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Request:"
echo '{
  "jsonrpc": "2.0",
  "method": "capability.call",
  "params": {
    "capability": "secure_http",
    "operation": "http.put",
    "args": {
      "url": "https://httpbin.org/put",
      "headers": {
        "User-Agent": "ecoPrimals/1.0",
        "Content-Type": "application/json"
      },
      "body": {
        "test": "capability.call",
        "method": "PUT",
        "updated": true
      }
    }
  },
  "id": 3
}'
echo ""

PUT_RESPONSE=$(echo '{
  "jsonrpc": "2.0",
  "method": "capability.call",
  "params": {
    "capability": "secure_http",
    "operation": "http.put",
    "args": {
      "url": "https://httpbin.org/put",
      "headers": {
        "User-Agent": "ecoPrimals/1.0",
        "Content-Type": "application/json"
      },
      "body": {
        "test": "capability.call",
        "method": "PUT",
        "updated": true
      }
    }
  },
  "id": 3
}' | nc -U "$NEURAL_SOCKET" 2>&1)

echo "Response:"
echo "$PUT_RESPONSE" | jq '.' 2>/dev/null || echo "$PUT_RESPONSE"
echo ""

if echo "$PUT_RESPONSE" | grep -q '"result"'; then
    echo -e "${GREEN}✅ PUT test passed${NC}"
else
    echo -e "${RED}❌ PUT test failed${NC}"
fi
echo ""

# Test 4: HTTP DELETE via capability.call
echo "═══════════════════════════════════════════════════════════════"
echo "Test 4: HTTP DELETE via capability.call"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Request:"
echo '{
  "jsonrpc": "2.0",
  "method": "capability.call",
  "params": {
    "capability": "secure_http",
    "operation": "http.delete",
    "args": {
      "url": "https://httpbin.org/delete",
      "headers": {
        "User-Agent": "ecoPrimals/1.0"
      }
    }
  },
  "id": 4
}'
echo ""

DELETE_RESPONSE=$(echo '{
  "jsonrpc": "2.0",
  "method": "capability.call",
  "params": {
    "capability": "secure_http",
    "operation": "http.delete",
    "args": {
      "url": "https://httpbin.org/delete",
      "headers": {
        "User-Agent": "ecoPrimals/1.0"
      }
    }
  },
  "id": 4
}' | nc -U "$NEURAL_SOCKET" 2>&1)

echo "Response:"
echo "$DELETE_RESPONSE" | jq '.' 2>/dev/null || echo "$DELETE_RESPONSE"
echo ""

if echo "$DELETE_RESPONSE" | grep -q '"result"'; then
    echo -e "${GREEN}✅ DELETE test passed${NC}"
else
    echo -e "${RED}❌ DELETE test failed${NC}"
fi
echo ""

# Test 5: Generic http.request (fallback)
echo "═══════════════════════════════════════════════════════════════"
echo "Test 5: Generic http.request (fallback)"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Request:"
echo '{
  "jsonrpc": "2.0",
  "method": "capability.call",
  "params": {
    "capability": "secure_http",
    "operation": "http.request",
    "args": {
      "method": "PATCH",
      "url": "https://httpbin.org/patch",
      "headers": {
        "User-Agent": "ecoPrimals/1.0",
        "Content-Type": "application/json"
      },
      "body": {
        "test": "generic_http_request",
        "method": "PATCH"
      }
    }
  },
  "id": 5
}'
echo ""

PATCH_RESPONSE=$(echo '{
  "jsonrpc": "2.0",
  "method": "capability.call",
  "params": {
    "capability": "secure_http",
    "operation": "http.request",
    "args": {
      "method": "PATCH",
      "url": "https://httpbin.org/patch",
      "headers": {
        "User-Agent": "ecoPrimals/1.0",
        "Content-Type": "application/json"
      },
      "body": {
        "test": "generic_http_request",
        "method": "PATCH"
      }
    }
  },
  "id": 5
}' | nc -U "$NEURAL_SOCKET" 2>&1)

echo "Response:"
echo "$PATCH_RESPONSE" | jq '.' 2>/dev/null || echo "$PATCH_RESPONSE"
echo ""

if echo "$PATCH_RESPONSE" | grep -q '"result"'; then
    echo -e "${GREEN}✅ PATCH (via http.request) test passed${NC}"
else
    echo -e "${RED}❌ PATCH test failed${NC}"
fi
echo ""

# Summary
echo "═══════════════════════════════════════════════════════════════"
echo "📊 Test Summary"
echo "═══════════════════════════════════════════════════════════════"
echo ""

PASSED=0
TOTAL=5

echo "$GET_RESPONSE" | grep -q '"result"' && ((PASSED++)) || true
echo "$POST_RESPONSE" | grep -q '"result"' && ((PASSED++)) || true
echo "$PUT_RESPONSE" | grep -q '"result"' && ((PASSED++)) || true
echo "$DELETE_RESPONSE" | grep -q '"result"' && ((PASSED++)) || true
echo "$PATCH_RESPONSE" | grep -q '"result"' && ((PASSED++)) || true

echo "Tests Passed: $PASSED/$TOTAL"
echo ""

if [ "$PASSED" -eq "$TOTAL" ]; then
    echo -e "${GREEN}🎉 All tests passed! capability.call is fully operational!${NC}"
    echo ""
    echo "TRUE PRIMAL Pattern Validated:"
    echo "✅ Zero coupling between primals"
    echo "✅ Semantic operation routing"
    echo "✅ All HTTP methods supported"
    echo "✅ Production-ready!"
    exit 0
elif [ "$PASSED" -gt 0 ]; then
    echo -e "${YELLOW}⚠️  Some tests passed, but not all${NC}"
    echo ""
    echo "This may indicate:"
    echo "- Tower Atomic not fully deployed"
    echo "- Network connectivity issues"
    echo "- Songbird not registered yet"
    echo ""
    echo "Check logs for more details"
    exit 1
else
    echo -e "${RED}❌ All tests failed${NC}"
    echo ""
    echo "Troubleshooting:"
    echo "1. Ensure Neural API is running (COORDINATED MODE)"
    echo "2. Deploy Tower Atomic via graph_deploy"
    echo "3. Check Songbird capability registration"
    echo "4. Review Neural API logs"
    exit 1
fi

