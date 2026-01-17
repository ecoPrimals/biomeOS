#!/usr/bin/env bash
#
# Ecosystem Deployment Script
# Executes the 3-layer primal deployment based on Neural API graphs
#
# Usage: ./scripts/deploy_ecosystem.sh [family_id]

set -euo pipefail

FAMILY_ID="${1:-nat0}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PRIMAL_BIN="$PROJECT_ROOT/plasmidBin/primals"
LOG_DIR="/tmp/primals"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Create log directory
mkdir -p "$LOG_DIR"

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                                                                          ║${NC}"
echo -e "${BLUE}║        🚀 ECOSYSTEM DEPLOYMENT - NEURAL API EXECUTION 🚀                ║${NC}"
echo -e "${BLUE}║                                                                          ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}Family ID:${NC} $FAMILY_ID"
echo -e "${GREEN}Primal Binaries:${NC} $PRIMAL_BIN"
echo -e "${GREEN}Logs:${NC} $LOG_DIR"
echo ""

# Function to check if socket exists
check_socket() {
    local socket_path="$1"
    local primal_name="$2"
    local max_wait=30
    local waited=0
    
    echo -n "   Waiting for socket..."
    while [ ! -S "$socket_path" ] && [ $waited -lt $max_wait ]; do
        sleep 1
        waited=$((waited + 1))
        echo -n "."
    done
    
    if [ -S "$socket_path" ]; then
        echo -e " ${GREEN}✓${NC}"
        return 0
    else
        echo -e " ${RED}✗ TIMEOUT${NC}"
        return 1
    fi
}

# Function to launch a primal
launch_primal() {
    local binary_name="$1"
    local primal_name="$2"
    local socket_path="$3"
    local capabilities="$4"
    
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}🚀 Launching:${NC} $primal_name"
    echo -e "${GREEN}   Binary:${NC} $binary_name"
    echo -e "${GREEN}   Socket:${NC} $socket_path"
    echo -e "${GREEN}   Capabilities:${NC} $capabilities"
    
    # Clean old socket
    if [ -S "$socket_path" ]; then
        echo "   Removing old socket..."
        rm -f "$socket_path"
    fi
    
    # Launch primal
    local binary_path="$PRIMAL_BIN/$binary_name"
    if [ ! -f "$binary_path" ]; then
        echo -e "${RED}   ✗ Binary not found: $binary_path${NC}"
        return 1
    fi
    
    chmod +x "$binary_path"
    
    local log_file="$LOG_DIR/${primal_name}-${FAMILY_ID}.log"
    echo "   Starting process (log: $log_file)..."
    
    # Start primal in background with environment
    env \
    FAMILY_ID="$FAMILY_ID" \
    RUST_LOG=info \
    SECURITY_ENDPOINT="http://localhost:8765" \
    nohup "$binary_path" > "$log_file" 2>&1 &
    
    local pid=$!
    echo -e "   ${GREEN}PID: $pid${NC}"
    
    # Wait for socket
    if check_socket "$socket_path" "$primal_name"; then
        echo -e "   ${GREEN}✅ $primal_name is UP!${NC}"
        return 0
    else
        echo -e "   ${RED}❌ $primal_name failed to start${NC}"
        echo "   Last 10 log lines:"
        tail -10 "$log_file" | sed 's/^/      /'
        return 1
    fi
}

# Layer 0: Security Foundation (BearDog must start first)
echo ""
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}   LAYER 0: SECURITY FOUNDATION (Required for discovery)${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# BearDog (Security) - MUST START FIRST  
# Note: BearDog uses its own socket naming: /tmp/beardog-{family}-{node}.sock
launch_primal \
    "beardog-server" \
    "beardog" \
    "/tmp/beardog-default-default.sock" \
    "security,crypto,trust,genetic_lineage"

echo ""
echo -e "${GREEN}✅ Security foundation established!${NC}"

# Layer 1: NUCLEUS Enclave
echo ""
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}   LAYER 1: NUCLEUS ENCLAVE (Tower + Node + Nest)${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Tower = Songbird (Discovery) - Now with BearDog available
SECURITY_ENDPOINT="http://localhost:8765" \
launch_primal \
    "songbird-orchestrator" \
    "songbird" \
    "/tmp/songbird-${FAMILY_ID}.sock" \
    "discovery,mesh,coordination"

# Node = ToadStool (Compute)
launch_primal \
    "toadstool" \
    "toadstool" \
    "/tmp/toadstool-${FAMILY_ID}.sock" \
    "compute,orchestration,collaborative_intelligence"

# Nest = NestGate (Storage)
launch_primal \
    "nestgate" \
    "nestgate" \
    "/tmp/nestgate-${FAMILY_ID}.sock" \
    "storage,persistence"

echo ""
echo -e "${GREEN}✅ NUCLEUS Enclave deployed!${NC}"

# Layer 2: Intelligence Layer
echo ""
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}   LAYER 2: INTELLIGENCE LAYER${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Squirrel (Intelligence)
launch_primal \
    "squirrel" \
    "squirrel" \
    "/tmp/squirrel-${FAMILY_ID}.sock" \
    "meta_ai,ai_routing,tool_orchestration,mcp"

echo ""
echo -e "${GREEN}✅ Intelligence layer deployed!${NC}"

# Layer 3: BenchTop UI
echo ""
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}   LAYER 3: UNIVERSAL BENCHTOP UI${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# PetalTongue (Universal UI - headless mode for server)
launch_primal \
    "petal-tongue-headless" \
    "petaltongue" \
    "/tmp/petaltongue-${FAMILY_ID}.sock" \
    "visualization,ui,sensory,real_time_events"

echo ""
echo -e "${GREEN}✅ BenchTop UI deployed!${NC}"

# Final Verification
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}   FINAL VERIFICATION${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Deployed Primals:"
echo ""

declare -a sockets=(
    "/tmp/beardog-default-default.sock:🔐 BearDog (Security Foundation)"
    "/tmp/songbird-${FAMILY_ID}.sock:🏰 Tower (Songbird)"
    "/tmp/toadstool-${FAMILY_ID}.sock:🧠 Node (ToadStool)"
    "/tmp/nestgate-${FAMILY_ID}.sock:💾 Nest (NestGate)"
    "/tmp/squirrel-${FAMILY_ID}.sock:🐿️  Squirrel (Intelligence)"
    "/tmp/petaltongue-${FAMILY_ID}.sock:🌸 PetalTongue (UI)"
)

all_healthy=true
for socket_info in "${sockets[@]}"; do
    IFS=':' read -r socket_path primal_label <<< "$socket_info"
    if [ -S "$socket_path" ]; then
        echo -e "  ${GREEN}✓${NC} $primal_label"
    else
        echo -e "  ${RED}✗${NC} $primal_label (socket missing)"
        all_healthy=false
    fi
done

echo ""
if $all_healthy; then
    echo -e "${BLUE}╔══════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║                                                                          ║${NC}"
    echo -e "${BLUE}║           🎊🎊🎊 ECOSYSTEM DEPLOYMENT COMPLETE! 🎊🎊🎊                  ║${NC}"
    echo -e "${BLUE}║                                                                          ║${NC}"
    echo -e "${BLUE}╚══════════════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${GREEN}All 6 primals are operational!${NC}"
    echo ""
    echo "Primal Topology:"
    echo "  Layer 0: Security Foundation"
    echo "    🔐 BearDog (crypto, trust, genetic lineage)"
    echo ""
    echo "  Layer 1: NUCLEUS Enclave"
    echo "    🏰 Songbird (discovery, mesh)"
    echo "    🧠 ToadStool (compute, orchestration)"
    echo "    💾 NestGate (storage, persistence)"
    echo ""
    echo "  Layer 2: Intelligence"
    echo "    🐿️  Squirrel (meta-AI, MCP)"
    echo ""
    echo "  Layer 3: Universal Interface"
    echo "    🌸 PetalTongue (benchTop UI)"
    echo ""
    echo "Next Steps:"
    echo "  • Test inter-primal discovery"
    echo "  • Verify genetic lineage"
    echo "  • Test capability-based queries"
    echo "  • Monitor real-time events"
    echo ""
    echo "Logs: $LOG_DIR"
    echo "View logs: tail -f $LOG_DIR/*.log"
    exit 0
else
    echo -e "${RED}❌ Deployment incomplete - some primals failed to start${NC}"
    echo ""
    echo "Check logs in: $LOG_DIR"
    exit 1
fi

