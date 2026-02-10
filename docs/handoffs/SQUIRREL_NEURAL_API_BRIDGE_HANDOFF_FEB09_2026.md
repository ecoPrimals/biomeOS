# Squirrel AI Bridge via Neural API - Evolution Handoff

**Date**: February 9, 2026
**Status**: VALIDATED - Full chain operational
**Priority**: Achieved

---

## Problem (SOLVED)

Squirrel needs `http.request` capability to call external AI APIs (Anthropic, OpenAI).
In TRUE PRIMAL architecture, Squirrel doesn't make HTTP calls directly -- it
delegates to a primal with `http.request` capability. Songbird provides this
but doesn't implement `discover_capabilities` JSON-RPC method, so Squirrel's
socket scanner can't find it.

## Solution: Explicit Capability Env Var + Neural API Routing

Two complementary mechanisms work together:

### 1. Direct Discovery (Squirrel -> Songbird)

```bash
# Squirrel's discovery.rs checks this FIRST (instant, no socket scan)
export HTTP_REQUEST_PROVIDER_SOCKET=/run/user/$(id -u)/biomeos/songbird.sock
```

This bypasses socket scanning entirely. Squirrel trusts the env var.

### 2. Neural API Semantic Routing (for proxy_http)

```
Neural API (proxy_http / capability.call)
    |
    v
CapabilityTranslationRegistry (121 translations):
    "http.request" -> songbird.http_request
    "network.http_request" -> songbird.http_request
    |
    v
Songbird (actual HTTP transport + BearDog TLS)
    |
    v
Internet (Anthropic API, OpenAI API) or Localhost (Ollama)
```

### What Exists

1. **Neural API routing** (`neural_api_server/routing.rs`):
   - `capability.call` dispatches to `CapabilityHandler::call()`
   - Translates semantic names to actual methods
   - Discovers primal sockets at runtime

2. **Capability translations** (`capability_translation.rs`):
   - `http.request` -> Songbird `http_request` (line 720)
   - `network.http_request` -> Songbird `http_request` (line 489)
   - Full crypto, storage, compute domains mapped

3. **API keys** (`ecoPrimals/testing-secrets/api-keys.toml`):
   - Anthropic: Claude 3 Sonnet (sk-ant-api03-...)
   - OpenAI: GPT-4 (sk-proj-...)
   - Together AI, HuggingFace, Cohere also available

4. **Ollama** (running on Tower, port 11434):
   - phi3 (3.8B), llama3.2 (3B/1B), tinyllama (1B)
   - OpenAI-compatible API at `http://localhost:11434/v1`

### Validated Chains

**Chain 1: Squirrel -> Anthropic (WORKING)**
```
Squirrel query_ai (model: claude-3-haiku-20240307)
  -> Anthropic adapter
  -> discover_capability("http.request") via HTTP_REQUEST_PROVIDER_SOCKET env
  -> Songbird socket (http.request JSON-RPC)
  -> BearDog TLS crypto
  -> HTTPS -> api.anthropic.com
  -> Claude 3 Haiku
  -> Response in ~786ms
```

**Chain 2: Neural API -> Anthropic (WORKING)**
```
Neural API proxy_http
  -> discover_capability("secure_http")
  -> Songbird socket (via nucleated symlink)
  -> BearDog TLS crypto
  -> HTTPS -> api.anthropic.com
```

**Chain 3: Songbird -> Ollama (WORKING)**
```
Songbird http.request
  -> HTTP -> localhost:11434 (Ollama)
  -> phi3/tinyllama/llama3.2 inference
  -> Response in ~2s
```

### Known Limitation: Model Access

The testing API key does NOT have access to `claude-3-opus-20240229`.
Squirrel defaults to Opus when no model is specified. Always pass
`model: "claude-3-haiku-20240307"` or `model: "claude-3-sonnet-20240229"`.

### Socket Nucleation Fix

Neural API generates family-ID-suffixed socket paths (e.g., `songbird-cf7e8729dc4ff05f.sock`)
but primals create plain sockets (e.g., `songbird.sock`). The startup script creates symlinks:

```bash
ln -sf songbird.sock songbird-${FAMILY_ID}.sock
```

---

## Current Live State

### Plasmodium Collective (2 gates, covalent bond)

| Gate | GPU | VRAM | RAM | CPU | Running Primals |
|------|-----|------|-----|-----|-----------------|
| Tower | RTX 4070 | 12 GB | 31 GB | 24 | BearDog, Songbird, NestGate, Toadstool, Squirrel |
| gate2 | RTX 3090 | 24 GB | 251 GB | 128 | NestGate (ZFS), Toadstool, Squirrel |
| **Total** | **2 GPUs** | **36 GB** | **282 GB** | **152** | |

### Squirrel Status

- Healthy on both gates (JSON-RPC socket)
- AI router initialized, 0 providers (needs http.request bridge)
- `query_ai` method registered and dispatches to router
- API keys available in `testing-secrets/api-keys.toml`

### NestGate Atomic

- Tower: filesystem backend, `storage.exists/store/retrieve` all working
- gate2: ZFS-optimized, native snapshots/dedup/compression

### Ollama (Tower)

- 4 models: phi3 (3.8B), llama3.2:3b, llama3.2:1b, tinyllama (1B)
- Inference validated: phi3 responds in ~3s on RTX 4070
- OpenAI-compatible API on localhost:11434

---

## Files Modified This Session

| File | Change |
|------|--------|
| `crates/biomeos-core/src/plasmodium.rs` | SSH-based remote gate query, fixed peer discovery |
| `crates/biomeos-core/src/model_cache.rs` | Two-phase exists->retrieve for NestGate |
| NestGate `cli.rs` + `main.rs` | Inverted boolean patch (downstream) |

## Remaining Evolution Items

| Change | Owner | Priority | Status |
|--------|-------|----------|--------|
| Fix default model (Opus -> Haiku) | Squirrel team | HIGH | Workaround: pass model explicitly |
| Songbird `discover_capabilities` method | Songbird team | MEDIUM | Workaround: env var bypass |
| Socket nucleation alignment | biomeOS team | LOW | Fixed: startup script creates symlinks |
| Ollama native adapter in Squirrel | Squirrel team | MEDIUM | Future: bypasses http.request entirely |

## Startup Configuration

Use the evolved `start_nucleus.sh` script:

```bash
NODE_ID=tower1 ./scripts/start_nucleus.sh full
```

This automatically:
- Loads API keys from `ecoPrimals/testing-secrets/api-keys.toml`
- Sets `HTTP_REQUEST_PROVIDER_SOCKET` for Squirrel
- Starts Neural API with capability registration
- Creates nucleated socket symlinks
