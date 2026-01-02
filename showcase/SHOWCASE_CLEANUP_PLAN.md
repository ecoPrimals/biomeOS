# 🧹 Showcase Cleanup & Reorganization Plan

**Date**: December 31, 2025  
**Goal**: Clean up overlaps, outdated assumptions, and reorganize for BTSP/P2P focus  
**Status**: Analysis → Cleanup → Rebuild  

---

## 🔍 **CURRENT STATE ANALYSIS**

### Directory Structure (21 categories - TOO MANY!)
```
00-local-capabilities/          # BiomeOS local features
00-substrate/                   # OVERLAP with 00-local-capabilities
01-nestgate/                    # NestGate showcase (planned, empty?)
01-single-primal/              # Individual primal demos
02-birdsong-p2p/               # P2P primitives
02-primal-pairs/               # OVERLAP with 02-birdsong-p2p?
03-chimera-patterns/           # Chimera patterns
03-full-ecosystem/             # Full ecosystem
03-multiplex/                  # Multiplex patterns
03-p2p-coordination/           # BTSP & BirdSong (our focus)
03-primal-adapter/             # Adapter patterns
03-primal-triples/             # Triple patterns
04-complete-ecosystem/         # OVERLAP with 03-full-ecosystem?
04-deployment-evolution/       # Deployment patterns
04-multi-primal-adaptation/    # OVERLAP with adapters?
05-chimera-patterns/           # DUPLICATE of 03-chimera-patterns!
05-lifecycle-negotiation/      # Lifecycle patterns
06-multiplex-patterns/         # DUPLICATE of 03-multiplex!
```

### Overlaps Identified

#### 1. **Substrate/Local Capabilities (2 directories)**
- `00-substrate/` - BiomeOS substrate demos
- `00-local-capabilities/` - BiomeOS local features
- **OVERLAP**: Both show BiomeOS without primals
- **Issue**: Confusing which to use

#### 2. **P2P & Coordination (2 directories)**
- `02-birdsong-p2p/` - P2P primitives
- `03-p2p-coordination/` - BTSP & BirdSong
- **OVERLAP**: Both about P2P
- **Decision**: Keep 03-p2p-coordination (more comprehensive)

#### 3. **Chimera Patterns (2 directories)**
- `03-chimera-patterns/`
- `05-chimera-patterns/`
- **OVERLAP**: DUPLICATE!
- **Decision**: Merge into one

#### 4. **Multiplex (2 directories)**
- `03-multiplex/`
- `06-multiplex-patterns/`
- **OVERLAP**: DUPLICATE!
- **Decision**: Merge into one

#### 5. **Ecosystem (2 directories)**
- `03-full-ecosystem/`
- `04-complete-ecosystem/`
- **OVERLAP**: Both show all primals
- **Decision**: Merge into one

#### 6. **Adapters (2 directories)**
- `03-primal-adapter/`
- `04-multi-primal-adaptation/`
- **OVERLAP**: Both about adaptation patterns
- **Decision**: Merge into one

### Outdated Assumptions Found

#### 1. **Phase1 Bins Path** (`../../primalBins/`)
- **Found in**: 8+ files
- **Issue**: Path doesn't exist, should be `../../primalBins/`
- **Fix**: Update all references

#### 2. **Hardcoded Ports**
- **Found in**: 30+ shell scripts
- **Issue**: Violates our "no hardcoding" principle
- **Fix**: Use discovery instead

#### 3. **Mock References**
- **Found in**: 12+ markdown files
- **Issue**: Confusing after "NO MOCKS" policy
- **Fix**: Remove or clarify historical context

#### 4. **Outdated Primal Names**
- **Found**: References to primals that may not exist
- **Issue**: Confusion about what's available
- **Fix**: Verify against `../../primalBins/`

---

## 🎯 **PROPOSED STRUCTURE** (Clean & Clear)

### Reorganized Showcase (10 categories max)

```
showcase/
├── common/                          # NEW - Shared utilities
│   ├── discovery.sh                 # Runtime discovery library
│   ├── health-check.sh              # Health monitoring
│   └── start-primal.sh              # Primal startup helper
│
├── 00-substrate/                    # MERGED from 00-local-capabilities
│   ├── 01-manifest-parsing/
│   ├── 02-capability-matching/
│   ├── 03-sovereignty-guardian/
│   ├── 04-client-registry/
│   └── 05-configuration/
│
├── 01-single-primal/                # KEEP - Individual primal demos
│   ├── nestgate/
│   ├── beardog/
│   ├── songbird/
│   ├── toadstool/
│   └── squirrel/
│
├── 02-primal-pairs/                 # KEEP - Two primal coordination
│   ├── nestgate-beardog/           # Encrypted storage
│   ├── songbird-beardog/           # BTSP foundation
│   ├── songbird-toadstool/         # Distributed compute
│   └── nestgate-toadstool/         # ML pipelines
│
├── 03-p2p-coordination/             # KEEP - BTSP & BirdSong FOCUS 🔥
│   ├── 01-btsp-tunnel-coordination/
│   ├── 02-birdsong-encryption/
│   ├── 03-lineage-gated-relay/
│   ├── 04-multi-tower-federation/
│   └── 05-full-ecosystem-integration/
│
├── 04-primal-adapter/               # MERGED from adapter dirs
│   ├── 01-universal-adapter/
│   ├── 02-multi-primal-adaptation/
│   └── 03-api-compatibility/
│
├── 05-chimera-patterns/             # MERGED from duplicates
│   ├── 01-loamspine-embed/
│   ├── 02-rhizocrypt-embed/
│   └── 03-performance-comparison/
│
├── 06-multiplex-patterns/           # MERGED from duplicates
│   ├── 01-multi-songbird/
│   ├── 02-storage-farm/
│   └── 03-compute-cluster/
│
├── 07-full-ecosystem/               # MERGED ecosystem dirs
│   ├── 01-five-primal-workflow/
│   ├── 02-ml-pipeline-complete/
│   └── 03-production-patterns/
│
├── 08-deployment/                   # KEEP - Deployment patterns
│   ├── 01-benchscale-validation/
│   ├── 02-multi-machine/
│   └── 03-production-deployment/
│
└── archive/                         # MOVE old/outdated demos here
    ├── 02-birdsong-p2p/            # Superseded by 03-p2p-coordination
    ├── 03-primal-triples/          # Unclear use case
    ├── 05-lifecycle-negotiation/   # Experimental
    └── outdated-docs/              # Old READMEs, etc.
```

**Total**: 10 categories (down from 21) ✅

---

## 🔧 **CLEANUP ACTIONS**

### Phase 1: Archive Redundant (1 hour)

#### Move to `archive/`
```bash
mv 00-local-capabilities archive/
# (We'll use 00-substrate as the merged version)

mv 02-birdsong-p2p archive/
# (Superseded by 03-p2p-coordination)

mv 03-full-ecosystem archive/
# (Will merge with 04-complete-ecosystem)

mv 03-primal-triples archive/
# (Unclear use case, can restore if needed)

mv 05-lifecycle-negotiation archive/
# (Experimental, not ready)
```

#### Merge Duplicates
```bash
# Merge chimera patterns
mv 05-chimera-patterns/* 03-chimera-patterns/ 2>/dev/null
rmdir 05-chimera-patterns

# Merge multiplex
mv 03-multiplex/* 06-multiplex-patterns/ 2>/dev/null
rmdir 03-multiplex

# Merge ecosystem
mv 03-full-ecosystem/* 04-complete-ecosystem/ 2>/dev/null

# Merge adapters
mv 03-primal-adapter/* 04-multi-primal-adaptation/ 2>/dev/null
```

### Phase 2: Update References (30 min)

#### Fix Path References
```bash
# Find and replace primalBins → primalBins
find . -type f \( -name "*.sh" -o -name "*.md" \) -exec sed -i 's|primalBins|primalBins|g' {} +
find . -type f \( -name "*.sh" -o -name "*.md" \) -exec sed -i 's|primalBins|primalBins|g' {} +
```

#### Remove Hardcoded Ports
```bash
# Audit files with hardcoded ports
grep -r "localhost:[0-9]" --include="*.sh" > hardcoded_ports.txt

# Manual review and update to use discovery
# (Each file needs individual attention)
```

#### Clarify Mock References
```bash
# Find mock references
grep -ri "mock" --include="*.md" > mock_references.txt

# Add clarification notes where needed
```

### Phase 3: Rename & Reorganize (1 hour)

#### Rename Directories for Clarity
```bash
# Rename to match new structure
mv 00-substrate 00-substrate-capabilities
mv 04-complete-ecosystem 07-full-ecosystem
mv 04-multi-primal-adaptation 04-primal-adapter
mv 04-deployment-evolution 08-deployment
mv 03-chimera-patterns 05-chimera-patterns
mv 06-multiplex-patterns 06-multiplex
```

#### Create Missing Structure
```bash
# Create common utilities directory
mkdir -p common

# Create archive for old content
mkdir -p archive/outdated-docs

# Ensure all numbered dirs follow convention
```

### Phase 4: Create Common Library (2 hours)

#### `common/discovery.sh`
```bash
#!/usr/bin/env bash
# BiomeOS Runtime Discovery Library
# NO HARDCODED ENDPOINTS - EVER

# Discover available binaries
discover_primal_bin() {
    local primal=$1
    
    # Check primalBins first
    if [ -f "../../primalBins/$primal" ]; then
        echo "../../primalBins/$primal"
        return 0
    fi
    
    # Check PATH
    which "$primal" 2>/dev/null
}

# Discover running primal by capability
discover_by_capability() {
    local capability=$1
    # Use mDNS, capability queries, etc.
    # NO HARDCODED PORTS
}

# Health check
check_primal_health() {
    local primal=$1
    local endpoint=$2
    curl -sf "$endpoint/health" >/dev/null
}

# Wait for primal to be ready
wait_for_primal() {
    local name=$1
    local endpoint=$2
    local max_wait=30
    
    for i in $(seq 1 $max_wait); do
        if check_primal_health "$name" "$endpoint"; then
            return 0
        fi
        sleep 1
    done
    return 1
}
```

#### `common/start-primal.sh`
```bash
#!/usr/bin/env bash
# Primal startup helper with health monitoring

source "$(dirname "$0")/discovery.sh"

start_primal() {
    local primal=$1
    local config=${2:-""}
    
    local bin=$(discover_primal_bin "$primal")
    if [ -z "$bin" ]; then
        echo "❌ $primal binary not found"
        return 1
    fi
    
    echo "🚀 Starting $primal..."
    if [ -n "$config" ]; then
        "$bin" --config "$config" &
    else
        "$bin" &
    fi
    
    local pid=$!
    echo "$pid" > "/tmp/$primal.pid"
    
    # Wait for health check
    # (need to discover endpoint first)
    
    echo "✅ $primal started (PID: $pid)"
}
```

### Phase 5: Update Documentation (1 hour)

#### Update Main README
```markdown
# BiomeOS Showcase - Clean & Focused

**Status**: Reorganized December 31, 2025  
**Categories**: 10 (down from 21)  
**Focus**: BTSP & BirdSong P2P  

## Structure

- `00-substrate/` - BiomeOS without primals
- `01-single-primal/` - Individual primal demos
- `02-primal-pairs/` - Two-primal coordination
- **`03-p2p-coordination/`** - **BTSP & BirdSong FOCUS** 🔥
- `04-primal-adapter/` - Adaptation patterns
- `05-chimera-patterns/` - Chimera integration
- `06-multiplex/` - Multiplex patterns
- `07-full-ecosystem/` - Complete workflows
- `08-deployment/` - Production deployment
- `archive/` - Historical/experimental content
```

#### Create CHANGELOG
```markdown
# Showcase Reorganization - December 31, 2025

## Changes Made

### Archived (Moved to archive/)
- `00-local-capabilities/` → Merged with 00-substrate
- `02-birdsong-p2p/` → Superseded by 03-p2p-coordination
- `03-primal-triples/` → Unclear use case
- `05-lifecycle-negotiation/` → Experimental

### Merged Duplicates
- `03-chimera-patterns/` + `05-chimera-patterns/` → `05-chimera-patterns/`
- `03-multiplex/` + `06-multiplex-patterns/` → `06-multiplex/`
- `03-full-ecosystem/` + `04-complete-ecosystem/` → `07-full-ecosystem/`
- `03-primal-adapter/` + `04-multi-primal-adaptation/` → `04-primal-adapter/`

### Updated References
- `primalBins` → `primalBins` (everywhere)
- Added common/discovery.sh for runtime discovery
- Removed hardcoded ports where possible
- Clarified mock references

### New Structure
- 21 categories → 10 categories
- Clear numbering scheme
- Common utilities directory
- Archive for historical content
```

---

## 📊 **VALIDATION CHECKLIST**

### After Cleanup
- [ ] All duplicates removed or merged
- [ ] All outdated paths fixed (`primalBins` → `primalBins`)
- [ ] Common utilities created (`common/*.sh`)
- [ ] Documentation updated (README, CHANGELOG)
- [ ] Archive directory created and populated
- [ ] Numbering scheme consistent (00-08)
- [ ] Each demo has README
- [ ] No hardcoded ports in new demos
- [ ] All scripts executable (`chmod +x`)

### Test Plan
```bash
# 1. Test discovery library
source common/discovery.sh
discover_primal_bin "beardog"
discover_primal_bin "songbird"

# 2. Test a simple demo
cd 01-single-primal/nestgate/
./demo.sh

# 3. Verify paths
find . -type f -name "*.sh" -exec grep -l "primalBins" {} \;
# Should return: 0 files

# 4. Check for hardcoded ports
find . -type f -name "*.sh" -not -path "*/archive/*" -exec grep -l "localhost:[0-9]" {} \;
# Review and update as needed
```

---

## 🎯 **SUCCESS METRICS**

### Quantitative
- **Directories**: 21 → 10 (52% reduction) ✅
- **Duplicates**: 6 identified → 0 remaining ✅
- **Outdated paths**: 8+ → 0 ✅
- **Hardcoded ports**: 30+ → Target <5 ✅
- **Common utilities**: 0 → 3+ created ✅

### Qualitative
- ✅ Clear directory purpose (no overlap)
- ✅ Consistent naming scheme
- ✅ Updated documentation
- ✅ Runtime discovery library
- ✅ Ready for BTSP/P2P buildout

---

## ⏱️ **TIMELINE**

### Day 1 (Today) - Cleanup
- **Hours 1-2**: Archive redundant directories
- **Hours 3-4**: Merge duplicates
- **Hours 5-6**: Fix outdated references
- **Hours 7-8**: Create common utilities

### Day 2 - Validation
- **Hours 1-2**: Test all changes
- **Hours 3-4**: Update documentation
- **Hours 5-6**: Validate with real demos
- **Hours 7-8**: Final review & commit

### Day 3 - BTSP Buildout
- **Start fresh** with clean structure
- **Build** `03-p2p-coordination/01-btsp-tunnel-coordination/`
- **No confusion** from old overlapping demos

---

## 🚨 **RISKS & MITIGATION**

### Risk 1: Breaking Existing Demos
**Mitigation**: Test each category after changes

### Risk 2: Losing Important Content
**Mitigation**: Archive, don't delete. Can restore if needed.

### Risk 3: Path References Break
**Mitigation**: Comprehensive find/replace with validation

### Risk 4: Taking Too Long
**Mitigation**: Focus on phase 1-2 first (archive + merge)

---

## 📁 **ARCHIVE POLICY**

### What Goes to Archive
✅ Duplicate directories  
✅ Experimental/incomplete demos  
✅ Superseded content  
✅ Outdated documentation  

### What Stays Active
✅ Working demos with unique value  
✅ Current best practices  
✅ BTSP & P2P focus content  
✅ Well-documented examples  

### Archive Structure
```
archive/
├── duplicates/
│   ├── 00-local-capabilities/
│   ├── 03-full-ecosystem/
│   ├── 03-multiplex/
│   └── 03-primal-adapter/
├── experimental/
│   ├── 03-primal-triples/
│   └── 05-lifecycle-negotiation/
├── superseded/
│   └── 02-birdsong-p2p/
└── outdated-docs/
    ├── old-READMEs/
    └── deprecated-plans/
```

---

## ✅ **IMMEDIATE ACTIONS** (Priority Order)

### 1. Create Archive (15 min)
```bash
cd showcase
mkdir -p archive/{duplicates,experimental,superseded,outdated-docs}
```

### 2. Archive Duplicates (30 min)
```bash
mv 05-chimera-patterns archive/duplicates/
mv 03-multiplex archive/duplicates/
mv 03-full-ecosystem archive/duplicates/
mv 03-primal-adapter archive/duplicates/
```

### 3. Archive Superseded/Experimental (15 min)
```bash
mv 02-birdsong-p2p archive/superseded/
mv 03-primal-triples archive/experimental/
mv 05-lifecycle-negotiation archive/experimental/
```

### 4. Fix Path References (30 min)
```bash
find . -type f \( -name "*.sh" -o -name "*.md" \) \
  -not -path "*/archive/*" \
  -exec sed -i 's|primalBins|primalBins|g' {} +
```

### 5. Create Common Utilities (1 hour)
```bash
mkdir common
# Create discovery.sh, start-primal.sh, health-check.sh
```

### 6. Update Documentation (30 min)
```bash
# Update main README with new structure
# Create SHOWCASE_REORGANIZATION_CHANGELOG.md
# Update each category README
```

---

**Status**: ✅ Plan Complete - Ready to Execute  
**Estimated Time**: 8-10 hours total  
**Priority**: HIGH - Blocking BTSP/P2P buildout  
**Next Action**: Execute Phase 1 (Archive redundant directories)  

🧹 **Let's clean this up before building BTSP!**

