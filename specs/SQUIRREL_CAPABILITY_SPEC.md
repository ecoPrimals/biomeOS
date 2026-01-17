# 🐿️ Squirrel Capability Specification

**Version**: v1.0.0  
**Date**: January 15, 2026  
**Status**: Production

---

## 📋 Overview

This specification defines Squirrel's capabilities for biomeOS integration.

### Primal Identity
- **Name**: squirrel
- **Type**: Meta-AI Orchestration Primal
- **Version**: 1.0.0
- **Capabilities**: `ai_routing`, `tool_orchestration`, `meta_ai`

---

## 🔌 Capabilities

### 1. `ai_routing`

**Description**: Multi-provider AI routing and intelligent provider selection

**Methods**:
- `ai.generate_text` - Text generation with provider selection
- `ai.generate_image` - Image generation (DALL-E, Stable Diffusion)
- `ai.list_providers` - List available AI providers
- `ai.get_provider_status` - Check provider health

**Providers Supported**:
- OpenAI (GPT-4, GPT-3.5, DALL-E)
- Ollama (Local models: Mistral, Llama, etc.)
- HuggingFace (Various models)

**Routing Constraints**:
- `require_local` - Use only local providers (privacy)
- `optimize_cost` - Minimize cost
- `optimize_quality` - Maximize quality
- `optimize_latency` - Minimize latency
- `prefer_local` - Prefer local, fallback to remote
- `max_cost` - Maximum cost per request
- `max_latency` - Maximum acceptable latency
- `min_quality` - Minimum quality threshold

**Socket Path**: `/run/user/<uid>/squirrel.sock`

**Protocol**: JSON-RPC 2.0

**Example**:
```json
{
  "jsonrpc": "2.0",
  "method": "ai.generate_text",
  "params": {
    "model": "mistral",
    "prompt": "Analyze this system...",
    "max_tokens": 500,
    "requirements": {
      "constraints": ["require_local"]
    }
  },
  "id": 1
}
```

---

### 2. `tool_orchestration`

**Description**: Universal tool execution via MCP protocol

**Methods**:
- `ai.execute` - Execute any registered tool/action
- `ai.list_actions` - List all registered actions
- `ai.get_action_schema` - Get input/output schema for action
- `provider.register` - Register new action provider
- `provider.list` - List all action providers

**Registered Actions** (7 total):
1. `text.generate` - Text generation
2. `image.generate` - Image generation
3. `primal.analyze` - PrimalPulse code analysis
4. `primal.audit_hardcoding` - PrimalPulse compliance audit
5. `rootpulse.semantic_commit` - PrimalPulse commit generation
6. `neural.graph_optimize` - PrimalPulse graph optimization
7. (More can be registered dynamically)

**Socket Path**: `/run/user/<uid>/squirrel.sock`

**Protocol**: JSON-RPC 2.0 + MCP

**Example**:
```json
{
  "jsonrpc": "2.0",
  "method": "ai.execute",
  "params": {
    "action": "primal.analyze",
    "input": {
      "primal_path": "/path/to/primal",
      "depth": "summary"
    }
  },
  "id": 1
}
```

---

### 3. `meta_ai`

**Description**: Ecosystem intelligence via PrimalPulse tools

**Methods**:
- `primal.analyze` - Analyze ecoPrimal architecture
- `primal.audit_hardcoding` - Audit for hardcoding violations
- `rootpulse.semantic_commit` - Generate semantic commits
- `neural.graph_optimize` - Optimize coordination graphs

**Use Cases**:
- Automated code analysis
- Compliance checking
- Commit message generation
- Coordination pattern optimization

**Socket Path**: `/run/user/<uid>/squirrel.sock`

**Protocol**: JSON-RPC 2.0

**Example**:
```json
{
  "jsonrpc": "2.0",
  "method": "ai.execute",
  "params": {
    "action": "neural.graph_optimize",
    "input": {
      "graph_description": "songbird -> toadstool -> squirrel",
      "purpose": "AI-powered orchestration",
      "expected_latency_ms": 5000
    }
  },
  "id": 1
}
```

---

## 🔍 Discovery

### Capability Advertisement

**Via NUCLEUS** (biomeOS discovery):
```json
{
  "primal_name": "squirrel",
  "capabilities": [
    {
      "name": "ai_routing",
      "version": "1.0.0",
      "methods": ["ai.generate_text", "ai.generate_image", "ai.list_providers"]
    },
    {
      "name": "tool_orchestration",
      "version": "1.0.0",
      "methods": ["ai.execute", "ai.list_actions", "provider.register"]
    },
    {
      "name": "meta_ai",
      "version": "1.0.0",
      "methods": ["primal.analyze", "primal.audit_hardcoding", "rootpulse.semantic_commit", "neural.graph_optimize"]
    }
  ],
  "socket_path": "/run/user/1000/squirrel.sock",
  "health_endpoint": "health"
}
```

### Socket Registry

**File**: `/run/user/<uid>/socket-registry.json`

**Format**:
```json
{
  "registry_version": "1.0",
  "last_updated": "2026-01-15T14:00:00Z",
  "sockets": {
    "ai_routing": {
      "socket_path": "/run/user/1000/squirrel.sock",
      "primal_name": "squirrel",
      "capabilities": ["ai_routing", "tool_orchestration", "meta_ai"],
      "health_endpoint": "health",
      "version": "1.0.0"
    },
    "tool_orchestration": {
      "socket_path": "/run/user/1000/squirrel.sock",
      "primal_name": "squirrel",
      "capabilities": ["ai_routing", "tool_orchestration", "meta_ai"],
      "health_endpoint": "health",
      "version": "1.0.0"
    },
    "meta_ai": {
      "socket_path": "/run/user/1000/squirrel.sock",
      "primal_name": "squirrel",
      "capabilities": ["ai_routing", "tool_orchestration", "meta_ai"],
      "health_endpoint": "health",
      "version": "1.0.0"
    }
  }
}
```

---

## ⚙️ Configuration

### Environment Variables

**Required**:
```bash
SQUIRREL_BIND_ADDRESS=127.0.0.1:9010
SQUIRREL_SOCKET=/run/user/1000/squirrel.sock
```

**Optional** (AI Providers):
```bash
OLLAMA_HOST=http://127.0.0.1:11434
OPENAI_API_KEY=sk-...
HUGGINGFACE_API_KEY=hf_...
```

**Optional** (Discovery):
```bash
SOCKET_REGISTRY_PATH=/run/user/1000/socket-registry.json
```

---

## 🧪 Health Checks

### Endpoint

**Method**: `health`

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "health",
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "status": "healthy",
    "version": "1.0.0",
    "uptime_seconds": 3600,
    "providers_available": 2,
    "actions_registered": 7
  },
  "id": 1
}
```

### HTTP Health Check (Optional)

**URL**: `http://127.0.0.1:9010/health`

**Response**:
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime_seconds": 3600
}
```

---

## 📊 Metrics

### Provider Metrics
- `ai_requests_total` - Total AI requests
- `ai_requests_by_provider` - Requests by provider
- `ai_request_latency_seconds` - Request latency histogram
- `ai_request_cost_usd` - Request cost counter
- `ai_provider_errors_total` - Provider errors

### Tool Orchestration Metrics
- `tool_executions_total` - Total tool executions
- `tool_execution_latency_seconds` - Execution latency
- `tool_execution_errors_total` - Execution errors

### PrimalPulse Metrics
- `primalpulse_analyses_total` - Code analyses
- `primalpulse_audits_total` - Hardcoding audits
- `primalpulse_commits_total` - Semantic commits
- `primalpulse_optimizations_total` - Graph optimizations

---

## 🔒 Security

### Authentication
- Unix socket permissions (user-level isolation)
- Optional API key validation (for remote providers)
- Rate limiting (per-client, per-provider)

### Privacy
- Local-first design (Ollama default)
- Explicit consent for remote providers
- No telemetry without opt-in

---

## 📚 References

### Squirrel Documentation
- `/home/eastgate/Development/ecoPrimals/phase1/squirrel/README.md`
- `/home/eastgate/Development/ecoPrimals/phase1/squirrel/USAGE_GUIDE.md`
- `/home/eastgate/Development/ecoPrimals/phase1/squirrel/BIOMEOS_HANDOFF_PACKAGE.md`

### biomeOS Integration
- `docs/primal-integrations/SQUIRREL_V1_DEPLOYMENT_JAN15.md`
- `docs/AI_SQUIRREL_INTEGRATION_EVOLUTION.md`

---

**Specification Version**: v1.0.0  
**Last Updated**: January 15, 2026  
**Status**: Production

