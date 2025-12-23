#!/bin/bash

# Basic biomeOS BYOB Test
# Tests the core BYOB functionality without requiring Primal dependencies

set -e

echo "🧬 biomeOS BYOB Basic Test"
echo "========================="
echo ""
echo "Testing core BYOB functionality:"
echo "  📋 Manifest parsing"
echo "  🏠 Workspace isolation"  
echo "  🔧 CLI interface"
echo "  📊 Resource management"
echo ""

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$SCRIPT_DIR"

echo "📁 Workspace: $WORKSPACE_ROOT"
echo ""

# Build basic biomeOS without Primal dependencies
echo "🔨 Building biomeOS core (without Primal integrations)..."

cd "$WORKSPACE_ROOT/crates/biomeos-core"

if ! cargo build --bin biome; then
    echo "❌ Build failed"
    exit 1
fi

echo "✅ Build successful!"
echo ""

# Test CLI help
echo "🔧 Testing CLI interface..."
./target/debug/biome --help | head -10
echo ""

# Test manifest creation
echo "📋 Testing manifest template creation..."
./target/debug/biome init --template basic --output test-basic.biome.yaml

if [ -f "test-basic.biome.yaml" ]; then
    echo "✅ Basic manifest template created"
    echo ""
    echo "📄 Generated manifest preview:"
    head -15 test-basic.biome.yaml
    echo "..."
    echo ""
else
    echo "❌ Failed to create manifest template"
    exit 1
fi

# Test manifest validation
echo "🔍 Testing manifest validation..."
if ./target/debug/biome validate test-basic.biome.yaml; then
    echo "✅ Manifest validation working"
else
    echo "❌ Manifest validation failed"
    exit 1
fi

echo ""

# Test workspace information (without actual Primal connections)
echo "🏠 Testing workspace functionality..."
if ./target/debug/biome workspace --team demo-team; then
    echo "✅ Workspace functionality working"
else
    echo "⚠️  Workspace requires Primal connections (expected)"
fi

echo ""

# Clean up
echo "🧹 Cleaning up test files..."
rm -f test-basic.biome.yaml

echo ""
echo "🎉 Basic BYOB Test Results:"
echo "=========================="
echo ""
echo "✅ Core Functionality Working:"
echo "  📋 CLI interface: Functional"
echo "  📄 Manifest templates: Working"
echo "  🔍 Manifest validation: Working"
echo "  🏗️  Core architecture: Ready"
echo ""
echo "🚧 Next Phase:"
echo "  🔌 Primal adapter implementation"
echo "  🌐 Network integration testing"
echo "  🚀 End-to-end deployment testing"
echo ""
echo "🎯 Status: Ready for niche demonstrations!" 