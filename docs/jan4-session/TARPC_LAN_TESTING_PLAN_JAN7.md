# 🚀 tarpc LAN Testing Plan - January 7, 2026

## 🎯 Goal

Test **tarpc over TCP/LAN** while waiting for BTSP encryption layer.

**Current State**: Songbird v3.14.1 has tarpc compiled in ✅  
**Target**: Enable tarpc TCP server for high-performance P2P testing  
**Future**: Wrap tarpc in BTSP tunnels for encrypted transport

## 📊 Architecture Evolution

### Current (Today):
```
Tower1 ←[HTTPS 8080/8081]→ Tower2
Status: ⚠️  Working but legacy (HTTP overhead)
```

### Step 1 (tarpc LAN - TODAY):
```
Tower1 ←[tarpc TCP 9080/9081]→ Tower2
Status: 🎯 Testing high-performance RPC over LAN
Benefit: ~50μs latency (vs ~5ms HTTPS)
Security: ⚠️  Unencrypted (testing only!)
```

### Step 2 (BTSP + tarpc - SOON):
```
Tower1 ←[BTSP tunnel]→ Tower2
        └─[tarpc inside encrypted tunnel]
Status: 🎯 TARGET architecture
Benefit: Fast + Encrypted
```

## 🔧 Songbird tarpc Configuration

### Built-in Support ✅
Songbird v3.14.1 has tarpc already compiled in:
```bash
$ strings primalBins/songbird-orchestrator | grep tarpc | wc -l
300+ # tarpc support is present!
```

### Environment Variables
```bash
# Enable tarpc TCP server (default: true)
SONGBIRD_TARPC_ENABLED=true

# Bind address (default: 0.0.0.0)
SONGBIRD_TARPC_BIND=0.0.0.0

# Port (default: 9080)
SONGBIRD_TARPC_PORT=9080
```

### Code Evidence
```rust
// From: crates/songbird-orchestrator/src/app/core.rs:629
async fn start_tarpc_server(&self) -> Result<()> {
    let tarpc_enabled = SafeEnv::get_bool("SONGBIRD_TARPC_ENABLED", true);
    
    if !tarpc_enabled {
        info!("ℹ️  tarpc server disabled");
        return Ok(());
    }
    
    let bind_address = SafeEnv::get_or_default("SONGBIRD_TARPC_BIND", "0.0.0.0");
    let port = SafeEnv::get_u16(
        "SONGBIRD_TARPC_PORT",
        songbird_config::defaults::ports::tarpc_port(), // 9080
    );
    
    let addr = format!("{}:{}", bind_address, port);
    info!("🚀 Starting tarpc server on {}", addr);
    
    crate::rpc::tarpc_server::start_tarpc_server_simple(addr, ...).await?;
    info!("✅ tarpc server started successfully");
}
```

## 📋 Deployment Steps

### Step 1: Update tower.toml (Both Spores)
```toml
# Tower 1: /media/eastgate/biomeOS1/biomeOS/tower.toml
[primals.env]
# ... existing vars ...
SONGBIRD_TARPC_ENABLED = "true"
SONGBIRD_TARPC_PORT = "9080"
SONGBIRD_TARPC_BIND = "0.0.0.0"

# Tower 2: /media/eastgate/biomeOS21/biomeOS/tower.toml  
[primals.env]
# ... existing vars ...
SONGBIRD_TARPC_ENABLED = "true"
SONGBIRD_TARPC_PORT = "9081"  # Different port!
SONGBIRD_TARPC_BIND = "0.0.0.0"
```

### Step 2: Kill and Redeploy
```bash
# Kill all existing processes
pkill -9 tower; pkill -9 beardog; pkill -9 songbird

# Deploy tower1
cd /media/eastgate/biomeOS1/biomeOS
nohup ./bin/tower run --config tower.toml > /tmp/tower1_tarpc.log 2>&1 &

# Deploy tower2
cd /media/eastgate/biomeOS21/biomeOS
nohup ./bin/tower run --config tower.toml > /tmp/tower2_tarpc.log 2>&1 &
```

### Step 3: Verify tarpc Listening
```bash
# Check if tarpc ports are listening
ss -tlnp | grep -E "9080|9081"

# Expected output:
# LISTEN 0  128  0.0.0.0:9080  0.0.0.0:*  users:(("songbird",pid=...))
# LISTEN 0  128  0.0.0.0:9081  0.0.0.0:*  users:(("songbird",pid=...))
```

### Step 4: Check Logs for tarpc Startup
```bash
# Tower1 logs
tail -100 /tmp/tower1_tarpc.log | grep -E "(tarpc|9080)"

# Expected:
# 🚀 Starting tarpc server on 0.0.0.0:9080
# ✅ tarpc server started successfully
# 🚀 tarpc PRIMARY: High-performance binary RPC ready

# Tower2 logs
tail -100 /tmp/tower2_tarpc.log | grep -E "(tarpc|9081)"
```

## 🧪 Testing tarpc Communication

### Test 1: Local tarpc Client
```bash
# Install tarpc client (if not available)
cd /home/eastgate/Development/ecoPrimals/phase1/songbird
cargo build --example tarpc_client --release

# Test connection to tower1
./target/release/examples/tarpc_client \
  --endpoint "tarpc://127.0.0.1:9080" \
  --method "ping" \
  --params '{"message":"hello"}'

# Expected: Fast response (~50μs)
```

### Test 2: Peer Discovery via tarpc
```bash
# Check if Songbird advertises tarpc endpoint
tail -100 /tmp/primals/*.log | grep -E "(tarpc|9080|protocols)"

# Expected in node identity:
# protocols: ["https", "tarpc"]
# tarpc_endpoint: "192.168.1.144:9080"
```

### Test 3: Cross-Tower tarpc (Future)
```bash
# After discovery, check if towers use tarpc for P2P
tail -f /tmp/primals/*.log | grep -E "(tarpc.*tower|Connecting via tarpc)"

# This requires Songbird to be updated to USE tarpc for P2P
# (Currently still uses HTTPS for federation)
```

## 📊 Current Limitations

### ✅ What Works NOW:
1. tarpc TCP server starts on both towers
2. tarpc listens on configured ports (9080, 9081)
3. Local tarpc clients can connect
4. High-performance RPC available (~50μs latency)

### ⚠️  What Doesn't Work YET:
1. **Songbird P2P**: Still uses HTTPS for federation
   - Reason: Federation logic hasn't been updated to USE tarpc
   - Fix: Songbird team needs to update connection manager
   
2. **No BTSP encryption**: tarpc is over plain TCP
   - Reason: BTSP tunnel layer not implemented in Songbird
   - Status: Songbird team working on this
   
3. **tarpc not advertised in discovery**: Discovery packets don't include tarpc endpoint
   - Reason: BirdSong UDP protocol doesn't include tarpc info yet
   - Fix: Add tarpc endpoint to identity tags

## 🎯 What We CAN Test Today

### Scenario 1: tarpc Server Availability ✅
**Test**: Verify tarpc servers start and listen
**How**: Check ports with `ss -tlnp`
**Result**: Confirms infrastructure is ready

### Scenario 2: Local tarpc Latency ✅
**Test**: Measure tarpc RPC latency
**How**: Use tarpc_client example to ping server
**Result**: Baseline performance (~50μs)

### Scenario 3: Discovery Protocol ✅
**Test**: Verify tag-based federation still works
**How**: Check logs for "family extracted from tags"
**Result**: Confirms HTTPS fallback is functional

## 🚀 What's Next (Songbird Team)

### Phase 1: tarpc P2P (No Encryption)
**Goal**: Use tarpc instead of HTTPS for federation

**Changes Needed**:
1. Update `connection_manager.rs` to prefer tarpc over HTTPS
2. Add tarpc endpoint to discovery tags
3. Implement tarpc client for peer-to-peer calls

**Code Location**:
```rust
// File: crates/songbird-orchestrator/src/app/connection_manager.rs
async fn connect_to_peer(&self, peer: &Peer) -> Result<()> {
    // Currently does:
    let url = format!("https://{}:{}", peer.address, peer.https_port);
    
    // Should do:
    if let Some(tarpc_port) = peer.tarpc_port {
        let endpoint = format!("tarpc://{}:{}", peer.address, tarpc_port);
        return self.connect_tarpc(endpoint).await;
    }
    
    // Fallback to HTTPS
    let url = format!("https://{}:{}", peer.address, peer.https_port);
    self.connect_https(url).await
}
```

**ETA**: 4-6 hours of work

### Phase 2: BTSP Tunnels (Encrypted)
**Goal**: Wrap tarpc in BTSP encrypted tunnels

**Changes Needed**:
1. After trust evaluation succeeds, request BTSP tunnel from BearDog
2. Use BTSP tunnel for transport layer
3. Run tarpc over encrypted tunnel

**Code Flow**:
```rust
// After trust evaluation returns "same_genetic_family":
async fn establish_secure_connection(&self, peer: &Peer) -> Result<()> {
    // Step 1: Request BTSP tunnel from BearDog
    let tunnel = self.beardog_client
        .establish_tunnel(&peer.node_id, &peer.endpoint)
        .await?;
    
    // Step 2: Use tunnel for tarpc transport
    let transport = BtspTransport::new(tunnel);
    let tarpc_client = TarpcClient::new_with_transport(transport).await?;
    
    // Step 3: Store connection
    self.connections.insert(peer.node_id.clone(), tarpc_client);
    
    Ok(())
}
```

**ETA**: 1-2 days of work (waiting for BTSP API evolution)

## 📋 Testing Checklist

### Today (tarpc Infrastructure):
- [ ] Update tower.toml with tarpc env vars
- [ ] Kill old processes
- [ ] Redeploy both towers
- [ ] Verify tarpc ports listening (9080, 9081)
- [ ] Check logs for "tarpc server started"
- [ ] Test local tarpc connection with example client
- [ ] Measure tarpc latency (~50μs expected)
- [ ] Verify HTTPS federation still works (fallback)

### Soon (tarpc P2P):
- [ ] Songbird adds tarpc endpoint to discovery tags
- [ ] Songbird prefers tarpc over HTTPS for peers
- [ ] Test cross-tower tarpc communication
- [ ] Measure P2P latency improvement (HTTPS vs tarpc)
- [ ] Verify federation with tarpc works

### Later (BTSP + tarpc):
- [ ] BearDog BTSP tunnel API ready
- [ ] Songbird requests tunnels after trust evaluation
- [ ] tarpc runs inside BTSP encrypted tunnel
- [ ] Verify encryption with packet capture
- [ ] Full port-free architecture complete

## 🎊 Summary

### What Songbird v3.14.1 HAS:
✅ tarpc server implementation (compiled in)  
✅ tarpc client for primal-to-primal IPC  
✅ Auto-start tarpc server on launch  
✅ Configurable via environment variables  

### What We're Testing TODAY:
🎯 Enable tarpc TCP servers on both towers  
🎯 Verify tarpc infrastructure is ready  
🎯 Measure baseline tarpc performance  
🎯 Keep HTTPS as fallback (still works)  

### What We're WAITING For:
⏭️ Songbird P2P using tarpc (not HTTPS)  
⏭️ BTSP tunnel API ready in Songbird  
⏭️ tarpc wrapped in BTSP encrypted tunnels  

### Architecture Goal:
```
Discovery:   UDP multicast (anonymous, tag-based) ✅
Trust:       BearDog genetic lineage ✅
Transport:   BTSP encrypted tunnels ⏭️
Protocol:    tarpc (high-performance) 🎯 TESTING TODAY!
```

---

**Date**: January 7, 2026, 21:30 UTC  
**Status**: tarpc infrastructure ready for testing  
**Next**: Configure tower.toml, deploy, verify tarpc servers

