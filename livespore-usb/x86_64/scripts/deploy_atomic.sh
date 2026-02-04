#!/bin/bash
# LiveSpore USB - Unified Atomic Deployment
# Architecture: x86_64
# Standard: PRIMAL_DEPLOYMENT_STANDARD v1.0
#
# EVOLUTION STATUS:
# - Phase 1 (Current): Shell scripts as scaffolding
# - Phase 2 (Target): Graph-based deployment via Neural API
# - Phase 3 (Future): Living graphs with adaptive optimization
#
# Usage:
#   ./deploy_atomic.sh tower     # Deploy Tower Atomic (BearDog + Songbird)
#   ./deploy_atomic.sh node      # Deploy Node Atomic (Tower + Toadstool)
#   ./deploy_atomic.sh nest      # Deploy Nest Atomic (Node + NestGate)
#   ./deploy_atomic.sh nucleus   # Deploy Complete NUCLEUS
#
# With graph execution (Phase 2):
#   ./deploy_atomic.sh --graph tower    # Use graph deployment
#   ./deploy_atomic.sh --graph nucleus  # Full graph-based NUCLEUS

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GRAPHS_DIR="$SCRIPT_DIR/../graphs"
PRIMAL_DIR="$SCRIPT_DIR/../primals"
ARCH="$(uname -m)"

# Environment
export FAMILY_ID="${FAMILY_ID:-livespore}"
export NODE_ID="${NODE_ID:-$(hostname)}"
export RUST_LOG="${RUST_LOG:-info}"

# Socket directory (5-tier resolution per PRIMAL_DEPLOYMENT_STANDARD)
resolve_socket_dir() {
    if [ -n "$BIOMEOS_SOCKET_DIR" ]; then
        echo "$BIOMEOS_SOCKET_DIR"
    elif [ -n "$XDG_RUNTIME_DIR" ]; then
        echo "$XDG_RUNTIME_DIR/biomeos"
    elif [ -d "/run/user/$(id -u)" ]; then
        echo "/run/user/$(id -u)/biomeos"
    elif [ -d "/data/local/tmp" ]; then
        echo "/data/local/tmp/biomeos"  # Android
    else
        echo "/tmp/biomeos"
    fi
}

SOCKET_DIR="$(resolve_socket_dir)"
mkdir -p "$SOCKET_DIR"

# Parse arguments
USE_GRAPH=false
ATOMIC_TYPE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --graph|-g)
            USE_GRAPH=true
            shift
            ;;
        tower|node|nest|nucleus)
            ATOMIC_TYPE="$1"
            shift
            ;;
        *)
            echo "Unknown argument: $1"
            exit 1
            ;;
    esac
done

if [ -z "$ATOMIC_TYPE" ]; then
    echo "Usage: $0 [--graph] <tower|node|nest|nucleus>"
    exit 1
fi

echo "═══════════════════════════════════════════════════════════════"
echo "🧬 biomeOS Atomic Deployment"
echo "═══════════════════════════════════════════════════════════════"
echo "Architecture: $ARCH"
echo "Atomic Type:  $ATOMIC_TYPE"
echo "Family ID:    $FAMILY_ID"
echo "Node ID:      $NODE_ID"
echo "Socket Dir:   $SOCKET_DIR"
echo "Graph Mode:   $USE_GRAPH"
echo ""

# Graph-based deployment (Phase 2)
deploy_via_graph() {
    local graph_id="$1"
    local graph_file="$GRAPHS_DIR/${graph_id}.toml"
    
    if [ ! -f "$graph_file" ]; then
        echo "❌ Graph not found: $graph_file"
        exit 1
    fi
    
    echo "📊 Deploying via graph: $graph_id"
    
    # Check if Neural API is running
    NEURAL_SOCKET="$SOCKET_DIR/neural-api-$FAMILY_ID.sock"
    
    if [ ! -e "$NEURAL_SOCKET" ]; then
        echo "⚠️  Neural API not running, starting..."
        # Start Neural API first
        if [ -x "$PRIMAL_DIR/biomeos" ]; then
            nohup "$PRIMAL_DIR/biomeos" neural-api \
                --socket "$NEURAL_SOCKET" \
                --graphs-dir "$GRAPHS_DIR" > /tmp/neural-api.log 2>&1 &
            sleep 3
        else
            echo "❌ biomeOS binary not found at $PRIMAL_DIR/biomeos"
            echo "   Falling back to shell deployment..."
            return 1
        fi
    fi
    
    # Execute graph via Neural API
    echo "Executing graph.execute..."
    RESULT=$(echo "{\"jsonrpc\":\"2.0\",\"method\":\"graph.execute\",\"params\":{\"graph_id\":\"$graph_id\",\"family_id\":\"$FAMILY_ID\"},\"id\":1}" | \
        nc -U "$NEURAL_SOCKET" -w 30)
    
    if echo "$RESULT" | grep -q '"error"'; then
        echo "❌ Graph execution failed:"
        echo "$RESULT" | python3 -m json.tool 2>/dev/null || echo "$RESULT"
        return 1
    fi
    
    echo "✅ Graph execution complete"
    echo "$RESULT" | python3 -m json.tool 2>/dev/null || echo "$RESULT"
    return 0
}

# Shell-based deployment (Phase 1 - Scaffolding)
deploy_via_script() {
    case "$ATOMIC_TYPE" in
        tower)
            echo "🚀 Starting Tower Atomic (Phase 1 - Shell)"
            "$SCRIPT_DIR/start_tower.sh"
            ;;
        node)
            echo "🚀 Starting Node Atomic (Phase 1 - Shell)"
            "$SCRIPT_DIR/start_node.sh"
            ;;
        nest)
            echo "🚀 Starting Nest Atomic (Phase 1 - Shell)"
            "$SCRIPT_DIR/start_nest.sh"
            ;;
        nucleus)
            echo "🚀 Starting Complete NUCLEUS (Phase 1 - Shell)"
            # Start all atomics in order
            "$SCRIPT_DIR/start_tower.sh"
            sleep 2
            "$SCRIPT_DIR/start_node.sh"
            sleep 2
            "$SCRIPT_DIR/start_nest.sh"
            ;;
    esac
}

# Map atomic type to graph ID
get_graph_id() {
    case "$1" in
        tower)   echo "tower_atomic_bootstrap" ;;
        node)    echo "node_atomic_compute" ;;
        nest)    echo "nest_deploy" ;;
        nucleus) echo "nucleus_complete" ;;
    esac
}

# Main deployment logic
if [ "$USE_GRAPH" = true ]; then
    GRAPH_ID=$(get_graph_id "$ATOMIC_TYPE")
    if deploy_via_graph "$GRAPH_ID"; then
        echo ""
        echo "✅ Graph-based deployment complete!"
    else
        echo ""
        echo "⚠️  Graph deployment failed, falling back to shell..."
        deploy_via_script
    fi
else
    deploy_via_script
fi

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "🎊 Deployment Complete!"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Active sockets:"
ls -lh "$SOCKET_DIR"/*.sock 2>/dev/null || echo "(none)"
echo ""
echo "Next steps:"
echo "  - Health check: echo '{\"jsonrpc\":\"2.0\",\"method\":\"health\",\"id\":1}' | nc -U <socket>"
echo "  - Graph deploy: $0 --graph $ATOMIC_TYPE"
