# Squirrel Evolution Needed - January 20, 2026

## 🎯 Current State

### What Works ✅
- **Squirrel UniBin**: Deployed and running
- **JSON-RPC Server**: Accepting `query_ai` requests
- **Tower Atomic**: BearDog + Songbird operational
- **Socket Communication**: All Unix sockets functional

### What's Missing ❌
**HTTP Delegation Adapters**

The adapters that:
1. Read API keys (ANTHROPIC_API_KEY, OPENAI_API_KEY)
2. Build provider-specific HTTP requests
3. Delegate to Songbird via JSON-RPC `http.post`

---

## 🔍 Code Analysis

### Current Adapters

**File**: `crates/main/src/api/ai/adapters/mod.rs`

```rust
// Test harness adapters DELETED! (OpenAI, Ollama, HuggingFace)
// Production uses capability_ai exclusively (Pure Rust Unix sockets)

mod universal;  // ← ONLY this exists
```

**Result**: Only `UniversalAiAdapter` exists

### What UniversalAiAdapter Does

**File**: `crates/main/src/api/ai/adapters/universal.rs`

- Connects to OTHER primals via Unix sockets
- Expects to discover AI providers via `AI_PROVIDER_SOCKETS`
- Does NOT delegate HTTP to Songbird
- Does NOT use API keys

**Purpose**: Capability-based discovery of other AI-providing primals

---

## 💡 The Vision (User's Correct Architecture)

### What Should Happen

```
User → Squirrel (query_ai)
       ↓
    Squirrel has:
    - ANTHROPIC_API_KEY
    - OPENAI_API_KEY
    - Songbird socket path
       ↓
    Squirrel builds HTTP request:
    - URL: https://api.anthropic.com/v1/messages
    - Headers: x-api-key, content-type
    - Body: {model, messages, max_tokens}
       ↓
    Squirrel → Songbird (http.post JSON-RPC)
       ↓
    Songbird makes HTTPS request
       ↓
    Response back through chain
```

### What Was Deleted

**Previous implementation** (deleted):
- `adapters/openai.rs` - OpenAI HTTP adapter
- `adapters/anthropic.rs` - Anthropic HTTP adapter  
- `adapters/ollama.rs` - Ollama HTTP adapter
- `adapters/huggingface.rs` - HuggingFace HTTP adapter

**Why deleted**: They used `reqwest` directly (not Pure Rust, has `ring` dependency)

**Problem**: They weren't replaced with Songbird-delegating versions!

---

## 🛠️ What Needs to Be Built

### Provider Adapters with Songbird Delegation

**New files needed**:
- `crates/main/src/api/ai/adapters/anthropic_via_songbird.rs`
- `crates/main/src/api/ai/adapters/openai_via_songbird.rs`
- `crates/main/src/api/ai/adapters/ollama_via_songbird.rs` (optional, usually local)

### Example: Anthropic Adapter

**File**: `anthropic_via_songbird.rs` (needs to be created)

```rust
//! Anthropic AI Adapter with Songbird HTTP Delegation
//! 
//! Pure Rust implementation: delegates HTTP to Songbird instead of using reqwest

use super::*;
use crate::error::PrimalError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use uuid::Uuid;

/// Anthropic-specific request/response types
#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    temperature: Option<f64>,
}

#[derive(Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    id: String,
    model: String,
    content: Vec<AnthropicContent>,
    usage: AnthropicUsage,
}

#[derive(Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Anthropic AI Provider via Songbird HTTP Delegation
pub struct AnthropicAdapter {
    api_key: String,
    songbird_socket: PathBuf,
}

impl AnthropicAdapter {
    /// Create new Anthropic adapter
    pub fn new() -> Result<Self, PrimalError> {
        // Get API key from environment
        let api_key = env::var("ANTHROPIC_API_KEY")
            .map_err(|_| PrimalError::ConfigError("ANTHROPIC_API_KEY not set".to_string()))?;
        
        // Get Songbird socket path
        let songbird_socket = env::var("SONGBIRD_ENDPOINT")
            .or_else(|_| env::var("SONGBIRD_SOCKET"))
            .map(PathBuf::from)
            .map_err(|_| PrimalError::ConfigError("SONGBIRD_ENDPOINT not set".to_string()))?;
        
        Ok(Self {
            api_key,
            songbird_socket,
        })
    }
    
    /// Send HTTP request via Songbird
    async fn send_http_via_songbird(
        &self,
        method: &str,
        url: &str,
        headers: &HashMap<String, String>,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        // Connect to Songbird Unix socket
        let mut stream = UnixStream::connect(&self.songbird_socket)
            .await
            .map_err(|e| {
                PrimalError::NetworkError(format!(
                    "Failed to connect to Songbird at {:?}: {}",
                    self.songbird_socket, e
                ))
            })?;
        
        // Build JSON-RPC request
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "http.request",  // or "http.post"
            "params": {
                "method": method,
                "url": url,
                "headers": headers,
                "body": body,
            },
            "id": Uuid::new_v4().to_string(),
        });
        
        // Send request
        let request_json = serde_json::to_vec(&request)?;
        stream.write_all(&request_json).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;
        
        // Read response
        let mut response_data = Vec::new();
        stream.read_to_end(&mut response_data).await?;
        
        // Parse JSON-RPC response
        let response: serde_json::Value = serde_json::from_slice(&response_data)?;
        
        // Extract result or error
        if let Some(error) = response.get("error") {
            return Err(PrimalError::NetworkError(format!(
                "Songbird HTTP error: {}",
                error
            )));
        }
        
        response
            .get("result")
            .cloned()
            .ok_or_else(|| PrimalError::ParsingError("No result in response".to_string()))
    }
}

#[async_trait]
impl AiProviderAdapter for AnthropicAdapter {
    fn provider_id(&self) -> &str {
        "anthropic"
    }
    
    fn provider_name(&self) -> &str {
        "Anthropic (Claude)"
    }
    
    fn is_local(&self) -> bool {
        false  // Cloud provider
    }
    
    fn cost_per_unit(&self) -> Option<f64> {
        Some(0.00001)  // Approximate cost per token
    }
    
    fn avg_latency_ms(&self) -> u64 {
        500  // Approximate average latency
    }
    
    fn quality_tier(&self) -> QualityTier {
        QualityTier::Premium
    }
    
    fn supports_text_generation(&self) -> bool {
        true
    }
    
    fn supports_image_generation(&self) -> bool {
        false
    }
    
    async fn is_available(&self) -> bool {
        // Check if API key is set and Songbird socket exists
        !self.api_key.is_empty() && self.songbird_socket.exists()
    }
    
    async fn generate_text(
        &self,
        request: TextGenerationRequest,
    ) -> Result<TextGenerationResponse, PrimalError> {
        // Build Anthropic-specific request
        let anthropic_request = AnthropicRequest {
            model: request
                .model
                .unwrap_or_else(|| "claude-3-opus-20240229".to_string()),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: request.prompt,
            }],
            max_tokens: request.max_tokens,
            temperature: Some(request.temperature as f64),
        };
        
        // Build headers with API key
        let mut headers = HashMap::new();
        headers.insert("x-api-key".to_string(), self.api_key.clone());
        headers.insert("anthropic-version".to_string(), "2023-06-01".to_string());
        headers.insert("content-type".to_string(), "application/json".to_string());
        
        // Send HTTP request via Songbird
        let response_json = self
            .send_http_via_songbird(
                "POST",
                "https://api.anthropic.com/v1/messages",
                &headers,
                &serde_json::to_value(&anthropic_request)?,
            )
            .await?;
        
        // Parse Anthropic response
        let anthropic_response: AnthropicResponse = serde_json::from_value(response_json)?;
        
        // Convert to universal format
        Ok(TextGenerationResponse {
            text: anthropic_response.content[0].text.clone(),
            provider_id: "anthropic".to_string(),
            model: anthropic_response.model,
            usage: Some(crate::api::ai::types::UsageInfo {
                prompt_tokens: anthropic_response.usage.input_tokens,
                completion_tokens: anthropic_response.usage.output_tokens,
                total_tokens: anthropic_response.usage.input_tokens
                    + anthropic_response.usage.output_tokens,
            }),
            latency_ms: None,  // TODO: Track request time
        })
    }
    
    async fn generate_image(
        &self,
        _request: ImageGenerationRequest,
    ) -> Result<ImageGenerationResponse, PrimalError> {
        Err(PrimalError::UnsupportedOperation(
            "Anthropic does not support image generation".to_string(),
        ))
    }
}
```

---

## 🚀 Evolution Roadmap

### Phase 1: Core Adapters (4-6 hours)
1. ✅ Architecture clarified
2. ⏳ Implement `anthropic_via_songbird.rs`
3. ⏳ Implement `openai_via_songbird.rs`
4. ⏳ Update `mod.rs` to export new adapters
5. ⏳ Update `router.rs` to initialize adapters from env vars

### Phase 2: Integration (2 hours)
1. ⏳ Configure Squirrel with API keys
2. ⏳ Deploy full stack (Tower + Squirrel)
3. ⏳ Test query_ai with Anthropic
4. ⏳ Test query_ai with OpenAI
5. ⏳ Test auto-routing

### Phase 3: Polish (2 hours)
1. ⏳ Add Ollama adapter (local AI)
2. ⏳ Add cost tracking
3. ⏳ Add latency metrics
4. ⏳ Update showcase examples

---

## 📋 Immediate Next Steps

### For Squirrel Team

**Create**: `crates/main/src/api/ai/adapters/anthropic_via_songbird.rs`
- Use template above
- Implement Anthropic API format
- Delegate HTTP to Songbird via JSON-RPC

**Create**: `crates/main/src/api/ai/adapters/openai_via_songbird.rs`
- Similar to Anthropic
- OpenAI API format
- Same Songbird delegation pattern

**Update**: `crates/main/src/api/ai/adapters/mod.rs`
```rust
mod universal;
mod anthropic_via_songbird;
mod openai_via_songbird;

pub use universal::{ProviderMetadata, UniversalAiAdapter};
pub use anthropic_via_songbird::AnthropicAdapter;
pub use openai_via_songbird::OpenAiAdapter;
```

**Update**: `crates/main/src/api/ai/router.rs`
```rust
pub async fn new_with_discovery(...) -> Result<Self> {
    let mut providers: Vec<Arc<dyn AiProviderAdapter>> = Vec::new();
    
    // Try to initialize Anthropic adapter
    if let Ok(adapter) = AnthropicAdapter::new() {
        if adapter.is_available().await {
            info!("✅ Anthropic provider available");
            providers.push(Arc::new(adapter));
        }
    }
    
    // Try to initialize OpenAI adapter
    if let Ok(adapter) = OpenAiAdapter::new() {
        if adapter.is_available().await {
            info!("✅ OpenAI provider available");
            providers.push(Arc::new(adapter));
        }
    }
    
    // ... rest of discovery logic
}
```

---

## ✅ Success Criteria

### When Complete

**User can do this**:
```bash
# Deploy Tower Atomic + Squirrel
export ANTHROPIC_API_KEY="sk-ant-..."
export SONGBIRD_ENDPOINT="/tmp/songbird-nat0.sock"
python3 scripts/deploy.py nat0

# Make AI call
echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"Hello!"},"id":1}' | nc -U /tmp/squirrel-nat0.sock

# Get response
{
  "jsonrpc": "2.0",
  "result": {
    "response": "Hello! I'm Claude...",
    "provider": "anthropic",
    "model": "claude-3-opus-20240229",
    "tokens_used": 15,
    "success": true
  },
  "id": 1
}
```

**Architecture validated**:
- ✅ Squirrel reads API keys from env
- ✅ Squirrel builds provider-specific requests
- ✅ Squirrel delegates HTTP to Songbird
- ✅ Songbird makes HTTPS to Anthropic
- ✅ Response flows back correctly
- ✅ 100% Pure Rust (no reqwest, no ring)
- ✅ TRUE PRIMAL (capability-based HTTP delegation)

---

## 📊 Estimated Effort

**Total**: 8-10 hours

**Breakdown**:
- Anthropic adapter: 2-3 hours
- OpenAI adapter: 2-3 hours
- Router updates: 1 hour
- Testing + debugging: 2-3 hours
- Documentation: 1 hour

**Timeline**: 1-2 days

---

**Created**: January 20, 2026 14:00 UTC  
**Status**: Architecture understood, ready for implementation  
**Handoff**: Squirrel team  
**Priority**: HIGH - Blocks full AI orchestration validation

