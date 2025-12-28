# BiomeOS Showcase

**Live demonstrations of biomeOS as the substrate for sovereign primal orchestration**

---

## 🎯 What Is This?

BiomeOS showcases demonstrate:
1. **BiomeOS as Substrate** - Deploys and manages primals as services
2. **Runtime Discovery** - Zero hardcoding, capability-based orchestration  
3. **Live Infrastructure** - Real primals, real deployments (NO MOCKS)
4. **benchScale Validated** - Every demo is a deployable, testable topology

---

## 🚀 Quick Start (5 Minutes)

### Prerequisites
```bash
# 1. Ensure primals are available
ls ../../primals/
# Should see: beardog, nestgate, songbird, squirrel, toadstool, petaltongue

# 2. Ensure Rust installed
cargo --version

# 3. (Optional) benchScale for validation
ls ../../../../primalTools/benchscale/
```

### Run Your First Demo
```bash
# Deploy NestGate via biomeOS
cd 01-nestgate/01-hello-nestgate
./demo.sh

# See runtime discovery in action
# No hardcoded endpoints!
```

---

## 📚 Documentation

### Essential Guides
- **[RUNTIME_DISCOVERY.md](./RUNTIME_DISCOVERY.md)** - Zero hardcoding patterns
- **[NO_MOCKS_POLICY.md](./NO_MOCKS_POLICY.md)** - Live-only enforcement
- **[CLEANUP_AND_DISCOVERY_PLAN.md](./CLEANUP_AND_DISCOVERY_PLAN.md)** - Organization strategy
- **[SHOWCASE_BUILDOUT_PLAN_DEC_28_2025.md](./SHOWCASE_BUILDOUT_PLAN_DEC_28_2025.md)** - Full buildout plan
- **[QUICK_ACTION_PLAN_DEC_28_2025.md](./QUICK_ACTION_PLAN_DEC_28_2025.md)** - Immediate actions

### Historical Context
- **[archive/](./archive/)** - Historical docs and session reports

---

## 🏗️ Showcase Structure

### Current Demos
```
00-local-capabilities/          # Local biomeOS capabilities
01-single-primal/              # Individual primal demos
02-primal-pairs/               # Primal coordination
03-p2p-coordination/           # BirdSong P2P and BTSP
```

### Coming Soon (Week 1-3)
```
00-substrate/                  # biomeOS as deployment substrate
01-nestgate/                   # NestGate showcase (our local star)
02-birdsong-p2p/              # BirdSong P2P deployment
03-multi-primal/              # Multi-primal workflows
04-production/                # Real-world deployment patterns
```

---

## 🎓 Core Principles

### 1. Runtime Discovery Only
```bash
# ❌ Hardcoded
NESTGATE_URL="http://localhost:9020"

# ✅ Runtime discovery
STORAGE=$(discover_capability "storage")
# Works with NestGate, MinIO, S3, or custom storage
```

### 2. Primals Have Only Self-Knowledge
```
Primal knows:
  ✓ "I provide 'storage' capability"
  ✓ "I serve on port 9020"
  ✓ "My health endpoint is /health"

Primal does NOT know:
  ✗ Other primals exist
  ✗ Other primals' endpoints
  ✗ How to coordinate

BiomeOS handles all discovery and coordination.
```

### 3. Live Infrastructure Only (NO MOCKS)
```
✅ Real primal binaries (primals/)
✅ Real HTTP endpoints
✅ Real service discovery
✅ Real performance metrics

❌ Mock servers
❌ Simulated responses
❌ Fake metrics
❌ "It would work if..." scenarios
```

### 4. benchScale Validated
```yaml
# Every demo has topology.yaml
name: demo-name
nodes: [...]
tests: [...]

# Deployable and testable
benchscale deploy --topology topology.yaml
benchscale validate
benchscale destroy
```

---

## 🔍 Quick Reference

### Discovery Pattern
```bash
#!/usr/bin/env bash
source common/discovery.sh

# Discover capabilities at runtime
STORAGE=$(discover_capability "storage")
COMPUTE=$(discover_capability "compute")
SECURITY=$(discover_capability "security")

# Use discovered providers
store_data "$STORAGE" "file.txt"
process_data "$COMPUTE" "workload"
encrypt_data "$SECURITY" "sensitive"

# No primal names, no hardcoded endpoints!
```

### Validation Pattern
```bash
# Run demo
./demo.sh

# Validate with benchScale
benchscale deploy --topology topology.yaml
benchscale validate --test validate.sh
benchscale destroy

# Demo proves: Works in dev AND deployment!
```

---

## 📋 Status

### Completed ✅
- [x] NO MOCKS policy enforced
- [x] Documentation cleanup (50 files archived)
- [x] Runtime discovery guide written
- [x] benchScale integration planned
- [x] Comprehensive code audit (A- grade)

### In Progress 🔄
- [ ] Create `common/discovery.sh` utilities
- [ ] Build `00-substrate/` demos
- [ ] Build `01-nestgate/` demos  
- [ ] Build `02-birdsong-p2p/` demos
- [ ] benchScale validation for all demos

### Next (Week 1)
- Deploy real primals
- Create foundation demos (00-substrate)
- Create NestGate showcase (01-nestgate)
- Integrate benchScale validation

---

## 🎯 Philosophy

### Zero Hardcoding
> "If a primal name appears in biomeOS code, we failed.  
>  If an endpoint is hardcoded, we failed.  
>  Discover at runtime, adapt to evolution."

### Live Infrastructure
> "Mocks hide gaps. Live primals expose reality.  
>  If it works with mocks but fails with real primals, it doesn't work.  
>  Showcase = validation, not theater."

### Primal Self-Knowledge
> "Primals know themselves, not others.  
>  'I provide storage' - not 'I connect to Songbird'.  
>  BiomeOS orchestrates, primals remain sovereign."

### benchScale Validation
> "Every showcase must be deployable.  
>  Demos that work in dev but fail in production prove nothing.  
>  benchScale makes showcases real."

---

## 🚀 Getting Started

### For Developers
1. **Read** [RUNTIME_DISCOVERY.md](./RUNTIME_DISCOVERY.md)
2. **Review** existing demos in `01-single-primal/`
3. **Study** discovery patterns in `common/`
4. **Build** following the patterns

### For Users
1. **Deploy** real primals: `../../deploy-real-primals.sh`
2. **Run** demos: `cd 01-single-primal/ && ./demo.sh`
3. **Explore** progressively: single → pairs → ecosystem
4. **Validate** with benchScale (optional)

### For Contributors
1. **Follow** NO MOCKS policy
2. **Use** runtime discovery only
3. **Create** benchScale topology.yaml
4. **Write** validation tests
5. **Document** gaps discovered

---

## 📊 Success Metrics

### Week 1 (Foundation)
- [ ] 5 working demos with runtime discovery
- [ ] All demos use real primals (no mocks)
- [ ] benchScale topologies created
- [ ] Validation scripts written

### Week 2 (Integration)
- [ ] BirdSong P2P demos
- [ ] Multi-primal coordination
- [ ] All demos benchScale validated
- [ ] Gap reports generated

### Week 3 (Production)
- [ ] Production deployment patterns
- [ ] Multi-machine federation
- [ ] Full ecosystem orchestration
- [ ] Complete documentation

---

## 🆘 Help & Support

### Common Issues
- **"Primal not found"**: Deploy first: `../../deploy-real-primals.sh`
- **"Discovery failed"**: Check `common/discovery.sh` is sourced
- **"Demo fails"**: Good! Document the gap, fix root cause

### Resources
- **Code Audit**: `../COMPREHENSIVE_CODE_AUDIT_DEC_28_2025.md`
- **Primal Binaries**: `../../primals/README.md`
- **benchScale**: `../../../../primalTools/benchscale/`
- **Archive**: `./archive/` (historical context)

---

## 🌱 Vision

**BiomeOS = Operating System for Primals**

Not just an orchestrator, but THE substrate that:
- Deploys primals as services
- Discovers capabilities at runtime
- Coordinates without hardcoding
- Validates via benchScale
- Enables zero-config evolution

**From boot loader to P2P tunnels, pure Rust throughout.**

---

**Status**: Active Development  
**Grade**: A- (92/100) - See code audit  
**Next**: Build Week 1 foundation demos

🚀 **Let's showcase digital sovereignty!** 🌱
