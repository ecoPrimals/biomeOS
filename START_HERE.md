# 🚀 Start Here - biomeOS Quick Orientation

**Welcome to biomeOS!** This guide will get you oriented quickly.

---

## 📍 **What is biomeOS?**

biomeOS is the **orchestrator** for the ecoPrimals ecosystem, enabling:

- **Composable Primal Deployments** - Mix and match primals (BearDog, Songbird, NestGate, etc.)
- **Cross-Primal Communication** - Unix sockets + JSON-RPC with semantic routing
- **Capability-Based Discovery** - Primals discover each other at runtime
- **Graph-Based Deployment** - Declare your stack in TOML, biomeOS handles the rest

**Think of it as:** Docker Compose + Kubernetes... but for primals, written in Pure Rust, with semantic routing.

---

## 🎯 **I Want To...**

### **Understand the Architecture**

→ Read: [BIOMEOS_ATOMICS_ARCHITECTURE.md](BIOMEOS_ATOMICS_ARCHITECTURE.md)

**Key Concepts:**
- **NUCLEUS Atomics:** Tower (security), Node (compute), Nest (storage)
- **Primal Self-Knowledge:** Each primal manages itself
- **Capability Routing:** `neural_api.call("http.request", ...)` → Songbird

### **See What's Been Accomplished**

→ Read: [ECOSYSTEM_HARVEST_COMPLETE_100_PERCENT.md](ECOSYSTEM_HARVEST_COMPLETE_100_PERCENT.md)

**Highlights:**
- ✅ 3/3 primal teams responded in <24 hours
- ✅ All A+/A++ quality implementations
- ✅ 5,000+ tests passing across ecosystem
- ✅ Socket standard established

### **Run the NUCLEUS Stack**

→ Read: [QUICK_START.md](QUICK_START.md)

**Quick Command:**
```bash
./scripts/quick_start_nucleus_test.sh
```

### **Integrate a New Primal**

→ Read: [BIOMEOS_PRIMAL_INTEGRATION_SPEC.md](BIOMEOS_PRIMAL_INTEGRATION_SPEC.md)

**Steps:**
1. Create Unix socket at `/run/user/$UID/biomeos/{primal}.sock`
2. Implement JSON-RPC 2.0 server
3. Register capabilities
4. Add to deployment graph

### **Run Tests**

→ Read: [NUCLEUS_TEST_INDEX.md](NUCLEUS_TEST_INDEX.md)

**Quick Command:**
```bash
cargo test --workspace
```

### **Deploy to Production**

→ Read: [DEPLOYMENT.md](DEPLOYMENT.md)

**Checklist:** [PRODUCTION_DEPLOYMENT_CHECKLIST.md](PRODUCTION_DEPLOYMENT_CHECKLIST.md)

### **Find Everything**

→ Read: [ROOT_INDEX.md](ROOT_INDEX.md) - Complete documentation index

---

## 🏗️ **Project Structure**

```
biomeOS/
├── crates/                     # Rust crates
│   ├── biomeos-nucleus/        # Core orchestrator
│   ├── biomeos-types/          # Shared types
│   ├── biomeos-chimera/        # Graph executor
│   └── biomeos-niche/          # Ecosystem manager
├── graphs/                     # Deployment graphs (.toml)
├── scripts/                    # Automation scripts
├── docs/                       # Documentation
│   └── handoffs/               # Primal handoff documents
├── specs/                      # Architecture specs
├── archive/                    # Archived sessions
└── *.md                        # Root documentation
```

---

## 🦀 **The Primals**

biomeOS orchestrates these autonomous primals:

| Primal | Purpose | Socket | Status |
|--------|---------|--------|--------|
| **BearDog** | Crypto/TLS/HSM | `beardog.sock` | ✅ Ready |
| **Songbird** | HTTP/Discovery/Federation | `songbird.sock` | ✅ Ready |
| **NestGate** | Storage/Models/Persistence | `nestgate.sock` | ✅ Ready |
| **Toadstool** | GPU Compute/AI Inference | `toadstool.sock` | ✅ Ready |
| **Squirrel** | AI Multi-Provider (local+online) | `squirrel.sock` | ✅ Ready |

**Socket Location:** `/run/user/$UID/biomeos/{primal}.sock`

---

## 🎯 **NUCLEUS Atomics**

biomeOS deploys primals in three **atomic compositions**:

### **1. Tower Atomic** (Security Foundation)
```
BearDog + Songbird
```
- Crypto operations
- TLS/HTTPS
- Service discovery
- Secure HTTP

### **2. Node Atomic** (Compute Foundation)
```
Tower + Toadstool
```
- Everything in Tower
- GPU compute (NVIDIA)
- Local AI inference
- Hardware acceleration

### **3. Nest Atomic** (Persistence Foundation)
```
Tower + NestGate
```
- Everything in Tower
- Persistent storage
- Model caching
- HuggingFace integration

---

## 📚 **Essential Reading**

### **New to biomeOS?**

1. **[README.md](README.md)** - Project overview
2. **[BIOMEOS_ATOMICS_ARCHITECTURE.md](BIOMEOS_ATOMICS_ARCHITECTURE.md)** - Architecture
3. **[QUICK_START.md](QUICK_START.md)** - Quick start guide
4. **[NUCLEUS_AI_INTEGRATION_GUIDE.md](NUCLEUS_AI_INTEGRATION_GUIDE.md)** - Integration guide

### **Recent Accomplishments?**

1. **[ECOSYSTEM_HARVEST_COMPLETE_100_PERCENT.md](ECOSYSTEM_HARVEST_COMPLETE_100_PERCENT.md)** - Ecosystem status
2. **[NUCLEUS_DEEP_DEBT_MISSION_COMPLETE.md](NUCLEUS_DEEP_DEBT_MISSION_COMPLETE.md)** - Mission summary
3. **[PRIMAL_HARVEST_COMPLETE.md](PRIMAL_HARVEST_COMPLETE.md)** - Primal harvests

### **Want Details?**

→ **[ROOT_INDEX.md](ROOT_INDEX.md)** - Complete documentation index (20+ documents organized)

---

## 🚀 **Quick Commands**

### **Build Everything**
```bash
cargo build --release --workspace
```

### **Run Tests**
```bash
cargo test --workspace
```

### **Start NUCLEUS Stack**
```bash
./scripts/quick_start_nucleus_test.sh
```

### **Check Socket Status**
```bash
ls -la /run/user/$(id -u)/biomeos/
```

### **Deploy Custom Graph**
```bash
cargo run --release -- deploy graphs/my_graph.toml
```

---

## 🎊 **Current Status (Jan 30, 2026)**

### **Production Ready: ✅**

- ✅ Socket standard established
- ✅ All 5 NUCLEUS primals ready
- ✅ 3/3 primal teams responded (<24h)
- ✅ 5,000+ tests passing (100%)
- ✅ All atomics functional

### **What Just Happened?**

**NUCLEUS Integration Mission (Jan 29-30, 2026):**
- Investigated socket discovery issues
- Fixed biomeOS discovery logic
- Created handoff documents for 3 primal teams
- **All 3 teams responded in <24 hours** with excellent implementations
- Achieved 100% ecosystem readiness

**This is unprecedented in distributed ecosystem development!**

---

## 💡 **Key Concepts**

### **Primal Self-Knowledge**

Each primal:
- Creates its own socket
- Discovers others at runtime
- Registers capabilities
- No compile-time coupling

### **Capability-Based Routing**

```rust
// Semantic capability calls
neural_api.call("http.request", request_params)    // → Songbird
neural_api.call("crypto.sign", sign_params)        // → BearDog
neural_api.call("storage.put", storage_params)     // → NestGate
neural_api.call("ai.query", ai_params)             // → Squirrel
```

### **Graph-Based Deployment**

```toml
# graphs/my_stack.toml
[[primals]]
name = "beardog"
binary = "../phase1/beardog/target/release/beardog"
args = ["server"]

[[primals]]
name = "songbird"
binary = "../phase1/songbird/target/release/songbird"
args = ["server"]
```

---

## 🤝 **Getting Help**

- **Documentation:** [ROOT_INDEX.md](ROOT_INDEX.md)
- **Architecture:** [BIOMEOS_ATOMICS_ARCHITECTURE.md](BIOMEOS_ATOMICS_ARCHITECTURE.md)
- **Testing:** [NUCLEUS_TEST_INDEX.md](NUCLEUS_TEST_INDEX.md)
- **Deployment:** [DEPLOYMENT.md](DEPLOYMENT.md)

---

## 🙏 **Credits**

**NUCLEUS Mission Success:**
- **NestGate Team** - A++ 99.7/100, 4-tier innovation
- **Songbird Team** - A+, Pure Rust XDG
- **BearDog Team** - A++ 100/100, 5-tier pattern

**Philosophy:** TRUE PRIMAL architecture - autonomous primals, ecosystem coordination.

---

**Next Steps:**
1. Read [README.md](README.md) for project overview
2. Check [ROOT_INDEX.md](ROOT_INDEX.md) for complete docs
3. Run [QUICK_START.md](QUICK_START.md) to deploy NUCLEUS
4. Explore [specs/](specs/) for architecture details

**🦀✨ Welcome to the ecosystem! ✨🦀**
