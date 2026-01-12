#!/bin/bash
# Final Root Cleanup - Aggressive consolidation
# Date: January 12, 2026

set -e

echo "🧹 Creating new archive directories..."
mkdir -p archive/docs-fossil-record/start-here-variants
mkdir -p archive/docs-fossil-record/petaltongue-integration
mkdir -p archive/docs-fossil-record/planning
mkdir -p archive/docs-fossil-record/achievements
mkdir -p archive/docs-fossil-record/investigations

echo "📦 Moving START_HERE variants..."
[ -f "START_HERE_JAN12_EVENING.md" ] && mv START_HERE_JAN12_EVENING.md archive/docs-fossil-record/start-here-variants/
[ -f "START_HERE_PETALTONGUE.md" ] && mv START_HERE_PETALTONGUE.md archive/docs-fossil-record/start-here-variants/

echo "📦 Moving petalTongue docs (keeping only PETALTONGUE_TUI_INTEGRATION.md)..."
[ -f "PETALTONGUE_DEPLOYMENT_GUIDE.md" ] && mv PETALTONGUE_DEPLOYMENT_GUIDE.md archive/docs-fossil-record/petaltongue-integration/
[ -f "PETALTONGUE_INTEGRATION_COMPLETE.md" ] && mv PETALTONGUE_INTEGRATION_COMPLETE.md archive/docs-fossil-record/petaltongue-integration/
[ -f "PETALTONGUE_SESSION_SUMMARY_JAN12.md" ] && mv PETALTONGUE_SESSION_SUMMARY_JAN12.md archive/docs-fossil-record/petaltongue-integration/
[ -f "PETALTONGUE_HARVEST_SUCCESS.md" ] && mv PETALTONGUE_HARVEST_SUCCESS.md archive/docs-fossil-record/petaltongue-integration/
[ -f "PETALTONGUE_JSONRPC_HANDOFF.md" ] && mv PETALTONGUE_JSONRPC_HANDOFF.md archive/docs-fossil-record/petaltongue-integration/
[ -f "PETALTONGUE_UI_ARCHITECTURE.md" ] && mv PETALTONGUE_UI_ARCHITECTURE.md archive/docs-fossil-record/petaltongue-integration/

echo "📦 Moving planning docs..."
[ -f "CLEANUP_PLAN_JAN12_EVENING.md" ] && mv CLEANUP_PLAN_JAN12_EVENING.md archive/docs-fossil-record/planning/
[ -f "RESPONSIBILITY_MATRIX_JAN12.md" ] && mv RESPONSIBILITY_MATRIX_JAN12.md archive/docs-fossil-record/planning/
[ -f "REFINED_ROADMAP.md" ] && mv REFINED_ROADMAP.md archive/docs-fossil-record/planning/
[ -f "FINAL_CLEANUP_PLAN.md" ] && mv FINAL_CLEANUP_PLAN.md archive/docs-fossil-record/planning/

echo "📦 Moving achievement docs..."
[ -f "TOWER_ATOMIC_SUCCESS_JAN12.md" ] && mv TOWER_ATOMIC_SUCCESS_JAN12.md archive/docs-fossil-record/achievements/
[ -f "NESTGATE_UNBLOCKED_JAN12.md" ] && mv NESTGATE_UNBLOCKED_JAN12.md archive/docs-fossil-record/achievements/
[ -f "QUICK_LINEAGE_REFERENCE.md" ] && mv QUICK_LINEAGE_REFERENCE.md archive/docs-fossil-record/achievements/

echo "📦 Moving investigation docs..."
[ -f "LIVESPORE_INVESTIGATION.md" ] && mv LIVESPORE_INVESTIGATION.md archive/docs-fossil-record/investigations/

echo "📦 Moving master index..."
[ -f "MASTER_DOCUMENTATION_INDEX.md" ] && mv MASTER_DOCUMENTATION_INDEX.md archive/docs-fossil-record/

echo "✅ Final cleanup complete!"
echo ""
echo "📊 Summary:"
echo "  • START_HERE variants: 2 → archive/docs-fossil-record/start-here-variants/"
echo "  • petalTongue docs: 6 → archive/docs-fossil-record/petaltongue-integration/"
echo "  • Planning docs: 4 → archive/docs-fossil-record/planning/"
echo "  • Achievements: 3 → archive/docs-fossil-record/achievements/"
echo "  • Investigations: 1 → archive/docs-fossil-record/investigations/"
echo "  • Master index: 1 → archive/docs-fossil-record/"
echo ""
echo "Root directory ultra-clean! Only ~14 essential docs remain."
