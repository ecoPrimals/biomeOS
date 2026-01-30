#!/bin/bash
# biomeOS Tower Atomic for Pixel 8a
# Run in Termux proot-distro (Debian/Ubuntu)

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FAMILY_ID="${FAMILY_ID:-default}"

# Create XDG runtime directory
mkdir -p /tmp/biomeos

echo "🐻🐕 Starting BearDog..."
$SCRIPT_DIR/primals/beardog server \
    --socket /tmp/biomeos/beardog-$FAMILY_ID.sock \
    --family-id $FAMILY_ID &
sleep 2

echo "🧠 Starting Neural API..."
FAMILY_ID=$FAMILY_ID $SCRIPT_DIR/neural-api-server \
    --socket /tmp/biomeos/neural-api-$FAMILY_ID.sock \
    --graphs $SCRIPT_DIR/graphs &
sleep 2

echo "🐦 Starting Songbird..."
FAMILY_ID=$FAMILY_ID \
BEARDOG_MODE=neural \
BEARDOG_SOCKET=/tmp/biomeos/beardog-$FAMILY_ID.sock \
$SCRIPT_DIR/primals/songbird server \
    --socket /tmp/biomeos/songbird-$FAMILY_ID.sock \
    --port 8081 &

echo ""
echo "✅ Tower Atomic running on Pixel 8a!"
echo ""
echo "Sockets:"
ls -la /tmp/biomeos/*.sock 2>/dev/null || echo "(waiting for sockets...)"
