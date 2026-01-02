#!/usr/bin/env bash
# Demo: BearDog + NestGate - Encrypted Storage
# Based on: nestgate/showcase/03_encryption_storage/
# Shows BiomeOS enabling encrypted storage with BearDog crypto

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$SCRIPT_DIR/../../../.."
PHASE1_BINS="$BIOMEOS_ROOT/primalBins"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo "╔════════════════════════════════════════════════════════╗"
echo "║  Phase 1 Core: BearDog + NestGate                     ║"
echo "║  Encrypted Storage with Genetic Cryptography          ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

echo -e "${BLUE}Based on: nestgate/showcase/03_encryption_storage/${NC}"
echo ""

GAP_REPORT="$SCRIPT_DIR/gaps-discovered.md"
mkdir -p "$SCRIPT_DIR/logs"

cat > "$GAP_REPORT" <<'EOF'
# Gaps Discovered: BearDog + NestGate Integration

## Encryption Integration Issues
- [ ] To be documented during demo

## Key Management Issues
- [ ] To be documented during demo

## Storage API Issues
- [ ] To be documented during demo
EOF

echo -e "${GREEN}Step 1: Start BearDog (Crypto Service)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

BEARDOG_BIN="$PHASE1_BINS/beardog-bin"
BEARDOG_PORT=9002

if [ ! -f "$BEARDOG_BIN" ]; then
    echo -e "${RED}✗ BearDog binary not found${NC}"
    exit 1
fi

echo "Starting BearDog crypto service..."
$BEARDOG_BIN --port $BEARDOG_PORT > "$SCRIPT_DIR/logs/beardog.log" 2>&1 &
BEARDOG_PID=$!
sleep 3

if ! kill -0 $BEARDOG_PID 2>/dev/null; then
    echo -e "${RED}✗ BearDog failed to start${NC}"
    exit 1
fi

echo -e "${GREEN}✓ BearDog started (PID: $BEARDOG_PID)${NC}"
echo ""

echo -e "${GREEN}Step 2: Start NestGate (Storage Service)${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

NESTGATE_BIN="$PHASE1_BINS/nestgate-bin"
NESTGATE_PORT=9000

if [ ! -f "$NESTGATE_BIN" ]; then
    echo -e "${RED}✗ NestGate binary not found${NC}"
    kill $BEARDOG_PID 2>/dev/null || true
    exit 1
fi

echo "Starting NestGate storage..."
$NESTGATE_BIN --port $NESTGATE_PORT > "$SCRIPT_DIR/logs/nestgate.log" 2>&1 &
NESTGATE_PID=$!
sleep 3

if ! kill -0 $NESTGATE_PID 2>/dev/null; then
    echo -e "${RED}✗ NestGate failed to start${NC}"
    kill $BEARDOG_PID 2>/dev/null || true
    exit 1
fi

echo -e "${GREEN}✓ NestGate started (PID: $NESTGATE_PID)${NC}"
echo ""

echo -e "${GREEN}Step 3: Encrypted Storage Pattern${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${BLUE}Integration Pattern:${NC}"
echo ""
echo "  // Get crypto service"
echo "  let beardog = BearDogClient::new(endpoint);"
echo "  let key = beardog.generate_key().await?;"
echo ""
echo "  // Encrypt data"
echo "  let encrypted = beardog.encrypt(data, key).await?;"
echo ""
echo "  // Store encrypted data"
echo "  let nestgate = NestGateClient::new(endpoint);"
echo "  nestgate.store(encrypted).await?;"
echo ""
echo "  // Later: Retrieve and decrypt"
echo "  let encrypted = nestgate.retrieve(id).await?;"
echo "  let data = beardog.decrypt(encrypted, key).await?;"
echo ""

echo -e "${BLUE}Benefits:${NC}"
echo "  ✓ Data encrypted at rest"
echo "  ✓ Genetic cryptography (lineage-based)"
echo "  ✓ Key management via BearDog"
echo "  ✓ ZFS reliability + encryption"
echo ""

echo -e "${BLUE}Use Cases:${NC}"
echo "  • Encrypted friend storage"
echo "  • Privacy-preserving backups"
echo "  • Secure data federation"
echo "  • Family cloud with encryption"
echo ""

echo -e "${GREEN}Cleanup${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
kill $NESTGATE_PID 2>/dev/null || true
kill $BEARDOG_PID 2>/dev/null || true
sleep 1
echo -e "${GREEN}✓ Services stopped${NC}"
echo ""

echo -e "${GREEN}Demo Complete!${NC}"
echo "Next: Run ../06-beardog-toadstool/demo.sh (Secure Compute)"
echo ""

