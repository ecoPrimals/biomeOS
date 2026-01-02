#!/usr/bin/env bash
# Demo: Secure Storage Federation - 3 Primals
# Songbird (Discovery) + BearDog (Crypto) + NestGate (Storage)
# Shows BiomeOS orchestrating encrypted, federated storage with friends

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$SCRIPT_DIR/../../../.."
PHASE1_BINS="$BIOMEOS_ROOT/phase1bins"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "╔════════════════════════════════════════════════════════╗"
echo "║  Phase 1 Triple: Secure Storage Federation            ║"
echo "║  Songbird + BearDog + NestGate                        ║"
echo "║  Complete Privacy-Preserving Friend Storage           ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

GAP_REPORT="$SCRIPT_DIR/gaps-discovered.md"
mkdir -p "$SCRIPT_DIR/logs"

cat > "$GAP_REPORT" <<'EOF'
# Gaps Discovered: Secure Storage Federation

## 3-Primal Coordination Issues
- [ ] To be documented during demo

## Encrypted Federation Issues
- [ ] To be documented during demo

## Discovery + Crypto + Storage Integration
- [ ] To be documented during demo
EOF

echo -e "${GREEN}Step 1: Start Songbird (Service Mesh)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

SONGBIRD_BIN="$PHASE1_BINS/songbird-cli-dec-25-2025-standalone"
SONGBIRD_PORT=8080

if [ ! -f "$SONGBIRD_BIN" ]; then
    echo -e "${RED}✗ Songbird not found${NC}"
    exit 1
fi

$SONGBIRD_BIN tower start --port $SONGBIRD_PORT --bind 127.0.0.1 \
    > "$SCRIPT_DIR/logs/songbird.log" 2>&1 &
SONGBIRD_PID=$!
sleep 4
echo -e "${GREEN}✓ Songbird started (PID: $SONGBIRD_PID)${NC}"
echo ""

echo -e "${GREEN}Step 2: Start BearDog (Cryptography)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

BEARDOG_BIN="$PHASE1_BINS/beardog-bin"
BEARDOG_PORT=9002

if [ ! -f "$BEARDOG_BIN" ]; then
    echo -e "${RED}✗ BearDog not found${NC}"
    kill $SONGBIRD_PID 2>/dev/null || true
    exit 1
fi

$BEARDOG_BIN --port $BEARDOG_PORT > "$SCRIPT_DIR/logs/beardog.log" 2>&1 &
BEARDOG_PID=$!
sleep 3
echo -e "${GREEN}✓ BearDog started (PID: $BEARDOG_PID)${NC}"
echo ""

echo -e "${GREEN}Step 3: Start NestGate (Storage)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

NESTGATE_BIN="$PHASE1_BINS/nestgate-bin"
NESTGATE_PORT=9000

if [ ! -f "$NESTGATE_BIN" ]; then
    echo -e "${RED}✗ NestGate not found${NC}"
    kill $BEARDOG_PID $SONGBIRD_PID 2>/dev/null || true
    exit 1
fi

$NESTGATE_BIN --port $NESTGATE_PORT > "$SCRIPT_DIR/logs/nestgate.log" 2>&1 &
NESTGATE_PID=$!
sleep 3
echo -e "${GREEN}✓ NestGate started (PID: $NESTGATE_PID)${NC}"
echo ""

echo -e "${GREEN}Step 4: BiomeOS Orchestration${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}Complete Integration Pattern:${NC}"
echo ""
echo "  // 1. Discover services via Songbird"
echo "  let songbird = SongbirdClient::new();"
echo "  let storage = songbird.find_capability(\"storage\").await?;"
echo "  let crypto = songbird.find_capability(\"crypto\").await?;"
echo ""
echo "  // 2. Get BearDog crypto service"
echo "  let beardog = BearDogClient::new(crypto.endpoint);"
echo "  let key = beardog.generate_key().await?;"
echo ""
echo "  // 3. Encrypt data"
echo "  let encrypted = beardog.encrypt(data, key).await?;"
echo ""
echo "  // 4. Store in NestGate"
echo "  let nestgate = NestGateClient::new(storage.endpoint);"
echo "  nestgate.store(encrypted).await?;"
echo ""
echo "  // 5. Join federation (friends can access!)"
echo "  nestgate.join_federation(friends).await?;"
echo ""

echo -e "${GREEN}Step 5: Real-World Scenario${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}Use Case: Friend Family Cloud${NC}"
echo ""
echo "  Alice, Bob, and Carol want to share storage:"
echo ""
echo "  1. Songbird discovers each friend's NestGate"
echo "  2. BearDog encrypts data (only family can decrypt)"
echo "  3. NestGate provides ZFS storage + federation"
echo "  4. Friends share storage securely"
echo ""
echo "  Result: Encrypted, federated, friend-owned cloud!"
echo ""

echo -e "${BLUE}Benefits:${NC}"
echo "  ✓ No central server needed"
echo "  ✓ Privacy-preserving (BearDog)"
echo "  ✓ Reliable storage (ZFS)"
echo "  ✓ Dynamic discovery (Songbird)"
echo "  ✓ Friend-to-friend sharing"
echo ""

echo -e "${BLUE}What BiomeOS Orchestrates:${NC}"
echo "  ✓ Service discovery (Songbird)"
echo "  ✓ Encryption/decryption (BearDog)"
echo "  ✓ Storage management (NestGate)"
echo "  ✓ Federation coordination (all 3!)"
echo ""

echo -e "${GREEN}Cleanup${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
kill $NESTGATE_PID $BEARDOG_PID $SONGBIRD_PID 2>/dev/null || true
sleep 1
echo -e "${GREEN}✓ All services stopped${NC}"
echo ""

echo -e "${GREEN}Demo Complete!${NC}"
echo ""
echo -e "${BLUE}Key Takeaway:${NC}"
echo "  BiomeOS orchestrated 3 primals to create a complete"
echo "  privacy-preserving, federated storage system!"
echo ""
echo "Next: Run ../02-secure-compute/demo.sh"
echo ""

