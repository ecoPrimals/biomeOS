#!/usr/bin/env bash
# 🌸 Deploy NUCLEUS + PetalTongue
#
# This script deploys a complete biomeOS NUCLEUS and visualizes it with PetalTongue.
#
# What it does:
# 1. Starts BearDog, ToadStool, NestGate, Squirrel (NUCLEUS primals)
# 2. Starts biomeOS API (topology provider)
# 3. Launches PetalTongue (visualization)
# 4. Enables 3D rendering (ToadStool GPU) and entropy collection (BearDog)
#
# Usage:
#   ./scripts/deploy-nucleus-with-ui.sh
#   ./scripts/deploy-nucleus-with-ui.sh --headless   # TUI mode (SSH-friendly)
#   ./scripts/deploy-nucleus-with-ui.sh --3d         # Enable 3D mode
#   ./scripts/deploy-nucleus-with-ui.sh --help       # Show help

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configuration
FAMILY_ID="${FAMILY_ID:-nat0}"
BIOMEOS_PORT="${BIOMEOS_PORT:-8080}"
BIOMEOS_URL="http://localhost:${BIOMEOS_PORT}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Mode flags
HEADLESS=false
ENABLE_3D=false
ENTROPY_COLLECTION=true

# PID tracking
declare -a PIDS=()

show_help() {
    cat << EOF
🌸 NUCLEUS + PetalTongue Deployment Script

USAGE:
    $0 [OPTIONS]

OPTIONS:
    -h, --help          Show this help message
    --headless          Use PetalTongue TUI mode (SSH-friendly)
    --3d                Enable 3D GPU rendering via ToadStool
    --no-entropy        Disable entropy collection
    --family-id ID      Set family ID (default: nat0)
    --port PORT         Set biomeOS API port (default: 8080)

EXAMPLES:
    # Deploy with GUI
    $0

    # Deploy with TUI (works over SSH)
    $0 --headless

    # Deploy with 3D rendering enabled
    $0 --3d

    # Custom family and port
    $0 --family-id mylab --port 9000

WHAT IT DEPLOYS:
    NUCLEUS Primals:
    • BearDog    - Security, crypto, keys, trust
    • ToadStool  - GPU compute, 3D rendering
    • NestGate   - Storage, provenance
    • Squirrel   - AI optimization (optional)

    Coordination:
    • biomeOS API - Topology provider, discovery
    • PetalTongue - Visualization, entropy collection

DOCUMENTATION:
    Full guide: PETALTONGUE_NUCLEUS_DEPLOYMENT_JAN13.md
    Integration: PETALTONGUE_INTEGRATION_JAN13.md
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        --headless)
            HEADLESS=true
            shift
            ;;
        --3d)
            ENABLE_3D=true
            shift
            ;;
        --no-entropy)
            ENTROPY_COLLECTION=false
            shift
            ;;
        --family-id)
            FAMILY_ID="$2"
            shift 2
            ;;
        --port)
            BIOMEOS_PORT="$2"
            BIOMEOS_URL="http://localhost:${BIOMEOS_PORT}"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Run '$0 --help' for usage information"
            exit 1
            ;;
    esac
done

cd "${PROJECT_ROOT}"

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}🛑 Stopping all services...${NC}"
    
    # Kill all tracked PIDs
    for pid in "${PIDS[@]}"; do
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
        fi
    done
    
    # Wait a moment
    sleep 2
    
    # Force kill if needed
    for pid in "${PIDS[@]}"; do
        if kill -0 "$pid" 2>/dev/null; then
            kill -9 "$pid" 2>/dev/null || true
        fi
    done
    
    echo -e "${GREEN}✅ All services stopped${NC}"
    exit 0
}

trap cleanup SIGINT SIGTERM

echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🌸 NUCLEUS + PetalTongue Deployment${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${BLUE}Configuration:${NC}"
echo -e "  Family ID:          ${FAMILY_ID}"
echo -e "  biomeOS API Port:   ${BIOMEOS_PORT}"
echo -e "  UI Mode:            $([ "$HEADLESS" = true ] && echo "Headless (TUI)" || echo "GUI")"
echo -e "  3D Rendering:       $([ "$ENABLE_3D" = true ] && echo "Enabled" || echo "Disabled")"
echo -e "  Entropy Collection: $([ "$ENTROPY_COLLECTION" = true ] && echo "Enabled" || echo "Disabled")"
echo ""

# Phase 1: Deploy NUCLEUS Primals
echo -e "${GREEN}━━━ Phase 1: Deploying NUCLEUS Primals ━━━${NC}"
echo ""

# Check for binaries
if [[ ! -x "plasmidBin/beardog" ]]; then
    echo -e "${RED}❌ BearDog binary not found in plasmidBin/${NC}"
    echo "Please ensure primals are built and harvested to plasmidBin/"
    exit 1
fi

# Start BearDog (Security)
echo -e "${BLUE}🔒 Starting BearDog (Security)...${NC}"
FAMILY_ID="${FAMILY_ID}" \
NODE_ID="nucleus-beardog" \
RUST_LOG=info \
./plasmidBin/beardog > /tmp/beardog.log 2>&1 &
BEARDOG_PID=$!
PIDS+=($BEARDOG_PID)
echo -e "${GREEN}✅ BearDog started (PID: ${BEARDOG_PID})${NC}"

# Start ToadStool (Compute/GPU)
if [[ -x "plasmidBin/toadstool" ]]; then
    echo -e "${BLUE}💻 Starting ToadStool (GPU/Compute)...${NC}"
    FAMILY_ID="${FAMILY_ID}" \
    NODE_ID="nucleus-toadstool" \
    RUST_LOG=info \
    ./plasmidBin/toadstool > /tmp/toadstool.log 2>&1 &
    TOADSTOOL_PID=$!
    PIDS+=($TOADSTOOL_PID)
    echo -e "${GREEN}✅ ToadStool started (PID: ${TOADSTOOL_PID})${NC}"
else
    echo -e "${YELLOW}⚠️  ToadStool binary not found (optional, skipping)${NC}"
fi

# Start NestGate (Storage)
if [[ -x "plasmidBin/nestgate" ]]; then
    echo -e "${BLUE}📦 Starting NestGate (Storage)...${NC}"
    FAMILY_ID="${FAMILY_ID}" \
    NODE_ID="nucleus-nestgate" \
    RUST_LOG=info \
    ./plasmidBin/nestgate > /tmp/nestgate.log 2>&1 &
    NESTGATE_PID=$!
    PIDS+=($NESTGATE_PID)
    echo -e "${GREEN}✅ NestGate started (PID: ${NESTGATE_PID})${NC}"
else
    echo -e "${YELLOW}⚠️  NestGate binary not found (optional, skipping)${NC}"
fi

# Start Squirrel (AI)
if [[ -x "plasmidBin/squirrel" ]]; then
    echo -e "${BLUE}🧠 Starting Squirrel (AI)...${NC}"
    FAMILY_ID="${FAMILY_ID}" \
    NODE_ID="nucleus-squirrel" \
    RUST_LOG=info \
    ./plasmidBin/squirrel > /tmp/squirrel.log 2>&1 &
    SQUIRREL_PID=$!
    PIDS+=($SQUIRREL_PID)
    echo -e "${GREEN}✅ Squirrel started (PID: ${SQUIRREL_PID})${NC}"
else
    echo -e "${YELLOW}⚠️  Squirrel binary not found (optional, skipping)${NC}"
fi

# Wait for primals to initialize
echo ""
echo -e "${BLUE}⏳ Waiting for primals to initialize (5 seconds)...${NC}"
sleep 5

# Check Unix sockets
echo -e "${BLUE}🔍 Checking Unix sockets...${NC}"
SOCKET_DIR="/run/user/$(id -u)"
SOCKET_COUNT=0
for socket in beardog toadstool nestgate squirrel; do
    if [[ -S "${SOCKET_DIR}/${socket}.sock" ]]; then
        echo -e "${GREEN}  ✅ ${socket}.sock${NC}"
        ((SOCKET_COUNT++))
    fi
done

if [[ $SOCKET_COUNT -eq 0 ]]; then
    echo -e "${RED}❌ No Unix sockets found! Primals may not have started correctly.${NC}"
    echo "Check logs in /tmp/{beardog,toadstool,nestgate,squirrel}.log"
    cleanup
fi

echo -e "${GREEN}✅ ${SOCKET_COUNT} primal(s) running${NC}"
echo ""

# Phase 2: Start biomeOS API
echo -e "${GREEN}━━━ Phase 2: Starting biomeOS API ━━━${NC}"
echo ""

echo -e "${BLUE}📡 Starting biomeOS API on port ${BIOMEOS_PORT}...${NC}"
BIOMEOS_PORT="${BIOMEOS_PORT}" \
BIOMEOS_BIND_ADDRESS="127.0.0.1" \
FAMILY_ID="${FAMILY_ID}" \
RUST_LOG=info \
cargo run -q -p biomeos-api > /tmp/biomeos-api.log 2>&1 &
API_PID=$!
PIDS+=($API_PID)

# Wait for API to start
echo -e "${BLUE}⏳ Waiting for API to be ready...${NC}"
for i in {1..30}; do
    if curl -s "http://localhost:${BIOMEOS_PORT}/api/v1/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ biomeOS API ready!${NC}"
        break
    fi
    if [[ $i -eq 30 ]]; then
        echo -e "${RED}❌ API not responding after 30s${NC}"
        echo "Check logs: /tmp/biomeos-api.log"
        cleanup
    fi
    sleep 1
done

# Test topology endpoint
PRIMAL_COUNT=$(curl -s "http://localhost:${BIOMEOS_PORT}/api/v1/topology" | jq -r '.primals | length' 2>/dev/null || echo "0")
echo -e "${GREEN}✅ Topology: ${PRIMAL_COUNT} primal(s) discovered${NC}"
echo ""

# Phase 3: Launch PetalTongue
echo -e "${GREEN}━━━ Phase 3: Launching PetalTongue UI ━━━${NC}"
echo ""

# Build environment variables
declare -a PT_ENV=()
PT_ENV+=("BIOMEOS_URL=${BIOMEOS_URL}")
PT_ENV+=("PETALTONGUE_REFRESH_INTERVAL=2.0")
PT_ENV+=("RUST_LOG=info")

if [[ "$ENABLE_3D" = true ]]; then
    PT_ENV+=("PETALTONGUE_RENDER_MODE=3d")
    PT_ENV+=("PETALTONGUE_RENDER_BACKEND=toadstool")
    echo -e "${BLUE}🎨 3D GPU rendering: ENABLED${NC}"
fi

if [[ "$ENTROPY_COLLECTION" = true ]]; then
    PT_ENV+=("PETALTONGUE_ENTROPY_COLLECTION=enabled")
    PT_ENV+=("PETALTONGUE_ENTROPY_TARGET=beardog")
    echo -e "${BLUE}🔒 Entropy collection: ENABLED${NC}"
fi

# Launch PetalTongue
if [[ "$HEADLESS" = true ]]; then
    echo -e "${BLUE}🌸 Launching PetalTongue (TUI mode)...${NC}"
    env "${PT_ENV[@]}" ./plasmidBin/petal-tongue-headless --mode terminal &
else
    echo -e "${BLUE}🌸 Launching PetalTongue (GUI mode)...${NC}"
    env "${PT_ENV[@]}" ./plasmidBin/petal-tongue &
fi

PT_PID=$!
PIDS+=($PT_PID)

echo -e "${GREEN}✅ PetalTongue launched (PID: ${PT_PID})${NC}"
echo ""

# Phase 4: Status Summary
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🎉 Deployment Complete!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

echo -e "${BLUE}📊 Running Services:${NC}"
echo -e "  • BearDog:    PID ${BEARDOG_PID}"
[[ -n "${TOADSTOOL_PID:-}" ]] && echo -e "  • ToadStool:  PID ${TOADSTOOL_PID}"
[[ -n "${NESTGATE_PID:-}" ]] && echo -e "  • NestGate:   PID ${NESTGATE_PID}"
[[ -n "${SQUIRREL_PID:-}" ]] && echo -e "  • Squirrel:   PID ${SQUIRREL_PID}"
echo -e "  • biomeOS API: PID ${API_PID}"
echo -e "  • PetalTongue: PID ${PT_PID}"
echo ""

echo -e "${BLUE}📡 Endpoints:${NC}"
echo -e "  • API Health:    ${BIOMEOS_URL}/api/v1/health"
echo -e "  • Topology:      ${BIOMEOS_URL}/api/v1/topology"
echo -e "  • Primals:       ${BIOMEOS_URL}/api/v1/primals"
echo -e "  • Real-time:     ${BIOMEOS_URL}/api/v1/events/stream"
echo ""

echo -e "${BLUE}📁 Logs:${NC}"
echo -e "  • BearDog:    /tmp/beardog.log"
echo -e "  • ToadStool:  /tmp/toadstool.log"
echo -e "  • NestGate:   /tmp/nestgate.log"
echo -e "  • Squirrel:   /tmp/squirrel.log"
echo -e "  • API:        /tmp/biomeos-api.log"
echo ""

if [[ "$HEADLESS" = false ]]; then
    echo -e "${BLUE}🌸 PetalTongue Controls:${NC}"
    echo -e "  • [Q] Quit"
    echo -e "  • [R] Refresh topology"
    echo -e "  • [3] Toggle 3D mode"
    echo -e "  • [E] Toggle entropy collection"
    echo ""
fi

echo -e "${YELLOW}Press Ctrl+C to stop all services${NC}"
echo ""

# Wait for all processes
wait
