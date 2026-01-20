# 🏰🐿️ Tower Atomic + Squirrel Deployment Plan

**Date**: January 19/20, 2026  
**Goal**: Deploy Tower Atomic + Squirrel for secure AI API communication  
**Method**: biomeOS neuralAPI (graph-based orchestration)

---

## 🎯 ARCHITECTURE

```
External: Anthropic API (HTTPS)
           ↕
Layer 3:  Songbird (HTTP/TLS with Pure Rust crypto via BearDog)
           ↕
Layer 2:  Tower Atomic (Unix sockets, JSON-RPC)
           ↕
Layer 1:  BearDog (Pure Rust crypto: Ed25519, X25519, ChaCha20-Poly1305)
           ↕
App:      Squirrel (AI orchestration, delegates to Tower Atomic)
```

**Key Benefits**:
- ✅ **Zero ring** (BearDog provides Pure Rust crypto)
- ✅ **Zero C dependencies** (entire stack Pure Rust)
- ✅ **Secure AI API calls** (Songbird uses BearDog for TLS)
- ✅ **Unix socket IPC** (fast, secure, no network ports internally)
- ✅ **Tower Atomic pattern** (proven working!)

---

## 📋 DEPLOYMENT PHASES

### **Phase 1: Deploy Tower Atomic via neuralAPI** ⏳

**Components**:
1. **BearDog** (security provider)
   - Binary: `plasmidBin/primals/beardog/beardog-x86_64-musl`
   - Mode: `server --socket /tmp/beardog.sock`
   - Provides: Pure Rust crypto operations

2. **Songbird** (HTTP/TLS + discovery)
   - Binary: `plasmidBin/primals/songbird`
   - Mode: `server -p 8080`
   - Env: `SONGBIRD_SECURITY_PROVIDER=/tmp/beardog.sock`
   - Provides: HTTP/TLS, service discovery

**Deployment Method**:
```bash
biomeos neural-api deploy tower_atomic.toml
```

**Expected**:
- BearDog starts, listens on Unix socket
- Songbird starts, discovers BearDog
- JWT secret obtained from BearDog
- HTTP server ready on port 8080
- Tower Atomic operational

---

### **Phase 2: Add Squirrel** ⏳

**Component**:
- **Squirrel** (AI orchestration)
  - Binary: `plasmidBin/primals/squirrel-x86_64-musl`
  - Mode: `server`
  - Env: Configure to use Tower Atomic for AI calls

**Configuration**:
```toml
# Squirrel config
[ai]
provider = "anthropic"
endpoint = "http://localhost:8080/ai/anthropic"  # Route through Songbird
use_tower_atomic = true

[security]
provider = "/tmp/beardog.sock"  # For JWT/auth
```

**Deployment Method**:
```bash
biomeos neural-api deploy tower_atomic_squirrel.toml
```

---

### **Phase 3: End-to-End Validation** ⏳

**Test Flow**:
1. Send AI request to Squirrel
2. Squirrel routes to Songbird
3. Songbird uses BearDog crypto for TLS
4. Songbird calls Anthropic API (HTTPS)
5. Response flows back through stack

**Test Command**:
```bash
# Via Unix socket to Squirrel
echo '{"jsonrpc":"2.0","method":"ai.chat","params":{"prompt":"Hello!"},"id":1}' | \
  nc -U /tmp/squirrel.sock
```

**Expected**:
- ✅ Squirrel receives request
- ✅ Routes through Tower Atomic
- ✅ Songbird handles HTTPS (Pure Rust TLS)
- ✅ Calls Anthropic API
- ✅ Returns AI response

---

## 🗂️ DEPLOYMENT GRAPH STRUCTURE

### **tower_atomic.toml** (Phase 1)

```toml
[metadata]
name = "tower_atomic"
version = "1.0.0"
description = "Tower Atomic: BearDog + Songbird (Pure Rust HTTP/TLS)"

[[services]]
id = "beardog"
binary = "plasmidBin/primals/beardog/beardog-x86_64-musl"
args = ["server", "--socket", "/tmp/beardog.sock"]
startup_order = 1

[[services]]
id = "songbird"
binary = "plasmidBin/primals/songbird"
args = ["server", "-p", "8080"]
env = { SONGBIRD_SECURITY_PROVIDER = "/tmp/beardog.sock" }
startup_order = 2
depends_on = ["beardog"]

[validation]
check_beardog_socket = "/tmp/beardog.sock"
check_songbird_port = 8080
```

---

### **tower_atomic_squirrel.toml** (Phase 2)

```toml
[metadata]
name = "tower_atomic_squirrel"
version = "1.0.0"
description = "Tower Atomic + Squirrel (AI via Pure Rust stack)"
extends = "tower_atomic.toml"

[[services]]
id = "squirrel"
binary = "plasmidBin/primals/squirrel-x86_64-musl"
args = ["server"]
env = {
  SQUIRREL_AI_ENDPOINT = "http://localhost:8080/ai",
  SQUIRREL_SECURITY_PROVIDER = "/tmp/beardog.sock",
  ANTHROPIC_API_KEY = "${ANTHROPIC_API_KEY}"
}
startup_order = 3
depends_on = ["beardog", "songbird"]

[validation]
check_squirrel_socket = "/tmp/squirrel.sock"
check_tower_atomic = true
```

---

## 🎯 VALIDATION CHECKPOINTS

### **Checkpoint 1: Tower Atomic Health** ✅

**Verify**:
- ✅ BearDog process running
- ✅ Songbird process running
- ✅ Unix socket exists: `/tmp/beardog.sock`
- ✅ HTTP port open: `8080`
- ✅ JWT generation working

**Command**:
```bash
biomeos neural-api health tower_atomic
```

---

### **Checkpoint 2: Squirrel Integration** ⏳

**Verify**:
- ✅ Squirrel process running
- ✅ Unix socket exists: `/tmp/squirrel.sock`
- ✅ Connected to Tower Atomic
- ✅ Can reach Songbird HTTP endpoint

**Command**:
```bash
biomeos neural-api health tower_atomic_squirrel
```

---

### **Checkpoint 3: AI API Call** ⏳

**Verify**:
- ✅ Squirrel receives request
- ✅ Routes through Songbird
- ✅ Songbird uses BearDog for TLS
- ✅ Calls Anthropic API
- ✅ Returns response

**Command**:
```bash
# Send test AI request
biomeos neural-api test ai_call
```

---

## 📊 SUCCESS CRITERIA

### **Tower Atomic** ✅ (Already Validated)

- ✅ BearDog provides crypto
- ✅ Songbird gets JWT from BearDog
- ✅ Unix socket communication works
- ✅ No crashes
- ✅ Pure Rust stack

### **Squirrel Integration** ⏳

- ⏳ Squirrel starts successfully
- ⏳ Connects to Tower Atomic
- ⏳ Can make HTTP requests via Songbird
- ⏳ Receives AI responses

### **End-to-End** ⏳

- ⏳ AI request → Squirrel → Songbird → Anthropic → Response
- ⏳ TLS handled by BearDog crypto (Pure Rust)
- ⏳ Zero ring, zero C dependencies
- ⏳ Production-ready performance

---

## 🚀 EXECUTION STEPS

### **Step 1: Prepare Binaries** ✅

```bash
# Already harvested:
# - BearDog: plasmidBin/primals/beardog/beardog-x86_64-musl (5.1M)
# - Songbird: plasmidBin/primals/songbird (13M)
# - Squirrel: plasmidBin/primals/squirrel-x86_64-musl (18M)
```

**Status**: ✅ All binaries ready

---

### **Step 2: Create Deployment Graphs** ⏳

```bash
# Create tower_atomic.toml
# Create tower_atomic_squirrel.toml
```

**Status**: ⏳ Ready to create

---

### **Step 3: Deploy Tower Atomic** ⏳

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
biomeos neural-api deploy graphs/tower_atomic.toml
```

**Expected**: BearDog + Songbird running, Tower Atomic operational

---

### **Step 4: Deploy Squirrel** ⏳

```bash
biomeos neural-api deploy graphs/tower_atomic_squirrel.toml
```

**Expected**: Squirrel + Tower Atomic + AI API communication working

---

### **Step 5: Validate End-to-End** ⏳

```bash
# Test AI call through full stack
biomeos neural-api test graphs/tower_atomic_squirrel.toml --test ai_call
```

**Expected**: AI response received, all components working

---

## 📈 TIMELINE

| Phase | Task | Duration | Status |
|-------|------|----------|--------|
| **Prep** | Binaries ready | - | ✅ Done |
| **Phase 1** | Create tower_atomic.toml | 15 min | ⏳ Next |
| **Phase 1** | Deploy Tower Atomic | 10 min | ⏳ |
| **Phase 1** | Validate Tower Atomic | 5 min | ⏳ |
| **Phase 2** | Create tower_atomic_squirrel.toml | 15 min | ⏳ |
| **Phase 2** | Deploy Squirrel | 10 min | ⏳ |
| **Phase 2** | Validate Integration | 10 min | ⏳ |
| **Phase 3** | End-to-End Test | 15 min | ⏳ |
| **Phase 3** | Document Results | 10 min | ⏳ |

**Total**: ~1.5 hours

---

## 🎊 EXPECTED OUTCOME

**Tower Atomic + Squirrel + Anthropic API**:
- ✅ Pure Rust HTTP/TLS stack (zero ring, zero C)
- ✅ Secure AI API communication
- ✅ Unix socket IPC (fast, secure)
- ✅ Production-ready architecture
- ✅ Proven pattern (Tower Atomic validated)

**Next**: Create deployment graphs and deploy via neuralAPI!

---

**Status**: Ready to deploy  
**Architecture**: Tower Atomic + Squirrel  
**Goal**: Secure AI API communication with Pure Rust stack

🏰🐿️✨ **Pure Rust AI Stack - Ready to Deploy!** ✨🐿️🏰

