# Squirrel HTTP Body Parsing Evolution Handoff

**Date**: January 29, 2026  
**From**: biomeOS Team  
**To**: Squirrel Team  
**Priority**: High  
**Status**: ✅ RESOLVED (but new issue found)

---

## UPDATE: Body Parsing FIXED ✅

The body parsing fix was correctly implemented in commit `6198cbc6` and subsequent commits. The Anthropic adapter now correctly handles `body` as a string.

**Validation Result**: Tower Atomic → Squirrel → Anthropic working at **~630ms latency**!

```json
{"jsonrpc":"2.0","result":{"latency_ms":693,"model":"claude-3-haiku-20240307","provider":"anthropic","response":"4","success":true,"tokens_used":18},"id":1}
```

---

## NEW ISSUE: Phase 4 Evolution Broke HTTP Mode

The "Phase 4: Deprecate vendor adapters" commit (`a5800d26`) removed the HTTP-based adapter initialization from `router.rs`. This breaks biomeOS integration.

### Current Behavior (Broken)

The new `AiRouter::new_with_discovery()` only discovers `ai.complete`, `ai.chat`, etc. capabilities. It no longer initializes `AnthropicAdapter` or `OpenAiAdapter` which use `http.request` capability.

### Required Behavior

The router should:
1. First try to discover `ai.xxx` capabilities (TRUE PRIMAL)
2. **Fall back** to HTTP-based adapters if no ai.xxx providers found

### Recommended Fix

In `router.rs`, after the `ai.xxx` discovery fails, initialize HTTP adapters:

```rust
// If no ai.xxx providers discovered, try HTTP-based adapters
if local_providers.is_empty() {
    info!("🔍 Initializing capability-based HTTP adapters...");
    
    // Anthropic via http.request
    if let Ok(adapter) = AnthropicAdapter::new().await {
        if adapter.is_available().await {
            info!("✅ Anthropic adapter available (HTTP via capability discovery)");
            local_providers.push(Arc::new(adapter));
        }
    }
    
    // OpenAI via http.request
    if let Ok(adapter) = OpenAiAdapter::new().await {
        if adapter.is_available().await {
            info!("✅ OpenAI adapter available (HTTP via capability discovery)");
            local_providers.push(Arc::new(adapter));
        }
    }
}
```

---

## ORIGINAL ISSUE (Now Resolved)

Squirrel's Anthropic adapter fails to parse HTTP responses from Songbird because Songbird returns the response body as a **JSON string**, but Squirrel's adapter expects a **JSON object**.

### Error Message

```
AI router error: Serialization error: invalid type: string "{\"content\":[{\"text\":\"Four.\",\"type\":\"text\"}]...}", expected struct AnthropicResponse
```

### Evidence

Direct Songbird `http.post` returns:
```json
{
  "result": {
    "body": "{\"content\":[{\"text\":\"Four.\",\"type\":\"text\"}],...}",
    "status_code": 200,
    "elapsed_ms": 586
  }
}
```

Note: `body` is a **string** containing JSON, not a parsed JSON object.

---

## Root Cause

In `crates/main/src/api/ai/adapters/anthropic.rs` lines 195-201:

```rust
// Parse HTTP response
let http_response: serde_json::Value = response_json
    .get("body")
    .cloned()
    .ok_or_else(|| PrimalError::ParsingError("No body in HTTP response".to_string()))?;

// Parse Anthropic response
let anthropic_response: AnthropicResponse = serde_json::from_value(http_response)?;
```

The code assumes `response_json.get("body")` returns a `Value::Object`, but Songbird returns a `Value::String`.

---

## Recommended Fix

Parse the body string as JSON before deserializing:

```rust
// Parse HTTP response - body comes as string from Songbird
let body_value = response_json
    .get("body")
    .cloned()
    .ok_or_else(|| PrimalError::ParsingError("No body in HTTP response".to_string()))?;

// BIOME OS FIX: Songbird returns body as string, parse it
let http_response: serde_json::Value = match body_value {
    serde_json::Value::String(s) => serde_json::from_str(&s)
        .map_err(|e| PrimalError::ParsingError(format!("Failed to parse body JSON: {}", e)))?,
    other => other, // Already parsed (for future compatibility)
};

// Parse Anthropic response
let anthropic_response: AnthropicResponse = serde_json::from_value(http_response)?;
```

### Same fix needed in OpenAI adapter

`crates/main/src/api/ai/adapters/openai.rs` likely has the same issue around line 195-200.

---

## Test Verification

After the fix, this should work:

```bash
# Start Squirrel with HTTP provider socket
HTTP_REQUEST_PROVIDER_SOCKET=/run/user/1000/biomeos/songbird-node-alpha.sock \
ANTHROPIC_API_KEY="<key>" \
./squirrel server --socket /tmp/squirrel.sock

# Test query
echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"What is 2+2?","model":"claude-3-haiku-20240307"},"id":1}' | nc -U /tmp/squirrel.sock
```

Expected result:
```json
{"jsonrpc":"2.0","result":{"text":"Four.","provider_id":"anthropic",...},"id":1}
```

---

## Current Workaround

Direct `http.post` calls to Songbird work correctly:

```bash
echo '{"jsonrpc":"2.0","method":"http.post","params":{
  "url":"https://api.anthropic.com/v1/messages",
  "headers":{"x-api-key":"<key>","anthropic-version":"2023-06-01","content-type":"application/json"},
  "body":"{\"model\":\"claude-3-haiku-20240307\",\"max_tokens\":20,\"messages\":[{\"role\":\"user\",\"content\":\"Hi\"}]}"
},"id":1}' | nc -U /run/user/1000/biomeos/songbird-node-alpha.sock
```

This returns status 200 with the correct response.

---

## Discovery Note

When using Squirrel with biomeOS, set the HTTP provider socket explicitly:

```bash
HTTP_REQUEST_PROVIDER_SOCKET=/run/user/1000/biomeos/songbird-<node>.sock
```

Otherwise, Squirrel's socket scan may find stale sockets from old processes.

---

## Impact

- **Blocked**: Squirrel → Tower Atomic → Anthropic/OpenAI integration
- **Working**: Direct http.post calls via Songbird
- **Working**: All other biomeOS deployments

---

## Files to Modify

1. `crates/main/src/api/ai/adapters/anthropic.rs` - lines 195-201
2. `crates/main/src/api/ai/adapters/openai.rs` - similar location

---

## Contact

biomeOS Team can assist with testing once the fix is deployed.

**Generated**: 2026-01-29  
**biomeOS Version**: Protocol Escalation Phase 1  
**Songbird Version**: v8.14.0

