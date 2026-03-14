> **HISTORICAL** — This handoff predates v2.37. See CURRENT_STATUS.md for latest.

# Squirrel Evolution Handoff - February 9, 2026

**Team**: Squirrel (phase1/squirrel)
**Priority**: MEDIUM - Improves AI routing flexibility
**Codebase**: `ecoPrimals/phase1/squirrel/`

---

## Context

Squirrel is the AI orchestration primal. It routes AI queries to local (Ollama) or cloud
(Anthropic, OpenAI) providers via HTTP delegation through Songbird.

The AI Bridge is validated:
- `query_ai` -> Anthropic Claude Haiku: 786ms response
- `query_ai` -> Songbird -> Ollama (phi3): ~2s response

Current bypasses:
- Must explicitly set `HTTP_REQUEST_PROVIDER_SOCKET` for Songbird discovery
- Must pass `model: "claude-3-haiku-20240307"` (default Opus not available on API key)

---

## Evolution Items

### 1. Configurable Default Model (HIGH PRIORITY)

**What**: Read default AI model from env var or config instead of hardcoding `claude-3-opus-20240229`.

**Where**: `crates/main/src/api/ai/adapters/anthropic.rs` -- the default model string.

**Fix**:
```rust
let model = params.get("model")
    .and_then(|v| v.as_str())
    .or_else(|| std::env::var("AI_DEFAULT_MODEL").ok().as_deref())
    .unwrap_or("claude-3-haiku-20240307");
```

**Also**: Add `AI_DEFAULT_MODEL` to startup docs.

**Estimated**: 15 lines

### 2. Ollama Native Adapter (MEDIUM PRIORITY)

**What**: Add a direct Ollama adapter that calls `http://localhost:11434` without going
through the `http.request` capability chain.

**Why**: Currently local AI requires: Squirrel -> discover http.request -> Songbird -> HTTP -> Ollama.
A native adapter would be: Squirrel -> TCP -> Ollama. This eliminates a hop and removes
the Songbird dependency for local inference.

**Where**: `crates/main/src/api/ai/adapters/` -- new `ollama.rs` adapter.

**How**:
- Detect Ollama via `OLLAMA_HOST` env var or `localhost:11434`
- Use Rust `TcpStream` directly (no external HTTP crate needed)
- Parse OpenAI-compatible `/v1/chat/completions` response format
- Register as `ollama` provider alongside `anthropic` and `openai`

**Estimated**: 150 lines

### 3. Provider Health Monitoring (LOW PRIORITY)

**What**: Check provider availability on startup and periodically. Route around failed providers.

**Why**: If Songbird goes down, cloud AI fails silently. Squirrel should detect this and
either fail fast or fall back to local Ollama.

**How**:
- Startup: probe each provider with a minimal request
- Runtime: track response times and error rates
- Failover: if a provider fails 3x consecutively, route to next available

**Estimated**: 100 lines

### 4. Multi-Family Socket Support (NEW)

**What**: Accept `--family-id` flag, create `squirrel-{family_id}.sock`.

**Estimated**: 10 lines

---

## Environment Variables Reference

| Variable | Purpose | Current Default |
|----------|---------|-----------------|
| `HTTP_REQUEST_PROVIDER_SOCKET` | Songbird socket for HTTP delegation | (required until Songbird evolves) |
| `AI_HTTP_PROVIDERS` | Comma-separated cloud providers | (none) |
| `AI_DEFAULT_MODEL` | Default AI model (NEW) | `claude-3-haiku-20240307` |
| `ANTHROPIC_API_KEY` | Anthropic API key | (none) |
| `OPENAI_API_KEY` | OpenAI API key | (none) |
| `OLLAMA_HOST` | Ollama endpoint (NEW) | `http://localhost:11434` |

---

## Discovery Flow

Current (with bypass):
```
Squirrel starts
  -> reads HTTP_REQUEST_PROVIDER_SOCKET env var
  -> connects to Songbird socket directly
  -> AI providers initialized
```

After Songbird evolves (no bypass):
```
Squirrel starts
  -> scans $XDG_RUNTIME_DIR/biomeos/*.sock
  -> sends discover_capabilities to each
  -> Songbird responds with ["http.request", ...]
  -> Squirrel selects Songbird as HTTP provider
  -> AI providers initialized
```
