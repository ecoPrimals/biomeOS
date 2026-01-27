#!/usr/bin/env bash
#
# LiveSpore Validation Script
#
# Validates system files on a LiveSpore and optionally updates them.
# Preserves the personal data vault.
#
# Usage:
#   ./validate_spore.sh [OPTIONS] <spore_path>
#
# Options:
#   --check     Validate only, report discrepancies (default)
#   --update    Update outdated files automatically
#   --strict    Fail if any mismatch
#   --manifest  Path to manifest file (default: embedded)
#
# Examples:
#   ./validate_spore.sh /media/user/USB/biomeOS
#   ./validate_spore.sh --update /media/eastgate/BEA6-BBCE/biomeOS
#
set -euo pipefail

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# CONFIGURATION
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
MODE="check"
MANIFEST_PATH=""
SPORE_PATH=""
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIOMEOS_ROOT="$(dirname "$SCRIPT_DIR")"

# Expected checksums (embedded from manifest)
declare -A EXPECTED_MD5=(
    ["primals/beardog"]="bf3c9015b16cb1f5ddaa1abbb50ce736"
    ["primals/songbird"]="c689394868ea4cd9967556a163679382"
    ["deploy.sh"]="8fa2d4e8b7d7fcb8fce62796cd84bae7"
)

declare -A EXPECTED_SIZE=(
    ["primals/beardog"]=7207608
    ["primals/songbird"]=22124976
    ["deploy.sh"]=10358
)

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# PARSE ARGUMENTS
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
while [[ $# -gt 0 ]]; do
    case "$1" in
        --check)
            MODE="check"
            shift
            ;;
        --update)
            MODE="update"
            shift
            ;;
        --strict)
            MODE="strict"
            shift
            ;;
        --manifest)
            MANIFEST_PATH="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [--check|--update|--strict] <spore_path>"
            echo ""
            echo "Options:"
            echo "  --check   Validate only, report discrepancies (default)"
            echo "  --update  Update outdated files automatically"
            echo "  --strict  Fail if any mismatch"
            exit 0
            ;;
        *)
            SPORE_PATH="$1"
            shift
            ;;
    esac
done

if [[ -z "$SPORE_PATH" ]]; then
    echo "Error: Spore path required"
    echo "Usage: $0 [--check|--update|--strict] <spore_path>"
    exit 1
fi

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# FUNCTIONS
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

check_file() {
    local rel_path="$1"
    local full_path="$SPORE_PATH/$rel_path"
    local expected_md5="${EXPECTED_MD5[$rel_path]:-}"
    local expected_size="${EXPECTED_SIZE[$rel_path]:-}"
    
    if [[ ! -f "$full_path" ]]; then
        echo "  ❌ MISSING: $rel_path"
        return 1
    fi
    
    local actual_size=$(stat -c%s "$full_path" 2>/dev/null || stat -f%z "$full_path")
    local actual_md5=$(md5sum "$full_path" 2>/dev/null | cut -d' ' -f1 || md5 -q "$full_path")
    
    if [[ "$actual_size" != "$expected_size" ]]; then
        echo "  ⚠️  SIZE MISMATCH: $rel_path"
        echo "      Expected: $expected_size bytes"
        echo "      Actual:   $actual_size bytes"
        return 2
    fi
    
    if [[ "$actual_md5" != "$expected_md5" ]]; then
        echo "  ⚠️  CHECKSUM MISMATCH: $rel_path"
        echo "      Expected: $expected_md5"
        echo "      Actual:   $actual_md5"
        return 3
    fi
    
    echo "  ✅ $rel_path (${actual_size} bytes)"
    return 0
}

update_file() {
    local rel_path="$1"
    local source_path=""
    
    case "$rel_path" in
        "primals/beardog")
            source_path="$BIOMEOS_ROOT/plasmidBin/primals/beardog/beardog-active"
            ;;
        "primals/songbird")
            source_path="$BIOMEOS_ROOT/plasmidBin/primals/songbird/songbird-active"
            ;;
        "deploy.sh")
            source_path="$BIOMEOS_ROOT/templates/livespore_deploy.sh"
            ;;
        *)
            echo "  ❌ Unknown file: $rel_path"
            return 1
            ;;
    esac
    
    if [[ ! -f "$source_path" ]]; then
        # Try relative to script
        source_path="$(dirname "$SCRIPT_DIR")/${source_path#$BIOMEOS_ROOT/}"
    fi
    
    if [[ -f "$source_path" ]]; then
        # Backup existing
        if [[ -f "$SPORE_PATH/$rel_path" ]]; then
            cp "$SPORE_PATH/$rel_path" "$SPORE_PATH/$rel_path.bak" 2>/dev/null || true
        fi
        
        # Copy new file (follow symlinks)
        cp -L "$source_path" "$SPORE_PATH/$rel_path"
        echo "  📦 UPDATED: $rel_path"
        return 0
    else
        echo "  ❌ Source not found: $source_path"
        return 1
    fi
}

ensure_vault() {
    local vault_dirs=(
        "vault"
        "vault/deployments"
        "vault/logs"
        "vault/certs"
        "vault/workdata"
        "vault/metrics"
        "vault/federation"
    )
    
    for dir in "${vault_dirs[@]}"; do
        mkdir -p "$SPORE_PATH/$dir"
    done
    
    # Initialize experience tracker if not exists
    if [[ ! -f "$SPORE_PATH/vault/experience.json" ]]; then
        local node_id="unknown"
        if [[ -f "$SPORE_PATH/.spore.json" ]]; then
            node_id=$(grep -o '"node_id"[[:space:]]*:[[:space:]]*"[^"]*"' "$SPORE_PATH/.spore.json" | sed 's/.*"\([^"]*\)"$/\1/' || echo "unknown")
        fi
        
        cat > "$SPORE_PATH/vault/experience.json" << EOF
{
  "spore_id": "$node_id",
  "created_at": "$(date -Iseconds)",
  "deployments": [],
  "federation_events": [],
  "total_uptime_seconds": 0,
  "hosts_connected": []
}
EOF
        echo "  📝 Created experience.json"
    fi
}

record_validation() {
    local result="$1"
    local updates="$2"
    
    if [[ -f "$SPORE_PATH/vault/experience.json" ]]; then
        # Use jq if available, otherwise simple append
        if command -v jq &> /dev/null; then
            local entry=$(cat << EOF
{
  "timestamp": "$(date -Iseconds)",
  "host": "$(hostname)",
  "ip": "$(hostname -I 2>/dev/null | awk '{print $1}' || echo 'unknown')",
  "validation_result": "$result",
  "updates_applied": $updates,
  "mode": "$MODE"
}
EOF
)
            jq --argjson entry "$entry" '.deployments += [$entry]' "$SPORE_PATH/vault/experience.json" > "$SPORE_PATH/vault/experience.json.tmp" && \
            mv "$SPORE_PATH/vault/experience.json.tmp" "$SPORE_PATH/vault/experience.json"
        fi
    fi
}

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# MAIN
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 LiveSpore Validation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Path: $SPORE_PATH"
echo "Mode: $MODE"
echo ""

# Verify spore exists
if [[ ! -d "$SPORE_PATH" ]]; then
    echo "❌ Error: Spore path does not exist: $SPORE_PATH"
    exit 1
fi

# Check for spore markers
if [[ ! -f "$SPORE_PATH/.family.seed" ]]; then
    echo "❌ Error: Not a valid spore (missing .family.seed)"
    exit 1
fi

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# STEP 1: Validate Seed
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo "🧬 Validating genetic seed..."
SEED_SIZE=$(stat -c%s "$SPORE_PATH/.family.seed" 2>/dev/null || stat -f%z "$SPORE_PATH/.family.seed")
if [[ "$SEED_SIZE" -eq 64 ]]; then
    echo "  ✅ .family.seed (64 bytes - genesis + node_key)"
else
    echo "  ⚠️  .family.seed ($SEED_SIZE bytes - expected 64)"
fi

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# STEP 2: Validate System Files
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo ""
echo "📦 Validating system files..."

ERRORS=0
UPDATES_NEEDED=()

for file in "primals/beardog" "primals/songbird" "deploy.sh"; do
    if ! check_file "$file"; then
        ERRORS=$((ERRORS + 1))
        UPDATES_NEEDED+=("$file")
    fi
done

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# STEP 3: Handle Results Based on Mode
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo ""

UPDATES_APPLIED=0
RESULT="unknown"

if [[ $ERRORS -eq 0 ]]; then
    RESULT="valid"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "✅ All system files valid"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
else
    case "$MODE" in
        check)
            RESULT="invalid"
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            echo "⚠️  $ERRORS file(s) need updating"
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            echo "Run with --update to fix"
            ;;
        update)
            echo "📦 Updating files..."
            for file in "${UPDATES_NEEDED[@]}"; do
                if update_file "$file"; then
                    UPDATES_APPLIED=$((UPDATES_APPLIED + 1))
                fi
            done
            RESULT="updated"
            echo ""
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            echo "✅ Updated $UPDATES_APPLIED file(s)"
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            ;;
        strict)
            RESULT="failed"
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            echo "❌ Validation failed (strict mode)"
            echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
            exit 1
            ;;
    esac
fi

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# STEP 4: Ensure Personal Data Vault
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo ""
echo "📁 Ensuring personal data vault..."
ensure_vault
echo "  ✅ Vault directories ready"

# Record this validation
record_validation "$RESULT" "$UPDATES_APPLIED"

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# SUMMARY
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Spore:   $SPORE_PATH"
echo "  Result:  $RESULT"
echo "  Updates: $UPDATES_APPLIED"
echo ""
echo "  System files validated against manifest"
echo "  Personal data vault preserved at vault/"
echo ""

# Get node info
if [[ -f "$SPORE_PATH/.spore.json" ]]; then
    NODE_ID=$(grep -o '"node_id"[[:space:]]*:[[:space:]]*"[^"]*"' "$SPORE_PATH/.spore.json" | sed 's/.*"\([^"]*\)"$/\1/' || echo "unknown")
    echo "  Node ID: $NODE_ID"
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

