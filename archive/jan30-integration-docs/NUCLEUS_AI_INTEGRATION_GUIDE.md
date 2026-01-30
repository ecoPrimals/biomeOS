# NUCLEUS Full AI Integration Testing Guide
**Date:** January 29, 2026  
**Hardware:** RTX 4070 12GB VRAM  
**API Keys:** Available (Anthropic, OpenAI, HuggingFace)  
**Status:** Ready for Integration Testing

---

## 🎯 **Overview**

Test all three NUCLEUS atomic configurations with Squirrel AI coordination:

1. **Tower Atomic** - BearDog + Songbird (security + discovery foundation)
2. **Node Atomic** - Tower + Toadstool (local compute with 4070 GPU)
3. **Nest Atomic** - Tower + NestGate (model persistence)
4. **Squirrel AI** - Multi-provider coordinator (local 4070 + online APIs)

---

## 🏗️ **NUCLEUS Architecture**

```
┌─────────────────────────────────────────────────────────────┐
│                    NUCLEUS Full Stack                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   TOWER      │  │     NODE     │  │     NEST     │     │
│  │  (Security)  │  │   (Compute)  │  │  (Storage)   │     │
│  ├──────────────┤  ├──────────────┤  ├──────────────┤     │
│  │ • BearDog    │  │ • Tower +    │  │ • Tower +    │     │
│  │ • Songbird   │  │ • Toadstool  │  │ • NestGate   │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│          ↓                ↓                  ↓              │
│  ┌──────────────────────────────────────────────────┐      │
│  │              SQUIRREL AI Coordinator             │      │
│  ├──────────────────────────────────────────────────┤      │
│  │  Local: Toadstool + RTX 4070 (12GB)             │      │
│  │  Online: Anthropic Claude API                    │      │
│  │  Online: OpenAI GPT API                          │      │
│  │  Cache: NestGate model persistence               │      │
│  └──────────────────────────────────────────────────┘      │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## 📋 **Prerequisites**

### **1. Hardware**
- ✅ RTX 4070 (12GB VRAM) - Confirmed
- ✅ CUDA 11.0+ drivers
- ✅ 50GB+ free disk space (for model cache)

### **2. Primal Binaries**

Build all required primals:

```bash
# Option 1: Build script (if available)
./scripts/build_primals_for_testing.sh

# Option 2: Build each manually
cd /home/eastgate/Development/ecoPrimals/<primal-repo>
cargo build --release

# Required primals:
#  - beardog (security/crypto)
#  - songbird (discovery/networking)
#  - nestgate (storage/persistence)
#  - toadstool (compute/GPU)
#  - squirrel (AI/MCP)
```

### **3. API Keys**
- ✅ Located at: `/home/eastgate/Development/ecoPrimals/testing-secrets/api-keys.toml`
- ✅ Anthropic: Available
- ✅ OpenAI: Available
- ✅ HuggingFace: Available

---

## 🚀 **Integration Test Execution**

### **Quick Start (Automated)**

```bash
# Run comprehensive integration test
./scripts/test_nucleus_ai_integration.sh

# This will:
# 1. Verify GPU (4070) and API keys
# 2. Deploy Tower Atomic (BearDog + Songbird)
# 3. Deploy Node Atomic (+ Toadstool with GPU)
# 4. Deploy Nest Atomic (+ NestGate for models)
# 5. Deploy Squirrel AI (multi-provider coordinator)
# 6. Test local AI inference (4070)
# 7. Test online AI query (Anthropic)
# 8. Test model caching (NestGate)
# 9. Test capability routing
```

### **Manual Step-by-Step**

#### **Step 1: Deploy Tower Atomic**

```bash
# Start BearDog (security foundation)
cd /path/to/beardog
export BEARDOG_SOCKET="/run/user/$(id -u)/biomeos/beardog.sock"
export FAMILY_ID="test-nucleus-$(date +%s)"
RUST_LOG=beardog=info cargo run --release -- server &
BEARDOG_PID=$!

# Wait for socket
sleep 3

# Test BearDog health
echo '{"jsonrpc":"2.0","method":"health","id":1}' | \
  nc -U "$BEARDOG_SOCKET"

# Start Songbird (discovery layer)
cd /path/to/songbird
export SONGBIRD_SOCKET="/run/user/$(id -u)/biomeos/songbird.sock"
export SONGBIRD_SECURITY_PROVIDER="beardog"
RUST_LOG=songbird=info cargo run --release -- server &
SONGBIRD_PID=$!

# Wait and test
sleep 3
echo '{"jsonrpc":"2.0","method":"health","id":2}' | \
  nc -U "$SONGBIRD_SOCKET"
```

#### **Step 2: Deploy Node Atomic (+ GPU)**

```bash
# Start Toadstool with 4070 GPU
cd /path/to/toadstool
export TOADSTOOL_SOCKET="/run/user/$(id -u)/biomeos/toadstool.sock"
export CUDA_VISIBLE_DEVICES="0"
export GPU_MEMORY_FRACTION="0.9"
RUST_LOG=toadstool=info cargo run --release -- server &
TOADSTOOL_PID=$!

# Wait and verify GPU
sleep 5
echo '{"jsonrpc":"2.0","method":"gpu.query_status","id":3}' | \
  nc -U "$TOADSTOOL_SOCKET"
```

#### **Step 3: Deploy Nest Atomic (+ Storage)**

```bash
# Start NestGate (model persistence)
cd /path/to/nestgate
export NESTGATE_SOCKET="/run/user/$(id -u)/biomeos/nestgate.sock"
export STORAGE_PATH="/var/tmp/biomeos/nestgate/models"
mkdir -p "$STORAGE_PATH"
RUST_LOG=nestgate=info cargo run --release -- server &
NESTGATE_PID=$!

# Wait and test storage
sleep 3
echo '{"jsonrpc":"2.0","method":"storage.list","id":4}' | \
  nc -U "$NESTGATE_SOCKET"
```

#### **Step 4: Deploy Squirrel AI**

```bash
# Start Squirrel with API keys
cd /path/to/squirrel
export SQUIRREL_SOCKET="/run/user/$(id -u)/biomeos/squirrel.sock"
export ANTHROPIC_API_KEY_FILE="/home/eastgate/Development/ecoPrimals/testing-secrets/api-keys.toml"
export OPENAI_API_KEY_FILE="/home/eastgate/Development/ecoPrimals/testing-secrets/api-keys.toml"
export HUGGINGFACE_TOKEN_FILE="/home/eastgate/Development/ecoPrimals/testing-secrets/api-keys.toml"
export LOCAL_MODEL_PROVIDER="toadstool"
export MODEL_CACHE_PROVIDER="nestgate"
RUST_LOG=squirrel=info cargo run --release -- server &
SQUIRREL_PID=$!

# Wait and test
sleep 5
echo '{"jsonrpc":"2.0","method":"health","id":5}' | \
  nc -U "$SQUIRREL_SOCKET"
```

---

## 🧪 **Test Scenarios**

### **Test 1: Local AI Inference (4070 GPU)**

```bash
# Query local model via Squirrel → Toadstool
cat << 'EOF' | nc -U /run/user/$(id -u)/biomeos/squirrel.sock
{
  "jsonrpc": "2.0",
  "method": "ai.local_inference",
  "params": {
    "prompt": "What are the key principles of biomeOS?",
    "model": "llama-3-8b",
    "max_tokens": 150
  },
  "id": 10
}
EOF
```

### **Test 2: Online AI Query (Anthropic)**

```bash
# Query Anthropic Claude via Squirrel
cat << 'EOF' | nc -U /run/user/$(id -u)/biomeos/squirrel.sock
{
  "jsonrpc": "2.0",
  "method": "ai.online_query",
  "params": {
    "provider": "anthropic",
    "model": "claude-3-5-sonnet-20241022",
    "prompt": "Explain capability-based discovery in 2 sentences.",
    "max_tokens": 100
  },
  "id": 11
}
EOF
```

### **Test 3: Model Caching (NestGate)**

```bash
# Cache model metadata to NestGate
cat << 'EOF' | nc -U /run/user/$(id -u)/biomeos/nestgate.sock
{
  "jsonrpc": "2.0",
  "method": "storage.store",
  "params": {
    "key": "models/llama-3-8b/metadata",
    "value": {
      "name": "Llama-3-8B",
      "size_gb": 8.5,
      "provider": "huggingface",
      "cached_at": "2026-01-29T19:00:00Z",
      "gpu_compatible": true
    }
  },
  "id": 12
}
EOF

# Retrieve cached metadata
cat << 'EOF' | nc -U /run/user/$(id -u)/biomeos/nestgate.sock
{
  "jsonrpc": "2.0",
  "method": "storage.retrieve",
  "params": {
    "key": "models/llama-3-8b/metadata"
  },
  "id": 13
}
EOF
```

### **Test 4: Capability Routing**

```bash
# Test capability.call routing via Neural API
# Squirrel decides: local vs online based on load

cat << 'EOF' | nc -U /run/user/$(id -u)/biomeos/squirrel.sock
{
  "jsonrpc": "2.0",
  "method": "ai.query",
  "params": {
    "prompt": "What is 2+2?",
    "prefer_local": true
  },
  "id": 14
}
EOF
```

### **Test 5: Cross-Primal Discovery**

```bash
# Query Songbird for capability providers
cat << 'EOF' | nc -U /run/user/$(id -u)/biomeos/songbird.sock
{
  "jsonrpc": "2.0",
  "method": "discovery.query",
  "params": {
    "capability": "ai"
  },
  "id": 15
}
EOF

# Should return: squirrel socket path
```

---

## 📊 **Expected Results**

### **Health Checks** ✅

All primals should respond to health checks:

```json
{
  "jsonrpc": "2.0",
  "result": {
    "healthy": true,
    "capabilities": ["..."],
    "uptime_seconds": 123
  },
  "id": 1
}
```

### **GPU Query** ✅

Toadstool should report 4070:

```json
{
  "jsonrpc": "2.0",
  "result": {
    "gpu_name": "NVIDIA GeForce RTX 4070",
    "vram_total_mb": 12282,
    "vram_available_mb": 11000,
    "cuda_version": "12.x"
  },
  "id": 3
}
```

### **Local AI Inference** ✅

Squirrel → Toadstool → Local Model:

```json
{
  "jsonrpc": "2.0",
  "result": {
    "response": "biomeOS follows TRUE PRIMAL principles: runtime discovery, capability-based routing, and primal autonomy...",
    "model": "llama-3-8b",
    "provider": "local",
    "latency_ms": 234
  },
  "id": 10
}
```

### **Online AI Query** ✅

Squirrel → Anthropic API:

```json
{
  "jsonrpc": "2.0",
  "result": {
    "response": "Capability-based discovery allows systems to find services by what they do rather than where they are. This enables runtime flexibility and primal autonomy.",
    "model": "claude-3-5-sonnet",
    "provider": "anthropic",
    "latency_ms": 560
  },
  "id": 11
}
```

---

## 🔍 **Troubleshooting**

### **GPU Not Detected**

```bash
# Check NVIDIA driver
nvidia-smi

# Verify CUDA
nvidia-smi --query-gpu=name,driver_version,cuda_version --format=csv

# Check Toadstool logs
journalctl -f | grep toadstool
```

### **API Key Errors**

```bash
# Verify API keys file
cat /home/eastgate/Development/ecoPrimals/testing-secrets/api-keys.toml | grep -E "anthropic|openai|hugging"

# Test Anthropic directly
curl https://api.anthropic.com/v1/messages \
  -H "x-api-key: $(grep anthropic_api_key testing-secrets/api-keys.toml | cut -d'"' -f2)" \
  -H "anthropic-version: 2023-06-01" \
  -H "content-type: application/json" \
  -d '{"model":"claude-3-5-sonnet-20241022","max_tokens":10,"messages":[{"role":"user","content":"Hi"}]}'
```

### **Socket Connection Issues**

```bash
# Check sockets exist
ls -la /run/user/$(id -u)/biomeos/*.sock

# Test socket connectivity
nc -U /run/user/$(id -u)/biomeos/beardog.sock <<< '{"jsonrpc":"2.0","method":"health","id":1}'

# Check permissions
ls -la /run/user/$(id -u)/biomeos/
```

### **Primal Not Starting**

```bash
# Check logs (if using systemd)
journalctl --user -u biomeos-beardog -f

# Or run in foreground for debugging
cd ../beardog
RUST_LOG=beardog=debug cargo run --release -- server
```

---

## 📁 **Key Files Created**

1. **graphs/nucleus_full_ai_test.toml** - Complete NUCLEUS + AI deployment graph
2. **scripts/test_nucleus_ai_integration.sh** - Automated integration test
3. **scripts/build_primals_for_testing.sh** - Build all primals
4. **NUCLEUS_AI_INTEGRATION_GUIDE.md** - This guide

---

## 🎯 **Test Checklist**

### **Phase 1: Deployment** ✅
- [ ] BearDog starts and socket created
- [ ] Songbird starts and discovers BearDog
- [ ] Toadstool starts and detects 4070 GPU
- [ ] NestGate starts and initializes storage
- [ ] Squirrel starts and loads API keys

### **Phase 2: Health Verification** ✅
- [ ] All primals respond to health checks
- [ ] Sockets accessible and responding
- [ ] GPU detected and available
- [ ] API keys loaded successfully

### **Phase 3: AI Capability Tests** ✅
- [ ] Local inference via Toadstool (4070)
- [ ] Online query via Anthropic
- [ ] Online query via OpenAI
- [ ] Model metadata cached to NestGate
- [ ] Model retrieval from NestGate

### **Phase 4: Integration Tests** ✅
- [ ] Capability routing works (semantic → primal)
- [ ] Cross-primal discovery functional
- [ ] Squirrel routes to best provider (local vs online)
- [ ] NestGate persistence across restarts

---

## 🚀 **Next Steps**

### **Immediate**

1. **Build Primals**
   ```bash
   # Navigate to each primal and build
   for primal in beardog songbird nestgate toadstool squirrel; do
     echo "Building $primal..."
     cd /home/eastgate/Development/ecoPrimals/$primal
     cargo build --release
   done
   ```

2. **Run Integration Test**
   ```bash
   cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
   ./scripts/test_nucleus_ai_integration.sh
   ```

3. **Verify Results**
   - Check all health endpoints respond
   - Test local AI inference (4070)
   - Test online AI query (Anthropic)
   - Test model caching (NestGate)

### **Advanced Testing**

1. **Model Download & Cache**
   ```bash
   # Download small model from HuggingFace
   # Cache to NestGate
   # Load from cache for inference
   ```

2. **Load Testing**
   ```bash
   # Multiple concurrent AI queries
   # Test Squirrel routing decisions
   # Verify local/online fallback
   ```

3. **Failover Testing**
   ```bash
   # Kill Toadstool mid-inference
   # Verify Squirrel falls back to online
   # Restart Toadstool, verify recovery
   ```

---

## 📊 **Success Criteria**

✅ **All primals start successfully**  
✅ **All health checks pass**  
✅ **GPU detected (4070 with 12GB)**  
✅ **Local AI inference works**  
✅ **Online AI queries work (Anthropic, OpenAI)**  
✅ **Model persistence works (NestGate)**  
✅ **Capability routing works (semantic → primal)**  
✅ **Cross-primal discovery functional**

---

## 💡 **Integration Architecture**

### **Data Flow: Local AI Query**

```
User Request
    ↓
Squirrel AI (capability router)
    ↓ (discover compute capability)
Songbird Discovery
    ↓ (returns: toadstool)
Toadstool Compute
    ↓ (GPU inference on 4070)
Model Output
    ↓ (cache to nestgate)
NestGate Persistence
    ↓
Response to User
```

### **Data Flow: Online AI Query**

```
User Request
    ↓
Squirrel AI (capability router)
    ↓ (decide: local busy, use online)
Anthropic/OpenAI API
    ↓ (via Songbird HTTP client)
Songbird → BearDog (TLS crypto)
    ↓
HTTPS Request
    ↓
AI API Response
    ↓
Squirrel formats response
    ↓
Response to User
```

---

## 🎉 **Ready to Test!**

Your RTX 4070 upgrade enables real local AI capabilities. Combined with online APIs via Squirrel, you now have a complete multi-provider AI system with:

- ✅ Local inference (4070 GPU)
- ✅ Online fallback (Anthropic, OpenAI)
- ✅ Model persistence (NestGate)
- ✅ Capability routing (semantic discovery)
- ✅ Cross-primal coordination (NUCLEUS)

**Start testing:** `./scripts/test_nucleus_ai_integration.sh`

**🦀✨ NUCLEUS + AI Integration Ready! ✨🦀**
