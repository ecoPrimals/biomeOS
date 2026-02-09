#!/bin/bash
# Dual USB Spore AI Deployment Script
# Deploys 2 USB spores as complementary AI coordinators:
#   - Spore 1: Local AI Coordinator (Toadstool compute)
#   - Spore 2: API AI Coordinator (External AI routing)
#
# Usage: ./deploy_dual_spore_ai.sh [spore1_path] [spore2_path]
#
# This simulates a LAN stack or sub-groups before full LAN validation.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default spore paths (detect mounted USB drives)
SPORE1_PATH="${1:-}"
SPORE2_PATH="${2:-}"

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}     biomeOS Dual Spore AI Deployment                       ${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

# Detect USB drives if paths not provided
detect_usb_spores() {
    echo -e "${YELLOW}Detecting USB LiveSpores...${NC}"
    
    local usb_drives=()
    for mount in /media/$USER/*; do
        if [ -d "$mount/biomeOS" ]; then
            usb_drives+=("$mount/biomeOS")
            echo -e "  Found: ${GREEN}$mount/biomeOS${NC}"
        fi
    done
    
    # Also check /run/media for some distros
    for mount in /run/media/$USER/*; do
        if [ -d "$mount/biomeOS" ]; then
            usb_drives+=("$mount/biomeOS")
            echo -e "  Found: ${GREEN}$mount/biomeOS${NC}"
        fi
    done
    
    if [ ${#usb_drives[@]} -lt 2 ]; then
        echo -e "${RED}Error: Need 2 USB LiveSpores, found ${#usb_drives[@]}${NC}"
        echo "Usage: $0 [spore1_path] [spore2_path]"
        echo "Example: $0 /media/user/USB1/biomeOS /media/user/USB2/biomeOS"
        exit 1
    fi
    
    SPORE1_PATH="${usb_drives[0]}"
    SPORE2_PATH="${usb_drives[1]}"
}

# Validate spore paths
validate_spores() {
    echo -e "${YELLOW}Validating spore paths...${NC}"
    
    for spore in "$SPORE1_PATH" "$SPORE2_PATH"; do
        if [ ! -d "$spore" ]; then
            echo -e "${RED}Error: Spore path does not exist: $spore${NC}"
            exit 1
        fi
        
        if [ ! -f "$spore/.family.seed" ]; then
            echo -e "${RED}Error: No family seed found at $spore/.family.seed${NC}"
            exit 1
        fi
        
        echo -e "  ✅ ${GREEN}$spore${NC}"
    done
}

# Update spore with latest binaries
update_spore_binaries() {
    local spore_path="$1"
    local spore_name="$2"
    
    echo -e "${YELLOW}Updating $spore_name binaries...${NC}"
    
    # Copy Neural API server
    if [ -f "$BIOMEOS_ROOT/plasmidBin/neural-api-server" ]; then
        cp "$BIOMEOS_ROOT/plasmidBin/neural-api-server" "$spore_path/plasmidBin/"
        echo -e "  ✅ neural-api-server"
    fi
    
    # Copy primal binaries
    for primal in beardog songbird squirrel toadstool; do
        if [ -f "$BIOMEOS_ROOT/plasmidBin/primals/$primal/$primal" ]; then
            mkdir -p "$spore_path/plasmidBin/primals/$primal"
            cp "$BIOMEOS_ROOT/plasmidBin/primals/$primal/$primal" "$spore_path/plasmidBin/primals/$primal/"
            echo -e "  ✅ $primal"
        fi
    done
    
    # Copy latest graphs
    cp "$BIOMEOS_ROOT/graphs/tower_atomic_bootstrap.toml" "$spore_path/graphs/"
    cp "$BIOMEOS_ROOT/graphs/node_atomic_compute.toml" "$spore_path/graphs/" 2>/dev/null || true
    echo -e "  ✅ deployment graphs"
}

# Create local AI coordinator config (Spore 1)
create_local_coordinator_config() {
    local spore_path="$1"
    
    echo -e "${YELLOW}Creating Local AI Coordinator config...${NC}"
    
    cat > "$spore_path/config/ai_coordinator_local.toml" << 'EOF'
# Local AI Coordinator Configuration
# Spore 1: Focuses on local compute (Toadstool)

[coordinator]
role = "local"
priority_providers = ["toadstool", "local"]
fallback_to_api = true

[toadstool]
enabled = true
socket = "/run/user/1000/biomeos/toadstool-1894e909e454.jsonrpc.sock"
capabilities = ["compute", "ai_local", "gpu"]

[resources]
# Local compute resources
max_gpu_memory_mb = 8192  # RTX 2070 SUPER
max_cpu_cores = 24
local_model_cache = "/var/cache/biomeos/models"

[routing]
# Route local AI tasks here
text_generation_local = true
image_inference_local = true
embedding_local = true
# Fallback to API coordinator for tasks requiring external models
text_generation_api = false
image_generation_api = false  # Route to Spore 2

[federation]
peer_socket = "/run/user/1000/biomeos/spore2-neural-api.sock"
sync_interval_ms = 5000
EOF

    echo -e "  ✅ Local coordinator config written"
}

# Create API coordinator config (Spore 2)
create_api_coordinator_config() {
    local spore_path="$1"
    
    echo -e "${YELLOW}Creating API AI Coordinator config...${NC}"
    
    cat > "$spore_path/config/ai_coordinator_api.toml" << 'EOF'
# API AI Coordinator Configuration
# Spore 2: Focuses on external API routing (Anthropic, OpenAI, HuggingFace)

[coordinator]
role = "api"
priority_providers = ["anthropic", "openai", "huggingface"]
fallback_to_local = true

[anthropic]
enabled = true
# API key loaded from environment
model_default = "claude-3-haiku-20240307"

[openai]
enabled = true
model_default = "gpt-4o-mini"

[huggingface]
enabled = true
model_default = "distilbert-base-uncased-finetuned-sst-2-english"

[routing]
# Route external API tasks here
text_generation_api = true
image_generation_api = true  # DALL-E
sentiment_analysis_api = true
# Local compute offloaded to Spore 1
text_generation_local = false
image_inference_local = false

[federation]
peer_socket = "/run/user/1000/biomeos/spore1-neural-api.sock"
sync_interval_ms = 5000
EOF

    echo -e "  ✅ API coordinator config written"
}

# Create launcher scripts
create_launcher_scripts() {
    local spore_path="$1"
    local spore_role="$2"  # "local" or "api"
    local spore_id="$3"    # "spore1" or "spore2"
    
    echo -e "${YELLOW}Creating launcher for $spore_role coordinator...${NC}"
    
    cat > "$spore_path/launch_ai_coordinator.sh" << EOF
#!/bin/bash
# Launch $spore_role AI Coordinator
# Auto-generated by deploy_dual_spore_ai.sh

set -e

SPORE_ROOT="\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
SOCKET_DIR="/run/user/\$(id -u)/biomeos"
FAMILY_ID=\$(cat "\$SPORE_ROOT/.family.seed" | head -c 8 | xxd -p)

# Ensure socket directory exists
mkdir -p "\$SOCKET_DIR"

# Export environment
export XDG_RUNTIME_DIR="/run/user/\$(id -u)"
export FAMILY_ID="\$FAMILY_ID"
export RUST_LOG="info"
export AI_COORDINATOR_ROLE="$spore_role"
export AI_COORDINATOR_CONFIG="\$SPORE_ROOT/config/ai_coordinator_${spore_role}.toml"

EOF

    if [ "$spore_role" = "local" ]; then
        cat >> "$spore_path/launch_ai_coordinator.sh" << 'EOF'
# Local coordinator needs Toadstool
export TOADSTOOL_SOCKET="$SOCKET_DIR/toadstool-$FAMILY_ID.jsonrpc.sock"

echo "Starting Local AI Coordinator (Toadstool-focused)..."
echo "  Family: $FAMILY_ID"
echo "  Role: local"
echo ""

# Start Tower Atomic first
"$SPORE_ROOT/scripts/bootstrap_tower_atomic.sh"

# Start Toadstool for local compute
echo "Starting Toadstool..."
"$SPORE_ROOT/plasmidBin/primals/toadstool/toadstool" server \
    --socket "$TOADSTOOL_SOCKET" \
    --family-id "$FAMILY_ID" &

echo "Local AI Coordinator ready!"
echo "  Neural API: $SOCKET_DIR/neural-api-$FAMILY_ID.sock"
echo "  Toadstool:  $TOADSTOOL_SOCKET"
EOF
    else
        cat >> "$spore_path/launch_ai_coordinator.sh" << 'EOF'
# API coordinator needs external API keys
if [ -f "$SPORE_ROOT/../testing-secrets/api-keys.env" ]; then
    source "$SPORE_ROOT/../testing-secrets/api-keys.env"
    echo "Loaded API keys from testing-secrets"
fi

# Or from environment
if [ -z "$ANTHROPIC_API_KEY" ]; then
    echo "Warning: ANTHROPIC_API_KEY not set"
fi

echo "Starting API AI Coordinator (External API-focused)..."
echo "  Family: $FAMILY_ID"
echo "  Role: api"
echo ""

# Start Tower Atomic (includes Squirrel for AI routing)
"$SPORE_ROOT/scripts/bootstrap_tower_atomic.sh"

echo "API AI Coordinator ready!"
echo "  Neural API: $SOCKET_DIR/neural-api-$FAMILY_ID.sock"
echo "  Squirrel:   $SOCKET_DIR/squirrel-$FAMILY_ID.sock"
EOF
    fi

    chmod +x "$spore_path/launch_ai_coordinator.sh"
    echo -e "  ✅ launch_ai_coordinator.sh"
}

# Create cross-spore test script
create_test_script() {
    echo -e "${YELLOW}Creating cross-spore test script...${NC}"
    
    cat > "$BIOMEOS_ROOT/scripts/test_dual_spore_ai.sh" << 'EOF'
#!/bin/bash
# Test Dual Spore AI Coordination
# Validates that both spores can work together

SPORE1_SOCKET="/run/user/$(id -u)/biomeos/spore1-neural-api.sock"
SPORE2_SOCKET="/run/user/$(id -u)/biomeos/spore2-neural-api.sock"

echo "═══════════════════════════════════════════════════════════"
echo "     Dual Spore AI Coordination Test                        "
echo "═══════════════════════════════════════════════════════════"
echo ""

# Test 1: Local compute via Spore 1
echo "Test 1: Local compute health (Spore 1 → Toadstool)"
echo '{"jsonrpc":"2.0","method":"capability.call","params":{"capability":"compute","operation":"health"},"id":1}' | \
    timeout 5 nc -U "$SPORE1_SOCKET" 2>/dev/null && echo "✅" || echo "❌"

# Test 2: External API via Spore 2
echo ""
echo "Test 2: External API health (Spore 2 → Squirrel)"
echo '{"jsonrpc":"2.0","method":"capability.call","params":{"capability":"ai","operation":"health"},"id":2}' | \
    timeout 5 nc -U "$SPORE2_SOCKET" 2>/dev/null && echo "✅" || echo "❌"

# Test 3: Cross-spore coordination
echo ""
echo "Test 3: Cross-spore AI query"
echo "  Sending text to Spore 2 (API) for analysis..."
RESULT=$(echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"What is 2+2?","model":"claude-3-haiku-20240307","max_tokens":50},"id":3}' | \
    timeout 30 nc -U "$SPORE2_SOCKET" 2>/dev/null)
if echo "$RESULT" | grep -q "result"; then
    echo "  ✅ API response received"
    echo "$RESULT" | jq -r '.result.response' 2>/dev/null | head -c 100
else
    echo "  ❌ No response"
fi

# Test 4: Federation trust
echo ""
echo "Test 4: Federation trust verification"
SPORE1_FAMILY=$(cat /media/$USER/*/biomeOS/.family.seed 2>/dev/null | head -1 | head -c 8 | xxd -p)
echo '{"jsonrpc":"2.0","method":"federation.verify_family_member","params":{"family_id":"'$SPORE1_FAMILY'"},"id":4}' | \
    timeout 5 nc -U "$SPORE2_SOCKET" 2>/dev/null && echo "✅ Federation verified" || echo "❌ Federation check failed"

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "     Test Complete                                          "
echo "═══════════════════════════════════════════════════════════"
EOF

    chmod +x "$BIOMEOS_ROOT/scripts/test_dual_spore_ai.sh"
    echo -e "  ✅ test_dual_spore_ai.sh"
}

# Main deployment
main() {
    # Detect spores if not provided
    if [ -z "$SPORE1_PATH" ] || [ -z "$SPORE2_PATH" ]; then
        detect_usb_spores
    fi
    
    validate_spores
    
    echo ""
    echo -e "${BLUE}Deployment Plan:${NC}"
    echo -e "  Spore 1 (Local AI):  ${GREEN}$SPORE1_PATH${NC}"
    echo -e "  Spore 2 (API AI):    ${GREEN}$SPORE2_PATH${NC}"
    echo ""
    
    # Update binaries
    update_spore_binaries "$SPORE1_PATH" "Spore 1"
    update_spore_binaries "$SPORE2_PATH" "Spore 2"
    
    # Create coordinator configs
    mkdir -p "$SPORE1_PATH/config"
    mkdir -p "$SPORE2_PATH/config"
    
    create_local_coordinator_config "$SPORE1_PATH"
    create_api_coordinator_config "$SPORE2_PATH"
    
    # Create launcher scripts
    create_launcher_scripts "$SPORE1_PATH" "local" "spore1"
    create_launcher_scripts "$SPORE2_PATH" "api" "spore2"
    
    # Create test script
    create_test_script
    
    echo ""
    echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}     Deployment Complete!                                   ${NC}"
    echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "To start the coordinators:"
    echo ""
    echo "  # On Spore 1 (Local AI Coordinator):"
    echo "  cd $SPORE1_PATH && ./launch_ai_coordinator.sh"
    echo ""
    echo "  # On Spore 2 (API AI Coordinator):"
    echo "  cd $SPORE2_PATH && ./launch_ai_coordinator.sh"
    echo ""
    echo "To test cross-spore coordination:"
    echo "  ./scripts/test_dual_spore_ai.sh"
    echo ""
}

main "$@"

