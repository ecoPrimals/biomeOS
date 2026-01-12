#!/bin/bash
# Cleanup Root Docs - Archive completed session & implementation docs
# Date: January 12, 2026

set -e

echo "🧹 Creating archive directories..."
mkdir -p archive/docs-fossil-record/jan12-session
mkdir -p archive/docs-fossil-record/implementations
mkdir -p archive/docs-fossil-record/handoffs

echo "📦 Moving Jan 12 session docs..."
[ -f "SESSION_COMPLETE_JAN12_AFTERNOON.md" ] && mv SESSION_COMPLETE_JAN12_AFTERNOON.md archive/docs-fossil-record/jan12-session/
[ -f "SESSION_FINAL_JAN12_2026.md" ] && mv SESSION_FINAL_JAN12_2026.md archive/docs-fossil-record/jan12-session/
[ -f "SESSION_FINAL_RUST_NEURAL_API.md" ] && mv SESSION_FINAL_RUST_NEURAL_API.md archive/docs-fossil-record/jan12-session/
[ -f "SESSION_SUMMARY_JAN12_ATOMIC_LIVESPORE.md" ] && mv SESSION_SUMMARY_JAN12_ATOMIC_LIVESPORE.md archive/docs-fossil-record/jan12-session/
[ -f "START_HERE_JAN12.md" ] && mv START_HERE_JAN12.md archive/docs-fossil-record/jan12-session/
[ -f "ROOT_DOCS_UPDATED_JAN12.md" ] && mv ROOT_DOCS_UPDATED_JAN12.md archive/docs-fossil-record/jan12-session/
[ -f "ATOMIC_DEPLOYMENT_PROGRESS_JAN12.md" ] && mv ATOMIC_DEPLOYMENT_PROGRESS_JAN12.md archive/docs-fossil-record/jan12-session/
[ -f "QUICK_STATUS_JAN12.md" ] && mv QUICK_STATUS_JAN12.md archive/docs-fossil-record/jan12-session/

echo "📦 Moving implementation complete docs..."
[ -f "TESTING_SUITE_COMPLETE.md" ] && mv TESTING_SUITE_COMPLETE.md archive/docs-fossil-record/implementations/
[ -f "POLISHED_TESTED_COMPLETE.md" ] && mv POLISHED_TESTED_COMPLETE.md archive/docs-fossil-record/implementations/
[ -f "NEURAL_API_EXECUTOR_COMPLETE.md" ] && mv NEURAL_API_EXECUTOR_COMPLETE.md archive/docs-fossil-record/implementations/
[ -f "RUST_EVOLUTION_COMPLETE.md" ] && mv RUST_EVOLUTION_COMPLETE.md archive/docs-fossil-record/implementations/
[ -f "GENETIC_LINEAGE_IMPLEMENTATION_COMPLETE.md" ] && mv GENETIC_LINEAGE_IMPLEMENTATION_COMPLETE.md archive/docs-fossil-record/implementations/
[ -f "GENETIC_LINEAGE_TEST_REPORT.md" ] && mv GENETIC_LINEAGE_TEST_REPORT.md archive/docs-fossil-record/implementations/
[ -f "PURE_RUST_EVOLUTION_COMPLETE.md" ] && mv PURE_RUST_EVOLUTION_COMPLETE.md archive/docs-fossil-record/implementations/
[ -f "LIVESPORE_PHASE1_COMPLETE.md" ] && mv LIVESPORE_PHASE1_COMPLETE.md archive/docs-fossil-record/implementations/
[ -f "NUCLEUS_EVOLUTION_COMPLETE.md" ] && mv NUCLEUS_EVOLUTION_COMPLETE.md archive/docs-fossil-record/implementations/
[ -f "UI_PHASES_4_5_6_COMPLETE.md" ] && mv UI_PHASES_4_5_6_COMPLETE.md archive/docs-fossil-record/implementations/
[ -f "NEURAL_API_ATOMIC_INTEGRATION_COMPLETE.md" ] && mv NEURAL_API_ATOMIC_INTEGRATION_COMPLETE.md archive/docs-fossil-record/implementations/

echo "📦 Moving handoff docs..."
[ -f "FINAL_HANDOFF_JAN11.md" ] && mv FINAL_HANDOFF_JAN11.md archive/docs-fossil-record/handoffs/
[ -f "PRODUCTION_HANDOFF_JAN11_2026.md" ] && mv PRODUCTION_HANDOFF_JAN11_2026.md archive/docs-fossil-record/handoffs/
[ -f "COLLABORATIVE_INTELLIGENCE_HANDOFF.md" ] && mv COLLABORATIVE_INTELLIGENCE_HANDOFF.md archive/docs-fossil-record/handoffs/
[ -f "PRIMAL_SOCKET_CONFIG_HANDOFF.md" ] && mv PRIMAL_SOCKET_CONFIG_HANDOFF.md archive/docs-fossil-record/handoffs/
[ -f "DEEP_DEBT_AUDIT_JAN11_2026.md" ] && mv DEEP_DEBT_AUDIT_JAN11_2026.md archive/docs-fossil-record/handoffs/
[ -f "DEPLOYMENT_TESTING_SUMMARY_JAN11.md" ] && mv DEPLOYMENT_TESTING_SUMMARY_JAN11.md archive/docs-fossil-record/handoffs/
[ -f "COLLABORATIVE_INTELLIGENCE_BIOMEOS_TRACKER.md" ] && mv COLLABORATIVE_INTELLIGENCE_BIOMEOS_TRACKER.md archive/docs-fossil-record/handoffs/

echo "✅ Cleanup complete!"
echo ""
echo "📊 Summary:"
echo "  • Jan 12 session docs: archive/docs-fossil-record/jan12-session/"
echo "  • Implementation docs: archive/docs-fossil-record/implementations/"
echo "  • Handoff docs: archive/docs-fossil-record/handoffs/"
echo ""
echo "Root directory cleaned! All docs preserved as fossil record."
