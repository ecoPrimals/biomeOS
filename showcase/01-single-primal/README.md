# 01 - Single Primal Discovery & Integration

**Purpose:** Demonstrate BiomeOS discovering and interacting with ONE real primal at a time  
**Duration:** 10-15 minutes per primal  
**Philosophy:** Real integration - NO MOCKS - finding real gaps

---

## 🎯 What This Demonstrates

BiomeOS can discover and interact with each Phase 1 primal individually:

1. **Songbird** - Service discovery and mesh coordination
2. **ToadStool** - Compute orchestration and task execution
3. **NestGate** - Storage operations and volume management
4. **BearDog** - Cryptography and security operations
5. **Squirrel** - AI agent management and MCP protocol

---

## 📋 Prerequisites

### 1. Phase 1 Binaries Available

```bash
cd ../../phase1bins/
ls -lh *-bin

# Should see:
#   songbird-bin
#   toadstool-bin
#   nestgate-bin
#   beardog-bin
#   squirrel-bin
```

If binaries are missing:
```bash
cd ../../phase1bins/
./pull-phase1-bins.sh
```

### 2. BiomeOS Built

```bash
cd ../..
cargo build --release
```

### 3. Ports Available

Check these ports are free:
- **3000** - Songbird
- **8080** - ToadStool
- **8002** - NestGate
- **9000** - BearDog
- **8001** - Squirrel

```bash
./common/check-ports.sh
```

---

## 🚀 Quick Start

### Run All Single Primal Demos

```bash
./run-all-single-primal-demos.sh
```

### Run Individual Demos

```bash
./songbird-discovery.sh    # Service discovery
./toadstool-compute.sh     # Compute orchestration
./nestgate-storage.sh      # Storage operations
./beardog-security.sh      # Crypto operations
./squirrel-ai.sh           # AI capabilities
```

---

## 📊 Demo Details

### Songbird Discovery Demo
**File:** `songbird-discovery.sh`  
**Duration:** ~3 minutes  
**What It Does:**
1. Start real Songbird binary
2. BiomeOS discovers via capability query
3. Register a test service
4. Query registered services
5. Monitor service health
6. Clean shutdown

**Expected Output:**
- Songbird starts on port 3000
- BiomeOS discovers endpoint
- Service registration succeeds
- Health checks pass
- Clean shutdown with no errors

**Gaps to Document:**
- [ ] Discovery timing issues
- [ ] Health check edge cases
- [ ] Service registration failures

---

### ToadStool Compute Demo
**File:** `toadstool-compute.sh`  
**Duration:** ~3 minutes  
**What It Does:**
1. Start real ToadStool binary
2. BiomeOS discovers compute capability
3. Submit simple compute task
4. Monitor task execution
5. Retrieve results
6. Clean shutdown

**Expected Output:**
- ToadStool starts on port 8080
- BiomeOS discovers compute endpoint
- Task submission succeeds
- Task completes successfully
- Results retrieved

**Gaps to Document:**
- [ ] Task submission edge cases
- [ ] Result retrieval timing
- [ ] Error handling

---

### NestGate Storage Demo
**File:** `nestgate-storage.sh`  
**Duration:** ~3 minutes  
**What It Does:**
1. Start real NestGate binary
2. BiomeOS discovers storage capability
3. Create test volume
4. Store test data
5. Retrieve and verify data
6. Clean shutdown

**Expected Output:**
- NestGate starts on port 8002
- BiomeOS discovers storage endpoint
- Volume creation succeeds
- Data store/retrieve works
- Data integrity verified

**Gaps to Document:**
- [ ] Volume creation edge cases
- [ ] Data integrity issues
- [ ] Storage quota handling

---

### BearDog Security Demo
**File:** `beardog-security.sh`  
**Duration:** ~3 minutes  
**What It Does:**
1. Start real BearDog binary
2. BiomeOS discovers security capability
3. Encrypt test data
4. Decrypt and verify
5. Test signature verification
6. Clean shutdown

**Expected Output:**
- BearDog starts on port 9000
- BiomeOS discovers crypto endpoint
- Encryption succeeds
- Decryption succeeds
- Signature verification works

**Gaps to Document:**
- [ ] Key management edge cases
- [ ] Encryption algorithm support
- [ ] Performance issues

---

### Squirrel AI Demo
**File:** `squirrel-ai.sh`  
**Duration:** ~3 minutes  
**What It Does:**
1. Start real Squirrel binary
2. BiomeOS discovers AI capability
3. Create test AI agent
4. Execute simple tool
5. Verify MCP protocol
6. Clean shutdown

**Expected Output:**
- Squirrel starts on port 8001
- BiomeOS discovers AI endpoint
- Agent creation succeeds
- Tool execution works
- MCP protocol functioning

**Gaps to Document:**
- [ ] Agent lifecycle issues
- [ ] MCP protocol edge cases
- [ ] Tool execution failures

---

## 🔍 Gap Discovery Process

As we run these demos, we document:

### 1. Discovery Gaps
- How long does discovery take?
- Does it work on first try?
- What fails in discovery?

### 2. Integration Gaps
- Do APIs match expectations?
- Are there authentication issues?
- Do timeouts work correctly?

### 3. Error Handling Gaps
- What happens when primal crashes?
- How are network errors handled?
- Are error messages clear?

### 4. Documentation Gaps
- What's unclear from primal docs?
- What assumptions are wrong?
- What's missing from specs?

---

## 📝 Gap Documentation Template

Each demo creates a gap report:

```markdown
# Gaps Found: [Primal Name]

## Discovery Issues
- [ ] Issue 1: Description
- [ ] Issue 2: Description

## Integration Issues
- [ ] Issue 1: Description
- [ ] Issue 2: Description

## Error Handling Issues
- [ ] Issue 1: Description
- [ ] Issue 2: Description

## Documentation Issues
- [ ] Issue 1: Description
- [ ] Issue 2: Description

## Follow-Up Actions
1. Action 1
2. Action 2
```

---

## 🎓 What We Learn

From each demo, we learn:

1. **What Works**
   - Successful integration patterns
   - Good API designs
   - Clear documentation

2. **What Doesn't Work**
   - Failed integration attempts
   - Confusing APIs
   - Missing documentation

3. **What's Unclear**
   - Ambiguous behavior
   - Undocumented features
   - Edge cases

4. **What to Improve**
   - BiomeOS adapter improvements
   - Primal API suggestions
   - Documentation updates

---

## 🚧 Common Issues & Solutions

### Primal Won't Start
```bash
# Check if already running
ps aux | grep [primal-name]

# Check port availability
netstat -tulpn | grep [port]

# Check binary permissions
ls -l ../../phase1bins/[primal]-bin

# Check for error logs
./common/check-primal-logs.sh [primal-name]
```

### Discovery Fails
```bash
# Verify primal is running
curl http://localhost:[port]/health

# Check BiomeOS discovery config
cat ~/.biomeos/config.toml

# Try explicit endpoint
export [PRIMAL]_ENDPOINT="http://localhost:[port]"
./[primal]-discovery.sh
```

### Connection Timeouts
```bash
# Increase timeout
export BIOMEOS_TIMEOUT=60

# Check network
ping localhost

# Check firewall
sudo ufw status
```

---

## 📊 Success Metrics

For each demo, we track:

- ✅ **Discovery Time**: How long to find primal
- ✅ **Connection Success**: Did connection establish?
- ✅ **Operation Success**: Did test operations work?
- ✅ **Error Handling**: How were errors handled?
- ✅ **Clean Shutdown**: Did shutdown work cleanly?

---

## 🎯 Next Steps

After completing single-primal demos:

1. **Document all gaps** found
2. **Move to 02-multi-primal** for cross-primal workflows
3. **Create issues** for significant gaps
4. **Update adapters** based on findings

---

**Remember:** We're using REAL primals to find REAL gaps. Document everything!
