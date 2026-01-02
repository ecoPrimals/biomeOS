#!/bin/bash
# Create USB Family Seed for Genetic Lineage
# Part of biomeOS USB v6.0 - Secure by Default
#
# This script generates a family genesis seed that will be shared across
# all towers deployed from this USB, enabling automatic trust while
# preserving per-tower uniqueness through local entropy mixing.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
USB_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SECRETS_DIR="$USB_ROOT/secrets"

echo "══════════════════════════════════════════════════════════════"
echo "🔐 USB Family Seed Generator"
echo "══════════════════════════════════════════════════════════════"
echo ""

# Create secrets directory
mkdir -p "$SECRETS_DIR"
chmod 700 "$SECRETS_DIR"

# Generate high-entropy seed (256 bits)
echo "📊 Generating high-entropy family seed..."
FAMILY_SEED=$(openssl rand -base64 32)

# Create unique family ID
FAMILY_ID="ecoPrimals-$(date +%Y%m%d)-$(openssl rand -hex 4)"
echo "   Family ID: $FAMILY_ID"

# Derive public family hash (for verification without exposing seed)
FAMILY_HASH=$(echo -n "$FAMILY_SEED" | sha256sum | awk '{print $1}')
echo "   Genesis Hash: $FAMILY_HASH"

# Create family seed file
cat > "$SECRETS_DIR/family-genesis.key" << EOF
{
  "version": "1.0",
  "family_id": "$FAMILY_ID",
  "genesis_seed": "$FAMILY_SEED",
  "genesis_hash": "$FAMILY_HASH",
  "created_at": "$(date -Iseconds)",
  "seed_type": "usb_base",
  "capabilities": ["tower", "orchestration", "federation"],
  "metadata": {
    "description": "Base family seed for USB deployment",
    "trust_model": "genetic_lineage",
    "architecture": "usb_seed_mixing"
  }
}
EOF

# Create public key file (safe to share for verification)
cat > "$SECRETS_DIR/family-genesis.pub" << EOF
{
  "family_id": "$FAMILY_ID",
  "genesis_hash": "$FAMILY_HASH",
  "created_at": "$(date -Iseconds)",
  "capabilities": ["tower", "orchestration", "federation"]
}
EOF

# Create README for security model
cat > "$SECRETS_DIR/README-SECURITY.txt" << 'EOF'
═══════════════════════════════════════════════════════════════
🔐 USB FAMILY SEED - SECURITY MODEL
═══════════════════════════════════════════════════════════════

This USB contains a FAMILY GENESIS SEED that enables automatic trust
between towers deployed from this USB while preserving privacy.

HOW IT WORKS:

1. USB Family Seed (Base DNA)
   • Created ONCE during USB package creation
   • Shared across all towers from this USB
   • Defines the "genetic family"

2. Local Mixing (Unique Identity)
   • Each tower: USB seed + machine entropy (hostname, MAC, UUID)
   • Creates unique child lineage per tower
   • Privacy preserved (unique identities)

3. Automatic Trust
   • Same genesis → AUTO-TRUST (cryptographic verification)
   • Different genesis → PROMPT USER (consent required)
   • No lineage → REJECT (secure by default)

SECURITY PROPERTIES:

✅ Secure by Default
   • Towers from this USB auto-trust each other
   • Reject towers from different USB/families
   • No manual configuration needed

✅ Privacy Preserved
   • Each tower has unique lineage
   • Cannot be linked without genesis hash
   • Local entropy ensures uniqueness

✅ Cryptographic Trust
   • BearDog cryptographic verification
   • Same genesis = same family proof
   • Cannot be forged or impersonated

DEPLOYMENT SCENARIOS:

✅ Home LAN (Tier 1: USB Seed Only)
   • Deploy USB to multiple towers
   • Automatic family trust
   • Secure for trusted networks

✅ Production (Tier 2: + Phone HSM)
   • Add phone entropy during deployment
   • Enhanced cryptographic strength
   • Proof of human presence

✅ Enterprise (Tier 3: + Hardware HSM)
   • Physical key required (SoloKeys/YubiKey)
   • Hardware-backed crypto
   • Maximum security

IMPORTANT FILES:

• family-genesis.key - PRIVATE seed (keep secure!)
• family-genesis.pub - PUBLIC hash (safe to share)
• README-SECURITY.txt - This file

DO NOT LOSE THIS USB! The family seed cannot be recovered.

═══════════════════════════════════════════════════════════════
Generated: $(date)
═══════════════════════════════════════════════════════════════
EOF

# Set permissions
chmod 400 "$SECRETS_DIR/family-genesis.key"   # Read-only for owner
chmod 444 "$SECRETS_DIR/family-genesis.pub"   # Read-only for all
chmod 444 "$SECRETS_DIR/README-SECURITY.txt"  # Read-only for all

echo ""
echo "══════════════════════════════════════════════════════════════"
echo "✅ USB Family Seed Created Successfully!"
echo "══════════════════════════════════════════════════════════════"
echo ""
echo "📍 Location: $SECRETS_DIR/"
echo ""
echo "Files created:"
echo "  • family-genesis.key     (PRIVATE - keep secure!)"
echo "  • family-genesis.pub     (PUBLIC - safe to share)"
echo "  • README-SECURITY.txt    (Security documentation)"
echo ""
echo "Family Details:"
echo "  • Family ID:      $FAMILY_ID"
echo "  • Genesis Hash:   $FAMILY_HASH"
echo "  • Created:        $(date)"
echo ""
echo "🔒 Security Model:"
echo "  • Towers from this USB will auto-trust each other"
echo "  • Each tower gets unique lineage (privacy)"
echo "  • Different USB families require user consent"
echo ""
echo "══════════════════════════════════════════════════════════════"
echo ""

