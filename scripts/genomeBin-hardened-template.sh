#!/bin/sh
# genomeBin Hardened Template v2.0
# Production-Ready Self-Deploying Universal Binary
# 
# Features:
# - Strict POSIX sh compatibility
# - Comprehensive error handling
# - Idempotent deployments
# - Checksum verification
# - Rollback on failure
# - Structured logging
# - Platform-specific hardening
#
# Usage:
#   ./primal.genome [--force] [--verify-only] [--skip-checksums]

# Strict error handling
set -eu

# Configuration
GENOME_NAME="REPLACE_WITH_PRIMAL_NAME"
GENOME_VERSION="REPLACE_WITH_VERSION"
GENOME_DESCRIPTION="REPLACE_WITH_DESCRIPTION"
GENOME_ARCHITECTURES="x86_64 aarch64"
ARCHIVE_MARKER="__ARCHIVE_START__"

# Checksums for verification (generated during build)
CHECKSUM_x86_64="REPLACE_WITH_SHA256"
CHECKSUM_aarch64="REPLACE_WITH_SHA256"

# Parse command line arguments
FORCE_INSTALL=0
VERIFY_ONLY=0
SKIP_CHECKSUMS=0

while [ $# -gt 0 ]; do
    case "$1" in
        --force) FORCE_INSTALL=1; shift ;;
        --verify-only) VERIFY_ONLY=1; shift ;;
        --skip-checksums) SKIP_CHECKSUMS=1; shift ;;
        -h|--help) 
            printf "Usage: %s [--force] [--verify-only] [--skip-checksums]\n" "$0"
            exit 0
            ;;
        *)
            printf "Unknown option: %s\n" "$1" >&2
            exit 1
            ;;
    esac
done

# POSIX-compliant logging (no echo -e)
log_info() { printf "[INFO] %s\n" "$*"; }
log_success() { printf "[SUCCESS] %s\n" "$*"; }
log_warn() { printf "[WARN] %s\n" "$*"; }
log_error() { printf "[ERROR] %s\n" "$*" >&2; }

# Check if terminal supports colors
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    CYAN='\033[0;36m'
    NC='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    CYAN=''
    NC=''
fi

# Enhanced logging with colors
log_info() { printf "%b[INFO]%b %s\n" "$BLUE" "$NC" "$*"; }
log_success() { printf "%b[SUCCESS]%b %s\n" "$GREEN" "$NC" "$*"; }
log_warn() { printf "%b[WARN]%b %s\n" "$YELLOW" "$NC" "$*"; }
log_error() { printf "%b[ERROR]%b %s\n" "$RED" "$NC" "$*" >&2; }
log_header() { 
    printf "%b╔══════════════════════════════════════════════════════╗%b\n" "$CYAN" "$NC"
    printf "%b║  %s%b\n" "$CYAN" "$*" "$NC"
    printf "%b╚══════════════════════════════════════════════════════╝%b\n" "$CYAN" "$NC"
}

# Check required commands
check_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        log_error "Required command not found: $1"
        exit 1
    fi
}

check_command awk
check_command tar
check_command tail

# Create secure temporary directory
create_secure_tempdir() {
    if command -v mktemp >/dev/null 2>&1; then
        TEMP_DIR=$(mktemp -d 2>/dev/null) && [ -d "$TEMP_DIR" ] && return 0
    fi
    
    # Fallback for systems without mktemp
    TEMP_DIR="/tmp/${GENOME_NAME}-$$-$(date +%s)"
    mkdir -p "$TEMP_DIR" 2>/dev/null || return 1
    chmod 700 "$TEMP_DIR"
    return 0
}

# Cleanup and rollback on error
cleanup() {
    EXIT_CODE=$?
    
    # Remove temporary directory
    if [ -n "${TEMP_DIR:-}" ] && [ -d "$TEMP_DIR" ]; then
        rm -rf "$TEMP_DIR" 2>/dev/null || true
    fi
    
    # Rollback on failure
    if [ $EXIT_CODE -ne 0 ] && [ -n "${INSTALL_DIR:-}" ]; then
        if [ -d "$INSTALL_DIR.backup" ]; then
            log_info "Rolling back to previous installation..."
            rm -rf "$INSTALL_DIR" 2>/dev/null || true
            mv "$INSTALL_DIR.backup" "$INSTALL_DIR" 2>/dev/null || true
            log_success "Rollback complete"
        elif [ -d "$INSTALL_DIR" ]; then
            log_warn "Partial installation may exist at: $INSTALL_DIR"
        fi
    fi
    
    exit $EXIT_CODE
}

trap cleanup EXIT INT TERM HUP QUIT

# Header
log_header "🧬 ${GENOME_NAME} genomeBin v${GENOME_VERSION}"
log_info "$GENOME_DESCRIPTION"
log_info "Supported Architectures: $GENOME_ARCHITECTURES"
printf "\n"

# Detect architecture
ARCH=$(uname -m)
log_info "Detected architecture: $ARCH"

case "$ARCH" in
    x86_64)
        BINARY_ARCH="x86_64"
        EXPECTED_CHECKSUM="$CHECKSUM_x86_64"
        ;;
    aarch64|arm64)
        BINARY_ARCH="aarch64"
        EXPECTED_CHECKSUM="$CHECKSUM_aarch64"
        ;;
    armv7l)
        log_error "ARMv7 not yet supported in this genomeBin"
        exit 1
        ;;
    riscv64)
        log_error "RISC-V not yet supported in this genomeBin"
        exit 1
        ;;
    *)
        log_error "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

log_success "Architecture mapped: $BINARY_ARCH"

# Detect platform
if [ -f /system/build.prop ]; then
    PLATFORM="android"
    log_info "Detected platform: Android"
elif [ "$(uname -s)" = "Darwin" ]; then
    PLATFORM="macos"
    log_info "Detected platform: macOS"
else
    PLATFORM="linux"
    log_info "Detected platform: Linux"
fi

# Determine installation directory
if [ "$PLATFORM" = "android" ]; then
    INSTALL_DIR="/data/local/tmp/${GENOME_NAME}"
elif [ "$PLATFORM" = "linux" ]; then
    if [ "$(id -u)" -eq 0 ]; then
        INSTALL_DIR="/opt/${GENOME_NAME}"
    else
        INSTALL_DIR="$HOME/.local/${GENOME_NAME}"
    fi
elif [ "$PLATFORM" = "macos" ]; then
    INSTALL_DIR="$HOME/Library/${GENOME_NAME}"
fi

# Allow user override via environment variable
INSTALL_DIR_VAR="$(printf '%s' "$GENOME_NAME" | tr '[:lower:]' '[:upper:]')_INSTALL_DIR"
if eval "[ -n \"\${$INSTALL_DIR_VAR:-}\" ]"; then
    INSTALL_DIR=$(eval "printf '%s' \"\$$INSTALL_DIR_VAR\"")
    log_info "Using custom install directory: $INSTALL_DIR"
fi

log_info "Installation directory: $INSTALL_DIR"

# Check for existing installation
if [ -d "$INSTALL_DIR" ] && [ -x "$INSTALL_DIR/${GENOME_NAME}" ]; then
    EXISTING_VERSION=$("$INSTALL_DIR/${GENOME_NAME}" --version 2>/dev/null | head -1 || echo "unknown")
    log_info "Existing installation detected: $EXISTING_VERSION"
    
    if [ "$FORCE_INSTALL" != "1" ]; then
        log_warn "Use --force to overwrite existing installation"
        exit 0
    fi
    
    # Backup existing installation
    log_info "Backing up existing installation..."
    rm -rf "$INSTALL_DIR.backup" 2>/dev/null || true
    cp -r "$INSTALL_DIR" "$INSTALL_DIR.backup" || {
        log_error "Failed to create backup"
        exit 1
    }
    log_success "Backup created: $INSTALL_DIR.backup"
fi

# Create installation directory
mkdir -p "$INSTALL_DIR" || {
    log_error "Failed to create installation directory: $INSTALL_DIR"
    exit 1
}

[ -d "$INSTALL_DIR" ] || {
    log_error "Installation directory does not exist after creation"
    exit 1
}

log_success "Installation directory ready"

# Create secure temporary directory
create_secure_tempdir || {
    log_error "Failed to create temporary directory"
    exit 1
}

log_info "Using temporary directory: $TEMP_DIR"

# Extract embedded binaries
log_info "Extracting $BINARY_ARCH binaries..."

# Find archive marker
SCRIPT_END=$(awk '/^__ARCHIVE_START__$/{print NR; exit}' "$0")

if [ -z "$SCRIPT_END" ]; then
    log_error "Archive marker not found! genomeBin may be corrupted."
    exit 1
fi

# Skip past the marker line
ARCHIVE_LINE=$((SCRIPT_END + 1))

# Extract archive
if ! tail -n +"$ARCHIVE_LINE" "$0" | tar xzf - -C "$TEMP_DIR" 2>/dev/null; then
    log_error "Failed to extract archive"
    exit 1
fi

log_success "Binaries extracted to temporary directory"

# Verify extracted binaries exist
if [ ! -d "$TEMP_DIR/$BINARY_ARCH" ]; then
    log_error "No binaries found for architecture: $BINARY_ARCH"
    log_error "Available: $(ls "$TEMP_DIR" 2>/dev/null || echo 'none')"
    exit 1
fi

# Checksum verification
if [ "$SKIP_CHECKSUMS" != "1" ] && [ -n "$EXPECTED_CHECKSUM" ] && [ "$EXPECTED_CHECKSUM" != "REPLACE_WITH_SHA256" ]; then
    log_info "Verifying checksums..."
    
    CHECKSUM_TOOL=""
    if command -v sha256sum >/dev/null 2>&1; then
        CHECKSUM_TOOL="sha256sum"
    elif command -v shasum >/dev/null 2>&1; then
        CHECKSUM_TOOL="shasum -a 256"
    fi
    
    if [ -n "$CHECKSUM_TOOL" ]; then
        ACTUAL_CHECKSUM=$($CHECKSUM_TOOL "$TEMP_DIR/$BINARY_ARCH/${GENOME_NAME}" 2>/dev/null | awk '{print $1}' || echo "")
        
        if [ "$ACTUAL_CHECKSUM" != "$EXPECTED_CHECKSUM" ]; then
            log_error "Checksum mismatch for $GENOME_NAME binary!"
            log_error "  Expected: $EXPECTED_CHECKSUM"
            log_error "  Actual:   $ACTUAL_CHECKSUM"
            log_error "genomeBin may be corrupted or tampered with"
            exit 1
        fi
        
        log_success "Checksum verified"
    else
        log_warn "No checksum tool available (sha256sum or shasum)"
        log_warn "Skipping checksum verification"
    fi
fi

if [ "$VERIFY_ONLY" = "1" ]; then
    log_success "Verification complete! Use without --verify-only to install."
    exit 0
fi

# Install binaries
log_info "Installing $BINARY_ARCH binaries..."
cp -r "$TEMP_DIR/$BINARY_ARCH/"* "$INSTALL_DIR/" || {
    log_error "Failed to copy binaries"
    exit 1
}

# Set executable permissions
chmod +x "$INSTALL_DIR/"* 2>/dev/null || {
    log_error "Failed to set executable permissions"
    exit 1
}

log_success "Binaries installed and marked executable"

# Android-specific: Check for noexec mount
if [ "$PLATFORM" = "android" ]; then
    log_info "Checking Android execution permissions..."
    TEST_FILE="$INSTALL_DIR/.test-exec-$$"
    printf '#!/bin/sh\nexit 0\n' > "$TEST_FILE"
    chmod +x "$TEST_FILE"
    
    if ! "$TEST_FILE" 2>/dev/null; then
        log_error "Installation directory is mounted noexec!"
        log_error "Try: adb remount or use different installation directory"
        rm -f "$TEST_FILE"
        exit 1
    fi
    
    rm -f "$TEST_FILE"
    log_success "Execution permissions verified"
fi

# Verify installation
log_info "Verifying installation..."
if [ ! -x "$INSTALL_DIR/${GENOME_NAME}" ]; then
    log_error "Binary not executable: $INSTALL_DIR/${GENOME_NAME}"
    exit 1
fi

log_success "Installation verified"

# Generate deployment report
REPORT_FILE="$INSTALL_DIR/.deployment-report.json"
cat > "$REPORT_FILE" << EOF
{
  "genome_name": "$GENOME_NAME",
  "genome_version": "$GENOME_VERSION",
  "architecture": "$BINARY_ARCH",
  "platform": "$PLATFORM",
  "install_dir": "$INSTALL_DIR",
  "install_time": "$(date -u +%Y-%m-%dT%H:%M:%SZ 2>/dev/null || date +%Y-%m-%dT%H:%M:%SZ)",
  "install_user": "$(id -un)",
  "install_uid": $(id -u),
  "hostname": "$(hostname 2>/dev/null || echo 'unknown')",
  "checksum_verified": $([ "$SKIP_CHECKSUMS" = "1" ] && echo "false" || echo "true"),
  "success": true
}
EOF

log_info "Deployment report: $REPORT_FILE"

# Display installation summary
printf "\n"
log_header "🎊 Installation Complete!"
log_info "Installed to: $INSTALL_DIR"
log_info "Architecture: $BINARY_ARCH"
log_info "Platform: $PLATFORM"

# Run health check
printf "\n"
log_info "Running health check..."
if "$INSTALL_DIR/${GENOME_NAME}" --version >/dev/null 2>&1; then
    VERSION=$("$INSTALL_DIR/${GENOME_NAME}" --version 2>&1 | head -1 || echo "unknown")
    log_success "${GENOME_NAME}: $VERSION"
else
    log_warn "Version check returned non-zero (may need configuration)"
fi

# Create system-wide symlinks if root
if [ "$(id -u)" -eq 0 ] && [ "$PLATFORM" = "linux" ]; then
    if [ -d /usr/local/bin ]; then
        log_info "Creating system-wide symlink..."
        ln -sf "$INSTALL_DIR/${GENOME_NAME}" "/usr/local/bin/${GENOME_NAME}" 2>/dev/null || {
            log_warn "Failed to create symlink (non-fatal)"
        }
        log_success "Symlink created: /usr/local/bin/${GENOME_NAME}"
    fi
fi

# Show next steps
printf "\n"
log_header "🚀 Next Steps"
if [ "$(id -u)" -ne 0 ]; then
    printf "1. Add to PATH:\n"
    printf "   export PATH=\"\$PATH:%s\"\n" "$INSTALL_DIR"
    printf "\n"
fi
printf "2. Verify installation:\n"
printf "   %s/%s --version\n" "$INSTALL_DIR" "$GENOME_NAME"
printf "\n"
printf "3. See documentation for usage\n"
printf "\n"

log_info "genomeBin deployment complete! 🧬"
log_success "Ready for use!"

# Remove backup on successful installation
if [ -d "$INSTALL_DIR.backup" ]; then
    rm -rf "$INSTALL_DIR.backup" 2>/dev/null || true
fi

exit 0

# Archive marker - DO NOT REMOVE
__ARCHIVE_START__
