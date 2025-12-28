# BiomeOS Showcase - Quick Start Guide
**Last Updated:** December 25, 2025  
**Status:** Ready to run - NO MOCKS, real integration only

---

## 🚀 Getting Started in 5 Minutes

### Step 1: Choose Your Path

**New to BiomeOS?** Start here:
```bash
cd showcase/00-local-capabilities/
./run-all-local-demos.sh
```
Shows BiomeOS core capabilities without any primals (10 min)

**Ready for Real Integration?** Start here:
```bash
cd showcase/01-single-primal/
./run-all-single-primal-demos.sh
```
Tests BiomeOS with real Phase 1 binaries (25 min)

---

## 📋 Prerequisites

### For 00-local-capabilities:
- ✅ BiomeOS built (`cargo build --release`)
- ✅ That's it! No primals needed.

### For 01-single-primal:
- ✅ Phase 1 binaries available
- ✅ Ports available (3000, 8080, 8002, 9000, 8001)

**Get Phase 1 binaries:**
```bash
cd ../phase1bins/
./pull-phase1-bins.sh
```

---

## 🎯 What's Available Now

| Scenario | Status | Duration | What It Shows |
|----------|--------|----------|---------------|
| **00-local-capabilities** | ✅ Ready | 10 min | BiomeOS core without primals |
| **01-single-primal** | ✅ Ready | 25 min | BiomeOS + each primal individually |
| **02-multi-primal** | 🔄 Next | TBD | Cross-primal orchestration |
| **03-05** | ✅ Done | - | Already complete (adapter demos) |
| **06-10** | ⏸️ Planned | - | Advanced scenarios |

---

## 📖 Demo Descriptions

### 00-local-capabilities

**Purpose:** See BiomeOS value before connecting any primals

**Demos:**
1. **Manifest Parsing** - Parse biome.yaml files
2. **Capability Matching** - Match requirements to capabilities
3. **Sovereignty Guardian** - Privacy protections
4. **Client Registry** - Client initialization

**Run:**
```bash
cd showcase/00-local-capabilities/
./run-all-local-demos.sh
```

---

### 01-single-primal

**Purpose:** Test BiomeOS discovering and using real Phase 1 primals

**Demos:**
1. **Songbird** - Service discovery and mesh
2. **ToadStool** - Compute orchestration
3. **NestGate** - Storage operations
4. **BearDog** - Crypto operations
5. **Squirrel** - AI agent management

**Run all:**
```bash
cd showcase/01-single-primal/
./run-all-single-primal-demos.sh
```

**Run individual:**
```bash
./songbird-discovery.sh
./toadstool-compute.sh
# etc.
```

---

## 🔍 Gap Discovery

**Philosophy:** We use showcase to find REAL gaps in live integration

**As you run demos:**
- Gaps are automatically documented
- Reports saved in `gaps/` directory
- Use findings to improve adapters

**View gaps:**
```bash
cd showcase/01-single-primal/gaps/
ls -l *.md
cat songbird-gaps.md
```

---

## 🎓 Learning Paths

### For New Users:
1. Run `00-local-capabilities` (understand BiomeOS)
2. Run `01-single-primal` (see primal integration)
3. Review gap reports (learn what's working)

### For Integration Testing:
1. Get Phase 1 binaries
2. Run `01-single-primal` demos
3. Document all gaps found
4. Coordinate fixes with primal teams

### For Developers:
1. Study demo source code
2. Review gap reports
3. Improve adapters based on findings
4. Re-run demos to verify fixes

---

## 🛠️ Common Issues

### Primal Won't Start

**Check binary exists:**
```bash
ls -lh ../phase1bins/*-bin
```

**Check port available:**
```bash
lsof -i :3000  # Songbird
lsof -i :8080  # ToadStool
# etc.
```

**Check logs:**
```bash
cat showcase/01-single-primal/logs/songbird.log
```

### Discovery Fails

**Verify primal running:**
```bash
curl http://localhost:3000/health  # Songbird
```

**Check endpoint:**
```bash
echo $SONGBIRD_ENDPOINT
```

**Try explicit endpoint:**
```bash
export SONGBIRD_ENDPOINT="http://localhost:3000"
./songbird-discovery.sh
```

---

## 📊 What to Expect

### 00-local-capabilities

**Output:**
- Manifest parsing examples
- Capability matching demonstrations
- Sovereignty policy examples
- Client registry initialization

**Success Criteria:**
- All demos complete without errors
- Clear output showing BiomeOS features
- Understanding of core capabilities

---

### 01-single-primal

**Output:**
- Real primal starts successfully
- BiomeOS discovers primal
- Test operations succeed
- Gap reports generated

**Success Criteria:**
- Primal starts on expected port
- Health checks pass
- Basic operations work
- Gaps documented for improvement

---

## 🎯 Next Steps After Running

### 1. Review Gap Reports

```bash
cd showcase/01-single-primal/gaps/
# Read each gap report
# Prioritize issues
# Coordinate with primal teams
```

### 2. Update Adapters

Based on gaps found:
- Fix API endpoint issues
- Improve error handling
- Update documentation
- Re-test

### 3. Move to Multi-Primal

After single-primal works:
```bash
cd showcase/02-multi-primal/
# Test cross-primal workflows
```

---

## 📚 Additional Resources

### Documentation:
- `README.md` - Main showcase overview
- `00-local-capabilities/README.md` - Local demos guide
- `01-single-primal/README.md` - Single primal guide
- `SESSION_COMPLETE_DEC_25_2025.md` - Full session summary

### Planning:
- `SHOWCASE_BUILDOUT_PLAN_DEC_25_2025.md` - Complete plan
- `SHOWCASE_REVIEW_SUMMARY_DEC_25_2025.md` - Phase 1 analysis
- `EXECUTION_SUMMARY_DEC_25_2025.md` - What we built

---

## 🤝 Contributing

### Found a Gap?

1. Document in appropriate gap report
2. Include:
   - What didn't work
   - What was expected
   - Steps to reproduce
   - Suggested fix

### Want to Add a Demo?

1. Follow existing demo structure
2. Include gap discovery
3. Test with real primals
4. Update README

---

## 💡 Tips

### Maximize Learning:
- Read demo scripts before running
- Watch output carefully
- Review gap reports immediately
- Test fixes incrementally

### Maximize Efficiency:
- Run local demos first (no setup)
- Test one primal at a time
- Document as you go
- Iterate quickly

### Maximize Quality:
- Use real binaries only
- Document all gaps
- Coordinate with primal teams
- Verify fixes work

---

## 🎉 Ready to Go!

**Start now:**
```bash
cd showcase/00-local-capabilities/
./run-all-local-demos.sh
```

**Then:**
```bash
cd showcase/01-single-primal/
./run-all-single-primal-demos.sh
```

**Then document and improve!**

---

**Questions?** Check the comprehensive documentation in each scenario directory.

**Issues?** Document in gap reports and coordinate fixes.

**Success?** Move to the next scenario and keep building!

---

*"Real primals, real gaps, real improvements. BiomeOS evolves through discovery."* 🌱

