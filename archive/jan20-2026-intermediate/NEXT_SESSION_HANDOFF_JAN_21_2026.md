# Next Session Handoff - Squirrel Integration

**Date**: January 21, 2026  
**Session Type**: Day 2 - Squirrel Integration  
**Estimated Time**: 2-3 hours  
**Prerequisites**: Terminal working, Neural API + Tower Atomic deployable

---

## 🎯 Session Goal

**Migrate Squirrel from `reqwest` to `neural-api-client`**, achieving:
- ✅ Zero `ring` dependency (100% Pure Rust)
- ✅ TRUE PRIMAL pattern (zero knowledge of Songbird/BearDog)
- ✅ Capability-based HTTP (via Neural API routing)
- ✅ Clean ecoBin harvest (zero C dependencies)

---

## ✅ What's Already Complete

### Neural API Routing Infrastructure (Day 1 + Day 2 Prep)

**Files Ready**:
1. `crates/biomeos-atomic-deploy/src/neural_router.rs` (420 lines) ✅
   - Capability-based discovery
   - Atomic composition (Tower, Nest, Node)
   - Runtime socket discovery
   - Metrics collection

2. `crates/biomeos-atomic-deploy/src/neural_api_server.rs` (+ 150 lines) ✅
   - `neural_api.proxy_http`
   - `neural_api.discover_capability`
   - `neural_api.route_to_primal`
   - `neural_api.get_routing_metrics`

3. `crates/neural-api-client/` (300+ lines) ✅
   - Complete client implementation
   - Error handling (thiserror)
   - Full documentation
   - Examples and tests

**Status**: Code complete, linter passed, awaiting build verification

---

## 📋 Session Checklist

### Phase 1: Verification (15-30 min)

#### Step 1: Fix Terminal
```bash
# If terminal still has issues, try:
# - Restart IDE
# - Open new terminal
# - Source fresh shell
```

#### Step 2: Build Verification
```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# Verify Neural Router
cargo check -p biomeos-atomic-deploy
# Expected: 0 errors

# Verify Neural API Client
cargo check -p neural-api-client
# Expected: 0 errors

# Run unit tests
cargo test -p biomeos-atomic-deploy --lib neural_router
cargo test -p neural-api-client
# Expected: All tests pass
```

**If Errors**: Most likely minor import/trait bound issues. Fix and re-run. Should take 15-30 min max.

**If Success**: Proceed to Phase 2.

---

### Phase 2: Squirrel Integration (2-3 hours)

#### Step 1: Add Neural API Client Dependency (5 min)

```bash
cd /home/eastgate/Development/ecoPrimals/phase1/squirrel
```

**Edit `Cargo.toml`**:
```toml
[dependencies]
neural-api-client = { path = "../../phase2/biomeOS/crates/neural-api-client" }

# DON'T remove reqwest yet - we'll replace calls first
```

#### Step 2: Create Wrapper Module (30 min)

**Create `crates/main/src/neural_api.rs`**:
```rust
//! Neural API client wrapper for Squirrel
//!
//! This module provides HTTP capabilities via Neural API routing,
//! eliminating direct reqwest dependency.

use anyhow::Result;
use neural_api_client::{NeuralApiClient, HttpResponse};
use serde_json::Value;
use std::collections::HashMap;

pub struct HttpClient {
    neural_client: NeuralApiClient,
}

impl HttpClient {
    /// Create new HTTP client (via Neural API)
    pub fn new(family_id: &str) -> Result<Self> {
        Ok(Self {
            neural_client: NeuralApiClient::discover(family_id)?,
        })
    }
    
    /// POST request via Neural API
    pub async fn post(
        &self,
        url: &str,
        headers: HashMap<String, String>,
        body: Value,
    ) -> Result<HttpResponse> {
        self.neural_client.proxy_http(
            "POST",
            url,
            Some(headers),
            Some(body)
        ).await
    }
    
    /// GET request via Neural API
    pub async fn get(
        &self,
        url: &str,
        headers: HashMap<String, String>,
    ) -> Result<HttpResponse> {
        self.neural_client.proxy_http(
            "GET",
            url,
            Some(headers),
            None
        ).await
    }
}
```

**Add to `crates/main/src/lib.rs`**:
```rust
pub mod neural_api;
```

#### Step 3: Find All `reqwest` Usage (5 min)

```bash
cd /home/eastgate/Development/ecoPrimals/phase1/squirrel
grep -r "reqwest::" crates/ --include="*.rs" > /tmp/squirrel_reqwest_usage.txt
cat /tmp/squirrel_reqwest_usage.txt
```

Take note of all files that need updating.

#### Step 4: Replace `reqwest` Calls (1-2 hours)

**Pattern**: Find this pattern:
```rust
// OLD (reqwest)
use reqwest::Client;

let client = Client::new();
let response = client
    .post(url)
    .header("x-api-key", api_key)
    .json(&body)
    .send()
    .await?;

let text = response.text().await?;
```

**Replace with**:
```rust
// NEW (neural-api-client)
use crate::neural_api::HttpClient;
use std::collections::HashMap;

let client = HttpClient::new(&node_id)?;  // Or from config
let response = client
    .post(
        url,
        HashMap::from([
            ("x-api-key".to_string(), api_key),
            ("content-type".to_string(), "application/json".to_string()),
        ]),
        body
    )
    .await?;

let text = response.body;
```

**Common Locations** (based on Squirrel's typical structure):
- `crates/tools/ai-tools/src/anthropic.rs` - Anthropic API calls
- `crates/tools/ai-tools/src/openai.rs` - OpenAI API calls (if exists)
- Any HTTP client initialization

**Tips**:
- Work file by file
- Run `cargo check` after each file
- Keep notes on changes

#### Step 5: Remove Old Dependencies (5 min)

**Edit `Cargo.toml`** - Remove these lines:
```toml
# DELETE:
reqwest = { version = "...", features = ["json"] }
openai = "..."  # If using reqwest internally
anthropic-sdk = "..."  # If using reqwest internally

# KEEP:
neural-api-client = { path = "..." }
```

#### Step 6: Build and Test (30 min)

```bash
cd /home/eastgate/Development/ecoPrimals/phase1/squirrel

# Full clean build
cargo clean
cargo build --release

# Expected: Should build successfully

# Run tests
cargo test --release

# Expected: Tests should pass (may need to update test mocks)
```

**If Build Errors**:
- Check import statements (`use` declarations)
- Verify all `reqwest::Client` replaced with `HttpClient`
- Check async/await syntax
- Look for `.json()` calls (no longer needed, body is already Value)

---

### Phase 3: Integration Testing (1 hour)

#### Step 1: Deploy Tower Atomic (10 min)

```bash
# Terminal 1: BearDog
cd /home/eastgate/Development/ecoPrimals/phase1/beardog
cargo run --release -- server \
  --socket /tmp/beardog-test.sock \
  --family-id test

# Terminal 2: Songbird  
cd /home/eastgate/Development/ecoPrimals/phase1/songbird
SONGBIRD_ORCHESTRATOR_SOCKET=/tmp/songbird-test.sock \
SONGBIRD_ORCHESTRATOR_FAMILY_ID=test \
cargo run --release -- orchestrator
```

**Verify**:
```bash
ls -la /tmp/beardog-test.sock /tmp/songbird-test.sock
# Both should exist
```

#### Step 2: Deploy Neural API (5 min)

```bash
# Terminal 3: Neural API
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
cargo run --release -- neural-api --family-id test
```

**Verify**:
```bash
ls -la /tmp/neural-api-test.sock
# Should exist
```

#### Step 3: Deploy Squirrel (5 min)

```bash
# Terminal 4: Squirrel
cd /home/eastgate/Development/ecoPrimals/phase1/squirrel
cargo run --release -- server \
  --socket /tmp/squirrel-test.sock \
  --node-id test  # Or whatever flag Squirrel uses
```

**Verify**:
```bash
ls -la /tmp/squirrel-test.sock
# Should exist
```

#### Step 4: Test Anthropic API Call (20 min)

**Create test script**: `test_neural_routing.sh`
```bash
#!/bin/bash

# Test Neural API → Tower → Anthropic flow

# Load API key
export ANTHROPIC_API_KEY=$(cat /home/eastgate/Development/ecoPrimals/testing-secrets/anthropic_api_key.txt)

# Call Squirrel with test prompt
# (Adjust based on Squirrel's actual API)

echo "Testing Squirrel → Neural API → Tower → Anthropic..."

# Example JSON-RPC call to Squirrel
echo '{
  "jsonrpc": "2.0",
  "method": "ai.chat",
  "params": {
    "model": "claude-3-opus-20240229",
    "messages": [{"role": "user", "content": "Say hello in exactly 5 words."}]
  },
  "id": 1
}' | nc -U /tmp/squirrel-test.sock

# Expected: Response from Claude via routing
```

**Run test**:
```bash
chmod +x test_neural_routing.sh
./test_neural_routing.sh
```

**Expected Result**:
- Squirrel receives request
- Squirrel calls `neural_api.proxy_http`
- Neural API discovers Tower Atomic
- Songbird makes HTTPS call to Anthropic
- Response flows back to test script
- **NO errors, successful Claude response**

#### Step 5: Verify Zero C Dependencies (5 min)

```bash
cd /home/eastgate/Development/ecoPrimals/phase1/squirrel

# Check dependency tree for ring/reqwest
cargo tree | grep -i "ring\|reqwest\|hyper"

# Expected: NO matches (or only in [dev-dependencies])

# Verify binary has no C deps
ldd target/release/squirrel

# Expected: "not a dynamic executable" (if musl) OR only standard libs
```

#### Step 6: Collect Metrics (5 min)

```bash
# Call Neural API for routing metrics
echo '{
  "jsonrpc": "2.0",
  "method": "neural_api.get_routing_metrics",
  "params": {},
  "id": 1
}' | nc -U /tmp/neural-api-test.sock
```

**Expected**: JSON response showing routing metrics from Squirrel's HTTP calls

---

### Phase 4: ecoBin Harvest (15 min)

#### Step 1: Build for Production (5 min)

```bash
cd /home/eastgate/Development/ecoPrimals/phase1/squirrel

# x86_64 Linux (musl for static)
cargo build --release --target x86_64-unknown-linux-musl

# ARM64 (if toolchain available)
cargo build --release --target aarch64-unknown-linux-musl
```

#### Step 2: Strip and Copy (5 min)

```bash
# Strip debug symbols
strip target/x86_64-unknown-linux-musl/release/squirrel

# Copy to plasmidBin
cp target/x86_64-unknown-linux-musl/release/squirrel \
   /home/eastgate/Development/ecoPrimals/plasmidBin/primals/squirrel/squirrel-x86_64-linux-musl

# ARM64 (if built)
strip target/aarch64-unknown-linux-musl/release/squirrel
cp target/aarch64-unknown-linux-musl/release/squirrel \
   /home/eastgate/Development/ecoPrimals/plasmidBin/primals/squirrel/squirrel-arm64-linux-musl
```

#### Step 3: Update Manifest (5 min)

**Edit `/home/eastgate/Development/ecoPrimals/plasmidBin/MANIFEST.md`**:

Update Squirrel entry:
```markdown
## Squirrel (A++ → A++ GOLD)

**Status**: ✅ TRUE PRIMAL - 100% Pure Rust + Neural API Routing  
**Version**: v0.X.X (updated: Jan 21, 2026)  
**ecoBin Grade**: A++ GOLD (zero C deps, TRUE PRIMAL compliant)

**Key Evolution**:
- ✅ Eliminated `reqwest` dependency
- ✅ Eliminated `ring` (via reqwest)
- ✅ Migrated to `neural-api-client`
- ✅ TRUE PRIMAL pattern (zero knowledge of other primals)
- ✅ Capability-based HTTP routing

**Binaries**:
- `squirrel-x86_64-linux-musl` (XM, updated Jan 21)
- `squirrel-arm64-linux-musl` (XM, updated Jan 21)

**Dependencies**: Zero C, 100% Pure Rust ✅
```

**Update version**:
```bash
echo "v0.21.0" > /home/eastgate/Development/ecoPrimals/plasmidBin/VERSION.txt
```

---

## 📊 Success Criteria

### Code Quality ✅
- [ ] Zero `reqwest` in dependency tree
- [ ] Zero `ring` in dependency tree
- [ ] All tests pass
- [ ] `cargo clippy` shows no warnings
- [ ] Build succeeds for x86_64 and ARM64

### Functionality ✅
- [ ] Squirrel can call Anthropic API via Neural API
- [ ] Response is correct (Claude generates text)
- [ ] Routing metrics show Squirrel → Neural API → Tower
- [ ] No errors in any primal's logs

### Architecture ✅
- [ ] Squirrel has zero knowledge of Songbird/BearDog
- [ ] All HTTP goes through `neural-api-client`
- [ ] Socket paths discovered at runtime (no hardcoding)
- [ ] TRUE PRIMAL pattern verified

### Binary ✅
- [ ] Static binary (musl) or minimal dynamic deps
- [ ] Size reduction (expect ~40% smaller)
- [ ] Harvest to plasmidBin successful
- [ ] Manifest updated

---

## 🚨 Potential Issues and Solutions

### Issue 1: Neural API Socket Not Found

**Symptom**: `NeuralApiClient::discover()` fails  
**Cause**: Neural API not running or wrong family_id  
**Solution**:
```bash
# Check if Neural API is running
ls -la /tmp/neural-api-*.sock

# Verify family_id matches
# In Squirrel: family_id = "test"
# Neural API must be: --family-id test
```

### Issue 2: HTTP Response Format Different

**Symptom**: `serde_json::from_str(&response.body)` fails  
**Cause**: Response body format from `neural-api-client` is different than `reqwest`  
**Solution**:
- `reqwest` `.text()` returns raw string
- `neural-api-client` `response.body` is also raw string
- Should be compatible, but check JSON structure

### Issue 3: Headers Not Passing Through

**Symptom**: API returns 401 Unauthorized  
**Cause**: Headers not properly passed to Neural API  
**Solution**:
```rust
// Ensure headers include ALL required headers
let headers = HashMap::from([
    ("x-api-key".to_string(), api_key),
    ("content-type".to_string(), "application/json".to_string()),
    ("anthropic-version".to_string(), "2023-06-01".to_string()),
]);
```

### Issue 4: Async Syntax Changes Needed

**Symptom**: Lifetime or async errors  
**Cause**: Different async signatures between reqwest and neural-api-client  
**Solution**:
- Both are `async fn`, should be compatible
- May need to adjust `.await?` placement
- Check for double `.await` or missing `.await`

### Issue 5: Test Mocks Broken

**Symptom**: Tests fail after removing reqwest  
**Cause**: Test mocks were using reqwest mock server  
**Solution**:
- Mock `NeuralApiClient` instead
- Or use integration tests with real Neural API
- Acceptable to skip HTTP tests temporarily

---

## 📚 Reference Documents

**Specifications**:
- `specs/NEURAL_API_CLIENT_SPECIFICATION.md` - Complete client spec
- `crates/neural-api-client/README.md` - Client library docs
- `wateringHole/PRIMAL_IPC_PROTOCOL.md` - Standard protocol

**Status Documents**:
- `EXTENDED_SESSION_COMPLETE_JAN_20_2026.md` - Day 1 + 2 prep summary
- `specs/NEURAL_API_ROUTING_SPECIFICATION.md` - Routing spec

**Code Locations**:
- Neural Router: `crates/biomeos-atomic-deploy/src/neural_router.rs`
- Neural API Server: `crates/biomeos-atomic-deploy/src/neural_api_server.rs`
- Neural API Client: `crates/neural-api-client/src/lib.rs`

---

## 🎯 Expected Outcomes

### After This Session

**Squirrel**:
- ✅ Zero C dependencies (100% Pure Rust)
- ✅ TRUE PRIMAL pattern (capability-based)
- ✅ Binary size reduced by ~40%
- ✅ Compile time reduced by ~33%
- ✅ Clean ecoBin harvest

**Ecosystem**:
- ✅ First primal fully migrated to Neural API routing
- ✅ Proof of concept for service mesh pattern
- ✅ Reference implementation for other primals
- ✅ Metrics showing routing in action

**Documentation**:
- ✅ Migration guide validated (this doc!)
- ✅ Real-world usage examples
- ✅ Known issues documented

---

## 📝 Session Notes Template

Use this template during the session:

```markdown
# Squirrel Neural API Migration - Session Notes

**Date**: Jan 21, 2026  
**Time Started**: __:__  
**Time Ended**: __:__

## Phase 1: Verification
- [ ] Terminal working: Yes/No
- [ ] Neural Router builds: Yes/No (errors: ___)
- [ ] Neural API Client builds: Yes/No (errors: ___)
- [ ] Tests pass: Yes/No (failures: ___)

## Phase 2: Integration
- [ ] Dependency added: Yes/No
- [ ] Wrapper module created: Yes/No
- [ ] Files needing changes: ___
- [ ] Files changed: ___
- [ ] reqwest removed: Yes/No
- [ ] Build successful: Yes/No (errors: ___)

## Phase 3: Testing
- [ ] Tower Atomic deployed: Yes/No
- [ ] Neural API deployed: Yes/No
- [ ] Squirrel deployed: Yes/No
- [ ] Anthropic test: Yes/No (response: ___)
- [ ] C deps check: Pass/Fail

## Phase 4: Harvest
- [ ] x86_64 build: Yes/No (size: ___ MB)
- [ ] ARM64 build: Yes/No (size: ___ MB)
- [ ] plasmidBin updated: Yes/No
- [ ] Manifest updated: Yes/No

## Issues Encountered
1. ___
2. ___

## Solutions Applied
1. ___
2. ___

## Final Status
- Overall success: Yes/No
- ecoBin Grade: ___
- Ready for production: Yes/No
```

---

## ✅ Checklist Summary

**Before Starting**:
- [ ] Terminal working
- [ ] Neural API code verified (build + tests)
- [ ] API keys available (`testing-secrets/anthropic_api_key.txt`)

**During Session**:
- [ ] Phase 1: Verification (15-30 min)
- [ ] Phase 2: Integration (2-3 hours)
- [ ] Phase 3: Testing (1 hour)
- [ ] Phase 4: Harvest (15 min)

**After Completion**:
- [ ] Squirrel 100% Pure Rust ✅
- [ ] TRUE PRIMAL pattern verified ✅
- [ ] ecoBin harvested to plasmidBin ✅
- [ ] Manifest updated ✅
- [ ] Session notes documented ✅

---

**Total Estimated Time**: 3-4 hours  
**Confidence**: 90% (client is ready, architecture proven)  
**Blockers**: None (all prep work complete)

**Ready to proceed!** 🚀

