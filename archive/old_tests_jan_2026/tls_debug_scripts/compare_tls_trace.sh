#!/bin/bash
# Compare our TLS implementation against OpenSSL's working implementation
# Date: January 23, 2026

echo "╔══════════════════════════════════════════════════════════════════════╗"
echo "║                                                                      ║"
echo "║      🔬 TLS 1.3 Trace Comparison - OpenSSL vs Songbird 🔬           ║"
echo "║                                                                      ║"
echo "╚══════════════════════════════════════════════════════════════════════╝"
echo ""

TARGET="api.github.com"
OUTPUT_DIR="/tmp/tls-comparison"
mkdir -p "$OUTPUT_DIR"

echo "🔍 Step 1: Capture OpenSSL TLS 1.3 session with keylog..."
echo ""

# Use OpenSSL to make a TLS 1.3 connection with keylog
SSLKEYLOGFILE="$OUTPUT_DIR/openssl_keylog.txt" \
openssl s_client -connect "$TARGET:443" \
  -tls1_3 \
  -ciphersuites "TLS_AES_128_GCM_SHA256" \
  -keylogfile "$OUTPUT_DIR/openssl_keylog.txt" \
  -msg \
  <<< "GET /zen HTTP/1.1
Host: $TARGET
Connection: close

" > "$OUTPUT_DIR/openssl_output.txt" 2>&1

echo "✅ OpenSSL session captured"
echo ""

echo "🔍 Step 2: Extract key material from keylog..."
echo ""

if [ -f "$OUTPUT_DIR/openssl_keylog.txt" ]; then
    echo "📋 Key Material (NSS Key Log Format):"
    cat "$OUTPUT_DIR/openssl_keylog.txt"
    echo ""
    
    # Extract handshake traffic secrets
    HANDSHAKE_SECRET=$(grep "CLIENT_HANDSHAKE_TRAFFIC_SECRET" "$OUTPUT_DIR/openssl_keylog.txt" | head -1 | awk '{print $3}')
    if [ -n "$HANDSHAKE_SECRET" ]; then
        echo "🔑 CLIENT_HANDSHAKE_TRAFFIC_SECRET: $HANDSHAKE_SECRET"
    fi
    
    # Extract application traffic secrets
    APP_SECRET=$(grep "CLIENT_TRAFFIC_SECRET_0" "$OUTPUT_DIR/openssl_keylog.txt" | head -1 | awk '{print $3}')
    if [ -n "$APP_SECRET" ]; then
        echo "🔑 CLIENT_TRAFFIC_SECRET_0: $APP_SECRET"
    fi
else
    echo "❌ No keylog file generated"
fi

echo ""
echo "🔍 Step 3: Check our Songbird's key material..."
echo ""

# Get latest handshake key from Songbird logs
OUR_KEY=$(grep "Key (hex):" /tmp/songbird-test-suite.log | tail -1 | awk '{print $NF}')
if [ -n "$OUR_KEY" ]; then
    echo "🔑 Our handshake key: $OUR_KEY"
    echo "   Length: $((${#OUR_KEY}/2)) bytes"
else
    echo "❌ Could not find our key in logs"
fi

echo ""
echo "════════════════════════════════════════════════════════════════════"
echo ""
echo "📊 ANALYSIS:"
echo ""
echo "If keys match: Our key derivation is correct ✅"
echo "If keys differ: Issue with ECDH or HKDF implementation ❌"
echo ""
echo "Next steps:"
echo "1. Compare keys manually"
echo "2. Use Wireshark with keylog to decrypt actual packets"
echo "3. Compare nonce/AAD construction"
echo ""
echo "📁 Files saved to: $OUTPUT_DIR"

