# Squirrel AI Provider Configuration - January 20, 2026

## 🎯 Discovery: Agnostic Infrastructure Exists!

### What I Found

**Squirrel DOES have agnostic AI infrastructure!**

**Location**: `crates/tools/ai-tools/examples/multi_model_config.toml`

**Configuration Example**:
```toml
[api_providers.anthropic]
enabled = true
# api_key is read from environment variable ANTHROPIC_API_KEY
default_model = "claude-3-haiku-20240307"
timeout_seconds = 45

[api_providers.openai]
enabled = true
# api_key is read from environment variable OPENAI_API_KEY
default_model = "gpt-3.5-turbo"
timeout_seconds = 30
```

---

## 📊 Current State

### Squirrel v2.0.0 Build Mode

**Current**: Production mode (default)
- Uses `UniversalAiAdapter` ONLY
- Expects AI providers via `AI_PROVIDER_SOCKETS`
- NO direct HTTP to AI APIs
- Requires capability discovery

**Available**: Development mode (`--features dev-direct-http`)
- Includes OpenAI, HuggingFace, Ollama adapters
- Direct HTTP access to AI providers
- Uses API keys from environment

### Evidence from Code

**File**: `config/development.toml`

```toml
# BUILD MODES (v1.1.0+):
# 
# Production Mode (default):
#   $ cargo build --release
#   - Uses UniversalAiAdapter ONLY (Unix sockets)
#   - NO direct HTTP to AI providers
#   - Requires Songbird AI proxy or AI_PROVIDER_SOCKETS
#
# Development Mode (--features dev-direct-http):
#   $ cargo build --release --features dev-direct-http
#   - Includes OpenAI, HuggingFace, Ollama adapters
#   - Direct HTTP access to AI providers (fast iteration)
#   - Requires API keys (see below)
```

---

## 🛠️ Solution Options

### Option 1: Rebuild with dev-direct-http Feature (FASTEST)

**What**: Enable built-in HTTP adapters (not using Songbird delegation)

**Build**:
```bash
cd /home/eastgate/Development/ecoPrimals/phase1/squirrel
cargo build --release --target x86_64-unknown-linux-musl --features dev-direct-http
```

**Pros**:
- Fast (works immediately)
- Uses existing tested adapters
- API key configuration already implemented

**Cons**:
- Uses `reqwest` (has `ring` dependency) - NOT Pure Rust!
- Direct HTTP (not delegating to Songbird)
- Not TRUE PRIMAL architecture

**Use Case**: Quick testing/validation only

### Option 2: Enable AI-Tools Infrastructure (RECOMMENDED)

**What**: Use Squirrel's `ai-tools` crate with proper configuration

**Steps**:
1. Ensure `ai-tools` crate is included in build
2. Create `squirrel.toml` with AI provider config (✅ Already created!)
3. Configure environment variables
4. Start Squirrel

**Configuration** (already created as `squirrel.toml`):
```toml
[ai.providers.anthropic]
enabled = true
provider_type = "http_delegated"  # Uses Songbird for HTTP
songbird_socket = "/tmp/songbird-nat0.sock"
default_model = "claude-3-opus-20240229"

[ai.providers.openai]
enabled = true
provider_type = "http_delegated"
songbird_socket = "/tmp/songbird-nat0.sock"
default_model = "gpt-4"
```

**Pros**:
- Uses agnostic infrastructure
- Can delegate to Songbird (Pure Rust!)
- Matches Squirrel's design philosophy
- Configuration-driven (no rebuild needed)

**Cons**:
- Requires Squirrel team to verify/enable
- May need code updates to wire config to router

**Status**: Configuration ready, needs Squirrel team verification

### Option 3: Create AI Provider Primals (PURE ARCHITECTURE)

**What**: Separate primals for Anthropic, OpenAI, etc.

**Each primal**:
- Reads API keys from environment
- Builds provider-specific HTTP requests
- Delegates to Songbird for HTTP
- Advertises to Squirrel via `AI_PROVIDER_SOCKETS`

**Pros**:
- TRUE PRIMAL architecture
- Clean separation of concerns
- Each provider independently updateable
- Pure Rust throughout

**Cons**:
- More work (2-3 days per provider)
- More moving parts

**Use Case**: Production long-term solution

---

## 📋 Immediate Recommendation

### For Quick Validation (Today)

**Handoff to Squirrel Team**:
"Review `ai-tools` crate and enable http_delegated provider type to use Songbird for HTTP requests"

**Files to Review**:
- `crates/tools/ai-tools/examples/multi_model_config.toml` - Configuration pattern
- `crates/tools/ai-tools/src/` - Implementation
- `crates/main/src/api/ai/router.rs` - How to integrate with router

**Goal**: Enable Squirrel to:
1. Load provider config from `squirrel.toml` (already created)
2. Read API keys from environment
3. Build provider-specific HTTP requests
4. Delegate to Songbird at `songbird_socket` path
5. Route between providers based on capabilities

### For Production (This Week)

**Option 3**: Build separate AI Provider Primals

**Timeline**: 2-3 days per provider

---

## 📁 Files Created

### Configuration (Ready to Use)

**File**: `/home/eastgate/Development/ecoPrimals/phase2/biomeOS/squirrel.toml`

**Contents**:
- AI provider configuration (Anthropic, OpenAI, Ollama)
- HTTP delegation to Songbird
- Routing rules
- Security settings
- Monitoring configuration

**Status**: ✅ Complete and ready

### Documentation

**This file**: `SQUIRREL_AI_PROVIDER_CONFIGURATION_JAN_20_2026.md`
- Architecture analysis
- Solution options
- Configuration examples
- Handoff to Squirrel team

---

## 🎯 Next Steps

### Immediate (Right Now)

**Decision Point**: Which option?

1. **Option 1** (dev-direct-http): Quick test, not Pure Rust
2. **Option 2** (ai-tools config): Use agnostic infra, needs team review
3. **Option 3** (provider primals): Pure architecture, longer timeline

### If Option 2 (RECOMMENDED)

**Handoff to Squirrel Team**:

**Task**: "Enable AI provider configuration with Songbird HTTP delegation"

**Requirements**:
1. Load provider config from `squirrel.toml`
2. Support `provider_type = "http_delegated"`
3. For each enabled provider:
   - Read API key from environment (`ANTHROPIC_API_KEY`, etc.)
   - Build provider-specific HTTP request
   - Connect to Songbird at `songbird_socket` path
   - Send JSON-RPC `http.post` request to Songbird
   - Parse response
4. Register providers with AI router
5. Enable routing between providers

**Files to Implement/Update**:
- `crates/main/src/api/ai/adapters/anthropic_via_songbird.rs` (new)
- `crates/main/src/api/ai/adapters/openai_via_songbird.rs` (new)
- `crates/main/src/api/ai/router.rs` (update to load from config)
- `crates/main/src/main.rs` (update to pass config to router)

**Template**: See `SQUIRREL_EVOLUTION_NEEDED_JAN_20_2026.md` for adapter implementation template

**Timeline**: 4-6 hours

---

## ✅ What's Ready

**Configuration**: ✅ Complete (`squirrel.toml`)

**Environment**:
```bash
export ANTHROPIC_API_KEY="sk-ant-..."
export OPENAI_API_KEY="sk-..."
export SONGBIRD_ENDPOINT="/tmp/songbird-nat0.sock"
```

**Tower Atomic**: ✅ Deployed (BearDog + Songbird)

**Squirrel v2.0.0**: ✅ Operational (waiting for provider config)

---

## 🔍 What We Discovered

1. ✅ Squirrel HAS agnostic AI infrastructure (`ai-tools` crate)
2. ✅ Configuration pattern exists (`multi_model_config.toml`)
3. ✅ Two build modes: production (capability discovery) vs dev (direct HTTP)
4. ✅ Config structure supports `provider_type = "http_delegated"`
5. ⚠️  HTTP delegation adapters need to be implemented/enabled

---

**Status**: Configuration ready, awaiting Squirrel team to enable HTTP delegation  
**Recommended**: Option 2 (use ai-tools infrastructure with Songbird delegation)  
**Timeline**: 4-6 hours (Squirrel team implementation)  
**Blocker**: Adapter implementation for http_delegated provider type

---

*The infrastructure exists - we just need to wire it up!* 🐿️🔌✨


