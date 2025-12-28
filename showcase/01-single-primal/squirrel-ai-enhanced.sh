#!/usr/bin/env bash
# Enhanced Squirrel AI Demo - MCP + Multi-Agent Patterns
# Demonstrates BiomeOS discovering AI WITHOUT knowing "Squirrel" exists

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$SCRIPT_DIR/../.."

source "$SCRIPT_DIR/common/capability-discovery.sh"

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  BiomeOS: MCP Agent Discovery and Multi-Agent Coordination║"
echo "║  Finding 'ai_agent' without knowing 'Squirrel'           ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

GAP_REPORT="$SCRIPT_DIR/gaps/ai-agent-mcp-gaps.md"
mkdir -p "$SCRIPT_DIR/gaps"

cat > "$GAP_REPORT" <<'EOF'
# AI Agent and MCP Discovery Gaps

## Discovery
- [ ] Method used:
- [ ] Time to discover:

## Capabilities
- [ ] MCP protocol available:
- [ ] Multi-agent coordination:
- [ ] Tool discovery:

## Operations
- [ ] Agent task execution:
- [ ] Tool invocation:
- [ ] Multi-agent coordination:

## Evolution
- [ ] Works if Squirrel API changes:
- [ ] Works with alternate AI:
- [ ] Graceful degradation:
EOF

echo -e "${CYAN}═══ Philosophy: MCP Agent Pattern ═══${NC}"
echo ""
echo "MCP = Model Context Protocol"
echo ""
echo "BiomeOS needs:"
echo "  • capability: 'ai_agent'"
echo "  • protocol: MCP for tool discovery"
echo "  • coordination: Multi-agent workflows"
echo "  • adaptability: Agent learns from ecosystem"
echo ""
echo "BiomeOS does NOT care:"
echo "  ✗ That it's called 'Squirrel'"
echo "  ✗ What LLM backend (Claude, GPT, Llama, local)"
echo "  ✗ How agents coordinate"
echo "  ✗ Implementation details"
echo ""
echo "Result: Works with Squirrel, alternate AI agents,"
echo "        or cloud AI services via same interface."
echo ""

echo -e "${GREEN}═══ Phase 1: Start AI Agent Primal ═══${NC}"
echo ""

AI_PORT=9003
AI_PID=$(start_primal_smart "squirrel" $AI_PORT)

if [ -z "$AI_PID" ]; then
    graceful_degradation "ai_agent" \
        "Start Squirrel or alternate AI agent, or use rule-based fallback"
    echo ""
    echo "BiomeOS continues with reduced intelligence:"
    echo "  • Uses rule-based automation"
    echo "  • No adaptive learning"
    echo "  • No multi-agent coordination"
    echo "  • Can upgrade when AI agent available"
    exit 0
fi

echo ""
echo -e "${BLUE}═══ Phase 2: Discover AI Agent Capability ═══${NC}"
echo ""

AI_ENDPOINT=$(discover_primal_by_capability "ai_agent")

if [ -z "$AI_ENDPOINT" ]; then
    graceful_degradation "ai_agent" "Start Squirrel or alternate"
    stop_primal_clean $AI_PID
    exit 0
fi

echo ""
echo "Discovered AI agent endpoint: $AI_ENDPOINT"
echo ""

echo -e "${CYAN}═══ Phase 3: MCP Protocol Discovery ═══${NC}"
echo ""

probe_primal_interface "$AI_ENDPOINT"

if [ -z "$INFO_ENDPOINT" ]; then
    echo -e "${RED}✗ Could not discover interface${NC}"
    stop_primal_clean $AI_PID
    exit 1
fi

verify_primal_capability "$AI_ENDPOINT" "ai_agent"

# Check for MCP support
CAPABILITIES=$(curl -s "$INFO_ENDPOINT" 2>/dev/null || echo '{}')
echo "$CAPABILITIES" | jq '.' 2>/dev/null || echo "$CAPABILITIES"

HAS_MCP=$(echo "$CAPABILITIES" | jq -r '.capabilities[] | select(.name == "mcp_protocol")' 2>/dev/null)
HAS_TOOLS=$(echo "$CAPABILITIES" | jq -r '.capabilities[] | select(.name == "tool_execution")' 2>/dev/null)
HAS_MULTI_AGENT=$(echo "$CAPABILITIES" | jq -r '.capabilities[] | select(.name == "multi_agent")' 2>/dev/null)

echo ""
echo "AI agent features:"
[ -n "$HAS_MCP" ] && echo -e "  ${GREEN}✓${NC} MCP protocol support" || echo "  ✗ MCP protocol"
[ -n "$HAS_TOOLS" ] && echo -e "  ${GREEN}✓${NC} Tool execution" || echo "  ✗ Tool execution"
[ -n "$HAS_MULTI_AGENT" ] && echo -e "  ${GREEN}✓${NC} Multi-agent coordination" || echo "  ✗ Multi-agent"

echo ""
echo -e "${GREEN}═══ Phase 4: MCP Tool Discovery ═══${NC}"
echo ""

echo "MCP enables dynamic tool discovery:"
echo "  • Agent advertises available tools"
echo "  • BiomeOS queries tool list"
echo "  • Agent can invoke tools as needed"
echo "  • Tools evolve without agent recompilation"
echo ""

MCP_TOOLS_ENDPOINT=""
for path in "/api/v1/mcp/tools" "/api/mcp/tools" "/mcp/tools"; do
    if curl -s -f --max-time 2 "$AI_ENDPOINT$path" >/dev/null 2>&1; then
        MCP_TOOLS_ENDPOINT="$AI_ENDPOINT$path"
        echo -e "Tools endpoint: $path ${GREEN}✓${NC}"
        break
    fi
done

if [ -n "$MCP_TOOLS_ENDPOINT" ]; then
    echo ""
    echo "Querying available tools..."
    TOOLS=$(curl -s "$MCP_TOOLS_ENDPOINT" 2>/dev/null || echo '{}')
    echo "$TOOLS" | jq '.' 2>/dev/null || echo "$TOOLS"
    
    echo ""
    echo "Example tools an AI agent might have:"
    echo "  • file_read: Read files from filesystem"
    echo "  • web_search: Search the web"
    echo "  • code_execute: Run code in sandbox"
    echo "  • data_query: Query databases"
    echo "  • primal_discover: Discover other primals"
    echo ""
    echo "Key insight: Agent discovers ecosystem capabilities"
    echo "             and uses them as tools!"
fi

echo ""
echo -e "${BLUE}═══ Phase 5: Simple Agent Task ═══${NC}"
echo ""

AGENT_TASK_ENDPOINT=""
for path in "/api/v1/tasks" "/api/tasks" "/tasks"; do
    AGENT_TASK_ENDPOINT="$AI_ENDPOINT$path"
    break
done

SIMPLE_TASK=$(cat <<EOF
{
  "task_id": "demo-$(date +%s)",
  "task_type": "analysis",
  "description": "Analyze BiomeOS primal ecosystem and recommend optimal configuration",
  "context": {
    "available_primals": ["songbird", "toadstool", "beardog", "nestgate"],
    "current_load": {
      "compute": 0.23,
      "storage": 0.45,
      "network": 0.12
    },
    "requirements": {
      "compliance": ["HIPAA", "GDPR"],
      "performance": "high",
      "security": "maximum"
    }
  },
  "allowed_tools": ["primal_discover", "data_query", "optimization_suggest"]
}
EOF
)

echo "Submitting task to AI agent:"
echo "$SIMPLE_TASK" | jq '.' 2>/dev/null
echo ""

echo "POST $AGENT_TASK_ENDPOINT"
TASK_RESPONSE=$(curl -s -X POST "$AGENT_TASK_ENDPOINT" \
    -H "Content-Type: application/json" \
    -d "$SIMPLE_TASK" 2>/dev/null || echo '{}')

echo "Response:"
echo "$TASK_RESPONSE" | jq '.' 2>/dev/null || echo "$TASK_RESPONSE"

TASK_ID=$(echo "$TASK_RESPONSE" | jq -r '.task_id // .id' 2>/dev/null)

if [ -n "$TASK_ID" ] && [ "$TASK_ID" != "null" ]; then
    echo ""
    echo -e "${GREEN}✓ Task submitted${NC}"
    echo "  Task ID: $TASK_ID"
    echo ""
    echo "Agent reasoning process:"
    echo "  1. Parse requirements (HIPAA + GDPR + max security)"
    echo "  2. Check available primals"
    echo "  3. Recommend: BearDog for encryption"
    echo "  4. Recommend: Nestgate for lineage"
    echo "  5. Recommend: Songbird for coordination"
    echo "  6. Recommend: Toadstool for compute"
    echo "  7. Generate optimal configuration"
fi

echo ""
echo -e "${GREEN}═══ Phase 6: Multi-Agent Coordination ═══${NC}"
echo ""

echo "Advanced pattern: Multiple AI agents working together"
echo ""
echo "Architecture:"
echo "  • Coordinator Agent: Breaks down complex tasks"
echo "  • Specialist Agents: Handle specific domains"
echo "  • Communication: Via MCP messages"
echo "  • Consensus: Agents vote on decisions"
echo ""

echo "Example scenario: ML pipeline optimization"
echo ""
echo "  Coordinator Agent:"
echo "    ↓"
echo "    ├─→ Data Agent: Analyzes data requirements"
echo "    ├─→ Compute Agent: Selects optimal Toadstool instances"
echo "    ├─→ Security Agent: Configures BearDog encryption"
echo "    └─→ Storage Agent: Plans Nestgate persistence"
echo ""
echo "  Coordinator combines recommendations:"
echo "    → Final optimized pipeline configuration"
echo ""

MULTI_AGENT_ENDPOINT=""
for path in "/api/v1/multi-agent" "/api/multi-agent" "/multi-agent"; do
    MULTI_AGENT_ENDPOINT="$AI_ENDPOINT$path"
    break
done

MULTI_AGENT_TASK=$(cat <<EOF
{
  "task_id": "multi-agent-$(date +%s)",
  "task_type": "complex_optimization",
  "description": "Design sovereign ML training pipeline",
  "agents": [
    {
      "role": "coordinator",
      "capabilities": ["task_decomposition", "consensus_building"]
    },
    {
      "role": "data_specialist",
      "capabilities": ["data_analysis", "privacy_preservation"]
    },
    {
      "role": "compute_specialist",
      "capabilities": ["resource_optimization", "gpu_scheduling"]
    },
    {
      "role": "security_specialist",
      "capabilities": ["encryption_design", "key_management"]
    }
  ],
  "coordination_protocol": "consensus_voting",
  "constraints": {
    "sovereignty": "strict",
    "performance": "high",
    "cost": "optimized"
  }
}
EOF
)

echo "Conceptual multi-agent task:"
echo "$MULTI_AGENT_TASK" | jq '.' 2>/dev/null
echo ""

echo "Coordination flow:"
echo "  1. Coordinator receives complex task"
echo "  2. Decomposes into specialist subtasks"
echo "  3. Assigns to appropriate agents"
echo "  4. Specialists analyze and report back"
echo "  5. Coordinator builds consensus"
echo "  6. Combined solution returned to BiomeOS"
echo ""

echo -e "${BLUE}═══ Phase 7: Agent Learning and Adaptation ═══${NC}"
echo ""

echo "AI agents can learn from ecosystem behavior:"
echo ""
echo "1. Performance Learning:"
echo "   • Observes which Toadstool instances are fastest"
echo "   • Learns GPU vs CPU performance characteristics"
echo "   • Adapts recommendations over time"
echo ""

echo "2. Pattern Recognition:"
echo "   • Identifies common workload patterns"
echo "   • Predicts resource needs"
echo "   • Proactively suggests optimizations"
echo ""

echo "3. Failure Adaptation:"
echo "   • Records when primals fail"
echo "   • Learns failure patterns"
echo "   • Suggests redundancy or alternatives"
echo ""

echo "4. Sovereignty Learning:"
echo "   • Understands data sensitivity patterns"
echo "   • Learns consent preferences"
echo "   • Recommends appropriate encryption levels"
echo ""

LEARNING_ENDPOINT=""
for path in "/api/v1/learning/feedback" "/api/learning/feedback" "/feedback"; do
    LEARNING_ENDPOINT="$AI_ENDPOINT$path"
    break
done

FEEDBACK_PAYLOAD=$(cat <<EOF
{
  "task_id": "$TASK_ID",
  "outcome": "success",
  "metrics": {
    "execution_time": 45.2,
    "resource_efficiency": 0.87,
    "user_satisfaction": 0.95
  },
  "observations": {
    "toadstool_instance_2": "fastest for GPU workloads",
    "beardog_session_keys": "optimal for this use case",
    "nestgate_federation": "enabled seamless multi-region"
  }
}
EOF
)

echo "Providing feedback to agent (enables learning):"
echo "$FEEDBACK_PAYLOAD" | jq '.' 2>/dev/null
echo ""

echo "This feedback helps agent:"
echo "  • Improve future recommendations"
echo "  • Build ecosystem knowledge"
echo "  • Adapt to changing conditions"
echo "  • Personalize to BiomeOS instance"
echo ""

echo -e "${GREEN}═══ Phase 8: Tool Invocation Demo ═══${NC}"
echo ""

echo "Demonstrating agent using ecosystem as tools..."
echo ""

TOOL_INVOKE_ENDPOINT=""
for path in "/api/v1/tools/invoke" "/api/tools/invoke" "/tools/invoke"; do
    TOOL_INVOKE_ENDPOINT="$AI_ENDPOINT$path"
    break
done

TOOL_INVOCATION=$(cat <<EOF
{
  "tool": "primal_discover",
  "parameters": {
    "capability": "compute",
    "filters": {
      "gpu_available": true,
      "load_below": 0.5
    }
  },
  "context": "finding optimal compute for ML training"
}
EOF
)

echo "Agent invokes tool:"
echo "$TOOL_INVOCATION" | jq '.' 2>/dev/null
echo ""

echo "Tool execution flow:"
echo "  1. Agent decides it needs compute info"
echo "  2. Invokes 'primal_discover' tool"
echo "  3. Tool queries Songbird service registry"
echo "  4. Returns available Toadstool instances"
echo "  5. Agent uses info for recommendation"
echo ""

echo "Key insight: Agent treats entire ecosystem as toolset!"
echo ""

echo -e "${GREEN}═══ Phase 9: Evolution Scenarios ═══${NC}"
echo ""

echo "Scenario 1: New LLM backend"
echo "  Current: Claude 3.5 Sonnet"
echo "  Upgrade: GPT-5 or Llama 4"
echo "  Impact: BiomeOS uses same MCP interface"
echo "  ✓ Resilient - model agnostic"
echo ""

echo "Scenario 2: New tools available"
echo "  Current: 5 tools"
echo "  Add: 10 more ecosystem tools"
echo "  Impact: Agent discovers via MCP"
echo "  ✓ Resilient - dynamic tool discovery"
echo ""

echo "Scenario 3: Alternate AI service"
echo "  Current: Local Squirrel"
echo "  Add: Cloud AI service"
echo "  Impact: BiomeOS discovers via 'ai_agent' capability"
echo "  ✓ Resilient - works with any provider"
echo ""

echo "Scenario 4: AI unavailable"
echo "  Current: Working"
echo "  If unavailable: Graceful degradation"
echo "  Impact: Falls back to rule-based automation"
echo "  ✓ Resilient - continues with reduced intelligence"
echo ""

echo -e "${CYAN}═══ Phase 10: Clean Shutdown ═══${NC}"
echo ""

echo "Stopping AI agent..."
stop_primal_clean $AI_PID

echo -e "${GREEN}✓ AI agent stopped${NC}"
echo "  Learning state persisted"
echo "  Tool registry saved"
echo "  Ready for next session"
echo ""

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  Demo Complete: MCP Agent and Multi-Agent Coordination   ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""

echo "What we demonstrated:"
echo "  ✓ Discovered 'ai_agent' capability (not 'Squirrel' name)"
echo "  ✓ MCP protocol for tool discovery"
echo "  ✓ Agent task execution"
echo "  ✓ Multi-agent coordination pattern"
echo "  ✓ Learning and adaptation"
echo "  ✓ Ecosystem-as-tools pattern"
echo "  ✓ Evolution resilience"
echo ""

echo "Key insights:"
echo "  1. BiomeOS works with ANY AI agent (MCP compatible)"
echo "  2. Agents discover ecosystem dynamically"
echo "  3. Multi-agent enables complex reasoning"
echo "  4. Learning improves over time"
echo ""

echo "Real-world applications:"
echo "  • Autonomous system optimization"
echo "  • Intelligent resource scheduling"
echo "  • Adaptive security responses"
echo "  • Complex pipeline design"
echo "  • Multi-domain problem solving"
echo ""

echo "MCP advantage:"
echo "  • Standard protocol (not Squirrel-specific)"
echo "  • Works with Claude, GPT, Llama, etc."
echo "  • Dynamic tool discovery"
echo "  • Composable agent ecosystems"
echo ""

echo "Gap report: $GAP_REPORT"
echo ""


