# BiomeOS Showcase - Master Index
**Complete reference for all showcase scenarios**  
**Updated:** December 25, 2025

---

## 📊 Scenario Overview

| # | Scenario | Status | Duration | Files | Purpose |
|---|----------|--------|----------|-------|---------|
| 00 | local-capabilities | ✅ Complete | 10 min | 6 | Core BiomeOS without primals |
| 01 | single-primal | ✅ Complete | 25 min | 8 | BiomeOS + individual primals |
| 02 | multi-primal | ⏸️ Planned | TBD | - | Cross-primal orchestration |
| 03 | primal-adapter | ✅ Complete | - | - | Adapter pattern (existing) |
| 04 | multi-primal-adaptation | ✅ Complete | - | - | Mock adapters (existing) |
| 05 | lifecycle-negotiation | ✅ Complete | - | - | Lifecycle demos (existing) |
| 06 | federation | ⏸️ Planned | TBD | - | Multi-tower (Songbird-inspired) |
| 07 | monitoring | ⏸️ Planned | TBD | - | Live visibility |
| 08 | failure-recovery | ⏸️ Planned | TBD | - | Chaos engineering |
| 09 | sovereignty | ⏸️ Planned | TBD | - | Privacy validation |
| 10 | integration | ⏸️ Planned | TBD | - | E2E testing |

**Progress: 5/11 (45%)**

---

## 🚀 Quick Navigation

### Start Here:
- **[QUICK_START.md](QUICK_START.md)** - Get started in 5 minutes
- **[README.md](README.md)** - Main showcase overview

### Planning & Review:
- **[SHOWCASE_BUILDOUT_PLAN_DEC_25_2025.md](SHOWCASE_BUILDOUT_PLAN_DEC_25_2025.md)** - Complete 4-week plan
- **[SHOWCASE_REVIEW_SUMMARY_DEC_25_2025.md](SHOWCASE_REVIEW_SUMMARY_DEC_25_2025.md)** - Phase 1 analysis
- **[SESSION_COMPLETE_DEC_25_2025.md](SESSION_COMPLETE_DEC_25_2025.md)** - Session summary

### Execution:
- **[EXECUTION_SUMMARY_DEC_25_2025.md](EXECUTION_SUMMARY_DEC_25_2025.md)** - What we built
- **[SHOWCASE_PROGRESS_DEC_25_2025.md](SHOWCASE_PROGRESS_DEC_25_2025.md)** - Progress tracking

---

## 📁 Scenario Details

### 00-local-capabilities ✅

**Location:** `showcase/00-local-capabilities/`  
**Status:** Complete and ready to run  
**Purpose:** Demonstrate BiomeOS core capabilities without any primals

**Files:**
- `README.md` - Comprehensive guide
- `01-manifest-parsing.sh` - Parse biome.yaml
- `02-capability-matching.sh` - Capability matching
- `04-sovereignty-guardian.sh` - Privacy protections
- `05-client-registry.sh` - Client initialization
- `run-all-local-demos.sh` - Automated runner
- `GAPS_DISCOVERED.md` - Gap tracking

**Run:**
```bash
cd 00-local-capabilities/
./run-all-local-demos.sh
```

**Demonstrates:**
- Manifest parsing and validation
- Capability-based discovery logic
- Sovereignty guardian protections
- Client registry management

---

### 01-single-primal ✅

**Location:** `showcase/01-single-primal/`  
**Status:** Complete and ready to test with real binaries  
**Purpose:** Test BiomeOS discovering and using each Phase 1 primal

**Files:**
- `README.md` - Integration guide
- `common/start-primal.sh` - Start any primal
- `common/stop-primal.sh` - Stop any primal
- `songbird-discovery.sh` - Service discovery demo
- `toadstool-compute.sh` - Compute orchestration
- `nestgate-storage.sh` - Storage operations
- `beardog-security.sh` - Crypto operations
- `squirrel-ai.sh` - AI agent management
- `run-all-single-primal-demos.sh` - Full suite

**Run:**
```bash
cd 01-single-primal/
./run-all-single-primal-demos.sh
```

**Demonstrates:**
- Real primal discovery
- API endpoint exploration
- Integration gap discovery
- Adapter validation

**Gap Reports:** Auto-generated in `gaps/` directory

---

### 02-multi-primal ⏸️

**Location:** `showcase/02-multi-primal/`  
**Status:** Planned  
**Purpose:** Cross-primal orchestration and workflows

**Planned Demos:**
- `storage-plus-discovery.sh` - NestGate + Songbird
- `compute-plus-discovery.sh` - ToadStool + Songbird
- `secure-storage.sh` - BearDog + NestGate
- `ai-compute.sh` - Squirrel + ToadStool
- `full-stack.sh` - All 5 primals

**Demonstrates:**
- Multi-primal coordination
- Cross-primal communication
- Unified configuration
- Complete ecosystem

---

### 03-05: Existing Scenarios ✅

**03-primal-adapter**
- Status: ✅ Complete
- Tests: 9/9 passing
- Purpose: Primal adapter pattern

**04-multi-primal-adaptation**
- Status: ✅ Complete
- Purpose: Mock primals for testing

**05-lifecycle-negotiation**
- Status: ✅ Complete
- Purpose: Lifecycle demos

---

### 06-federation ⏸️

**Location:** `showcase/06-federation/`  
**Status:** Planned (Week 4)  
**Purpose:** Multi-tower federation (Songbird-inspired)

**Planned Demos:**
- `local-federation.sh` - Simulated multi-tower
- `tower-discovery.sh` - Cross-tower mesh
- `proximity-routing.sh` - Geographic optimization

**Inspired By:** Songbird's excellent multi-tower demos

---

### 07-monitoring ⏸️

**Location:** `showcase/07-monitoring/`  
**Status:** Planned (Week 4)  
**Purpose:** Live ecosystem monitoring

**Planned Demos:**
- `health-dashboard.sh` - Live health status
- `metrics-aggregation.sh` - Real-time metrics
- `topology-visualization.sh` - Ecosystem topology

---

### 08-failure-recovery ⏸️

**Location:** `showcase/08-failure-recovery/`  
**Status:** Planned (Week 4)  
**Purpose:** Chaos engineering and resilience

**Planned Demos:**
- `primal-crash.sh` - Handle primal failure
- `network-partition.sh` - Handle network issues
- `rolling-update.sh` - Zero-downtime updates
- `chaos-monkey.sh` - Random failure injection

---

### 09-sovereignty ⏸️

**Location:** `showcase/09-sovereignty/`  
**Status:** Planned (Week 4)  
**Purpose:** Privacy and consent validation

**Planned Demos:**
- `consent-management.sh` - Data consent flow
- `privacy-protection.sh` - Privacy enforcement
- `audit-trail.sh` - Complete audit log
- `vendor-lock-prevention.sh` - Portability validation

---

### 10-integration ⏸️

**Location:** `showcase/10-integration/`  
**Status:** Planned (Week 4)  
**Purpose:** End-to-end integration testing

**Planned Demos:**
- `e2e-workflow.sh` - Complete workflows
- `cross-primal-apis.sh` - API contract validation
- `real-vs-mock.sh` - Compare implementations

---

## 🎯 Running the Complete Showcase

### Full Test Suite (when all complete):
```bash
cd showcase/
./run-all-showcases.sh  # (to be created)
```

### Current Available Tests:
```bash
# Test local capabilities
cd 00-local-capabilities/ && ./run-all-local-demos.sh

# Test single primal integration
cd ../01-single-primal/ && ./run-all-single-primal-demos.sh
```

### Estimated Total Time:
- Current available: ~35 minutes
- When all complete: ~3-4 hours

---

## 📈 Progress Tracking

### Completed (5/11):
- ✅ 00-local-capabilities
- ✅ 01-single-primal
- ✅ 03-primal-adapter
- ✅ 04-multi-primal-adaptation
- ✅ 05-lifecycle-negotiation

### Next Up:
- 🔄 02-multi-primal (Week 2)
- ⏸️ 06-10 (Week 3-4)

---

## 🔍 Gap Discovery Status

### Active Gap Reports:
```
01-single-primal/gaps/
├── songbird-gaps.md    (to be generated)
├── toadstool-gaps.md   (to be generated)
├── nestgate-gaps.md    (to be generated)
├── beardog-gaps.md     (to be generated)
└── squirrel-gaps.md    (to be generated)
```

### Review Process:
1. Run demos with real binaries
2. Gap reports auto-generated
3. Review and prioritize
4. Coordinate fixes
5. Re-run to verify

---

## 📚 Documentation Matrix

| Document | Purpose | Audience |
|----------|---------|----------|
| QUICK_START.md | Get started fast | Everyone |
| MASTER_INDEX.md | Complete reference | Everyone |
| README.md | Overview | New users |
| SHOWCASE_BUILDOUT_PLAN | Implementation plan | Developers |
| SHOWCASE_REVIEW_SUMMARY | Phase 1 analysis | Architects |
| SESSION_COMPLETE | Session summary | Team |

---

## 🎓 Learning Paths

### New User Path:
1. Read QUICK_START.md
2. Run 00-local-capabilities
3. Run 01-single-primal
4. Review gap reports

### Developer Path:
1. Review all documentation
2. Study demo source code
3. Run all available demos
4. Analyze gap reports
5. Improve adapters

### Integration Tester Path:
1. Get Phase 1 binaries
2. Run 01-single-primal
3. Document all gaps
4. Coordinate with primal teams
5. Verify fixes

---

## 🎯 Success Metrics

### For Each Scenario:
- ✅ All demos executable
- ✅ Clear documentation
- ✅ Gap discovery active
- ✅ Real integration (no mocks)

### Overall:
- Current: 5/11 scenarios (45%)
- Target: 11/11 scenarios (100%)
- Timeline: 4 weeks

---

## 🚀 Quick Commands

```bash
# Get started
cd showcase/
cat QUICK_START.md

# Test local
cd 00-local-capabilities/ && ./run-all-local-demos.sh

# Test integration
cd 01-single-primal/ && ./run-all-single-primal-demos.sh

# Review gaps
cd 01-single-primal/gaps/ && ls -l

# Check status
cat MASTER_INDEX.md
```

---

## 📞 Support

**Questions?**
- Check scenario README files
- Review comprehensive documentation
- Examine demo source code

**Issues?**
- Document in gap reports
- Review troubleshooting sections
- Check common issues in QUICK_START.md

**Contributions?**
- Follow existing patterns
- Include gap discovery
- Test with real primals
- Update documentation

---

**Last Updated:** December 25, 2025  
**Status:** 45% complete, excellent progress  
**Next:** Run demos, discover gaps, build 02-multi-primal

---

*"BiomeOS orchestrates. Primals provide capabilities. Showcase reveals truth."* 🌱

