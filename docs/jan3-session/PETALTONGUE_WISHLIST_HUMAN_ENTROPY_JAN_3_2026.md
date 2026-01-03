# 🌸 PetalTongue x biomeOS MVP - Wishlist & Vision

**Date**: January 3, 2026 (Evening)  
**Status**: Strategic Planning Document  
**Vision**: PetalTongue as the universal interface for human-centered, sovereign trust

---

## 🎯 Executive Vision

**Core Concept**: PetalTongue becomes the **human entropy capture interface** for the ecoPrimals ecosystem, specifically feeding BearDog's genetic lineage system while handling all user complexity.

**Philosophy**: 
- **Sing a song** → Audio entropy
- **Take a video** → Visual entropy
- **Tell a story** → Narrative entropy  
- **Paint a picture** → Creative entropy

PetalTongue captures the richness, BearDog secures the lineage.

---

## 📋 Wishlist by Priority

### 🔥 Priority 1: MVP Foundation (Week 1)

#### 1.1 Topology Edge Visualization Fix ⭐
**Status**: Minor issue blocking edge display  
**Need**: 
- Parse biomeOS topology response format: `{"nodes": [...], "edges": [...]}`
- Display edges between primals in force-directed graph
- Show trust relationships visually

**Impact**: Medium (nodes work, edges needed for full picture)  
**Effort**: 1-2 hours (PetalTongue team)

#### 1.2 Trust Level Visualization ⭐⭐
**Status**: Track B Phase 2 (ready to implement)  
**Need**:
- Color-code nodes by trust level (0-3)
  - Level 0 (None): Red
  - Level 1 (Limited): Yellow
  - Level 2 (Elevated): Blue
  - Level 3 (Highest): Green
- Show trust level in node labels
- Animate trust elevation changes

**Impact**: High (core MVP feature)  
**Effort**: 1-2 days (PetalTongue team)

#### 1.3 Genetic Lineage Display ⭐⭐⭐
**Status**: BearDog data available, UI needed  
**Need**:
- Display `family_id` in node info panel
- Show genetic lineage relationships
- Visual indicator for same-family vs different-family
- Lineage tree/graph view (optional)

**Impact**: High (key security feature)  
**Effort**: 2-3 days (PetalTongue team)

---

### 🎨 Priority 2: Human Entropy Capture (Week 2-3)

#### 2.1 Audio Entropy Capture 🎵
**Vision**: Sing a song, hum a tune, speak a passphrase  
**Need**:
- Audio recording interface in PetalTongue
- Real-time waveform visualization
- Duration: 10-30 seconds recommended
- Format: Raw audio samples (high entropy)
- Privacy: Never saved to disk, direct stream to BearDog

**Architecture**:
```
PetalTongue (UI)
  ↓ [Audio Stream - WebRTC/Raw Bytes]
biomeOS API (Proxy)
  ↓ [Secure Channel]
BearDog Tunnel (Entropy Processor)
  ↓ [Mix with USB Seed]
Genetic Lineage (Unique Node Identity)
```

**Security**:
- Stream-only (never write to disk)
- Ephemeral buffer (cleared after use)
- BearDog handles all crypto operations
- PetalTongue only provides UI

**Impact**: Revolutionary (human-centric trust)  
**Effort**: 1 week (collaborative: PetalTongue UI + biomeOS proxy + BearDog processor)

#### 2.2 Video Entropy Capture 📹
**Vision**: Record yourself, show your environment, gesture authentication  
**Need**:
- Video recording interface (webcam)
- Preview window
- Duration: 5-15 seconds
- Format: Raw video frames or hashed samples
- Privacy: Never saved, stream only

**Use Cases**:
- Facial recognition (optional, privacy-aware)
- Gesture-based authentication
- Environmental uniqueness capture
- Liveness detection

**Impact**: High (multi-modal entropy)  
**Effort**: 1.5 weeks (collaborative)

#### 2.3 Story/Text Entropy Capture 📝
**Vision**: Tell a story, write a memory, describe your tower  
**Need**:
- Text input dialog
- Markdown support (optional)
- Character limit: 100-500 chars recommended
- Typing pattern analysis (keystroke dynamics)
- Privacy: Never saved, sent directly to BearDog

**Use Cases**:
- Personal narrative
- Tower name/description
- Security questions (but not traditional)
- Passphrase with meaning

**Impact**: Medium (accessible to all)  
**Effort**: 3-5 days (collaborative)

#### 2.4 Drawing/Painting Entropy Capture 🎨
**Vision**: Paint a picture, doodle, sign your name  
**Need**:
- Canvas interface (HTML5 canvas or similar)
- Drawing tools (brush, eraser, colors)
- Pressure sensitivity (if available)
- Stroke timing/pattern capture
- Privacy: Never saved, stroke data to BearDog

**Use Cases**:
- Signature authentication
- Creative expression
- Mouse/stylus dynamics
- Spatial pattern uniqueness

**Impact**: High (creative, accessible, high entropy)  
**Effort**: 1 week (collaborative)

---

### 🔒 Priority 3: Progressive Trust UI (Week 3-4)

#### 3.1 Trust Decision Dialog ⭐⭐⭐
**Status**: Required for Track 2 completion  
**Need**:
- Modal dialog when new peer discovered
- Display peer information:
  - Node ID
  - Family ID (same/different)
  - Capabilities
  - Current trust level
- Actions:
  - Reject (Level 0)
  - Accept Limited (Level 1)
  - Elevate to Federated (Level 2) - requires human entropy
  - Grant Full Trust (Level 3) - requires human entropy + approval

**Impact**: Critical (Track 2 blocker)  
**Effort**: 3-5 days (PetalTongue team)

#### 3.2 Trust Elevation Workflow
**Status**: Architecture defined, needs implementation  
**Need**:
- Step 1: Select peer to elevate
- Step 2: Choose entropy method (audio/video/story/drawing)
- Step 3: Capture entropy via PetalTongue
- Step 4: Stream to BearDog for mixing
- Step 5: Confirm elevation
- Step 6: Visual feedback (node color change)

**Impact**: High (human-centric trust)  
**Effort**: 1 week (collaborative)

#### 3.3 Trust Audit Trail
**Status**: Nice-to-have for MVP  
**Need**:
- Log of trust decisions
- Who elevated whom, when, how
- Entropy method used
- Trust level history
- Revocation capability

**Impact**: Medium (security audit)  
**Effort**: 3-5 days (PetalTongue team)

---

### 🏗️ Priority 4: Architecture & Integration (Ongoing)

#### 4.1 BearDog Entropy API 🔐
**Status**: Needs design & implementation  
**Need** (BearDog team):
- `POST /api/v1/entropy/audio` - Stream audio samples
- `POST /api/v1/entropy/video` - Stream video frames
- `POST /api/v1/entropy/text` - Submit text entropy
- `POST /api/v1/entropy/drawing` - Submit stroke data
- All endpoints:
  - Accept streaming data (chunked)
  - Mix with USB seed + local entropy
  - Return derived keys/identity updates
  - Never persist raw entropy

**Impact**: Critical (enables all entropy capture)  
**Effort**: 1-2 weeks (BearDog team)

#### 4.2 biomeOS Entropy Proxy
**Status**: Needs design & implementation  
**Need** (biomeOS team):
- Proxy endpoints: `/api/v1/entropy/*`
- Stream-through architecture (no buffering)
- Routing to BearDog tunnel
- Security: TLS, authentication
- Never log entropy data

**Impact**: Critical (secure channel)  
**Effort**: 3-5 days (biomeOS team)

#### 4.3 PetalTongue Entropy Modules
**Status**: Modular architecture needed  
**Need** (PetalTongue team):
- Module: Audio capture (microphone access)
- Module: Video capture (webcam access)
- Module: Text input (secure input field)
- Module: Drawing canvas (stroke capture)
- All modules:
  - Stream-only (no persistence)
  - Clear buffers after use
  - User consent dialogs
  - Privacy indicators (recording indicator)

**Impact**: High (user experience)  
**Effort**: 2 weeks (PetalTongue team)

---

## 🎨 User Experience Vision

### Scenario 1: First Tower Setup (Human Entropy Mixing)

**Current (USB Seed Only)**:
```
1. Plug in USB with family seed
2. Tower derives identity automatically
3. Result: Same family, but predictable identity
```

**Vision (USB Seed + Human Entropy)**:
```
1. Plug in USB with family seed
2. PetalTongue launches: "Welcome! Let's make this tower uniquely yours."
3. Choose entropy method:
   🎵 "Sing a song for 15 seconds"
   📹 "Record a video greeting"
   📝 "Tell us about your tower"
   🎨 "Draw your tower's symbol"
4. PetalTongue captures, streams to BearDog
5. BearDog mixes: USB seed + local entropy + human entropy
6. Result: Same family, unique identity, human-centered
```

### Scenario 2: Trust Elevation (Peer Approval)

**Vision**:
```
1. New peer discovered: "tower2" (family: iidn, trust: Limited)
2. PetalTongue shows dialog:
   "tower2 wants to join federation"
   Current: Level 1 (BirdSong only)
   Elevate to: Level 2 (Full federation)
3. User clicks "Elevate"
4. PetalTongue: "Let's verify this is really you. Choose a method:"
   🎵 "Sing the family song"
   📹 "Show your face"
   📝 "Tell the secret story"
   🎨 "Draw the family symbol"
5. User provides entropy
6. BearDog verifies + elevates trust
7. Visual feedback: Node color changes Yellow → Blue
8. Federation established with human approval
```

---

## 🔐 Security & Privacy Principles

### Never Persist Raw Entropy
- All entropy data streams through memory only
- No disk writes, no logs, no cache
- Buffers cleared immediately after use
- BearDog processes and discards raw data

### User Consent & Awareness
- Explicit consent before capture
- Visual indicators when recording
- Clear explanation of what entropy is used for
- Right to refuse/cancel at any time

### Sovereignty & Control
- User chooses entropy method
- User sees what's captured (preview)
- User can retry if unhappy
- No remote access to capture devices

### Privacy-First Design
- Entropy never leaves local network (LAN only for MVP)
- TLS for all communications
- BearDog validates but doesn't store
- PetalTongue doesn't analyze, just captures

---

## 📊 MVP Scope (Realistic)

### Must-Have (Week 1-2)
1. ✅ Topology edge visualization fix
2. ✅ Trust level color-coding
3. ✅ Genetic lineage display
4. ✅ One entropy method: Audio OR Text (simplest first)

### Should-Have (Week 3-4)
1. ✅ Trust decision dialog
2. ✅ Trust elevation workflow
3. ✅ Second entropy method (complete audio + text)

### Nice-to-Have (Week 5-6)
1. 🔄 Video entropy capture
2. 🔄 Drawing entropy capture
3. 🔄 Trust audit trail
4. 🔄 Multi-modal entropy (combine methods)

---

## 🎯 Success Metrics

### Technical
- ✅ Zero entropy data persisted to disk
- ✅ Stream latency < 100ms
- ✅ Successful entropy mixing (BearDog confirms)
- ✅ Trust elevation completes in < 30 seconds

### User Experience
- ✅ User understands what entropy is (education)
- ✅ User feels in control (consent, preview, retry)
- ✅ Process feels natural, not intimidating
- ✅ Multi-modal options (accessible to all)

### Security
- ✅ Unique node identities (no collisions)
- ✅ Human-verifiable trust elevation
- ✅ Audit trail of trust decisions
- ✅ No remote access to capture devices

---

## 🚀 Implementation Phases

### Phase 1: Foundation (Week 1)
**Team: PetalTongue**
- Fix topology edge parsing
- Add trust level visualization
- Add genetic lineage display

**Deliverable**: MVP-ready visualization

### Phase 2: Audio Entropy (Week 2-3)
**Team: BearDog + biomeOS + PetalTongue**
- BearDog: Entropy API design + implementation
- biomeOS: Entropy proxy endpoints
- PetalTongue: Audio capture UI

**Deliverable**: First human entropy capture working

### Phase 3: Trust UI (Week 3-4)
**Team: PetalTongue + biomeOS**
- Trust decision dialog
- Trust elevation workflow
- Visual feedback system

**Deliverable**: Progressive trust with human approval

### Phase 4: Expansion (Week 5-6)
**Team: All**
- Add text entropy capture
- Add video entropy (optional)
- Add drawing entropy (optional)
- Comprehensive testing

**Deliverable**: Multi-modal human entropy system

---

## 📚 Documentation Needed

### For Users
1. "What is Human Entropy?" - Educational guide
2. "How to Capture Entropy" - Step-by-step
3. "Trust Elevation Guide" - When and how
4. "Privacy FAQ" - What's captured, what's not

### For Developers
1. "Entropy API Specification" - BearDog endpoints
2. "Streaming Architecture" - Data flow
3. "Security Considerations" - Privacy, consent
4. "Integration Guide" - PetalTongue ↔ biomeOS ↔ BearDog

### For Operators
1. "Deployment Checklist" - Entropy system setup
2. "Troubleshooting Guide" - Common issues
3. "Security Audit" - Verification steps
4. "Backup & Recovery" - Key management

---

## 💡 Future Vision (Post-MVP)

### Multi-Modal Entropy Fusion
- Combine audio + video + drawing for strongest entropy
- Adaptive entropy (choose based on environment)
- Continuous entropy (periodic re-mixing)

### Biometric Integration (Privacy-Aware)
- Facial recognition (local only, never sent)
- Voice fingerprinting (local only)
- Behavioral biometrics (typing, mouse patterns)
- All processed locally, only hashes sent

### Social Entropy
- Group activities for multi-tower trust
- Shared stories/songs for family authentication
- Collaborative art for group lineage

### Accessibility
- Audio-only mode (for blind users)
- Visual-only mode (for deaf users)
- Text-only mode (for minimal devices)
- Haptic feedback integration

---

## 🎊 Bottom Line

**Vision**: PetalTongue becomes the **human interface** for the ecoPrimals trust system, capturing rich human entropy while BearDog handles the cryptographic heavy lifting.

**Philosophy**: 
- **Humans are creative** → Capture that creativity as entropy
- **Humans are sovereign** → Give them control and choice
- **Humans are diverse** → Offer multiple modalities
- **Humans are private** → Never persist, always protect

**Architecture**:
```
🌸 PetalTongue (Human Interface)
   • Captures: audio, video, text, drawings
   • Streams: never persists
   • Visualizes: trust, lineage, topology
   
   ↓ [Secure Stream]
   
🏗️ biomeOS (Orchestration Layer)
   • Proxies: routes entropy securely
   • Never logs: stream-through only
   • Coordinates: primal interactions
   
   ↓ [Encrypted Channel]
   
🐻 BearDog (Security Layer)
   • Mixes: USB seed + local + human entropy
   • Derives: unique node identities
   • Validates: trust elevations
   • Secures: never persists raw entropy
```

**Result**: Human-centered, sovereign, multi-modal trust system that scales from single towers to global federations while maintaining privacy and security.

---

**Status**: Strategic Vision Document  
**Next Steps**: 
1. Review with PetalTongue, BearDog, biomeOS teams
2. Prioritize MVP features (Week 1-2)
3. Design entropy API (BearDog team)
4. Implement foundation (PetalTongue team)

🌸🔒🎨 **Human creativity meets cryptographic security!** 🎨🔒🌸

