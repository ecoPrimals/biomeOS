#!/usr/bin/env bash
# Federation Demo - Multi-Tower Coordination
# Shows automatic tower discovery and capability federation

set -e

# Source discovery utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../common/discovery.sh"

# Set primals directory
export PRIMALS_DIR="$SCRIPT_DIR/../../../primals"

echo "╔══════════════════════════════════════════════════════════╗"
echo "║                                                          ║"
echo "║     🌐 BiomeOS Federation Demo                          ║"
echo "║                                                          ║"
echo "║  Demonstrating: Multi-tower automatic coordination      ║"
echo "║                                                          ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
sleep 1

# Step 1: Local Tower Status
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "STEP 1: Local Tower Status"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

LOCAL_TOWER=$(hostname)
echo -e "${BLUE}🏰 Local Tower: $LOCAL_TOWER${NC}"
echo ""

# Discover local capabilities
echo -e "${BLUE}🔍 Discovering local capabilities...${NC}"
echo ""

local_caps=()

# Check storage
if STORAGE=$(discover_capability "storage" 2>&1 | grep -o "http://[^[:space:]]*"); then
    echo -e "${GREEN}  ✅ Storage: NestGate ($STORAGE)${NC}"
    local_caps+=("storage")
fi

# Check encryption
if CRYPTO=$(discover_capability "encryption" 2>&1 | tail -1 | grep "/"); then
    echo -e "${GREEN}  ✅ Encryption: BearDog (CLI)${NC}"
    local_caps+=("encryption")
fi

# Check compute
if COMPUTE=$(discover_capability "compute" 2>&1 | tail -1 | grep "/"); then
    echo -e "${GREEN}  ✅ Compute: Toadstool (CLI)${NC}"
    local_caps+=("compute")
fi

echo ""
echo -e "${GREEN}📊 Local Tower Capabilities: ${#local_caps[@]}${NC}"
echo ""
sleep 2

# Step 2: Federation Discovery
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "STEP 2: Federation Discovery"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "${BLUE}🔍 Checking for Songbird orchestrator (federation)...${NC}"
echo ""

# Check if Songbird is running
if pgrep -f songbird-orchestrator > /dev/null 2>&1; then
    SONGBIRD_PID=$(pgrep -f songbird-orchestrator)
    echo -e "${GREEN}✅ Songbird orchestrator running (PID: $SONGBIRD_PID)${NC}"
    echo ""
    
    # Check Songbird logs for peer discovery
    echo -e "${BLUE}📡 Checking for discovered peers...${NC}"
    echo ""
    
    LOG_FILE="$SCRIPT_DIR/../../../logs/primals/songbird.log"
    if [ -f "$LOG_FILE" ]; then
        # Look for recent peer discoveries
        peer_count=$(grep -c "Discovered peer:" "$LOG_FILE" 2>/dev/null || echo "0")
        federation_count=$(grep -c "joined federation" "$LOG_FILE" 2>/dev/null || echo "0")
        
        if [ "$peer_count" -gt 0 ]; then
            echo -e "${GREEN}✅ Federation Active!${NC}"
            echo ""
            echo "   Peer Discoveries: $peer_count"
            echo "   Federation Joins: $federation_count"
            echo ""
            
            # Show recent peer info
            echo "   Recent Peers:"
            grep "Discovered peer:" "$LOG_FILE" | tail -3 | while read -r line; do
                peer_info=$(echo "$line" | sed 's/.*Discovered peer: //' | cut -d',' -f1)
                echo "     • $peer_info"
            done
            echo ""
            
            federation_mode=true
        else
            echo -e "${YELLOW}⚠  Songbird running but no peers discovered yet${NC}"
            echo "   (This is normal - discovery takes 30-60 seconds)"
            echo ""
            federation_mode=false
        fi
    else
        echo -e "${YELLOW}⚠  Songbird log not found${NC}"
        federation_mode=false
    fi
else
    echo -e "${YELLOW}⚠  Songbird not running - federation unavailable${NC}"
    echo ""
    echo "   To enable federation:"
    echo "   1. Run: ./start-songbird.sh"
    echo "   2. Wait 30-60 seconds for peer discovery"
    echo "   3. Re-run this demo"
    echo ""
    federation_mode=false
fi

sleep 2

# Step 3: Capability Federation
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "STEP 3: Federated Capabilities"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if [ "$federation_mode" = true ]; then
    echo -e "${GREEN}🌐 Federation Mode: ACTIVE${NC}"
    echo ""
    echo "Capability Distribution:"
    echo ""
    
    # Local capabilities
    echo "  📍 Local Tower ($LOCAL_TOWER):"
    for cap in "${local_caps[@]}"; do
        echo "     • $cap"
    done
    echo ""
    
    # Federated capabilities (from Songbird)
    echo "  🌐 Federated Towers:"
    if grep -q "Discovered peer:" "$LOG_FILE" 2>/dev/null; then
        grep "Discovered peer:" "$LOG_FILE" | tail -2 | while read -r line; do
            peer_name=$(echo "$line" | sed 's/.*peer: //' | cut -d' ' -f1)
            capabilities=$(echo "$line" | sed 's/.*capabilities: //' | cut -d',' -f1)
            echo "     • $peer_name: $capabilities"
        done
    fi
    echo ""
    
    total_caps=$((${#local_caps[@]} + peer_count))
    echo -e "${GREEN}📊 Total Federated Capabilities: $total_caps across multiple towers${NC}"
    
else
    echo -e "${YELLOW}🏠 Local Mode: No federation${NC}"
    echo ""
    echo "Capability Distribution:"
    echo ""
    echo "  📍 Local Tower ($LOCAL_TOWER):"
    for cap in "${local_caps[@]}"; do
        echo "     • $cap"
    done
    echo ""
    echo -e "${YELLOW}💡 Federation would aggregate capabilities from multiple towers${NC}"
fi

echo ""
sleep 2

# Step 4: Federation Benefits
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "STEP 4: Federation Benefits"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "🎯 What Federation Provides:"
echo ""
echo "  1. ${GREEN}Automatic Discovery${NC}"
echo "     • No manual peer configuration"
echo "     • mDNS/UDP broadcasts (port 2300)"
echo "     • Towers find each other automatically"
echo ""

echo "  2. ${GREEN}Capability Aggregation${NC}"
echo "     • Access capabilities from any tower"
echo "     • Load balancing across providers"
echo "     • Redundancy and fault tolerance"
echo ""

echo "  3. ${GREEN}Trust Escalation${NC}"
echo "     • Level 0: Anonymous (anyone can join)"
echo "     • Level 1: Capability (verified functions)"
echo "     • Level 2: Identity (lineage verified)"
echo "     • Level 3: Hardware (physical ceremony)"
echo ""

echo "  4. ${GREEN}Zero Configuration${NC}"
echo "     • Start BiomeOS on each tower"
echo "     • Federation forms automatically"
echo "     • No config files, no hardcoded peers"
echo ""

if [ "$federation_mode" = true ]; then
    echo "✅ ${GREEN}Your tower is federated!${NC}"
    echo "   Other towers can discover and use your capabilities"
else
    echo "💡 ${YELLOW}Start Songbird to enable federation${NC}"
    echo "   Run: ./start-songbird.sh"
fi

echo ""
sleep 2

# Step 5: Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Demo Complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "📚 What you learned:"
echo "   1. Towers discover each other via mDNS/UDP"
echo "   2. Capabilities aggregate across federation"
echo "   3. Zero configuration required"
echo "   4. Trust escalates progressively"
echo ""

echo "🔗 Federation Status:"
if [ "$federation_mode" = true ]; then
    echo "   Status: ✅ ACTIVE"
    echo "   Peers: $peer_count discovered"
    echo "   Mode: Multi-tower federation"
else
    echo "   Status: 🏠 LOCAL ONLY"
    echo "   Mode: Single tower"
    echo "   Action: Start Songbird to enable federation"
fi

echo ""
echo "🌐 Real-World Use Cases:"
echo "   • Research labs: Share compute/storage"
echo "   • Personal devices: Seamless capability access"
echo "   • Community mesh: Share resources"
echo ""

echo "🔗 Next demos:"
echo "   • 05-custom-primals: Add your own to federation"
echo "   • ../01-nestgate: Deep dive into storage"
echo "   • ../02-birdsong-p2p: P2P tunnels with BearDog"
echo ""

echo "🌱 BiomeOS: Federation without configuration"
echo ""

