# 🐿️ Squirrel + Anthropic Integration Guide

**Date**: January 20, 2026  
**Status**: 🔧 **ARCHITECTURE CLARIFICATION NEEDED**

---

## 🎯 CORRECT ARCHITECTURE

### **Songbird is NOT an AI Provider!**

```
❌ WRONG: AI_PROVIDER_SOCKETS="/tmp/songbird-nat0.sock"
✅ RIGHT: Squirrel discovers http.request → finds Songbird
```

### **Two Types of AI Providers**

#### **1. External AI APIs (Anthropic, OpenAI)**
These are **NOT primals**. They are HTTP APIs.

**Architecture**:
```
Squirrel Anthropic Adapter
  ↓
  Check ANTHROPIC_API_KEY ✅
  ↓
  discover_capability("http.request") → Songbird socket
  ↓
  Build Anthropic API request (JSON)
  ↓
  Call Songbird's http.request RPC
  ↓
  Songbird makes HTTPS POST to api.anthropic.com
  ↓
  Response flows back to Squirrel
```

**Environment Variables**:
```bash
export ANTHROPIC_API_KEY="sk-ant-api03-..."
export CAPABILITY_REGISTRY_SOCKET="/tmp/neural-api-nat0.sock"
# NO AI_PROVIDER_SOCKETS for external APIs!
```

#### **2. Local AI Primals (ToadStool, future primals)**
These **ARE primals** that provide `ai.generate_text` capability.

**Architecture**:
```
Squirrel
  ↓
  Check AI_PROVIDER_SOCKETS or discover_capability("ai.generate_text")
  ↓
  Find ToadStool socket
  ↓
  Call ToadStool's ai.generate_text RPC
  ↓
  ToadStool runs local AI model
  ↓
  Response flows back to Squirrel
```

**Environment Variables**:
```bash
export AI_PROVIDER_SOCKETS="/tmp/toadstool-nat0.sock"
export CAPABILITY_REGISTRY_SOCKET="/tmp/neural-api-nat0.sock"
```

---

## 🔧 CURRENT ISSUE

### **Problem**
Squirrel treats Songbird as an AI provider:
```
export AI_PROVIDER_SOCKETS="/tmp/songbird-nat0.sock"
```

Squirrel connects and tries to call:
```json
{"jsonrpc":"2.0","method":"ai.generate_text","params":{...}}
```

Songbird responds:
```json
{"error": {"code": -32601, "message": "Method not found: ai.generate_text"}}
```

### **Root Cause**
Songbird only provides:
- `http.request`
- `discover_capabilities`
- `health`

Songbird does NOT provide:
- `ai.generate_text` ❌
- `ai.chat` ❌
- `query_ai` ❌

---

## ✅ SOLUTION

### **Option 1: Anthropic Adapter Uses HTTP Delegation** (RECOMMENDED)

**File**: `phase1/squirrel/crates/main/src/api/ai/adapters/anthropic.rs`

The `delegate_http` function already exists! It needs to be called by the adapter's `generate_text` method:

```rust
async fn generate_text(&self, request: TextGenerationRequest) -> Result<TextGenerationResponse, PrimalError> {
    // 1. Build Anthropic API request
    let url = "https://api.anthropic.com/v1/messages";
    let headers = hashmap! {
        "anthropic-version" => "2023-06-01",
        "content-type" => "application/json",
        "x-api-key" => &self.api_key,
    };
    let body = serde_json::json!({
        "model": request.model,
        "max_tokens": request.max_tokens,
        "messages": [{"role": "user", "content": request.prompt}]
    });

    // 2. Delegate HTTP call to Songbird (via capability discovery)
    let response = self.delegate_http("POST", url, headers, body).await?;

    // 3. Parse Anthropic response
    let content = response["content"][0]["text"].as_str().unwrap();
    
    Ok(TextGenerationResponse {
        text: content.to_string(),
        model: request.model,
        ...
    })
}
```

**Environment**:
```bash
export ANTHROPIC_API_KEY="sk-ant-api03-..."
export CAPABILITY_REGISTRY_SOCKET="/tmp/neural-api-nat0.sock"
# Squirrel will discover http.request → Songbird
```

### **Option 2: Remove Anthropic from Production Build** (ALTERNATIVE)

Keep `dev-direct-http` feature for development, but in production:
- External AI APIs require separate AI provider primals
- Squirrel only orchestrates local AI primals (ToadStool, etc.)

---

## 🧪 TESTING

### **Test 1: Verify HTTP Delegation**
```bash
# Start Songbird
export SONGBIRD_SECURITY_PROVIDER="/tmp/beardog-nat0.sock"
export SONGBIRD_FAMILY_ID="nat0"
songbird-x86_64 server

# Test HTTP delegation directly
echo '{
  "jsonrpc":"2.0",
  "method":"http.request",
  "params":{
    "method":"POST",
    "url":"https://api.anthropic.com/v1/messages",
    "headers":{
      "anthropic-version":"2023-06-01",
      "content-type":"application/json",
      "x-api-key":"sk-ant-api03-..."
    },
    "body":{
      "model":"claude-3-opus-20240229",
      "max_tokens":50,
      "messages":[{"role":"user","content":"Hello!"}]
    }
  },
  "id":1
}' | nc -N -U /tmp/songbird-nat0.sock
```

### **Test 2: Squirrel with Corrected Architecture**
```bash
# Start Squirrel (NO AI_PROVIDER_SOCKETS for external APIs!)
export ANTHROPIC_API_KEY="sk-ant-api03-..."
export CAPABILITY_REGISTRY_SOCKET="/tmp/neural-api-nat0.sock"
squirrel server

# Query AI
echo '{
  "jsonrpc":"2.0",
  "method":"query_ai",
  "params":{
    "prompt":"Hello!",
    "model":"claude-3-opus-20240229",
    "max_tokens":50
  },
  "id":1
}' | nc -N -U /tmp/squirrel-nat0.sock
```

**Expected**: Squirrel's Anthropic adapter discovers `http.request`, uses Songbird, gets response.

---

## 📊 CAPABILITIES SUMMARY

| Primal | Provides | Used For |
|--------|----------|----------|
| **Songbird** | `http.request`, `discovery.*`, `security.verify` | HTTP delegation to external APIs |
| **ToadStool** | `ai.generate_text`, `ai.embedding` | Local AI inference |
| **Squirrel** | `ai.*`, `tool.*` | AI orchestration, routing |
| **BearDog** | `crypto.*`, `security.*` | Encryption, JWT |

### **Capability Discovery Flow**

```
Squirrel needs to call Anthropic:
  ↓
1. Check ANTHROPIC_API_KEY (exists)
  ↓
2. discover_capability("http.request") → Neural API
  ↓
3. Neural API returns: "/tmp/songbird-nat0.sock"
  ↓
4. Squirrel calls Songbird's http.request RPC
  ↓
5. Songbird makes HTTPS call to Anthropic
  ↓
6. Response flows back
```

---

## 🚀 DEPLOYMENT

### **Tower Atomic + Squirrel (External AI)**
```bash
# 1. Start BearDog
beardog-x86_64-musl server

# 2. Start Songbird (HTTP provider)
export SONGBIRD_SECURITY_PROVIDER="/tmp/beardog-nat0.sock"
export SONGBIRD_FAMILY_ID="nat0"
songbird-x86_64 server

# 3. Start Squirrel (AI orchestrator)
export ANTHROPIC_API_KEY="sk-ant-api03-..."
export CAPABILITY_REGISTRY_SOCKET="/tmp/neural-api-nat0.sock"
squirrel-x86_64 server

# 4. Test
echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"Hello!"},"id":1}' | nc -N -U /tmp/squirrel-nat0.sock
```

---

## 📝 HANDOFF TO SQUIRREL TEAM

### **Task**: Update Anthropic Adapter to Use HTTP Delegation

**File**: `phase1/squirrel/crates/main/src/api/ai/adapters/anthropic.rs`

**Current State**: `delegate_http` function exists but not used

**Needed**: Wire up `generate_text` to call `delegate_http`

**Estimated Effort**: 30 minutes

**Priority**: 🔴 CRITICAL (blocks external AI integration)

---

**🐿️🐦✨ TRUE PRIMAL ARCHITECTURE - AI VIA CAPABILITY DISCOVERY! ✨🐦🐿️**

