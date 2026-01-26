#!/bin/bash

echo "=== Songbird TLS Diagnostics ==="

# Clean up any previous runs
pkill -9 neural-api-server 2>/dev/null
pkill -9 beardog 2>/dev/null
pkill -9 songbird 2>/dev/null
sleep 2

# 1. Start Neural API
echo "[1/7] Starting Neural API..."
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
RUST_LOG=info ./target/release/neural-api-server \
  --graphs-dir graphs \
  --family-id nat0 \
  --socket /tmp/neural-api-nat0.sock > /tmp/neural-api-diag.log 2>&1 &
NEURAL_PID=$!
sleep 2

# 2. Deploy Tower Atomic (BearDog + Songbird)
echo "[2/7] Deploying Tower Atomic (BearDog + Songbird)..."
echo '{"jsonrpc":"2.0","method":"neural_api.execute_graph","params":{"graph_id":"tower_atomic_bootstrap"},"id":1}' | \
  timeout 30 nc -U /tmp/neural-api-nat0.sock > /dev/null &
DEPLOY_PID=$!
sleep 5

# Check if primals started
if [ ! -S /tmp/beardog-nat0.sock ] || [ ! -S /tmp/songbird-nat0.sock ]; then
    echo "❌ Tower Atomic failed to deploy"
    echo "   BearDog socket: $(test -S /tmp/beardog-nat0.sock && echo '✅' || echo '❌')"
    echo "   Songbird socket: $(test -S /tmp/songbird-nat0.sock && echo '✅' || echo '❌')"
    exit 1
fi

echo "   ✅ Tower Atomic deployed"

# 3. Test HTTPS
echo "[3/7] Testing HTTPS (20 second timeout)..."
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://api.github.com/zen"},"id":1}' | \
  timeout 20 nc -U /tmp/songbird-nat0.sock > /tmp/https-response.log 2>&1 &
TEST_PID=$!

# 4. Wait for test
echo "[4/7] Waiting for HTTPS test to complete or timeout..."
sleep 18

# 5. Kill test if still running
if kill -0 $TEST_PID 2>/dev/null; then
    echo "[5/7] Test still running after 18s, killing..."
    kill $TEST_PID 2>/dev/null
    TEST_RESULT="TIMEOUT"
else
    echo "[5/7] Test completed before timeout"
    wait $TEST_PID
    TEST_EXIT=$?
    if [ $TEST_EXIT -eq 0 ]; then
        TEST_RESULT="SUCCESS"
    else
        TEST_RESULT="FAILED"
    fi
fi

# 6. Stop primals
echo "[6/7] Stopping primals..."
pkill -9 songbird 2>/dev/null
pkill -9 beardog 2>/dev/null
pkill -9 neural-api-server 2>/dev/null
sleep 1

# 7. Analyze logs
echo "[7/7] Analyzing logs..."
echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "                      KEY LOG MESSAGES                            "
echo "═══════════════════════════════════════════════════════════════════"

# Check for Songbird logs (might be in multiple places)
SONGBIRD_LOG=""
if [ -f /tmp/songbird-tls-diag.log ]; then
    SONGBIRD_LOG="/tmp/songbird-tls-diag.log"
elif journalctl -u songbird --since "1 minute ago" --no-pager | grep -q "TLS"; then
    SONGBIRD_LOG="journalctl"
else
    # Try to find logs in neural-api output
    SONGBIRD_LOG="/tmp/neural-api-diag.log"
fi

if [ "$SONGBIRD_LOG" = "journalctl" ]; then
    echo "📋 Songbird logs (from journalctl):"
    journalctl -u songbird --since "1 minute ago" --no-pager | \
      grep -E "(Starting TLS|keypair|ClientHello|ServerHello|ECDH|session|TIMEOUT|ERROR)" | tail -30
elif [ -f "$SONGBIRD_LOG" ]; then
    echo "📋 Songbird logs (from $SONGBIRD_LOG):"
    grep -E "(Starting TLS|keypair|ClientHello|ServerHello|ECDH|session|TIMEOUT|ERROR)" "$SONGBIRD_LOG" 2>/dev/null | tail -30 || echo "(No TLS-related logs found)"
else
    echo "⚠️  No Songbird logs found"
fi

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "                          DIAGNOSIS                                "
echo "═══════════════════════════════════════════════════════════════════"

# Check each step
DIAGNOSIS=""

if grep -q "Starting TLS" "$SONGBIRD_LOG" 2>/dev/null; then
    echo "✅ TLS handshake initiated"
    DIAGNOSIS="$DIAGNOSIS\n✅ TLS handshake initiated"
else
    echo "❌ TLS handshake NOT initiated (Songbird didn't detect HTTPS?)"
    DIAGNOSIS="$DIAGNOSIS\n❌ TLS handshake NOT initiated"
fi

if grep -q "Generated client keypair" "$SONGBIRD_LOG" 2>/dev/null || \
   grep -q "Generated client keypair" /tmp/neural-api-diag.log 2>/dev/null; then
    echo "✅ Keypair generation: Working"
    DIAGNOSIS="$DIAGNOSIS\n✅ Keypair generation: Working"
else
    echo "❌ Keypair generation: FAILED or HANGING"
    echo "   → Test BearDog directly:"
    echo "   echo '{\"jsonrpc\":\"2.0\",\"method\":\"crypto.x25519_generate_ephemeral\",\"params\":{},\"id\":1}' | nc -U /tmp/beardog-nat0.sock"
    DIAGNOSIS="$DIAGNOSIS\n❌ Keypair generation: FAILED or HANGING"
fi

if grep -q "Sending ClientHello" "$SONGBIRD_LOG" 2>/dev/null || \
   grep -q "Sending ClientHello" /tmp/neural-api-diag.log 2>/dev/null; then
    echo "✅ ClientHello construction: Working"
    DIAGNOSIS="$DIAGNOSIS\n✅ ClientHello construction: Working"
else
    echo "❌ ClientHello construction: FAILED"
    DIAGNOSIS="$DIAGNOSIS\n❌ ClientHello construction: FAILED"
fi

if grep -q "Received ServerHello" "$SONGBIRD_LOG" 2>/dev/null || \
   grep -q "Received ServerHello" /tmp/neural-api-diag.log 2>/dev/null; then
    echo "✅ ServerHello reception: Working"
    DIAGNOSIS="$DIAGNOSIS\n✅ ServerHello reception: Working"
else
    echo "❌ ServerHello reception: FAILED or TIMEOUT"
    echo "   → Run packet capture:"
    echo "   sudo tcpdump -i any -w /tmp/tls.pcap host api.github.com and port 443"
    DIAGNOSIS="$DIAGNOSIS\n❌ ServerHello reception: FAILED or TIMEOUT"
fi

if grep -q "Computed shared secret" "$SONGBIRD_LOG" 2>/dev/null || \
   grep -q "shared secret" /tmp/neural-api-diag.log 2>/dev/null; then
    echo "✅ ECDH: Working"
    DIAGNOSIS="$DIAGNOSIS\n✅ ECDH: Working"
else
    echo "❌ ECDH: FAILED or HANGING"
    echo "   → Test BearDog ECDH directly"
    DIAGNOSIS="$DIAGNOSIS\n❌ ECDH: FAILED or HANGING"
fi

if grep -q "TLS session keys derived" "$SONGBIRD_LOG" 2>/dev/null || \
   grep -q "session keys" /tmp/neural-api-diag.log 2>/dev/null; then
    echo "✅ Key derivation: Working"
    DIAGNOSIS="$DIAGNOSIS\n✅ Key derivation: Working"
else
    echo "❌ Key derivation: FAILED or HANGING"
    echo "   → Test BearDog tls.derive_secrets directly"
    DIAGNOSIS="$DIAGNOSIS\n❌ Key derivation: FAILED or HANGING"
fi

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "                         TEST RESULT                               "
echo "═══════════════════════════════════════════════════════════════════"

echo "Test Result: $TEST_RESULT"

if [ -s /tmp/https-response.log ]; then
    echo ""
    echo "Response received (first 200 chars):"
    head -c 200 /tmp/https-response.log
    echo ""
fi

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "                         FULL LOGS                                 "
echo "═══════════════════════════════════════════════════════════════════"
echo "Neural API: /tmp/neural-api-diag.log"
echo "HTTPS Response: /tmp/https-response.log"
if [ "$SONGBIRD_LOG" != "journalctl" ]; then
    echo "Songbird: $SONGBIRD_LOG"
fi
echo ""
echo "To view full Songbird logs:"
echo "  grep -E '(TLS|handshake|keypair|ClientHello|ServerHello)' /tmp/neural-api-diag.log | less"
echo ""
echo "═══════════════════════════════════════════════════════════════════"

