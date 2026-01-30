#!/system/bin/sh
# 🎊 NUCLEUS Mobile Deployment Script for Pixel 8a
# =============================================================================
# 
# Deploys Tower Atomic (BearDog + Songbird) on GrapheneOS
# Architecture: ARM64 (aarch64)
# Device: Google Pixel 8a
# OS: GrapheneOS (Android 16)
#
# Date: January 30, 2026
# Status: Production deployment validation
#
# =============================================================================

set -e

echo "🦀✨ NUCLEUS Mobile Deployment - Pixel 8a ✨🦀"
echo "═══════════════════════════════════════════════════"
echo ""

# Environment setup
export BIOMEOS_ROOT="/data/local/tmp/biomeos"
export PRIMAL_DIR="$BIOMEOS_ROOT/primals"
export FAMILY_SEED="$BIOMEOS_ROOT/.family.seed"

# Android doesn't have XDG_RUNTIME_DIR, use /data/local/tmp
export XDG_RUNTIME_DIR="/data/local/tmp"
mkdir -p "$XDG_RUNTIME_DIR/biomeos"

# Read family ID from seed (first 16 bytes as hex)
if [ -f "$FAMILY_SEED" ]; then
  FAMILY_ID=$(xxd -p -l 16 "$FAMILY_SEED" | tr -d '\n')
  echo "📍 Family ID from .family.seed: $FAMILY_ID"
else
  FAMILY_ID="pixel8a-mobile"
  echo "⚠️  Using default family ID: $FAMILY_ID"
fi

export FAMILY_ID
export NODE_ID="pixel8a-node1"
export RUST_LOG=info

echo ""
echo "Environment:"
echo "  Device: $(getprop ro.product.model)"
echo "  Android: $(getprop ro.build.version.release)"
echo "  Architecture: $(uname -m)"
echo "  Family ID: $FAMILY_ID"
echo "  Runtime Dir: $XDG_RUNTIME_DIR/biomeos"
echo ""

# Clean old sockets
rm -f "$XDG_RUNTIME_DIR/biomeos/"*.sock 2>/dev/null || true

echo "═══════════════════════════════════════════════════"
echo "🏰 Starting Tower Atomic (BearDog + Songbird)"
echo "═══════════════════════════════════════════════════"
echo ""

# Start BearDog (Security Foundation)
echo "🐻 Starting BearDog (Security & Genetics)..."
cd "$BIOMEOS_ROOT"  # Change to writable directory
export BEARDOG_SOCKET="$XDG_RUNTIME_DIR/biomeos/beardog.sock"
"$PRIMAL_DIR/beardog" server --socket "$BEARDOG_SOCKET" > "$BIOMEOS_ROOT/beardog.log" 2>&1 &
BEARDOG_PID=$!
echo "  PID: $BEARDOG_PID"
echo "  Socket: $BEARDOG_SOCKET"

# Wait for BearDog socket
sleep 8
BEARDOG_SOCK="$BEARDOG_SOCKET"

if [ -S "$BEARDOG_SOCK" ]; then
  echo "  ✅ BearDog ready!"
else
  echo "  ❌ BearDog socket not found at: $BEARDOG_SOCK"
  echo "  Checking directory..."
  ls -la "$XDG_RUNTIME_DIR/biomeos/" || echo "Directory not accessible"
  echo "  Log tail:"
  tail -15 "$BIOMEOS_ROOT/beardog.log"
  exit 1
fi

echo ""
echo "🎵 Starting Songbird (Discovery & Federation)..."
export SONGBIRD_SECURITY_PROVIDER="beardog"
export SONGBIRD_SOCKET="$XDG_RUNTIME_DIR/biomeos/songbird.sock"

cd "$BIOMEOS_ROOT"  # Change to writable directory
"$PRIMAL_DIR/songbird" server --socket "$SONGBIRD_SOCKET" > "$BIOMEOS_ROOT/songbird.log" 2>&1 &
SONGBIRD_PID=$!
echo "  PID: $SONGBIRD_PID"
echo "  Socket: $SONGBIRD_SOCKET"

# Wait for Songbird socket
sleep 5
SONGBIRD_SOCK="$XDG_RUNTIME_DIR/biomeos/songbird-$FAMILY_ID.sock"
if [ ! -S "$SONGBIRD_SOCK" ]; then
  SONGBIRD_SOCK="$XDG_RUNTIME_DIR/biomeos/songbird.sock"
fi

if [ -S "$SONGBIRD_SOCK" ]; then
  echo "  ✅ Songbird socket: $SONGBIRD_SOCK"
else
  echo "  ❌ Songbird socket not found"
  echo "  Log tail:"
  tail -10 "$BIOMEOS_ROOT/songbird.log"
  exit 1
fi

echo ""
echo "═══════════════════════════════════════════════════"
echo "✅ Tower Atomic Deployed on Pixel 8a!"
echo "═══════════════════════════════════════════════════"
echo ""
echo "Active Sockets:"
ls -lh "$XDG_RUNTIME_DIR/biomeos/"*.sock 2>/dev/null || echo "  (checking...)"
echo ""
echo "Process Status:"
echo "  BearDog:  PID $BEARDOG_PID"
echo "  Songbird: PID $SONGBIRD_PID"
echo ""
echo "Logs:"
echo "  BearDog:  $BIOMEOS_ROOT/beardog.log"
echo "  Songbird: $BIOMEOS_ROOT/songbird.log"
echo ""
echo "🎊 NUCLEUS Tower Atomic operational on mobile!"
echo ""
echo "To check status:"
echo "  adb shell ls -lh /data/local/tmp/biomeos/*.sock"
echo "  adb shell cat /data/local/tmp/biomeos/beardog.log"
echo ""
