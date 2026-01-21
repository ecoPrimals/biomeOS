# 🚧 End-to-End Testing - Current Blockers

**Date**: January 21, 2026 02:11 UTC  
**Status**: ⚠️ **BLOCKED** - Environment Variable Passing Issues  
**Goal**: Test Pure Rust AI Stack (Squirrel → Songbird → Anthropic)

---

## 🎯 OBJECTIVE

Test live end-to-end AI query flow:
```
User → Squirrel → Songbird → Anthropic API → Response
```

---

## ✅ WHAT'S READY

1. **Songbird v4.3.0** - HTTP Delegation Provider
   - ✅ RPC methods: `discover_capabilities`, `health`, `http.request`
   - ✅ Connection handling fixed (EOF for read_to_end)
   - ✅ Production ready

2. **Squirrel Event-Driven Discovery**
   - ✅ Registry-first discovery (not socket scan)
   - ✅ Fixed Neural API method name
   - ✅ Binary rebuilt and harvested

3. **Neural API Registry**
   - ✅ Has `http.request` → `/tmp/songbird-nat0.sock` registered
   - ✅ Responding to discovery queries

---

## ⚠️ CURRENT BLOCKERS

### **1. Environment Variable Passing**

**Problem**: `ANTHROPIC_API_KEY` and `CAPABILITY_REGISTRY_SOCKET` aren't reaching Squirrel process.

**Evidence**:
```bash
# Logs show:
"⚠️  Set ANTHROPIC_API_KEY or OPENAI_API_KEY"
# Even though script exports them!
```

**Root Cause**: Background processes (`nohup`, `&`) don't inherit exports from parent shell.

**Solution Needed**:
- Pass variables inline with command: `VAR=value command`
- Or use `env` command: `env VAR=value command`
- Or source script instead of executing: `. ./deploy.sh`

---

### **2. Shell Environment Pollution**

**Problem**: Shell has `AI_PROVIDER_SOCKETS=/tmp/songbird-nat0.sock` set globally.

**Evidence**:
```bash
$ echo $AI_PROVIDER_SOCKETS
/tmp/songbird-nat0.sock
```

**Root Cause**: Unknown - possibly from previous testing or a startup script.

**Solution**: `unset AI_PROVIDER_SOCKETS` before starting Squirrel.

---

### **3. Anthropic Adapter Not Initializing**

**Problem**: No "✅ Anthropic adapter available" log appearing.

**Expected Flow**:
```
1. Anthrop Adapter::new() checks ANTHROPIC_API_KEY
2. Adapter::is_available() discovers http.request
3. If both succeed → adapter added to router
```

**Actual Flow**:
```
1. ANTHROPIC_API_KEY not set (env var issue)
2. Or http.request discovery times out
3. Adapter fails is_available() check
4. Not added to router
```

---

## 🔧 FIXES ATTEMPTED

### **Attempt 1: Deployment Script**
```bash
export ANTHROPIC_API_KEY="..."
export CAPABILITY_REGISTRY_SOCKET="/tmp/neural-api-nat0.sock"
/path/to/squirrel server &
```
**Result**: ❌ Variables not inherited by background process

### **Attempt 2: `env -i` Clean Environment**
```bash
env -i VAR=value /path/to/squirrel
```
**Result**: ❌ Stripped ALL variables (including PATH!)

### **Attempt 3: `nohup` with Exports**
```bash
ANTHROPIC_API_KEY="..." nohup /path/to/squirrel &
```
**Result**: ⏳ Not tested yet (should work!)

---

## ✅ RECOMMENDED SOLUTION

### **Option 1: Inline Environment (RECOMMENDED)**

```bash
#!/bin/bash
pkill -9 squirrel
sleep 2

# Pass variables inline with command
ANTHROPIC_API_KEY="sk-ant-api03-..." \
CAPABILITY_REGISTRY_SOCKET="/tmp/neural-api-nat0.sock" \
/home/eastgate/Development/ecoPrimals/plasmidBin/primals/squirrel/squirrel-x86_64 \
  server --socket /tmp/squirrel-nat0.sock \
  > /tmp/squirrel.log 2>&1 &

sleep 10

# Verify environment
cat /proc/$(pgrep squirrel)/environ | tr '\0' '\n' | grep ANTHROPIC
```

### **Option 2: Systemd Service (PRODUCTION)**

```ini
[Unit]
Description=Squirrel AI Orchestrator
After=network.target neural-api.service songbird.service

[Service]
Type=simple
User=eastgate
WorkingDirectory=/tmp
Environment="ANTHROPIC_API_KEY=sk-ant-..."
Environment="CAPABILITY_REGISTRY_SOCKET=/tmp/neural-api-nat0.sock"
ExecStart=/path/to/squirrel server --socket /tmp/squirrel-nat0.sock
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

---

## 🧪 MANUAL TESTING PROCEDURE

```bash
# 1. Start Neural API (if not running)
ps aux | grep neural-api || ./target/release/neural-api-server &

# 2. Start BearDog
export BEARDOG_FAMILY_ID="nat0"
/plasmidBin/primals/beardog/beardog-x86_64-musl server &
sleep 3

# 3. Start Songbird
export SONGBIRD_SECURITY_PROVIDER="/tmp/beardog-nat0.sock"
export SONGBIRD_FAMILY_ID="nat0"
/plasmidBin/primals/songbird/songbird-x86_64 server &
sleep 4

# 4. Verify Songbird registered
echo '{"jsonrpc":"2.0","method":"neural_api.discover_capability","params":{"capability":"http.request"},"id":1}' | \
  nc -N -U /tmp/neural-api-nat0.sock | jq '.result.primary_socket'
# Expected: "/tmp/songbird-nat0.sock"

# 5. Start Squirrel WITH INLINE VARS
pkill -9 squirrel
unset AI_PROVIDER_SOCKETS  # Critical!

ANTHROPIC_API_KEY="sk-ant-REDACTED" \
CAPABILITY_REGISTRY_SOCKET="/tmp/neural-api-nat0.sock" \
/plasmidBin/primals/squirrel/squirrel-x86_64 server --socket /tmp/squirrel-nat0.sock \
  > /tmp/squirrel.log 2>&1 &

sleep 10

# 6. Verify Squirrel environment
cat /proc/$(pgrep squirrel)/environ | tr '\0' '\n' | grep -E "ANTHROPIC|CAPABILITY"

# 7. Check logs
tail -30 /tmp/squirrel.log | grep -E "Anthropic adapter|AI router initialized"
# Expected: "✅ Anthropic adapter available"

# 8. Test AI query
echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"Hello!"},"id":1}' | \
  timeout 60 nc -N -U /tmp/squirrel-nat0.sock | jq '.'
```

---

## 📊 EXPECTED RESULTS

### **Squirrel Logs (Success)**
```
🔍 Discovering capability: http.request
✅ Found http.request via capability registry  ← <1ms!
✅ Anthropic adapter available (HTTP via capability discovery)
✅ OpenAI adapter available (HTTP via capability discovery)
✅ AI router initialized with 2 provider(s) via capability discovery
```

### **AI Query Response (Success)**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "text": "Hello! ...",
    "model": "claude-3-haiku-20240307",
    "usage": {"input_tokens": 10, "output_tokens": 50}
  },
  "id": 1
}
```

---

## 🎯 SUCCESS CRITERIA

- [ ] ANTHROPIC_API_KEY reaches Squirrel process
- [ ] CAPABILITY_REGISTRY_SOCKET reaches Squirrel process
- [ ] Discovery takes <1ms (event-driven, not 2s timeout)
- [ ] Anthropic adapter initializes
- [ ] OpenAI adapter initializes  
- [ ] AI router has 2+ providers
- [ ] `query_ai` RPC succeeds
- [ ] Real Anthropic API response received
- [ ] Total latency < API latency + 100ms

---

## 🔄 HANDOFF

### **For Next Session**
1. Use inline environment variable passing (Option 1 above)
2. Verify variables reach process with `/proc/PID/environ`
3. Check logs for "Anthropic adapter available"
4. Test end-to-end AI query
5. Measure latency

### **For Squirrel Team**
If environment variables are confirmed correct but adapters still don't initialize:
- Add debug logging to `AnthropicAdapter::new()`
- Add debug logging to `Anthropic Adapter::is_available()`
- Check if `discover_capability("http.request")` is succeeding
- Verify timeout values (should be >1ms for registry query)

---

## 📚 RELATED DOCUMENTATION

- `SONGBIRD_V4_REHARVEST_COMPLETE_JAN_20_2026.md` - Songbird RPC methods
- `SQUIRREL_EVENT_DRIVEN_DISCOVERY_FIX_JAN_20_2026.md` - Discovery fix details
- `SQUIRREL_ANTHROPIC_INTEGRATION_JAN_20_2026.md` - Two-tier architecture
- `SONGBIRD_SQUIRREL_INTEGRATION_COMPLETE_JAN_20_2026.md` - Integration overview

---

## 🎊 SUMMARY

**Infrastructure**: ✅ Ready  
**Binaries**: ✅ Harvested  
**Architecture**: ✅ Documented  
**Event-Driven Discovery**: ✅ Implemented  
**Environment Setup**: ⚠️ Needs fixing  

**Blocking Issue**: Environment variable passing to background processes.

**Estimated Fix Time**: 15-30 minutes

**The Pure Rust AI stack is ready - just needs proper environment configuration!** 🚀

---

*The ecological way: Configure once, discover instantly, route intelligently* 🧬🌍⚡

