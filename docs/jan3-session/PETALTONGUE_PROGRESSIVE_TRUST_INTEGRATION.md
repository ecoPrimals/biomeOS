# 🌸 PetalTongue Integration: Progressive Trust UI

**Date**: January 3, 2026  
**Context**: PetalTongue as universal interface for progressive trust elevation  
**Vision**: Multi-modal, accessible, capability-based trust management

---

## 🎯 Perfect Alignment

### PetalTongue's Current Capabilities

**Already Production-Ready** ✅:
- Grade: A (94/100)
- 151+ tests passing
- Capability-based architecture
- Zero hardcoding
- BiomeOS API client
- Multi-modal (visual + audio)
- Tool integration framework

**Key Features**:
1. ✅ **Runtime discovery** - Discovers primals dynamically
2. ✅ **Capability-based** - No hardcoded assumptions
3. ✅ **BiomeOS client** - Already integrated
4. ✅ **Tool framework** - External tool integration ready
5. ✅ **Accessibility** - Multi-modal (blind users supported)
6. ✅ **Digital sovereignty** - User-controlled, transparent

---

## 🔒 Integration with Progressive Trust Model

### The Perfect UI for Trust Elevation

**Why PetalTongue?**

1. **Already discovers primals** - Can show trust levels
2. **Tool framework** - Can integrate trust management tools
3. **BiomeOS client** - Can query Universal Primal Client
4. **Multi-modal** - Can alert via audio for blind users
5. **Capability-based** - Already understands primal capabilities
6. **Production-ready** - No development needed for basic integration

---

## 🏗️ Architecture Integration

### Current Stack (After Our Evolution)

```
┌─────────────────────────────────────────────────────────────┐
│ PetalTongue (UI)                                            │
│   • Visual topology                                         │
│   • Audio sonification                                      │
│   • Tool integration                                        │
└─────────────────┬───────────────────────────────────────────┘
                  │ HTTP
┌─────────────────▼───────────────────────────────────────────┐
│ biomeOS                                                     │
│   • Universal Primal Client (NEW)                           │
│   • Primal discovery                                        │
│   • API negotiation                                         │
└─────────────────┬───────────────────────────────────────────┘
                  │
      ┌───────────┼───────────┐
      │           │           │
┌─────▼────┐ ┌───▼────┐ ┌───▼────────┐
│ BearDog  │ │Songbird│ │ Other      │
│ (Trust)  │ │(Orch.) │ │ Primals... │
└──────────┘ └────────┘ └────────────┘
```

---

### Enhanced Stack (Progressive Trust UI)

```
┌─────────────────────────────────────────────────────────────┐
│ PetalTongue (Universal Interface)                           │
│   ┌─────────────────────────────────────────────────────┐   │
│   │ Trust Management View (NEW)                         │   │
│   │   • Discovered peers with trust levels              │   │
│   │   • Elevation prompts (human approval)              │   │
│   │   • Capability restrictions display                 │   │
│   │   • Multi-modal alerts (visual + audio)             │   │
│   └─────────────────────────────────────────────────────┘   │
│   │                                                         │
│   ┌─────────────────────────────────────────────────────┐   │
│   │ Existing Views                                      │   │
│   │   • Topology graph                                  │   │
│   │   • Timeline view                                   │   │
│   │   • Traffic view                                    │   │
│   │   • Tool integration                                │   │
│   └─────────────────────────────────────────────────────┘   │
└─────────────────┬───────────────────────────────────────────┘
                  │ HTTP
┌─────────────────▼───────────────────────────────────────────┐
│ biomeOS (Orchestration Layer)                              │
│   ┌─────────────────────────────────────────────────────┐   │
│   │ Universal Primal Client                             │   │
│   │   • Format adapters (wrapped/unwrapped/auto)        │   │
│   │   • Protocol adapters (HTTP/tarpc/gRPC)             │   │
│   │   • Trust level enforcement                         │   │
│   │   • Capability filtering                            │   │
│   └─────────────────────────────────────────────────────┘   │
└─────────────────┬───────────────────────────────────────────┘
                  │
      ┌───────────┼───────────────────┐
      │           │                   │
┌─────▼────────┐ ┌▼──────────┐ ┌─────▼──────────┐
│ BearDog      │ │ Songbird  │ │ Other Primals  │
│ (Trust)      │ │ (Orch.)   │ │                │
│              │ │           │ │ • NestGate     │
│ Multi-level  │ │ Limited   │ │ • Squirrel     │
│ trust        │ │ connection│ │ • ToadStool    │
│ responses    │ │ w/ caps   │ │ • ...          │
└──────────────┘ └───────────┘ └────────────────┘
```

---

## 🎨 UI/UX Design

### New Trust Management View

```rust
// In petal-tongue-ui/src/app.rs

impl App {
    fn render_trust_management_view(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔒 Trust Management");
        
        // Get peers from Songbird via biomeOS
        let peers = self.get_discovered_peers();
        
        for peer in peers {
            ui.horizontal(|ui| {
                // Peer info
                ui.label(&peer.name);
                
                // Trust level badge
                match peer.trust_level {
                    0 => ui.colored_label(Color32::RED, "❌ No Trust"),
                    1 => ui.colored_label(Color32::YELLOW, "🔒 Limited"),
                    2 => ui.colored_label(Color32::LIGHT_BLUE, "✅ Elevated"),
                    3 => ui.colored_label(Color32::GREEN, "🔓 Highest"),
                    _ => ui.label("Unknown"),
                };
                
                // Family info
                if let Some(family) = &peer.family_id {
                    ui.label(format!("Family: {}", family));
                }
                
                // Capabilities summary
                ui.label(format!(
                    "Can: {} | Cannot: {}",
                    peer.allowed_capabilities.len(),
                    peer.denied_capabilities.len()
                ));
                
                // Elevation button (if applicable)
                if peer.trust_level < 3 {
                    if ui.button("⬆ Elevate Trust").clicked() {
                        self.show_elevation_prompt(peer);
                    }
                }
            });
            
            // Collapsible details
            ui.collapsing("Details", |ui| {
                ui.label(format!("Peer ID: {}", peer.id));
                ui.label(format!("Endpoint: {}", peer.endpoint));
                
                ui.label("Allowed Capabilities:");
                for cap in &peer.allowed_capabilities {
                    ui.label(format!("  ✅ {}", cap));
                }
                
                ui.label("Denied Capabilities:");
                for cap in &peer.denied_capabilities {
                    ui.label(format!("  ❌ {}", cap));
                }
            });
            
            ui.separator();
        }
    }
    
    fn show_elevation_prompt(&mut self, peer: DiscoveredPeer) {
        // Modal dialog for elevation
        egui::Window::new("⬆ Elevate Trust")
            .collapsible(false)
            .show(ctx, |ui| {
                ui.heading(format!("Elevate trust for {}?", peer.name));
                
                ui.label(format!(
                    "This peer is from family '{}' ({})",
                    peer.family_id.unwrap_or("unknown".to_string()),
                    if peer.family_id == Some(self.my_family_id.clone()) {
                        "same as yours"
                    } else {
                        "different from yours"
                    }
                ));
                
                ui.label("Current capabilities:");
                ui.label(format!("  Can: {:?}", peer.allowed_capabilities));
                
                ui.label(format!("\nElevate to level {}?", peer.trust_level + 1));
                
                // Show what will be allowed
                let next_caps = self.get_capabilities_for_level(peer.trust_level + 1);
                ui.label("Will be allowed:");
                for cap in &next_caps.allowed {
                    ui.label(format!("  + {}", cap));
                }
                
                ui.horizontal(|ui| {
                    if ui.button("✅ Approve").clicked() {
                        self.elevate_peer_trust(peer.id, peer.trust_level + 1);
                        self.close_elevation_prompt();
                    }
                    
                    if ui.button("❌ Deny").clicked() {
                        self.close_elevation_prompt();
                    }
                });
            });
    }
}
```

---

### Audio Alerts for Blind Users

```rust
// In petal-tongue-graph/src/audio_sonification.rs

impl AudioSonification {
    /// Sonify trust level changes
    pub fn sonify_trust_alert(&self, peer: &DiscoveredPeer, event: TrustEvent) {
        match event {
            TrustEvent::NewPeerDiscovered => {
                // Rising chime for new peer
                let frequency = match peer.trust_level {
                    0 => 200.0,  // Low (no trust)
                    1 => 400.0,  // Medium (limited)
                    2 => 600.0,  // Higher (elevated)
                    3 => 800.0,  // High (highest)
                    _ => 300.0,
                };
                
                self.play_chime(frequency, Duration::from_millis(500));
                
                // Voice: "New peer discovered: <name>, trust level <level>"
                self.speak(format!(
                    "New peer discovered: {}, trust level {}",
                    peer.name, peer.trust_level
                ));
            }
            
            TrustEvent::ElevationRequested => {
                // Alert tone
                self.play_alert_tone();
                
                // Voice: "Peer <name> requests trust elevation. Do you approve?"
                self.speak(format!(
                    "Peer {} requests trust elevation to level {}. Do you approve?",
                    peer.name, peer.trust_level + 1
                ));
            }
            
            TrustEvent::TrustElevated => {
                // Success chime
                self.play_success_chime();
                
                // Voice: "Trust elevated for <name>"
                self.speak(format!("Trust elevated for {}", peer.name));
            }
        }
    }
}
```

---

## 🔌 Integration Points

### 1. BiomeOS API Extension

**New Endpoints for PetalTongue**:

```rust
// GET /api/v1/peers/discovered
// Returns list of discovered peers with trust levels

{
  "peers": [
    {
      "id": "tower2",
      "name": "tower2",
      "family_id": "iidn",
      "trust_level": 1,
      "allowed_capabilities": ["discovery", "coordination/*"],
      "denied_capabilities": ["data/*", "commands/*"],
      "elevation_available": true,
      "endpoint": "https://192.168.1.134:8080"
    }
  ]
}

// POST /api/v1/peers/{peer_id}/elevate
// Request to elevate peer trust

Request:
{
  "requested_level": 2,
  "evidence": {
    "type": "human_approval",
    "timestamp": "2026-01-03T16:00:00Z",
    "method": "petaltongue_ui"
  }
}

Response:
{
  "success": true,
  "new_level": 2,
  "new_allowed_capabilities": [...],
  "message": "Trust elevated to level 2"
}
```

---

### 2. PetalTongue BiomeOS Client Enhancement

```rust
// In petal-tongue-api/src/biomeos_client.rs

impl BiomeOSClient {
    /// Get discovered peers with trust levels
    pub async fn get_discovered_peers(&self) -> Result<Vec<DiscoveredPeer>> {
        let url = format!("{}/api/v1/peers/discovered", self.base_url);
        let response = self.client.get(&url).send().await?;
        let data: PeersResponse = response.json().await?;
        Ok(data.peers)
    }
    
    /// Elevate peer trust
    pub async fn elevate_peer_trust(
        &self,
        peer_id: &str,
        requested_level: u8,
    ) -> Result<ElevationResponse> {
        let url = format!("{}/api/v1/peers/{}/elevate", self.base_url, peer_id);
        let request = ElevationRequest {
            requested_level,
            evidence: ElevationEvidence {
                evidence_type: "human_approval".to_string(),
                timestamp: chrono::Utc::now(),
                method: "petaltongue_ui".to_string(),
            },
        };
        
        let response = self.client.post(&url).json(&request).send().await?;
        let data: ElevationResponse = response.json().await?;
        Ok(data)
    }
}
```

---

### 3. Universal Primal Client Integration

**PetalTongue uses biomeOS's Universal Primal Client** (already built):

```rust
// PetalTongue → biomeOS → Universal Primal Client → BearDog/Songbird

// In biomeOS Universal Primal Client
impl UniversalPrimalClient {
    /// Get discovered peers (queries Songbird via discovery)
    pub async fn get_discovered_peers(&self) -> Result<Vec<DiscoveredPeer>> {
        // Discover Songbird primal
        let songbird = self.discover_primal_by_capability("orchestration").await?;
        
        // Call Songbird's peer list API
        let response = self.call(&songbird, "peers/list", EmptyRequest).await?;
        
        // For each peer, get trust evaluation from BearDog
        let mut peers_with_trust = Vec::new();
        for peer in response.peers {
            let trust = self.evaluate_peer_trust(&peer).await?;
            peers_with_trust.push(DiscoveredPeer {
                id: peer.id,
                name: peer.name,
                family_id: trust.family_id,
                trust_level: trust.trust_level,
                allowed_capabilities: trust.allowed_capabilities,
                denied_capabilities: trust.denied_capabilities,
                elevation_available: trust.elevation_path.is_some(),
                endpoint: peer.endpoint,
            });
        }
        
        Ok(peers_with_trust)
    }
    
    async fn evaluate_peer_trust(&self, peer: &Peer) -> Result<TrustEvaluation> {
        // Discover BearDog primal
        let beardog = self.discover_primal_by_capability("security").await?;
        
        // Call BearDog's trust evaluation API
        let request = TrustEvaluationRequest {
            peer_id: peer.id.clone(),
            peer_tags: peer.tags.clone(),
            connection_info: peer.connection_info.clone(),
        };
        
        let response: TrustEvaluationResponse = self.call(&beardog, "trust/evaluate", request).await?;
        
        Ok(TrustEvaluation {
            trust_level: response.trust_level,
            family_id: response.family_id,
            allowed_capabilities: response.allowed_capabilities,
            denied_capabilities: response.denied_capabilities,
            elevation_path: response.elevation_path,
        })
    }
}
```

---

## 🎵 Multi-Modal Experience

### Visual Mode (Sighted Users)

```
┌────────────────────────────────────────────────────────────┐
│ PetalTongue - Trust Management                            │
├────────────────────────────────────────────────────────────┤
│                                                            │
│ 🔒 Discovered Peers                                        │
│                                                            │
│ ┌────────────────────────────────────────────────────┐    │
│ │ tower2        🔒 Limited       Family: iidn        │    │
│ │ Can: discovery, coordination                       │    │
│ │ Cannot: data/*, commands/*                         │    │
│ │ [⬆ Elevate Trust]                                  │    │
│ │                                                    │    │
│ │ Details ▼                                          │    │
│ │   Peer ID: e4c0e057-a3c8-5b59-9705-1520b199d607   │    │
│ │   Endpoint: https://192.168.1.134:8080            │    │
│ │   Allowed:                                         │    │
│ │     ✅ discovery                                    │    │
│ │     ✅ coordination/birdsong                       │    │
│ │   Denied:                                          │    │
│ │     ❌ data/*                                       │    │
│ │     ❌ commands/*                                   │    │
│ └────────────────────────────────────────────────────┘    │
│                                                            │
│ ┌────────────────────────────────────────────────────┐    │
│ │ pop-os        ✅ Elevated      Family: iidn        │    │
│ │ Can: discovery, coordination, federation           │    │
│ │ Cannot: keys/*, data/write                         │    │
│ │ [⬆ Elevate to Highest]                             │    │
│ └────────────────────────────────────────────────────┘    │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

---

### Audio Mode (Blind Users)

**Soundscape**:
- **Base tone**: Continuous ambient sound indicating system health
- **Peer tones**: Each peer has a unique instrument/frequency
- **Trust level**: Volume indicates trust level
  - Quiet = Limited
  - Medium = Elevated
  - Loud = Highest
- **Alerts**: Distinct chimes for new peers or elevation requests

**Voice Narration** (on demand or auto):
```
AI: "PetalTongue ready. 2 peers discovered."

AI: "Peer 1: tower2, Limited trust, same family."
    "Can coordinate. Cannot access data."
    "Elevate trust?"

User: "Yes"

AI: "Trust elevated to level 2. Full federation enabled."

AI: "Peer 2: pop-os, Elevated trust, same family."
    "Full federation active."

[Soundscape plays: Two tones, one for each peer]
```

---

## 🚀 Implementation Plan

### Phase 1: Basic Integration (1-2 days)

**biomeOS Side**:
1. Add `/api/v1/peers/discovered` endpoint
2. Add `/api/v1/peers/{id}/elevate` endpoint
3. Wire to Universal Primal Client

**PetalTongue Side**:
1. Enhance `BiomeOSClient` with trust APIs
2. Create basic trust management view
3. Display peers with trust levels

---

### Phase 2: Elevation UI (2-3 days)

**PetalTongue Side**:
1. Implement elevation prompt dialog
2. Add capability detail views
3. Integrate elevation API calls
4. Add success/error feedback

---

### Phase 3: Audio Alerts (1-2 days)

**PetalTongue Side**:
1. Implement trust event sonification
2. Add voice narration
3. Test with blind users
4. Refine audio UX

---

### Phase 4: Advanced Features (1-2 weeks)

**Both Sides**:
1. Human entropy integration (phone HSM, SoloKey)
2. Trust history/audit log
3. Capability usage metrics
4. Trust level analytics
5. Multi-tower visualization

---

## 🎊 Benefits of This Integration

### 1. For Users

**Accessibility**:
- ✅ Blind users can manage trust via audio
- ✅ Sighted users get visual topology + trust
- ✅ Multi-modal alerts (visual + audio)

**Sovereignty**:
- ✅ User controls trust elevation
- ✅ Transparent trust decisions
- ✅ Audit trail visible

**Usability**:
- ✅ One UI for everything
- ✅ Discover peers, see trust, elevate
- ✅ Capability-based restrictions visible

---

### 2. For Developers

**Clean Architecture**:
- ✅ PetalTongue stays capability-based
- ✅ biomeOS handles trust logic
- ✅ BearDog provides trust evaluation
- ✅ Songbird provides discovery
- ✅ Clear separation of concerns

**Reusability**:
- ✅ Universal Primal Client works for all primals
- ✅ PetalTongue's tool framework extends easily
- ✅ Trust UI can be reused by other clients

---

### 3. For Ecosystem

**Interoperability**:
- ✅ PetalTongue becomes reference UI implementation
- ✅ Other UIs can follow same patterns
- ✅ Progressive trust model is proven

**Evolution**:
- ✅ PetalTongue helps evolve primal APIs
- ✅ Multi-primal integration tested
- ✅ Capability discovery validated

---

## 📊 Technical Specifications

### Data Flow

```
User Action:
  "Elevate trust for tower2"
    ↓
PetalTongue UI:
  Show elevation prompt
    ↓
User confirms:
  "Yes, approve"
    ↓
PetalTongue BiomeOS Client:
  POST /api/v1/peers/tower2/elevate
  { "requested_level": 2, "evidence": {...} }
    ↓
biomeOS API:
  Receive request
    ↓
biomeOS Universal Primal Client:
  1. Discover BearDog
  2. Call BearDog /trust/elevate
  3. Update Songbird connection
    ↓
BearDog:
  Record elevation
  Return new trust evaluation
    ↓
Songbird:
  Update peer connection
  Apply new capability restrictions
    ↓
biomeOS:
  Return success to PetalTongue
    ↓
PetalTongue:
  Update UI
  Play success chime (audio)
  Update peer badge (visual)
```

---

### State Management

```rust
// In PetalTongue

struct TrustManagementState {
    discovered_peers: Vec<DiscoveredPeer>,
    my_family_id: String,
    elevation_in_progress: Option<String>,  // peer_id
    last_update: std::time::Instant,
    refresh_interval: std::time::Duration,
}

impl TrustManagementState {
    async fn refresh(&mut self, client: &BiomeOSClient) -> Result<()> {
        if self.last_update.elapsed() > self.refresh_interval {
            self.discovered_peers = client.get_discovered_peers().await?;
            self.last_update = std::time::Instant::now();
        }
        Ok(())
    }
}
```

---

## 🔒 Security Considerations

### 1. Elevation Requires Explicit User Action

- ❌ No auto-elevation without user
- ✅ User must click "Approve"
- ✅ User must understand what they're approving

### 2. Clear Capability Display

- ✅ Show what will be allowed/denied
- ✅ Use plain language, not jargon
- ✅ Visual + audio explanations

### 3. Audit Trail

- ✅ Log all elevation events
- ✅ Show trust history in UI
- ✅ User can review past decisions

---

## 🎯 Success Metrics

### Integration Success

- [ ] PetalTongue connects to biomeOS trust APIs
- [ ] Discovered peers displayed with trust levels
- [ ] Elevation prompts work
- [ ] Trust elevation succeeds
- [ ] UI updates reflect new trust levels

### Audio Success

- [ ] Blind user can hear new peer alerts
- [ ] Voice narration explains trust levels
- [ ] Audio feedback for elevation success

### E2E Success

- [ ] Two towers discover each other
- [ ] PetalTongue shows both with Limited trust
- [ ] User elevates one to Elevated
- [ ] Full federation established
- [ ] Capability restrictions enforced

---

## 📋 Next Steps

### Immediate (This Week)

1. **biomeOS**: Implement `/api/v1/peers/discovered` endpoint
2. **biomeOS**: Implement `/api/v1/peers/{id}/elevate` endpoint
3. **PetalTongue**: Enhance `BiomeOSClient` with trust methods
4. **PetalTongue**: Create basic trust management view

### Short-Term (Next Week)

1. **PetalTongue**: Implement elevation prompt UI
2. **PetalTongue**: Add audio alerts for trust events
3. **Integration**: Test two-tower elevation
4. **Documentation**: User guide for trust management

### Long-Term (Next Month)

1. Human entropy integration
2. Trust analytics and history
3. Advanced multi-tower visualizations
4. Community user testing

---

## 🎊 Conclusion

**PetalTongue is the PERFECT UI for progressive trust!**

**Why**:
1. ✅ Already production-ready (Grade A)
2. ✅ Capability-based architecture (aligns perfectly)
3. ✅ BiomeOS client (easy integration)
4. ✅ Multi-modal (accessible to all)
5. ✅ Tool framework (extends naturally)
6. ✅ Zero hardcoding (can evolve dynamically)

**Impact**:
- ✅ Progressive trust becomes **tangible** and **usable**
- ✅ Blind users can manage trust via audio
- ✅ One UI for discovery, topology, AND trust
- ✅ Proves the entire architecture works E2E
- ✅ Helps evolve primal APIs through real usage

---

**Status**: Architecture defined, ready to implement  
**Timeline**: 1-2 weeks for Phase 1-2, ~1 month for full integration  
**Confidence**: HIGH - PetalTongue's architecture is perfectly aligned

🌸 **PetalTongue: The universal tongue for trust!** 🔒🚀

