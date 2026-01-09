# 🌱 START HERE - biomeOS Quick Start

**Welcome to biomeOS!** This guide will get you up and running quickly.

---

## 🎊 Current Status

**PRODUCTION-READY!** (January 9, 2026)

✅ Core infrastructure  
✅ NUCLEUS secure discovery (14 tests passing)  
✅ Topology API  
✅ Zero unsafe code  
✅ Deep Debt Evolution Complete (Phases 1 & 2)  
✅ Zero production mocks  
✅ Zero hardcoded endpoints  
✅ LAN Federation working  

---

## 📚 Essential Documents

### **New to biomeOS?**
1. **[README.md](README.md)** - Overview, architecture, quick start
2. **[STATUS.md](STATUS.md)** - Current status, statistics, next steps
3. **[ROADMAP.md](ROADMAP.md)** - Phased implementation plan

### **Deep Dive**
- **[docs/DEEP_DEBT_FINAL_STATUS_JAN9.md](docs/DEEP_DEBT_FINAL_STATUS_JAN9.md)** - Production-ready status (600+ lines)
- **[docs/DEEP_DEBT_EXECUTION_PLAN_JAN9.md](docs/DEEP_DEBT_EXECUTION_PLAN_JAN9.md)** - Complete analysis (430 lines)
- **[specs/](specs/)** - 30+ technical specifications
- **[SESSION_SUMMARY_JAN9_FINAL.md](SESSION_SUMMARY_JAN9_FINAL.md)** - Session summary

### **Integration**
- **[docs/PETALTONGUE_TEAM_HANDOFF_JAN9.md](docs/PETALTONGUE_TEAM_HANDOFF_JAN9.md)** - UI integration plan
- **[NEURAL_API_STATUS.md](NEURAL_API_STATUS.md)** - Neural API progress
- **[NEURAL_API_ROADMAP.md](NEURAL_API_ROADMAP.md)** - Neural API phases

---

## 🚀 Quick Start

### **1. Build Everything**
```bash
cargo build --workspace
```

### **2. Run Tests**
```bash
# All tests
cargo test --workspace

# NUCLEUS tests
cargo test --package biomeos-federation nucleus_tests
```

### **3. Start API Server**
```bash
# Standalone mode (demo, no primals required)
BIOMEOS_STANDALONE_MODE=true cargo run --package biomeos-api

# Live mode (discovers real primals)
cargo run --package biomeos-api
```

### **4. Test Endpoints**
```bash
# Health check
curl http://localhost:3000/api/v1/health | jq '.'

# Topology (for petalTongue)
curl http://localhost:3000/api/v1/topology | jq '.'
```

---

## 🏗️ Architecture Overview

### **Primals** (Sovereign Services)
- **biomeOS**: Orchestrator (this project)
- **Songbird**: P2P, discovery, BTSP
- **BearDog**: Security, encryption, identity
- **Toadstool**: Compute, workload management
- **NestGate**: Storage, provenance
- **petalTongue**: Universal UI

### **Niches** (Deployment Patterns)
- **Tower**: Communication (biomeOS + Songbird + BearDog)
- **Node**: Compute (Toadstool + optional BearDog)
- **Nest**: Data (NestGate + BearDog + Songbird)
- **UI**: Interface (petalTongue + biomeOS)

### **Communication**
- **Primary**: Unix sockets (JSON-RPC)
- **Discovery**: UDP multicast (Songbird)
- **Secure**: BTSP tunnels (BearDog + Songbird)

---

## 🧬 NUCLEUS (Secure Discovery)

5-layer verification protocol:

1. **Physical Discovery** (Songbird) - UDP multicast
2. **Identity Verification** (BearDog) - Ed25519 signatures
3. **Capability Verification** (biomeOS) - Query primal
4. **Trust Evaluation** (BearDog) - Genetic lineage
5. **Registration** (biomeOS) - Add to registry

**Trust Levels**: 0 (Unknown) → 4 (Highest/Sibling)

---

## 📋 Common Tasks

### **Deploy a Tower (USB Spore)**
```bash
cargo run --bin biomeos-spore -- create \
  --niche tower \
  --output /path/to/usb
```

### **Run NUCLEUS Tests**
```bash
cargo test --package biomeos-federation nucleus_tests
```

### **Check Unwraps (Deep Debt)**
```bash
grep -r "\.unwrap()\|\.expect(" \
  --include="*.rs" \
  --exclude="*test*.rs" \
  crates/ | wc -l
```

### **Build Specific Crate**
```bash
cargo build --package biomeos-api
cargo build --package biomeos-federation
cargo build --package biomeos-spore
```

---

## 🔍 Finding Things

### **Specifications**
```bash
ls specs/
# 30+ specs: NUCLEUS, Neural API, BYOB, Federation, etc.
```

### **Documentation**
```bash
ls docs/
# Guides, architecture, evolution plans, handoffs
```

### **Examples**
```bash
ls examples/
# Demo applications, config examples
```

### **Graphs**
```bash
ls graphs/
# Deployment graphs for tower, node, nest, ui
```

---

## 🎯 Next Steps

### **For Developers**
1. Read [README.md](README.md) for architecture
2. Check [STATUS.md](STATUS.md) for current state
3. Review [ROADMAP.md](ROADMAP.md) for phases
4. Explore [specs/](specs/) for deep dives

### **For Contributors**
1. Review deep debt principles in [docs/DEEP_DEBT_FINAL_STATUS_JAN9.md](docs/DEEP_DEBT_FINAL_STATUS_JAN9.md)
2. See evolution patterns (mocks, hardcoding, unwraps)
3. Follow modern Rust patterns (zero unsafe, graceful errors, capability-based)

### **For Integrators**
1. Read [docs/PETALTONGUE_TEAM_HANDOFF_JAN9.md](docs/PETALTONGUE_TEAM_HANDOFF_JAN9.md) for UI
2. Check topology API: `http://localhost:3000/api/v1/topology`
3. Review [specs/NUCLEUS_SECURE_DISCOVERY_PROTOCOL.md](specs/NUCLEUS_SECURE_DISCOVERY_PROTOCOL.md)

---

## 🆘 Troubleshooting

### **Build Fails**
```bash
# Clean and rebuild
cargo clean
cargo build --workspace
```

### **Tests Fail**
```bash
# Run specific test
cargo test --package biomeos-federation nucleus_tests -- --nocapture
```

### **API Won't Start**
```bash
# Check if port 3000 is in use
lsof -i :3000

# Use standalone mode
BIOMEOS_STANDALONE_MODE=true cargo run --package biomeos-api
```

---

## 📞 Getting Help

- **Documentation**: Check [docs/](docs/) and [specs/](specs/)
- **Status**: See [STATUS.md](STATUS.md)
- **Recent Work**: See [SESSION_SUMMARY_JAN9.md](SESSION_SUMMARY_JAN9.md)
- **Issues**: Check build logs, test output

---

## 🎊 Quick Wins

**Want to see it work right now?**

```bash
# 1. Start API server (standalone mode)
BIOMEOS_STANDALONE_MODE=true cargo run --package biomeos-api &

# 2. Test it (in another terminal)
curl http://localhost:3000/api/v1/health | jq '.'
curl http://localhost:3000/api/v1/topology | jq '.primals[] | {id, type, health}'

# 3. Stop it
pkill biomeos-api
```

**See tests pass:**
```bash
cargo test --package biomeos-federation nucleus_tests
```

---

**Welcome to the ecosystem!** 🌱✨

For more details, see [README.md](README.md) and [STATUS.md](STATUS.md).
