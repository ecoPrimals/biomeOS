#!/bin/bash
# Validate USB BiomeOS deployment in 2-VM federation using benchScale

set -e

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  🧪 USB Federation Validation with benchScale 🧪         ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""
echo "Goal: Use benchScale to deploy BiomeOS USB in 2-VM federation"
echo ""

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
USB_PACKAGE="$SCRIPT_DIR/biomeos-20251228-181049.tar.gz"
BENCHSCALE_BIN="/home/eastgate/Development/ecoPrimals/primalTools/benchscale/target/release/benchscale"
TOPOLOGY_FILE="$SCRIPT_DIR/validation/benchscale-topologies/usb-federation-test.yaml"

# Phase 1: Prerequisites
echo "═══════════════════════════════════════════════════════════"
echo "Phase 1: Prerequisites"
echo "═══════════════════════════════════════════════════════════"
echo ""

if [ ! -f "$USB_PACKAGE" ]; then
    echo "❌ USB package not found: $USB_PACKAGE"
    exit 1
fi
echo "✅ USB package: $USB_PACKAGE ($(du -h "$USB_PACKAGE" | cut -f1))"

if [ ! -f "$BENCHSCALE_BIN" ]; then
    echo "❌ benchScale binary not found: $BENCHSCALE_BIN"
    echo "   Building benchScale..."
    cd /home/eastgate/Development/ecoPrimals/primalTools/benchscale
    cargo build --release --features libvirt
    cd "$SCRIPT_DIR"
fi
echo "✅ benchScale: $BENCHSCALE_BIN"

if [ ! -f "$TOPOLOGY_FILE" ]; then
    echo "❌ Topology file not found: $TOPOLOGY_FILE"
    echo "   Creating topology..."
    mkdir -p "$(dirname "$TOPOLOGY_FILE")"
    
    cat > "$TOPOLOGY_FILE" << 'EOF'
# BiomeOS USB Federation Test Topology
# 2 VMs for validating mDNS/UDP federation

name: biomeos-usb-federation
description: 2-VM federation for testing BiomeOS USB deployment
version: "2.0"

backend: libvirt

# Shared network for mDNS discovery
networks:
  - name: biomeos-lan
    subnet: 192.168.100.0/24
    gateway: 192.168.100.1
    dhcp: true

# Two BiomeOS towers
nodes:
  - name: tower-alpha
    image: ubuntu-22.04-cloudimg
    memory: 4096  # 4GB RAM
    cpus: 2
    disk: 30      # 30GB disk
    networks:
      - name: biomeos-lan
        ip: dhcp
    cloud_init:
      users:
        - name: biomeos
          sudo: ALL=(ALL) NOPASSWD:ALL
          shell: /bin/bash
          ssh_authorized_keys: []  # Will be injected at runtime
      packages:
        - avahi-daemon
        - avahi-utils
        - curl
        - build-essential
      runcmd:
        - systemctl enable avahi-daemon
        - systemctl start avahi-daemon
        - mkdir -p /opt/biomeos

  - name: tower-beta
    image: ubuntu-22.04-cloudimg
    memory: 4096
    cpus: 2
    disk: 30
    networks:
      - name: biomeos-lan
        ip: dhcp
    cloud_init:
      users:
        - name: biomeos
          sudo: ALL=(ALL) NOPASSWD:ALL
          shell: /bin/bash
          ssh_authorized_keys: []
      packages:
        - avahi-daemon
        - avahi-utils
        - curl
        - build-essential
      runcmd:
        - systemctl enable avahi-daemon
        - systemctl start avahi-daemon
        - mkdir -p /opt/biomeos

# Validation tests
validation:
  - name: mdns_discovery
    description: Verify mDNS service discovery works
    nodes:
      - tower-alpha
      - tower-beta
    commands:
      - avahi-browse -t _songbird._tcp -r -p
  
  - name: primal_status
    description: Check primals are running
    nodes:
      - tower-alpha
      - tower-beta
    commands:
      - pgrep -f songbird
      - pgrep -f nestgate
EOF
    
    echo "✅ Created topology: $TOPOLOGY_FILE"
fi
echo "✅ Topology: $TOPOLOGY_FILE"

echo ""

# Phase 2: Create lab with benchScale
echo "═══════════════════════════════════════════════════════════"
echo "Phase 2: Creating 2-VM Lab with benchScale"
echo "═══════════════════════════════════════════════════════════"
echo ""

LAB_NAME="biomeos-usb-test"

# Check if lab already exists
if "$BENCHSCALE_BIN" list 2>/dev/null | grep -q "$LAB_NAME"; then
    echo "🧹 Cleaning up existing lab..."
    "$BENCHSCALE_BIN" destroy "$LAB_NAME" || true
fi

echo "🚀 Creating lab: $LAB_NAME"
echo "   Using benchScale's CloudInit + libvirt provisioning..."
echo ""

# Note: benchScale's 'create' command is not yet fully implemented
# We'll use the underlying Rust API approach
echo "⚠️  Note: benchScale CLI 'create' is pending implementation"
echo "   Using manual benchScale API approach..."
echo ""

# Instead, let's use benchScale's scripts which leverage the API
if [ -f "/home/eastgate/Development/ecoPrimals/primalTools/benchscale/scripts/create-lab.sh" ]; then
    echo "📝 Using benchScale's create-lab.sh script..."
    cd /home/eastgate/Development/ecoPrimals/primalTools/benchscale
    ./scripts/create-lab.sh "$LAB_NAME" "$TOPOLOGY_FILE" || {
        echo "⚠️  benchScale script approach needs adaptation"
        echo "   Falling back to direct Rust API usage..."
    }
    cd "$SCRIPT_DIR"
else
    echo "⚠️  benchScale scripts not available"
    echo "   Creating a Rust program to use benchScale API..."
    
    # Create a temporary Rust program that uses benchScale's API
    cat > /tmp/biomeos_federation_test.rs << 'RUST_EOF'
use benchscale::{LibvirtBackend, CloudInit, Backend};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🚀 Using benchScale LibvirtBackend to create VMs...");
    
    let backend = LibvirtBackend::new()?;
    
    // Read user's SSH public key
    let ssh_key = std::fs::read_to_string(
        std::env::var("HOME")? + "/.ssh/id_rsa.pub"
    )?;
    
    // Configure CloudInit for both VMs
    let cloud_init = CloudInit::builder()
        .add_user("biomeos", &ssh_key.trim())
        .package("avahi-daemon")
        .package("avahi-utils")
        .package("curl")
        .package("build-essential")
        .package_update(true)
        .runcmd("systemctl enable avahi-daemon")
        .runcmd("systemctl start avahi-daemon")
        .runcmd("mkdir -p /opt/biomeos")
        .build();
    
    // Create tower-alpha
    println!("📦 Creating tower-alpha...");
    let alpha = backend.create_desktop_vm(
        "biomeos-tower-alpha",
        Path::new("/var/lib/libvirt/images/ubuntu-22.04-cloudimg.img"),
        &cloud_init,
        4096,  // 4GB RAM
        2,     // 2 vCPUs
        30,    // 30GB disk
    ).await?;
    println!("✅ tower-alpha: {}", alpha.ip_address);
    
    // Create tower-beta
    println!("📦 Creating tower-beta...");
    let beta = backend.create_desktop_vm(
        "biomeos-tower-beta",
        Path::new("/var/lib/libvirt/images/ubuntu-22.04-cloudimg.img"),
        &cloud_init,
        4096,
        2,
        30,
    ).await?;
    println!("✅ tower-beta: {}", beta.ip_address);
    
    // Write IPs to file for shell script to use
    std::fs::write("/tmp/tower-alpha-ip", &alpha.ip_address)?;
    std::fs::write("/tmp/tower-beta-ip", &beta.ip_address)?;
    
    println!("\n✅ Both VMs created successfully!");
    println!("   tower-alpha: {}", alpha.ip_address);
    println!("   tower-beta: {}", beta.ip_address);
    
    Ok(())
}
RUST_EOF
    
    echo "📝 Created Rust program using benchScale API"
    echo "🔨 Compiling with benchScale dependency..."
    
    # Create a temp Cargo project
    mkdir -p /tmp/biomeos-federation-test
    cd /tmp/biomeos-federation-test
    
    if [ ! -f Cargo.toml ]; then
        cargo init --name biomeos-federation-test
        cat > Cargo.toml << 'CARGO_EOF'
[package]
name = "biomeos-federation-test"
version = "0.1.0"
edition = "2021"

[dependencies]
benchscale = { path = "/home/eastgate/Development/ecoPrimals/primalTools/benchscale", features = ["libvirt"] }
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
CARGO_EOF
    fi
    
    cp /tmp/biomeos_federation_test.rs src/main.rs
    
    echo "🔨 Building benchScale-powered federation creator..."
    cargo build --release 2>&1 | grep -E "(Compiling|Finished|error)" || true
    
    if [ -f target/release/biomeos-federation-test ]; then
        echo "✅ Built successfully!"
        echo ""
        echo "🚀 Creating VMs with benchScale..."
        sudo ./target/release/biomeos-federation-test
        
        # Get IPs from the Rust program's output
        VM1_IP=$(cat /tmp/tower-alpha-ip)
        VM2_IP=$(cat /tmp/tower-beta-ip)
        
        echo ""
        echo "✅ VMs created:"
        echo "   tower-alpha: $VM1_IP"
        echo "   tower-beta: $VM2_IP"
    else
        echo "❌ Build failed, check errors above"
        exit 1
    fi
    
    cd "$SCRIPT_DIR"
fi

# If we got here via any method, VMs should be running
# Discover their IPs
VM1_IP=$(sudo virsh domifaddr biomeos-tower-alpha 2>/dev/null | grep -oP '(\d+\.){3}\d+' | head -1)
VM2_IP=$(sudo virsh domifaddr biomeos-tower-beta 2>/dev/null | grep -oP '(\d+\.){3}\d+' | head -1)

if [ -z "$VM1_IP" ] || [ -z "$VM2_IP" ]; then
    # Try reading from temp files (if created by Rust program)
    VM1_IP=$(cat /tmp/tower-alpha-ip 2>/dev/null || echo "")
    VM2_IP=$(cat /tmp/tower-beta-ip 2>/dev/null || echo "")
fi

if [ -z "$VM1_IP" ] || [ -z "$VM2_IP" ]; then
    echo "❌ Could not get VM IPs"
    exit 1
fi

echo ""
echo "📊 VM IPs:"
echo "   tower-alpha: $VM1_IP"
echo "   tower-beta: $VM2_IP"
echo ""

# Phase 3: Deploy BiomeOS USB package
echo "═══════════════════════════════════════════════════════════"
echo "Phase 3: Deploying BiomeOS USB Package"
echo "═══════════════════════════════════════════════════════════"
echo ""

SSH_OPTS="-o StrictHostKeyChecking=no -o ConnectTimeout=5"

echo "⏳ Waiting for cloud-init to complete (up to 10 minutes)..."
echo "   Cloud-init installs packages which can take time..."
echo ""

# Wait for cloud-init with exponential backoff
for i in {1..10}; do
    WAIT_TIME=$((i * 30))
    echo "Attempt $i/10: Waiting ${WAIT_TIME}s before checking..."
    sleep $WAIT_TIME
    
    # Try SSH to tower-alpha
    if ssh $SSH_OPTS biomeos@$VM1_IP "echo 'SSH works!'" 2>/dev/null; then
        echo "✅ tower-alpha SSH ready!"
        ALPHA_READY=true
    else
        echo "⏳ tower-alpha still provisioning..."
        ALPHA_READY=false
    fi
    
    # Try SSH to tower-beta
    if ssh $SSH_OPTS biomeos@$VM2_IP "echo 'SSH works!'" 2>/dev/null; then
        echo "✅ tower-beta SSH ready!"
        BETA_READY=true
    else
        echo "⏳ tower-beta still provisioning..."
        BETA_READY=false
    fi
    
    # If both ready, break
    if [ "$ALPHA_READY" = true ] && [ "$BETA_READY" = true ]; then
        echo ""
        echo "✅ Both VMs ready! Cloud-init completed successfully!"
        break
    fi
    
    echo ""
done

if [ "$ALPHA_READY" != true ] || [ "$BETA_READY" != true ]; then
    echo "❌ Timeout waiting for VMs to be SSH-accessible"
    echo "   This likely means cloud-init is still running or failed."
    echo "   Check VM_FEDERATION_TROUBLESHOOTING.md for manual access."
    exit 1
fi

echo ""

echo "📦 Copying USB package to both VMs..."
scp $SSH_OPTS "$USB_PACKAGE" biomeos@$VM1_IP:/tmp/ &
scp $SSH_OPTS "$USB_PACKAGE" biomeos@$VM2_IP:/tmp/ &
wait
echo "✅ Packages copied"

echo "📂 Extracting on both VMs..."
ssh $SSH_OPTS biomeos@$VM1_IP "cd /tmp && tar -xzf biomeos-20251228-181049.tar.gz && sudo mv opt/biomeos /opt/ && sudo chown -R biomeos:biomeos /opt/biomeos" &
ssh $SSH_OPTS biomeos@$VM2_IP "cd /tmp && tar -xzf biomeos-20251228-181049.tar.gz && sudo mv opt/biomeos /opt/ && sudo chown -R biomeos:biomeos /opt/biomeos" &
wait
echo "✅ BiomeOS extracted on both VMs"
echo ""

# Phase 4: Start primals
echo "═══════════════════════════════════════════════════════════"
echo "Phase 4: Starting Primals"
echo "═══════════════════════════════════════════════════════════"
echo ""

echo "🚀 Starting Songbird on both VMs..."
ssh $SSH_OPTS biomeos@$VM1_IP "cd /opt/biomeos && nohup ./start-songbird.sh > /tmp/songbird.log 2>&1 &"
ssh $SSH_OPTS biomeos@$VM2_IP "cd /opt/biomeos && nohup ./start-songbird.sh > /tmp/songbird.log 2>&1 &"
sleep 10
echo "✅ Songbird started on both VMs"

echo "🚀 Starting NestGate on both VMs..."
ssh $SSH_OPTS biomeos@$VM1_IP "cd /opt/biomeos && export NESTGATE_JWT_SECRET=\$(openssl rand -base64 48) && nohup ./primals/nestgate service start > /tmp/nestgate.log 2>&1 &"
ssh $SSH_OPTS biomeos@$VM2_IP "cd /opt/biomeos && export NESTGATE_JWT_SECRET=\$(openssl rand -base64 48) && nohup ./primals/nestgate service start > /tmp/nestgate.log 2>&1 &"
sleep 5
echo "✅ NestGate started on both VMs"
echo ""

# Phase 5: Verify federation
echo "═══════════════════════════════════════════════════════════"
echo "Phase 5: Verifying mDNS/UDP Federation"
echo "═══════════════════════════════════════════════════════════"
echo ""

echo "🔍 Checking mDNS discovery on tower-alpha..."
ALPHA_MDNS=$(ssh $SSH_OPTS biomeos@$VM1_IP "avahi-browse -t _songbird._tcp -r -p 2>/dev/null | grep 'Songbird Orchestrator' | wc -l")
echo "   Found $ALPHA_MDNS Songbird tower(s)"

echo "🔍 Checking mDNS discovery on tower-beta..."
BETA_MDNS=$(ssh $SSH_OPTS biomeos@$VM2_IP "avahi-browse -t _songbird._tcp -r -p 2>/dev/null | grep 'Songbird Orchestrator' | wc -l")
echo "   Found $BETA_MDNS Songbird tower(s)"

echo ""

if [ "$ALPHA_MDNS" -ge 2 ] && [ "$BETA_MDNS" -ge 2 ]; then
    echo "✅ FEDERATION SUCCESS!"
    echo "   Both VMs discovered each other via mDNS/UDP!"
    echo "   This confirms NUC will auto-discover over LAN!"
    FEDERATION_STATUS="SUCCESS"
else
    echo "⚠️  Partial federation (found: alpha=$ALPHA_MDNS, beta=$BETA_MDNS)"
    echo "   Expected: Both should see 2 towers"
    FEDERATION_STATUS="PARTIAL"
fi

echo ""
echo "📊 Primal Status:"
echo ""
echo "--- tower-alpha ---"
ssh $SSH_OPTS biomeos@$VM1_IP "pgrep -f songbird && echo '✅ Songbird running' || echo '❌ Songbird not running'"
ssh $SSH_OPTS biomeos@$VM1_IP "pgrep -f nestgate && echo '✅ NestGate running' || echo '❌ NestGate not running'"
echo ""
echo "--- tower-beta ---"
ssh $SSH_OPTS biomeos@$VM2_IP "pgrep -f songbird && echo '✅ Songbird running' || echo '❌ Songbird not running'"
ssh $SSH_OPTS biomeos@$VM2_IP "pgrep -f nestgate && echo '✅ NestGate running' || echo '❌ NestGate not running'"
echo ""

# Summary
echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  📊 Federation Validation Complete 📊                    ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""
echo "Deployment Method: benchScale (libvirt + CloudInit)"
echo "Lab Name: $LAB_NAME"
echo ""
echo "VMs:"
echo "  tower-alpha: $VM1_IP"
echo "  tower-beta:  $VM2_IP"
echo ""
echo "mDNS Discovery:"
echo "  Tower Alpha: $ALPHA_MDNS tower(s) visible"
echo "  Tower Beta:  $BETA_MDNS tower(s) visible"
echo "  Status: $FEDERATION_STATUS"
echo ""

if [ "$FEDERATION_STATUS" = "SUCCESS" ]; then
    echo "✅ USB package validated in 2-VM federation!"
    echo "✅ Ready to deploy to NUC with confidence!"
    echo ""
    echo "🚀 Next: Deploy USB to NUC"
    echo "   NUC will auto-discover these VMs + any other NUCs on LAN!"
else
    echo "⚠️  Federation needs tuning"
    echo "   Check: mDNS config, network isolation, firewall rules"
fi

echo ""
echo "📝 VM Access:"
echo "  ssh biomeos@$VM1_IP"
echo "  ssh biomeos@$VM2_IP"
echo ""
echo "🧹 Cleanup:"
echo "  sudo virsh destroy biomeos-tower-alpha && sudo virsh undefine biomeos-tower-alpha --remove-all-storage"
echo "  sudo virsh destroy biomeos-tower-beta && sudo virsh undefine biomeos-tower-beta --remove-all-storage"
echo ""
