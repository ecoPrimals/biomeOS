#!/bin/bash

# Federation ISO Generator
# Packages BYOB manifests and Rust binaries into distributable ISO
# Pure Rust ecosystem - no external dependencies

set -e

# Configuration
ISO_NAME="ecoPrimals-federation-$(date +%Y%m%d).iso"
ISO_DIR="/tmp/federation-iso"
WORKSPACE_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VERSION="1.0.0"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}🔧 $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check dependencies
check_dependencies() {
    log_info "Checking dependencies..."
    
    # Check for required tools
    for cmd in cargo docker genisoimage; do
        if ! command -v $cmd &> /dev/null; then
            log_error "$cmd is required but not installed"
            exit 1
        fi
    done
    
    # Check Rust version
    local rust_version=$(rustc --version | cut -d' ' -f2)
    if [[ $(echo "$rust_version 1.70.0" | tr ' ' '\n' | sort -V | head -1) != "1.70.0" ]]; then
        log_error "Rust 1.70+ is required, found $rust_version"
        exit 1
    fi
    
    log_success "All dependencies satisfied"
}

# Create ISO directory structure
create_iso_structure() {
    log_info "Creating ISO directory structure..."
    
    rm -rf "$ISO_DIR"
    mkdir -p "$ISO_DIR"/{manifests,binaries,configs,docs,scripts}
    
    # Create metadata
    cat > "$ISO_DIR/metadata.toml" << EOF
# EcoPrimals Federation ISO Metadata
name = "ecoPrimals-federation"
version = "$VERSION"
build_date = "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
description = "Self-contained federation system for basement towers"
architecture = "x86_64"
rust_version = "$(rustc --version)"

[components]
songbird = "Federation coordination and orchestration"
beardog = "Gaming-grade security and networking"
nestgate = "Distributed storage with ZFS"
toadstool = "Container orchestration and compute"
squirrel = "AI/ML workloads with GPU acceleration"
biomeos = "Team workspace and BYOB management"

[manifests]
federation-demo = "Basic federation demonstration"
songbird-coordination = "Songbird coordination showcase"
federation-showcase = "Complete ecosystem demonstration"
EOF
    
    log_success "ISO structure created"
}

# Build Rust binaries
build_binaries() {
    log_info "Building Rust binaries..."
    
    cd "$WORKSPACE_ROOT"
    
    # Build all federation components
    local components=("songbird" "beardog" "nestgate" "toadstool" "squirrel" "biomeOS")
    
    for component in "${components[@]}"; do
        if [ -d "$component" ]; then
            log_info "Building $component..."
            
            cd "$component"
            cargo build --release --bin "${component}-federation" || {
                log_warning "No federation binary for $component, building default"
                cargo build --release || {
                    log_error "Failed to build $component"
                    exit 1
                }
            }
            
            # Copy binary to ISO
            local binary_name="${component}-federation"
            if [ -f "target/release/$binary_name" ]; then
                cp "target/release/$binary_name" "$ISO_DIR/binaries/"
                log_success "$component binary built and copied"
            else
                log_warning "$component federation binary not found, using default"
                if [ -f "target/release/$component" ]; then
                    cp "target/release/$component" "$ISO_DIR/binaries/"
                fi
            fi
            
            cd "$WORKSPACE_ROOT"
        fi
    done
    
    log_success "All binaries built"
}

# Copy BYOB manifests
copy_manifests() {
    log_info "Copying BYOB manifests..."
    
    # Copy federation manifests
    cp "$WORKSPACE_ROOT/biomeOS/templates/federation-demo.yaml" "$ISO_DIR/manifests/"
    cp "$WORKSPACE_ROOT/biomeOS/templates/songbird-coordination.yaml" "$ISO_DIR/manifests/"
    cp "$WORKSPACE_ROOT/biomeOS/templates/federation-showcase.yaml" "$ISO_DIR/manifests/"
    
    # Copy other useful manifests
    find "$WORKSPACE_ROOT/biomeOS/templates" -name "*.yaml" -exec cp {} "$ISO_DIR/manifests/" \;
    
    log_success "Manifests copied"
}

# Copy configuration files
copy_configs() {
    log_info "Copying configuration files..."
    
    # Copy Dockerfiles
    find "$WORKSPACE_ROOT" -name "Dockerfile.federation" -exec cp {} "$ISO_DIR/configs/" \;
    
    # Copy example configurations
    find "$WORKSPACE_ROOT" -name "*.toml" -path "*/config/*" -exec cp {} "$ISO_DIR/configs/" \;
    
    log_success "Configurations copied"
}

# Copy documentation
copy_docs() {
    log_info "Copying documentation..."
    
    # Copy key documentation files
    cp "$WORKSPACE_ROOT/ecoPrimals/SELF_CONTAINED_FEDERATION_SUMMARY.md" "$ISO_DIR/docs/"
    
    # Copy README files
    find "$WORKSPACE_ROOT" -name "README.md" -exec cp {} "$ISO_DIR/docs/" \;
    
    # Copy specification files
    find "$WORKSPACE_ROOT" -name "*.md" -path "*/specs/*" -exec cp {} "$ISO_DIR/docs/" \;
    
    log_success "Documentation copied"
}

# Create deployment scripts
create_scripts() {
    log_info "Creating deployment scripts..."
    
    # Create installation script
    cat > "$ISO_DIR/scripts/install.sh" << 'EOF'
#!/bin/bash

# EcoPrimals Federation Installation Script
# Installs federation components from ISO

set -e

echo "🌐 Installing EcoPrimals Federation System"
echo "========================================="

# Check if running as root
if [[ $EUID -eq 0 ]]; then
   echo "This script should not be run as root"
   exit 1
fi

# Create directories
sudo mkdir -p /opt/ecoprimal/{bin,config,data,logs}
sudo mkdir -p /etc/ecoprimal
sudo mkdir -p /var/lib/ecoprimal

# Copy binaries
sudo cp binaries/* /opt/ecoprimal/bin/
sudo chmod +x /opt/ecoprimal/bin/*

# Copy configurations
sudo cp configs/* /etc/ecoprimal/
sudo cp manifests/* /opt/ecoprimal/config/

# Create systemd services
sudo tee /etc/systemd/system/ecoprimal-federation.service > /dev/null << 'EOL'
[Unit]
Description=EcoPrimals Federation System
After=network.target

[Service]
Type=simple
User=ecoprimal
Group=ecoprimal
WorkingDirectory=/opt/ecoprimal
ExecStart=/opt/ecoprimal/bin/songbird-federation --config /etc/ecoprimal/federation.toml
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOL

# Create ecoprimal user
sudo useradd -r -s /bin/false ecoprimal || true
sudo chown -R ecoprimal:ecoprimal /opt/ecoprimal /var/lib/ecoprimal

# Enable and start service
sudo systemctl daemon-reload
sudo systemctl enable ecoprimal-federation.service

echo "✅ EcoPrimals Federation installed successfully!"
echo "   Start with: sudo systemctl start ecoprimal-federation"
echo "   Check status: sudo systemctl status ecoprimal-federation"
echo "   View logs: sudo journalctl -u ecoprimal-federation -f"
EOF

    # Create deployment script
    cat > "$ISO_DIR/scripts/deploy.sh" << 'EOF'
#!/bin/bash

# Federation Deployment Script
# Deploys BYOB manifests for federation demonstration

set -e

echo "🚀 Deploying EcoPrimals Federation Demos"
echo "======================================="

# Check if biome CLI is available
if ! command -v biome &> /dev/null; then
    echo "biome CLI not found. Installing..."
    if [ -f "/opt/ecoprimal/bin/biome" ]; then
        export PATH="/opt/ecoprimal/bin:$PATH"
    else
        echo "Please install biome CLI first"
        exit 1
    fi
fi

# Deploy federation demos
echo "Deploying federation demo..."
biome deploy manifests/federation-demo.yaml

echo "Deploying Songbird coordination..."
biome deploy manifests/songbird-coordination.yaml

echo "Deploying complete federation showcase..."
biome deploy manifests/federation-showcase.yaml

echo "✅ All federation demos deployed!"
echo "   Access dashboard: http://localhost:3000"
echo "   Check status: biome status"
echo "   View logs: biome logs"
EOF

    # Create uninstall script
    cat > "$ISO_DIR/scripts/uninstall.sh" << 'EOF'
#!/bin/bash

# EcoPrimals Federation Uninstall Script

set -e

echo "🗑️ Uninstalling EcoPrimals Federation System"
echo "==========================================="

# Stop and disable service
sudo systemctl stop ecoprimal-federation.service || true
sudo systemctl disable ecoprimal-federation.service || true

# Remove systemd service
sudo rm -f /etc/systemd/system/ecoprimal-federation.service

# Remove directories
sudo rm -rf /opt/ecoprimal
sudo rm -rf /etc/ecoprimal
sudo rm -rf /var/lib/ecoprimal

# Remove user
sudo userdel ecoprimal || true

# Reload systemd
sudo systemctl daemon-reload

echo "✅ EcoPrimals Federation uninstalled successfully!"
EOF

    # Make scripts executable
    chmod +x "$ISO_DIR/scripts"/*.sh
    
    log_success "Deployment scripts created"
}

# Create ISO image
create_iso() {
    log_info "Creating ISO image..."
    
    # Create README for ISO
    cat > "$ISO_DIR/README.txt" << EOF
EcoPrimals Federation ISO
========================

This ISO contains the complete EcoPrimals federation system for basement towers.

Contents:
- /manifests/     BYOB manifests for federation deployment
- /binaries/      Pre-built Rust binaries
- /configs/       Configuration files and Dockerfiles
- /docs/          Documentation and specifications
- /scripts/       Installation and deployment scripts

Quick Start:
1. Mount or extract this ISO
2. Run: sudo ./scripts/install.sh
3. Run: ./scripts/deploy.sh
4. Access: http://localhost:3000

For detailed documentation, see docs/SELF_CONTAINED_FEDERATION_SUMMARY.md

Build Information:
- Version: $VERSION
- Build Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)
- Architecture: x86_64
- Rust Version: $(rustc --version)

Support:
- GitHub: https://github.com/ecoprimal/stagingArea
- Documentation: https://docs.ecoprimal.com
- Issues: https://github.com/ecoprimal/stagingArea/issues
EOF
    
    # Create the ISO
    genisoimage -o "$ISO_NAME" \
        -V "EcoPrimals_Federation_$VERSION" \
        -R -J -l \
        -b isolinux/isolinux.bin \
        -c isolinux/boot.cat \
        -no-emul-boot \
        -boot-load-size 4 \
        -boot-info-table \
        "$ISO_DIR" || {
        # Fallback without boot sector for data-only ISO
        genisoimage -o "$ISO_NAME" \
            -V "EcoPrimals_Federation_$VERSION" \
            -R -J -l \
            "$ISO_DIR"
    }
    
    log_success "ISO created: $ISO_NAME"
}

# Generate checksums
generate_checksums() {
    log_info "Generating checksums..."
    
    sha256sum "$ISO_NAME" > "$ISO_NAME.sha256"
    md5sum "$ISO_NAME" > "$ISO_NAME.md5"
    
    log_success "Checksums generated"
}

# Main execution
main() {
    echo "🏗️ EcoPrimals Federation ISO Builder"
    echo "===================================="
    echo "Building ISO: $ISO_NAME"
    echo "Workspace: $WORKSPACE_ROOT"
    echo ""
    
    check_dependencies
    create_iso_structure
    build_binaries
    copy_manifests
    copy_configs
    copy_docs
    create_scripts
    create_iso
    generate_checksums
    
    echo ""
    echo "🎉 Federation ISO Build Complete!"
    echo "================================"
    echo ""
    echo "📦 ISO File: $ISO_NAME"
    echo "📊 Size: $(du -h "$ISO_NAME" | cut -f1)"
    echo "🔐 SHA256: $(cat "$ISO_NAME.sha256" | cut -d' ' -f1)"
    echo "🔐 MD5: $(cat "$ISO_NAME.md5" | cut -d' ' -f1)"
    echo ""
    echo "📋 Installation Instructions:"
    echo "1. Mount or extract the ISO"
    echo "2. Run: sudo ./scripts/install.sh"
    echo "3. Run: ./scripts/deploy.sh"
    echo "4. Access: http://localhost:3000"
    echo ""
    echo "🌐 Ready to distribute to basement towers!"
    
    # Cleanup
    rm -rf "$ISO_DIR"
}

# Run main function
main "$@" 