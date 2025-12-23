#!/bin/bash

# Quick BiomeOS Enhanced UI Demo
# Showcases all the major features we've built

echo "🌿 BiomeOS Enhanced UI - Quick Demo"
echo "===================================="
echo ""

echo "🎯 Available Demo Modes:"
echo "1. BYOB (Build Your Own Biome) - Team Management"
echo "2. ISO Creator - Custom Distribution Builder" 
echo "3. Niche Manager - Package Lifecycle Management"
echo "4. YAML Editor - Advanced Configuration"
echo "5. Full UI - All Features Together"
echo ""

read -p "Select demo mode (1-5): " choice

case $choice in
    1)
        echo "🏗️ Launching BYOB Demo..."
        echo "Features: Team workspaces, resource isolation, deployment monitoring"
        ./target/release/biomeos-ui --byob
        ;;
    2)
        echo "💿 Launching ISO Creator Demo..."
        echo "Features: Custom distributions, niche integration, multi-arch support"
        ./target/release/biomeos-ui --iso-creator
        ;;
    3)
        echo "📦 Launching Niche Manager Demo..."
        echo "Features: Package creation, testing framework, marketplace"
        ./target/release/biomeos-ui --niche-manager
        ;;
    4)
        echo "📝 Launching YAML Editor Demo..."
        echo "Features: Advanced editing, validation, templates"
        ./target/release/biomeos-ui --yaml-editor
        ;;
    5)
        echo "🌟 Launching Full UI Demo..."
        echo "Features: All components integrated with cross-workflows"
        ./target/release/biomeos-ui
        ;;
    *)
        echo "❌ Invalid choice. Please run again and select 1-5."
        exit 1
        ;;
esac

echo ""
echo "✨ Demo completed! The biomeOS Enhanced UI is ready for production use." 