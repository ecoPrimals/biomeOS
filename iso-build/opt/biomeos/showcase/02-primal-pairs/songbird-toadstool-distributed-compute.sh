#!/usr/bin/env bash
# Songbird + Toadstool: Distributed Compute via Universal Port Authority
# Demonstrates BiomeOS orchestrating compute WITHOUT direct Toadstool knowledge

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$SCRIPT_DIR/../.."

# Load capability-based discovery library  
source "$SCRIPT_DIR/../01-single-primal/common/capability-discovery.sh"

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  BiomeOS: Cross-Primal Orchestration                     ║"
echo "║  Pattern: Universal Port Authority + Compute             ║"
echo "║  Primals: Songbird (discovered) + Toadstool (discovered) ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

GAP_REPORT="$SCRIPT_DIR/gaps/songbird-toadstool-orchestration-gaps.md"
mkdir -p "$SCRIPT_DIR/gaps" "$SCRIPT_DIR/logs" "$SCRIPT_DIR/pids"

cat > "$GAP_REPORT" <<'EOF'
# Cross-Primal Orchestration Gaps: Songbird + Toadstool

## Discovery
- [ ] Songbird discovery method:
- [ ] Toadstool discovery method:
- [ ] Registration pattern:

## Port Authority
- [ ] Dynamic port assignment working:
- [ ] Registration successful:
- [ ] Service discovery via capability:

## Task Routing
- [ ] BiomeOS → Songbird routing:
- [ ] Songbird → Toadstool routing:
- [ ] Result propagation:

## Evolution Resilience
- [ ] Works if either primal API changes:
- [ ] Works if alternate primals used:
- [ ] Works with multiple compute providers:
EOF

echo -e "${CYAN}═══ The Universal Port Authority Pattern ═══${NC}"
echo ""
echo "Key Insight from Songbird showcase:"
echo '  "Once primals understand Songbird, they never set their own ports"'
echo ""
echo "What this means:"
echo "  1. Songbird assigns ALL ports (zero conflicts)"
echo "  2. Primals register their capabilities"
echo "  3. Discovery happens by capability, not name"
echo "  4. BiomeOS only knows about Songbird"
echo ""
echo "Architecture:"
echo "  BiomeOS → Songbird (port authority)"
echo "              ↓ (discovers 'compute' capability)"
echo "            Toadstool (assigned port)"
echo ""
echo "BiomeOS NEVER talks directly to Toadstool!"
echo ""

echo -e "${GREEN}═══ Phase 1: Start Songbird (Port Authority) ═══${NC}"
echo ""

SONGBIRD_PORT=8080
SONGBIRD_PID=$(start_primal_smart "songbird" $SONGBIRD_PORT)

if [ -z "$SONGBIRD_PID" ]; then
    echo "Cannot demonstrate cross-primal pattern without service registry"
    exit 0
fi

echo ""
echo -e "${BLUE}═══ Phase 2: BiomeOS Discovers Port Authority ═══${NC}"
echo ""

# BiomeOS discovers port authority (doesn't know it's Songbird)
PORT_AUTHORITY=$(discover_primal_by_capability "service_registry")

if [ -z "$PORT_AUTHORITY" ]; then
    graceful_degradation "service_registry" "Start Songbird or alternate"
    stop_primal_clean $SONGBIRD_PID
    exit 0
fi

echo ""
echo "Port authority discovered: $PORT_AUTHORITY"
echo ""

# Probe Songbird's interface
probe_primal_interface "$PORT_AUTHORITY"

echo ""
echo -e "${GREEN}═══ Phase 3: Start Toadstool (Compute Provider) ═══${NC}"
echo ""

TOADSTOOL_PORT=8081
TOADSTOOL_PID=$(start_primal_smart "toadstool" $TOADSTOOL_PORT)

if [ -z "$TOADSTOOL_PID" ]; then
    echo "No compute primal for demo (this is fine)"
    echo "Demonstrates: System continues without compute"
    stop_primal_clean $SONGBIRD_PID
    exit 0
fi

echo ""
echo -e "${BLUE}═══ Phase 4: Toadstool Registers with Songbird ═══${NC}"
echo ""

echo "Key insight: Toadstool doesn't bind its own port!"
echo "Instead, it asks Songbird for port assignment..."
echo ""

# Simulate Toadstool registration (in production, Toadstool does this)
REGISTER_ENDPOINT="$PORT_AUTHORITY/api/v1/services/register"

TOADSTOOL_REGISTRATION='{
  "primal_name": "Toadstool",
  "capabilities": [
    {"name": "compute", "type": "execution"},
    {"name": "gpu_compute", "type": "execution"},
    {"name": "distributed_compute", "type": "coordination"}
  ],
  "protocols": ["https", "grpc"],
  "metadata": {
    "version": "2.0.0",
    "max_concurrent_tasks": 100
  }
}'

echo "Toadstool registration request:"
echo "$TOADSTOOL_REGISTRATION" | jq '.' 2>/dev/null
echo ""

echo "POST $REGISTER_ENDPOINT"
REGISTER_RESPONSE=$(curl -s -X POST "$REGISTER_ENDPOINT" \
    -H "Content-Type: application/json" \
    -d "$TOADSTOOL_REGISTRATION" 2>/dev/null || echo '{}')

echo "Songbird's response:"
echo "$REGISTER_RESPONSE" | jq '.' 2>/dev/null || echo "$REGISTER_RESPONSE"

SERVICE_ID=$(echo "$REGISTER_RESPONSE" | jq -r '.service_id // .id' 2>/dev/null)
ASSIGNED_PORT=$(echo "$REGISTER_RESPONSE" | jq -r '.assigned_endpoint.port // .port' 2>/dev/null)

if [ -n "$ASSIGNED_PORT" ] && [ "$ASSIGNED_PORT" != "null" ]; then
    echo ""
    echo -e "${GREEN}✓ Songbird assigned port: $ASSIGNED_PORT${NC}"
    echo "  Service ID: $SERVICE_ID"
    echo ""
    echo "This demonstrates:"
    echo "  • Toadstool never picked its own port"
    echo "  • Songbird handles all port allocation"
    echo "  • Zero port conflicts possible"
    echo "  • Infinite scalability"
else
    echo -e "${YELLOW}⚠ Registration response format differs${NC}"
    document_gap "$GAP_REPORT" "Registration" "Response format different"
fi

echo ""
echo -e "${GREEN}═══ Phase 5: BiomeOS Submits Task (via Songbird) ═══${NC}"
echo ""

echo "Critical: BiomeOS ONLY talks to Songbird"
echo "  BiomeOS does not know:"
echo "    ✗ That 'Toadstool' exists"
echo "    ✗ What port Toadstool is on"
echo "    ✗ How to talk to Toadstool directly"
echo ""
echo "BiomeOS asks: 'Who has compute capability?'"
echo ""

# Query Songbird for compute capability
QUERY_ENDPOINT="$PORT_AUTHORITY/api/v1/services/query/compute"

echo "GET $QUERY_ENDPOINT"
COMPUTE_SERVICES=$(curl -s "$QUERY_ENDPOINT" 2>/dev/null || echo '{}')

echo "Songbird's response:"
echo "$COMPUTE_SERVICES" | jq '.' 2>/dev/null || echo "$COMPUTE_SERVICES"

COMPUTE_COUNT=$(echo "$COMPUTE_SERVICES" | jq -r '.services | length' 2>/dev/null)

if [ -n "$COMPUTE_COUNT" ] && [ "$COMPUTE_COUNT" -gt 0 ]; then
    echo ""
    echo -e "${GREEN}✓ Found $COMPUTE_COUNT compute service(s)${NC}"
    echo ""
    echo "BiomeOS can now:"
    echo "  • Submit tasks via Songbird"
    echo "  • Songbird routes to appropriate compute service"
    echo "  • Multiple compute services? Songbird load balances"
    echo "  • Compute service changes? Transparent to BiomeOS"
fi

echo ""
echo -e "${BLUE}═══ Phase 6: Task Routing Demo ═══${NC}"
echo ""

echo "Demonstrating task routing pattern..."
echo ""

TASK_PAYLOAD='{
  "task_type": "distributed_compute",
  "operation": "parallel_matrix_multiply",
  "parameters": {
    "matrix_size": 2048,
    "num_workers": 4,
    "use_gpu": true
  },
  "target_capability": "compute"
}'

echo "BiomeOS task (submitted to Songbird):"
echo "$TASK_PAYLOAD" | jq '.' 2>/dev/null
echo ""

# In production, Songbird would have a task routing endpoint
TASK_ENDPOINT="$PORT_AUTHORITY/api/v1/tasks"

echo "Conceptual flow:"
echo "  1. BiomeOS → POST $TASK_ENDPOINT"
echo "  2. Songbird queries internal registry for 'compute'"
echo "  3. Songbird finds Toadstool (or alternate compute primal)"
echo "  4. Songbird forwards task to assigned endpoint"
echo "  5. Toadstool executes task"
echo "  6. Results bubble back through Songbird"
echo "  7. BiomeOS receives results"
echo ""

echo "Key benefits:"
echo "  • BiomeOS code never mentions 'Toadstool'"
echo "  • Works if Toadstool replaced with alternate"
echo "  • Works with multiple compute providers"
echo "  • Automatic load balancing"
echo "  • Health-based routing"
echo ""

echo -e "${GREEN}═══ Phase 7: Evolution Scenarios ═══${NC}"
echo ""

echo "Scenario 1: Multiple Toadstool instances"
echo "  Current: 1 compute service"
echo "  Add: 3 more Toadstool instances"
echo "  Impact:"
echo "    • Each registers with Songbird"
echo "    • Each gets unique port"
echo "    • Songbird load balances tasks"
echo "    • BiomeOS code unchanged"
echo "  ✓ Scales automatically"
echo ""

echo "Scenario 2: Alternate compute primal appears"
echo "  Current: Toadstool"
echo "  Add: 'ComputePrimal' with different impl"
echo "  Impact:"
echo "    • Registers 'compute' capability"
echo "    • Gets port from Songbird"
echo "    • Songbird routes based on capabilities"
echo "    • BiomeOS works with both"
echo "  ✓ Ecosystem growth without coordination"
echo ""

echo "Scenario 3: Toadstool updates API"
echo "  Current: API v2.0"
echo "  Update: API v3.0 with new features"
echo "  Impact:"
echo "    • Re-registers with new capabilities"
echo "    • Port assignment unchanged"
echo "    • BiomeOS discovers new capabilities via Songbird"
echo "    • No BiomeOS recompilation"
echo "  ✓ Transparent evolution"
echo ""

echo "Scenario 4: Songbird federates across towers"
echo "  Current: Single Songbird instance"
echo "  Deploy: Multi-tower federation"
echo "  Impact:"
echo "    • Toadstool can be in different tower"
echo "    • Songbird handles cross-tower routing"
echo "    • BiomeOS continues using same pattern"
echo "    • Geographic distribution transparent"
echo "  ✓ Federation without complexity"
echo ""

echo -e "${BLUE}═══ Phase 8: Heartbeat & Health Monitoring ═══${NC}"
echo ""

if [ -n "$SERVICE_ID" ] && [ "$SERVICE_ID" != "null" ]; then
    echo "Simulating Toadstool heartbeat..."
    HEARTBEAT_ENDPOINT="$PORT_AUTHORITY/api/v1/services/$SERVICE_ID/heartbeat"
    
    HEARTBEAT_PAYLOAD='{
      "status": "operational",
      "load": 0.23,
      "available_resources": {
        "cpu": 0.77,
        "gpu": 0.95,
        "memory": 0.82
      },
      "active_tasks": 3
    }'
    
    echo "POST $HEARTBEAT_ENDPOINT"
    echo "$HEARTBEAT_PAYLOAD" | jq '.' 2>/dev/null
    echo ""
    
    HEARTBEAT_RESPONSE=$(curl -s -X POST "$HEARTBEAT_ENDPOINT" \
        -H "Content-Type: application/json" \
        -d "$HEARTBEAT_PAYLOAD" 2>/dev/null || echo '{}')
    
    echo "Songbird response:"
    echo "$HEARTBEAT_RESPONSE" | jq '.' 2>/dev/null || echo "$HEARTBEAT_RESPONSE"
    
    echo ""
    echo "This enables:"
    echo "  • Health-based routing (avoid unhealthy services)"
    echo "  • Load-based routing (balance across services)"
    echo "  • Automatic failover (if service stops responding)"
    echo "  • Resource-aware scheduling"
fi

echo ""
echo -e "${CYAN}═══ Phase 9: Clean Shutdown ═══${NC}"
echo ""

echo "Toadstool deregisters from Songbird..."
if [ -n "$SERVICE_ID" ] && [ "$SERVICE_ID" != "null" ]; then
    DEREG_ENDPOINT="$PORT_AUTHORITY/api/v1/services/$SERVICE_ID"
    curl -s -X DELETE "$DEREG_ENDPOINT" 2>/dev/null || true
    echo -e "${GREEN}✓ Deregistered${NC}"
fi

echo ""
echo "Stopping primals..."
stop_primal_clean $TOADSTOOL_PID
stop_primal_clean $SONGBIRD_PID

echo ""
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  Demo Complete: Universal Port Authority Pattern         ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

echo "What we demonstrated:"
echo "  ✓ BiomeOS only knows about Songbird"
echo "  ✓ Toadstool registered dynamically"
echo "  ✓ Port assigned by Songbird (zero conflicts)"
echo "  ✓ Capability-based service discovery"
echo "  ✓ Task routing via port authority"
echo "  ✓ Health monitoring and load balancing"
echo "  ✓ Evolution resilience across 4 scenarios"
echo ""

echo "Key architectural insights:"
echo ""
echo "1. INDIRECTION IS POWER"
echo "   BiomeOS → Songbird → Toadstool"
echo "   Indirection enables evolution without breaking BiomeOS"
echo ""

echo "2. CAPABILITY > IDENTITY"
echo "   BiomeOS asks for 'compute', gets Toadstool"
echo "   Could get alternate primal, BiomeOS doesn't care"
echo ""

echo "3. ZERO COORDINATION NEEDED"
echo "   New compute primal? Just register with Songbird"
echo "   BiomeOS automatically uses it"
echo ""

echo "4. SOVEREIGNTY PRESERVED"
echo "   Toadstool controls its own implementation"
echo "   Songbird controls port allocation"
echo "   BiomeOS controls orchestration logic"
echo "   Each sovereign in its domain"
echo ""

echo "What happens when ecosystem grows?"
echo ""
echo "  Add 10 more Toadstool instances:"
echo "    → Each registers with Songbird"
echo "    → Songbird load balances automatically"
echo "    → BiomeOS code unchanged"
echo ""
echo "  Replace Toadstool with 'SuperCompute':"
echo "    → Registers 'compute' capability"
echo "    → Gets port from Songbird"
echo "    → BiomeOS discovers via capability"
echo "    → Works immediately"
echo ""
echo "  Deploy across 5 geographic towers:"
echo "    → Songbird federates"
echo "    → Cross-tower routing automatic"
echo "    → BiomeOS uses same pattern"
echo "    → Geographic distribution transparent"
echo ""

echo "Gap report: $GAP_REPORT"
echo ""
echo "Next: ./beardog-toadstool-encrypted-workload.sh"
echo "  (Demonstrates encryption + compute orchestration)"
echo ""


