#!/bin/bash

# biomeOS ISO Creation Script
# Creates a bootable biomeOS ISO with all components

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/build"
ISO_DIR="${BUILD_DIR}/iso"
OUTPUT_DIR="${PROJECT_ROOT}/dist"
ISO_NAME="biomeOS-$(date +%Y%m%d-%H%M%S).iso"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check dependencies
check_dependencies() {
    log_info "Checking dependencies..."
    
    local missing_deps=()
    
    # Check for required tools
    for dep in cargo rust-src genisoimage grub-mkrescue xorriso; do
        if ! command -v "$dep" &> /dev/null; then
            missing_deps+=("$dep")
        fi
    done
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_info "Please install:"
        log_info "  Ubuntu/Debian: sudo apt-get install build-essential cargo rust-src genisoimage grub-common grub-pc-bin xorriso"
        log_info "  Fedora/CentOS: sudo dnf install cargo rust-src genisoimage grub2-tools xorriso"
        log_info "  Arch Linux: sudo pacman -S cargo rust-src cdrtools grub xorriso"
        exit 1
    fi
    
    log_success "All dependencies found"
}

# Clean previous builds
clean_build() {
    log_info "Cleaning previous builds..."
    rm -rf "${BUILD_DIR}"
    rm -rf "${OUTPUT_DIR}"
    mkdir -p "${BUILD_DIR}"
    mkdir -p "${ISO_DIR}"
    mkdir -p "${OUTPUT_DIR}"
    log_success "Clean complete"
}

# Build biomeOS components
build_components() {
    log_info "Building biomeOS components..."
    
    cd "${PROJECT_ROOT}"
    
    # Build all components in release mode
    log_info "Building release binaries..."
    cargo build --release
    
    # Build UI component
    log_info "Building UI component..."
    cargo build --release --bin biomeos-ui
    
    # Build showcase if it exists
    if [ -d "showcase" ]; then
        log_info "Building showcase..."
        cargo build --release --bin showcase
    fi
    
    log_success "All components built successfully"
}

# Create ISO directory structure
create_iso_structure() {
    log_info "Creating ISO directory structure..."
    
    # Create directory structure
    mkdir -p "${ISO_DIR}/boot/grub"
    mkdir -p "${ISO_DIR}/biome"
    mkdir -p "${ISO_DIR}/biome/bin"
    mkdir -p "${ISO_DIR}/biome/templates"
    mkdir -p "${ISO_DIR}/biome/specs"
    mkdir -p "${ISO_DIR}/biome/examples"
    
    # Copy binaries
    log_info "Copying binaries..."
    cp "${PROJECT_ROOT}/target/release/biomeos-ui" "${ISO_DIR}/biome/bin/" || log_warning "biomeos-ui not found"
    cp "${PROJECT_ROOT}/target/release/demo" "${ISO_DIR}/biome/bin/" || log_warning "demo not found"
    
    if [ -f "${PROJECT_ROOT}/target/release/showcase" ]; then
        cp "${PROJECT_ROOT}/target/release/showcase" "${ISO_DIR}/biome/bin/"
    fi
    
    # Copy templates and specs
    log_info "Copying templates and specifications..."
    cp -r "${PROJECT_ROOT}/templates/"* "${ISO_DIR}/biome/templates/" || log_warning "Templates not found"
    cp -r "${PROJECT_ROOT}/specs/"* "${ISO_DIR}/biome/specs/" || log_warning "Specs not found"
    
    # Copy example biome.yaml files
    if [ -d "${PROJECT_ROOT}/specs/examples" ]; then
        cp "${PROJECT_ROOT}/specs/examples/"*.yaml "${ISO_DIR}/biome/examples/" || log_warning "Example YAML files not found"
    fi
    
    log_success "ISO directory structure created"
}

# Create GRUB configuration
create_grub_config() {
    log_info "Creating GRUB configuration..."
    
    cat > "${ISO_DIR}/boot/grub/grub.cfg" << 'EOF'
set timeout=10
set default=0

# biomeOS Main Menu
menuentry "biomeOS - Sovereignty-First Operating System" {
    echo "Starting biomeOS..."
    echo "Loading biomeOS components..."
    echo ""
    echo "biomeOS Features:"
    echo "  - 🐕 BearDog: Genetic Security Keys"
    echo "  - 🎼 Songbird: Service Mesh"
    echo "  - 🏰 NestGate: Sovereign Storage"
    echo "  - 🍄 Toadstool: Universal Runtime"
    echo "  - 🐿️ Squirrel: AI Agent Platform"
    echo ""
    echo "Starting biomeOS UI..."
    
    # Boot into minimal Linux environment
    linux /boot/vmlinuz root=/dev/ram0 init=/sbin/init
    initrd /boot/initrd.img
}

menuentry "biomeOS - Demo Mode" {
    echo "Starting biomeOS Demo..."
    echo "This will demonstrate biomeOS capabilities"
    echo ""
    echo "Demo includes:"
    echo "  - Biome YAML editing"
    echo "  - Primal orchestration"
    echo "  - Real-time monitoring"
    echo "  - Sovereignty features"
    echo ""
    
    # Boot demo
    linux /boot/vmlinuz root=/dev/ram0 init=/sbin/init demo=1
    initrd /boot/initrd.img
}

menuentry "biomeOS - YAML Editor" {
    echo "Starting biomeOS YAML Editor..."
    echo "Edit and create biome.yaml files"
    echo ""
    echo "Available templates:"
    echo "  - Basic Development"
    echo "  - AI Research"
    echo "  - Secure Enterprise"
    echo ""
    
    # Boot YAML editor
    linux /boot/vmlinuz root=/dev/ram0 init=/sbin/init yaml_editor=1
    initrd /boot/initrd.img
}

menuentry "biomeOS - Installation" {
    echo "biomeOS Installation"
    echo "Install biomeOS to your system"
    echo ""
    echo "Installation will:"
    echo "  - Create biomeOS partition"
    echo "  - Install all Primals"
    echo "  - Configure sovereignty features"
    echo "  - Set up genetic security keys"
    echo ""
    
    # Boot installer
    linux /boot/vmlinuz root=/dev/ram0 init=/sbin/init install=1
    initrd /boot/initrd.img
}

menuentry "Exit to Shell" {
    echo "Dropping to shell..."
    echo "Available commands:"
    echo "  - biomeos-ui    : Start the UI"
    echo "  - demo          : Run demo"
    echo "  - ls /biome     : List biomeOS files"
    echo ""
    
    # Boot to shell
    linux /boot/vmlinuz root=/dev/ram0 init=/bin/bash
    initrd /boot/initrd.img
}
EOF
    
    log_success "GRUB configuration created"
}

# Create init script
create_init_script() {
    log_info "Creating init script..."
    
    mkdir -p "${ISO_DIR}/sbin"
    
    cat > "${ISO_DIR}/sbin/init" << 'EOF'
#!/bin/bash

# biomeOS Init Script
echo "biomeOS - Sovereignty-First Operating System"
echo "============================================="
echo ""

# Set up environment
export PATH="/biome/bin:$PATH"
export BIOME_ROOT="/biome"
export BIOME_TEMPLATES="/biome/templates"
export BIOME_SPECS="/biome/specs"
export BIOME_EXAMPLES="/biome/examples"

# Create temp directories
mkdir -p /tmp
mkdir -p /var/tmp

# Check boot parameters
CMDLINE=$(cat /proc/cmdline 2>/dev/null || echo "")

if echo "$CMDLINE" | grep -q "demo=1"; then
    echo "Starting biomeOS Demo..."
    echo "Press Ctrl+C to exit demo"
    if [ -x "/biome/bin/demo" ]; then
        exec /biome/bin/demo
    else
        echo "Demo not available, starting UI..."
        exec /biome/bin/biomeos-ui
    fi
elif echo "$CMDLINE" | grep -q "yaml_editor=1"; then
    echo "Starting biomeOS YAML Editor..."
    if [ -x "/biome/bin/biomeos-ui" ]; then
        exec /biome/bin/biomeos-ui --yaml-editor
    else
        echo "YAML Editor not available"
        exec /bin/bash
    fi
elif echo "$CMDLINE" | grep -q "install=1"; then
    echo "Starting biomeOS Installation..."
    echo "This is a demonstration ISO. Installation not implemented yet."
    echo "Starting UI instead..."
    exec /biome/bin/biomeos-ui
else
    echo "Starting biomeOS UI..."
    echo "Available files:"
    ls -la /biome/bin/ 2>/dev/null || echo "No binaries found"
    echo ""
    echo "Templates:"
    ls -la /biome/templates/ 2>/dev/null || echo "No templates found"
    echo ""
    echo "Examples:"
    ls -la /biome/examples/ 2>/dev/null || echo "No examples found"
    echo ""
    
    if [ -x "/biome/bin/biomeos-ui" ]; then
        exec /biome/bin/biomeos-ui
    else
        echo "UI not available, dropping to shell..."
        exec /bin/bash
    fi
fi
EOF
    
    chmod +x "${ISO_DIR}/sbin/init"
    log_success "Init script created"
}

# Create minimal kernel and initrd (placeholder)
create_boot_files() {
    log_info "Creating boot files..."
    
    # For now, create placeholder files
    # In a real implementation, these would be actual kernel and initrd
    mkdir -p "${ISO_DIR}/boot"
    
    # Create placeholder kernel
    echo "biomeOS kernel placeholder" > "${ISO_DIR}/boot/vmlinuz"
    
    # Create placeholder initrd
    echo "biomeOS initrd placeholder" > "${ISO_DIR}/boot/initrd.img"
    
    log_warning "Created placeholder boot files. Replace with actual kernel and initrd for real ISO."
}

# Create ISO manifest
create_manifest() {
    log_info "Creating ISO manifest..."
    
    cat > "${ISO_DIR}/biome/MANIFEST.yaml" << EOF
# biomeOS ISO Manifest
apiVersion: biomeOS/v1
kind: ISO
metadata:
  name: "${ISO_NAME}"
  created: "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  version: "$(cd "${PROJECT_ROOT}" && git describe --tags --always 2>/dev/null || echo 'unknown')"
  build_host: "$(hostname)"
  builder: "$(whoami)"

contents:
  binaries:
    - name: "biomeos-ui"
      path: "/biome/bin/biomeos-ui"
      description: "biomeOS User Interface"
    - name: "demo"
      path: "/biome/bin/demo"
      description: "biomeOS Demo Application"

  templates:
    - name: "biome.yaml"
      path: "/biome/templates/biome.yaml"
      description: "Default biome template"

  examples:
    - name: "basic-development.biome.yaml"
      path: "/biome/examples/basic-development.biome.yaml"
      description: "Basic development environment"
    - name: "ai-research.biome.yaml"
      path: "/biome/examples/ai-research.biome.yaml"
      description: "AI research environment"
    - name: "secure-enterprise.biome.yaml"
      path: "/biome/examples/secure-enterprise.biome.yaml"
      description: "Secure enterprise environment"

  specifications:
    - name: "BIOME_YAML_SPECIFICATION.md"
      path: "/biome/specs/BIOME_YAML_SPECIFICATION.md"
      description: "Complete biome.yaml specification"

features:
  - "Sovereignty-first design"
  - "Genetic beardog keys"
  - "AI cat door protection"
  - "Crypto locks for security"
  - "Zero vendor lock-in"
  - "Real-time monitoring"
  - "YAML editing capabilities"
  - "Multi-Primal orchestration"

boot_options:
  - "Standard biomeOS"
  - "Demo mode"
  - "YAML editor"
  - "Installation mode"
  - "Shell access"
EOF
    
    log_success "ISO manifest created"
}

# Create the ISO
create_iso() {
    log_info "Creating ISO image..."
    
    # Create ISO using genisoimage/mkisofs
    if command -v genisoimage &> /dev/null; then
        genisoimage -o "${OUTPUT_DIR}/${ISO_NAME}" \
                   -b boot/grub/grub.cfg \
                   -no-emul-boot \
                   -boot-load-size 4 \
                   -boot-info-table \
                   -R -J -v \
                   -V "biomeOS" \
                   -A "biomeOS Sovereignty-First Operating System" \
                   -publisher "biomeOS Project" \
                   -preparer "biomeOS ISO Builder" \
                   "${ISO_DIR}"
    elif command -v mkisofs &> /dev/null; then
        mkisofs -o "${OUTPUT_DIR}/${ISO_NAME}" \
                -b boot/grub/grub.cfg \
                -no-emul-boot \
                -boot-load-size 4 \
                -boot-info-table \
                -R -J -v \
                -V "biomeOS" \
                -A "biomeOS Sovereignty-First Operating System" \
                -publisher "biomeOS Project" \
                -preparer "biomeOS ISO Builder" \
                "${ISO_DIR}"
    else
        log_error "Neither genisoimage nor mkisofs found"
        exit 1
    fi
    
    log_success "ISO created: ${OUTPUT_DIR}/${ISO_NAME}"
}

# Generate checksums
generate_checksums() {
    log_info "Generating checksums..."
    
    cd "${OUTPUT_DIR}"
    
    # Generate multiple checksums
    sha256sum "${ISO_NAME}" > "${ISO_NAME}.sha256"
    md5sum "${ISO_NAME}" > "${ISO_NAME}.md5"
    
    log_success "Checksums generated"
}

# Create README
create_readme() {
    log_info "Creating README..."
    
    cat > "${OUTPUT_DIR}/README.md" << EOF
# biomeOS ISO

## Overview
This is a bootable biomeOS ISO created on $(date).

## Features
- 🐕 **BearDog**: Genetic security keys with inverse scaling economics
- 🎼 **Songbird**: Service mesh and orchestration
- 🏰 **NestGate**: Sovereign storage with ZFS
- 🍄 **Toadstool**: Universal runtime for all workloads
- 🐿️ **Squirrel**: AI agent platform with MCP support

## Boot Options
1. **Standard biomeOS**: Full biomeOS experience
2. **Demo Mode**: Demonstration of capabilities
3. **YAML Editor**: Edit biome.yaml files
4. **Installation**: Install to system (placeholder)
5. **Shell**: Command line access

## Usage
1. Burn to DVD or create bootable USB
2. Boot from the media
3. Select desired boot option
4. Follow on-screen instructions

## Files
- \`${ISO_NAME}\`: Main ISO file
- \`${ISO_NAME}.sha256\`: SHA256 checksum
- \`${ISO_NAME}.md5\`: MD5 checksum

## Verification
\`\`\`bash
# Verify SHA256
sha256sum -c ${ISO_NAME}.sha256

# Verify MD5
md5sum -c ${ISO_NAME}.md5
\`\`\`

## System Requirements
- x86_64 architecture
- 2GB RAM minimum
- 4GB disk space for installation
- Boot from DVD/USB support

## Source Code
biomeOS is open source and available at the project repository.

## License
Licensed under AGPL-3.0 with Digital Sovereignty provisions.
EOF
    
    log_success "README created"
}

# Main execution
main() {
    log_info "Starting biomeOS ISO creation..."
    echo "=================================="
    
    check_dependencies
    clean_build
    build_components
    create_iso_structure
    create_grub_config
    create_init_script
    create_boot_files
    create_manifest
    create_iso
    generate_checksums
    create_readme
    
    echo ""
    echo "=================================="
    log_success "biomeOS ISO creation complete!"
    echo ""
    log_info "Created files:"
    ls -lh "${OUTPUT_DIR}/"
    echo ""
    log_info "ISO Size: $(du -h "${OUTPUT_DIR}/${ISO_NAME}" | cut -f1)"
    echo ""
    log_info "To test the ISO:"
    log_info "  qemu-system-x86_64 -cdrom \"${OUTPUT_DIR}/${ISO_NAME}\" -m 2048 -enable-kvm"
    echo ""
    log_info "To create bootable USB:"
    log_info "  sudo dd if=\"${OUTPUT_DIR}/${ISO_NAME}\" of=/dev/sdX bs=4M status=progress"
    log_info "  (Replace /dev/sdX with your USB device)"
}

# Run main function
main "$@" 

# biomeOS ISO Creation Script
# Creates a bootable biomeOS ISO with all components

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/build"
ISO_DIR="${BUILD_DIR}/iso"
OUTPUT_DIR="${PROJECT_ROOT}/dist"
ISO_NAME="biomeOS-$(date +%Y%m%d-%H%M%S).iso"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check dependencies
check_dependencies() {
    log_info "Checking dependencies..."
    
    local missing_deps=()
    
    # Check for required tools
    for dep in cargo rust-src genisoimage grub-mkrescue xorriso; do
        if ! command -v "$dep" &> /dev/null; then
            missing_deps+=("$dep")
        fi
    done
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_info "Please install:"
        log_info "  Ubuntu/Debian: sudo apt-get install build-essential cargo rust-src genisoimage grub-common grub-pc-bin xorriso"
        log_info "  Fedora/CentOS: sudo dnf install cargo rust-src genisoimage grub2-tools xorriso"
        log_info "  Arch Linux: sudo pacman -S cargo rust-src cdrtools grub xorriso"
        exit 1
    fi
    
    log_success "All dependencies found"
}

# Clean previous builds
clean_build() {
    log_info "Cleaning previous builds..."
    rm -rf "${BUILD_DIR}"
    rm -rf "${OUTPUT_DIR}"
    mkdir -p "${BUILD_DIR}"
    mkdir -p "${ISO_DIR}"
    mkdir -p "${OUTPUT_DIR}"
    log_success "Clean complete"
}

# Build biomeOS components
build_components() {
    log_info "Building biomeOS components..."
    
    cd "${PROJECT_ROOT}"
    
    # Build all components in release mode
    log_info "Building release binaries..."
    cargo build --release
    
    # Build UI component
    log_info "Building UI component..."
    cargo build --release --bin biomeos-ui
    
    # Build showcase if it exists
    if [ -d "showcase" ]; then
        log_info "Building showcase..."
        cargo build --release --bin showcase
    fi
    
    log_success "All components built successfully"
}

# Create ISO directory structure
create_iso_structure() {
    log_info "Creating ISO directory structure..."
    
    # Create directory structure
    mkdir -p "${ISO_DIR}/boot/grub"
    mkdir -p "${ISO_DIR}/biome"
    mkdir -p "${ISO_DIR}/biome/bin"
    mkdir -p "${ISO_DIR}/biome/templates"
    mkdir -p "${ISO_DIR}/biome/specs"
    mkdir -p "${ISO_DIR}/biome/examples"
    
    # Copy binaries
    log_info "Copying binaries..."
    cp "${PROJECT_ROOT}/target/release/biomeos-ui" "${ISO_DIR}/biome/bin/" || log_warning "biomeos-ui not found"
    cp "${PROJECT_ROOT}/target/release/demo" "${ISO_DIR}/biome/bin/" || log_warning "demo not found"
    
    if [ -f "${PROJECT_ROOT}/target/release/showcase" ]; then
        cp "${PROJECT_ROOT}/target/release/showcase" "${ISO_DIR}/biome/bin/"
    fi
    
    # Copy templates and specs
    log_info "Copying templates and specifications..."
    cp -r "${PROJECT_ROOT}/templates/"* "${ISO_DIR}/biome/templates/" || log_warning "Templates not found"
    cp -r "${PROJECT_ROOT}/specs/"* "${ISO_DIR}/biome/specs/" || log_warning "Specs not found"
    
    # Copy example biome.yaml files
    if [ -d "${PROJECT_ROOT}/specs/examples" ]; then
        cp "${PROJECT_ROOT}/specs/examples/"*.yaml "${ISO_DIR}/biome/examples/" || log_warning "Example YAML files not found"
    fi
    
    log_success "ISO directory structure created"
}

# Create GRUB configuration
create_grub_config() {
    log_info "Creating GRUB configuration..."
    
    cat > "${ISO_DIR}/boot/grub/grub.cfg" << 'EOF'
set timeout=10
set default=0

# biomeOS Main Menu
menuentry "biomeOS - Sovereignty-First Operating System" {
    echo "Starting biomeOS..."
    echo "Loading biomeOS components..."
    echo ""
    echo "biomeOS Features:"
    echo "  - 🐕 BearDog: Genetic Security Keys"
    echo "  - 🎼 Songbird: Service Mesh"
    echo "  - 🏰 NestGate: Sovereign Storage"
    echo "  - 🍄 Toadstool: Universal Runtime"
    echo "  - 🐿️ Squirrel: AI Agent Platform"
    echo ""
    echo "Starting biomeOS UI..."
    
    # Boot into minimal Linux environment
    linux /boot/vmlinuz root=/dev/ram0 init=/sbin/init
    initrd /boot/initrd.img
}

menuentry "biomeOS - Demo Mode" {
    echo "Starting biomeOS Demo..."
    echo "This will demonstrate biomeOS capabilities"
    echo ""
    echo "Demo includes:"
    echo "  - Biome YAML editing"
    echo "  - Primal orchestration"
    echo "  - Real-time monitoring"
    echo "  - Sovereignty features"
    echo ""
    
    # Boot demo
    linux /boot/vmlinuz root=/dev/ram0 init=/sbin/init demo=1
    initrd /boot/initrd.img
}

menuentry "biomeOS - YAML Editor" {
    echo "Starting biomeOS YAML Editor..."
    echo "Edit and create biome.yaml files"
    echo ""
    echo "Available templates:"
    echo "  - Basic Development"
    echo "  - AI Research"
    echo "  - Secure Enterprise"
    echo ""
    
    # Boot YAML editor
    linux /boot/vmlinuz root=/dev/ram0 init=/sbin/init yaml_editor=1
    initrd /boot/initrd.img
}

menuentry "biomeOS - Installation" {
    echo "biomeOS Installation"
    echo "Install biomeOS to your system"
    echo ""
    echo "Installation will:"
    echo "  - Create biomeOS partition"
    echo "  - Install all Primals"
    echo "  - Configure sovereignty features"
    echo "  - Set up genetic security keys"
    echo ""
    
    # Boot installer
    linux /boot/vmlinuz root=/dev/ram0 init=/sbin/init install=1
    initrd /boot/initrd.img
}

menuentry "Exit to Shell" {
    echo "Dropping to shell..."
    echo "Available commands:"
    echo "  - biomeos-ui    : Start the UI"
    echo "  - demo          : Run demo"
    echo "  - ls /biome     : List biomeOS files"
    echo ""
    
    # Boot to shell
    linux /boot/vmlinuz root=/dev/ram0 init=/bin/bash
    initrd /boot/initrd.img
}
EOF
    
    log_success "GRUB configuration created"
}

# Create init script
create_init_script() {
    log_info "Creating init script..."
    
    mkdir -p "${ISO_DIR}/sbin"
    
    cat > "${ISO_DIR}/sbin/init" << 'EOF'
#!/bin/bash

# biomeOS Init Script
echo "biomeOS - Sovereignty-First Operating System"
echo "============================================="
echo ""

# Set up environment
export PATH="/biome/bin:$PATH"
export BIOME_ROOT="/biome"
export BIOME_TEMPLATES="/biome/templates"
export BIOME_SPECS="/biome/specs"
export BIOME_EXAMPLES="/biome/examples"

# Create temp directories
mkdir -p /tmp
mkdir -p /var/tmp

# Check boot parameters
CMDLINE=$(cat /proc/cmdline 2>/dev/null || echo "")

if echo "$CMDLINE" | grep -q "demo=1"; then
    echo "Starting biomeOS Demo..."
    echo "Press Ctrl+C to exit demo"
    if [ -x "/biome/bin/demo" ]; then
        exec /biome/bin/demo
    else
        echo "Demo not available, starting UI..."
        exec /biome/bin/biomeos-ui
    fi
elif echo "$CMDLINE" | grep -q "yaml_editor=1"; then
    echo "Starting biomeOS YAML Editor..."
    if [ -x "/biome/bin/biomeos-ui" ]; then
        exec /biome/bin/biomeos-ui --yaml-editor
    else
        echo "YAML Editor not available"
        exec /bin/bash
    fi
elif echo "$CMDLINE" | grep -q "install=1"; then
    echo "Starting biomeOS Installation..."
    echo "This is a demonstration ISO. Installation not implemented yet."
    echo "Starting UI instead..."
    exec /biome/bin/biomeos-ui
else
    echo "Starting biomeOS UI..."
    echo "Available files:"
    ls -la /biome/bin/ 2>/dev/null || echo "No binaries found"
    echo ""
    echo "Templates:"
    ls -la /biome/templates/ 2>/dev/null || echo "No templates found"
    echo ""
    echo "Examples:"
    ls -la /biome/examples/ 2>/dev/null || echo "No examples found"
    echo ""
    
    if [ -x "/biome/bin/biomeos-ui" ]; then
        exec /biome/bin/biomeos-ui
    else
        echo "UI not available, dropping to shell..."
        exec /bin/bash
    fi
fi
EOF
    
    chmod +x "${ISO_DIR}/sbin/init"
    log_success "Init script created"
}

# Create minimal kernel and initrd (placeholder)
create_boot_files() {
    log_info "Creating boot files..."
    
    # For now, create placeholder files
    # In a real implementation, these would be actual kernel and initrd
    mkdir -p "${ISO_DIR}/boot"
    
    # Create placeholder kernel
    echo "biomeOS kernel placeholder" > "${ISO_DIR}/boot/vmlinuz"
    
    # Create placeholder initrd
    echo "biomeOS initrd placeholder" > "${ISO_DIR}/boot/initrd.img"
    
    log_warning "Created placeholder boot files. Replace with actual kernel and initrd for real ISO."
}

# Create ISO manifest
create_manifest() {
    log_info "Creating ISO manifest..."
    
    cat > "${ISO_DIR}/biome/MANIFEST.yaml" << EOF
# biomeOS ISO Manifest
apiVersion: biomeOS/v1
kind: ISO
metadata:
  name: "${ISO_NAME}"
  created: "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  version: "$(cd "${PROJECT_ROOT}" && git describe --tags --always 2>/dev/null || echo 'unknown')"
  build_host: "$(hostname)"
  builder: "$(whoami)"

contents:
  binaries:
    - name: "biomeos-ui"
      path: "/biome/bin/biomeos-ui"
      description: "biomeOS User Interface"
    - name: "demo"
      path: "/biome/bin/demo"
      description: "biomeOS Demo Application"

  templates:
    - name: "biome.yaml"
      path: "/biome/templates/biome.yaml"
      description: "Default biome template"

  examples:
    - name: "basic-development.biome.yaml"
      path: "/biome/examples/basic-development.biome.yaml"
      description: "Basic development environment"
    - name: "ai-research.biome.yaml"
      path: "/biome/examples/ai-research.biome.yaml"
      description: "AI research environment"
    - name: "secure-enterprise.biome.yaml"
      path: "/biome/examples/secure-enterprise.biome.yaml"
      description: "Secure enterprise environment"

  specifications:
    - name: "BIOME_YAML_SPECIFICATION.md"
      path: "/biome/specs/BIOME_YAML_SPECIFICATION.md"
      description: "Complete biome.yaml specification"

features:
  - "Sovereignty-first design"
  - "Genetic beardog keys"
  - "AI cat door protection"
  - "Crypto locks for security"
  - "Zero vendor lock-in"
  - "Real-time monitoring"
  - "YAML editing capabilities"
  - "Multi-Primal orchestration"

boot_options:
  - "Standard biomeOS"
  - "Demo mode"
  - "YAML editor"
  - "Installation mode"
  - "Shell access"
EOF
    
    log_success "ISO manifest created"
}

# Create the ISO
create_iso() {
    log_info "Creating ISO image..."
    
    # Create ISO using genisoimage/mkisofs
    if command -v genisoimage &> /dev/null; then
        genisoimage -o "${OUTPUT_DIR}/${ISO_NAME}" \
                   -b boot/grub/grub.cfg \
                   -no-emul-boot \
                   -boot-load-size 4 \
                   -boot-info-table \
                   -R -J -v \
                   -V "biomeOS" \
                   -A "biomeOS Sovereignty-First Operating System" \
                   -publisher "biomeOS Project" \
                   -preparer "biomeOS ISO Builder" \
                   "${ISO_DIR}"
    elif command -v mkisofs &> /dev/null; then
        mkisofs -o "${OUTPUT_DIR}/${ISO_NAME}" \
                -b boot/grub/grub.cfg \
                -no-emul-boot \
                -boot-load-size 4 \
                -boot-info-table \
                -R -J -v \
                -V "biomeOS" \
                -A "biomeOS Sovereignty-First Operating System" \
                -publisher "biomeOS Project" \
                -preparer "biomeOS ISO Builder" \
                "${ISO_DIR}"
    else
        log_error "Neither genisoimage nor mkisofs found"
        exit 1
    fi
    
    log_success "ISO created: ${OUTPUT_DIR}/${ISO_NAME}"
}

# Generate checksums
generate_checksums() {
    log_info "Generating checksums..."
    
    cd "${OUTPUT_DIR}"
    
    # Generate multiple checksums
    sha256sum "${ISO_NAME}" > "${ISO_NAME}.sha256"
    md5sum "${ISO_NAME}" > "${ISO_NAME}.md5"
    
    log_success "Checksums generated"
}

# Create README
create_readme() {
    log_info "Creating README..."
    
    cat > "${OUTPUT_DIR}/README.md" << EOF
# biomeOS ISO

## Overview
This is a bootable biomeOS ISO created on $(date).

## Features
- 🐕 **BearDog**: Genetic security keys with inverse scaling economics
- 🎼 **Songbird**: Service mesh and orchestration
- 🏰 **NestGate**: Sovereign storage with ZFS
- 🍄 **Toadstool**: Universal runtime for all workloads
- 🐿️ **Squirrel**: AI agent platform with MCP support

## Boot Options
1. **Standard biomeOS**: Full biomeOS experience
2. **Demo Mode**: Demonstration of capabilities
3. **YAML Editor**: Edit biome.yaml files
4. **Installation**: Install to system (placeholder)
5. **Shell**: Command line access

## Usage
1. Burn to DVD or create bootable USB
2. Boot from the media
3. Select desired boot option
4. Follow on-screen instructions

## Files
- \`${ISO_NAME}\`: Main ISO file
- \`${ISO_NAME}.sha256\`: SHA256 checksum
- \`${ISO_NAME}.md5\`: MD5 checksum

## Verification
\`\`\`bash
# Verify SHA256
sha256sum -c ${ISO_NAME}.sha256

# Verify MD5
md5sum -c ${ISO_NAME}.md5
\`\`\`

## System Requirements
- x86_64 architecture
- 2GB RAM minimum
- 4GB disk space for installation
- Boot from DVD/USB support

## Source Code
biomeOS is open source and available at the project repository.

## License
Licensed under AGPL-3.0 with Digital Sovereignty provisions.
EOF
    
    log_success "README created"
}

# Main execution
main() {
    log_info "Starting biomeOS ISO creation..."
    echo "=================================="
    
    check_dependencies
    clean_build
    build_components
    create_iso_structure
    create_grub_config
    create_init_script
    create_boot_files
    create_manifest
    create_iso
    generate_checksums
    create_readme
    
    echo ""
    echo "=================================="
    log_success "biomeOS ISO creation complete!"
    echo ""
    log_info "Created files:"
    ls -lh "${OUTPUT_DIR}/"
    echo ""
    log_info "ISO Size: $(du -h "${OUTPUT_DIR}/${ISO_NAME}" | cut -f1)"
    echo ""
    log_info "To test the ISO:"
    log_info "  qemu-system-x86_64 -cdrom \"${OUTPUT_DIR}/${ISO_NAME}\" -m 2048 -enable-kvm"
    echo ""
    log_info "To create bootable USB:"
    log_info "  sudo dd if=\"${OUTPUT_DIR}/${ISO_NAME}\" of=/dev/sdX bs=4M status=progress"
    log_info "  (Replace /dev/sdX with your USB device)"
}

# Run main function
main "$@" 