#!/bin/bash
# TRUE Dark Forest Demo - USB Genetic Lineage Verification
# Demonstrates challenge-response protocol for TRUE Dark Forest handshake

set -e

SOCKET="/run/user/$(id -u)/biomeos/beardog-test.sock"

echo "═══════════════════════════════════════════════════════════════════"
echo "🌑 TRUE DARK FOREST DEMO - USB Genetic Lineage Verification"
echo "═══════════════════════════════════════════════════════════════════"
echo ""

# Step 1: Check BearDog Status
echo "Step 1: Check BearDog Primal Info..."
echo ""
printf '{"jsonrpc":"2.0","method":"primal.info","params":{},"id":1}\n' | \
  nc -U "$SOCKET" | jq '.result | {name, version, status, capabilities, features: .features.genetic}'
echo ""

# Step 2: List Genetic Methods
echo "Step 2: List Genetic Methods..."
echo ""
printf '{"jsonrpc":"2.0","method":"rpc.methods","params":{},"id":1}\n' | \
  nc -U "$SOCKET" | jq -r '.result.by_namespace.genetic | .[]'
echo ""

# Step 3-5: Challenge-Response Cycle
echo "Step 3-5: Complete Challenge-Response Cycle..."
echo ""
echo "Generating challenge from usb_alpha..."
CHALLENGE=$(printf '{"jsonrpc":"2.0","method":"genetic.generate_challenge","params":{"challenger_node_id":"usb_alpha"},"id":1}\n' | \
  nc -U "$SOCKET")

CHALLENGE_ID=$(echo "$CHALLENGE" | jq -r '.result.challenge_id')
NONCE=$(echo "$CHALLENGE" | jq -r '.result.nonce')
CHALLENGER=$(echo "$CHALLENGE" | jq -r '.result.challenger')

echo "✅ Challenge generated:"
echo "   ID: $CHALLENGE_ID"
echo "   Nonce: ${NONCE:0:40}..."
echo "   Challenger: $CHALLENGER"
echo ""

echo "usb_beta responding to challenge..."
RESPONSE=$(printf "{\"jsonrpc\":\"2.0\",\"method\":\"genetic.respond_to_challenge\",\"params\":{\"challenge_id\":\"$CHALLENGE_ID\",\"nonce\":\"$NONCE\",\"responder\":\"usb_beta\"},\"id\":1}\n" | \
  nc -U "$SOCKET")

RESPONSE_SIG=$(echo "$RESPONSE" | jq -r '.result.response')
RESPONDER=$(echo "$RESPONSE" | jq -r '.result.responder')

echo "✅ Response generated:"
echo "   Signature (HMAC-SHA512): ${RESPONSE_SIG:0:40}..."
echo "   Responder: $RESPONDER"
echo ""

echo "usb_alpha verifying usb_beta's lineage..."
VERIFY=$(printf "{\"jsonrpc\":\"2.0\",\"method\":\"genetic.verify_challenge_response\",\"params\":{\"challenge_id\":\"$CHALLENGE_ID\",\"response\":\"$RESPONSE_SIG\",\"responder\":\"usb_beta\"},\"id\":1}\n" | \
  nc -U "$SOCKET")

VERIFIED=$(echo "$VERIFY" | jq -r '.result.verified')
FAMILY_VERIFIED=$(echo "$VERIFY" | jq -r '.result.family_verified')

echo "✅ Verification result:"
echo "$VERIFY" | jq '.result'
echo ""

if [ "$VERIFIED" = "true" ] && [ "$FAMILY_VERIFIED" = "true" ]; then
  echo "🎊 ═══════════════════════════════════════════════════════════════"
  echo "🎊 ✅ TRUE DARK FOREST LINEAGE VERIFIED!"
  echo "🎊 ═══════════════════════════════════════════════════════════════"
  echo "🎊"
  echo "🎊 • Same family (dark_forest_alpha) confirmed"
  echo "🎊 • HMAC-SHA512 signature valid"
  echo "🎊 • Genetic lineage proven"
  echo "🎊 • Constant-time verification"
  echo "🎊 • Ready for encrypted connection!"
  echo "🎊"
  echo "🎊 ═══════════════════════════════════════════════════════════════"
else
  echo "❌ Verification failed"
  echo "   This would happen with different families (correct behavior)"
fi

echo ""
echo "═══════════════════════════════════════════════════════════════════"
echo "🏆 TRUE DARK FOREST DEMO COMPLETE"
echo "═══════════════════════════════════════════════════════════════════"
