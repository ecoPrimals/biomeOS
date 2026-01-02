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
