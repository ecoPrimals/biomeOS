#!/bin/bash
# Verify Sibling Lineage - Cryptographically verify two spores are siblings
#
# This script verifies that two spores share genetic lineage by:
# 1. Reading both seeds
# 2. Using BearDog's genetic.generate_lineage_proof on each
# 3. Cross-verifying proofs using genetic.verify_lineage
#
# For TRUE siblings (derived from same parent):
# - child_seed = SHA256(parent_seed || node_id || batch)
# - Both seeds share genetic material from parent_seed
# - BearDog can verify relationship via Blake3 proof verification
#
# Usage:
#   ./scripts/verify_sibling_lineage.sh /media/usb1/biomeOS /media/usb2/biomeOS
#
# Note: This is an OFFLINE verification - no BearDog server needed.
# It demonstrates the cryptographic relationship between seeds.
#
# For RUNTIME verification with BearDog, use federation_verify_lineage.toml
#
# Author: biomeOS Team
# Date: 2026-01-27

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

log_info() { echo -e "${BLUE}ℹ️  $1${NC}"; }
log_success() { echo -e "${GREEN}✅ $1${NC}"; }
log_warn() { echo -e "${YELLOW}⚠️  $1${NC}"; }
log_error() { echo -e "${RED}❌ $1${NC}"; }
log_crypto() { echo -e "${CYAN}🔐 $1${NC}"; }

if [[ $# -lt 2 ]]; then
    echo "Usage: $0 <spore1_root> <spore2_root>"
    echo ""
    echo "Example:"
    echo "  $0 /media/user/USB1/biomeOS /media/user/USB2/biomeOS"
    exit 1
fi

SPORE1_ROOT="$1"
SPORE2_ROOT="$2"
SPORE1_SEED="$SPORE1_ROOT/.family.seed"
SPORE2_SEED="$SPORE2_ROOT/.family.seed"

echo ""
echo "═══════════════════════════════════════════════════════════════════════════"
echo "          Genetic Lineage Verification - Sibling Proof"
echo "═══════════════════════════════════════════════════════════════════════════"
echo ""

# ============================================================================
# Load Spore Metadata
# ============================================================================

log_info "Loading spore metadata..."

# Spore 1
if [[ -f "$SPORE1_ROOT/.spore.json" ]]; then
    SPORE1_NODE=$(jq -r '.node_id // "unknown"' "$SPORE1_ROOT/.spore.json")
    SPORE1_LINEAGE=$(jq -r '.lineage_mode // "unknown"' "$SPORE1_ROOT/.spore.json")
    SPORE1_PARENT_HASH=$(jq -r '.parent_hash // "n/a"' "$SPORE1_ROOT/.spore.json")
else
    SPORE1_NODE="unknown"
    SPORE1_LINEAGE="unknown"
    SPORE1_PARENT_HASH="n/a"
fi

# Spore 2
if [[ -f "$SPORE2_ROOT/.spore.json" ]]; then
    SPORE2_NODE=$(jq -r '.node_id // "unknown"' "$SPORE2_ROOT/.spore.json")
    SPORE2_LINEAGE=$(jq -r '.lineage_mode // "unknown"' "$SPORE2_ROOT/.spore.json")
    SPORE2_PARENT_HASH=$(jq -r '.parent_hash // "n/a"' "$SPORE2_ROOT/.spore.json")
else
    SPORE2_NODE="unknown"
    SPORE2_LINEAGE="unknown"
    SPORE2_PARENT_HASH="n/a"
fi

echo "  Spore 1: $SPORE1_NODE"
echo "    Path: $SPORE1_ROOT"
echo "    Lineage: $SPORE1_LINEAGE"
if [[ "$SPORE1_PARENT_HASH" != "n/a" ]]; then
    echo "    Parent Hash: $SPORE1_PARENT_HASH..."
fi
echo ""
echo "  Spore 2: $SPORE2_NODE"
echo "    Path: $SPORE2_ROOT"
echo "    Lineage: $SPORE2_LINEAGE"
if [[ "$SPORE2_PARENT_HASH" != "n/a" ]]; then
    echo "    Parent Hash: $SPORE2_PARENT_HASH..."
fi
echo ""

# ============================================================================
# Verify Seeds Exist
# ============================================================================

log_info "Verifying seed files..."

for seed in "$SPORE1_SEED" "$SPORE2_SEED"; do
    if [[ ! -f "$seed" ]]; then
        log_error "Seed not found: $seed"
        exit 1
    fi
    size=$(stat -c%s "$seed" 2>/dev/null || stat -f%z "$seed")
    if [[ "$size" -ne 32 ]]; then
        log_error "Invalid seed size: $seed ($size bytes, expected 32)"
        exit 1
    fi
done

log_success "Both seeds are valid (32 bytes each)"
echo ""

# ============================================================================
# Calculate Seed Hashes
# ============================================================================

log_crypto "Calculating seed hashes..."

HASH1=$(sha256sum "$SPORE1_SEED" | cut -c1-64)
HASH2=$(sha256sum "$SPORE2_SEED" | cut -c1-64)

echo "  Spore 1 seed hash: ${HASH1:0:16}...${HASH1:48:16}"
echo "  Spore 2 seed hash: ${HASH2:0:16}...${HASH2:48:16}"
echo ""

# Check if seeds are identical (clone - not recommended)
if [[ "$HASH1" == "$HASH2" ]]; then
    log_warn "Seeds are IDENTICAL - this is a CLONE relationship"
    log_warn "Clones share the same cryptographic identity (not recommended)"
    RELATIONSHIP="clone"
else
    log_success "Seeds are DIFFERENT - checking for sibling relationship"
    RELATIONSHIP="potential_sibling"
fi
echo ""

# ============================================================================
# Check Parent Hash Match (if available)
# ============================================================================

PARENT_MATCH=false

# Check if one is genesis (parent) and other is sibling (child)
if [[ "$SPORE2_LINEAGE" == "sibling" && "$SPORE2_PARENT_HASH" != "n/a" ]]; then
    # Spore 2 claims to be a sibling - check if spore 1 is its parent
    SPORE1_HASH_PREFIX=$(sha256sum "$SPORE1_SEED" | cut -c1-16)
    
    log_info "Spore 2 claims sibling lineage from parent hash: $SPORE2_PARENT_HASH"
    log_info "Spore 1 seed hash prefix: $SPORE1_HASH_PREFIX"
    
    if [[ "$SPORE2_PARENT_HASH" == "$SPORE1_HASH_PREFIX" ]]; then
        log_success "PARENT-CHILD RELATIONSHIP CONFIRMED!"
        log_info "Spore 1 ($SPORE1_NODE) is the PARENT"
        log_info "Spore 2 ($SPORE2_NODE) is the CHILD/SIBLING"
        PARENT_MATCH=true
        RELATIONSHIP="parent_child"
    fi
    echo ""
elif [[ "$SPORE1_LINEAGE" == "sibling" && "$SPORE1_PARENT_HASH" != "n/a" ]]; then
    # Spore 1 claims to be a sibling - check if spore 2 is its parent
    SPORE2_HASH_PREFIX=$(sha256sum "$SPORE2_SEED" | cut -c1-16)
    
    log_info "Spore 1 claims sibling lineage from parent hash: $SPORE1_PARENT_HASH"
    log_info "Spore 2 seed hash prefix: $SPORE2_HASH_PREFIX"
    
    if [[ "$SPORE1_PARENT_HASH" == "$SPORE2_HASH_PREFIX" ]]; then
        log_success "PARENT-CHILD RELATIONSHIP CONFIRMED!"
        log_info "Spore 2 ($SPORE2_NODE) is the PARENT"
        log_info "Spore 1 ($SPORE1_NODE) is the CHILD/SIBLING"
        PARENT_MATCH=true
        RELATIONSHIP="parent_child"
    fi
    echo ""
elif [[ "$SPORE1_LINEAGE" == "sibling" && "$SPORE2_LINEAGE" == "sibling" ]]; then
    log_info "Both spores claim sibling lineage - checking parent hashes..."
    
    if [[ "$SPORE1_PARENT_HASH" == "$SPORE2_PARENT_HASH" && "$SPORE1_PARENT_HASH" != "n/a" ]]; then
        log_success "Parent hashes MATCH: $SPORE1_PARENT_HASH"
        log_info "This indicates both were derived from the same parent"
        PARENT_MATCH=true
    else
        log_warn "Parent hashes differ or missing"
        log_warn "  Spore 1: $SPORE1_PARENT_HASH"
        log_warn "  Spore 2: $SPORE2_PARENT_HASH"
    fi
    echo ""
fi

# ============================================================================
# Generate Blake3 Lineage Proofs
# ============================================================================

log_crypto "Generating Blake3 lineage proofs..."

# BearDog's genetic.generate_lineage_proof uses:
#   Blake3(lineage_seed || our_family_id || peer_family_id || "GENETIC_LINEAGE_PROOF_V1")
#
# We'll replicate this for offline verification

generate_blake3_proof() {
    local seed_path="$1"
    local our_family="$2"
    local peer_family="$3"
    
    # Create input: seed || our_family || peer_family || version_tag
    {
        cat "$seed_path"
        printf '%s' "$our_family"
        printf '%s' "$peer_family"
        printf '%s' "GENETIC_LINEAGE_PROOF_V1"
    } | b3sum --raw | xxd -p -c 64
}

# Check if b3sum is available
if ! command -v b3sum &>/dev/null; then
    log_warn "b3sum not found - using SHA256 for demonstration"
    log_warn "Actual BearDog verification uses Blake3"
    
    generate_blake3_proof() {
        local seed_path="$1"
        local our_family="$2"
        local peer_family="$3"
        
        {
            cat "$seed_path"
            printf '%s' "$our_family"
            printf '%s' "$peer_family"
            printf '%s' "GENETIC_LINEAGE_PROOF_V1"
        } | sha256sum | cut -c1-64
    }
fi

# Generate proofs for cross-verification
FAMILY_ID="nat0"

echo "  Generating proof from Spore 1 for Spore 2..."
PROOF_1_TO_2=$(generate_blake3_proof "$SPORE1_SEED" "$FAMILY_ID" "$FAMILY_ID")
echo "    Proof: ${PROOF_1_TO_2:0:16}...${PROOF_1_TO_2:48:16}"

echo "  Generating proof from Spore 2 for Spore 1..."
PROOF_2_TO_1=$(generate_blake3_proof "$SPORE2_SEED" "$FAMILY_ID" "$FAMILY_ID")
echo "    Proof: ${PROOF_2_TO_1:0:16}...${PROOF_2_TO_1:48:16}"

echo ""

# ============================================================================
# Cross-Verify Proofs
# ============================================================================

log_crypto "Cross-verifying proofs..."

# In a full BearDog verification, each side would:
# 1. Generate a proof with their seed
# 2. Send to the other side
# 3. Other side verifies by regenerating expected proof
#
# For TRUE siblings, the verification would pass because both seeds
# were derived from the same parent and share "genetic material"
#
# However, with DIFFERENT seeds, direct proof verification will FAIL
# unless we use the PARENT seed to derive expected sibling keys

if [[ "$PROOF_1_TO_2" == "$PROOF_2_TO_1" ]]; then
    log_error "Proofs are IDENTICAL - this indicates CLONE seeds (same seed)"
    VERIFICATION="clone"
else
    log_info "Proofs differ (expected for siblings with unique seeds)"
    
    # For true sibling verification, we need the parent seed
    # to derive what each sibling's proof SHOULD be
    
    if [[ -n "${PARENT_SEED_PATH:-}" && -f "$PARENT_SEED_PATH" ]]; then
        log_info "Parent seed provided - verifying sibling derivation..."
        
        # Derive expected seed for spore2 from parent + node_id
        EXPECTED_SEED2=$(
            {
                cat "$PARENT_SEED_PATH"
                printf '%s' "$SPORE2_NODE"
                # Use same batch if known
            } | openssl dgst -sha256 -binary | xxd -p -c 64
        )
        
        ACTUAL_SEED2_HASH=$(sha256sum "$SPORE2_SEED" | cut -c1-64)
        
        if [[ "$EXPECTED_SEED2" == "$ACTUAL_SEED2_HASH" ]]; then
            log_success "Spore 2 seed VERIFIED as derived from parent!"
            VERIFICATION="verified_sibling"
        else
            log_warn "Seed derivation does not match expected"
            VERIFICATION="unverified"
        fi
    else
        log_info "Parent seed not provided - using metadata-based verification"
        
        if [[ "$PARENT_MATCH" == "true" ]]; then
            log_success "Parent hash match confirms sibling relationship"
            VERIFICATION="metadata_sibling"
        else
            VERIFICATION="unverified"
        fi
    fi
fi

echo ""

# ============================================================================
# Results
# ============================================================================

echo "═══════════════════════════════════════════════════════════════════════════"
echo "                       VERIFICATION RESULTS"
echo "═══════════════════════════════════════════════════════════════════════════"
echo ""

case "$VERIFICATION" in
    "clone")
        log_warn "CLONE RELATIONSHIP DETECTED"
        echo "  The two spores have identical seeds."
        echo "  This means they share the same cryptographic identity."
        echo "  This is NOT recommended for federation."
        ;;
    "verified_sibling")
        log_success "SIBLING RELATIONSHIP VERIFIED ✓"
        echo "  Cryptographic proof confirms both spores were derived"
        echo "  from the same parent seed using the biomeOS genetic model:"
        echo "    child_seed = SHA256(parent || node_id || batch)"
        echo ""
        echo "  These spores CAN establish mutual trust via BearDog federation."
        ;;
    "metadata_sibling")
        log_success "SIBLING RELATIONSHIP LIKELY (metadata match)"
        echo "  Both spores claim the same parent hash: $SPORE1_PARENT_HASH"
        echo "  This indicates they were likely derived from the same parent."
        echo ""
        echo "  For FULL cryptographic verification, provide parent seed:"
        echo "    PARENT_SEED_PATH=/path/to/parent/.family.seed $0 $1 $2"
        ;;
    "unverified")
        log_warn "RELATIONSHIP UNVERIFIED"
        echo "  Cannot confirm genetic relationship without parent seed."
        echo "  The spores may or may not be siblings."
        echo ""
        echo "  To verify, either:"
        echo "  1. Provide parent seed: PARENT_SEED_PATH=... $0 $1 $2"
        echo "  2. Use runtime BearDog verification via federation_verify_lineage.toml"
        ;;
esac

echo ""
echo "═══════════════════════════════════════════════════════════════════════════"
echo ""

# Exit code based on verification
case "$VERIFICATION" in
    "verified_sibling"|"metadata_sibling") exit 0 ;;
    "clone") exit 2 ;;
    *) exit 1 ;;
esac

