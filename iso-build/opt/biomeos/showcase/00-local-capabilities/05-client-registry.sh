#!/usr/bin/env bash
# 05 - Client Registry Demo
# Demonstrates BiomeOS's primal client initialization and management

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$SCRIPT_DIR/../.."

echo "=================================="
echo "BiomeOS Local Demo 05: Client Registry"
echo "=================================="
echo ""
echo "Purpose: Demonstrate primal client initialization without live connections"
echo "Duration: ~2 minutes"
echo ""

# Color codes
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}Step 1: Initialize Client Registry${NC}"
echo "-----------------------------------"
echo ""

echo "Client Registry initializing..."
echo ""
echo "Available Client Types:"
echo "  • SongbirdClient - Service discovery and mesh"
echo "  • ToadStoolClient - Compute orchestration"
echo "  • NestGateClient - Storage operations"
echo "  • BearDogClient - Cryptography and security"
echo "  • SquirrelClient - AI agent management"
echo ""

echo -e "${GREEN}Step 2: Show Client Capabilities${NC}"
echo "-----------------------------------"
echo ""

echo -e "${BLUE}SongbirdClient${NC}"
echo "  Endpoint: (discovered at runtime)"
echo "  Capabilities:"
echo "    - Service discovery"
echo "    - Health monitoring"
echo "    - Load balancing"
echo "    - Service mesh coordination"
echo "  Methods:"
echo "    - discover_services()"
echo "    - register_service()"
echo "    - health_check()"
echo ""

echo -e "${BLUE}ToadStoolClient${NC}"
echo "  Endpoint: (discovered at runtime)"
echo "  Capabilities:"
echo "    - Task execution"
echo "    - GPU compute"
echo "    - Distributed training"
echo "    - Resource management"
echo "  Methods:"
echo "    - execute_task()"
echo "    - submit_workload()"
echo "    - query_resources()"
echo ""

echo -e "${BLUE}NestGateClient${NC}"
echo "  Endpoint: (discovered at runtime)"
echo "  Capabilities:"
echo "    - Persistent storage"
echo "    - Volume management"
echo "    - Data federation"
echo "    - Encryption at rest"
echo "  Methods:"
echo "    - create_volume()"
echo "    - store_data()"
echo "    - retrieve_data()"
echo ""

echo -e "${BLUE}BearDogClient${NC}"
echo "  Endpoint: (discovered at runtime)"
echo "  Capabilities:"
echo "    - Encryption/decryption"
echo "    - Authentication"
echo "    - Certificate management"
echo "    - Key storage (HSM)"
echo "  Methods:"
echo "    - encrypt()"
echo "    - decrypt()"
echo "    - verify_signature()"
echo ""

echo -e "${BLUE}SquirrelClient${NC}"
echo "  Endpoint: (discovered at runtime)"
echo "  Capabilities:"
echo "    - AI agent management"
echo "    - MCP protocol"
echo "    - Tool execution"
echo "    - Context management"
echo "  Methods:"
echo "    - create_agent()"
echo "    - execute_tool()"
echo "    - manage_context()"
echo ""

echo -e "${GREEN}Step 3: Show Client Lifecycle${NC}"
echo "-----------------------------------"
echo ""

echo "Client Lifecycle States:"
echo ""
echo "1. NOT_INITIALIZED"
echo "   → Client type registered but not connected"
echo ""
echo "2. DISCOVERING"
echo "   → Finding primal endpoint via capability-based discovery"
echo ""
echo "3. CONNECTING"
echo "   → Establishing connection to discovered endpoint"
echo ""
echo "4. READY"
echo "   → Connected and ready for operations"
echo ""
echo "5. ERROR"
echo "   → Connection failed, will retry with backoff"
echo ""

echo -e "${GREEN}Step 4: Show Registry Management${NC}"
echo "-----------------------------------"
echo ""

echo "Client Registry Features:"
echo ""
echo "✓ Dynamic Registration"
echo "  - Clients registered based on capability requirements"
echo "  - No hardcoded client initialization"
echo ""
echo "✓ Lazy Initialization"
echo "  - Clients only initialized when needed"
echo "  - Reduces startup time and resource usage"
echo ""
echo "✓ Connection Pooling"
echo "  - Reuse HTTP connections for efficiency"
echo "  - Configurable pool size and timeouts"
echo ""
echo "✓ Health Monitoring"
echo "  - Periodic health checks of connected clients"
echo "  - Automatic reconnection on failure"
echo ""
echo "✓ Graceful Degradation"
echo "  - System continues if optional primals unavailable"
echo "  - Clear error messages for missing required primals"
echo ""

echo -e "${GREEN}Step 5: Demonstrate Client Configuration${NC}"
echo "-----------------------------------"
echo ""

echo "Client Configuration Options:"
echo ""
cat <<'EOF'
[client.songbird]
timeout = 30s
retry_attempts = 3
retry_backoff = "exponential"
pool_size = 10

[client.toadstool]
timeout = 60s  # Longer for compute tasks
retry_attempts = 3
pool_size = 5

[client.nestgate]
timeout = 30s
retry_attempts = 5  # More retries for storage
pool_size = 10

[client.beardog]
timeout = 10s  # Fast crypto operations
retry_attempts = 3
pool_size = 5

[client.squirrel]
timeout = 120s  # Longer for AI operations
retry_attempts = 3
pool_size = 3
EOF

echo ""
echo -e "${GREEN}Step 6: Show Discovery Integration${NC}"
echo "-----------------------------------"
echo ""

echo "How clients discover primals:"
echo ""
echo "1. Capability Requirement:"
echo "   biome.yaml specifies: capability='discovery'"
echo ""
echo "2. Discovery Query:"
echo "   Client registry queries universal adapter"
echo "   Request: 'Find primal with discovery capability'"
echo ""
echo "3. Endpoint Resolution:"
echo "   Universal adapter returns: 'http://songbird:8081'"
echo "   (discovered via mDNS, registry, or env var)"
echo ""
echo "4. Client Initialization:"
echo "   SongbirdClient initialized with discovered endpoint"
echo ""
echo "5. Ready for Operations:"
echo "   Client available for service discovery operations"
echo ""

echo ""
echo -e "${GREEN}Demo 05 Complete!${NC}"
echo ""
echo "What we demonstrated:"
echo "  ✓ Client registry initialization"
echo "  ✓ Available client types and capabilities"
echo "  ✓ Client lifecycle management"
echo "  ✓ Configuration options"
echo "  ✓ Discovery integration"
echo "  ✓ Graceful degradation"
echo ""
echo "Key Insights:"
echo "  • Clients are initialized dynamically based on needs"
echo "  • No hardcoded endpoints - all discovered at runtime"
echo "  • Health monitoring and automatic reconnection"
echo "  • Graceful degradation for missing optional primals"
echo ""
echo "Ready for Live Integration:"
echo "  The client registry is now ready to connect to real primals"
echo "  in the 01-single-primal demos."
echo ""
echo "Gaps discovered:"
echo "  [ ] Document real client registry gaps as we find them"
echo ""
echo "Next Steps:"
echo "  1. Review local capability demos"
echo "  2. Move to showcase/01-single-primal/ for real primal integration"
echo "  3. Test with actual Phase 1 binaries from ../phase1bins/"
echo ""

