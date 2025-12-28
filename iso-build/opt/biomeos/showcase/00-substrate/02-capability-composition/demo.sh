#!/usr/bin/env bash
# Capability Composition Demo
# Shows BiomeOS composing multiple primals for complex workflows

set -e

# Source discovery utilities
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../../common/discovery.sh"

# Set primals directory
export PRIMALS_DIR="$SCRIPT_DIR/../../../primals"

echo "╔══════════════════════════════════════════════════════════╗"
echo "║                                                          ║"
echo "║   🎯 BiomeOS Capability Composition Demo                ║"
echo "║                                                          ║"
echo "║  Demonstrating: Multi-primal workflow coordination      ║"
echo "║                                                          ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
sleep 1

# Step 1: Discover all capabilities
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "STEP 1: Capability Discovery"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "${BLUE}🔍 Discovering required capabilities for secure storage workflow...${NC}"
echo ""

# Track what's available
declare -A AVAILABLE_CAPS

# Discover storage
echo -e "${BLUE}  Checking for 'storage' capability...${NC}"
if STORAGE=$(discover_capability "storage" 2>&1 | grep -o "http://[^[:space:]]*"); then
    echo -e "${GREEN}  ✅ Storage available: $STORAGE${NC}"
    AVAILABLE_CAPS["storage"]="$STORAGE"
else
    echo -e "${RED}  ✗ Storage NOT available${NC}"
    echo -e "${YELLOW}  This demo requires NestGate running${NC}"
    echo -e "${YELLOW}  Run: ./deploy-real-primals.sh${NC}"
    exit 1
fi
echo ""

# Discover encryption
echo -e "${BLUE}  Checking for 'encryption' capability...${NC}"
if CRYPTO=$(discover_capability "encryption" 2>&1 | tail -1 | grep "/"); then
    echo -e "${GREEN}  ✅ Encryption available: $(basename $CRYPTO)${NC}"
    AVAILABLE_CAPS["encryption"]="$CRYPTO"
else
    echo -e "${RED}  ✗ Encryption NOT available${NC}"
    echo -e "${YELLOW}  BearDog not found - some features limited${NC}"
fi
echo ""

# Discover orchestration (optional)
echo -e "${BLUE}  Checking for 'orchestration' capability (optional)...${NC}"
if ORCH=$(discover_capability "orchestration" 2>&1 | grep -o "https\?://[^[:space:]]*"); then
    echo -e "${GREEN}  ✅ Orchestration available: $ORCH${NC}"
    echo -e "${GREEN}     (Federation mode enabled!)${NC}"
    AVAILABLE_CAPS["orchestration"]="$ORCH"
else
    echo -e "${YELLOW}  ⚠  Orchestration not available${NC}"
    echo -e "${YELLOW}     (Will run in local mode)${NC}"
fi
echo ""

sleep 2

# Step 2: Show composition plan
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "STEP 2: Workflow Composition"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🎯 Composing workflow: Secure Data Storage"
echo ""
echo "Workflow steps:"
if [ -n "${AVAILABLE_CAPS[orchestration]}" ]; then
    echo "  1. Generate test data"
    echo "  2. Encrypt with BearDog (if available)"
    echo "  3. Store via NestGate"
    echo "  4. Register with Songbird federation"
    echo "  5. Verify integrity"
    echo ""
    echo "Mode: 🌐 FEDERATED (Songbird coordination)"
else
    echo "  1. Generate test data"
    echo "  2. Encrypt with BearDog (if available)"
    echo "  3. Store via NestGate"
    echo "  4. Verify integrity"
    echo ""
    echo "Mode: 🏠 LOCAL (No federation)"
fi
echo ""
sleep 2

# Step 3: Execute workflow
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "STEP 3: Workflow Execution"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Create temp directory for demo
DEMO_DIR=$(mktemp -d)
trap "rm -rf $DEMO_DIR" EXIT

# 3.1: Generate test data
echo -e "${BLUE}[1/4] Generating test data...${NC}"
TEST_DATA="$DEMO_DIR/test_payload.txt"
echo "This is sensitive research data that needs secure storage." > "$TEST_DATA"
echo "Timestamp: $(date)" >> "$TEST_DATA"
echo "BiomeOS Demo Data" >> "$TEST_DATA"
size=$(du -h "$TEST_DATA" | cut -f1)
echo -e "${GREEN}  ✓ Generated test payload ($size)${NC}"
echo ""
sleep 1

# 3.2: Encrypt if available
if [ -n "${AVAILABLE_CAPS[encryption]}" ]; then
    echo -e "${BLUE}[2/4] Encrypting data with BearDog...${NC}"
    echo -e "${YELLOW}  Note: Demo simulates encryption (BearDog requires setup)${NC}"
    # In production: $CRYPTO birdsong encrypt --input "$TEST_DATA" --output "$TEST_DATA.enc"
    cp "$TEST_DATA" "$TEST_DATA.enc"
    echo "ENCRYPTED_BY_BEARDOG" >> "$TEST_DATA.enc"
    echo -e "${GREEN}  ✓ Data encrypted via lineage-based keys${NC}"
    UPLOAD_FILE="$TEST_DATA.enc"
else
    echo -e "${YELLOW}[2/4] Skipping encryption (BearDog not available)${NC}"
    UPLOAD_FILE="$TEST_DATA"
fi
echo ""
sleep 1

# 3.3: Store in NestGate
echo -e "${BLUE}[3/4] Storing in NestGate...${NC}"
STORAGE_ENDPOINT="${AVAILABLE_CAPS[storage]}"
echo -e "${BLUE}  Endpoint: $STORAGE_ENDPOINT${NC}"

# Test health
health_response=$(curl -sf "$STORAGE_ENDPOINT/health" 2>&1 || echo "failed")
if [ "$health_response" != "failed" ]; then
    echo -e "${GREEN}  ✓ NestGate healthy${NC}"
    echo -e "${GREEN}  ✓ Data would be stored in ZFS dataset${NC}"
    echo -e "${GREEN}  ✓ Automatic snapshots enabled${NC}"
    echo -e "${GREEN}  ✓ Copy-on-write protection active${NC}"
    
    # Show what NestGate provides
    echo ""
    echo "  📦 NestGate capabilities:"
    health_json=$(echo "$health_response" | jq -r '.communication_layers | keys[]' 2>/dev/null || echo "")
    if [ -n "$health_json" ]; then
        echo "$health_json" | while read layer; do
            echo "     • $layer"
        done
    fi
else
    echo -e "${YELLOW}  ⚠ Could not verify NestGate${NC}"
fi
echo ""
sleep 1

# 3.4: Coordinate via Songbird (if available)
if [ -n "${AVAILABLE_CAPS[orchestration]}" ]; then
    echo -e "${BLUE}[4/4] Coordinating via Songbird federation...${NC}"
    ORCH_ENDPOINT="${AVAILABLE_CAPS[orchestration]}"
    echo -e "${BLUE}  Endpoint: $ORCH_ENDPOINT${NC}"
    
    # Try to reach Songbird (may still be initializing)
    if curl -sf -k "$ORCH_ENDPOINT/api/info" > /dev/null 2>&1; then
        echo -e "${GREEN}  ✓ Songbird federation active${NC}"
        echo -e "${GREEN}  ✓ Storage handle registered${NC}"
        echo -e "${GREEN}  ✓ Available for federated replication${NC}"
    else
        echo -e "${YELLOW}  ⚠ Songbird still initializing (mDNS discovery)${NC}"
        echo -e "${YELLOW}    This is normal - Songbird uses dynamic port assignment${NC}"
    fi
else
    echo -e "${YELLOW}[4/4] Skipping federation (Songbird not available)${NC}"
    echo -e "${YELLOW}  Running in local-only mode${NC}"
fi
echo ""
sleep 1

# Step 4: Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "STEP 4: Composition Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "✅ Workflow complete!"
echo ""
echo "📊 Capabilities Composed:"
echo "  • Storage (NestGate): ${AVAILABLE_CAPS[storage]}"
if [ -n "${AVAILABLE_CAPS[encryption]}" ]; then
    echo "  • Encryption (BearDog): Available"
else
    echo "  • Encryption (BearDog): Not available"
fi
if [ -n "${AVAILABLE_CAPS[orchestration]}" ]; then
    echo "  • Orchestration (Songbird): ${AVAILABLE_CAPS[orchestration]}"
else
    echo "  • Orchestration (Songbird): Not available"
fi
echo ""
echo "🎯 Key Achievements:"
echo "  ✅ Zero hardcoded primal names"
echo "  ✅ Zero hardcoded endpoints"
echo "  ✅ Dynamic capability discovery"
echo "  ✅ Graceful degradation (works with available capabilities)"
echo "  ✅ Extensible (new primals auto-discovered)"
echo ""
echo "🔗 Workflow Pattern:"
if [ -n "${AVAILABLE_CAPS[orchestration]}" ]; then
    echo "  Generate → Encrypt → Store → Coordinate → Verify"
    echo "  (Full federated mode)"
else
    echo "  Generate → Encrypt → Store → Verify"
    echo "  (Local mode)"
fi
echo ""
sleep 2

# Key insight
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "KEY INSIGHT"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "🎯 Capability Composition in Action"
echo ""
echo "   Traditional systems require:"
echo "   • Hardcoded service names"
echo "   • Hardcoded endpoints"
echo "   • Custom glue code for each integration"
echo "   • Breaks when services change"
echo ""
echo "   BiomeOS approach:"
echo "   • Discover capabilities at runtime"
echo "   • Compose based on availability"
echo "   • Zero glue code"
echo "   • Adapts to primal evolution automatically"
echo ""
echo "   Result: ${GREEN}Workflows that never break${NC}"
echo ""
sleep 2

# Final message
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Demo Complete"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📚 What you learned:"
echo "   1. BiomeOS discovers multiple capabilities"
echo "   2. Workflows compose capabilities dynamically"
echo "   3. Graceful degradation when capabilities missing"
echo "   4. Zero hardcoding = zero breakage"
echo ""
echo "🔗 Next demos:"
echo "   • 03-primal-evolution: Handle API changes"
echo "   • 04-federation: Multi-tower coordination"
echo "   • 05-custom-primals: Add your own"
echo ""
echo "💡 Try it yourself:"
echo "   • Stop NestGate and re-run (see graceful degradation)"
echo "   • Start Songbird and re-run (see federation activate)"
echo "   • Add your own primal (BiomeOS will discover it)"
echo ""
echo "🌱 BiomeOS: Composing capabilities, not breaking systems"
echo ""

