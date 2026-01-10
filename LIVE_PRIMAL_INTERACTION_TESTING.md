# 🧪 Live Primal Interaction Testing Plan

**Date**: January 10, 2026  
**Session**: 18+ Hour Epic  
**Status**: ⏳ **IN PROGRESS**

---

## 📊 **CURRENT STATUS**

### **Primals Available for Testing:**

| Primal | Binary | Server | Socket | Status |
|--------|--------|--------|--------|--------|
| Songbird | ✅ 28MB | ✅ Unix Socket | `/run/user/{uid}/songbird-{family}.sock` | READY |
| BearDog | ✅ 4.5MB | ✅ Unix Socket | `/run/user/{uid}/beardog-{family}.sock` | READY |
| Squirrel | ✅ 15MB | ✅ Unix Socket | `/run/user/{uid}/squirrel-{family}.sock` | READY |
| NestGate | ✅ 4.3MB | ✅ Unix Socket | `/run/user/{uid}/nestgate-{family}.sock` | READY |
| ToadStool | ✅ 22MB | ⚠️ TCP | `127.0.0.1:9944` (hardcoded) | NEEDS FIX |
| petalTongue | ✅ 21MB | ⚠️ Wiring | Needs Songbird discovery | NEEDS WIRING |

**Ready for Testing**: **4/6** (Songbird, BearDog, Squirrel, NestGate)

### **Currently Running:**
- ❌ No primals started yet (no Unix sockets found)
- ⚠️ songbird-bin stuck on `--help` (PID 190723)
- ✅ petalTongue GUI running (PID 189717) - waiting for primals

---

## 🎯 **TEST PLAN: LIVE PRIMAL INTERACTIONS**

### **Phase 1: Start Core Primals** (IMMEDIATE)

**Goal**: Get 4 primals running with Unix sockets

```bash
# Set family ID for all primals
export FAMILY_ID=nat0

# Terminal 1: Start Songbird (discovery)
export SONGBIRD_FAMILY_ID=$FAMILY_ID
./bin/primals/songbird &

# Terminal 2: Start NestGate (storage)
export NESTGATE_FAMILY_ID=$FAMILY_ID
export SONGBIRD_FAMILY_ID=$FAMILY_ID
./bin/primals/nestgate &

# Terminal 3: Start Squirrel (AI)
export SQUIRREL_FAMILY_ID=$FAMILY_ID
export SONGBIRD_FAMILY_ID=$FAMILY_ID
./bin/primals/squirrel &

# Terminal 4: Start BearDog (security)
export BEARDOG_FAMILY_ID=$FAMILY_ID
export SONGBIRD_FAMILY_ID=$FAMILY_ID
./bin/primals/beardog &

# Verify all sockets exist
ls -la /run/user/$(id -u)/*-${FAMILY_ID}.sock
```

### **Phase 2: Test Songbird Discovery** (5 min)

**Goal**: Verify all primals register with Songbird

```bash
# Query Songbird for all registered services
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "discover_by_capability",
    "params": {"capability": "storage"},
    "id": 1
  }'

# Expected: NestGate appears
# Expected: Capabilities: storage, persistence, key-value, blob-storage

# Query for compute capability
# Expected: Would find ToadStool (when Unix socket fixed)

# Query for security
# Expected: BearDog appears

# Query for AI
# Expected: Squirrel appears
```

### **Phase 3: Test NestGate Storage** (10 min)

**Goal**: Verify NestGate storage methods work

```bash
# Connect via Unix socket
socat - UNIX-CONNECT:/run/user/$(id -u)/nestgate-${FAMILY_ID}.sock

# Send JSON-RPC store request
{"jsonrpc":"2.0","method":"storage.store","params":{"key":"test:user","data":{"name":"Alice"}},"id":1}

# Send retrieve request
{"jsonrpc":"2.0","method":"storage.retrieve","params":{"key":"test:user"},"id":2}

# Send list request
{"jsonrpc":"2.0","method":"storage.list","params":{"prefix":"test:"},"id":3}

# Send stats request
{"jsonrpc":"2.0","method":"storage.stats","params":{},"id":4}
```

### **Phase 4: Test Squirrel AI** (10 min)

**Goal**: Verify Squirrel AI methods work

```bash
# Connect via Unix socket
socat - UNIX-CONNECT:/run/user/$(id -u)/squirrel-${FAMILY_ID}.sock

# Send inference request
{"jsonrpc":"2.0","method":"infer","params":{"model":"default","prompt":"Hello, what is 2+2?"},"id":1}

# Send pattern detection
{"jsonrpc":"2.0","method":"detect_patterns","params":{"data":[1,2,3,4,5]},"id":2}
```

### **Phase 5: Test BearDog Security** (10 min)

**Goal**: Verify BearDog crypto methods work

```bash
# Connect via Unix socket
socat - UNIX-CONNECT:/run/user/$(id -u)/beardog-${FAMILY_ID}.sock

# Generate keys
{"jsonrpc":"2.0","method":"generate_keypair","params":{"key_type":"ed25519"},"id":1}

# Encrypt data
{"jsonrpc":"2.0","method":"encrypt","params":{"data":"sensitive","algorithm":"aes256"},"id":2}
```

### **Phase 6: Test Inter-Primal Coordination** (15 min)

**Goal**: Test primals working together

```rust
// Rust integration test
#[tokio::test]
async fn test_full_ecosystem_coordination() {
    // 1. Discover all primals via Songbird
    let songbird = SongbirdClient::discover("nat0").await?;
    let primals = songbird.discover_by_capability("*").await?;
    
    // 2. Store encrypted data (BearDog + NestGate)
    let beardog = BearDogClient::discover("nat0").await?;
    let nestgate = NestGateClient::discover("nat0").await?;
    
    let encrypted = beardog.encrypt(b"secret data").await?;
    nestgate.store_blob("encrypted:data", &encrypted).await?;
    
    // 3. Retrieve and decrypt
    let encrypted_back = nestgate.retrieve_blob("encrypted:data").await?;
    let decrypted = beardog.decrypt(&encrypted_back).await?;
    
    assert_eq!(decrypted, b"secret data");
    
    // 4. Ask Squirrel for analysis
    let squirrel = SquirrelClient::discover("nat0").await?;
    let analysis = squirrel.analyze_system_optimization().await?;
    
    // 5. Store analysis results
    nestgate.store("analysis:results", &analysis).await?;
}
```

---

## 🏗️ **NICHE SETUP: BASE NICHES**

### **What Are Niches?**

**Niches** are **biomes** - complete environments where primals and chimeras operate together.

**Existing Niche Definitions**:
- ✅ `niches/nest.toml` - Storage/persistence niche (NestGate-focused)
- ✅ `niches/tower.toml` - Compute/execution niche (ToadStool-focused)
- ✅ `niches/compute-node.toml` - Distributed compute node

### **Nest Niche** (Storage/Persistence)

**Purpose**: Storage and persistence layer

**Primals Required**:
- ✅ NestGate (storage) - READY
- ✅ Songbird (discovery) - READY
- ✅ BearDog (encryption) - READY

**Setup**:
```bash
# Create nest niche instance
biomeos niche create nest --family nat0

# Verify configuration
biomeos niche status nest

# Deploy (starts all required primals)
biomeos niche deploy nest
```

**Use Cases**:
- Persistent data storage
- Encrypted blob storage
- Key-value operations
- Multi-tenant isolation

### **Tower Niche** (Compute/Execution)

**Purpose**: Compute execution and resource management

**Primals Required**:
- ⚠️ ToadStool (compute) - NEEDS UNIX SOCKET FIX
- ✅ Songbird (orchestration) - READY
- ✅ Squirrel (AI optimization) - READY

**Setup** (BLOCKED):
```bash
# BLOCKED: ToadStool needs Unix socket support
# Once fixed:
biomeos niche create tower --family nat0
biomeos niche deploy tower
```

**Use Cases**:
- Distributed compute workloads
- GPU resource pooling
- Container execution
- AI model inference

### **Node Niche** (Networking/Communication)

**Purpose**: Networking and communication layer

**Primals Required**:
- ✅ Songbird (coordination) - READY
- ✅ BearDog (secure tunnels) - READY
- ✅ NestGate (message persistence) - READY

**Setup**:
```bash
# Create node niche instance
biomeos niche create node --family nat0

# Deploy (starts all required primals)
biomeos niche deploy node
```

**Use Cases**:
- Peer-to-peer networking
- Secure BTSP tunnels
- Message routing
- Service coordination

---

## 🧪 **INTEGRATION TEST SUITE**

### **Test 1: Songbird Discovery**
```rust
#[tokio::test]
async fn test_songbird_discovers_all_primals() {
    let songbird = SongbirdClient::discover("nat0").await?;
    
    // Should find all 4 operational primals
    let primals = songbird.discover_by_capability("*").await?;
    assert!(primals.len() >= 4); // Songbird, BearDog, Squirrel, NestGate
    
    // Check specific capabilities
    let storage = songbird.discover_by_capability("storage").await?;
    assert_eq!(storage[0].name, "nestgate");
    
    let ai = songbird.discover_by_capability("ai").await?;
    assert_eq!(ai[0].name, "squirrel");
    
    let security = songbird.discover_by_capability("security").await?;
    assert_eq!(security[0].name, "beardog");
}
```

### **Test 2: NestGate + BearDog Integration**
```rust
#[tokio::test]
async fn test_encrypted_storage_workflow() {
    let beardog = BearDogClient::discover("nat0").await?;
    let nestgate = NestGateClient::discover("nat0").await?;
    
    // Encrypt sensitive data
    let plaintext = b"confidential information";
    let encrypted = beardog.encrypt(plaintext).await?;
    
    // Store encrypted data
    nestgate.store_blob("sensitive:doc", &encrypted.data).await?;
    
    // Retrieve and decrypt
    let stored = nestgate.retrieve_blob("sensitive:doc").await?;
    let decrypted = beardog.decrypt(&stored).await?;
    
    assert_eq!(decrypted, plaintext);
}
```

### **Test 3: Squirrel + NestGate Integration**
```rust
#[tokio::test]
async fn test_ai_analysis_persistence() {
    let squirrel = SquirrelClient::discover("nat0").await?;
    let nestgate = NestGateClient::discover("nat0").await?;
    
    // Get AI analysis
    let analysis = squirrel.analyze_system_optimization().await?;
    
    // Store analysis results
    nestgate.store("analysis:latest", &analysis).await?;
    
    // Retrieve and verify
    let stored_analysis = nestgate.retrieve("analysis:latest").await?;
    assert_eq!(stored_analysis, analysis);
}
```

### **Test 4: Full Ecosystem Workflow**
```rust
#[tokio::test]
async fn test_full_4_primal_workflow() {
    // 1. Discover all via Songbird
    let songbird = SongbirdClient::discover("nat0").await?;
    let primals = songbird.discover_by_capability("*").await?;
    assert!(primals.len() >= 4);
    
    // 2. Get clients for all primals
    let beardog = BearDogClient::discover("nat0").await?;
    let nestgate = NestGateClient::discover("nat0").await?;
    let squirrel = SquirrelClient::discover("nat0").await?;
    
    // 3. Encrypt user data
    let user_data = json!({"name": "Alice", "role": "admin"});
    let encrypted = beardog.encrypt(serde_json::to_vec(&user_data)?).await?;
    
    // 4. Store encrypted data
    nestgate.store_blob("users:alice", &encrypted.data).await?;
    
    // 5. Ask AI for recommendations
    let recommendation = squirrel.infer(
        "default",
        "What security measures for admin users?"
    ).await?;
    
    // 6. Store recommendation
    nestgate.store("recommendations:alice", &recommendation).await?;
    
    // 7. Verify all data persists
    assert!(nestgate.retrieve_blob("users:alice").await.is_ok());
    assert!(nestgate.retrieve("recommendations:alice").await.is_ok());
}
```

---

## 📋 **EXECUTION PLAN**

### **Step 1: Kill Stuck Processes** (IMMEDIATE)
```bash
# Kill stuck songbird-bin --help
kill 190723

# Verify clean state
ps aux | grep -E "(songbird|beardog|squirrel|nestgate)" | grep -v grep
```

### **Step 2: Start Core 4 Primals** (5 min)
```bash
# Start in separate terminals
export FAMILY_ID=nat0

# Terminal 1: Songbird
export SONGBIRD_FAMILY_ID=$FAMILY_ID
./bin/primals/songbird

# Terminal 2: NestGate  
export NESTGATE_FAMILY_ID=$FAMILY_ID
export SONGBIRD_FAMILY_ID=$FAMILY_ID
./bin/primals/nestgate

# Terminal 3: Squirrel
export SQUIRREL_FAMILY_ID=$FAMILY_ID
export SONGBIRD_FAMILY_ID=$FAMILY_ID
./bin/primals/squirrel

# Terminal 4: BearDog
export BEARDOG_FAMILY_ID=$FAMILY_ID
export SONGBIRD_FAMILY_ID=$FAMILY_ID
./bin/primals/beardog
```

### **Step 3: Verify All Running** (2 min)
```bash
# Check sockets exist
ls -la /run/user/$(id -u)/*-nat0.sock

# Check processes
ps aux | grep -E "(songbird|beardog|squirrel|nestgate)" | grep -v grep

# Test Songbird discovery
# (use method from Phase 2)
```

### **Step 4: Run Integration Tests** (10 min)
```bash
# Run biomeOS integration test suite
cargo test --package biomeos-core --features live --test integration_tests

# Run specific primal tests
cargo test --package biomeos-core songbird_integration --features live
cargo test --package biomeos-core nestgate_integration --features live
cargo test --package biomeos-core squirrel_integration --features live
cargo test --package biomeos-core beardog_integration --features live
```

### **Step 5: Setup Nest Niche** (10 min)
```bash
# Create and deploy nest niche
biomeos niche create nest --family nat0
biomeos niche status nest
biomeos niche deploy nest

# Verify niche is operational
biomeos niche health nest
```

### **Step 6: petalTongue Live Discovery** (5 min)
```bash
# petalTongue should now discover all 4 primals via Songbird
# Check the GUI (already running) - should show live ecosystem!
```

---

## ✅ **SUCCESS CRITERIA**

### **Phase 1 Success:**
- ✅ 4 primals running (Songbird, BearDog, Squirrel, NestGate)
- ✅ 4 Unix sockets exist
- ✅ All primals registered with Songbird
- ✅ Health checks passing

### **Phase 2 Success:**
- ✅ All 7 NestGate methods work
- ✅ Squirrel AI inference works
- ✅ BearDog encryption works
- ✅ Songbird discovery works

### **Phase 3 Success:**
- ✅ Inter-primal communication works
- ✅ Encrypted storage workflow complete
- ✅ AI + persistence integration works
- ✅ Full 4-primal workflow passes

### **Phase 4 Success:**
- ✅ Nest niche deployed
- ✅ petalTongue shows live ecosystem
- ✅ All integration tests passing

---

## 🎯 **DELIVERABLES**

1. ✅ 4 operational primals with Unix sockets
2. ✅ Songbird discovery working
3. ✅ Integration test results
4. ✅ Nest niche deployed
5. ✅ petalTongue live visualization
6. 📊 Test coverage report
7. 📚 Interaction patterns documented

---

**Last Updated**: 2026-01-10  
**Status**: ⏳ Ready to start testing  
**Next Action**: Kill stuck process, start 4 primals  

🧪 **Let's Test the Ecosystem!** 🎊

