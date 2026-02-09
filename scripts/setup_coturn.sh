#!/bin/bash
# ═══════════════════════════════════════════════════════════════════════════════
# coturn Setup Script for biomeOS Self-Hosted STUN
# ═══════════════════════════════════════════════════════════════════════════════
#
# TEMPORARY: Using coturn while Songbird builds pure Rust STUN server
#
# Run with: sudo ./setup_coturn.sh
#
# After running:
#   1. Enable in /etc/default/coturn: TURNSERVER_ENABLED=1
#   2. Start: sudo systemctl start coturn
#   3. Test: stunclient localhost
#   4. Update config/stun/multi_tier.toml to enable self-hosted STUN

set -e

echo "╔═══════════════════════════════════════════════════════════════════════════╗"
echo "║              biomeOS Self-Hosted STUN Setup (coturn)                     ║"
echo "╚═══════════════════════════════════════════════════════════════════════════╝"

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root: sudo $0"
    exit 1
fi

echo ""
echo "=== Installing coturn ===" 
apt-get update
apt-get install -y coturn

echo ""
echo "=== Detecting network configuration ===" 
LOCAL_IP=$(hostname -I | awk '{print $1}')
PUBLIC_IP=$(curl -s ifconfig.me || echo "UNKNOWN")

echo "Local IP:  $LOCAL_IP"
echo "Public IP: $PUBLIC_IP"

echo ""
echo "=== Creating biomeOS coturn configuration ===" 

cat > /etc/turnserver.conf << EOF
# ═══════════════════════════════════════════════════════════════════════════════
# biomeOS coturn Configuration
# ═══════════════════════════════════════════════════════════════════════════════
# TEMPORARY: Bridge until Songbird has pure Rust STUN server
# Created: $(date -Iseconds)

# ─────────────────────────────────────────────────────────────────────────────
# Network Configuration
# ─────────────────────────────────────────────────────────────────────────────

# STUN/TURN listening port (standard)
listening-port=3478

# Listen on all interfaces (or specify)
listening-ip=$LOCAL_IP

# External IP for NAT traversal
# NOTE: Update this if your public IP changes
external-ip=$PUBLIC_IP/$LOCAL_IP

# Relay IP (for TURN relay functionality)
relay-ip=$LOCAL_IP

# ─────────────────────────────────────────────────────────────────────────────
# Security Configuration (Minimal for STUN-only)
# ─────────────────────────────────────────────────────────────────────────────

# Fingerprint for STUN compliance
fingerprint

# Realm for TURN authentication (required even for STUN)
realm=biomeos.local
server-name=tower.biomeos.local

# Log settings
log-file=/var/log/turnserver.log
verbose

# ─────────────────────────────────────────────────────────────────────────────
# STUN-Only Mode (No TURN credentials required)
# ─────────────────────────────────────────────────────────────────────────────

# Allow anonymous STUN binding requests
# This is safe - STUN only returns your public IP:port
no-auth

# Don't require authentication for STUN
allow-loopback-peers

# ─────────────────────────────────────────────────────────────────────────────
# TURN Relay (Optional - Enable for family relay)
# ─────────────────────────────────────────────────────────────────────────────

# Uncomment for TURN relay functionality (requires auth)
# lt-cred-mech
# user=biomeos:CHANGE_THIS_PASSWORD

# ─────────────────────────────────────────────────────────────────────────────
# Performance
# ─────────────────────────────────────────────────────────────────────────────

# Max relay sessions
total-quota=100

# Max data rate per session (bytes/sec)
bps-capacity=100000000

# Deny peer connections to local network (security)
denied-peer-ip=10.0.0.0-10.255.255.255
denied-peer-ip=172.16.0.0-172.31.255.255
denied-peer-ip=192.168.0.0-192.168.255.255

EOF

echo "Configuration written to /etc/turnserver.conf"

echo ""
echo "=== Enabling coturn service ===" 
sed -i 's/TURNSERVER_ENABLED=0/TURNSERVER_ENABLED=1/' /etc/default/coturn 2>/dev/null || \
    echo "TURNSERVER_ENABLED=1" >> /etc/default/coturn

echo ""
echo "=== Starting coturn ===" 
systemctl enable coturn
systemctl restart coturn

echo ""
echo "=== Checking status ===" 
systemctl status coturn --no-pager | head -10

echo ""
echo "╔═══════════════════════════════════════════════════════════════════════════╗"
echo "║                        SETUP COMPLETE                                    ║"
echo "╠═══════════════════════════════════════════════════════════════════════════╣"
echo "║                                                                           ║"
echo "║  coturn is now running on:                                               ║"
echo "║    Local:  $LOCAL_IP:3478                                      ║"
echo "║    Public: $PUBLIC_IP:3478 (if port forwarded)                 ║"
echo "║                                                                           ║"
echo "║  Test locally:                                                           ║"
echo "║    stunclient localhost 3478                                             ║"
echo "║                                                                           ║"
echo "║  Test via Songbird:                                                      ║"
echo "║    echo '{\"jsonrpc\":\"2.0\",\"method\":\"stun.get_public_address\",     ║"
echo "║      \"params\":{\"server\":\"$LOCAL_IP:3478\"},\"id\":1}' |    ║"
echo "║      nc -U /run/user/1000/biomeos/songbird-1894e909e454.sock                     ║"
echo "║                                                                           ║"
echo "║  Next: Update config/stun/multi_tier.toml to enable self-hosted STUN    ║"
echo "║                                                                           ║"
echo "╚═══════════════════════════════════════════════════════════════════════════╝"
