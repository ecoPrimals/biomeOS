#!/bin/bash
# Quick start script for live primal deployment
# No sudo required - runs in userspace

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PRIMALS_DIR="$SCRIPT_DIR/primals"

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 LIVE PRIMAL DEPLOYMENT (Updated Binaries)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Available Primals:"
echo "  • beardog (4.6M) - Security & Entropy"
echo "  • nestgate (3.4M) - Storage & Sovereignty"
echo "  • songbird (24M) - Routing & Federation"
echo "  • squirrel (2.9M) - AI & MCP"
echo "  • toadstool (20M) - Compute & GPU"
echo "  • petaltongue (16M) - UI & Visualization"
echo "  • loamspine (9.2M) - Legacy/Phase1"
echo ""

# Check binaries exist
if [ ! -d "$PRIMALS_DIR" ]; then
    echo "❌ Primals directory not found: $PRIMALS_DIR"
    exit 1
fi

echo "📋 Checking binary availability..."
for primal in beardog nestgate songbird squirrel toadstool petaltongue; do
    if [ -x "$PRIMALS_DIR/$primal" ]; then
        version=$("$PRIMALS_DIR/$primal" --version 2>&1 | head -1 || echo "unknown")
        echo "  ✅ $primal - $version"
    else
        echo "  ❌ $primal - not found or not executable"
    fi
done

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Deployment Strategy"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Option 1: Start All Primals (Background)"
echo "  for primal in beardog nestgate songbird squirrel toadstool petaltongue; do"
echo "    ./primals/\$primal serve > logs/\$primal.log 2>&1 &"
echo "  done"
echo ""
echo "Option 2: Start Individual Primals"
echo "  ./primals/beardog serve &       # Port 9040"
echo "  ./primals/nestgate serve &      # Port 9020"
echo "  ./primals/songbird serve &      # Port 9000"
echo "  ./primals/squirrel serve &      # Port 9010"
echo "  ./primals/toadstool serve &     # Port 9030"
echo "  ./primals/petaltongue serve &   # Port 8080"
echo ""
echo "Option 3: Docker Compose (Future)"
echo "  docker-compose up -d"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Verification"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Check running primals:"
echo "  ps aux | grep -E '(beardog|nestgate|songbird)' | grep -v grep"
echo ""
echo "Test health endpoints:"
echo "  curl http://localhost:9040/health  # BearDog"
echo "  curl http://localhost:9020/health  # Nestgate"
echo "  curl http://localhost:9000/health  # Songbird"
echo ""
echo "Run showcase demos:"
echo "  cd showcase/01-single-primal/"
echo "  ./songbird-discovery.sh"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "⚠️  NOTE: These are LIVE primals, not mocks!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Ready to deploy! See options above."
echo ""

