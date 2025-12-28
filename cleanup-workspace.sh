#!/usr/bin/env bash
# Comprehensive workspace cleanup for biomeOS
# Archives old docs, cleans build artifacts, prepares for commit

set -e
cd "$(dirname "$0")"

echo "🧹 BiomeOS Workspace Cleanup"
echo "========================================"
echo ""

# Create archive structure in parent
ARCHIVE_DIR="../archive/biomeOS-docs-dec28-2025"
echo "📁 Creating archive directory: $ARCHIVE_DIR"
mkdir -p "$ARCHIVE_DIR"/{root-docs,old-reports,session-summaries}

# Archive dated root documentation
echo "📦 Archiving root documentation..."
dated_docs=(
  "*DEC_27_2025.md"
  "*DEC_28_2025.md"
  "SESSION_*.md"
  "*COMPLETE*.md"
  "*CLEANUP*.md"
  "*WORKSPACE*.md"
  "*AUDIT*.md"
)

count=0
for pattern in "${dated_docs[@]}"; do
  # Use find to handle glob patterns safely
  while IFS= read -r file; do
    if [ -f "$file" ]; then
      mv "$file" "$ARCHIVE_DIR/root-docs/"
      echo "  ✓ Archived: $file"
      count=$((count + 1))
    fi
  done < <(find . -maxdepth 1 -name "$pattern" 2>/dev/null)
done

echo "  📊 Archived $count root documentation files"

# Keep only essential root docs
echo ""
echo "📋 Essential root docs retained:"
for essential in README.md START_HERE.md ROOT_INDEX.md QUICK_REFERENCE.md; do
  if [ -f "$essential" ]; then
    echo "  ✓ $essential"
  fi
done

# Clean build artifacts
echo ""
echo "🗑️  Cleaning build artifacts..."
if [ -d "target" ]; then
  target_size=$(du -sh target 2>/dev/null | cut -f1)
  echo "  Removing target/ ($target_size)..."
  rm -rf target
  echo "  ✓ Cleaned target/"
fi

# Clean Cargo.lock from subdirectories (keep workspace root one)
echo ""
echo "🔒 Cleaning redundant Cargo.lock files..."
lock_count=$(find . -name "Cargo.lock" -not -path "./Cargo.lock" | wc -l)
if [ "$lock_count" -gt 0 ]; then
  find . -name "Cargo.lock" -not -path "./Cargo.lock" -delete
  echo "  ✓ Removed $lock_count redundant Cargo.lock files"
fi

# Clean showcase archives to parent
echo ""
echo "📦 Moving showcase archive to parent..."
if [ -d "showcase/archive" ]; then
  mv showcase/archive "$ARCHIVE_DIR/showcase-archive"
  echo "  ✓ Moved showcase/archive to parent"
fi

# Archive old docs/reports
echo ""
echo "📦 Archiving old docs/reports..."
if [ -d "docs/reports" ]; then
  old_reports=$(find docs/reports -name "*DEC*.md" 2>/dev/null | wc -l)
  if [ "$old_reports" -gt 0 ]; then
    mkdir -p "$ARCHIVE_DIR/old-reports"
    find docs/reports -name "*DEC*.md" -exec mv {} "$ARCHIVE_DIR/old-reports/" \;
    echo "  ✓ Archived $old_reports report files"
  fi
fi

# Create archive index
echo ""
echo "📝 Creating archive index..."
cat > "$ARCHIVE_DIR/README.md" << 'ARCHIVE_EOF'
# BiomeOS Archive - December 28, 2025

Archived documentation and reports from biomeOS cleanup.

## Contents

### root-docs/
Root-level dated documentation:
- Audit reports (Dec 27-28)
- Session summaries
- Cleanup reports
- Workspace status reports

### showcase-archive/
Showcase development history:
- Session reports by date (2025-12-24 through 12-28)
- Gap reports and analysis
- Old buildout plans

### old-reports/
Historical doc reports from docs/reports/

## Context

This archive was created during the Dec 28, 2025 comprehensive cleanup:
- **Root docs**: Reduced from many dated files to essential docs only
- **Showcase**: Cleaned and organized with runtime discovery patterns
- **Build artifacts**: Removed to reduce workspace size

## Active Documentation

See biomeOS root for current documentation:
- `README.md` - Main entry point
- `START_HERE.md` - Getting started guide
- `ROOT_INDEX.md` - Complete navigation
- `COMPREHENSIVE_CODE_AUDIT_DEC_28_2025.md` - Latest audit (in archive)

## Philosophy Established

Dec 28, 2025 marked a shift to:
- **Runtime discovery** - Zero hardcoding
- **Live infrastructure** - NO MOCKS policy
- **benchScale validation** - Real deployments
- **Clean workspace** - Focused on active development

---

*Archive created: December 28, 2025*  
*Preserved for historical context and project evolution tracking*
ARCHIVE_EOF

echo "  ✓ Created archive index"

# Summary
echo ""
echo "✅ Cleanup Complete!"
echo "========================================"
echo ""
echo "📊 Summary:"
echo "  Root docs archived: $count files"
echo "  Build artifacts: Cleaned (target/)"
echo "  Cargo.lock files: Cleaned redundant"
echo "  Archive location: $ARCHIVE_DIR"
echo ""
echo "📁 Current workspace:"
ws_size=$(du -sh . 2>/dev/null | cut -f1)
echo "  Total size: $ws_size"
echo "  Essential docs: $(find . -maxdepth 1 -name "*.md" | wc -l) files"
echo "  Active showcase demos: $(find showcase -type d -name "*-*" | wc -l) directories"
echo ""
echo "🗂️  Archive structure:"
echo "  $ARCHIVE_DIR/root-docs/ ($count files)"
echo "  $ARCHIVE_DIR/showcase-archive/ (historical)"
echo "  $ARCHIVE_DIR/old-reports/ (reports)"
echo "  $ARCHIVE_DIR/README.md (index)"
echo ""
echo "📖 Read $ARCHIVE_DIR/README.md for historical context"
echo ""
echo "🚀 Ready for git commit!"
echo ""

