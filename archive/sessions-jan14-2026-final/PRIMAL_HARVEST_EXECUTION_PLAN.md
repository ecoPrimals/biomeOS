# 🌾 Primal Harvest Execution Plan - January 14, 2026

**Date**: January 14, 2026  
**Goal**: Harvest Toadstool, NestGate, and Squirrel binaries for NUCLEUS LiveSpore  
**Status**: 🎯 **READY TO EXECUTE**

---

## 📋 **Primals to Harvest**

### **1. Squirrel** (AI/ML Coordinator)
- **Location**: `ecoPrimals/phase1/squirrel/`
- **Status**: ✅ Local, up-to-date
- **Action**: Rebuild and harvest
- **Priority**: 1 (ready immediately!)

### **2. NestGate** (Storage)
- **Location**: `ecoPrimals/phase1/nestgate/`
- **Status**: 🔄 Needs pull from remote
- **Action**: Pull, review, rebuild, harvest
- **Priority**: 2

### **3. Toadstool** (Compute + GPU)
- **Location**: `ecoPrimals/phase1/toadstool/`
- **Status**: 🔄 Needs pull from remote
- **Action**: Pull, review, rebuild, harvest
- **Priority**: 3

---

## 🚀 **Execution Sequence**

### **Phase 1: Harvest Squirrel** (Immediate - 5 min)

**Location**: `ecoPrimals/phase1/squirrel/`

```bash
# 1. Navigate to squirrel
cd /home/eastgate/Development/ecoPrimals/phase1/squirrel

# 2. Check current status
git status
cargo --version

# 3. Clean build
cargo clean
cargo build --release

# 4. Verify binary
ls -lh target/release/squirrel
./target/release/squirrel --version
./target/release/squirrel --capability  # Should show AI capabilities

# 5. Harvest to plasmidBin
cp target/release/squirrel \
   /home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/primals/squirrel

chmod +x /home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/primals/squirrel

# 6. Verify harvest
ls -lh /home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/primals/squirrel
/home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/primals/squirrel --version

# ✅ Squirrel harvested!
```

**Success Criteria**:
- ✅ Binary builds without errors
- ✅ `--version` shows version number
- ✅ `--capability` shows AI capabilities JSON
- ✅ Binary copied to `plasmidBin/primals/`
- ✅ Binary is executable

---

### **Phase 2: Update & Harvest NestGate** (15 min)

**Location**: `ecoPrimals/phase1/nestgate/`

```bash
# 1. Navigate to nestgate
cd /home/eastgate/Development/ecoPrimals/phase1/nestgate

# 2. Check current status
git status
git log --oneline -5  # See recent local commits

# 3. Pull latest changes
git fetch origin
git log HEAD..origin/master --oneline  # Preview incoming changes

# REVIEW: Check what's changed
git diff HEAD..origin/master

# 4. Pull if safe
git pull origin master

# 5. Review changes
# Read CHANGELOG.md or recent commits to understand updates
cat CHANGELOG.md 2>/dev/null || git log -5 --pretty=format:"%h - %s (%ar)" --graph

# 6. Check dependencies
cargo check

# 7. Run tests (if available)
cargo test --release 2>&1 | tail -20

# 8. Clean build
cargo clean
cargo build --release

# 9. Verify binary
ls -lh target/release/nestgate
./target/release/nestgate --version || ./target/release/nestgate --help

# 10. Harvest to plasmidBin
cp target/release/nestgate \
   /home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/primals/nestgate

chmod +x /home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/primals/nestgate

# 11. Verify harvest
ls -lh /home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/primals/nestgate

# ✅ NestGate harvested!
```

**Review Checklist**:
- [ ] Check for breaking API changes
- [ ] Review new capabilities added
- [ ] Check for configuration changes
- [ ] Verify compatibility with Songbird discovery
- [ ] Test basic functionality

---

### **Phase 3: Update & Harvest Toadstool** (15 min)

**Location**: `ecoPrimals/phase1/toadstool/`

```bash
# 1. Navigate to toadstool
cd /home/eastgate/Development/ecoPrimals/phase1/toadstool

# 2. Check current status
git status
git log --oneline -5  # See recent local commits

# 3. Pull latest changes
git fetch origin
git log HEAD..origin/master --oneline  # Preview incoming changes

# REVIEW: Check what's changed (especially barraCUDA updates!)
git diff HEAD..origin/master

# 4. Pull if safe
git pull origin master

# 5. Review changes
# This is critical - check for barraCUDA Rust CUDA implementation updates!
cat CHANGELOG.md 2>/dev/null || git log -5 --pretty=format:"%h - %s (%ar)" --graph

# 6. Check dependencies
cargo check

# 7. Run tests (if available)
cargo test --release 2>&1 | tail -20

# 8. Clean build
cargo clean
cargo build --release

# 9. Verify binary
ls -lh target/release/toadstool
./target/release/toadstool --version || ./target/release/toadstool --help

# 10. Harvest to plasmidBin
cp target/release/toadstool \
   /home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/primals/toadstool

chmod +x /home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/primals/toadstool

# 11. Verify harvest
ls -lh /home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/primals/toadstool

# ✅ Toadstool harvested!
```

**Review Checklist** (Critical for Toadstool!):
- [ ] Check barraCUDA Rust CUDA implementation status
- [ ] Review GPU backend changes (CUDA, ROCm, OpenCL, Vulkan, WebGPU)
- [ ] Check for 3D rendering API additions (for petalTongue!)
- [ ] Verify compute.gpu capability
- [ ] Test basic GPU detection

---

### **Phase 4: Verify All Harvests** (5 min)

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# List all harvested primals
ls -lh plasmidBin/primals/

# Should see:
# - beardog (✅ already harvested)
# - songbird (✅ already harvested)
# - petal-tongue (✅ already harvested)
# - petal-tongue-headless (✅ already harvested)
# - squirrel (🔄 just harvested)
# - nestgate (🔄 just harvested)
# - toadstool (🔄 just harvested)

# Verify all are executable
ls -la plasmidBin/primals/ | grep -E "^-rwx"

# Test each binary
echo "=== Testing Harvested Binaries ==="
for primal in squirrel nestgate toadstool; do
    echo ""
    echo "Testing: $primal"
    ./plasmidBin/primals/$primal --version 2>/dev/null || \
    ./plasmidBin/primals/$primal --help 2>/dev/null | head -5 || \
    echo "  ⚠️  No --version or --help flag"
done

# Update plasmidBin MANIFEST
cat > plasmidBin/HARVEST_STATUS.md << 'EOF'
# Primal Harvest Status - January 14, 2026

## ✅ Harvested Binaries (6 Total)

| Primal | Version | Size | Status | Date |
|--------|---------|------|--------|------|
| BearDog | v0.9.0 | ~35MB | ✅ Harvested | Jan 14 |
| Songbird | v3.22.0 | ~28MB | ✅ Harvested | Jan 14 |
| petalTongue | v0.5.0 | ~35MB | ✅ Harvested | Jan 13 |
| petalTongue-headless | v0.5.0 | ~3.2MB | ✅ Harvested | Jan 13 |
| Squirrel | TBD | TBD | ✅ Harvested | Jan 14 |
| NestGate | TBD | TBD | ✅ Harvested | Jan 14 |
| Toadstool | TBD | TBD | ✅ Harvested | Jan 14 |

## 🎯 Ready for NUCLEUS Deployment

All 6 core primals harvested and ready for:
- NUCLEUS LiveSpore deployment
- Local NUCLEUS testing
- Full ecosystem visualization with petalTongue
- AI/agentic capabilities with Squirrel
- GPU/3D rendering with Toadstool

**Next**: Deploy full NUCLEUS using `graphs/nucleus_deploy.toml`
EOF

cat plasmidBin/HARVEST_STATUS.md

# ✅ All primals harvested and verified!
```

---

## 📊 **Post-Harvest Actions**

### **1. Update Documentation**

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# Update plasmidBin/MANIFEST.md with new harvest info
# Update NUCLEUS_LIVESPORE_DEPLOYMENT_PLAN.md to mark harvests complete
# Update STATUS.md with harvest completion
```

### **2. Test NUCLEUS Deployment**

```bash
# Deploy full NUCLEUS locally to test
cargo run -p biomeos-atomic-deploy -- \
    --graph graphs/nucleus_deploy.toml \
    --family-id nat0 \
    --log-level debug

# Expected: All 6 primals start successfully
```

### **3. Create LiveSpore USB**

```bash
# Once NUCLEUS tests pass, create USB image
./scripts/prepare-nucleus-livespore.sh
```

---

## ⚠️ **Important Notes**

### **For NestGate & Toadstool Pulls**:

1. **Review Changes Carefully**:
   - Check `git diff` output before pulling
   - Look for API changes that might affect biomeOS integration
   - Check for new capabilities or configuration requirements

2. **Breaking Changes**:
   - If you see breaking API changes, document them
   - May need to update biomeOS client code
   - Test thoroughly before harvesting

3. **Toadstool barraCUDA**:
   - This is the Rust CUDA implementation
   - Check for major GPU backend updates
   - Verify 3D rendering API is stable
   - May have new capabilities for petalTongue!

### **Compatibility Checks**:

After harvesting, verify each primal:
- ✅ Responds to `--version` (discovery)
- ✅ Responds to `--capability` (for Squirrel)
- ✅ Works with Songbird discovery
- ✅ Accepts family-id parameter
- ✅ Creates proper Unix socket

---

## 🎯 **Success Criteria**

### **Phase 1 Complete**:
- ✅ Squirrel binary in `plasmidBin/primals/`
- ✅ Shows version and capabilities

### **Phase 2 Complete**:
- ✅ NestGate changes reviewed
- ✅ NestGate binary in `plasmidBin/primals/`
- ✅ No breaking changes identified

### **Phase 3 Complete**:
- ✅ Toadstool changes reviewed (especially barraCUDA!)
- ✅ Toadstool binary in `plasmidBin/primals/`
- ✅ GPU capabilities verified

### **All Phases Complete**:
- ✅ 7 binaries in `plasmidBin/primals/` (6 primals + headless UI)
- ✅ All executable
- ✅ MANIFEST.md updated
- ✅ Ready for NUCLEUS deployment!

---

## 🚀 **Ready to Execute?**

**Estimated Time**: 35-40 minutes total
- Squirrel: 5 min
- NestGate: 15 min (including review)
- Toadstool: 15 min (including review)
- Verification: 5 min

**Start with**: Phase 1 (Squirrel) - it's already local and ready!

---

**Created**: January 14, 2026  
**Status**: 🎯 READY TO EXECUTE  
**Next**: Start with Squirrel harvest!

**"From source to binary, the harvest begins!"** 🌾✨

