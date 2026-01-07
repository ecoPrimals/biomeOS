# 📦 Phase 1 Deployment Guide - Trust Policy Evolution

**Date**: January 7, 2026  
**Version**: Phase 1 - Dual Representation  
**Status**: 🎯 **READY FOR DEPLOYMENT**  
**Estimated Time**: ~2 days (primal team implementation) + 1 day (deployment & testing)

---

## 🎯 What is Phase 1?

Phase 1 adds **dual representation** to trust evaluation responses:
- BearDog returns BOTH integer and string trust_level
- BearDog adds capability hints (what's allowed/denied)
- Songbird accepts both integer and string formats
- **Result**: Federation unblocked, backward compatible, forward compatible

---

## 📋 Prerequisites

### **Before Phase 1**:
- ✅ BearDog v0.16.0: Capability-based IPC working
- ✅ Songbird v3.13.0: Protocol-agnostic (SecurityAdapter)
- ✅ biomeOS towers: Running with Unix socket configuration
- ✅ Discovery: Working (UDP multicast)
- ⚠️ Trust evaluation: Blocked by schema mismatch

### **After Phase 1**:
- ✅ Trust evaluation: Working (schema compatible)
- ✅ Federation: Established (peers added)
- ✅ Genetic lineage trust: Working (same family, level 1)

---

## 🚀 Deployment Steps

### **Step 1: Primal Teams Implement Changes** (~2 days)

#### **BearDog Team** (~1 day)

**File**: `/home/eastgate/Development/ecoPrimals/phase1/beardog/crates/beardog-tunnel/src/unix_socket_ipc.rs`

**Change**: Update `trust.evaluate_peer` response format

**Before**:
```json
{
  "trust_level": 0,
  "reason": "unknown_family",
  "peer_id": "tower2",
  "peer_family": null,
  "our_family": "nat0"
}
```

**After**:
```json
{
  "trust_level": 0,                    // Keep integer (compact)
  "trust_level_name": "anonymous",     // Add string (readable)
  "reason": "unknown_family",
  "peer_id": "tower2",
  "peer_family": null,
  "our_family": "nat0",
  "capabilities": {                    // Add capability hints
    "allowed": [],
    "denied": ["*"]
  }
}
```

**Mapping Table**:
```rust
fn trust_level_to_name(level: u8) -> &'static str {
    match level {
        0 => "anonymous",  // No trust
        1 => "limited",    // Same family - BirdSong only
        2 => "elevated",   // Human approved - full federation
        3 => "highest",    // Human entropy - all operations
        _ => "unknown",
    }
}

fn trust_level_to_capabilities(level: u8) -> (Vec<String>, Vec<String>) {
    match level {
        0 => (vec![], vec!["*".to_string()]),  // Deny all
        1 => (
            vec!["birdsong/*".to_string(), "coordination/*".to_string(), "health".to_string()],
            vec!["data/*".to_string(), "commands/*".to_string()],
        ),
        2 => (
            vec!["birdsong/*".to_string(), "coordination/*".to_string(), "federation/*".to_string(), "data/read".to_string()],
            vec!["data/write".to_string(), "commands/sensitive".to_string()],
        ),
        3 => (vec!["*".to_string()], vec![]),  // Allow all
        _ => (vec![], vec!["*".to_string()]),
    }
}
```

#### **Songbird Team** (~1 day)

**File**: `/home/eastgate/Development/ecoPrimals/phase1/songbird/crates/songbird-universal/src/trust_types.rs`

**Change**: Accept both integer and string trust_level

**Add to struct**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustEvaluationResponse {
    /// Decision: "auto_accept", "prompt_user", or "reject"
    pub decision: String,
    
    /// Trust level: supports both integer and string
    #[serde(deserialize_with = "deserialize_flexible_trust_level")]
    pub trust_level: String,
    
    /// Reason for decision
    pub reason: String,
    
    /// Optional capability hints from security provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<CapabilityHints>,
    
    // ... rest of fields
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityHints {
    pub allowed: Vec<String>,
    pub denied: Vec<String>,
}

fn deserialize_flexible_trust_level<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Deserialize};
    
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum TrustLevelValue {
        Int(u8),
        String(String),
    }
    
    match TrustLevelValue::deserialize(deserializer)? {
        TrustLevelValue::Int(0) => Ok("anonymous".to_string()),
        TrustLevelValue::Int(1) => Ok("limited".to_string()),
        TrustLevelValue::Int(2) => Ok("elevated".to_string()),
        TrustLevelValue::Int(3) => Ok("highest".to_string()),
        TrustLevelValue::String(s) => Ok(s),
        _ => Err(de::Error::custom("invalid trust level")),
    }
}
```

---

### **Step 2: Build Updated Binaries** (~30 minutes)

#### **BearDog**:
```bash
cd /home/eastgate/Development/ecoPrimals/phase1/beardog
cargo build --release -p beardog-server
```

**Output**: `target/release/beardog-server`

#### **Songbird**:
```bash
cd /home/eastgate/Development/ecoPrimals/phase1/songbird
cargo build --release -p songbird-orchestrator
```

**Output**: `target/release/songbird-orchestrator`

---

### **Step 3: Test with Direct Commands** (~15 minutes)

#### **Test BearDog Response Format**:
```bash
# Start new BearDog binary
FAMILY_ID=nat0 NODE_ID=test ./target/release/beardog-server &
sleep 2

# Test trust evaluation (should include both int and string)
echo '{"jsonrpc":"2.0","method":"trust.evaluate_peer","params":{"peer_id":"tower2","peer_family":"nat0"},"id":1}' | nc -U /tmp/beardog-nat0-test.sock | jq

# Expected response:
# {
#   "result": {
#     "trust_level": 1,
#     "trust_level_name": "limited",
#     "reason": "same_genetic_family",
#     "capabilities": {
#       "allowed": ["birdsong/*", "coordination/*", "health"],
#       "denied": ["data/*", "commands/*"]
#     }
#   },
#   "id": 1
# }
```

#### **Test Songbird Parsing**:
```bash
# Start new Songbird binary
SONGBIRD_FAMILY_ID=nat0 SONGBIRD_NODE_ID=test SECURITY_ENDPOINT=unix:///tmp/beardog-nat0-test.sock ./target/release/songbird-orchestrator &
sleep 2

# Watch logs for trust evaluation
tail -f /tmp/primals/songbird-test.log | grep -E '(trust|parse|error)'

# Should see:
# ✅ Trust evaluation successful
# ✅ Trust level: limited
# ✅ Peer accepted (same family)
```

---

### **Step 4: Deploy to biomeOS Towers** (~30 minutes)

#### **Stop Running Towers**:
```bash
# Kill all existing processes
pkill -9 tower
pkill -9 beardog-server
pkill -9 songbird-orchestrator
sleep 2

# Verify all stopped
ps aux | grep -E '(tower|beardog|songbird)' | grep -v grep
```

#### **Update Binaries**:
```bash
# Copy to primalBins
cp /home/eastgate/Development/ecoPrimals/phase1/beardog/target/release/beardog-server \
   /home/eastgate/Development/ecoPrimals/phase2/biomeOS/primalBins/beardog

cp /home/eastgate/Development/ecoPrimals/phase1/songbird/target/release/songbird-orchestrator \
   /home/eastgate/Development/ecoPrimals/phase2/biomeOS/primalBins/songbird

# Update USB spores
cp /home/eastgate/Development/ecoPrimals/phase2/biomeOS/primalBins/beardog \
   /media/eastgate/biomeOS1/biomeOS/primals/beardog

cp /home/eastgate/Development/ecoPrimals/phase2/biomeOS/primalBins/songbird \
   /media/eastgate/biomeOS1/biomeOS/primals/songbird

cp /home/eastgate/Development/ecoPrimals/phase2/biomeOS/primalBins/beardog \
   /media/eastgate/biomeOS21/biomeOS/primals/beardog

cp /home/eastgate/Development/ecoPrimals/phase2/biomeOS/primalBins/songbird \
   /media/eastgate/biomeOS21/biomeOS/primals/songbird
```

#### **Update VERSION.txt**:
```bash
# Update both spores to reflect Phase 1
# Add:
# beardog.version = "v0.16.1-phase1-dual-representation"
# songbird.version = "v3.13.1-phase1-flexible-parsing"
# phase1_features = "dual_representation, capability_hints, flexible_parsing"
```

---

### **Step 5: Start Towers and Verify** (~15 minutes)

#### **Start Tower 1**:
```bash
cd /media/eastgate/biomeOS1/biomeOS
./activate-tower.sh > /tmp/tower1-startup.log 2>&1 &
sleep 5
```

#### **Start Tower 2**:
```bash
cd /media/eastgate/biomeOS21/biomeOS  
./activate-tower.sh > /tmp/tower2-startup.log 2>&1 &
sleep 5
```

#### **Check Sockets**:
```bash
ls -lh /tmp/beardog-nat0-tower*.sock /tmp/songbird-nat0-tower*.sock
# Expected: All 4 sockets present
```

#### **Monitor Logs**:
```bash
# Tower 1 Songbird
tail -f /tmp/primals/*tower1*.log | grep -E '(discovered|trust|federation)'

# Expected within 30 seconds:
# 🔍 Discovered peer: tower2
# 🔍 Evaluating trust for peer: tower2
# ✅ Trust level: limited (same genetic family)
# ✅ Peer 'tower2' accepted - trust level 1
# 🤝 Peer 'tower2' joined federation
```

---

### **Step 6: Verify Federation** (~10 minutes)

#### **Test 1: Peer Discovery API**:
```bash
echo '{"jsonrpc":"2.0","method":"discovery.list_peers","id":1}' | nc -U /tmp/songbird-nat0-tower1.sock | jq

# Expected:
# {
#   "result": {
#     "total": 1,
#     "peers": [{
#       "peer_id": "tower2",
#       "trust_level": "limited",
#       "family_id": "nat0"
#     }]
#   }
# }
```

#### **Test 2: Trust Evaluation Direct**:
```bash
# Get tower2's peer ID from discovery
PEER_ID=$(echo '{"jsonrpc":"2.0","method":"discovery.list_peers","id":1}' | nc -U /tmp/songbird-nat0-tower1.sock | jq -r '.result.peers[0].peer_id')

# Test BearDog trust evaluation
echo "{\"jsonrpc\":\"2.0\",\"method\":\"trust.evaluate_peer\",\"params\":{\"peer_id\":\"$PEER_ID\",\"peer_family\":\"nat0\"},\"id\":1}" | nc -U /tmp/beardog-nat0-tower1.sock | jq

# Expected:
# {
#   "result": {
#     "trust_level": 1,
#     "trust_level_name": "limited",
#     "reason": "same_genetic_family",
#     "capabilities": {...}
#   }
# }
```

#### **Test 3: Check Logs for Errors**:
```bash
# Should have NO parse errors
grep -i "parse.*error\|failed.*parse" /tmp/primals/*tower*.log

# Expected: No output (no parse errors!)
```

---

## ✅ Success Criteria

### **Must Pass**:
- [ ] BearDog returns both `trust_level` (int) and `trust_level_name` (string)
- [ ] BearDog returns `capabilities` with allowed/denied lists
- [ ] Songbird parses responses without errors
- [ ] Tower 1 discovers Tower 2
- [ ] Tower 2 discovers Tower 1
- [ ] Trust evaluation succeeds (level 1 - "limited")
- [ ] Peers added to federation
- [ ] No "parse error" in logs
- [ ] `discovery.list_peers` returns both towers

### **Should Pass**:
- [ ] Capability hints match trust level (level 1 → birdsong/coordination only)
- [ ] Genetic lineage reason ("same_genetic_family")
- [ ] Federation established within 30 seconds of startup

### **Nice to Have**:
- [ ] tarpc protocol auto-negotiation (future)
- [ ] Trust escalation to level 2+ (Phase 2)
- [ ] Contact key exchange (Phase 3)

---

## 🐛 Troubleshooting

### **Issue**: Parse errors still occurring

**Check**:
```bash
# Verify binary versions
/home/eastgate/Development/ecoPrimals/phase2/biomeOS/primalBins/beardog --version
/home/eastgate/Development/ecoPrimals/phase2/biomeOS/primalBins/songbird --version
```

**Fix**: Rebuild and redeploy binaries

---

### **Issue**: Federation not establishing

**Check**:
```bash
# Verify discovery working
grep "Discovered peer" /tmp/primals/*tower*.log

# Verify trust evaluation happening
grep "Evaluating trust" /tmp/primals/*tower*.log

# Verify no errors
grep -i "error\|failed" /tmp/primals/*tower*.log | tail -20
```

**Fix**: Check logs for specific error, verify family IDs match

---

### **Issue**: Capabilities missing in response

**Check**:
```bash
# Test BearDog directly
echo '{"jsonrpc":"2.0","method":"trust.evaluate_peer","params":{"peer_id":"test","peer_family":"nat0"},"id":1}' | nc -U /tmp/beardog-nat0-tower1.sock | jq '.result.capabilities'

# Should show: {"allowed": [...], "denied": [...]}
```

**Fix**: Verify BearDog Phase 1 implementation complete

---

## 📚 Documentation Updates

### **After Phase 1 Deployment**:

1. **Update `DUAL_TOWER_TEST_RESULTS.md`**:
   - Mark "Trust Evaluation" as ✅ **WORKING**
   - Mark "Federation" as ✅ **WORKING**
   - Update status to "✅ **FEDERATION COMPLETE**"

2. **Update `VERSION.txt` on USB spores**:
   - Update beardog.version
   - Update songbird.version
   - Add phase1_status = "deployed"

3. **Create `PHASE1_SUCCESS.md`**:
   - Document test results
   - Include before/after comparisons
   - Capture sample logs
   - Note performance metrics

---

## 🎯 Next Steps After Phase 1

### **Short-Term**:
1. ✅ Verify federation working on LAN (separate physical towers)
2. ✅ Document success
3. ✅ Update USB spores with Phase 1 binaries

### **Medium-Term** (Phase 2 - ~2 weeks):
1. Design trust policy schema (YAML/TOML)
2. Implement policy loading and signature verification
3. Test with custom policies (different org requirements)

### **Long-Term** (Phase 3 - ~3 weeks):
1. Design contact key exchange protocol
2. Implement DH key exchange with lineage proofs
3. Test NAT traversal and P2P encryption

---

## 📊 Metrics to Capture

### **Performance**:
- Time to federation (discovery → trust eval → peer added)
- Trust evaluation latency (Songbird → BearDog round-trip)
- Discovery latency (UDP multicast)

### **Reliability**:
- Parse error count (should be 0)
- Failed trust evaluations (should be 0 for same family)
- Connection failures (should be 0)

### **Capability Validation**:
- Allowed operations at level 1 (birdsong/coordination)
- Denied operations at level 1 (data/commands)
- Correct enforcement of capability restrictions

---

**Version**: Phase 1 Deployment Guide v1.0  
**Date**: January 7, 2026  
**Status**: 🎯 **READY FOR DEPLOYMENT**

---

*"Don't patch the schema - evolve the architecture!"* 🔐

