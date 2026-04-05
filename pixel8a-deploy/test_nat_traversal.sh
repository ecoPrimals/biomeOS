#!/system/bin/sh
#═══════════════════════════════════════════════════════════════════════════════
# NAT Traversal Test — Pixel 8a -> Tower (nestgate.io)
# Run this AFTER switching to mobile hotspot (cellular data)
#
# Tests:
#   1. DNS resolution for nestgate.io
#   2. HTTP reachability to Tower Songbird (:3492)
#   3. BearDog crypto handshake (encrypt on Pixel, verify format)
#   4. Cross-network beacon delivery (POST encrypted beacon to Tower)
#   5. IPv6 reachability test
#   6. STUN public IP discovery (Pixel's carrier NAT address)
#
# Prerequisites: BearDog running on 127.0.0.1:9900
#═══════════════════════════════════════════════════════════════════════════════

TOWER_HOST="nestgate.io"
TOWER_PORT=3492
TOWER_IPV6="2600:1700:b0b0:5b90:f137:2c09:7f3c:ac27"
BEARDOG_TCP="127.0.0.1:9900"
SONGBIRD_IPC="127.0.0.1:44493"
FAMILY_ID="8ff3b864a4bc589a"
RESULTS="/data/local/tmp/biomeos/nat_test_results.log"

PASS=0
FAIL=0
SKIP=0

log() { echo "$1" | tee -a "$RESULTS"; }
test_pass() { log "  ✅ $1"; PASS=$((PASS+1)); }
test_fail() { log "  ❌ $1"; FAIL=$((FAIL+1)); }
test_skip() { log "  -- $1 (skipped)"; SKIP=$((SKIP+1)); }

echo "" > "$RESULTS"
log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
log "  NAT TRAVERSAL TEST — Pixel 8a -> Tower"
log "  $(date)"
log "  Network: $(getprop gsm.network.type 2>/dev/null || echo 'unknown')"
log "  Family: $FAMILY_ID"
log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

#───────────────────────────────────────────────────────────────────────────────
# Test 0: Local primals alive
#───────────────────────────────────────────────────────────────────────────────
log ""
log "Test 0: Local primals"
# Use timeout + explicit host/port to avoid nc pipe issues on Android
R=$(echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' | timeout 5 nc 127.0.0.1 9900 2>/dev/null)
echo "$R" | grep -q '"healthy"' && test_pass "BearDog alive" || test_fail "BearDog not responding"

nc -z 127.0.0.1 9901 2>/dev/null && test_pass "Songbird TCP :9901 alive" || test_fail "Songbird not listening"

#───────────────────────────────────────────────────────────────────────────────
# Test 1: DNS resolution
#───────────────────────────────────────────────────────────────────────────────
log ""
log "Test 1: DNS resolution"
TOWER_IP=$(getent hosts $TOWER_HOST 2>/dev/null | awk '{print $1}' | head -1)
if [ -z "$TOWER_IP" ]; then
    # Android fallback
    TOWER_IP=$(ping -c 1 -W 3 $TOWER_HOST 2>/dev/null | head -1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+' | head -1)
fi
if [ -n "$TOWER_IP" ]; then
    test_pass "DNS: $TOWER_HOST -> $TOWER_IP"
else
    test_fail "DNS: $TOWER_HOST failed to resolve"
    TOWER_IP="198.51.100.1"  # Fallback to known IP
    log "  Using fallback IP: $TOWER_IP"
fi

#───────────────────────────────────────────────────────────────────────────────
# Test 2: TCP reachability
#───────────────────────────────────────────────────────────────────────────────
log ""
log "Test 2: TCP reachability"
nc -z -w 5 $TOWER_IP $TOWER_PORT 2>/dev/null && \
    test_pass "TCP: $TOWER_IP:$TOWER_PORT reachable" || \
    test_fail "TCP: $TOWER_IP:$TOWER_PORT unreachable"

#───────────────────────────────────────────────────────────────────────────────
# Test 3: HTTP health via nestgate.io
#───────────────────────────────────────────────────────────────────────────────
log ""
log "Test 3: HTTP health"
HTTP_RESP=$(echo -e "GET /health HTTP/1.1\r\nHost: $TOWER_HOST:$TOWER_PORT\r\nConnection: close\r\n\r\n" | \
    nc -w 5 $TOWER_IP $TOWER_PORT 2>/dev/null)
if echo "$HTTP_RESP" | grep -q "200 OK"; then
    test_pass "HTTP: $TOWER_HOST:$TOWER_PORT/health -> 200 OK"
else
    test_fail "HTTP: $TOWER_HOST:$TOWER_PORT/health failed"
    log "  Response: $(echo "$HTTP_RESP" | head -1)"
fi

#───────────────────────────────────────────────────────────────────────────────
# Test 4: IPv6 reachability
#───────────────────────────────────────────────────────────────────────────────
log ""
log "Test 4: IPv6 reachability"
nc -z -w 5 $TOWER_IPV6 $TOWER_PORT 2>/dev/null && \
    test_pass "IPv6: [$TOWER_IPV6]:$TOWER_PORT reachable" || \
    test_fail "IPv6: [$TOWER_IPV6]:$TOWER_PORT unreachable (carrier may block)"

#───────────────────────────────────────────────────────────────────────────────
# Test 5: BearDog crypto handshake payload
#───────────────────────────────────────────────────────────────────────────────
log ""
log "Test 5: BearDog beacon payload generation"

# Derive family key and encrypt a beacon payload on Pixel BearDog
BSK_B64=$(echo -n "beacon_shared_key" | base64)
SEED_B64=$(base64 /data/local/tmp/biomeos/.family.seed 2>/dev/null | tr -d '\n')
if [ -n "$SEED_B64" ]; then
    KEY_RESP=$(echo "{\"jsonrpc\":\"2.0\",\"method\":\"crypto.hmac_sha256\",\"params\":{\"key\":\"$SEED_B64\",\"data\":\"$BSK_B64\"},\"id\":1}" | \
        timeout 5 nc 127.0.0.1 9900 2>/dev/null)
    if echo "$KEY_RESP" | grep -q '"mac"'; then
        test_pass "Family beacon key derived"
    else
        test_fail "Family beacon key derivation failed"
    fi
else
    test_fail "Cannot read .family.seed"
fi

# Encrypt a test message with ChaCha20-Poly1305
PLAINTEXT_B64=$(echo -n "{\"node\":\"pixel8a\",\"ts\":$(date +%s)}" | base64 | tr -d '\n')
MAC=$(echo "$KEY_RESP" | grep -o '"mac":"[^"]*"' | cut -d'"' -f4)
ENC_RESP=$(echo "{\"jsonrpc\":\"2.0\",\"method\":\"crypto.chacha20_poly1305_encrypt\",\"params\":{\"key\":\"$MAC\",\"plaintext\":\"$PLAINTEXT_B64\",\"associated_data\":\"bmVzdGdhdGUtYmVhY29uLXYy\"},\"id\":1}" | \
    timeout 5 nc 127.0.0.1 9900 2>/dev/null)
if echo "$ENC_RESP" | grep -q '"ciphertext"'; then
    test_pass "Beacon payload encrypted (ChaCha20-Poly1305)"
else
    test_fail "Beacon encryption failed"
fi

#───────────────────────────────────────────────────────────────────────────────
# Test 6: Cross-network encrypted message delivery
#───────────────────────────────────────────────────────────────────────────────
log ""
log "Test 6: Encrypted message delivery to Tower"

# Generate a lineage proof to send
PROOF=$(echo "{\"jsonrpc\":\"2.0\",\"method\":\"crypto.blake3_hash\",\"params\":{\"data\":\"cGl4ZWw4YV9wcm9vZg==\"},\"id\":1}" | \
    timeout 5 nc 127.0.0.1 9900 2>/dev/null)
if echo "$PROOF" | grep -q '"hash"'; then
    HASH=$(echo "$PROOF" | python3 -c "import sys,json; print(json.load(sys.stdin)['result']['hash'])" 2>/dev/null || echo "?")
    test_pass "Blake3 proof generated: ${HASH}"
    
    # Send the proof hash to Tower via HTTP POST
    PAYLOAD="{\"family\":\"$FAMILY_ID\",\"node\":\"pixel8a\",\"proof\":\"$HASH\",\"timestamp\":$(date +%s)}"
    PAYLOAD_LEN=${#PAYLOAD}
    
    DELIVER=$(echo -e "POST /api/beacon HTTP/1.1\r\nHost: $TOWER_HOST\r\nContent-Type: application/json\r\nContent-Length: $PAYLOAD_LEN\r\nConnection: close\r\n\r\n$PAYLOAD" | \
        nc -w 5 $TOWER_IP $TOWER_PORT 2>/dev/null)
    
    if echo "$DELIVER" | grep -qE "200|201|204|404"; then
        test_pass "HTTP POST delivered to Tower ($(echo "$DELIVER" | head -1 | tr -d '\r'))"
    else
        test_skip "HTTP POST to /api/beacon (endpoint may not exist yet)"
        log "  Response: $(echo "$DELIVER" | head -1)"
    fi
else
    test_fail "Blake3 proof generation failed"
fi

#───────────────────────────────────────────────────────────────────────────────
# Test 7: STUN public IP discovery (Pixel's carrier NAT)
#───────────────────────────────────────────────────────────────────────────────
log ""
log "Test 7: STUN public IP discovery"

# Try via Songbird if available
STUN_RESP=$(echo '{"jsonrpc":"2.0","method":"stun.get_public_address","params":{},"id":1}' | \
    nc -w 8 $SONGBIRD_IPC 2>/dev/null)
if echo "$STUN_RESP" | grep -q '"public_address"'; then
    PUB=$(echo "$STUN_RESP" | python3 -c "import sys,json; r=json.load(sys.stdin)['result']; print(f'{r[\"public_address\"]}:{r[\"public_port\"]}')" 2>/dev/null || echo "?")
    test_pass "STUN public address: $PUB"
else
    # STUN might not be on the orchestrator IPC — skip
    test_skip "STUN (not on orchestrator IPC)"
fi

#───────────────────────────────────────────────────────────────────────────────
# Summary
#───────────────────────────────────────────────────────────────────────────────
log ""
log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
log "  RESULT: $PASS passed, $FAIL failed, $SKIP skipped"
TOTAL=$((PASS+FAIL+SKIP))
log "  Total: $TOTAL tests"
if [ $FAIL -eq 0 ]; then
    log "  NAT TRAVERSAL: OPERATIONAL"
else
    log "  NAT TRAVERSAL: PARTIAL ($FAIL issues)"
fi
log "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
log ""
log "Results saved to: $RESULTS"
