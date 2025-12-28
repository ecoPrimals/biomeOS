#!/bin/bash
set -e
echo "🚀 Installing BiomeOS..."
sudo cp -r ./opt/biomeos /opt/
sudo chown -R $USER:$USER /opt/biomeos
echo "✅ Installed to /opt/biomeos"
echo ""
echo "To start:"
echo "  cd /opt/biomeos"
echo "  ./deploy-real-primals.sh"
