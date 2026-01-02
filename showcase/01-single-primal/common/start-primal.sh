#!/usr/bin/env bash
# Common script to start a Phase 1 primal binary

set -e

# Usage: ./start-primal.sh <primal-name> <port> [additional-args]
PRIMAL_NAME="$1"
PORT="$2"
shift 2
ADDITIONAL_ARGS="$@"

if [ -z "$PRIMAL_NAME" ] || [ -z "$PORT" ]; then
    echo "Usage: $0 <primal-name> <port> [additional-args]"
    echo "Example: $0 songbird 3000"
    exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PHASE1_BINS="$SCRIPT_DIR/../../../primalBins"
BINARY="$PHASE1_BINS/${PRIMAL_NAME}-bin"
LOG_DIR="$SCRIPT_DIR/../logs"
PID_DIR="$SCRIPT_DIR/../pids"

mkdir -p "$LOG_DIR" "$PID_DIR"

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}Starting $PRIMAL_NAME...${NC}"

# Check binary exists
if [ ! -f "$BINARY" ]; then
    echo -e "${RED}Error: Binary not found: $BINARY${NC}"
    echo "Run: cd $PHASE1_BINS && ./pull-phase1-bins.sh"
    exit 1
fi

# Check if already running
if [ -f "$PID_DIR/${PRIMAL_NAME}.pid" ]; then
    OLD_PID=$(cat "$PID_DIR/${PRIMAL_NAME}.pid")
    if kill -0 "$OLD_PID" 2>/dev/null; then
        echo -e "${YELLOW}$PRIMAL_NAME already running (PID: $OLD_PID)${NC}"
        echo "Stop it first with: ./common/stop-primal.sh $PRIMAL_NAME"
        exit 1
    else
        rm "$PID_DIR/${PRIMAL_NAME}.pid"
    fi
fi

# Check port availability
if lsof -Pi :$PORT -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo -e "${RED}Error: Port $PORT already in use${NC}"
    echo "Check with: lsof -i :$PORT"
    exit 1
fi

# Start primal in background
LOG_FILE="$LOG_DIR/${PRIMAL_NAME}.log"
echo "Starting $PRIMAL_NAME on port $PORT..."
echo "Binary: $BINARY"
echo "Log: $LOG_FILE"

# Set port environment variable
export PORT="$PORT"

# Start the primal
"$BINARY" $ADDITIONAL_ARGS > "$LOG_FILE" 2>&1 &
PID=$!

# Save PID
echo "$PID" > "$PID_DIR/${PRIMAL_NAME}.pid"

# Wait a bit for startup
sleep 2

# Check if still running
if ! kill -0 "$PID" 2>/dev/null; then
    echo -e "${RED}Error: $PRIMAL_NAME failed to start${NC}"
    echo "Check log: $LOG_FILE"
    tail -20 "$LOG_FILE"
    exit 1
fi

# Try to connect
echo "Waiting for $PRIMAL_NAME to be ready..."
MAX_WAIT=30
WAITED=0

while [ $WAITED -lt $MAX_WAIT ]; do
    if curl -s "http://localhost:$PORT/health" >/dev/null 2>&1; then
        echo -e "${GREEN}✓ $PRIMAL_NAME is ready!${NC}"
        echo "  PID: $PID"
        echo "  Port: $PORT"
        echo "  Endpoint: http://localhost:$PORT"
        exit 0
    fi
    sleep 1
    WAITED=$((WAITED + 1))
done

echo -e "${YELLOW}Warning: $PRIMAL_NAME started but health check failed${NC}"
echo "  PID: $PID"
echo "  Port: $PORT"
echo "  May still be initializing..."
echo "Check log: $LOG_FILE"
exit 0

