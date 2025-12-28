# 🎯 Primal Architecture Reality - Dec 28, 2025

## Critical Insight

**Primals evolve independently. BiomeOS must discover and adapt, NOT standardize.**

---

## The Reality of Primal Evolution

### Each Primal Team Evolves Their API

**NestGate Team** (Storage):
- Standalone server with `service start`
- Requires JWT authentication for security
- REST API with `/health`, `/api/v1/zfs/*`
- **Current**: Running on port 9020 ✅

**Songbird Team** (Orchestration):
- Started as the reference API
- Federation and discovery focus
- Self-hosting orchestrator
- **Current**: Different architecture (needs investigation)

**BearDog Team** (Cryptography):
- CLI-focused tool, not standalone server
- Designed for in-house integration
- Fully decentralized crypto operations
- **Current**: Library/CLI, not a server

**Toadstool Team** (Compute):
- Universal runtime environment
- Manages biome instances
- CLI tool for orchestration
- **Current**: Launcher, not standalone server

**Squirrel Team** (Configuration):
- Interactive CLI tool
- Configuration management
- Command registry system
- **Current**: CLI/REPL, not a server

---

## BiomeOS Architectural Principle

### ❌ WRONG Approach
```rust
// Expecting standardization
let nestgate = PrimalClient::new("http://localhost:9020");
let songbird = PrimalClient::new("http://localhost:9000");
let beardog = PrimalClient::new("http://localhost:9040");
// This assumes all primals have same API pattern!
```

### ✅ CORRECT Approach
```rust
// Agnostic discovery and adaptation
let storage_capability = discover_capability("storage").await?;
match storage_capability {
    Capability::RestApi { endpoint, methods } => {
        // Adapt to REST API
    },
    Capability::CliTool { binary, commands } => {
        // Adapt to CLI tool
    },
    Capability::Library { interface } => {
        // Adapt to library integration
    },
    _ => {
        // Discover at runtime what it actually is
    }
}
```

---

## Current Deployment Status

### What Actually Runs as Standalone Servers?

#### ✅ NestGate (Port 9020)
```bash
NESTGATE_JWT_SECRET=$(openssl rand -base64 48) \
NESTGATE_API_PORT=9020 \
./primals/nestgate service start --port 9020
```
**Health**: http://localhost:9020/health
**Status**: Running with JWT security ✅

#### 🔍 Songbird (Investigation Needed)
- Orchestrator architecture
- May self-host or CLI-based
- **Action**: NestGate team / Songbird team coordination needed

#### 📦 BearDog (Library/CLI Integration)
- Not a standalone server
- In-house cryptography
- Fully decentralized
- **Integration**: Via biomeOS as substrate

#### 🧪 Toadstool (Runtime Launcher)
- Launches and manages biomes
- Not standalone
- **Integration**: BiomeOS deploys via Toadstool

#### 🛠️ Squirrel (CLI Tool)
- Configuration management
- Interactive REPL
- **Integration**: Invoked by biomeOS when needed

---

## Evolution Strategy

### 1. BiomeOS as Agnostic Substrate

**Core Capability**: Runtime Discovery
```yaml
# BiomeOS discovers what's available
discovered_services:
  - type: rest_api
    name: nestgate
    endpoint: http://localhost:9020
    health: /health
    capabilities: ["storage", "zfs", "snapshots"]
  
  - type: cli_tool
    name: beardog
    binary: ./primals/beardog
    capabilities: ["encryption", "birdsong", "lineage"]
  
  - type: runtime
    name: toadstool
    binary: ./primals/toadstool
    capabilities: ["biome_management", "isolation"]
```

### 2. No Forced Standardization

Each primal team evolves independently:
- **NestGate**: May add GraphQL, gRPC, new auth methods
- **Songbird**: May change federation protocol
- **BearDog**: May add HSM integration, new crypto primitives
- **Toadstool**: May add new runtime targets
- **Squirrel**: May evolve CLI interface

**BiomeOS adapts** via:
1. Capability discovery
2. API introspection
3. Runtime adaptation
4. Zero hardcoded assumptions

### 3. Showcase Demonstrates Adaptation

**Week 1**: Foundation
- 00-substrate: BiomeOS discovers what's available
- 01-nestgate: One primal working (REST API)
- Demonstrate: Agnostic discovery patterns

**Week 2**: Multi-Architecture
- Integrate BearDog (CLI/library)
- Integrate Toadstool (runtime)
- Demonstrate: Different integration patterns

**Week 3**: Real Coordination
- BirdSong P2P via BearDog + NestGate
- Toadstool manages deployment
- Demonstrate: Composed capabilities

---

## Questions for Primal Teams

### NestGate Team
1. JWT for standalone - ✅ Implemented
2. Evolution roadmap?
   - New API endpoints?
   - New auth methods?
   - GraphQL/gRPC plans?

### Songbird Team
3. Server architecture?
   - Standalone daemon?
   - CLI orchestrator?
   - Both?

### BearDog Team
4. Integration pattern?
   - Library linkage?
   - CLI invocation?
   - Service daemon planned?

### Toadstool Team
5. BiomeOS deployment?
   - How does Toadstool launch biomes?
   - API for biomeOS to invoke?

### Squirrel Team
6. Configuration pattern?
   - File-based only?
   - API available?
   - Runtime queries?

---

## BiomeOS Development Focus

### Immediate (Today)
1. ✅ NestGate running with proper security
2. Build showcase demos with ONE working primal
3. Demonstrate agnostic discovery

### This Week
1. Investigate Songbird architecture
2. Design BearDog integration pattern
3. Design Toadstool integration pattern
4. Create adapter layer for each

### Ongoing
1. **NEVER** hardcode primal APIs
2. **ALWAYS** discover at runtime
3. **ADAPT** to whatever primals expose
4. **DOCUMENT** integration patterns as they emerge

---

## Key Insight from User

> "Primals will have different APIs based on evolution.  
>  BiomeOS should be able to consume agnostically rather than expect standardization.  
>  We started with Songbird API and began to evolve our infra."

This means:
- ✅ Started with Songbird patterns (good foundation)
- ✅ Each primal evolves differently (expected)
- ✅ BiomeOS must adapt (not enforce)
- ✅ Discovery is the only constant

---

## Success Criteria

### NOT Success
- All primals running on standard ports ❌
- All primals with same API structure ❌
- All primals as REST services ❌

### ACTUAL Success
- BiomeOS discovers available primals ✅
- BiomeOS adapts to each primal's API ✅
- BiomeOS composes capabilities ✅
- Showcase works with real primals (however they work) ✅
- **Zero code changes** when primals evolve ✅

---

## Next Actions

1. **Document current reality**: NestGate running, others have different architectures
2. **Build first demos**: With NestGate only (one primal)
3. **Investigate architectures**: Contact teams / check sibling repos
4. **Design adapters**: For each integration pattern
5. **Demonstrate agnosticism**: Showcase adapts to reality

**The goal is NOT to make primals uniform.**  
**The goal is to make BiomeOS infinitely adaptive.**

---

*Reality documented: Dec 28, 2025*  
*BiomeOS will discover, adapt, and compose - NOT standardize*

