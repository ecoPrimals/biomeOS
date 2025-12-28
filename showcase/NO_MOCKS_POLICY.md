# 🚫 SHOWCASE POLICY - NO MOCKS ALLOWED

**Date**: December 28, 2025  
**Status**: ENFORCED

---

## Policy: LIVE DEMONSTRATIONS ONLY

**ALL showcase demonstrations MUST use real, live primals.**

### ❌ FORBIDDEN
- Mock HTTP servers
- Python mock scripts
- Simulated responses
- Test doubles
- Stub implementations

### ✅ REQUIRED
- Real primal binaries from `primals/`
- Live HTTP endpoints
- Actual service discovery
- Real capability negotiation
- Production-level interactions

---

## Why This Policy?

### 1. Expose Real Gaps
Mocks hide problems. Live primals expose:
- Integration failures
- Discovery issues
- Protocol mismatches
- Performance problems
- Security gaps

### 2. Validate Evolution
Showcase is our validation layer:
- **If it works in showcase** → Feature is real
- **If it fails in showcase** → Gap identified
- **If it's not in showcase** → Not validated

### 3. Demonstrate Reality
Showcase is for demonstration:
- To stakeholders
- To users
- To ourselves

**Demonstrations must be real, or they're lies.**

---

## Enforcement

### Removed (Dec 28, 2025)
- ❌ `showcase/04-multi-primal-adaptation/mock-primals/` (5 mock scripts)
- ❌ `showcase/05-lifecycle-negotiation/lifecycle-mocks/` (3 mock scripts)
- ❌ `showcase/04-multi-primal-adaptation/demo-mock.sh`

### Required for All Demos
1. Check real primals running: `ps aux | grep -E "(beardog|songbird)"`
2. Use actual endpoints: `http://localhost:PORT` (real services)
3. Handle failures gracefully: If primal not running, demo fails visibly
4. Document gaps: If something doesn't work, that's the point!

---

## How to Use Showcase

### 1. Deploy Real Primals First
```bash
# Deploy real primals
./deploy-real-primals.sh

# Verify running
ps aux | grep -E "(beardog|songbird|nestgate)" | grep -v grep
```

### 2. Run Showcase Demo
```bash
cd showcase/01-single-primal/
./songbird-discovery.sh  # Uses REAL Songbird
```

### 3. Document Failures
If demo fails:
1. ✅ **GOOD** - We found a gap!
2. Document the failure
3. Add to integration TODO list
4. Fix the root cause

**Mocking would hide the gap. Failure exposes it.**

---

## Integration with primalTools

### benchScale Location
```
/home/eastgate/Development/ecoPrimals/primalTools/benchscale/
```

### Usage in Showcase
```bash
# For VM federation demos
cd showcase/03-p2p-coordination/
# Reference benchscale for multi-VM deployments
BENCHSCALE_PATH=../../../../primalTools/benchscale
```

### bingoCube (Future)
```
/home/eastgate/Development/ecoPrimals/primalTools/bingoCube/
```
- Workflow orchestration tool
- Can be used for complex multi-primal demos
- Integration TBD

---

## Testing vs Showcase

### Testing (Mocks Allowed)
```
tests/                 ✅ Mocks OK here
crates/*/tests/        ✅ Unit tests with mocks
examples/              ✅ Example code, can use mocks
```

### Showcase (NO Mocks)
```
showcase/              ❌ LIVE ONLY
  01-single-primal/    ❌ Real primals required
  02-primal-pairs/     ❌ Real interactions required
  03-full-ecosystem/   ❌ Full live ecosystem required
```

---

## Gaps This Policy Exposes

### Already Identified
1. ❌ Real primals not deployed
2. ❌ benchScale integration incomplete
3. ❌ CLI binary missing
4. ❌ VM federation untested
5. ❌ Multi-tower coordination untested

### To Be Discovered
- Running showcase with real primals will expose more!
- Each failure is a learning opportunity
- Each success is validated functionality

---

## Compliance

### Check Compliance
```bash
# Should return nothing
find showcase/ -name "*mock*"

# Should return nothing
grep -r "python3.*http.server" showcase/

# Should return nothing  
grep -r "MockPrimal" showcase/
```

### Current Status (Dec 28, 2025)
✅ All mock directories removed
✅ All mock scripts removed
✅ Showcase is now mock-free
⚠️ **Showcase may not run yet - that's OK!**

**Next**: Deploy real primals and run showcase to find gaps!

---

## Philosophy

> "Showcase is our reality check. Mocks are comfortable lies. We choose uncomfortable truth."

**BiomeOS Showcase**: Live demonstrations, real gaps, honest progress. 🌱

---

**Policy Owner**: BiomeOS Core Team  
**Last Updated**: December 28, 2025  
**Status**: ENFORCED

