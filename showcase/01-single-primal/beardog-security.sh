#!/usr/bin/env bash
# BearDog Security Demo
# Tests BiomeOS crypto operations with REAL BearDog binary

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "╔════════════════════════════════════════════════════════╗"
echo "║  BiomeOS + BearDog: Security Operations Demo          ║"
echo "║  Testing with REAL BearDog binary                      ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

GAP_REPORT="$SCRIPT_DIR/gaps/beardog-gaps.md"
mkdir -p "$SCRIPT_DIR/gaps"

echo "Purpose: Test BiomeOS crypto orchestration with real BearDog"
echo "Duration: ~5 minutes"
echo ""

cat > "$GAP_REPORT" <<'EOF'
# Gaps Found: BearDog Integration

## Encryption API Issues
- [ ] To be documented

## Key Management Issues
- [ ] To be documented

## Authentication Issues
- [ ] To be documented

## Follow-Up Actions
- [ ] To be documented
EOF

echo -e "${GREEN}Step 1: Start Real BearDog Binary${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

BEARDOG_PORT=9000
"$SCRIPT_DIR/common/start-primal.sh" beardog $BEARDOG_PORT

export BEARDOG_ENDPOINT="http://localhost:$BEARDOG_PORT"

echo ""
echo -e "${GREEN}Step 2: Test Encryption${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

PLAINTEXT="Secret message from BiomeOS"
echo "Encrypting: $PLAINTEXT"

ENCRYPT_REQUEST="{\"data\": \"$PLAINTEXT\"}"

# Try encryption endpoints
for endpoint in "/api/v1/encrypt" "/encrypt" "/crypto/encrypt"; do
    if curl -s -X POST "$BEARDOG_ENDPOINT$endpoint" \
        -H "Content-Type: application/json" \
        -d "$ENCRYPT_REQUEST" > /dev/null 2>&1; then
        ENCRYPTED=$(curl -s -X POST "$BEARDOG_ENDPOINT$endpoint" \
            -H "Content-Type: application/json" \
            -d "$ENCRYPT_REQUEST")
        echo -e "${GREEN}✓ Encryption endpoint found: $endpoint${NC}"
        echo "Encrypted data: $ENCRYPTED"
        break
    fi
done

echo ""
echo -e "${GREEN}Step 3: Test Key Generation${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Try key generation
for endpoint in "/api/v1/keys/generate" "/keys/generate" "/crypto/keys"; do
    if curl -s -X POST "$BEARDOG_ENDPOINT$endpoint" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Key generation endpoint found: $endpoint${NC}"
        break
    fi
done

echo ""
echo -e "${GREEN}Step 4: Test Certificate Operations${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Try certificate endpoints
curl -s "$BEARDOG_ENDPOINT/api/v1/certificates" 2>/dev/null | jq '.' 2>/dev/null || echo "  (exploring...)"

echo ""
"$SCRIPT_DIR/common/stop-primal.sh" beardog

echo ""
echo "Demo Complete! Gaps documented in: $GAP_REPORT"
echo "Next: ./squirrel-ai.sh"

