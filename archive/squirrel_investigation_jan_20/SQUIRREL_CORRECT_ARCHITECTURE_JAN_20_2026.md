# Squirrel Correct Architecture - January 20, 2026

## 🎯 Architecture Clarity

### What Squirrel Actually Is

**Squirrel IS the multi-provider AI orchestrator itself!**

Squirrel:
- ✅ Accepts AI requests via JSON-RPC (from users/biomeOS/Neural API)
- ✅ Routes to best provider based on cost/quality/privacy
- ✅ Manages API keys for multiple providers (Anthropic, OpenAI, etc.)
- ✅ Delegates HTTP to Songbird via Unix socket (Pure Rust!)
- ✅ Supports local AI (Ollama) and cloud providers

###Communication Flow

```
User/Neural API
    ↓ (Unix socket, JSON-RPC)
Squirrel (query_ai)
    ├─→ Has API keys for Anthropic, OpenAI, etc.
    ├─→ Decides which provider to use
    ├─→ Builds HTTP request for chosen provider
    └─→ Delegates HTTP to Songbird via Unix socket
         ↓ (Unix socket, JSON-RPC http.post)
     Songbird (HTTP client capability)
         ↓ (HTTPS, reqwest + rustls)
     External AI API (Anthropic, OpenAI, etc.)
```

---

## 📋 What Was Wrong

### Misunderstanding 1: "AI Provider Primal"
**Wrong**: Thought we needed a separate "AI Provider Primal" to wrap external APIs  
**Correct**: Squirrel IS the AI provider/orchestrator

### Misunderstanding 2: "Capability Discovery"
**Wrong**: Thought Squirrel discovers OTHER primals that offer AI  
**Correct**: Squirrel offers AI capabilities, and discovers Songbird for HTTP delegation

### Misunderstanding 3: "dev-direct-http Feature"
**Wrong**: Thought we needed to enable dev-direct-http to make API calls  
**Correct**: Production mode is fine; Squirrel delegates HTTP to Songbird

---

## ✅ Correct Configuration

### Squirrel Needs

**1. API Keys** (as environment variables):
```bash
export ANTHROPIC_API_KEY="sk-ant-..."
export OPENAI_API_KEY="sk-..."
export HUGGINGFACE_API_KEY="hf_..."
```

**2. Songbird Socket** (for HTTP delegation):
```bash
export SONGBIRD_ENDPOINT="/tmp/songbird-nat0.sock"
# OR
export SONGBIRD_SOCKET="/tmp/songbird-nat0.sock"
```

**3. Own Socket** (where it listens):
```bash
export SQUIRREL_SOCKET="/tmp/squirrel-nat0.sock"
```

### Tower Atomic Needs

**BearDog** (security):
- Socket: `/tmp/beardog-nat0.sock`
- Family: `nat0`

**Songbird** (HTTP client):
- Socket: `/tmp/songbird-nat0.sock`
- Security provider: `/tmp/beardog-nat0.sock`
- Family: `nat0`

---

## 🔧 Implementation Location

### HTTP Delegation Code

**File**: `ecoPrimals/phase1/squirrel/PRIMAL_COMMUNICATION_ARCHITECTURE.md`

**Lines 86-120**: Shows how Squirrel delegates HTTP to Songbird

```rust
// Discover Songbird's http.client capability
let songbird_socket = discover_capability("http.client").await?;

// Send HTTP request via JSON-RPC
let request = json!({
    "jsonrpc": "2.0",
    "method": "http.post",
    "params": {
        "url": "https://api.anthropic.com/v1/messages",
        "headers": { "x-api-key": env::var("ANTHROPIC_API_KEY")? },
        "body": { ... }
    },
    "id": 1
});
```

### Provider Routing Logic

**File**: `crates/main/src/api/ai/router.rs`

**What it does**:
- Selects best provider (OpenAI vs Anthropic vs Ollama)
- Builds provider-specific HTTP request
- Delegates to Songbird for HTTP execution
- Parses response and returns to user

---

## 🚀 Correct Deployment

### Step 1: Start Tower Atomic

```bash
# BearDog (security)
./plasmidBin/primals/beardog/beardog-x86_64-musl server \
  --socket /tmp/beardog-nat0.sock \
  --family-id nat0 \
  > /tmp/beardog-nat0.log 2>&1 &

# Songbird (HTTP client, bonded to BearDog)
env SONGBIRD_SOCKET="/tmp/songbird-nat0.sock" \
    SONGBIRD_SECURITY_PROVIDER="/tmp/beardog-nat0.sock" \
    SONGBIRD_ORCHESTRATOR_FAMILY_ID="nat0" \
  ./plasmidBin/primals/songbird/songbird-x86_64-musl server \
  > /tmp/songbird-nat0.log 2>&1 &
```

### Step 2: Start Squirrel (with API keys and Songbird endpoint)

```bash
env SQUIRREL_SOCKET="/tmp/squirrel-nat0.sock" \
    SONGBIRD_ENDPOINT="/tmp/songbird-nat0.sock" \
    ANTHROPIC_API_KEY="sk-ant-..." \
    OPENAI_API_KEY="sk-..." \
  ./plasmidBin/primals/squirrel/squirrel-x86_64-musl server \
  > /tmp/squirrel-nat0.log 2>&1 &
```

### Step 3: Test AI Call

```bash
echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"Hello! Respond in one sentence.","provider":"anthropic"},"id":1}' | nc -U /tmp/squirrel-nat0.sock
```

**Expected flow**:
1. Squirrel receives `query_ai` request
2. Selects Anthropic provider (or auto-selects)
3. Builds Anthropic API request with ANTHROPIC_API_KEY
4. Sends to Songbird via `http.post` JSON-RPC
5. Songbird makes HTTPS request to Anthropic
6. Songbird returns response to Squirrel
7. Squirrel parses and returns to user

---

## 📊 Architecture Verification

### Showcase Evidence

**File**: `showcase/00-standalone/03-multi-provider-routing/README.md`

**Lines 22-30**: Shows Squirrel IS the multi-provider orchestrator

**Lines 58-66**: Shows provider configuration via environment (API keys)

**Lines 88-96**: Shows Squirrel routing to OpenAI, Anthropic, Ollama automatically

### Communication Architecture

**File**: `PRIMAL_COMMUNICATION_ARCHITECTURE.md`

**Lines 86-120**: Confirms HTTP delegation to Songbird

**Quote**:
> "Squirrel → OpenAI API (via Songbird)"  
> "Squirrel → Anthropic API (via Songbird)"  
> "Why Delegate? Eliminates reqwest (and ring!), 100% Pure Rust in Squirrel"

---

## 🔍 What Needs to Happen

### Code Review Needed

**File**: `crates/main/src/api/ai/adapters/`

**Question**: Are there HTTP adapters that:
1. Read API keys from environment (ANTHROPIC_API_KEY, etc.)
2. Build provider-specific HTTP requests
3. Delegate to Songbird via JSON-RPC http.post?

**If YES**: Just configure correctly and test  
**If NO**: Need to implement HTTP delegation adapters

### Provider Adapter Pattern

**Expected implementation**:

```rust
// File: crates/main/src/api/ai/adapters/anthropic.rs (example)

pub struct AnthropicAdapter {
    api_key: String,
    songbird_socket: PathBuf,
}

impl AnthropicAdapter {
    pub fn new() -> Result<Self> {
        Ok(Self {
            api_key: env::var("ANTHROPIC_API_KEY")?,
            songbird_socket: discover_capability("http.client").await?,
        })
    }
    
    async fn generate_text(&self, request: TextGenerationRequest) -> Result<TextGenerationResponse> {
        // Build Anthropic API request
        let anthropic_request = json!({
            "model": "claude-3-opus-20240229",
            "messages": [...],
            "max_tokens": request.max_tokens,
        });
        
        // Delegate to Songbird via Unix socket
        let http_response = self.send_http_via_songbird(
            "POST",
            "https://api.anthropic.com/v1/messages",
            &[("x-api-key", &self.api_key)],
            &anthropic_request,
        ).await?;
        
        // Parse Anthropic response
        let anthropic_response: AnthropicResponse = serde_json::from_value(http_response)?;
        
        // Convert to universal format
        Ok(TextGenerationResponse {
            text: anthropic_response.content[0].text,
            provider_id: "anthropic".to_string(),
            model: anthropic_response.model,
            ...
        })
    }
    
    async fn send_http_via_songbird(...) -> Result<Value> {
        // Connect to Songbird Unix socket
        let stream = UnixStream::connect(&self.songbird_socket).await?;
        
        // Send JSON-RPC http.post request
        let request = json!({
            "jsonrpc": "2.0",
            "method": "http.post",
            "params": {
                "url": url,
                "headers": headers,
                "body": body,
            },
            "id": Uuid::new_v4(),
        });
        
        // Send, receive, parse response
        ...
    }
}
```

---

## 🎯 Next Steps

### 1. Review Squirrel Code

Check if HTTP delegation adapters exist:
```bash
cd /home/eastgate/Development/ecoPrimals/phase1/squirrel
find crates/main/src/api/ai/adapters -name "*.rs" -exec grep -l "songbird\|http.post" {} \;
```

### 2. Test Current Deployment

Try API call with current deployment:
```bash
# Already deployed: BearDog, Songbird, Squirrel
echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"Test"},"id":1}' | nc -U /tmp/squirrel-nat0.sock
```

Check logs to see what Squirrel tries to do

### 3. Implement or Fix Adapters

**If missing**: Implement HTTP delegation adapters for each provider  
**If present**: Fix configuration or bugs

### 4. Full Integration Test

Complete flow with actual API call to Anthropic via Songbird

---

## 📚 Resources

### Key Documentation
- `showcase/00_START_HERE.md` - Shows Squirrel as multi-provider orchestrator
- `showcase/00-standalone/03-multi-provider-routing/` - Provider routing demos
- `PRIMAL_COMMUNICATION_ARCHITECTURE.md` - HTTP delegation pattern
- `showcase/00-local-primal/load-api-keys.sh` - API key loading example

### Code Locations
- `crates/main/src/api/ai/router.rs` - AI routing logic
- `crates/main/src/api/ai/adapters/` - Provider adapters
- `crates/main/src/rpc/types.rs` - JSON-RPC types

---

## 🎉 When This Works

### Expected Behavior

**User request**:
```bash
echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"Hello!"},"id":1}' | nc -U /tmp/squirrel-nat0.sock
```

**Squirrel thinks**:
- "Query is simple, use cheaper provider"
- "Anthropic is available, good quality, medium cost"
- "Build Anthropic request with API key"
- "Send to Songbird for HTTP"

**Songbird executes**:
- "Received HTTP POST request for api.anthropic.com"
- "Making HTTPS connection with rustls"
- "Sending request..."
- "Got response, returning to Squirrel"

**User receives**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "response": "Hello! I'm Claude, pleased to meet you.",
    "provider": "anthropic",
    "model": "claude-3-opus-20240229",
    "tokens_used": 15,
    "latency_ms": 342,
    "success": true
  },
  "id": 1
}
```

---

**Created**: January 20, 2026 13:45 UTC  
**Status**: Architecture clarified, ready to implement/test  
**Next**: Review Squirrel adapters, test delegation

