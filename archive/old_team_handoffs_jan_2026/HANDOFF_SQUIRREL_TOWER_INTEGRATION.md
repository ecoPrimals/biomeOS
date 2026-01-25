# 🐿️ Squirrel Team: Tower Atomic Integration Handoff
## January 23, 2026 - Priority: READY (After Songbird)

**Time Estimate**: 2-4 hours  
**Priority**: MEDIUM - Wait for Songbird integration completion  
**Status**: Infrastructure ready, waiting for HTTPS validation

---

## 🎯 OBJECTIVE

**Deploy Squirrel with Tower Atomic and validate AI calls through 100% Pure Rust HTTPS stack!**

**What This Proves**:
- Complete AI ecosystem in Pure Rust
- Zero C dependencies from Squirrel → Anthropic
- Capability-based primal communication working
- Neural API orchestration functional

---

## ⏳ PREREQUISITES

### Wait for Songbird Integration ✓

**Status**: Songbird team verifying ClientHello extensions  
**ETA**: 30-60 minutes  
**Indicator**: Songbird returns HTTP 200 from real endpoints

**Check Before Starting**:
```bash
# Test Songbird HTTPS
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://httpbin.org/get"},"id":1}' | \
  nc -N -U /tmp/songbird-nat0.sock | jq '.result.status'

# Expected: 200
```

**If not 200**: Wait for Songbird team to complete integration testing

---

## 📋 INTEGRATION STEPS

### Phase 1: Configuration (15 min)

**1. Prepare API Keys**

```bash
# Copy testing secrets
cp /home/eastgate/Development/ecoPrimals/testing-secrets/anthropic-api-key.txt \
   /home/eastgate/Development/ecoPrimals/phase1/squirrel/secrets/

# Set environment variable
export ANTHROPIC_API_KEY=$(cat /home/eastgate/Development/ecoPrimals/testing-secrets/anthropic-api-key.txt)
```

**2. Review Squirrel Configuration**

**File**: `showcase/ai_providers.toml` (or equivalent)

**Should have**:
```toml
[anthropic]
enabled = true
model = "claude-3-opus-20240229"
# API key from environment: ANTHROPIC_API_KEY

[discovery]
# Discover http.request capability from Songbird
capability = "http.request"
# Discover via Neural API
registry_socket = "/tmp/neural-api-nat0.sock"
```

---

### Phase 2: Build & Harvest (20 min)

**1. Build Squirrel**

```bash
cd /home/eastgate/Development/ecoPrimals/phase1/squirrel
cargo build --release

# Check build success
ls -lh target/release/squirrel
```

**2. Harvest to plasmidBin**

```bash
cp target/release/squirrel \
   /home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/primals/squirrel/squirrel-ecoBin-tower-integration

# Link
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/primals/squirrel
ln -sf squirrel-ecoBin-tower-integration squirrel-active

# Also copy to central plasmidBin
cp squirrel-ecoBin-tower-integration /home/eastgate/Development/ecoPrimals/plasmidBin/squirrel
```

---

### Phase 3: Deployment Graph (30 min)

**1. Create Squirrel Deployment Graph**

**File**: `graphs/tower_squirrel_ecosystem.toml`

```toml
# Tower Atomic + Squirrel AI Ecosystem
graph_version = "1.0"
description = "Complete AI ecosystem: BearDog + Songbird + Squirrel"

# BearDog (Crypto)
[[nodes]]
id = "beardog"
binary = "./plasmidBin/primals/beardog/beardog"
args = ["server", "--socket", "/tmp/beardog-nat0.sock", "--family-id", "nat0"]
socket_path = "/tmp/beardog-nat0.sock"
family_id = "nat0"
depends_on = []

[nodes.capabilities_provided]
"crypto.ecdh" = "ecdh_derive"
"crypto.encrypt" = "encrypt"
"crypto.decrypt" = "decrypt"
"crypto.encrypt_aes_128_gcm" = "encrypt_aes_128_gcm"
"crypto.decrypt_aes_128_gcm" = "decrypt_aes_128_gcm"
"tls.derive_handshake_secrets" = "tls.derive_handshake_secrets"
"tls.derive_application_secrets" = "tls.derive_application_secrets"
"tls.compute_finished_verify_data" = "tls.compute_finished_verify_data"

# Songbird (TLS/HTTP)
[[nodes]]
id = "songbird"
binary = "./plasmidBin/primals/songbird/songbird-active"
args = ["server"]
socket_path = "/tmp/songbird-nat0.sock"
family_id = "nat0"
depends_on = ["beardog"]

[nodes.capabilities_provided]
"http.request" = "http.request"
"http.get" = "http.get"
"http.post" = "http.post"

# Squirrel (AI Orchestrator)
[[nodes]]
id = "squirrel"
binary = "./plasmidBin/primals/squirrel/squirrel-active"
args = ["server", "--socket", "/tmp/squirrel-nat0.sock"]
socket_path = "/tmp/squirrel-nat0.sock"
family_id = "nat0"
depends_on = ["songbird"]

[nodes.capabilities_provided]
"ai.generate_text" = "generate_text"
"ai.query" = "query_ai"

[nodes.environment]
ANTHROPIC_API_KEY = "$ANTHROPIC_API_KEY"
AI_PROVIDER_SOCKETS = "/tmp/songbird-nat0.sock"
```

**2. Deploy**

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# Deploy via Neural API
cargo run --release -p biomeos-atomic-deploy --bin neural-deploy -- tower_squirrel_ecosystem
```

---

### Phase 4: Validation (30 min)

**1. Test AI Query**

```bash
# Simple AI query
echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"Hello! Please respond with a greeting.","model":"claude-3-opus-20240229"},"id":1}' | \
  nc -N -U /tmp/squirrel-nat0.sock | jq '.'
```

**Expected Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "response": "Hello! I'm Claude, an AI assistant created by Anthropic...",
    "model": "claude-3-opus-20240229",
    "usage": {
      "input_tokens": 10,
      "output_tokens": 25
    }
  }
}
```

**2. Test Multi-Turn Conversation**

```bash
# Follow-up query
echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"What is 2+2?","model":"claude-3-opus-20240229"},"id":2}' | \
  nc -N -U /tmp/squirrel-nat0.sock | jq '.result.response'
```

**3. Test Error Handling**

```bash
# Invalid model
echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"Test","model":"invalid-model"},"id":3}' | \
  nc -N -U /tmp/squirrel-nat0.sock | jq '.error'

# Expected: Graceful error message
```

---

### Phase 5: Performance Testing (1 hour)

**1. Latency Test**

```bash
# Time 10 queries
for i in {1..10}; do
  time echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"Quick test","model":"claude-3-opus-20240229"},"id":'$i'}' | \
    nc -N -U /tmp/squirrel-nat0.sock > /dev/null
done
```

**Expected**: Each query < 2 seconds (mostly AI processing time)

**2. Concurrent Queries**

```bash
# 5 parallel queries
for i in {1..5}; do
  (echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"Parallel test '$i'","model":"claude-3-opus-20240229"},"id":'$i'}' | \
    nc -N -U /tmp/squirrel-nat0.sock > /tmp/squirrel-test-$i.json) &
done
wait

# Check results
for i in {1..5}; do
  cat /tmp/squirrel-test-$i.json | jq '.result.response' 2>/dev/null || echo "Query $i failed"
done
```

**Expected**: All 5 queries succeed

---

## 🎯 SUCCESS CRITERIA

### Functional

- [ ] Squirrel discovers Songbird's `http.request` capability
- [ ] AI query reaches Anthropic API
- [ ] Response parsed and returned correctly
- [ ] Multi-turn conversation works
- [ ] Error handling graceful

### Performance

- [ ] Query latency < 2 seconds (end-to-end)
- [ ] Concurrent queries succeed
- [ ] No memory leaks (stable over 100+ queries)
- [ ] CPU usage reasonable (< 50% during query)

### Architecture

- [ ] Zero C dependencies (Squirrel → Songbird → Anthropic)
- [ ] Capability discovery working (via Neural API)
- [ ] Unix socket communication stable
- [ ] Genetic bonding correct (security context inherited)

---

## 🔍 DEBUGGING

### If Squirrel Can't Discover Songbird

**Check**:
```bash
# Is Songbird advertising http.request?
echo '{"jsonrpc":"2.0","method":"discover_capabilities","params":{},"id":1}' | \
  nc -N -U /tmp/songbird-nat0.sock | jq '.result.capabilities'

# Expected: ["http.request", "http.get", "http.post", ...]
```

**Fix**: Ensure Songbird's `discover_capabilities` method returns `http.request`

---

### If AI Query Fails

**Check Logs**:
```bash
tail -100 /tmp/squirrel-nat0.log
tail -100 /tmp/songbird-nat0.log
```

**Common Issues**:
1. API key not set: `export ANTHROPIC_API_KEY=...`
2. Songbird not responding: Restart Songbird
3. Network issue: Check HTTPS with simple endpoint first

---

### If Response Parsing Fails

**Check**:
```bash
# Raw HTTPS response
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"POST","url":"https://api.anthropic.com/v1/messages","headers":{"x-api-key":"'$ANTHROPIC_API_KEY'","anthropic-version":"2023-06-01","content-type":"application/json"},"body":{"model":"claude-3-opus-20240229","max_tokens":100,"messages":[{"role":"user","content":"Hello"}]}},"id":1}' | \
  nc -N -U /tmp/songbird-nat0.sock | jq '.'
```

**Expected**: JSON response from Anthropic

---

## 🎊 SUCCESS CELEBRATION

### After This Works

**You'll have proven**:
- ✅ Complete AI ecosystem in Pure Rust
- ✅ Squirrel → Songbird → Anthropic (zero C deps!)
- ✅ Capability-based discovery working
- ✅ Neural API orchestration functional
- ✅ **PRODUCTION-READY AI INFRASTRUCTURE!**

**This is HUGE for ecoPrimals!** 🚀

---

## 📞 COORDINATION

**Before Starting**: Confirm with Songbird team that HTTPS validation is complete

**During Testing**: Post results in team chat

**After Success**: Update this document with findings

**If Blocked**: Contact biomeOS team with specific error messages

---

**Priority**: MEDIUM (after Songbird)  
**Time**: 2-4 hours  
**Impact**: **COMPLETE AI ECOSYSTEM VALIDATION!** 🎯

**The infrastructure is ready - just waiting for Songbird green light!** 💪

