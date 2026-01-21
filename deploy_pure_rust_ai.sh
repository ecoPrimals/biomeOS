#!/bin/bash
# Pure Rust AI Stack Deployment
# Date: January 21, 2026

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 Deploying Pure Rust AI Stack${NC}"

# Kill old processes
echo "Stopping old processes..."
pkill -9 squirrel 2>/dev/null || true
pkill -9 songbird 2>/dev/null || true
pkill -9 beardog 2>/dev/null || true
sleep 2

# Clean old sockets
rm -f /tmp/squirrel-nat0.sock
rm -f /tmp/songbird-nat0.sock  
rm -f /tmp/beardog-nat0.sock

echo -e "${GREEN}✅ Clean slate${NC}"

# 1. Start BearDog (security provider)
echo -e "${GREEN}Starting BearDog...${NC}"
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
export BEARDOG_FAMILY_ID="nat0"
nohup /home/eastgate/Development/ecoPrimals/plasmidBin/primals/beardog/beardog-x86_64-musl server \
  > /tmp/beardog-nat0.log 2>&1 &
sleep 3
echo -e "${GREEN}✅ BearDog running${NC}"

# 2. Start Songbird (HTTP provider)
echo -e "${GREEN}Starting Songbird...${NC}"
export SONGBIRD_SECURITY_PROVIDER="/tmp/beardog-nat0.sock"
export SONGBIRD_FAMILY_ID="nat0"
nohup /home/eastgate/Development/ecoPrimals/plasmidBin/primals/songbird/songbird-x86_64 server \
  > /tmp/songbird-nat0.log 2>&1 &
sleep 4
echo -e "${GREEN}✅ Songbird running${NC}"

# 3. Start Squirrel (AI orchestrator)
echo -e "${GREEN}Starting Squirrel...${NC}"
export ANTHROPIC_API_KEY="sk-ant-REDACTED"
export CAPABILITY_REGISTRY_SOCKET="/tmp/neural-api-nat0.sock"
unset AI_PROVIDER_SOCKETS  # Critical: Don't treat Songbird as AI provider!

/home/eastgate/Development/ecoPrimals/plasmidBin/primals/squirrel/squirrel-x86_64 server \
  --socket /tmp/squirrel-nat0.sock \
  > /tmp/squirrel-pure-rust.log 2>&1 &

sleep 12
echo -e "${GREEN}✅ Squirrel running${NC}"

# Verify stack
echo ""
echo -e "${GREEN}═══════════════════════════════════════════════${NC}"
echo -e "${GREEN}Stack Status${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════${NC}"

# Check BearDog
if [ -S /tmp/beardog-nat0.sock ]; then
    echo -e "  ✅ BearDog: /tmp/beardog-nat0.sock"
else
    echo -e "  ❌ BearDog: Socket not found"
fi

# Check Songbird  
if [ -S /tmp/songbird-nat0.sock ]; then
    echo -e "  ✅ Songbird: /tmp/songbird-nat0.sock"
else
    echo -e "  ❌ Songbird: Socket not found"
fi

# Check Squirrel
if [ -S /tmp/squirrel-nat0.sock ]; then
    echo -e "  ✅ Squirrel: /tmp/squirrel-nat0.sock"
else
    echo -e "  ❌ Squirrel: Socket not found"
fi

echo ""
echo -e "${GREEN}Squirrel Initialization:${NC}"
tail -15 /tmp/squirrel-pure-rust.log | grep -E "Anthropic adapter|AI router initialized|provider\(s\)"

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════${NC}"
echo -e "${GREEN}Pure Rust AI Stack Deployed!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════${NC}"
echo ""
echo "Test with:"
echo "  echo '{\"jsonrpc\":\"2.0\",\"method\":\"query_ai\",\"params\":{\"prompt\":\"Hello!\"},\"id\":1}' | nc -N -U /tmp/squirrel-nat0.sock"

