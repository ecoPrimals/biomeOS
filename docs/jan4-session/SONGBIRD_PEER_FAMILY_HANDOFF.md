# 📨 Songbird: Peer Family Discovery - Handoff

**Date**: January 7, 2026 04:10 UTC  
**Priority**: **HIGH** - Blocks federation  
**Status**: Ready for Songbird team  
**Estimated Time**: 1-2 hours  

---

## 🎯 Problem

Songbird successfully discovers peers and calls BearDog for trust evaluation, but doesn't provide the peer's family ID. BearDog receives `peer_family: ""` (empty) and correctly rejects with `"unknown_family"`.

**Current Error**:
```
❌ BearDog says REJECT peer 117ae58c... (unknown_family)
```

**Root Cause**: Songbird doesn't extract or pass peer family information to BearDog.

---

## ✅ What's Working

1. ✅ Schema compatibility (decision field, trust_level parsing)
2. ✅ Songbird → BearDog IPC communication
3. ✅ BearDog trust evaluation logic
4. ✅ Discovery (peers are found)

**Only Missing**: Peer family information in trust evaluation request.

---

## 🔧 Required Fix (Phase 1 - String-Based)

### Goal:
Pass peer's family ID to BearDog so it can compare: `peer_family == our_family`

### Approach:
Songbird needs to:
1. Extract peer family ID (from discovery, config, or convention)
2. Include it in trust evaluation request to BearDog

---

## 📋 Implementation Options

### Option A: Extract from Discovery (Recommended)

**If peers advertise family in UDP discovery**:

```rust
// In discovery packet handling
let discovered_peer = DiscoveredPeer {
    node_id: packet.node_id,
    family_id: packet.family_id,  // ← Extract this
    capabilities: packet.capabilities,
    endpoint: packet.endpoint,
};

// Later, in trust evaluation
let trust_request = TrustEvaluationRequest {
    peer_id: peer.node_id,
    peer_family: peer.family_id,  // ← Pass to BearDog
    peer_tags: peer.tags,
};
```

**Changes Needed**:
1. Update discovery packet format to include `family_id`
2. Store `family_id` in `DiscoveredPeer` struct
3. Pass `family_id` to BearDog in trust evaluation

**Timeline**: 1-2 hours

---

### Option B: Use Convention (Quick Fix)

**If all LAN peers share the same family**:

```rust
// Read our family from config
let our_family = env::var("SONGBIRD_FAMILY_ID").unwrap_or("nat0".to_string());

// Assume LAN peers share our family
let trust_request = TrustEvaluationRequest {
    peer_id: peer.node_id,
    peer_family: our_family.clone(),  // ← Assume same family for LAN
    peer_tags: peer.tags,
};
```

**Changes Needed**:
1. Read `SONGBIRD_FAMILY_ID` from environment
2. Pass it as `peer_family` for all LAN peers

**Timeline**: 30 minutes

**Pros**: Quick, works for single-family LANs  
**Cons**: Won't work for cross-family federation

---

### Option C: Query Peer's Capabilities (Proper)

**Query peer's identity endpoint**:

```rust
// After discovering peer, query their identity
let peer_identity = query_peer_identity(&peer.endpoint).await?;

let trust_request = TrustEvaluationRequest {
    peer_id: peer.node_id,
    peer_family: peer_identity.family_id,  // ← From peer's response
    peer_tags: peer.tags,
};
```

**Changes Needed**:
1. Add `query_peer_identity()` function
2. Call peer's `/identity` or similar endpoint
3. Extract `family_id` from response
4. Pass to BearDog

**Timeline**: 2-3 hours

**Pros**: Works for any peer, proper architecture  
**Cons**: Extra network call, more complex

---

## 📊 Comparison

| Option | Time | Complexity | Works For | Recommended |
|--------|------|------------|-----------|-------------|
| **A: Discovery** | 1-2h | Medium | Multi-family | ✅ Yes (if discovery updated) |
| **B: Convention** | 30m | Low | Single-family | ✅ Yes (quick fix) |
| **C: Query** | 2-3h | High | Any scenario | Later (Phase 2) |

---

## 🎯 Recommended Approach

**Start with Option B (Convention)**, then evolve to Option A (Discovery).

### Phase 1: Convention-Based (Immediate)

```rust
// In Songbird's trust evaluation code
impl SongbirdOrchestrator {
    fn evaluate_peer_trust(&self, peer: &DiscoveredPeer) -> Result<TrustDecision> {
        // Read our family from environment
        let our_family = env::var("SONGBIRD_FAMILY_ID")
            .unwrap_or_else(|_| "nat0".to_string());
        
        // For LAN peers, assume same family (Phase 1)
        let peer_family = if peer.is_local_network() {
            our_family.clone()
        } else {
            "unknown".to_string()  // External peers need explicit family
        };
        
        // Call BearDog with peer_family
        let request = json!({
            "peer_id": peer.node_id,
            "peer_family": peer_family,  // ← NOW PROVIDED!
            "peer_tags": peer.tags,
        });
        
        let response = self.beardog_client.evaluate_trust(request).await?;
        // ...
    }
}
```

**Result**: Federation works for same-family LANs immediately!

### Phase 2: Discovery-Based (Future)

Update UDP discovery to include family:

```rust
// In discovery packet
#[derive(Serialize, Deserialize)]
struct DiscoveryPacket {
    node_id: String,
    family_id: String,  // ← Add this
    capabilities: Vec<String>,
    endpoint: String,
}
```

---

## 🧪 Testing

### Test 1: Same Family (Should Auto-Accept)

```bash
# Tower 1
export SONGBIRD_FAMILY_ID=nat0

# Tower 2
export SONGBIRD_FAMILY_ID=nat0

# Expected: trust_level: 1 (limited), decision: "auto_accept"
```

### Test 2: Different Family (Should Reject)

```bash
# Tower 1
export SONGBIRD_FAMILY_ID=nat0

# Tower 2
export SONGBIRD_FAMILY_ID=other

# Expected: trust_level: 0 (none), decision: "reject"
```

### Test 3: Unknown Family (Should Reject)

```bash
# Tower 1
export SONGBIRD_FAMILY_ID=nat0

# Tower 2
# (no peer_family provided)

# Expected: trust_level: 0 (none), decision: "reject", reason: "unknown_family"
```

---

## 📋 Files to Modify

### Option B (Convention - Recommended for Phase 1):

1. **`crates/songbird-orchestrator/src/trust/peer_trust.rs`**
   - Read `SONGBIRD_FAMILY_ID` from environment
   - Pass as `peer_family` in BearDog request

2. **`crates/songbird-orchestrator/src/security_capability_client.rs`**
   - Update `TrustEvaluationRequest` to include `peer_family`

**Estimated Changes**: ~20 lines

### Option A (Discovery - Phase 2):

1. **`crates/songbird-discovery/src/anonymous_discovery.rs`**
   - Add `family_id` to discovery packet
   - Extract `family_id` from received packets

2. **`crates/songbird-types/src/discovery.rs`**
   - Add `family_id` field to `DiscoveredPeer`

3. **`crates/songbird-orchestrator/src/trust/peer_trust.rs`**
   - Use `peer.family_id` in trust evaluation

**Estimated Changes**: ~50 lines

---

## 💡 Important Notes

### For Songbird Team:

1. **You don't need to understand genetic lineage**
   - Just pass the family ID string to BearDog
   - BearDog handles the trust logic

2. **Phase 1 is string-based**
   - Simple string comparison: `peer_family == our_family`
   - Phase 2 will add cryptographic verification (BearDog's job)

3. **Multiple identities (future)**
   - Eventually, one person may have multiple family IDs
   - BearDog will verify cryptographic lineage
   - Songbird just passes credentials

---

## 🎯 Success Criteria

After this fix:

- [ ] Songbird provides `peer_family` in trust evaluation requests
- [ ] BearDog receives non-empty `peer_family`
- [ ] Same-family peers: `trust_level: 1`, `decision: "auto_accept"`
- [ ] Different-family peers: `trust_level: 0`, `decision: "reject"`
- [ ] Federation works for same-family towers
- [ ] Zero "unknown_family" errors in logs

---

## 📞 Questions?

**Q**: Where does `peer_family` come from?  
**A**: Phase 1: Assume same as our family for LAN peers. Phase 2: From discovery or peer query.

**Q**: What if peer doesn't have a family?  
**A**: Pass empty string or "unknown". BearDog will reject (trust_level: 0).

**Q**: Do we need to verify the family cryptographically?  
**A**: No! That's BearDog's job. Phase 1 is just string passing. Phase 2 (future) will add crypto.

**Q**: What about multiple identities per person?  
**A**: Future (Phase 3). For now, one family ID per tower is fine.

---

## 🚀 Next Steps

1. **Songbird team**: Implement Option B (convention-based)
2. **Test**: Verify same-family federation works
3. **Deploy**: Update both towers
4. **Verify**: Check logs for successful trust evaluation
5. **Evolve**: Plan Option A (discovery-based) for Phase 2

---

**Bottom Line**: Add one field (`peer_family`) to the trust evaluation request. That's it! BearDog handles the rest.

---

_Handoff Date: January 7, 2026 04:10 UTC_  
_Status: Ready for Songbird team_  
_Priority: HIGH - Unblocks federation_

