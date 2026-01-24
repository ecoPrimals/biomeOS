# 🚀 START HERE - biomeOS Quick Guide
## Updated: January 25, 2026

**Welcome to biomeOS!** This guide will get you oriented quickly.

---

## 📍 YOUR STARTING POINT

### **Are you...**

**New to the project?**  
→ Start with [`README.md`](./README.md) for project overview  
→ Then review [`MASTER_EXECUTION_PLAN_JAN_24_2026.md`](./MASTER_EXECUTION_PLAN_JAN_24_2026.md)

**A team member starting work?**  
→ See [**Immediate Priorities**](#immediate-priorities) below  
→ Review your team's handoff document

**Looking for documentation?**  
→ Use [`DOCS_INDEX.md`](./DOCS_INDEX.md) for comprehensive navigation

---

## 🎯 IMMEDIATE PRIORITIES (This Week)

### **Priority 1: Dual-Mode Implementation** (4-6 hours)
**Owner**: Songbird Team  
**Guide**: [`TEAM_HANDOFF_DUAL_MODE_IMPLEMENTATION_JAN_24_2026.md`](./TEAM_HANDOFF_DUAL_MODE_IMPLEMENTATION_JAN_24_2026.md)

**What**: Implement BearDogClient dual-mode support  
**Why**: Enables primal independence + self-testing  
**Output**: Primals can work without Neural API (testing) or with it (production)

### **Priority 2: HTTPS Validation** (2-3 hours)
**Owner**: All Teams  
**Guide**: [`TOWER_ATOMIC_CLIENT_SERVER_SELF_TEST_PLAN_JAN_24_2026.md`](./TOWER_ATOMIC_CLIENT_SERVER_SELF_TEST_PLAN_JAN_24_2026.md)

**What**: Run client/server self-test to find transcript differences  
**Why**: We're 99.95% there—keys and encryption proven correct!  
**Output**: HTTP 200 OK! 🎉

### **Priority 3: Deep Debt Phase 2** (7-10 hours)
**Owner**: biomeOS Team  
**Guide**: [`DEEP_DEBT_PHASE_2_STATUS_JAN_24_2026.md`](./DEEP_DEBT_PHASE_2_STATUS_JAN_24_2026.md)

**What**: Continue strategic refactoring (40% complete)  
**Why**: Production-ready, maintainable codebase  
**Output**: neural_executor.rs < 800 lines

---

## 📚 ESSENTIAL DOCUMENTS

### **Master Plans** (Start Here):
1. [`MASTER_EXECUTION_PLAN_JAN_24_2026.md`](./MASTER_EXECUTION_PLAN_JAN_24_2026.md)
   - 4 phases, 2-3 week timeline
   - All team priorities and dependencies

2. [`DEEP_DEBT_EXECUTION_BIOMEOS_JAN_24_2026.md`](./DEEP_DEBT_EXECUTION_BIOMEOS_JAN_24_2026.md)
   - 5 phases of code quality improvement
   - Strategic refactoring plans

### **Implementation Guides** (Copy-Paste Ready):
3. [`TEAM_HANDOFF_DUAL_MODE_IMPLEMENTATION_JAN_24_2026.md`](./TEAM_HANDOFF_DUAL_MODE_IMPLEMENTATION_JAN_24_2026.md)
   - Complete dual-mode implementation code

4. [`TOWER_ATOMIC_CLIENT_SERVER_SELF_TEST_PLAN_JAN_24_2026.md`](./TOWER_ATOMIC_CLIENT_SERVER_SELF_TEST_PLAN_JAN_24_2026.md)
   - HTTPS validation strategy

### **Architecture** (Understand the System):
5. [`ARCHITECTURAL_CLARITY_NEURAL_API_EVOLUTION_ENGINE_JAN_24_2026.md`](./ARCHITECTURAL_CLARITY_NEURAL_API_EVOLUTION_ENGINE_JAN_24_2026.md)
   - Neural API as evolution engine

6. [`ARCHITECTURAL_EVOLUTION_PRIMAL_INDEPENDENCE_JAN_24_2026.md`](./ARCHITECTURAL_EVOLUTION_PRIMAL_INDEPENDENCE_JAN_24_2026.md)
   - TRUE PRIMAL principles

### **Progress Tracking**:
7. [`SESSION_COMPLETE_22_HOURS_JAN_24_2026.md`](./SESSION_COMPLETE_22_HOURS_JAN_24_2026.md)
   - Latest session summary (comprehensive)

8. [`DEEP_DEBT_PHASE_1_PROGRESS_JAN_24_2026.md`](./DEEP_DEBT_PHASE_1_PROGRESS_JAN_24_2026.md)
   - Phase 1 completion details

---

## 🏗️ PROJECT STRUCTURE

```
biomeOS/
├── README.md                  ← Project overview
├── START_HERE.md             ← You are here!
├── DOCS_INDEX.md             ← Full doc navigation
├── MASTER_EXECUTION_PLAN_JAN_24_2026.md  ← Master roadmap
│
├── crates/                    ← Rust crates
│   ├── biomeos-types/        ← Shared types + defaults.rs (NEW!)
│   ├── biomeos-atomic-deploy/← Neural API + executor/ (NEW!)
│   └── ...                   ← 25 total crates
│
├── graphs/                    ← Neural API deployment graphs
├── scripts/                   ← Utility scripts
├── docs/                      ← Additional documentation
├── archive/                   ← Historical documents
│   └── session_jan_24_2026/  ← Jan 24-25 session docs
└── ...
```

---

## 💻 QUICK START

### **1. Clone and Build**
```bash
# Clone (if not already done)
git clone <repository-url>
cd biomeOS

# Build all crates
cargo build --release

# Run tests
cargo test
```

### **2. Deploy Tower Atomic** (BearDog + Songbird)
```bash
# Using Neural API (production mode)
./target/release/biomeos neural-api \
  --graph graphs/tower_atomic_bootstrap.toml

# Check logs
tail -f /tmp/neural-api.log
```

### **3. Verify Deployment**
```bash
# Check BearDog
./scripts/test_beardog_health.sh

# Test HTTPS (once validated)
./scripts/test_https_endpoints.sh
```

---

## 🎯 KEY CONCEPTS

### **TRUE PRIMAL Architecture**
- ✅ **Self-knowledge only** - Primals know themselves, discover others
- ✅ **Runtime discovery** - No hardcoded cross-primal knowledge
- ✅ **Capability-based** - Request by capability, not by name
- ✅ **Unix sockets** - JSON-RPC 2.0 over Unix domain sockets

### **Neural API**
- ✅ **Evolution engine** - Enables primal evolution without breaking clients
- ✅ **Discovery** - Capability-based service discovery
- ✅ **Semantic translation** - Stable names → provider-specific methods
- ✅ **Orchestration** - Multi-primal workflows (neural graphs)

### **Dual-Mode Operation**
- ✅ **Direct mode** - Primal-to-primal (testing, simple deployments)
- ✅ **Neural API mode** - Via orchestration layer (production, evolution)
- ✅ **Both valid** - Choose based on use case

---

## 📊 CURRENT STATUS

### **Progress**:
- **HTTPS**: 99.95% (keys correct, encryption correct)
- **Deep Debt Phase 1**: 100% ✅
- **Deep Debt Phase 2**: 40% 🔄
- **Documentation**: 21,500+ lines ✅
- **Team Readiness**: 100% ✅

### **This Week's Goals**:
1. Dual-mode implementation complete
2. HTTPS validation complete (HTTP 200 OK!)
3. Phase 2 refactoring progressing

### **Next 2-3 Weeks**:
- Neural API evolution
- Capability-based discovery
- Production-ready system

---

## 🤝 GETTING HELP

### **Documentation**:
- **Overview**: [`README.md`](./README.md)
- **Navigation**: [`DOCS_INDEX.md`](./DOCS_INDEX.md)
- **Architecture**: Architecture docs (see Essential Documents)
- **Implementation**: Team handoff guides

### **Code**:
- **Runtime config**: `crates/biomeos-types/src/defaults.rs`
- **Executor**: `crates/biomeos-atomic-deploy/src/executor/`
- **Examples**: `crates/*/examples/`

### **Questions**:
- Check documentation first
- Review relevant handoff documents
- Refer to master execution plan

---

## 🔍 COMMON TASKS

### **"I need to add a new primal..."**
→ See [`MASTER_EXECUTION_PLAN_JAN_24_2026.md`](./MASTER_EXECUTION_PLAN_JAN_24_2026.md) Phase 3

### **"I need to understand the architecture..."**
→ Read architecture docs (items 5-6 in Essential Documents)

### **"I need to test HTTPS..."**
→ Follow [`TOWER_ATOMIC_CLIENT_SERVER_SELF_TEST_PLAN_JAN_24_2026.md`](./TOWER_ATOMIC_CLIENT_SERVER_SELF_TEST_PLAN_JAN_24_2026.md)

### **"I need to add configuration..."**
→ Use `biomeos_types::defaults::RuntimeConfig` or environment variables

### **"I need to understand deep debt..."**
→ Review [`DEEP_DEBT_EXECUTION_BIOMEOS_JAN_24_2026.md`](./DEEP_DEBT_EXECUTION_BIOMEOS_JAN_24_2026.md)

---

## ✅ CHECKLIST FOR NEW CONTRIBUTORS

- [ ] Read [`README.md`](./README.md)
- [ ] Review [`MASTER_EXECUTION_PLAN_JAN_24_2026.md`](./MASTER_EXECUTION_PLAN_JAN_24_2026.md)
- [ ] Understand TRUE PRIMAL architecture
- [ ] Build project (`cargo build --release`)
- [ ] Run tests (`cargo test`)
- [ ] Review your team's handoff document
- [ ] Start on immediate priorities

---

## 🎊 RECENT ACHIEVEMENTS

### **22+ Hour Session (Jan 24-25, 2026)**:
- ✅ Comprehensive deep debt audit (25 crates, 150k LOC)
- ✅ Runtime configuration system (defaults.rs)
- ✅ Strategic refactoring started (3 modules, 630 lines extracted)
- ✅ Master execution plans (21,500+ lines documentation)
- ✅ Team handoffs (copy-paste ready code)
- ✅ Architecture clarity (evolution engine, primal independence)

**Impact**: Foundation-changing infrastructure for production-ready deployments

---

**"Start with the plan, dive into details!"** 🎯  
**"Documentation is your friend!"** 📚  
**"TRUE PRIMAL: Self-knowledge only, runtime discovery!"** 🔬  
**"Production-ready in 2-3 weeks!"** 🚀  

**Welcome to biomeOS! Let's build something amazing!** 🎉
