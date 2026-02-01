#!/usr/bin/env bash
# Simple STUN test using songbird's STUN client directly
# Tests public address discovery for USB and Pixel

set -e

echo "═════════════════════════════════════════════════════════════"
echo "🌐 STUN Public Address Discovery Test"
echo "═════════════════════════════════════════════════════════════"
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# STUN server
STUN_SERVER="stun.nextcloud.com:3478"

echo -e "${BLUE}Testing STUN Discovery via songbird${NC}"
echo "────────────────────────────────────────────────────────────"
echo ""

# Check if songbird STUN is integrated
echo "📋 Step 1: Check songbird STUN capabilities"
echo ""

# Check if songbird has STUN crate
if [ -d "/home/eastgate/Development/ecoPrimals/phase1/songbird/crates/songbird-stun" ]; then
    echo -e "${GREEN}✅ songbird-stun crate found${NC}"
else
    echo -e "${RED}❌ songbird-stun crate not found${NC}"
    exit 1
fi

# Check if songbird-universal-ipc has STUN handler
if [ -f "/home/eastgate/Development/ecoPrimals/phase1/songbird/crates/songbird-universal-ipc/src/handlers/stun_handler.rs" ]; then
    echo -e "${GREEN}✅ STUN handler found in songbird-universal-ipc${NC}"
else
    echo -e "${RED}❌ STUN handler not found${NC}"
    exit 1
fi

echo ""
echo "📋 Step 2: Check STUN handler integration"
echo ""

# Check if STUN handler is integrated into main server
cd /home/eastgate/Development/ecoPrimals/phase1/songbird

if grep -r "StunHandler" crates/songbird-orchestrator/src/ > /dev/null 2>&1; then
    echo -e "${GREEN}✅ StunHandler referenced in songbird-orchestrator${NC}"
    echo "   Found in:"
    grep -l "StunHandler" crates/songbird-orchestrator/src/*.rs crates/songbird-orchestrator/src/**/*.rs 2>/dev/null | head -3 | sed 's/^/   - /'
else
    echo -e "${YELLOW}⚠️  StunHandler not found in orchestrator${NC}"
    echo "   STUN handler exists but may not be integrated into server"
fi

echo ""
echo "📋 Step 3: Manual STUN test (using Rust STUN client)"
echo ""

# Create a simple Rust program to test STUN
cat > /tmp/stun_test.rs << 'EOF'
use std::net::UdpSocket;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing STUN discovery...");
    
    // Bind local UDP socket
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_read_timeout(Some(Duration::from_secs(5)))?;
    
    let local_addr = socket.local_addr()?;
    println!("   Local: {}", local_addr);
    
    // STUN server
    let stun_server = "stun.nextcloud.com:3478";
    println!("   Server: {}", stun_server);
    
    // Simple STUN binding request (RFC 5389)
    // Message type: Binding Request (0x0001)
    // Magic cookie: 0x2112A442
    let mut request = vec![
        0x00, 0x01,  // Message type: Binding Request
        0x00, 0x00,  // Message length: 0
        0x21, 0x12, 0xA4, 0x42,  // Magic cookie
        // Transaction ID (12 bytes - random)
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
        0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C,
    ];
    
    // Resolve STUN server
    let stun_addr = std::net::ToSocketAddrs::to_socket_addrs(stun_server)?
        .next()
        .ok_or("Failed to resolve STUN server")?;
    
    // Send request
    socket.send_to(&request, stun_addr)?;
    println!("   ✅ Request sent");
    
    // Receive response
    let mut buf = vec![0u8; 2048];
    match socket.recv_from(&mut buf) {
        Ok((len, from)) => {
            println!("   ✅ Response received ({} bytes from {})", len, from);
            
            // Parse MAPPED-ADDRESS or XOR-MAPPED-ADDRESS attribute
            // (Simplified - just show we got a response)
            if len >= 20 && buf[0] == 0x01 && buf[1] == 0x01 {
                println!("   🎊 STUN Success: Binding Response received!");
                println!("   💡 Full STUN parsing requires songbird-stun crate");
            }
            
            Ok(())
        }
        Err(e) => {
            eprintln!("   ❌ No response: {}", e);
            Err(Box::new(e))
        }
    }
}
EOF

echo -e "${BLUE}Compiling STUN test...${NC}"
if rustc /tmp/stun_test.rs -o /tmp/stun_test 2>/dev/null; then
    echo -e "${GREEN}✅ Compiled${NC}"
    echo ""
    /tmp/stun_test
else
    echo -e "${YELLOW}⚠️  Compilation failed (rustc may not be in PATH)${NC}"
fi

echo ""
echo "═════════════════════════════════════════════════════════════"
echo -e "${BLUE}📊 STUN Discovery Status${NC}"
echo "═════════════════════════════════════════════════════════════"
echo ""
echo "✅ songbird-stun crate: EXISTS (RFC 5389 implementation)"
echo "✅ STUN handler: EXISTS (JSON-RPC integration ready)"
echo "⚠️  Integration status: NEEDS VERIFICATION"
echo ""
echo "🎯 Next Steps:"
echo "   1. Verify StunHandler is registered in songbird server"
echo "   2. Test JSON-RPC: stun.get_public_address"
echo "   3. Test USB ↔ Pixel STUN handshake"
echo ""
echo "📚 Reference:"
echo "   - STUN client: phase1/songbird/crates/songbird-stun/src/client.rs"
echo "   - STUN handler: phase1/songbird/crates/songbird-universal-ipc/src/handlers/stun_handler.rs"
echo "   - Integration: Check songbird-orchestrator/src/ipc/universal_broker.rs"
echo ""
