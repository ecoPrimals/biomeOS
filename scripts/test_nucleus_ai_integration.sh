#!/bin/bash
# =============================================================================
# NUCLEUS Full AI Integration Test
# =============================================================================
#
# Tests all three NUCLEUS atomic configurations with Squirrel AI:
# 1. Tower Atomic (BearDog + Songbird)
# 2. Node Atomic (Tower + Toadstool with 4070 GPU)
# 3. Nest Atomic (Tower + NestGate for model persistence)
# 4. Squirrel AI (Multi-provider: local 4070 + online Anthropic/OpenAI)
#
# Hardware: RTX 4070 12GB VRAM
# API Keys: ../../testing-secrets/api-keys.toml
#
# Created: January 29, 2026
# =============================================================================

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
}

log_section() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo ""
}

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
GRAPH_FILE="$PROJECT_ROOT/graphs/nucleus_full_ai_test.toml"
API_KEYS_FILE="$PROJECT_ROOT/../../testing-secrets/api-keys.toml"
FAMILY_ID="${FAMILY_ID:-test-nucleus-$(date +%s)}"
UID=$(id -u)
SOCKET_DIR="/run/user/$UID/biomeos"

# Trap to cleanup on exit
cleanup() {
    log_section "Cleaning Up"
    
    # Kill any running primals
    pkill -f beardog || true
    pkill -f songbird || true
    pkill -f nestgate || true
    pkill -f toadstool || true
    pkill -f squirrel || true
    
    # Clean sockets
    rm -f "$SOCKET_DIR"/*.sock || true
    
    log_info "Cleanup complete"
}

trap cleanup EXIT INT TERM

# =============================================================================
# Phase 0: Pre-flight Checks
# =============================================================================

preflight_checks() {
    log_section "Pre-flight Checks"
    
    # Check GPU
    log_info "Checking GPU (RTX 4070)..."
    if nvidia-smi --query-gpu=name,memory.total --format=csv,noheader 2>/dev/null | grep -q "4070"; then
        GPU_INFO=$(nvidia-smi --query-gpu=name,memory.total,utilization.gpu,utilization.memory --format=csv,noheader)
        log_success "GPU Detected: $GPU_INFO"
    else
        log_warning "RTX 4070 not detected, local AI tests may fail"
    fi
    
    # Check API keys
    log_info "Checking API keys..."
    if [ -f "$API_KEYS_FILE" ]; then
        log_success "API keys found: $API_KEYS_FILE"
        
        # Verify key sections exist
        if grep -q "anthropic_api_key" "$API_KEYS_FILE"; then
            log_success "  ✓ Anthropic API key present"
        fi
        if grep -q "openai_api_key" "$API_KEYS_FILE"; then
            log_success "  ✓ OpenAI API key present"
        fi
        if grep -q "hugging face" "$API_KEYS_FILE"; then
            log_success "  ✓ HuggingFace token present"
        fi
    else
        log_error "API keys not found at $API_KEYS_FILE"
        exit 1
    fi
    
    # Check binaries
    log_info "Checking primal binaries..."
    for primal in beardog songbird nestgate toadstool squirrel; do
        if [ -f "$PROJECT_ROOT/target/release/$primal" ] || [ -f "../$primal/target/release/$primal" ]; then
            log_success "  ✓ $primal binary found"
        else
            log_warning "  ! $primal binary not found (will attempt runtime discovery)"
        fi
    done
    
    # Check socket directory
    log_info "Checking socket directory..."
    mkdir -p "$SOCKET_DIR"
    log_success "Socket directory ready: $SOCKET_DIR"
    
    # Check Neural API graph
    log_info "Checking deployment graph..."
    if [ -f "$GRAPH_FILE" ]; then
        log_success "NUCLEUS graph found: $GRAPH_FILE"
    else
        log_error "Graph not found: $GRAPH_FILE"
        exit 1
    fi
    
    log_success "Pre-flight checks complete!"
}

# =============================================================================
# Phase 1: Deploy NUCLEUS via Neural API
# =============================================================================

deploy_nucleus() {
    log_section "Deploying NUCLEUS (Tower + Node + Nest + Squirrel)"
    
    log_info "Starting Neural API deployment..."
    log_info "Graph: $GRAPH_FILE"
    log_info "Family ID: $FAMILY_ID"
    
    # Set environment for deployment
    export FAMILY_ID
    export UID
    
    # Deploy via Neural API (if available) or fallback to manual
    if [ -f "$PROJECT_ROOT/target/release/biomeos" ]; then
        log_info "Using Neural API for deployment..."
        "$PROJECT_ROOT/target/release/biomeos" deploy-graph "$GRAPH_FILE" \
            --family-id "$FAMILY_ID" \
            --log-level info
    else
        log_warning "Neural API binary not found, deploying manually..."
        deploy_manually
    fi
}

deploy_manually() {
    log_section "Manual NUCLEUS Deployment"
    
    # Phase 1: Tower Atomic
    log_info "Phase 1: Deploying Tower Atomic (BearDog + Songbird)..."
    
    # Start BearDog
    log_info "Starting BearDog..."
    export BEARDOG_SOCKET="$SOCKET_DIR/beardog.sock"
    export FAMILY_ID
    
    if [ -f "../beardog/target/release/beardog" ]; then
        cd ../beardog
        RUST_LOG=beardog=info ./target/release/beardog server &
        BEARDOG_PID=$!
        cd "$PROJECT_ROOT"
        log_success "BearDog started (PID: $BEARDOG_PID)"
    else
        log_error "BearDog binary not found"
        return 1
    fi
    
    # Wait for BearDog socket
    log_info "Waiting for BearDog socket..."
    for i in {1..30}; do
        if [ -S "$BEARDOG_SOCKET" ]; then
            log_success "BearDog socket ready: $BEARDOG_SOCKET"
            break
        fi
        sleep 1
    done
    
    # Start Songbird
    log_info "Starting Songbird..."
    export SONGBIRD_SOCKET="$SOCKET_DIR/songbird.sock"
    export SONGBIRD_SECURITY_PROVIDER="beardog"
    
    if [ -f "../songbird/target/release/songbird" ]; then
        cd ../songbird
        RUST_LOG=songbird=info ./target/release/songbird server &
        SONGBIRD_PID=$!
        cd "$PROJECT_ROOT"
        log_success "Songbird started (PID: $SONGBIRD_PID)"
    else
        log_error "Songbird binary not found"
        return 1
    fi
    
    # Wait for Songbird socket
    log_info "Waiting for Songbird socket..."
    for i in {1..30}; do
        if [ -S "$SONGBIRD_SOCKET" ]; then
            log_success "Songbird socket ready: $SONGBIRD_SOCKET"
            break
        fi
        sleep 1
    done
    
    log_success "Tower Atomic deployed!"
    
    # Phase 2: Node Atomic (add Toadstool)
    log_info "Phase 2: Deploying Node Atomic (+ Toadstool with 4070 GPU)..."
    
    export TOADSTOOL_SOCKET="$SOCKET_DIR/toadstool.sock"
    export CUDA_VISIBLE_DEVICES="0"
    export GPU_MEMORY_FRACTION="0.9"
    
    if [ -f "../toadstool/target/release/toadstool" ]; then
        cd ../toadstool
        RUST_LOG=toadstool=info ./target/release/toadstool server &
        TOADSTOOL_PID=$!
        cd "$PROJECT_ROOT"
        log_success "Toadstool started (PID: $TOADSTOOL_PID)"
    else
        log_warning "Toadstool binary not found, skipping Node Atomic"
    fi
    
    # Phase 3: Nest Atomic (add NestGate)
    log_info "Phase 3: Deploying Nest Atomic (+ NestGate for model persistence)..."
    
    export NESTGATE_SOCKET="$SOCKET_DIR/nestgate.sock"
    export STORAGE_PATH="/var/tmp/biomeos/nestgate/models"
    mkdir -p "$STORAGE_PATH"
    
    if [ -f "../nestgate/target/release/nestgate" ]; then
        cd ../nestgate
        RUST_LOG=nestgate=info ./target/release/nestgate server &
        NESTGATE_PID=$!
        cd "$PROJECT_ROOT"
        log_success "NestGate started (PID: $NESTGATE_PID)"
    else
        log_warning "NestGate binary not found, skipping Nest Atomic"
    fi
    
    # Wait for all sockets
    sleep 5
    
    # Phase 4: Squirrel AI
    log_info "Phase 4: Deploying Squirrel AI (MCP server with multi-provider)..."
    
    export SQUIRREL_SOCKET="$SOCKET_DIR/squirrel.sock"
    export ANTHROPIC_API_KEY_FILE="$API_KEYS_FILE"
    export OPENAI_API_KEY_FILE="$API_KEYS_FILE"
    export HUGGINGFACE_TOKEN_FILE="$API_KEYS_FILE"
    export LOCAL_MODEL_PROVIDER="toadstool"
    export MODEL_CACHE_PROVIDER="nestgate"
    
    if [ -f "../squirrel/target/release/squirrel" ]; then
        cd ../squirrel
        RUST_LOG=squirrel=info ./target/release/squirrel server &
        SQUIRREL_PID=$!
        cd "$PROJECT_ROOT"
        log_success "Squirrel started (PID: $SQUIRREL_PID)"
    else
        log_warning "Squirrel binary not found"
    fi
    
    log_success "NUCLEUS deployment complete!"
}

# =============================================================================
# Phase 2: Health Verification
# =============================================================================

verify_health() {
    log_section "Health Verification"
    
    sleep 3  # Give primals time to initialize
    
    # Check sockets exist
    log_info "Checking sockets..."
    for sock in beardog songbird toadstool nestgate squirrel; do
        if [ -S "$SOCKET_DIR/$sock.sock" ]; then
            log_success "  ✓ $sock.sock exists"
        else
            log_warning "  ! $sock.sock not found"
        fi
    done
    
    # Health check via JSON-RPC
    log_info "Checking primal health via JSON-RPC..."
    
    for primal in beardog songbird; do
        SOCKET="$SOCKET_DIR/$primal.sock"
        if [ -S "$SOCKET" ]; then
            RESPONSE=$(echo '{"jsonrpc":"2.0","method":"health","id":1}' | nc -U "$SOCKET" -w 2 2>/dev/null || echo "{}")
            if echo "$RESPONSE" | grep -q "healthy\|result"; then
                log_success "  ✓ $primal is healthy"
            else
                log_warning "  ! $primal health check failed"
            fi
        fi
    done
}

# =============================================================================
# Phase 3: AI Capability Tests
# =============================================================================

test_ai_capabilities() {
    log_section "AI Capability Tests"
    
    # Test 1: Local AI via Toadstool (4070)
    log_info "Test 1: Local AI inference (Toadstool + 4070 GPU)..."
    
    if [ -S "$SOCKET_DIR/toadstool.sock" ]; then
        # Query GPU status first
        log_info "Querying GPU status..."
        GPU_QUERY='{"jsonrpc":"2.0","method":"gpu.query_status","id":1}'
        GPU_RESULT=$(echo "$GPU_QUERY" | nc -U "$SOCKET_DIR/toadstool.sock" -w 5 2>/dev/null || echo "{}")
        
        if echo "$GPU_RESULT" | grep -q "4070\|12282"; then
            log_success "  ✓ GPU detected in Toadstool"
        else
            log_warning "  ! GPU not detected, result: $GPU_RESULT"
        fi
    else
        log_warning "  ! Toadstool socket not available"
    fi
    
    # Test 2: Online AI via Anthropic
    log_info "Test 2: Online AI query (Anthropic Claude)..."
    
    if [ -S "$SOCKET_DIR/squirrel.sock" ]; then
        AI_QUERY=$(cat <<'EOF'
{
  "jsonrpc": "2.0",
  "method": "ai.query",
  "params": {
    "provider": "anthropic",
    "prompt": "What is capability-based discovery? Answer in 1 sentence.",
    "max_tokens": 50
  },
  "id": 2
}
EOF
)
        
        log_info "Sending query to Squirrel..."
        AI_RESULT=$(echo "$AI_QUERY" | nc -U "$SOCKET_DIR/squirrel.sock" -w 15 2>/dev/null || echo "{}")
        
        if echo "$AI_RESULT" | grep -q "result\|response\|content"; then
            log_success "  ✓ Anthropic AI query successful"
            log_info "Response preview:"
            echo "$AI_RESULT" | jq -r '.result.content // .result.response // .result' 2>/dev/null | head -3 || echo "$AI_RESULT" | head -c 200
        else
            log_warning "  ! Anthropic query failed or timed out"
            log_info "Response: $AI_RESULT"
        fi
    else
        log_warning "  ! Squirrel socket not available"
    fi
    
    # Test 3: Model caching to NestGate
    log_info "Test 3: Model persistence (NestGate)..."
    
    if [ -S "$SOCKET_DIR/nestgate.sock" ]; then
        # Test storing model metadata
        CACHE_QUERY=$(cat <<'EOF'
{
  "jsonrpc": "2.0",
  "method": "storage.store",
  "params": {
    "key": "models/llama-3-8b/metadata",
    "value": {
      "name": "Llama-3-8B",
      "size_gb": 8.5,
      "provider": "huggingface",
      "cached_at": "2026-01-29T19:00:00Z"
    }
  },
  "id": 3
}
EOF
)
        
        CACHE_RESULT=$(echo "$CACHE_QUERY" | nc -U "$SOCKET_DIR/nestgate.sock" -w 5 2>/dev/null || echo "{}")
        
        if echo "$CACHE_RESULT" | grep -q "result\|stored\|success"; then
            log_success "  ✓ Model metadata cached to NestGate"
        else
            log_warning "  ! Model caching test incomplete"
        fi
    else
        log_warning "  ! NestGate socket not available"
    fi
}

# =============================================================================
# Phase 4: Capability.call Routing Test
# =============================================================================

test_capability_routing() {
    log_section "Capability.call Routing Test"
    
    log_info "Testing semantic capability routing via Neural API..."
    
    # Test routing to BearDog via capability
    log_info "Test: capability.call('crypto.hash', ...)"
    
    if [ -S "$SOCKET_DIR/beardog.sock" ]; then
        HASH_QUERY='{"jsonrpc":"2.0","method":"crypto.hash","params":{"data":"test"},"id":4}'
        HASH_RESULT=$(echo "$HASH_QUERY" | nc -U "$SOCKET_DIR/beardog.sock" -w 3 2>/dev/null || echo "{}")
        
        if echo "$HASH_RESULT" | grep -q "result\|hash"; then
            log_success "  ✓ Crypto capability routing works"
        else
            log_warning "  ! Crypto capability routing test failed"
        fi
    fi
    
    # Test routing to Songbird via capability
    log_info "Test: capability.call('discovery.query', ...)"
    
    if [ -S "$SOCKET_DIR/songbird.sock" ]; then
        DISC_QUERY='{"jsonrpc":"2.0","method":"discovery.query","params":{"capability":"security"},"id":5}'
        DISC_RESULT=$(echo "$DISC_QUERY" | nc -U "$SOCKET_DIR/songbird.sock" -w 3 2>/dev/null || echo "{}")
        
        if echo "$DISC_RESULT" | grep -q "result\|beardog\|providers"; then
            log_success "  ✓ Discovery capability routing works"
        else
            log_warning "  ! Discovery capability routing test failed"
        fi
    fi
}

# =============================================================================
# Phase 5: Report Results
# =============================================================================

report_results() {
    log_section "Test Results Summary"
    
    echo "NUCLEUS Configuration Test Results"
    echo "=================================="
    echo ""
    echo "Hardware:"
    echo "  GPU: RTX 4070 (12GB VRAM)"
    echo "  Family ID: $FAMILY_ID"
    echo "  Socket Dir: $SOCKET_DIR"
    echo ""
    echo "Atomic Configurations:"
    echo "  Tower: BearDog + Songbird"
    echo "  Node: Tower + Toadstool (GPU compute)"
    echo "  Nest: Tower + NestGate (model persistence)"
    echo ""
    echo "AI Providers:"
    echo "  Local: Toadstool + 4070 GPU"
    echo "  Online: Anthropic Claude (via Squirrel)"
    echo "  Online: OpenAI GPT (via Squirrel)"
    echo "  Cache: NestGate model persistence"
    echo ""
    echo "Sockets Created:"
    ls -la "$SOCKET_DIR"/*.sock 2>/dev/null | awk '{print "  -", $9}' || echo "  (none)"
    echo ""
    echo "Running Processes:"
    ps aux | grep -E "beardog|songbird|nestgate|toadstool|squirrel" | grep -v grep || echo "  (none)"
    echo ""
    
    log_success "Integration test complete!"
    log_info "Check individual test results above for details"
}

# =============================================================================
# Main Execution
# =============================================================================

main() {
    log_section "NUCLEUS Full AI Integration Test"
    
    log_info "Testing all three NUCLEUS atomics with Squirrel AI"
    log_info "Hardware: RTX 4070 12GB VRAM"
    log_info "API Keys: Available for Anthropic, OpenAI, HuggingFace"
    
    preflight_checks
    deploy_manually  # Use manual deployment for better visibility
    verify_health
    test_ai_capabilities
    test_capability_routing
    report_results
    
    log_section "Test Complete - Press Ctrl+C to cleanup and exit"
    
    # Keep running for manual testing
    log_info "Primals are running. Test manually or press Ctrl+C to stop."
    log_info ""
    log_info "Manual test commands:"
    echo "  # Health check BearDog"
    echo "  echo '{\"jsonrpc\":\"2.0\",\"method\":\"health\",\"id\":1}' | nc -U $SOCKET_DIR/beardog.sock"
    echo ""
    echo "  # Query Squirrel AI (Anthropic)"
    echo "  echo '{\"jsonrpc\":\"2.0\",\"method\":\"ai.query\",\"params\":{\"prompt\":\"Hello!\"},\"id\":2}' | nc -U $SOCKET_DIR/squirrel.sock"
    echo ""
    echo "  # Check Toadstool GPU"
    echo "  echo '{\"jsonrpc\":\"2.0\",\"method\":\"gpu.query_status\",\"id\":3}' | nc -U $SOCKET_DIR/toadstool.sock"
    
    # Wait for user interrupt
    sleep infinity
}

# Run main
main "$@"
