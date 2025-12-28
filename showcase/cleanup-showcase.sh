#!/usr/bin/env bash
# Cleanup and reorganize showcase documentation

set -e
cd "$(dirname "$0")"

echo "🧹 BiomeOS Showcase Cleanup"
echo "================================"
echo ""

# Create archive structure
echo "📁 Creating archive directories..."
mkdir -p archive/2025-12-{24,25,26,27,28}
mkdir -p archive/session-reports
mkdir -p archive/gap-reports
mkdir -p archive/old-plans

# Archive dated docs by day
echo "📦 Archiving dated documentation..."
for date in 24 25 26 27 28; do
  count=$(find . -maxdepth 1 -name "*DEC_${date}_2025.md" 2>/dev/null | wc -l)
  if [ "$count" -gt 0 ]; then
    find . -maxdepth 1 -name "*DEC_${date}_2025.md" -exec mv {} "archive/2025-12-${date}/" \;
    echo "  ✓ Moved $count files to archive/2025-12-${date}/"
  fi
done

# Archive session reports
echo "📦 Archiving session reports..."
find . -maxdepth 1 -name "SESSION_*.md" -exec mv {} archive/session-reports/ \; 2>/dev/null || true
find . -maxdepth 1 -name "EXECUTION_*.md" -exec mv {} archive/session-reports/ \; 2>/dev/null || true
find . -maxdepth 1 -name "VERIFICATION_*.md" -exec mv {} archive/session-reports/ \; 2>/dev/null || true
find . -maxdepth 1 -name "IMPLEMENTATION_*.md" -exec mv {} archive/session-reports/ \; 2>/dev/null || true
find . -maxdepth 1 -name "PROGRESS_*.md" -exec mv {} archive/session-reports/ \; 2>/dev/null || true

# Archive completion reports
echo "📦 Archiving completion reports..."
find . -maxdepth 1 -name "COMPLETE_*.md" -exec mv {} archive/session-reports/ \; 2>/dev/null || true
find . -maxdepth 1 -name "FINAL_*.md" -exec mv {} archive/session-reports/ \; 2>/dev/null || true
find . -maxdepth 1 -name "SUCCESS_*.md" -exec mv {} archive/session-reports/ \; 2>/dev/null || true
find . -maxdepth 1 -name "*MIRACLE*.md" -exec mv {} archive/session-reports/ \; 2>/dev/null || true

# Archive gap reports
echo "📦 Archiving gap reports..."
find . -maxdepth 1 -name "GAPS_*.md" -exec mv {} archive/gap-reports/ \; 2>/dev/null || true
find . -maxdepth 1 -name "GAP_*.md" -exec mv {} archive/gap-reports/ \; 2>/dev/null || true

# Archive old plans
echo "📦 Archiving old buildout plans..."
find . -maxdepth 1 -name "*PLAN*.md" ! -name "*DEC_28_2025.md" -exec mv {} archive/old-plans/ \; 2>/dev/null || true
find . -maxdepth 1 -name "ACTION_PLAN.md" -exec mv {} archive/old-plans/ \; 2>/dev/null || true

# Archive phase reports
echo "📦 Archiving phase reports..."
find . -maxdepth 1 -name "PHASE*.md" -exec mv {} archive/session-reports/ \; 2>/dev/null || true

# Archive demo-specific old docs
echo "📦 Archiving old demo documentation..."
find . -maxdepth 1 -name "API_ADAPTER_*.md" -exec mv {} archive/session-reports/ \; 2>/dev/null || true
find . -maxdepth 1 -name "PRIMAL_ADAPTER_*.md" -exec mv {} archive/session-reports/ \; 2>/dev/null || true
find . -maxdepth 1 -name "P2P_*.md" -exec mv {} archive/session-reports/ \; 2>/dev/null || true

# Create archive index
echo "📝 Creating archive index..."
cat > archive/README.md << 'ARCHIVE_EOF'
# BiomeOS Showcase Archive

Historical documentation from showcase development phases.

## Directory Structure

### By Date (2025-12)
- **24/** - Initial showcase buildout
- **25/** - Christmas milestone, 100% test success
- **26/** - API adapter completion
- **27/** - P2P coordination implementation
- **28/** - Comprehensive audit and new buildout plan

### By Type
- **session-reports/** - Session summaries and progress reports
- **gap-reports/** - Gap analysis and discovery reports
- **old-plans/** - Previous buildout and action plans

## Key Milestones

### December 24
- Initial showcase framework established
- Primal adapter pattern implemented
- Local capability demos created

### December 25
- 100% test pass rate achieved
- "Christmas Miracle" - Songbird CLI integration success
- All Phase1 adapters functional

### December 26
- API adapter implementation complete
- Live demo reports generated
- Path fix for binary execution

### December 27
- P2P coordination complete
- Multi-tower federation demos
- Showcase evolution planning

### December 28
- NO MOCKS policy enforced
- Comprehensive code audit (A- grade, 92/100)
- New buildout plan with benchScale integration
- Runtime discovery patterns established

## Current Active Documentation

See parent directory for:
- `README.md` - Main showcase entry point
- `NO_MOCKS_POLICY.md` - Live-only enforcement policy
- `CLEANUP_AND_DISCOVERY_PLAN.md` - This cleanup initiative
- `SHOWCASE_BUILDOUT_PLAN_DEC_28_2025.md` - Current buildout plan
- `QUICK_ACTION_PLAN_DEC_28_2025.md` - Immediate actions

## Philosophy Evolution

The showcase has evolved from:
1. **Mock-based demos** → **Live primals only**
2. **Hardcoded endpoints** → **Runtime discovery**
3. **Dev-only demos** → **benchScale validated deployments**
4. **Primal-specific code** → **Capability-based orchestration**

This archive preserves the journey while keeping active docs clean and focused.

---

*Archive created: December 28, 2025*
ARCHIVE_EOF

# Count archived files
echo ""
echo "✅ Cleanup Complete!"
echo "================================"
echo ""
echo "📊 Summary:"
echo "  Active docs in root: $(find . -maxdepth 1 -name "*.md" | wc -l) files"
echo "  Archived files: $(find archive -name "*.md" | wc -l) files"
echo ""
echo "📁 Active documentation:"
find . -maxdepth 1 -name "*.md" -exec basename {} \; | sort | sed 's/^/  ✓ /'
echo ""
echo "🗂️  Archive structure:"
echo "  ✓ archive/2025-12-24/ ($(find archive/2025-12-24 -name "*.md" 2>/dev/null | wc -l) files)"
echo "  ✓ archive/2025-12-25/ ($(find archive/2025-12-25 -name "*.md" 2>/dev/null | wc -l) files)"
echo "  ✓ archive/2025-12-26/ ($(find archive/2025-12-26 -name "*.md" 2>/dev/null | wc -l) files)"
echo "  ✓ archive/2025-12-27/ ($(find archive/2025-12-27 -name "*.md" 2>/dev/null | wc -l) files)"
echo "  ✓ archive/2025-12-28/ ($(find archive/2025-12-28 -name "*.md" 2>/dev/null | wc -l) files)"
echo "  ✓ archive/session-reports/ ($(find archive/session-reports -name "*.md" 2>/dev/null | wc -l) files)"
echo "  ✓ archive/gap-reports/ ($(find archive/gap-reports -name "*.md" 2>/dev/null | wc -l) files)"
echo "  ✓ archive/old-plans/ ($(find archive/old-plans -name "*.md" 2>/dev/null | wc -l) files)"
echo ""
echo "📖 Read archive/README.md for historical context"
echo ""
echo "🚀 Next steps:"
echo "  1. Review active docs in root"
echo "  2. Create RUNTIME_DISCOVERY.md"
echo "  3. Update main README.md"
echo "  4. Create common/discovery.sh"
echo ""

