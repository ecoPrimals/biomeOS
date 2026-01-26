#!/usr/bin/env bash
#
# Stop all running primals in the ecosystem
#
# Usage: ./scripts/stop_ecosystem.sh

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                                                                          ║${NC}"
echo -e "${BLUE}║                🛑 STOPPING ECOSYSTEM PRIMALS 🛑                         ║${NC}"
echo -e "${BLUE}║                                                                          ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Find and stop all primal processes
echo "Finding running primals..."
primal_pids=$(ps aux | grep -E "(songbird|beardog|toadstool|nestgate|squirrel|petal-tongue)" | grep -v grep | awk '{print $2}' || true)

if [ -z "$primal_pids" ]; then
    echo -e "${GREEN}✓${NC} No primals currently running"
else
    echo "Stopping primals:"
    for pid in $primal_pids; do
        primal_name=$(ps -p $pid -o comm= 2>/dev/null || echo "unknown")
        echo -n "  Stopping $primal_name (PID $pid)..."
        kill $pid 2>/dev/null && echo -e " ${GREEN}✓${NC}" || echo -e " ${RED}✗${NC}"
    done
    
    # Give them time to shut down gracefully
    sleep 2
    
    # Force kill if still running
    for pid in $primal_pids; do
        if ps -p $pid > /dev/null 2>&1; then
            echo "  Force killing PID $pid..."
            kill -9 $pid 2>/dev/null || true
        fi
    done
fi

# Clean up socket files
echo ""
echo "Cleaning up sockets..."
rm -f /tmp/songbird-*.sock 2>/dev/null && echo -e "  ${GREEN}✓${NC} Songbird sockets removed" || true
rm -f /tmp/beardog-*.sock 2>/dev/null && echo -e "  ${GREEN}✓${NC} BearDog sockets removed" || true
rm -f /tmp/toadstool-*.sock 2>/dev/null && echo -e "  ${GREEN}✓${NC} ToadStool sockets removed" || true
rm -f /tmp/nestgate-*.sock 2>/dev/null && echo -e "  ${GREEN}✓${NC} NestGate sockets removed" || true
rm -f /tmp/squirrel-*.sock 2>/dev/null && echo -e "  ${GREEN}✓${NC} Squirrel sockets removed" || true
rm -f /tmp/petaltongue-*.sock 2>/dev/null && echo -e "  ${GREEN}✓${NC} PetalTongue sockets removed" || true
rm -f /tmp/petal-tongue-*.sock 2>/dev/null || true

echo ""
echo -e "${GREEN}✅ Ecosystem stopped and cleaned up${NC}"
echo ""

