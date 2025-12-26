# 02 - Multi-Primal Orchestration

**Demonstrate BiomeOS orchestrating multiple primals together**

**Duration**: 20-30 minutes  
**Primals Required**: 2-5 from `../phase1bins/`  
**Complexity**: Medium-High

---

## 🎯 Purpose

This scenario demonstrates BiomeOS's **CORE VALUE**: orchestrating multiple primals together in coordinated workflows!

1. **Storage + Discovery** - NestGate + Songbird coordination
2. **Compute + Discovery** - ToadStool + Songbird coordination  
3. **Full Ecosystem** - All 5 primals orchestrated together
4. **Real Composition** - BiomeOS managing live primal interactions

**This is where BiomeOS shines: making primals work together!**

---

## 🚀 Quick Start

### Run Full Stack Demo (Recommended)
```bash
./full-stack.sh
```

This starts ALL 5 primals and shows BiomeOS orchestrating them!

### Run Individual Combinations
```bash
./storage-plus-discovery.sh   # NestGate + Songbird
./compute-plus-discovery.sh   # ToadStool + Songbird
```

---

## 📋 Demonstrations

### 1. Storage + Discovery (NestGate + Songbird) ✅

**Script**: `./storage-plus-discovery.sh`

**What it demonstrates**:
- Start Songbird (service discovery)
- Start NestGate (storage)
- BiomeOS discovers both
- BiomeOS registers NestGate with Songbird
- BiomeOS coordinates storage operations through discovered service
- **Cross-primal coordination!**

**Workflow**:
```
1. Songbird starts (port 8081)
2. NestGate starts (port 8082)
3. BiomeOS discovers both by capability
4. BiomeOS: Register NestGate → Songbird
5. BiomeOS: Store data → NestGate (discovered via Songbird)
6. BiomeOS: Query storage services → Songbird
7. Show coordinated workflow complete
```

---

### 2. Compute + Discovery (ToadStool + Songbird) ✅

**Script**: `./compute-plus-discovery.sh`

**What it demonstrates**:
- Start Songbird (service discovery)
- Start ToadStool (compute)
- BiomeOS discovers both
- BiomeOS registers ToadStool with Songbird
- BiomeOS deploys workload to discovered compute
- **Service mesh coordination!**

**Workflow**:
```
1. Songbird starts (port 8081)
2. ToadStool starts (port 8080)
3. BiomeOS discovers both by capability
4. BiomeOS: Register ToadStool → Songbird
5. BiomeOS: Deploy workload → ToadStool (discovered via Songbird)
6. BiomeOS: Query compute status → Songbird
7. Show mesh coordination complete
```

---

### 3. Full Ecosystem (All 5 Primals) ✅✅✅

**Script**: `./full-stack.sh`

**What it demonstrates**:
- **THE COMPLETE ECOSYSTEM!**
- All 5 primals working together
- BiomeOS orchestrating everything
- Real multi-primal workflows
- **This is BiomeOS's purpose!**

**Primals Started**:
1. **Songbird** (8081) - Service discovery & mesh
2. **ToadStool** (8080) - Compute orchestration
3. **NestGate** (8082) - Storage management
4. **BearDog** (9000) - Security & crypto
5. **Squirrel** (9010) - AI capabilities

**Workflow**:
```
Phase 1: Startup & Discovery
1. Start all 5 primals
2. BiomeOS discovers all by capability
3. BiomeOS builds capability map

Phase 2: Service Mesh
4. BiomeOS → Songbird: Register all services
5. BiomeOS builds service topology
6. Show complete mesh

Phase 3: Coordinated Workflow
7. BiomeOS → BearDog: Encrypt data
8. BiomeOS → NestGate: Store encrypted data (via Songbird)
9. BiomeOS → ToadStool: Deploy processing workload
10. BiomeOS → Squirrel: Analyze system state
11. BiomeOS → Songbird: Query all service health
12. Show complete orchestration!

Phase 4: Demonstrate Composition
13. BiomeOS coordinates multi-primal workflow:
    - Security (BearDog) + Storage (NestGate)
    - Discovery (Songbird) + Compute (ToadStool)
    - AI (Squirrel) + All others
14. Show true ecosystem composition
```

---

## 🎓 What You'll Learn

### BiomeOS's Core Value
- **Composition**: Makes primals work together
- **Coordination**: Orchestrates workflows across primals
- **Discovery**: Finds services dynamically
- **Delegation**: Each primal does what it's best at

### Multi-Primal Patterns
- **Service Registration**: Register with discovery
- **Service Query**: Find services by capability
- **Cross-Primal Calls**: Primal A → Primal B (via BiomeOS)
- **Workflow Chains**: Multi-step operations

### Real Production Patterns
- **Service Mesh**: Songbird coordinates routing
- **Secure Storage**: BearDog + NestGate
- **Compute Orchestration**: Songbird + ToadStool
- **AI Integration**: Squirrel + All others

---

## 📊 Expected Output (Full Stack)

```
🌱 BiomeOS Full Ecosystem Orchestration
========================================

Phase 1: Starting Primals
   ✅ Songbird started (8081) - Discovery
   ✅ ToadStool started (8080) - Compute
   ✅ NestGate started (8082) - Storage
   ✅ BearDog started (9000) - Security
   ✅ Squirrel started (9010) - AI

Phase 2: BiomeOS Discovery
   ✅ Discovering primals by capability...
   ✅ Found 5 primals:
      - Songbird (discovery, service-mesh)
      - ToadStool (compute, execution)
      - NestGate (storage, persistence)
      - BearDog (security, crypto)
      - Squirrel (ai, inference)
   ✅ Capability map complete

Phase 3: Service Mesh Setup
   ✅ BiomeOS → Songbird: Register ToadStool
   ✅ BiomeOS → Songbird: Register NestGate
   ✅ BiomeOS → Songbird: Register BearDog
   ✅ BiomeOS → Songbird: Register Squirrel
   ✅ Service mesh: 4 services registered
   ✅ Topology: Complete

Phase 4: Coordinated Workflow
   ✅ BiomeOS workflow: Secure Data Storage
      1. BearDog: Encrypt("test data")
      2. NestGate: Store(encrypted) [via Songbird]
      3. NestGate: Verify storage
   ✅ Encrypted data stored securely!

   ✅ BiomeOS workflow: Compute Orchestration
      4. ToadStool: Deploy workload [via Songbird]
      5. ToadStool: Monitor resources
   ✅ Workload deployed and monitored!

   ✅ BiomeOS workflow: AI Analysis
      6. Squirrel: Analyze system state
      7. Squirrel: Provide optimization suggestions
   ✅ System analyzed and optimized!

Phase 5: Health & Status
   ✅ BiomeOS → Songbird: Query all service health
   ✅ All services: Healthy
   ✅ Mesh status: Operational
   ✅ Ecosystem: Coordinated

🎉 Full Ecosystem Orchestration Complete!

What you just saw:
  • BiomeOS orchestrated 5 primals simultaneously
  • Each primal did what it's best at (delegation)
  • Multi-step workflows across primals
  • Service mesh coordination (via Songbird)
  • Secure data handling (BearDog + NestGate)
  • Compute orchestration (via ToadStool)
  • AI integration (via Squirrel)
  • REAL ecosystem composition!

This is BiomeOS's purpose: making primals work together! 🌱
```

---

## ⚙️ Prerequisites

### 1. All Binaries Available
```bash
ls -lh ../../phase1bins/{songbird,toadstool,nestgate,beardog,squirrel}-bin
```

### 2. BiomeOS Built
```bash
cd ../..
cargo build --release
```

### 3. Ports Available
- 8080 (ToadStool)
- 8081 (Songbird)
- 8082 (NestGate)
- 9000 (BearDog)
- 9010 (Squirrel)

---

## 🔧 Troubleshooting

### Port conflicts
```bash
# Check what's running
netstat -tulpn | grep -E "8080|8081|8082|9000|9010"

# Kill all primals
pkill -f "songbird|toadstool|nestgate|beardog|squirrel"
```

### Primal won't start
```bash
# Check logs
tail -f /tmp/songbird.log
tail -f /tmp/toadstool.log
# etc...
```

### BiomeOS can't discover
```bash
# Check primals are running
for port in 8080 8081 8082 9000 9010; do
    curl -s http://localhost:$port/health || echo "Port $port not responding"
done
```

---

## 🎯 Success Criteria

✅ **All primals start successfully**  
✅ **BiomeOS discovers all primals**  
✅ **Service mesh is established**  
✅ **Cross-primal workflows complete**  
✅ **No direct primal-to-primal calls** (all via BiomeOS)  
✅ **Clean orchestration demonstrated**

---

## 💡 Key Concepts Demonstrated

### 1. Pure Orchestration
- BiomeOS doesn't implement features
- BiomeOS delegates to specialists
- Each primal does what it's best at

### 2. Service Mesh
- Songbird provides discovery
- BiomeOS coordinates routing
- Dynamic topology

### 3. Capability-Based
- Discover by "what" not "who"
- No hardcoded dependencies
- Flexible composition

### 4. Real Production
- Actual primal binaries
- Real API calls
- Production patterns

---

## 📚 What Makes This Special

### Before BiomeOS
- Manual primal coordination
- Hardcoded endpoints
- Complex integration code
- Tight coupling

### With BiomeOS
- ✅ Automatic discovery
- ✅ Dynamic routing
- ✅ Simple orchestration
- ✅ Loose coupling
- ✅ **Primals just work together!**

---

## 🎯 Next Steps

After completing this scenario:

1. **Understand** multi-primal orchestration
2. **See** real ecosystem composition
3. **Appreciate** BiomeOS's core value
4. **Move to** `../03-chimera-composition/` for advanced fusion

---

**Status**: Ready for full ecosystem demonstration  
**Prerequisites**: All phase1bins binaries, BiomeOS built  
**Learning Value**: **CRITICAL** (This is BiomeOS's purpose!)

---

*"BiomeOS makes primals work together. This is our reason to exist."* 🌱✨

