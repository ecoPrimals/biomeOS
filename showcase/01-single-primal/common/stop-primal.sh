#!/usr/bin/env bash
# Common script to stop a Phase 1 primal binary

set -e

PRIMAL_NAME="$1"

if [ -z "$PRIMAL_NAME" ]; then
    echo "Usage: $0 <primal-name>"
    echo "Example: $0 songbird"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PID_DIR="$SCRIPT_DIR/../pids"
PID_FILE="$PID_DIR/${PRIMAL_NAME}.pid"

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

if [ ! -f "$PID_FILE" ]; then
    echo -e "${YELLOW}No PID file found for $PRIMAL_NAME${NC}"
    
    # Try to find by process name
    PIDS=$(pgrep -f "${PRIMAL_NAME}-bin" || true)
    if [ -n "$PIDS" ]; then
        echo -e "${YELLOW}Found running process(es), killing...${NC}"
        echo "$PIDS" | xargs kill -TERM 2>/dev/null || true
        sleep 2
        # Force kill if still running
        echo "$PIDS" | xargs kill -9 2>/dev/null || true
        echo -e "${GREEN}✓ Stopped $PRIMAL_NAME${NC}"
    else
        echo -e "${GREEN}$PRIMAL_NAME not running${NC}"
    fi
    exit 0
fi

PID=$(cat "$PID_FILE")

if ! kill -0 "$PID" 2>/dev/null; then
    echo -e "${YELLOW}$PRIMAL_NAME not running (stale PID: $PID)${NC}"
    rm "$PID_FILE"
    exit 0
fi

echo "Stopping $PRIMAL_NAME (PID: $PID)..."

# Try graceful shutdown first
kill -TERM "$PID" 2>/dev/null || true

# Wait up to 10 seconds for graceful shutdown
WAITED=0
while [ $WAITED -lt 10 ]; do
    if ! kill -0 "$PID" 2>/dev/null; then
        echo -e "${GREEN}✓ $PRIMAL_NAME stopped gracefully${NC}"
        rm "$PID_FILE"
        exit 0
    fi
    sleep 1
    WAITED=$((WAITED + 1))
done

# Force kill if still running
echo -e "${YELLOW}Forcing shutdown...${NC}"
kill -9 "$PID" 2>/dev/null || true
sleep 1

if ! kill -0 "$PID" 2>/dev/null; then
    echo -e "${GREEN}✓ $PRIMAL_NAME stopped (forced)${NC}"
    rm "$PID_FILE"
else
    echo -e "${RED}Error: Could not stop $PRIMAL_NAME${NC}"
    exit 1
fi

