# 🔒 Handoff: Progressive Trust Model - All Teams

**Date**: January 3, 2026  
**To**: BearDog, Songbird, biomeOS teams  
**From**: biomeOS Architecture Review  
**Priority**: Critical - Architectural Redesign

---

## 🎯 Executive Summary

**Current Status**: We're 90% to first federation, but discovered a critical architectural issue.

**Immediate Blocker**: Genetic lineage not advertised in UDP discovery packets

**Deeper Issue**: Binary trust model (auto-accept vs. reject) is too simplistic and potentially insecure

**Proposal**: Progressive trust model with capability-based access control

---

## 🚨 Two Parallel Tracks

### Track 1: Immediate Fix (Still Needed!)

**Problem**: Lineage not in discovery packets → cannot compare families → cannot trust

**Fix**: Songbird must include genetic lineage in UDP announcements

**Timeline**: Needed for ANY federation to work

**Team**: Songbird

---

### Track 2: Architecture Redesign (Long-term)

**Problem**: Binary trust (full accept or reject) is too risky

**Fix**: Progressive trust with capability restrictions

**Timeline**: Can implement after Track 1 is working

**Teams**: All three (BearDog, Songbird, biomeOS)

---

## 📊 Track 1: Immediate Fix (Songbird)

### The Problem

**Current UDP Announcement**:
```rust
DiscoveryAnnouncement {
    peer_id: "pop-os",
    version: "v3.0",
    capabilities: ["orchestration", "federation"],
    endpoint: "https://192.168.1.144:8080",
    // genetic_lineage_tag: MISSING!
}
```

**Result**: Peer receives announcement but has NO lineage information

**BearDog sees**: `peer_has_no_genetic_lineage` → Cannot evaluate trust

---

### The Fix

**Add genetic lineage to announcements**:
```rust
// On startup, query BearDog for identity
let identity = query_beardog_identity("http://localhost:9000/api/v1/trust/identity").await?;

// Include in discovery announcement
DiscoveryAnnouncement {
    peer_id: "pop-os",
    version: "v3.0",
    capabilities: ["orchestration", "federation"],
    endpoint: "https://192.168.1.144:8080",
    identity_attestations: identity.identity_attestations,  // ADD THIS!
    // OR simpler:
    genetic_lineage_tag: Some(identity.encryption_tag),  // "beardog:family:iidn:pop-os_abc"
}
```

---

### Testing Track 1

```rust
#[tokio::test]
async fn test_lineage_in_discovery_packet() {
    let tower = start_tower_with_lineage("iidn").await;
    
    // Capture UDP packet
    let packet = capture_next_discovery_packet().await;
    
    // VERIFY: Packet contains lineage
    assert!(packet.contains("identity_attestations"));
    assert!(packet.contains("iidn"));
    
    // OR if using simpler format:
    assert!(packet.contains("genetic_lineage_tag"));
    assert!(packet.contains("beardog:family:iidn:"));
}
```

---

## 🏗️ Track 2: Progressive Trust Model (All Teams)

### The Core Insight

**From User**: "BirdSong doesn't put the bird at risk"

**Translation**:
- Genetic lineage = "I recognize you're from my species"
- NOT "Come into my nest"
- NOT "Access my resources"
- JUST "I'll listen to your song"

**Trust should progress**:
1. **Level 1**: Same family → Can coordinate (BirdSong)
2. **Level 2**: Human approves → Can federate
3. **Level 3**: Human adds entropy → Can access sensitive operations

---

### Trust Levels Defined

```rust
pub enum TrustLevel {
    /// No lineage or different family → Reject
    None = 0,
    
    /// Same genetic family → LIMITED coordination
    /// Can: Hear BirdSong, see capabilities, health checks
    /// CANNOT: Access data, execute commands, full federation
    Limited = 1,
    
    /// Human approved → FULL federation
    /// Can: Everything in Limited + federation + resource sharing
    /// CANNOT: Sensitive operations, key access
    Elevated = 2,
    
    /// Human entropy added → HIGHEST trust
    /// Can: Everything including sensitive operations
    Highest = 3,
}
```

---

## 🐻 BearDog Changes (Track 2)

### Current API

```rust
// GET /api/v1/trust/identity
{
  "encryption_tag": "beardog:family:iidn:pop-os_abc",
  "family_id": "iidn",
  "identity_attestations": [...]
}

// POST /api/v1/trust/evaluate
Request: {
  "peer_id": "tower2",
  "peer_tags": ["beardog:family:iidn:tower2"],
  "connection_info": {...}
}

Response: {
  "decision": "auto_accept",  // Binary: accept or reject
  "confidence": 1.0,
  "reason": "same_genetic_family"
}
```

---

### Proposed API (Track 2)

```rust
// POST /api/v1/trust/evaluate (ENHANCED)
Request: {
  "peer_id": "tower2",
  "peer_tags": ["beardog:family:iidn:tower2"],
  "connection_info": {...},
  "requested_operation": "data/read"  // NEW: What operation?
}

Response: {
  "decision": "limited_accept",  // NEW: More than binary
  "trust_level": 1,  // NEW: 0-3
  "confidence": 1.0,
  "reason": "same_genetic_family",
  "reason_code": "same_genetic_family",
  
  // NEW: Capability restrictions
  "allowed_capabilities": [
    "discovery",
    "coordination/birdsong",
    "health",
    "capabilities"
  ],
  "denied_capabilities": [
    "data/*",
    "commands/*",
    "federation/*",
    "keys/*"
  ],
  
  // NEW: How to elevate trust
  "elevation_path": {
    "next_level": 2,
    "requirements": ["human_approval"],
    "method": "user_consent_ui"
  }
}
```

---

### BearDog Implementation

```rust
pub fn evaluate_trust_with_capabilities(
    &self,
    request: TrustEvaluationRequest,
) -> TrustEvaluationResponse {
    // Step 1: Determine base trust level
    let trust_level = if same_genetic_family(request.peer_tags) {
        TrustLevel::Limited  // Same family = limited trust
    } else if different_family(request.peer_tags) {
        TrustLevel::None  // Different family = no auto-trust
    } else {
        TrustLevel::None  // No lineage = no trust
    };
    
    // Step 2: Define allowed capabilities for this level
    let (allowed, denied) = match trust_level {
        TrustLevel::Limited => (
            vec!["discovery", "coordination/*", "health", "capabilities"],
            vec!["data/*", "commands/*", "federation/*", "keys/*"]
        ),
        TrustLevel::Elevated => (
            vec!["discovery", "coordination/*", "health", "capabilities", "federation/*", "data/read"],
            vec!["data/write", "commands/sensitive", "keys/*"]
        ),
        TrustLevel::Highest => (
            vec!["*"],  // Everything
            vec![]  // Nothing denied
        ),
        _ => (vec![], vec!["*"]),  // Nothing allowed
    };
    
    // Step 3: Check if requested operation is allowed
    let operation_allowed = if let Some(op) = request.requested_operation {
        is_operation_allowed(&op, &allowed, &denied)
    } else {
        true  // No specific operation requested
    };
    
    // Step 4: Return response
    TrustEvaluationResponse {
        decision: if operation_allowed && trust_level > TrustLevel::None {
            format!("{}_accept", trust_level)
        } else {
            "reject".to_string()
        },
        trust_level: trust_level as u8,
        allowed_capabilities: allowed,
        denied_capabilities: denied,
        elevation_path: Some(ElevationPath {
            next_level: trust_level as u8 + 1,
            requirements: vec!["human_approval".to_string()],
            method: "user_consent_ui".to_string(),
        }),
        // ... existing fields
    }
}
```

---

### New BearDog API: Elevation

```rust
// POST /api/v1/trust/elevate
Request: {
  "peer_id": "tower2",
  "current_level": 1,
  "requested_level": 2,
  "evidence": {
    "type": "human_approval",
    "timestamp": "2026-01-03T16:00:00Z",
    "method": "user_consent_ui"
  }
}

Response: {
  "success": true,
  "new_level": 2,
  "message": "Trust elevated to level 2 (Elevated)",
  "new_allowed_capabilities": [...]
}
```

---

## 🐦 Songbird Changes

### Track 1: Immediate Fix (CRITICAL)

```rust
// On startup
async fn initialize_discovery(&mut self) -> Result<()> {
    // Query BearDog for identity
    let identity = self.query_beardog_identity().await?;
    
    // Store for discovery announcements
    self.genetic_lineage = Some(identity.encryption_tag);
    self.identity_attestations = Some(identity.identity_attestations);
    
    // Start discovery with lineage
    self.start_discovery().await?;
    
    Ok(())
}

// In discovery announcements
fn build_announcement(&self) -> DiscoveryAnnouncement {
    DiscoveryAnnouncement {
        peer_id: self.id.clone(),
        version: "v3.0".to_string(),
        capabilities: self.capabilities.clone(),
        endpoint: self.endpoint.clone(),
        
        // ADD THIS!
        identity_attestations: self.identity_attestations.clone(),
        // OR simpler:
        genetic_lineage_tag: self.genetic_lineage.clone(),
    }
}
```

---

### Track 2: Capability Enforcement (Long-term)

```rust
async fn handle_discovered_peer(&mut self, peer: DiscoveredPeer) -> Result<()> {
    // Evaluate trust with BearDog
    let trust = self.evaluate_trust(&peer).await?;
    
    // Establish connection based on trust level
    match trust.trust_level {
        0 => {
            info!("❌ Rejecting peer {} (no lineage)", peer.id);
            return Ok(());
        }
        
        1 => {
            info!("🔒 Limited connection to {} (same family)", peer.id);
            
            // Establish LIMITED connection
            let connection = self.establish_limited_connection(
                peer.clone(),
                trust.allowed_capabilities.clone()
            ).await?;
            
            // Store connection with restrictions
            self.connections.insert(peer.id.clone(), Connection {
                peer,
                trust_level: 1,
                allowed_capabilities: trust.allowed_capabilities,
                connection_type: ConnectionType::Limited(connection),
            });
            
            // Notify user about peer
            self.notify_user_of_limited_peer(&peer, &trust).await?;
        }
        
        2 => {
            info!("✅ Full federation with {} (human approved)", peer.id);
            self.establish_full_federation(peer).await?;
        }
        
        3 => {
            info!("🔓 Highest trust with {} (human entropy)", peer.id);
            self.establish_full_trust(peer).await?;
        }
        
        _ => unreachable!(),
    }
    
    Ok(())
}
```

---

### Capability Filtering

```rust
async fn call_peer_operation(
    &self,
    peer_id: &str,
    operation: &str,
    request: serde_json::Value,
) -> Result<serde_json::Value> {
    // Get connection
    let connection = self.connections.get(peer_id)
        .ok_or_else(|| anyhow!("Peer not connected"))?;
    
    // Check if operation is allowed at this trust level
    if !is_operation_allowed(operation, &connection.allowed_capabilities) {
        return Err(anyhow!(
            "Operation '{}' not allowed for peer '{}' at trust level {}",
            operation,
            peer_id,
            connection.trust_level
        ));
    }
    
    // Make the call
    connection.call(operation, request).await
}
```

---

## 🏗️ biomeOS Changes (Track 2)

### Universal Primal Client Enhancement

```rust
impl UniversalPrimalClient {
    pub async fn call<Req, Res>(
        &self,
        primal: &PrimalHandle,
        operation: &str,
        request: Req,
    ) -> Result<Res>
    where
        Req: Serialize + Send,
        Res: DeserializeOwned + Send,
    {
        // NEW: Check trust level and capability restrictions
        self.enforce_capability_restrictions(primal, operation).await?;
        
        // Existing logic
        let endpoint = primal.primary_endpoint()
            .ok_or_else(|| ApiError::Other { ... })?;
        
        let url = format!("{}/api/v1/{}", endpoint.url, operation);
        let body = serde_json::to_vec(&request)?;
        
        let response = self.protocol_adapter
            .request(&url, Method::POST, Some(body))
            .await?;
        
        let result: Res = self.format_adapter.parse(response).await?;
        Ok(result)
    }
    
    async fn enforce_capability_restrictions(
        &self,
        primal: &PrimalHandle,
        operation: &str,
    ) -> Result<()> {
        // Query trust level for this primal
        let trust = self.get_trust_level(primal).await?;
        
        // Check if operation is allowed
        if !self.is_operation_allowed(operation, &trust.allowed_capabilities) {
            return Err(ApiError::Forbidden {
                message: format!(
                    "Operation '{}' requires trust level {}, but primal has level {}",
                    operation,
                    required_level_for_operation(operation),
                    trust.trust_level
                ),
            });
        }
        
        Ok(())
    }
}
```

---

## 🧪 Testing Strategy

### Track 1 Tests (Immediate)

```rust
#[tokio::test]
async fn test_lineage_in_udp_packet() {
    let tower = start_tower("iidn").await;
    let packet = capture_udp_packet().await;
    
    assert!(packet.contains("identity_attestations"));
    assert!(packet.contains("iidn"));
}

#[tokio::test]
async fn test_peer_can_extract_lineage() {
    let tower1 = start_tower("iidn").await;
    let tower2 = start_tower("iidn").await;
    
    // Wait for discovery
    sleep(Duration::from_secs(5)).await;
    
    // Verify tower2 extracted tower1's lineage
    let peers = tower2.discovered_peers();
    assert_eq!(peers.len(), 1);
    assert_eq!(peers[0].family_id, Some("iidn".to_string()));
}
```

---

### Track 2 Tests (Long-term)

```rust
#[tokio::test]
async fn test_limited_trust_restricts_operations() {
    let tower1 = start_tower("iidn").await;
    let tower2 = start_tower("iidn").await;
    
    wait_for_discovery().await;
    
    // At trust level 1, can call health
    let result = tower2.call(&tower1, "health").await;
    assert!(result.is_ok());
    
    // But CANNOT call data/read
    let result = tower2.call(&tower1, "data/read").await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ApiError::Forbidden { .. }));
}

#[tokio::test]
async fn test_trust_elevation() {
    let tower1 = start_tower("iidn").await;
    let tower2 = start_tower("iidn").await;
    
    wait_for_discovery().await;
    
    // Initially level 1
    let trust = tower2.get_trust_level(&tower1).await?;
    assert_eq!(trust.level, 1);
    
    // Simulate human approval
    tower2.simulate_human_approval(&tower1).await?;
    
    // Now level 2
    let trust = tower2.get_trust_level(&tower1).await?;
    assert_eq!(trust.level, 2);
    
    // Can now call data/read
    let result = tower2.call(&tower1, "data/read").await;
    assert!(result.is_ok());
}
```

---

## 📋 Implementation Timeline

### Week 1: Track 1 (Critical Path)

**Monday-Tuesday**: Songbird team implements lineage advertisement  
**Wednesday**: Deploy to both towers, test discovery  
**Thursday**: Verify federation works (with current binary trust)  
**Friday**: Document success and gaps

---

### Week 2-3: Track 2 Design

**Week 2**: 
- Design trust level system in detail
- Define capability taxonomies
- Design elevation APIs

**Week 3**:
- BearDog: Implement multi-level trust responses
- Songbird: Implement capability enforcement
- biomeOS: Implement operation filtering

---

### Week 4: Track 2 Implementation

**BearDog**: New trust evaluation API with levels  
**Songbird**: Limited connection establishment  
**biomeOS**: Universal client capability filtering

---

### Week 5: Human Interaction

**UI**: User consent prompts  
**Elevation**: Trust elevation flows  
**Human Entropy**: Phone HSM / SoloKey integration

---

## 🎯 Success Criteria

### Track 1 (Immediate)

✅ Lineage visible in UDP packets  
✅ Peers can extract family_id  
✅ BearDog can compare families  
✅ Auto-accept works for same family  
✅ Historic first federation achieved

---

### Track 2 (Long-term)

✅ Multi-level trust implemented  
✅ Capability restrictions enforced  
✅ Operations filtered by trust level  
✅ Human approval flow working  
✅ Trust elevation tested  
✅ E2E tests with trust levels passing

---

## 🔒 Security Benefits of Track 2

### Current Risk (Binary Trust)

⚠️ Compromised USB → Full access to all towers  
⚠️ No human oversight  
⚠️ No capability restrictions  
⚠️ All-or-nothing trust

---

### Reduced Risk (Progressive Trust)

✅ Compromised USB → Limited access (can only coordinate)  
✅ Human oversight required for federation  
✅ Capability-based restrictions  
✅ Progressive trust levels  
✅ Clear audit trail

---

## 📞 Questions & Clarifications

### For BearDog Team

**Q**: Can you implement multi-level trust responses?  
**Timeline**: Week 2-3

**Q**: Can you add elevation API?  
**Timeline**: Week 4

---

### For Songbird Team

**Q**: Can you fix lineage advertisement? (CRITICAL)  
**Timeline**: ASAP (Week 1)

**Q**: Can you implement capability enforcement?  
**Timeline**: Week 3-4

---

### For biomeOS Team

**Q**: Can you enhance Universal Client with capability filtering?  
**Timeline**: Week 3-4

**Q**: Can you build human approval UI?  
**Timeline**: Week 5

---

## 🎊 Conclusion

**Immediate**: We still need Track 1 (lineage advertisement) to proceed

**Long-term**: Track 2 (progressive trust) makes the system truly secure

**BirdSong Analogy**: Same family = can hear the song, NOT come in the nest

**User Was Right**: We need to think this through more deeply

---

**Status**: Deep analysis complete, two-track plan defined  
**Track 1**: Songbird fixes lineage advertisement (CRITICAL)  
**Track 2**: All teams implement progressive trust (IMPORTANT)

🔒 **Building sovereign, secure-by-default, human-centric trust!** 🔒

