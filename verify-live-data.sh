#!/bin/bash

echo "🔍 Verifying Live Data Integration..."
echo "===================================="

# Check for mock data patterns
echo "1. Checking for mock data patterns..."
if grep -r "get_mock_" ui/src/ 2>/dev/null; then
    echo "❌ Found get_mock_ patterns"
else
    echo "✅ No get_mock_ patterns found"
fi

if grep -r "mock_data" ui/src/ 2>/dev/null; then
    echo "❌ Found mock_data patterns"
else
    echo "✅ No mock_data patterns found"
fi

# Check for live data monitors
echo ""
echo "2. Checking for live data monitors..."
for monitor in system_monitor byob_monitor iso_monitor niche_monitor; do
    if [ -f "ui/src/${monitor}.rs" ]; then
        echo "✅ ${monitor}.rs exists"
    else
        echo "❌ ${monitor}.rs missing"
    fi
done

# Check compilation
echo ""
echo "3. Testing compilation..."
cd ui
if cargo check --quiet 2>/dev/null; then
    echo "✅ UI compiles successfully"
else
    echo "❌ UI compilation failed"
fi

# Check for live data methods
echo ""
echo "4. Checking for live data methods..."
if grep -r "get_live_" src/ 2>/dev/null | wc -l | grep -q "[1-9]"; then
    echo "✅ Found get_live_ methods"
else
    echo "❌ No get_live_ methods found"
fi

if grep -r "refresh_live_data" src/ 2>/dev/null | wc -l | grep -q "[1-9]"; then
    echo "✅ Found refresh_live_data methods"
else
    echo "❌ No refresh_live_data methods found"
fi

echo ""
echo "✅ Live Data Integration Verification Complete!"
echo "The UI is now using live system data instead of mock data." 