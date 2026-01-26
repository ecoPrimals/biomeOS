# 🏗️ Infrastructure Evolution Roadmap

**Last Updated**: January 26, 2026  
**Status**: Tower Atomic operational, evolution paths defined

---

## Current State

### ✅ Operational

| Component | Status | Description |
|-----------|--------|-------------|
| **Neural API** | ✅ | Graph-based orchestration, capability.call routing |
| **Tower Atomic** | ✅ | BearDog + Songbird, Pure Rust TLS 1.3 |
| **capability.call** | ✅ | Semantic operation routing, 39 mappings |
| **Graph Deployment** | ✅ | `tower_atomic_bootstrap.toml` |

### 🔧 Started (In Crates)

| Component | Location | Description |
|-----------|----------|-------------|
| **Terraria** | `primal.terraria` capability | Primal process incubation/lifecycle |
| **Incubation** | `biomeos-spore/incubation.rs` | Spore deployment with entropy mixing |
| **Health Monitoring** | `primal_orchestrator.rs` | Lifecycle state management |
| **Zombie Reaping** | `primal_impls.rs` | Process cleanup via `try_wait()` |
| **Process Launcher** | `primal_launcher.rs` | Modern Rust process management |

---

## Evolution Priorities

### Phase 1: Hardening (Current)

```
Priority 1: Health Monitoring via capability.call
┌─────────────────────────────────────────────────────────┐
│ capability.call("primal.health", "check", {            │
│   primal: "beardog-nat0"                               │
│ })                                                      │
│                                                         │
│ → Neural API routes to health endpoint                  │
│ → Returns: { status: "healthy", uptime: "2h 15m" }     │
└─────────────────────────────────────────────────────────┘
```

**Tasks**:
- [ ] Implement `primal.health` capability in Neural API
- [ ] Add health endpoint to each primal
- [ ] Periodic health checks via background task

### Phase 2: Terraria Incubation

```
Priority 2: Primal Lifecycle via Terraria
┌─────────────────────────────────────────────────────────┐
│ capability.call("primal.terraria", "incubate", {       │
│   primal: "squirrel",                                  │
│   config: { model: "gpt-4", temperature: 0.7 }         │
│ })                                                      │
│                                                         │
│ → Neural API creates new primal instance               │
│ → Registers capabilities with Neural API                │
│ → Returns: { primal_id: "squirrel-nat0-tower1" }       │
└─────────────────────────────────────────────────────────┘
```

**Tasks**:
- [ ] Wire `biomeos-spore/incubation.rs` to Neural API
- [ ] Implement terraria.incubate, terraria.status, terraria.stop
- [ ] Add graph-based configuration for new primals

### Phase 3: Apoptosis (Zombie Prevention)

```
Priority 3: Graceful Primal Termination
┌─────────────────────────────────────────────────────────┐
│ capability.call("primal.apoptosis", "initiate", {      │
│   primal: "squirrel-nat0-tower1",                      │
│   reason: "shutdown",                                   │
│   timeout_ms: 5000                                      │
│ })                                                      │
│                                                         │
│ → Neural API sends SIGTERM                              │
│ → Waits for graceful shutdown                           │
│ → Reaps zombie if needed                                │
│ → Unregisters capabilities                              │
└─────────────────────────────────────────────────────────┘
```

**Tasks**:
- [ ] Implement `primal.apoptosis` capability
- [ ] Integrate with `primal_impls.rs` zombie reaping
- [ ] Add timeout-based SIGKILL escalation
- [ ] Capability deregistration on termination

### Phase 4: Ecosystem Coordination

```
Priority 4: Multi-Tower Coordination
┌─────────────────────────────────────────────────────────┐
│ capability.call("ecosystem.coordinate", "join", {      │
│   tower_id: "tower-beta",                              │
│   socket: "/tmp/neural-api-beta.sock"                  │
│ })                                                      │
│                                                         │
│ → Neural APIs federate                                  │
│ → Capability discovery spans towers                     │
│ → Load balancing across instances                       │
└─────────────────────────────────────────────────────────┘
```

---

## Capability Registry (Future)

| Capability | Operations | Status |
|-----------|------------|--------|
| `crypto` | sha256, encrypt, decrypt, sign | ✅ BearDog |
| `secure_http` | http.request, http.get, http.post | ✅ Songbird |
| `primal.germination` | spawn, configure | ✅ biomeOS |
| `primal.terraria` | incubate, status, list | 🔧 Started |
| `primal.apoptosis` | initiate, force, status | 📋 Planned |
| `primal.health` | check, monitor, report | 📋 Planned |
| `ecosystem.nucleation` | bootstrap, verify | ✅ biomeOS |
| `ecosystem.coordination` | join, leave, status | 📋 Planned |
| `graph.execution` | deploy, status, rollback | ✅ biomeOS |
| `ai.inference` | chat, embed, complete | 📋 Squirrel |

---

## Graph Evolution

### Current: `tower_atomic_bootstrap.toml`

```toml
[graph]
id = "tower_atomic_bootstrap"
coordination = "Sequential"

[[nodes]]
id = "germinate_beardog"
[nodes.primal]
by_capability = "security"

[[nodes]]
id = "germinate_songbird"
[nodes.primal]
by_capability = "discovery"
```

### Future: `ecosystem_full.toml`

```toml
[graph]
id = "ecosystem_full"
coordination = "Phased"

# Phase 1: Security Foundation
[[nodes]]
id = "tower_atomic"
graph = "tower_atomic_bootstrap"

# Phase 2: AI Layer
[[nodes]]
id = "squirrel_ai"
depends_on = ["tower_atomic"]
[nodes.primal]
by_capability = "ai"

# Phase 3: Storage
[[nodes]]
id = "nestgate_storage"
depends_on = ["tower_atomic"]
[nodes.primal]
by_capability = "storage"

# Phase 4: UI
[[nodes]]
id = "petaltongue_ui"
depends_on = ["squirrel_ai", "nestgate_storage"]
[nodes.primal]
by_capability = "ui"
```

---

## HTTP Requests via capability.call

All HTTP requests should go through Neural API:

```rust
// In Squirrel (AI primal)
async fn call_openai(messages: Vec<Message>) -> Result<Response> {
    capability_call("secure_http", "http.request", json!({
        "url": "https://api.openai.com/v1/chat/completions",
        "method": "POST",
        "headers": {
            "Authorization": format!("Bearer {}", api_key),
            "Content-Type": "application/json"
        },
        "body": {
            "model": "gpt-4",
            "messages": messages
        }
    })).await
}
```

---

## Code Locations

### Existing Infrastructure

| Feature | File | Status |
|---------|------|--------|
| Process Launch | `crates/biomeos-atomic-deploy/src/primal_launcher.rs` | ✅ |
| Zombie Reaping | `crates/biomeos-core/src/primal_impls.rs:227-258` | ✅ |
| Health Check | `crates/biomeos-core/src/primal_orchestrator.rs` | 🔧 |
| Incubation | `crates/biomeos-spore/src/incubation.rs` | 🔧 |
| capability.call | `crates/biomeos-atomic-deploy/src/neural_api_server.rs` | ✅ |

### Future Implementation

| Feature | Proposed Location |
|---------|------------------|
| Terraria Handler | `neural_api_server.rs` → `handle_terraria()` |
| Apoptosis Handler | `neural_api_server.rs` → `handle_apoptosis()` |
| Health Handler | `neural_api_server.rs` → `handle_health()` |
| Federation | `neural_router.rs` → `federate()` |

---

## Next Steps

1. **Immediate**: Document deployment via `./deploy_tower_atomic.sh`
2. **This Week**: Implement `primal.health` capability
3. **Next Week**: Wire terraria incubation to Neural API
4. **Future**: Multi-tower federation

---

## Deployment

```bash
# Deploy Tower Atomic
./deploy_tower_atomic.sh

# Check status
./deploy_tower_atomic.sh status

# Stop
./deploy_tower_atomic.sh stop

# Test HTTP via capability.call
echo '{"jsonrpc":"2.0","method":"capability.call","params":{"capability":"secure_http","operation":"http.request","args":{"url":"https://api.github.com/zen","method":"GET"}},"id":1}' | nc -U /tmp/neural-api.sock
```

---

**Architecture Principle**: Everything via `capability.call` through Neural API. No direct primal-to-primal communication.

