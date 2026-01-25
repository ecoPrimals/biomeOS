# 🧪 Tower Atomic Integration Testing Guide

**Date**: January 25, 2026  
**Status**: ✅ **READY FOR TESTING** - Both components complete  
**Purpose**: Integration testing guide for next session

---

## 🎯 **CURRENT STATUS**

### ✅ **BOTH SIDES READY**

**Songbird** (v5.28.0):
- ✅ HTTP IPC implemented
- ✅ `http.request` method working
- ✅ `secure_http` capability registered
- ✅ Pure Rust TLS 1.3
- ✅ Grade A quality
- ✅ Built and ready (`24M binary`)

**biomeOS**:
- ✅ Neural API `proxy_http` method
- ✅ Capability discovery working
- ✅ Routing infrastructure complete
- ✅ A+ verification grade
- ✅ Built and ready (multiple binaries)

---

## 📋 **INTEGRATION TESTING CHECKLIST**

### Pre-Test Setup (5 minutes)
```bash
# Terminal 1: Start Songbird
cd /home/eastgate/Development/ecoPrimals/phase1/songbird
export RUST_LOG=info
export SONGBIRD_SOCKET_PATH="/run/user/$(id -u)/songbird-nat0.sock"
./target/release/songbird

# Terminal 2: Start biomeOS Neural API
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
export RUST_LOG=info
export BIOMEOS_SOCKET_PATH="/run/user/$(id -u)/neural-api-nat0.sock"
cargo run --release -p biomeos neural-api

# Terminal 3: Test client
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
# Ready for testing
```

---

### Test 1: Songbird Direct (5 minutes)
**Goal**: Verify Songbird http.request works

```bash
# Test Songbird directly via Unix socket
cat <<EOF | nc -U /run/user/$(id -u)/songbird-nat0.sock
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "http.request",
  "params": {
    "url": "https://api.github.com/zen",
    "method": "GET",
    "headers": {
      "User-Agent": "ecoPrimals-Test/1.0"
    }
  }
}
EOF
```

**Expected Result**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "status_code": 200,
    "body": "<zen quote>",
    "elapsed_ms": <time>
  }
}
```

**Success Criteria**:
- ✅ Response received
- ✅ status_code: 200
- ✅ body contains zen quote
- ✅ No errors

---

### Test 2: Neural API Discovery (5 minutes)
**Goal**: Verify Neural API can discover Songbird

```bash
# Test capability discovery
cat <<EOF | nc -U /run/user/$(id -u)/neural-api-nat0.sock
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "neural_api.discover",
  "params": {
    "capability": "secure_http"
  }
}
EOF
```

**Expected Result**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "capability": "secure_http",
    "primals": [
      {
        "name": "songbird",
        "socket": "/run/user/1000/songbird-nat0.sock"
      }
    ]
  }
}
```

**Success Criteria**:
- ✅ Songbird discovered
- ✅ Socket path correct
- ✅ Capability matches

---

### Test 3: Neural API Proxy (10 minutes)
**Goal**: Verify Neural API → Songbird → GitHub works

```bash
# Test via Neural API proxy_http
cat <<EOF | nc -U /run/user/$(id -u)/neural-api-nat0.sock
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "neural_api.proxy_http",
  "params": {
    "method": "GET",
    "url": "https://api.github.com/zen",
    "headers": {
      "User-Agent": "ecoPrimals-Neural-API/1.0"
    }
  }
}
EOF
```

**Expected Result**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "status_code": 200,
    "body": "<zen quote>",
    "elapsed_ms": <time>
  }
}
```

**Success Criteria**:
- ✅ Neural API discovers Songbird
- ✅ Request forwarded successfully
- ✅ GitHub responds
- ✅ Response returned through chain

---

### Test 4: GitHub API Endpoints (15 minutes)
**Goal**: Test various GitHub API calls

```bash
# Test 1: Get repository info
cat <<EOF | nc -U /run/user/$(id -u)/neural-api-nat0.sock
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "neural_api.proxy_http",
  "params": {
    "method": "GET",
    "url": "https://api.github.com/repos/ecoPrimals/biomeOS",
    "headers": {
      "User-Agent": "ecoPrimals/1.0"
    }
  }
}
EOF

# Test 2: Get user info
cat <<EOF | nc -U /run/user/$(id -u)/neural-api-nat0.sock
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "neural_api.proxy_http",
  "params": {
    "method": "GET",
    "url": "https://api.github.com/users/ecoPrimals",
    "headers": {
      "User-Agent": "ecoPrimals/1.0"
    }
  }
}
EOF

# Test 3: List commits
cat <<EOF | nc -U /run/user/$(id -u)/neural-api-nat0.sock
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "neural_api.proxy_http",
  "params": {
    "method": "GET",
    "url": "https://api.github.com/repos/ecoPrimals/biomeOS/commits",
    "headers": {
      "User-Agent": "ecoPrimals/1.0"
    }
  }
}
EOF
```

**Success Criteria**:
- ✅ All requests succeed
- ✅ Proper JSON responses
- ✅ GitHub rate limit not hit
- ✅ Consistent performance

---

## 🎯 **SUCCESS CRITERIA**

### Minimum (Must Pass)
1. ✅ Songbird responds to http.request
2. ✅ Neural API discovers Songbird
3. ✅ At least one GitHub API call succeeds

### Complete (All Pass)
1. ✅ All 4 test phases pass
2. ✅ Multiple GitHub endpoints work
3. ✅ Performance acceptable (<1s)
4. ✅ No errors in logs

---

## 📊 **EXPECTED RESULTS**

### Timeline
- **Test Duration**: 30-45 minutes
- **Setup**: 5 minutes
- **Testing**: 25-35 minutes
- **Analysis**: 5 minutes

### Performance
- **Latency**: <1s per request
- **Success Rate**: 100%
- **Error Rate**: 0%

---

## 🐛 **TROUBLESHOOTING**

### Songbird Not Starting
```bash
# Check if socket exists
ls -la /run/user/$(id -u)/songbird-nat0.sock

# Check logs
tail -f /tmp/songbird.log

# Check if port conflicts
lsof -i :8443
```

### Neural API Not Discovering
```bash
# Check socket scanning
export RUST_LOG=debug
# Restart Neural API
# Check discovery logs
```

### GitHub API Fails
```bash
# Test direct connection
curl -v https://api.github.com/zen

# Check DNS
nslookup api.github.com

# Check TLS
openssl s_client -connect api.github.com:443
```

---

## 📝 **TEST REPORT TEMPLATE**

After testing, document:

```markdown
# Tower Atomic Integration Test Report

**Date**: [date]
**Tester**: [name]
**Duration**: [time]

## Results

### Test 1: Songbird Direct
- Status: [✅/❌]
- Response time: [ms]
- Notes: [observations]

### Test 2: Neural API Discovery
- Status: [✅/❌]
- Discovered: [yes/no]
- Notes: [observations]

### Test 3: Neural API Proxy
- Status: [✅/❌]
- Response time: [ms]
- Notes: [observations]

### Test 4: GitHub API Endpoints
- Status: [✅/❌]
- Requests: [success/total]
- Notes: [observations]

## Overall
- Grade: [A+/A/B/C/F]
- Recommendation: [proceed/fix/investigate]
- Blockers: [list any issues]

## Next Steps
1. [action item]
2. [action item]
```

---

## 🚀 **NEXT STEPS AFTER SUCCESS**

### Immediate
1. ✅ Document success
2. ✅ Create test report
3. ✅ Commit integration tests

### Short Term
1. ⏳ Add automated integration tests
2. ⏳ Set up monitoring
3. ⏳ Create deployment guide

### Medium Term
1. ⏳ Tower Atomic graph deployment
2. ⏳ Production hardening
3. ⏳ Performance optimization

---

## 📚 **REFERENCE DOCUMENTS**

### biomeOS
- `archive/session_jan_25_2026_final/SONGBIRD_IPC_HANDOFF_JAN_25_2026.md`
- `archive/session_jan_25_2026_final/NEURAL_API_HTTP_EVOLUTION_JAN_25_2026.md`

### Songbird
- `SONGBIRD_HTTP_IPC_HANDOFF_COMPLETE_JAN_25_2026.md`
- `crates/songbird-universal-ipc/src/handlers/http_handler.rs`

---

**🦀✨ Ready for Integration Testing | Both Sides Complete | Success Expected ✨🦀**

**Status**: ✅ **READY** - Schedule full integration test session  
**Risk**: ✅ **LOW** - Both teams delivered Grade A work  
**Timeline**: ✅ **30-45 minutes** for complete testing

