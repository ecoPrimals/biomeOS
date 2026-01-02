# 🐦 Songbird Track 2 Handoff: UDP Lineage Advertisement

**Date**: January 3, 2026 (Evening)  
**From**: biomeOS Team  
**To**: Songbird Team  
**Priority**: 🔥 **CRITICAL - Blocks All Federation**  
**Timeline**: Week 1

---

## 🎯 Executive Summary

**BearDog Track 1 is complete!** Progressive trust is ready.

**Critical Blocker**: Songbird UDP discovery packets don't include genetic lineage → BearDog can't evaluate trust during discovery.

**Fix Needed**: Query BearDog for identity on startup, include `identity_attestations` in UDP discovery packets.

**Impact**: Unblocks entire federation system with progressive trust!

---

## 🔥 Why This is Critical

### Current Problem

**Songbird Discovery Flow** (Today):
```
1. Songbird starts up
2. Sends UDP discovery: {name, capabilities, endpoint}
3. Peer receives discovery
4. Peer queries Songbird for more info
```

**Gap**: No lineage info in discovery → BearDog must make trust decision **blind**

### What We Need

**Songbird Discovery Flow** (Progressive Trust):
```
1. Songbird starts up
2. Query BearDog: GET /api/v1/trust/identity
3. Cache identity_attestations
4. Send UDP discovery: {name, capabilities, endpoint, identity_attestations}
5. Peer receives discovery
6. Peer can immediately evaluate trust level!
```

**Result**: BearDog can make **informed trust decisions** during discovery!

---

## 📋 Required Changes

### Step 1: Query BearDog on Startup

**Location**: Songbird initialization code

**Add**:
```rust
// Query BearDog for our identity
let beardog_url = env::var("BEARDOG_URL")
    .unwrap_or_else(|_| "http://localhost:9000".to_string());

let identity_response = reqwest::get(format!("{}/api/v1/trust/identity", beardog_url))
    .await?
    .json::<IdentityResponse>()
    .await?;

// Cache for use in discovery
let identity_attestations = identity_response.identity_attestations;
```

**Response Structure**:
```json
{
  "node_id": "tower1",
  "identity_attestations": {
    "genetic_lineage": {
      "family_id": "abc123",
      "node_role": "tower",
      "lineage_proof": "..."
    }
  }
}
```

### Step 2: Include in UDP Discovery

**Location**: UDP discovery packet creation

**Current**:
```rust
DiscoveryAnnouncement {
    service_name: "songbird-orchestrator".to_string(),
    capabilities: vec!["orchestration", "discovery"],
    endpoint: "http://localhost:8080".to_string(),
}
```

**Add**:
```rust
DiscoveryAnnouncement {
    service_name: "songbird-orchestrator".to_string(),
    capabilities: vec!["orchestration", "discovery"],
    endpoint: "http://localhost:8080".to_string(),
    // ADD THIS:
    identity_attestations: Some(identity_attestations.clone()),
}
```

### Step 3: Update DiscoveryAnnouncement Struct

**Add field**:
```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DiscoveryAnnouncement {
    pub service_name: String,
    pub capabilities: Vec<String>,
    pub endpoint: String,
    // ADD THIS:
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity_attestations: Option<IdentityAttestations>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdentityAttestations {
    pub genetic_lineage: Option<GeneticLineage>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GeneticLineage {
    pub family_id: String,
    pub node_role: String,
    pub lineage_proof: String,
}
```

---

## 🔄 Complete Flow

### Tower 1 (Sender)

```
1. Songbird starts
2. Query BearDog: GET http://localhost:9000/api/v1/trust/identity
3. Cache identity_attestations
4. Send UDP: {
     name: "songbird",
     endpoint: "http://192.168.1.100:8080",
     identity_attestations: {
       genetic_lineage: {family_id: "abc123", ...}
     }
   }
```

### Tower 2 (Receiver)

```
5. Receive UDP discovery packet
6. Extract identity_attestations
7. Query BearDog: POST /api/v1/trust/evaluate
   Body: {
     peer_id: "tower1",
     peer_lineage: "family:abc123",
     requested_operation: "coordination/announce"
   }
8. BearDog responds: {
     trust_level_numeric: 1,
     allowed_capabilities: ["coordination/*", "discovery"],
     denied_capabilities: ["data/*", "federation/*"]
   }
9. Songbird: Allow limited coordination, deny federation
```

**Result**: Trust evaluated **during discovery**, not after!

---

## 🎯 Success Criteria

### Must Have (Week 1)

- [x] Query BearDog for identity on startup
- [ ] Include `identity_attestations` in UDP packets
- [ ] Peers can extract lineage from discovery
- [ ] BearDog trust evaluation works with lineage
- [ ] Test: Two towers discover each other with limited trust

### Nice to Have (Future)

- [ ] Cache identity with TTL/refresh
- [ ] Handle BearDog unavailable gracefully
- [ ] Metrics for trust evaluations

---

## 🧪 Testing

### Unit Tests

```rust
#[test]
fn test_discovery_with_lineage() {
    let announcement = DiscoveryAnnouncement {
        service_name: "songbird".to_string(),
        capabilities: vec!["orchestration".to_string()],
        endpoint: "http://localhost:8080".to_string(),
        identity_attestations: Some(IdentityAttestations {
            genetic_lineage: Some(GeneticLineage {
                family_id: "test123".to_string(),
                node_role: "tower".to_string(),
                lineage_proof: "...".to_string(),
            }),
        }),
    };
    
    let serialized = serde_json::to_string(&announcement).unwrap();
    assert!(serialized.contains("identity_attestations"));
    assert!(serialized.contains("test123"));
}
```

### Integration Test

```bash
# Terminal 1: Start BearDog
./beardog-server &

# Terminal 2: Start Songbird Tower 1
BEARDOG_URL=http://localhost:9000 ./songbird-orchestrator &

# Terminal 3: Start Songbird Tower 2
BEARDOG_URL=http://localhost:9000 ./songbird-orchestrator --port 8081 &

# Terminal 4: Check logs
# Should see: "Discovered peer with lineage family:abc123"
# Should see: "Trust level: 1 (Limited)"
# Should see: "Allowed capabilities: coordination/*"
```

---

## ⏰ Timeline

**Day 1 (Jan 4)**:
- [ ] Morning: Implement identity query on startup
- [ ] Afternoon: Add identity_attestations to struct
- [ ] Evening: Update UDP packet creation

**Day 2 (Jan 5)**:
- [ ] Morning: Unit tests
- [ ] Afternoon: Integration testing (two towers)
- [ ] Evening: Verify BearDog trust evaluation

**Day 3 (Jan 6)**:
- [ ] Cleanup and documentation
- [ ] Deploy to test environment
- [ ] Handoff back to biomeOS for full integration

---

## 🔍 Edge Cases

### BearDog Unavailable

**Scenario**: Songbird starts but BearDog isn't running

**Handling**:
```rust
let identity_attestations = match query_beardog_identity().await {
    Ok(identity) => Some(identity.identity_attestations),
    Err(e) => {
        tracing::warn!("BearDog unavailable, announcing without lineage: {}", e);
        None  // Still send discovery, just without lineage
    }
};
```

**Result**: Graceful degradation - discovery works, but without trust info

### Lineage Update

**Scenario**: BearDog lineage changes after Songbird starts

**Current**: Cache doesn't update (acceptable for v1)

**Future**: Subscribe to BearDog for lineage updates

---

## 📖 API Reference

### BearDog Identity Endpoint

**Request**:
```
GET http://localhost:9000/api/v1/trust/identity
```

**Response**:
```json
{
  "node_id": "tower1",
  "identity_attestations": {
    "genetic_lineage": {
      "family_id": "abc123",
      "node_role": "tower",
      "lineage_proof": "base64EncodedProof...",
      "attested_at": "2026-01-03T16:00:00Z"
    }
  }
}
```

### BearDog Trust Evaluation (For Reference)

**Request**:
```
POST http://localhost:9000/api/v1/trust/evaluate
Content-Type: application/json

{
  "peer_id": "tower2",
  "peer_lineage": "family:abc123",
  "requested_operation": "coordination/announce"
}
```

**Response**:
```json
{
  "trust_level": "Limited",
  "trust_level_numeric": 1,
  "allowed_capabilities": ["discovery", "coordination/*"],
  "denied_capabilities": ["data/*", "federation/*"],
  "elevation_path": {
    "next_level": 2,
    "requirements": ["human_approval"],
    "method": "user_consent_ui"
  }
}
```

---

## 🎊 Impact

### Security

**Before**: 
- Discovery packets anonymous
- BearDog makes blind trust decisions
- All-or-nothing trust

**After**:
- Discovery packets include lineage
- BearDog makes **informed** trust decisions
- **Progressive** trust based on capabilities

### Federation

**Before**:
- Binary trust (full access or no access)

**After**:
- **Limited trust** for coordination (Level 1)
- **Elevated trust** for federation (Level 2) 
- **Highest trust** for sensitive ops (Level 3)

### Developer Experience

**Before**:
- Manual trust configuration
- No visibility into trust levels

**After**:
- **Automatic** trust evaluation
- Clear capability boundaries
- Audit trail of trust decisions

---

## ❓ Questions?

**Q**: What if BearDog isn't running?  
**A**: Graceful degradation - send discovery without lineage. Peers will deny federation but allow basic discovery.

**Q**: Does this break backward compatibility?  
**A**: No! `identity_attestations` is optional. Old Songbird versions ignore it, new versions use it.

**Q**: What about performance?  
**A**: Identity query happens once on startup (cached). UDP packets slightly larger (~200 bytes), negligible impact.

**Q**: When can we test?  
**A**: Immediately! BearDog v0.12.0-progressive-trust is ready now.

---

## 🚀 Next Steps

### For Songbird Team (This Week)

**Priority 1**: Implement identity query + UDP inclusion  
**Priority 2**: Test with two towers  
**Priority 3**: Deploy and handoff

### For biomeOS Team

**Waiting**: Songbird implementation (Week 1)  
**Then**: Full integration testing with progressive trust  
**Then**: PetalTongue UI (Week 5)

### For BearDog Team

✅ **Standing by** - Track 1 complete, ready to support integration

---

## 📊 Success Metrics

### Week 1 End Goal

- ✅ Songbird queries BearDog on startup
- ✅ UDP packets include lineage
- ✅ Two towers discover each other
- ✅ BearDog evaluates trust (Level 1 - Limited)
- ✅ Coordination works (discovery, health, capabilities)
- ⏸️ Federation blocked (requires Level 2 - Elevated)

**Result**: First progressive trust federation! 🎊

---

**Status**: 🔥 **CRITICAL - Week 1 Priority**  
**Blocker**: Yes - Blocks all progressive trust federation  
**Timeline**: 3 days for implementation + testing  
**Support**: BearDog team standing by

🐦🔒🚀 **Let's enable progressive trust federation!** 🚀🔒🐦
