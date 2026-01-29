#!/bin/bash
# biomeOS LAN Handshake Test Script
# Tests connectivity between two towers over LAN

set -e

LOCAL_TOWER="${1:-192.168.1.144}"
REMOTE_TOWER="${2:-192.168.1.134}"
FAMILY_ID="${3:-nat0}"

echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║              biomeOS LAN Handshake Test                          ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo ""
echo "Local Tower:  ${LOCAL_TOWER}"
echo "Remote Tower: ${REMOTE_TOWER}"
echo "Family ID:    ${FAMILY_ID}"
echo ""

# Test 1: Network connectivity
echo "=== Test 1: Network Ping ==="
if ping -c 1 -W 2 "${REMOTE_TOWER}" > /dev/null 2>&1; then
    echo "   ✅ Network reachable"
else
    echo "   ❌ Network unreachable"
    exit 1
fi

# Test 2: Songbird discovery port
echo ""
echo "=== Test 2: Songbird Discovery Port (8080) ==="
if nc -z -w 2 "${REMOTE_TOWER}" 8080 2>/dev/null; then
    echo "   ✅ Port 8080 open"
else
    echo "   ⚠️  Port 8080 closed (Songbird may not be running)"
fi

# Test 3: UDP Discovery beacon
echo ""
echo "=== Test 3: UDP Discovery Beacons ==="
echo "   Listening for beacons on port 23000 for 5 seconds..."
timeout 5 python3 -c "
import socket
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
s.bind(('0.0.0.0', 23000))
s.settimeout(5)
try:
    data, addr = s.recvfrom(4096)
    print(f'   ✅ Received beacon from {addr[0]}:{addr[1]} ({len(data)} bytes)')
except socket.timeout:
    print('   ⚠️  No beacons received')
" 2>/dev/null || echo "   ⚠️  UDP test failed"

# Test 4: HTTP health check (if Songbird running with HTTP gateway)
echo ""
echo "=== Test 4: HTTP Health Check ==="
HEALTH=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 2 "http://${REMOTE_TOWER}:8080/health" 2>/dev/null || echo "000")
if [ "$HEALTH" = "200" ]; then
    echo "   ✅ HTTP health check passed"
elif [ "$HEALTH" = "000" ]; then
    echo "   ⚠️  HTTP not available (Songbird may be socket-only)"
else
    echo "   ⚠️  HTTP returned status ${HEALTH}"
fi

echo ""
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║              LAN Test Complete                                   ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
