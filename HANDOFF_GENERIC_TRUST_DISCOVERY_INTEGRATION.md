# 🤝 Handoff: Generic Trust & Discovery Integration

**Date**: January 3, 2026  
**To**: Songbird Team & BearDog Team  
**From**: biomeOS Integration Testing  
**Priority**: 🔥 **HIGH** - Blocking federation  
**Timeline**: Rest of day (January 3, 2026)

---

## 🎯 Executive Summary

**Status**: Two-tower federation **99% working** - only integration contract issues remain!

**Achievement**: 🎊
- ✅ UDP multicast discovery: Perfect
- ✅ Genetic lineage: Active on both towers
- ✅ Network connectivity: Excellent
- ✅ Services: All running, stable, secure

**Blocker**: 🚧
- ❌ Peers can't see each other's trust credentials
- ❌ API contract mismatch preventing automatic trust

**Request**: Evolve to **generic, capability-based integration** (not Songbird ↔ BearDog specific hardcoding)

---

## 🧬 Philosophy: Agnostic Integration

### The Vision

**DON'T DO THIS** ❌:
```rust
// Hardcoded, brittle, primal-specific
if security_provider == "beardog" {
    call_beardog_specific_api()
} else if security_provider == "toadstool_crypto" {
    call_toadstool_specific_api()
}
```

**DO THIS INSTEAD** ✅:
```rust
// Generic, capability-based, future-proof
let security_provider = discover_by_capability("security", "trust_evaluation");
let trust_decision = security_provider.evaluate_trust(
    peer_credentials,  // Generic structure
    context            // Generic structure
);
```

### Why This Matters

**Short-term**: Fixes Songbird ↔ BearDog today  
**Long-term**: Works with ToadStool crypto, future primals, hardware HSMs, etc.

**Principle**: Each primal should:
1. Advertise **what it can do** (capabilities)
2. Accept **generic, well-documented payloads**
3. Return **generic, well-documented responses**
4. **Never assume** who's calling or being called

---

## 🔍 Three Integration Gaps (Root Cause Analysis)

### Gap 1: Trust Credentials Not in Discovery 🚨

**Problem**: When Songbird discovers a peer via UDP multicast, the discovery announcement **doesn't include** the peer's trust credentials (genetic lineage tag).

**Impact**: 
- Peer is discovered ✅
- But peer's identity is unknown ❌
- Can't evaluate trust without identity

**Current Discovery Packet**:
```json
{
  "peer_id": "496fe99e-0c8f-5a10-8d76-a0d52db5ee92",
  "capabilities": ["orchestration", "federation"],
  "version": "3.0",
  "endpoint": "https://192.168.1.144:8080"
  // Missing: trust_credentials!
}
```

**Should Be**:
```json
{
  "peer_id": "496fe99e-0c8f-5a10-8d76-a0d52db5ee92",
  "capabilities": ["orchestration", "federation"],
  "version": "3.0",
  "endpoint": "https://192.168.1.144:8080",
  "trust_credentials": {
    "provider": "beardog",
    "tags": ["beardog:family:iidn:pop-os_e80d4c1b"],
    "lineage_id": "lineage:tower:1767374822:...",
    "family_id": "iidn"
  }
}
```

**BUT WAIT**: This is still too BearDog-specific! ⚠️

**Generic Solution**:
```json
{
  "peer_id": "496fe99e-0c8f-5a10-8d76-a0d52db5ee92",
  "capabilities": ["orchestration", "federation"],
  "version": "3.0",
  "endpoint": "https://192.168.1.144:8080",
  "identity_attestations": [
    {
      "provider_capability": "security/identity",
      "format": "tag_list",
      "data": {
        "tags": ["beardog:family:iidn:pop-os_e80d4c1b"],
        "family_id": "iidn"
      }
    }
  ]
}
```

**Why Generic?**
- Works with BearDog today
- Works with ToadStool tomorrow
- Works with hardware HSMs next week
- Works with future cryptographic systems

---

### Gap 2: Trust API Contract Mismatch 🚨

**Problem**: Songbird calls BearDog's trust evaluation API but **doesn't send required fields**.

**Error**:
```
Failed to deserialize JSON: missing field `peer_tags` at line 1 column 24
```

**What Songbird Currently Sends**:
```json
{
  "peer_id": "496fe99e-0c8f-5a10-8d76-a0d52db5ee92",
  "connection_info": {...}
  // Missing: peer_tags!
}
```

**What BearDog Expects**:
```json
{
  "peer_id": "...",
  "peer_tags": ["..."],  // REQUIRED!
  "context": {...}
}
```

**BUT WAIT**: This reveals the deeper issue! ⚠️

**Root Cause**: No standardized API contract for trust evaluation across primals.

**Generic Solution**: Define a **Universal Trust Evaluation Request** format.

---

### Gap 3: Response Format Mismatch 🚨

**Problem**: BearDog returns a valid response, but Songbird **can't parse it**.

**Error**:
```
Failed to parse security provider trust evaluation response
```

**Root Cause**: BearDog and Songbird have different ideas of what the response looks like.

**Generic Solution**: Define a **Universal Trust Evaluation Response** format.

---

## 🏗️ Proposed Solution: Universal Trust API

### Design Principles

1. **Capability-Based**: Discover trust providers by capability, not name
2. **Generic Payloads**: Work for any cryptographic system
3. **Extensible**: Easy to add new fields without breaking changes
4. **Self-Describing**: Include format/version info
5. **Provider-Agnostic**: Orchestrator doesn't need to know provider details

---

## 📋 Specification: Universal Trust Evaluation API

### Endpoint (for any security provider)

**Discovery**: By capability `"security/trust_evaluation"` or `"trust"`

**HTTP Method**: `POST`  
**Path**: `/api/v1/trust/evaluate` (or provider's equivalent)

**Content-Type**: `application/json`

---

### Request Format (Generic)

```json
{
  "request_format": "universal_trust_v1",
  "evaluator": {
    "peer_id": "string",
    "attestations": [
      {
        "provider": "string (optional)",
        "format": "string (e.g., 'tag_list', 'certificate', 'key')",
        "data": {}
      }
    ]
  },
  "context": {
    "discovery_method": "string (e.g., 'udp_multicast', 'manual', 'registry')",
    "first_seen_at": "ISO8601 timestamp",
    "endpoint": "string (e.g., 'https://192.168.1.144:8080')",
    "capabilities": ["string"],
    "custom": {}
  }
}
```

**Field Descriptions**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `request_format` | string | Yes | Version of this request format (e.g., "universal_trust_v1") |
| `evaluator.peer_id` | string | Yes | Unique ID of the peer being evaluated |
| `evaluator.attestations` | array | Yes | List of identity attestations from the peer |
| `attestations[].provider` | string | No | Optional hint about which provider issued this |
| `attestations[].format` | string | Yes | Format of the attestation data (extensible) |
| `attestations[].data` | object | Yes | The actual attestation data (format-specific) |
| `context` | object | Yes | Context about how/when this peer was discovered |
| `context.custom` | object | No | Provider-specific context (extensible) |

**Example (BearDog Genetic Lineage)**:
```json
{
  "request_format": "universal_trust_v1",
  "evaluator": {
    "peer_id": "496fe99e-0c8f-5a10-8d76-a0d52db5ee92",
    "attestations": [
      {
        "provider": "beardog",
        "format": "tag_list",
        "data": {
          "tags": ["beardog:family:iidn:pop-os_e80d4c1b"],
          "family_id": "iidn"
        }
      }
    ]
  },
  "context": {
    "discovery_method": "udp_multicast",
    "first_seen_at": "2026-01-02T17:32:47Z",
    "endpoint": "https://192.168.1.144:8080",
    "capabilities": ["orchestration", "federation"]
  }
}
```

**Example (Future: ToadStool Certificate)**:
```json
{
  "request_format": "universal_trust_v1",
  "evaluator": {
    "peer_id": "abc123",
    "attestations": [
      {
        "provider": "toadstool",
        "format": "x509_certificate",
        "data": {
          "certificate": "-----BEGIN CERTIFICATE-----...",
          "chain": ["..."]
        }
      }
    ]
  },
  "context": {
    "discovery_method": "mDNS",
    "first_seen_at": "2026-01-02T18:00:00Z",
    "endpoint": "https://10.0.0.5:9000"
  }
}
```

---

### Response Format (Generic)

```json
{
  "response_format": "universal_trust_v1",
  "decision": "string (auto_accept | prompt_user | reject)",
  "confidence": "number (0.0-1.0)",
  "reason": "string (human-readable)",
  "reason_code": "string (machine-readable)",
  "metadata": {
    "same_family": "boolean (optional)",
    "trust_level": "string (optional)",
    "provider": "string (optional)"
  },
  "expires_at": "ISO8601 timestamp (optional)",
  "custom": {}
}
```

**Field Descriptions**:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `response_format` | string | Yes | Version of this response format |
| `decision` | enum | Yes | One of: `auto_accept`, `prompt_user`, `reject` |
| `confidence` | number | Yes | Trust confidence (0.0 = no trust, 1.0 = full trust) |
| `reason` | string | Yes | Human-readable explanation |
| `reason_code` | string | Yes | Machine-readable code (e.g., `same_genetic_family`) |
| `metadata` | object | No | Additional context (provider-specific) |
| `expires_at` | string | No | When this trust decision expires (for caching) |
| `custom` | object | No | Provider-specific response data (extensible) |

**Decision Semantics**:

| Decision | Meaning | Action |
|----------|---------|--------|
| `auto_accept` | High trust, same family/group | Automatically form connection |
| `prompt_user` | Unknown or different family | Require user consent (UI) |
| `reject` | Known malicious or policy violation | Block connection, log event |

**Example (BearDog: Same Family)**:
```json
{
  "response_format": "universal_trust_v1",
  "decision": "auto_accept",
  "confidence": 1.0,
  "reason": "Same genetic family (iidn)",
  "reason_code": "same_genetic_family",
  "metadata": {
    "same_family": true,
    "family_id": "iidn",
    "provider": "beardog",
    "lineage_verified": true
  },
  "expires_at": "2026-01-03T17:32:47Z"
}
```

**Example (BearDog: Different Family)**:
```json
{
  "response_format": "universal_trust_v1",
  "decision": "prompt_user",
  "confidence": 0.5,
  "reason": "Valid lineage but different genetic family (xyz vs iidn)",
  "reason_code": "different_genetic_family",
  "metadata": {
    "same_family": false,
    "peer_family_id": "xyz",
    "our_family_id": "iidn",
    "provider": "beardog"
  }
}
```

**Example (BearDog: No Lineage)**:
```json
{
  "response_format": "universal_trust_v1",
  "decision": "prompt_user",
  "confidence": 0.0,
  "reason": "Peer has no genetic lineage",
  "reason_code": "peer_has_no_genetic_lineage",
  "metadata": {
    "provider": "beardog"
  }
}
```

**Example (Future: ToadStool Certificate Valid)**:
```json
{
  "response_format": "universal_trust_v1",
  "decision": "auto_accept",
  "confidence": 0.95,
  "reason": "Valid X.509 certificate from trusted CA",
  "reason_code": "valid_certificate",
  "metadata": {
    "provider": "toadstool",
    "certificate_valid": true,
    "issuer": "CN=ToadStool Root CA"
  }
}
```

---

## 📦 Task Breakdown

### 🐦 Songbird Team Tasks

#### Task 1: Add Identity Attestations to Discovery Announcements

**File**: `songbird-discovery/src/anonymous_discovery.rs` (or equivalent)

**Current** (simplified):
```rust
struct DiscoveryAnnouncement {
    peer_id: String,
    capabilities: Vec<String>,
    version: String,
    endpoint: String,
}
```

**New** (generic):
```rust
struct DiscoveryAnnouncement {
    peer_id: String,
    capabilities: Vec<String>,
    version: String,
    endpoint: String,
    identity_attestations: Vec<IdentityAttestation>,  // NEW!
}

struct IdentityAttestation {
    provider_capability: String,  // e.g., "security/identity"
    format: String,               // e.g., "tag_list"
    data: serde_json::Value,      // Flexible, provider-specific
}
```

**Implementation Steps**:
1. On startup, query security provider (BearDog) for identity: `GET /api/v1/trust/identity`
2. Cache the response: `{ tags: [...], family_id: "..." }`
3. Convert to generic `IdentityAttestation`:
   ```rust
   IdentityAttestation {
       provider_capability: "security/identity",
       format: "tag_list",
       data: json!({
           "tags": response.tags,
           "family_id": response.family_id
       })
   }
   ```
4. Include in every UDP multicast discovery announcement

**Acceptance Criteria**:
- ✅ Discovery packets include `identity_attestations` field
- ✅ Peers can see each other's identity attestations
- ✅ Works without knowing provider is BearDog (generic)

---

#### Task 2: Parse Peer Identity Attestations from Discovery

**File**: `songbird-discovery/src/anonymous_discovery.rs`

**Goal**: When receiving a discovery announcement from a peer, extract and store their identity attestations.

**Implementation**:
```rust
// When parsing discovery packet
let peer = DiscoveredPeer {
    peer_id: announcement.peer_id,
    endpoint: announcement.endpoint,
    capabilities: announcement.capabilities,
    identity_attestations: announcement.identity_attestations,  // NEW!
};
```

**Acceptance Criteria**:
- ✅ Peer identity attestations are extracted from discovery packets
- ✅ Stored in `DiscoveredPeer` struct
- ✅ Available for trust evaluation

---

#### Task 3: Call Trust API with Generic Payload

**File**: `songbird-orchestrator/src/trust/peer_trust.rs`

**Current** (broken):
```rust
let request = json!({
    "peer_id": peer.id,
    // Missing attestations!
});
```

**New** (generic):
```rust
let request = json!({
    "request_format": "universal_trust_v1",
    "evaluator": {
        "peer_id": peer.id,
        "attestations": peer.identity_attestations.iter().map(|att| {
            json!({
                "provider": att.provider_capability,
                "format": att.format,
                "data": att.data
            })
        }).collect::<Vec<_>>()
    },
    "context": {
        "discovery_method": peer.discovery_method,
        "first_seen_at": peer.first_seen_at.to_rfc3339(),
        "endpoint": peer.endpoint,
        "capabilities": peer.capabilities
    }
});
```

**Acceptance Criteria**:
- ✅ Request includes all required fields
- ✅ Attestations are passed through generically
- ✅ No BearDog-specific hardcoding

---

#### Task 4: Parse Generic Trust Response

**File**: `songbird-orchestrator/src/trust/peer_trust.rs`

**Current** (broken):
```rust
// Assumes BearDog-specific response format
```

**New** (generic):
```rust
#[derive(Deserialize)]
struct UniversalTrustResponse {
    response_format: String,
    decision: TrustDecision,  // auto_accept | prompt_user | reject
    confidence: f64,
    reason: String,
    reason_code: String,
    metadata: Option<serde_json::Value>,
    expires_at: Option<String>,
}

enum TrustDecision {
    AutoAccept,
    PromptUser,
    Reject,
}

// Parse response
let response: UniversalTrustResponse = serde_json::from_str(&body)?;

match response.decision {
    TrustDecision::AutoAccept => {
        info!("✅ AUTO-ACCEPT: {} (confidence: {})", response.reason, response.confidence);
        Ok(TrustResult::AutoAccept)
    }
    TrustDecision::PromptUser => {
        warn!("⚠️  PROMPT USER: {}", response.reason);
        Ok(TrustResult::PromptUser)
    }
    TrustDecision::Reject => {
        warn!("❌ REJECT: {}", response.reason);
        Ok(TrustResult::Reject)
    }
}
```

**Acceptance Criteria**:
- ✅ Parses generic response format
- ✅ Handles all three decision types
- ✅ Logs reason codes for debugging
- ✅ Works with any provider (not just BearDog)

---

### 🐻 BearDog Team Tasks

#### Task 1: Accept Generic Trust Evaluation Requests

**File**: `beardog-tunnel/src/api/trust.rs` (or equivalent)

**Current**:
```rust
#[derive(Deserialize)]
struct TrustEvaluationRequest {
    peer_id: String,
    peer_tags: Vec<String>,  // BearDog-specific!
    context: HashMap<String, String>,
}
```

**New** (generic, but backward compatible):
```rust
#[derive(Deserialize)]
struct TrustEvaluationRequest {
    // Option 1: New generic format
    request_format: Option<String>,
    evaluator: Option<EvaluatorInfo>,
    context: Option<serde_json::Value>,
    
    // Option 2: Legacy fields (for backward compatibility)
    peer_id: Option<String>,
    peer_tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct EvaluatorInfo {
    peer_id: String,
    attestations: Vec<IdentityAttestation>,
}

#[derive(Deserialize)]
struct IdentityAttestation {
    provider: Option<String>,
    format: String,
    data: serde_json::Value,
}
```

**Implementation**:
```rust
async fn evaluate_trust(req: TrustEvaluationRequest) -> Result<TrustEvaluationResponse> {
    // Support both formats
    let (peer_id, tags) = if let Some(ref fmt) = req.request_format {
        if fmt == "universal_trust_v1" {
            // New generic format
            let evaluator = req.evaluator.ok_or("Missing evaluator")?;
            let tags = extract_tags_from_attestations(&evaluator.attestations)?;
            (evaluator.peer_id, tags)
        } else {
            return Err("Unsupported request format");
        }
    } else {
        // Legacy format
        let peer_id = req.peer_id.ok_or("Missing peer_id")?;
        let tags = req.peer_tags.ok_or("Missing peer_tags")?;
        (peer_id, tags)
    };
    
    // Existing trust evaluation logic
    evaluate_genetic_lineage(&peer_id, &tags).await
}

fn extract_tags_from_attestations(attestations: &[IdentityAttestation]) -> Result<Vec<String>> {
    for att in attestations {
        if att.format == "tag_list" {
            if let Some(tags) = att.data.get("tags").and_then(|t| t.as_array()) {
                return Ok(tags.iter()
                    .filter_map(|t| t.as_str().map(String::from))
                    .collect());
            }
        }
    }
    Ok(vec![])  // No tags found (might be different attestation type)
}
```

**Acceptance Criteria**:
- ✅ Accepts new generic format (`universal_trust_v1`)
- ✅ Backward compatible with legacy format (if needed)
- ✅ Extracts tags from generic attestations
- ✅ No breaking changes

---

#### Task 2: Return Generic Trust Response

**File**: `beardog-tunnel/src/api/trust.rs`

**Current** (BearDog-specific):
```rust
#[derive(Serialize)]
struct TrustEvaluationResponse {
    success: bool,
    data: TrustDecisionData,
}
```

**New** (generic):
```rust
#[derive(Serialize)]
struct TrustEvaluationResponse {
    // Generic format
    response_format: String,
    decision: String,  // "auto_accept" | "prompt_user" | "reject"
    confidence: f64,
    reason: String,
    reason_code: String,
    metadata: serde_json::Value,
    expires_at: Option<String>,
    
    // Legacy wrapper (for backward compatibility)
    success: Option<bool>,
    data: Option<serde_json::Value>,
}
```

**Implementation**:
```rust
async fn evaluate_genetic_lineage(peer_id: &str, tags: &[String]) -> Result<TrustEvaluationResponse> {
    let decision = determine_trust_decision(tags).await?;
    
    let (decision_str, confidence, reason, reason_code, metadata) = match decision {
        TrustDecision::SameFamily(family_id) => (
            "auto_accept",
            1.0,
            format!("Same genetic family ({})", family_id),
            "same_genetic_family",
            json!({
                "same_family": true,
                "family_id": family_id,
                "provider": "beardog",
                "lineage_verified": true
            })
        ),
        TrustDecision::DifferentFamily { peer_family, our_family } => (
            "prompt_user",
            0.5,
            format!("Valid lineage but different genetic family ({} vs {})", peer_family, our_family),
            "different_genetic_family",
            json!({
                "same_family": false,
                "peer_family_id": peer_family,
                "our_family_id": our_family,
                "provider": "beardog"
            })
        ),
        TrustDecision::NoLineage => (
            "prompt_user",
            0.0,
            "Peer has no genetic lineage".to_string(),
            "peer_has_no_genetic_lineage",
            json!({
                "provider": "beardog"
            })
        ),
    };
    
    Ok(TrustEvaluationResponse {
        response_format: "universal_trust_v1".to_string(),
        decision: decision_str.to_string(),
        confidence,
        reason,
        reason_code: reason_code.to_string(),
        metadata,
        expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(24)),
        success: Some(true),  // Legacy
        data: None,           // Legacy (can omit)
    })
}
```

**Acceptance Criteria**:
- ✅ Returns generic response format (`universal_trust_v1`)
- ✅ Includes all required fields (decision, confidence, reason, reason_code)
- ✅ Metadata is extensible (JSON object)
- ✅ Works with Songbird's generic parser

---

#### Task 3: (Optional) Cache Trust Decisions

**Goal**: Avoid re-evaluating the same peer repeatedly.

**Implementation**:
```rust
struct TrustCache {
    cache: Arc<RwLock<HashMap<String, CachedTrustDecision>>>,
}

struct CachedTrustDecision {
    decision: TrustEvaluationResponse,
    expires_at: DateTime<Utc>,
}

impl TrustCache {
    async fn get_or_evaluate(&self, peer_id: &str, tags: &[String]) -> Result<TrustEvaluationResponse> {
        // Check cache
        if let Some(cached) = self.cache.read().await.get(peer_id) {
            if cached.expires_at > Utc::now() {
                return Ok(cached.decision.clone());
            }
        }
        
        // Evaluate
        let decision = evaluate_genetic_lineage(peer_id, tags).await?;
        
        // Cache
        let expires_at = decision.expires_at
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|| Utc::now() + Duration::hours(1));
        
        self.cache.write().await.insert(peer_id.to_string(), CachedTrustDecision {
            decision: decision.clone(),
            expires_at,
        });
        
        Ok(decision)
    }
}
```

**Acceptance Criteria**:
- ✅ Trust decisions cached for `expires_at` duration
- ✅ Reduces redundant evaluations
- ✅ Cache invalidation works

---

## 🧪 Integration Testing

### Test 1: Two Towers, Same Family

**Setup**:
- Tower A: USB family `iidn`
- Tower B: USB family `iidn`

**Expected**:
1. Both towers discover each other via UDP multicast ✅
2. Discovery packets include identity attestations ✅
3. Songbird extracts peer attestations ✅
4. Songbird calls BearDog trust API with generic payload ✅
5. BearDog evaluates: Same family → `auto_accept` ✅
6. Songbird parses response: `auto_accept` ✅
7. **Mesh connection formed automatically** ✅

---

### Test 2: Two Towers, Different Families

**Setup**:
- Tower A: USB family `iidn`
- Tower B: USB family `xyz`

**Expected**:
1. Both discover each other ✅
2. Identity attestations exchanged ✅
3. BearDog evaluates: Different families → `prompt_user` ✅
4. Songbird: Connection **not** formed automatically ✅
5. UI prompt: "Tower B (family xyz) wants to connect. Accept?" ⏸️

---

### Test 3: Tower with Lineage, Peer without Lineage

**Setup**:
- Tower A: USB family `iidn`
- Peer B: No genetic lineage (legacy)

**Expected**:
1. Discovery ✅
2. Peer B's attestations: empty or missing ✅
3. BearDog evaluates: No lineage → `prompt_user` ✅
4. Songbird: Connection **not** formed automatically ✅
5. Safe default behavior ✅

---

## 📊 Success Criteria

### Must Have (Today)

- ✅ Discovery announcements include identity attestations
- ✅ Peers can see each other's attestations
- ✅ Trust API accepts generic payloads
- ✅ Trust API returns generic responses
- ✅ Songbird ↔ BearDog integration works
- ✅ Same family → automatic mesh formation

### Should Have (Today)

- ✅ No BearDog-specific hardcoding in Songbird
- ✅ No Songbird-specific assumptions in BearDog
- ✅ Generic enough for future primals
- ✅ Good error messages for debugging

### Nice to Have (Future)

- ✅ Trust decision caching
- ✅ User consent UI (for `prompt_user`)
- ✅ Trust decision audit log
- ✅ Support for multiple attestation types (certificates, keys, etc.)

---

## 🎯 Timeline

**Total Estimated Time**: 4-6 hours

### Songbird Team

| Task | Estimated Time | Priority |
|------|----------------|----------|
| Add attestations to discovery | 1-2 hours | 🔥 High |
| Parse peer attestations | 30 min | 🔥 High |
| Call trust API with generic payload | 1 hour | 🔥 High |
| Parse generic trust response | 1 hour | 🔥 High |
| **Total** | **3.5-4.5 hours** | |

### BearDog Team

| Task | Estimated Time | Priority |
|------|----------------|----------|
| Accept generic trust requests | 1-2 hours | 🔥 High |
| Return generic trust responses | 1 hour | 🔥 High |
| Cache trust decisions (optional) | 1 hour | Medium |
| **Total** | **2-3 hours** | |

---

## 📝 Communication

### Coordination Points

**Checkpoint 1** (After 2 hours):
- Songbird: Discovery attestations implemented?
- BearDog: Generic request parsing implemented?
- Test: Can we see attestations in logs?

**Checkpoint 2** (After 4 hours):
- Songbird: Trust API call with generic payload?
- BearDog: Generic response returned?
- Test: Does trust evaluation work end-to-end?

**Final Test** (After 6 hours):
- Full two-tower test with automatic mesh formation
- Document any remaining gaps
- Plan next steps

---

## 🚀 Expected Outcome

**End of Day**:
- ✅ Two towers with same USB family automatically form mesh
- ✅ Generic trust evaluation working
- ✅ No primal-specific hardcoding
- ✅ Foundation for future primals (ToadStool, hardware HSMs, etc.)
- ✅ Production-ready federation

**Next Phase** (Future):
- User consent UI for `prompt_user` decisions
- Hardware HSM support
- ToadStool cryptography integration
- Audit logging
- Trust revocation

---

## 🙏 Thank You!

Thank you for your excellent work on both primals! The foundation is solid:
- UDP multicast: Perfect ✅
- Genetic lineage: Working ✅
- Services: Stable ✅

These integration fixes are the final piece to unlock automatic, secure-by-default federation.

**We're 99% there!** 🎉

---

*Generic Trust & Discovery Integration Handoff*  
*biomeOS Team*  
*January 3, 2026*

