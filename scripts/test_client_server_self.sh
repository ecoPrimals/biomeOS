#!/bin/bash
set -e

echo "═══════════════════════════════════════════════════════════"
echo "🔬 TOWER ATOMIC CLIENT/SERVER SELF-TEST"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Cleanup
echo "Step 1: Cleanup..."
pkill -9 -f "beardog|songbird|server_test|client_test" 2>/dev/null || true
rm -f /tmp/beardog.sock
rm -f /tmp/server-transcript.log /tmp/client-transcript.log
sleep 2
echo "✅ Cleanup complete"
echo ""

# Start BearDog
echo "Step 2: Starting BearDog..."
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
RUST_LOG=info ./plasmidBin/primals/beardog/beardog server \
  --socket /tmp/beardog.sock \
  --family-id test \
  > /tmp/beardog-test.log 2>&1 &
BEARDOG_PID=$!
echo "✅ BearDog started (PID: $BEARDOG_PID)"
sleep 3
echo ""

# Start Songbird Server (talks to BearDog directly)
echo "Step 3: Starting Songbird TLS Server..."
RUST_LOG=info \
  ./plasmidBin/primals/songbird/server_test \
  --port 8443 \
  --beardog-socket /tmp/beardog.sock \
  > /tmp/server-transcript.log 2>&1 &
SERVER_PID=$!
echo "✅ Songbird Server started (PID: $SERVER_PID)"
sleep 5
echo ""

# Make client request (talks to BearDog directly)
echo "Step 4: Starting Songbird TLS Client..."
RUST_LOG=info \
  ./plasmidBin/primals/songbird/client_test \
  --url https://localhost:8443 \
  --skip-verify \
  --beardog-socket /tmp/beardog.sock \
  > /tmp/client-transcript.log 2>&1
CLIENT_EXIT=$?
echo "✅ Client request complete (exit code: $CLIENT_EXIT)"
sleep 2
echo ""

# Stop servers
echo "Step 5: Stopping servers..."
kill $SERVER_PID $BEARDOG_PID 2>/dev/null || true
sleep 1
echo "✅ Servers stopped"
echo ""

echo "═══════════════════════════════════════════════════════════"
echo "📊 TRANSCRIPT COMPARISON"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Check if logs exist
if [ ! -f /tmp/client-transcript.log ] || [ ! -f /tmp/server-transcript.log ]; then
    echo "❌ Log files missing!"
    echo "Client log exists: $([ -f /tmp/client-transcript.log ] && echo 'YES' || echo 'NO')"
    echo "Server log exists: $([ -f /tmp/server-transcript.log ] && echo 'YES' || echo 'NO')"
    exit 1
fi

# Extract transcripts
echo "Extracting client transcript..."
grep "CLIENT.*0000:" /tmp/client-transcript.log | awk '{print $NF}' > /tmp/client.hex || true
CLIENT_LINES=$(wc -l < /tmp/client.hex)

echo "Extracting server transcript..."
grep "SERVER.*0000:" /tmp/server-transcript.log | awk '{print $NF}' > /tmp/server.hex || true
SERVER_LINES=$(wc -l < /tmp/server.hex)

echo ""
echo "📝 Transcript Info:"
echo "   Client transcript: $CLIENT_LINES lines"
echo "   Server transcript: $SERVER_LINES lines"
echo ""

# If no transcript lines found, show what we have
if [ "$CLIENT_LINES" -eq 0 ] || [ "$SERVER_LINES" -eq 0 ]; then
    echo "⚠️  No transcript hex dumps found in logs!"
    echo ""
    echo "Client log (last 30 lines):"
    tail -30 /tmp/client-transcript.log
    echo ""
    echo "Server log (last 30 lines):"
    tail -30 /tmp/server-transcript.log
    exit 1
fi

# Compare
echo "Comparing transcripts line by line..."
if diff -u /tmp/client.hex /tmp/server.hex > /tmp/transcript-diff.txt 2>&1; then
    echo ""
    echo "✅✅✅ TRANSCRIPTS MATCH PERFECTLY! ✅✅✅"
    echo ""
    echo "This means:"
    echo "  ✅ Both sides compute SAME transcript"
    echo "  ✅ Both sides will derive SAME keys"
    echo "  ✅ Issue must be elsewhere"
else
    echo ""
    echo "❌ DIFFERENCES FOUND!"
    echo ""
    echo "First 50 lines of diff:"
    head -50 /tmp/transcript-diff.txt
    echo ""
    echo "Full diff saved to: /tmp/transcript-diff.txt"
    echo ""
    echo "This reveals:"
    echo "  ❌ Exact byte differences between client and server"
    echo "  ❌ Which message differs"
    echo "  ❌ What needs to be fixed"
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "📁 Log Files:"
echo "   Client: /tmp/client-transcript.log"
echo "   Server: /tmp/server-transcript.log"
echo "   BearDog: /tmp/beardog-test.log"
echo "   Diff: /tmp/transcript-diff.txt"
echo "═══════════════════════════════════════════════════════════"

