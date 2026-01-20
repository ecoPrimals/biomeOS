# Manual Deployment Guide - Tower Atomic + Squirrel

**Date**: January 20, 2026  
**Status**: Ready for deployment and validation  
**Goal**: Deploy Tower Atomic + Squirrel and make API calls to Anthropic

---

## Quick Deployment

### Prerequisites
```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
export ANTHROPIC_API_KEY="sk-ant-REDACTED"
```

### Method 1: Python Script (Recommended)
```bash
chmod +x scripts/deploy.py
python3 scripts/deploy.py nat0
```

### Method 2: Bash Script
```bash
chmod +x scripts/deploy_tower_squirrel_simple.sh
./scripts/deploy_tower_squirrel_simple.sh nat0
```

### Method 3: Manual Step-by-Step

```bash
# Clean up
pkill -f "beardog.*nat0" 2>/dev/null || true
pkill -f "songbird.*nat0" 2>/dev/null || true
pkill -f "squirrel.*nat0" 2>/dev/null || true
sleep 1
rm -f /tmp/beardog-nat0.sock /tmp/songbird-nat0.sock /tmp/squirrel-nat0.sock

# Phase 1: Start BearDog
./plasmidBin/primals/beardog/beardog-x86_64-musl server \
  --socket /tmp/beardog-nat0.sock \
  --family-id nat0 \
  > /tmp/beardog-nat0.log 2>&1 &

# Wait for BearDog (2 seconds)
sleep 2

# Verify BearDog socket
ls -lh /tmp/beardog-nat0.sock
# Should show: srwxrwxr-x ... /tmp/beardog-nat0.sock

# Phase 2: Start Songbird (bonded to BearDog)
env SONGBIRD_SOCKET="/tmp/songbird-nat0.sock" \
    SONGBIRD_SECURITY_PROVIDER="/tmp/beardog-nat0.sock" \
    SONGBIRD_ORCHESTRATOR_FAMILY_ID="nat0" \
  ./plasmidBin/primals/songbird/songbird-x86_64-musl server \
  > /tmp/songbird-nat0.log 2>&1 &

# Wait for Songbird (2 seconds)
sleep 2

# Verify Songbird socket
ls -lh /tmp/songbird-nat0.sock
# Should show: srwxrwxr-x ... /tmp/songbird-nat0.sock

# Phase 3: Start Squirrel (inherits from Tower)
env SQUIRREL_SOCKET="/tmp/squirrel-nat0.sock" \
    SONGBIRD_ENDPOINT="/tmp/songbird-nat0.sock" \
    ANTHROPIC_API_KEY="$ANTHROPIC_API_KEY" \
  ./plasmidBin/primals/squirrel/squirrel-x86_64-musl server \
  > /tmp/squirrel-nat0.log 2>&1 &

# Wait for Squirrel (2 seconds)
sleep 2

# Verify Squirrel socket
ls -lh /tmp/squirrel-nat0.sock
# Should show: srwxrwxr-x ... /tmp/squirrel-nat0.sock
```

---

## Validation

### Check All Sockets
```bash
ls -lh /tmp/*-nat0.sock
```

**Expected output**:
```
srwxrwxr-x ... /tmp/beardog-nat0.sock
srwxrwxr-x ... /tmp/songbird-nat0.sock
srwxrwxr-x ... /tmp/squirrel-nat0.sock
```

### Check Processes
```bash
ps aux | grep -E "(beardog|songbird|squirrel)" | grep nat0 | grep -v grep
```

**Expected output**: 3 processes running

### Check Logs
```bash
# BearDog
tail -20 /tmp/beardog-nat0.log

# Songbird
tail -20 /tmp/songbird-nat0.log

# Squirrel
tail -20 /tmp/squirrel-nat0.log
```

**What to look for**:
- BearDog: Should show "Server started" or similar
- Songbird: Should show security provider connected, socket ready
- Squirrel: Should show "Squirrel AI/MCP Primal Ready!"

---

## Test API Calls

### Test 1: Simple Chat
```bash
echo '{"jsonrpc":"2.0","method":"ai.chat","params":{"messages":[{"role":"user","content":"Hello! Please respond with just one sentence."}]},"id":1}' | nc -U /tmp/squirrel-nat0.sock
```

**Expected**: JSON-RPC response with Anthropic's reply

### Test 2: Capability Check
```bash
echo '{"jsonrpc":"2.0","method":"ai.capabilities","params":{},"id":2}' | nc -U /tmp/squirrel-nat0.sock
```

**Expected**: List of available AI capabilities

### Test 3: Health Check
```bash
echo '{"jsonrpc":"2.0","method":"health","params":{},"id":3}' | nc -U /tmp/squirrel-nat0.sock
```

**Expected**: Health status of Squirrel

---

## Troubleshooting

### BearDog Socket Missing
```bash
tail -50 /tmp/beardog-nat0.log
```

**Common issues**:
- Permission denied: Check file permissions
- Address already in use: Run cleanup first
- Missing family ID: Verify `--family-id nat0` flag

### Songbird Socket Missing
```bash
tail -50 /tmp/songbird-nat0.log
```

**Common issues**:
- "No security provider configured": Check that `SONGBIRD_SECURITY_PROVIDER=/tmp/beardog-nat0.sock` is set
- BearDog socket not found: Ensure BearDog started first
- Permission denied: Check socket permissions

### Squirrel Socket Missing
```bash
tail -50 /tmp/squirrel-nat0.log
```

**Common issues**:
- Songbird not found: Ensure Songbird started first
- API key missing: Check `ANTHROPIC_API_KEY` is set
- Socket path wrong: Verify `SQUIRREL_SOCKET` environment variable

### API Call Fails
```bash
# Check if Squirrel socket exists
ls -lh /tmp/squirrel-nat0.sock

# Check if nc can connect
echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' | nc -U /tmp/squirrel-nat0.sock

# Check Squirrel logs for errors
tail -50 /tmp/squirrel-nat0.log

# Check Songbird logs for HTTP errors
tail -50 /tmp/songbird-nat0.log
```

---

## Cleanup

### Stop All Primals
```bash
pkill -f "beardog.*nat0"
pkill -f "songbird.*nat0"
pkill -f "squirrel.*nat0"
```

### Remove Sockets
```bash
rm -f /tmp/beardog-nat0.sock /tmp/songbird-nat0.sock /tmp/squirrel-nat0.sock
```

### Archive Logs
```bash
mkdir -p logs/$(date +%Y%m%d-%H%M%S)
mv /tmp/beardog-nat0.log /tmp/songbird-nat0.log /tmp/squirrel-nat0.sock logs/$(date +%Y%m%d-%H%M%S)/
```

---

## Architecture

### Tower Atomic (Covalent Bonding)
```
BearDog  → Songbird
(Security)  (Communications)
     ↓          ↓
  Family ID shared
  Genetic lineage
```

### AI Orchestration (Genetic Inheritance)
```
Squirrel
   ↓
Inherits from Tower Atomic:
- Family ID: nat0
- Security context from BearDog
- Communications via Songbird
```

### Communication Flow
```
User → Squirrel (Unix Socket, JSON-RPC)
     ↓
Squirrel → Songbird (Unix Socket, JSON-RPC, requests HTTPS)
     ↓
Songbird → Anthropic API (HTTPS)
     ↓
Anthropic API → Response
     ↓
Songbird → Squirrel
     ↓
Squirrel → User
```

---

## Success Criteria

✅ **Deployment Complete**:
- All 3 sockets exist in `/tmp/`
- All 3 processes running
- Logs show successful startup

✅ **API Validation**:
- Simple chat request returns valid response from Anthropic
- Response contains actual AI-generated content
- Round-trip time < 5 seconds

✅ **Architecture Validation**:
- BearDog provides security foundation
- Songbird handles HTTPS to Anthropic
- Squirrel orchestrates AI requests
- All communication via Unix sockets (zero HTTP internally)

---

## Next Steps

Once deployed and validated:

1. **Test Complex AI Requests**: Multi-turn conversations, longer prompts
2. **Performance Testing**: Concurrent requests, latency measurements
3. **Error Handling**: Network failures, invalid requests, rate limiting
4. **Monitoring**: Metrics collection, health checks, resource usage
5. **Evolution**: Deploy via Neural API graphs (once DAG system evolved)

---

## Files Created/Modified

**Deployment Scripts**:
- `scripts/deploy.py` - Python-based deployment (recommended)
- `scripts/deploy_tower_squirrel_simple.sh` - Bash-based deployment
- `scripts/deploy_tower_squirrel_manual.sh` - Original bash script (needs update)

**Documentation**:
- `TOWER_ATOMIC_READY_JAN_20_2026.md` - Quick start guide
- `DEPLOYMENT_SYSTEM_EVOLUTION_PLAN_JAN_20_2026.md` - 6-week roadmap
- This file - Comprehensive manual deployment guide

---

## Terminal Issues Note

If experiencing shell corruption errors like:
```
--: eval: line 7: unexpected EOF while looking for matching `)'
```

**Solutions**:
1. Open a **new terminal window/tab**
2. Use the Python script (`scripts/deploy.py`) instead of bash
3. Run commands manually one at a time
4. Restart Cursor IDE if issue persists

---

**Ready to deploy!** 🚀

Choose Method 1 (Python script) for most reliable deployment.


