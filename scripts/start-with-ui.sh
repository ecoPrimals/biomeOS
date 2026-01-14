#!/usr/bin/env bash
# 🌸 Start biomeOS with PetalTongue UI
#
# This script starts the biomeOS API server and PetalTongue visualization UI.
# 
# Usage:
#   ./scripts/start-with-ui.sh          # Start both services
#   ./scripts/start-with-ui.sh --help   # Show this help

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
BIOMEOS_PORT="${BIOMEOS_PORT:-8080}"
BIOMEOS_URL="http://localhost:${BIOMEOS_PORT}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

show_help() {
    cat << EOF
🌸 biomeOS + PetalTongue UI Launcher

USAGE:
    $0 [OPTIONS]

OPTIONS:
    -h, --help              Show this help message
    -p, --port PORT         Set biomeOS API port (default: 8080)
    --headless              Use headless mode (TUI instead of GUI)
    --api-only              Only start biomeOS API (no UI)
    --ui-only               Only start PetalTongue UI (assumes API running)

EXAMPLES:
    # Start both API and GUI
    $0

    # Start on custom port
    $0 --port 3000

    # Start with headless TUI (works over SSH)
    $0 --headless

    # Start only API server
    $0 --api-only

ENVIRONMENT VARIABLES:
    BIOMEOS_PORT            Port for biomeOS API (default: 8080)
    PETALTONGUE_REFRESH     Refresh interval in seconds (default: 5.0)
    RUST_LOG                Log level (default: info)

BINARIES:
    API:        cargo run -p biomeos-api
    GUI:        plasmidBin/petal-tongue
    Headless:   plasmidBin/petal-tongue-headless

DOCUMENTATION:
    Full guide: PETALTONGUE_INTEGRATION_JAN13.md
    API docs:   docs/api/
EOF
}

# Parse arguments
HEADLESS=false
API_ONLY=false
UI_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -p|--port)
            BIOMEOS_PORT="$2"
            BIOMEOS_URL="http://localhost:${BIOMEOS_PORT}"
            shift 2
            ;;
        --headless)
            HEADLESS=true
            shift
            ;;
        --api-only)
            API_ONLY=true
            shift
            ;;
        --ui-only)
            UI_ONLY=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Run '$0 --help' for usage information"
            exit 1
            ;;
    esac
done

cd "${PROJECT_ROOT}"

# Check if binaries exist
if [[ "${UI_ONLY}" == false && "${API_ONLY}" == false ]]; then
    if [[ "${HEADLESS}" == true ]]; then
        if [[ ! -x "plasmidBin/petal-tongue-headless" ]]; then
            echo -e "${YELLOW}⚠️  PetalTongue headless binary not found in plasmidBin/${NC}"
            echo "Building PetalTongue..."
            (cd ../petalTongue && cargo build --release --bin petal-tongue-headless)
            cp ../petalTongue/target/release/petal-tongue-headless plasmidBin/
        fi
    else
        if [[ ! -x "plasmidBin/petal-tongue" ]]; then
            echo -e "${YELLOW}⚠️  PetalTongue binary not found in plasmidBin/${NC}"
            echo "Building PetalTongue..."
            (cd ../petalTongue && cargo build --release --bin petal-tongue)
            cp ../petalTongue/target/release/petal-tongue plasmidBin/
        fi
    fi
fi

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}🛑 Stopping services...${NC}"
    if [[ -n "${API_PID:-}" ]]; then
        kill "${API_PID}" 2>/dev/null || true
    fi
    if [[ -n "${UI_PID:-}" ]]; then
        kill "${UI_PID}" 2>/dev/null || true
    fi
    exit 0
}

trap cleanup SIGINT SIGTERM

# Start biomeOS API
if [[ "${UI_ONLY}" == false ]]; then
    echo -e "${GREEN}🚀 Starting biomeOS API on port ${BIOMEOS_PORT}...${NC}"
    BIOMEOS_PORT="${BIOMEOS_PORT}" cargo run -p biomeos-api &
    API_PID=$!
    
    # Wait for API to start
    echo -e "${BLUE}⏳ Waiting for API to start...${NC}"
    for i in {1..30}; do
        if curl -s "http://localhost:${BIOMEOS_PORT}/api/v1/health" > /dev/null 2>&1; then
            echo -e "${GREEN}✅ biomeOS API ready!${NC}"
            break
        fi
        if [[ $i -eq 30 ]]; then
            echo -e "${YELLOW}⚠️  API not responding after 30s, continuing anyway...${NC}"
        fi
        sleep 1
    done
fi

# Start PetalTongue UI
if [[ "${API_ONLY}" == false ]]; then
    echo -e "${GREEN}🌸 Starting PetalTongue UI...${NC}"
    
    if [[ "${HEADLESS}" == true ]]; then
        echo -e "${BLUE}📊 Headless mode (TUI)${NC}"
        BIOMEOS_URL="${BIOMEOS_URL}" ./plasmidBin/petal-tongue-headless --mode terminal &
    else
        echo -e "${BLUE}🖼️  GUI mode${NC}"
        BIOMEOS_URL="${BIOMEOS_URL}" ./plasmidBin/petal-tongue &
    fi
    UI_PID=$!
    
    echo -e "${GREEN}✅ PetalTongue launched!${NC}"
fi

# Show status
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🌳 biomeOS + PetalTongue Running!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

if [[ "${UI_ONLY}" == false ]]; then
    echo -e "${BLUE}📡 API Server:${NC}       http://localhost:${BIOMEOS_PORT}"
    echo -e "${BLUE}📊 Health:${NC}          http://localhost:${BIOMEOS_PORT}/api/v1/health"
    echo -e "${BLUE}🗺️  Topology:${NC}        http://localhost:${BIOMEOS_PORT}/api/v1/topology"
fi

if [[ "${API_ONLY}" == false ]]; then
    if [[ "${HEADLESS}" == true ]]; then
        echo -e "${BLUE}🌸 UI Mode:${NC}          Headless (TUI)"
    else
        echo -e "${BLUE}🌸 UI Mode:${NC}          GUI (graphical)"
    fi
fi

echo ""
echo -e "${YELLOW}Press Ctrl+C to stop all services${NC}"
echo ""

# Wait for processes
if [[ "${UI_ONLY}" == false ]]; then
    wait "${API_PID}"
fi

if [[ "${API_ONLY}" == false ]]; then
    wait "${UI_PID}"
fi

