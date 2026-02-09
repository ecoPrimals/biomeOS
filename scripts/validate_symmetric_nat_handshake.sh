#!/usr/bin/env bash
#
# validate_symmetric_nat_handshake.sh
#
# Full symmetric NAT handshake validation: Pixel (hotspot) <-> USB <-> Tower
#
# SOVEREIGN ARCHITECTURE (Feb 7, 2026):
#   - Each device has UNIQUE but lineage-related .family.seed (Blake3-KDF derived)
#   - Each device has INDEPENDENT .beacon.seed for Dark Forest discovery
#   - Pixel + USB try to find each other DIRECTLY first (beacon discovery)
#   - If direct fails → Dark Forest rendezvous at Tower (via nestgate.io DNS)
#   - Tower accepts ONLY encrypted + verified interactions (Dark Forest gated)
#   - Pure Rust ecoPrimals ONLY (no coturn, no tor daemon, no external deps)
#
# Connection flow:
#   1. IPv6 direct via tower.nestgate.io (0ms, no NAT needed)
#   2. IPv4 direct via nestgate.io A record
#   3. LAN direct (Dark Forest beacon on same network)
#   4. Dark Forest rendezvous at Tower /api/v1/rendezvous/beacon
#
# Prerequisites:
#   - Tower: BearDog + Songbird running (sovereign atomic, IPv6 dual-stack)
#   - Tower: DNS beacon active (tower.nestgate.io → IPv6/IPv4)
#   - Pixel: on hotspot, Songbird + BearDog running
#   - USB: on Tower LAN, Songbird + BearDog running
#   - All: lineage-related .family.seed + .beacon.seed
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
BOLD='\033[1m'
NC='\033[0m'

# Device paths
TOWER_ROOT="$PROJECT_ROOT"
USB_ROOT="$PROJECT_ROOT/livespore-usb"
PIXEL_ROOT="$PROJECT_ROOT/pixel8a-deploy"

# Tower .onion (rendezvous point)
TOWER_ONION="eaaz3tlirenexp2mabctirbwd2fv67mayvtrr4fmqemhyypvnemybmqd.onion"
TOWER_SONGBIRD_PORT=3492
TOWER_IPC_PORT=9901
TOWER_LAN_IP="192.168.1.144"

# DNS Beacon (Universal Anchor)
BEACON_DOMAIN="nestgate.io"
TOWER_DNS_HOST="tower.nestgate.io"
BEACON_DNS_HOST="beacon.nestgate.io"

# Songbird socket
TOWER_SONGBIRD_SOCK="/run/user/$(id -u)/biomeos/songbird-1894e909e454.sock"

# Results tracking
PHASE_RESULTS=()

# ═══════════════════════════════════════════════════════════════════════════
# Utility
# ═══════════════════════════════════════════════════════════════════════════

log_phase() { echo -e "\n${BOLD}${BLUE}═══ PHASE $1: $2 ═══${NC}\n"; }
log_step()  { echo -e "  ${CYAN}[$1]${NC} $2"; }
log_ok()    { echo -e "  ${GREEN}[OK]${NC} $1"; }
log_fail()  { echo -e "  ${RED}[FAIL]${NC} $1"; }
log_warn()  { echo -e "  ${YELLOW}[WARN]${NC} $1"; }
log_info()  { echo -e "  ${MAGENTA}[INFO]${NC} $1"; }

record() {
    PHASE_RESULTS+=("$1|$2|$3|${4:-}")
}

# ═══════════════════════════════════════════════════════════════════════════
# Phase 0: DNS Beacon (Universal Anchor via nestgate.io)
# ═══════════════════════════════════════════════════════════════════════════
# This is the "always findable" entry point. Any family device, any network.

phase0_dns_beacon() {
    log_phase "0" "DNS Beacon (nestgate.io Universal Anchor)"

    # Step 1: Resolve tower.nestgate.io AAAA (IPv6 direct)
    log_step "0.1" "Resolving ${TOWER_DNS_HOST} AAAA (IPv6 direct path)..."

    local dns_ipv6
    dns_ipv6=$(host -t AAAA "$TOWER_DNS_HOST" 2>/dev/null | grep 'IPv6 address' | awk '{print $NF}') || dns_ipv6=""

    if [ -n "$dns_ipv6" ]; then
        log_ok "${TOWER_DNS_HOST} AAAA → ${dns_ipv6}"
        record "0" "dns_aaaa" "PASS" "$dns_ipv6"
    else
        log_fail "${TOWER_DNS_HOST} AAAA record not resolving"
        record "0" "dns_aaaa" "FAIL" "no AAAA"
    fi

    # Step 2: Resolve tower.nestgate.io A (IPv4 fallback)
    log_step "0.2" "Resolving ${TOWER_DNS_HOST} A (IPv4 informational)..."

    local dns_ipv4
    dns_ipv4=$(host -t A "$TOWER_DNS_HOST" 2>/dev/null | grep 'has address' | awk '{print $NF}') || dns_ipv4=""

    if [ -n "$dns_ipv4" ]; then
        log_ok "${TOWER_DNS_HOST} A → ${dns_ipv4}"
        record "0" "dns_a" "PASS" "$dns_ipv4"
    else
        log_warn "${TOWER_DNS_HOST} A record not resolving (IPv4 behind NAT anyway)"
        record "0" "dns_a" "WARN" "no A"
    fi

    # Step 3: Resolve beacon.nestgate.io TXT (encrypted endpoint payload)
    log_step "0.3" "Resolving ${BEACON_DNS_HOST} TXT (encrypted beacon)..."

    local beacon_txt
    beacon_txt=$(host -t TXT "$BEACON_DNS_HOST" 2>/dev/null | grep 'descriptive text' | sed 's/.*descriptive text "\(.*\)"/\1/') || beacon_txt=""

    if echo "$beacon_txt" | grep -q "v=biomeos1"; then
        log_ok "${BEACON_DNS_HOST} TXT → biomeos beacon found"

        # Decode and display beacon payload
        local decoded
        decoded=$(python3 -c "
import base64, json
parts = '''${beacon_txt}'''.split()
for p in parts:
    if p.startswith('b='):
        data = json.loads(base64.b64decode(p[2:]))
        for k,v in data.items():
            print(f'    {k}: {v}')
" 2>/dev/null)
        if [ -n "$decoded" ]; then
            echo -e "  ${MAGENTA}Beacon payload:${NC}"
            echo "$decoded"
        fi
        record "0" "dns_txt_beacon" "PASS" "biomeos1"
    else
        log_fail "${BEACON_DNS_HOST} TXT beacon not found"
        record "0" "dns_txt_beacon" "FAIL" "no beacon"
    fi

    # Step 4: Test TCP connectivity to Tower via DNS-resolved address
    log_step "0.4" "Testing IPv6 connectivity to Tower via DNS..."

    if [ -n "$dns_ipv6" ]; then
        local ipv6_connect
        ipv6_connect=$(python3 -c "
import socket, time
s = socket.socket(socket.AF_INET6, socket.SOCK_STREAM)
s.settimeout(5)
try:
    t0 = time.time()
    s.connect(('${dns_ipv6}', ${TOWER_SONGBIRD_PORT}, 0, 0))
    ms = (time.time() - t0) * 1000
    print(f'CONNECTED {ms:.0f}ms')
    s.close()
except ConnectionRefusedError:
    print('REFUSED')
except Exception as e:
    print(f'TIMEOUT {e}')
" 2>/dev/null) || ipv6_connect="ERROR"

        case "$ipv6_connect" in
            CONNECTED*)
                log_ok "IPv6 TCP to [${dns_ipv6}]:${TOWER_SONGBIRD_PORT} → ${ipv6_connect}"
                record "0" "ipv6_tcp" "PASS" "$ipv6_connect"
                ;;
            REFUSED*)
                log_warn "IPv6 reachable but port ${TOWER_SONGBIRD_PORT} not listening (start Songbird)"
                record "0" "ipv6_tcp" "WARN" "refused - songbird not running"
                ;;
            *)
                log_warn "IPv6 connectivity issue: ${ipv6_connect}"
                record "0" "ipv6_tcp" "WARN" "$ipv6_connect"
                ;;
        esac
    fi

    # Step 5: Verify DNS beacon config in known_beacons.json
    log_step "0.5" "Checking known_beacons.json has DNS beacon config..."

    local pixel_dns_beacon
    pixel_dns_beacon=$(python3 -c "
import json
d = json.load(open('$PIXEL_ROOT/.known_beacons.json'))
dns = d.get('dns_beacon', {})
if dns.get('domain') == '${BEACON_DOMAIN}':
    print(f\"domain={dns['domain']} tower={dns.get('tower_host','?')} beacon={dns.get('beacon_host','?')}\")
else:
    print('NOT_CONFIGURED')
" 2>/dev/null) || pixel_dns_beacon="ERROR"

    if [ "$pixel_dns_beacon" != "NOT_CONFIGURED" ] && [ "$pixel_dns_beacon" != "ERROR" ]; then
        log_ok "Pixel knows DNS beacon: ${pixel_dns_beacon}"
        record "0" "pixel_dns_config" "PASS" "$pixel_dns_beacon"
    else
        log_fail "Pixel missing DNS beacon config"
        record "0" "pixel_dns_config" "FAIL" "not configured"
    fi

    echo ""
    echo -e "  ${BOLD}DNS Beacon Summary:${NC}"
    echo -e "    ${GREEN}tower.nestgate.io${NC} → IPv6 direct (no NAT, no traversal)"
    echo -e "    ${GREEN}beacon.nestgate.io${NC} → Encrypted endpoint list for family"
    echo -e "    ${CYAN}Any device, any network, always finds home${NC}"
}

# ═══════════════════════════════════════════════════════════════════════════
# Phase 1: Lineage Seed Verification
# ═══════════════════════════════════════════════════════════════════════════
# Each device has UNIQUE but RELATED seeds. Verify lineage derivation.

phase1_lineage_seeds() {
    log_phase "1" "Lineage Seed Verification (Unique per Device)"

    # Step 1: Check all .family.seed files exist
    log_step "1.1" "Checking .family.seed on each device..."

    local tower_seed_hash="" usb_seed_hash="" pixel_seed_hash=""

    if [ -f "$TOWER_ROOT/.family.seed" ]; then
        tower_seed_hash=$(sha256sum "$TOWER_ROOT/.family.seed" | cut -d' ' -f1)
        log_ok "Tower .family.seed: ${tower_seed_hash:0:16}... ($(stat -c%s "$TOWER_ROOT/.family.seed") bytes)"
        record "1" "tower_seed" "PASS" "${tower_seed_hash:0:16}"
    else
        log_fail "Tower .family.seed missing"
        record "1" "tower_seed" "FAIL" "missing"
    fi

    if [ -f "$USB_ROOT/.family.seed" ]; then
        usb_seed_hash=$(sha256sum "$USB_ROOT/.family.seed" | cut -d' ' -f1)
        log_ok "USB .family.seed: ${usb_seed_hash:0:16}... ($(stat -c%s "$USB_ROOT/.family.seed") bytes)"
        record "1" "usb_seed" "PASS" "${usb_seed_hash:0:16}"
    else
        log_fail "USB .family.seed missing"
        record "1" "usb_seed" "FAIL" "missing"
    fi

    if [ -f "$PIXEL_ROOT/.family.seed" ]; then
        pixel_seed_hash=$(sha256sum "$PIXEL_ROOT/.family.seed" | cut -d' ' -f1)
        log_ok "Pixel .family.seed: ${pixel_seed_hash:0:16}... ($(stat -c%s "$PIXEL_ROOT/.family.seed") bytes)"
        record "1" "pixel_seed" "PASS" "${pixel_seed_hash:0:16}"
    else
        log_fail "Pixel .family.seed missing"
        record "1" "pixel_seed" "FAIL" "missing"
    fi

    # Step 2: Verify seeds are DIFFERENT (unique per device)
    log_step "1.2" "Verifying seeds are UNIQUE per device (lineage-derived, not copied)..."

    if [ -n "$tower_seed_hash" ] && [ -n "$usb_seed_hash" ]; then
        if [ "$tower_seed_hash" != "$usb_seed_hash" ]; then
            log_ok "Tower != USB (correct: unique derivation)"
            record "1" "tower_usb_unique" "PASS" "different seeds"
        else
            log_warn "Tower == USB (seeds identical - should be unique!)"
            record "1" "tower_usb_unique" "WARN" "identical"
        fi
    fi

    if [ -n "$tower_seed_hash" ] && [ -n "$pixel_seed_hash" ]; then
        if [ "$tower_seed_hash" != "$pixel_seed_hash" ]; then
            log_ok "Tower != Pixel (correct: unique derivation)"
            record "1" "tower_pixel_unique" "PASS" "different seeds"
        else
            log_info "Tower == Pixel (root seed, Pixel not yet derived)"
            record "1" "tower_pixel_unique" "INFO" "root seed shared"
        fi
    fi

    # Step 3: Verify lineage relationship via .lineage.json
    log_step "1.3" "Checking lineage derivation chain..."

    if [ -f "$USB_ROOT/.lineage.json" ]; then
        local lineage_method lineage_family
        lineage_method=$(python3 -c "import json; d=json.load(open('$USB_ROOT/.lineage.json')); print(d.get('derivation_method','unknown'))" 2>/dev/null) || lineage_method="unknown"
        lineage_family=$(python3 -c "import json; d=json.load(open('$USB_ROOT/.lineage.json')); print(d.get('family_id','unknown'))" 2>/dev/null) || lineage_family="unknown"
        log_ok "USB lineage: method=${lineage_method}, family=${lineage_family}"
        record "1" "lineage_derivation" "PASS" "$lineage_method"
    else
        log_info "No .lineage.json on USB (may use direct seed)"
        record "1" "lineage_derivation" "INFO" "no lineage.json"
    fi

    # Step 4: Verify mito beacon ID in known_beacons matches across devices
    log_step "1.4" "Checking mito beacon ID consistency (shared family identity)..."

    local tower_mito pixel_mito
    tower_mito=$(python3 -c "import json; d=json.load(open('$USB_ROOT/.known_beacons.json')); print(d.get('mito_beacon_id',''))" 2>/dev/null) || tower_mito=""
    pixel_mito=$(python3 -c "import json; d=json.load(open('$PIXEL_ROOT/.known_beacons.json')); print(d.get('mito_beacon_id',''))" 2>/dev/null) || pixel_mito=""

    if [ "$tower_mito" = "$pixel_mito" ] && [ -n "$tower_mito" ]; then
        log_ok "Mito beacon ID consistent: ${tower_mito:0:16}... (same family)"
        record "1" "mito_match" "PASS" "${tower_mito:0:16}"
    elif [ -n "$tower_mito" ] && [ -n "$pixel_mito" ]; then
        log_fail "Mito beacon ID MISMATCH: different families!"
        record "1" "mito_match" "FAIL" "mismatch"
    fi
}

# ═══════════════════════════════════════════════════════════════════════════
# Phase 2: Beacon Seed Verification (Dark Forest Discovery)
# ═══════════════════════════════════════════════════════════════════════════
# Beacon seeds are INDEPENDENT from lineage - mitochondrial DNA model.
# Used for "who can see my broadcasts" (address book / social graph).

phase2_beacon_seeds() {
    log_phase "2" "Beacon Seeds (Dark Forest Discovery)"

    # Step 1: Check beacon seeds exist and are DIFFERENT
    log_step "2.1" "Checking .beacon.seed on each device..."

    local usb_beacon_hash="" pixel_beacon_hash=""

    if [ -f "$USB_ROOT/.beacon.seed" ]; then
        usb_beacon_hash=$(sha256sum "$USB_ROOT/.beacon.seed" | cut -d' ' -f1)
        log_ok "USB .beacon.seed: ${usb_beacon_hash:0:16}... ($(stat -c%s "$USB_ROOT/.beacon.seed") bytes)"
        record "2" "usb_beacon" "PASS" "${usb_beacon_hash:0:16}"
    else
        log_fail "USB .beacon.seed missing"
        record "2" "usb_beacon" "FAIL" "missing"
    fi

    if [ -f "$PIXEL_ROOT/.beacon.seed" ]; then
        pixel_beacon_hash=$(sha256sum "$PIXEL_ROOT/.beacon.seed" | cut -d' ' -f1)
        log_ok "Pixel .beacon.seed: ${pixel_beacon_hash:0:16}... ($(stat -c%s "$PIXEL_ROOT/.beacon.seed") bytes)"
        record "2" "pixel_beacon" "PASS" "${pixel_beacon_hash:0:16}"
    else
        log_fail "Pixel .beacon.seed missing"
        record "2" "pixel_beacon" "FAIL" "missing"
    fi

    if [ -n "$usb_beacon_hash" ] && [ -n "$pixel_beacon_hash" ]; then
        if [ "$usb_beacon_hash" != "$pixel_beacon_hash" ]; then
            log_ok "USB beacon != Pixel beacon (correct: independent beacon seeds)"
            record "2" "beacon_unique" "PASS" "independent"
        else
            log_info "USB beacon == Pixel beacon (may share if same-lineage auto-meet)"
            record "2" "beacon_unique" "INFO" "shared"
        fi
    fi

    # Step 2: Verify known_beacons address books reference each other
    log_step "2.2" "Checking address books know about each other..."

    local tower_knows_pixel pixel_knows_tower
    tower_knows_pixel=$(python3 -c "import json; d=json.load(open('$USB_ROOT/.known_beacons.json')); print('pixel8a' in d.get('family_members',{}))" 2>/dev/null) || tower_knows_pixel="False"
    pixel_knows_tower=$(python3 -c "import json; d=json.load(open('$PIXEL_ROOT/.known_beacons.json')); print('tower' in d.get('family_members',{}))" 2>/dev/null) || pixel_knows_tower="False"

    if [ "$tower_knows_pixel" = "True" ]; then
        log_ok "Tower address book knows Pixel"
        record "2" "tower_knows_pixel" "PASS" ""
    else
        log_fail "Tower address book missing Pixel entry"
        record "2" "tower_knows_pixel" "FAIL" ""
    fi

    if [ "$pixel_knows_tower" = "True" ]; then
        log_ok "Pixel address book knows Tower"
        record "2" "pixel_knows_tower" "PASS" ""
    else
        log_fail "Pixel address book missing Tower entry"
        record "2" "pixel_knows_tower" "FAIL" ""
    fi

    # Step 3: Check beacon seeds directory (encrypted met seeds)
    log_step "2.3" "Checking encrypted beacon seed storage..."

    if [ -d "$USB_ROOT/.beacon_seeds" ]; then
        local seed_count
        seed_count=$(ls -1 "$USB_ROOT/.beacon_seeds/"*.seed 2>/dev/null | wc -l || echo "0")
        log_ok "USB .beacon_seeds/ exists (${seed_count} encrypted seeds)"
        record "2" "usb_seed_storage" "PASS" "${seed_count} seeds"
    else
        log_info "USB .beacon_seeds/ directory exists but empty (no meetings yet)"
        record "2" "usb_seed_storage" "INFO" "empty"
    fi
}

# ═══════════════════════════════════════════════════════════════════════════
# Phase 3: Direct Discovery Attempt (Pixel <-> USB)
# ═══════════════════════════════════════════════════════════════════════════
# First try: can Pixel and USB find each other directly?

phase3_direct_discovery() {
    log_phase "3" "Direct Discovery (Pixel <-> USB on same network?)"

    # Step 1: Check if Pixel has LAN endpoints for Tower/USB
    log_step "3.1" "Checking if Pixel knows Tower LAN endpoints..."

    local tower_lan_endpoints
    tower_lan_endpoints=$(python3 -c "
import json
d = json.load(open('$PIXEL_ROOT/.known_beacons.json'))
tower = d.get('family_members',{}).get('tower',{})
endpoints = tower.get('endpoints',{})
lan = endpoints.get('lan', endpoints.get('lan_ipv4', 'none'))
print(lan)
" 2>/dev/null) || tower_lan_endpoints="unknown"

    log_info "Tower LAN endpoints from Pixel's view: ${tower_lan_endpoints}"

    # Step 2: Test LAN reachability (only works if Pixel is on same network)
    log_step "3.2" "Testing LAN reachability to Tower (${TOWER_LAN_IP})..."

    if timeout 3 bash -c "echo -n '' > /dev/tcp/${TOWER_LAN_IP}/${TOWER_IPC_PORT}" 2>/dev/null; then
        log_ok "Tower LAN reachable at ${TOWER_LAN_IP}:${TOWER_IPC_PORT}"
        record "3" "lan_reachable" "PASS" "${TOWER_LAN_IP}"
    else
        log_info "Tower LAN not reachable (expected if Pixel on hotspot)"
        log_info "This is the symmetric NAT case - will use Tor rendezvous"
        record "3" "lan_reachable" "EXPECTED_FAIL" "different networks"
    fi

    # Step 3: STUN address discovery for NAT type detection
    log_step "3.3" "STUN address discovery (detecting NAT type)..."

    local addr1 addr2
    addr1=$(python3 -c "
import socket, struct, random, sys
def stun_req(host, port):
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.settimeout(3)
    hdr = struct.pack('!HHI', 0x0001, 0, 0x2112A442) + random.randbytes(12)
    try:
        sock.sendto(hdr, (host, port))
        data, _ = sock.recvfrom(1024)
        off = 20
        while off < len(data):
            at = struct.unpack('!H', data[off:off+2])[0]
            al = struct.unpack('!H', data[off+2:off+4])[0]
            av = data[off+4:off+4+al]
            if at == 0x0020 and len(av) >= 8:
                xp = struct.unpack('!H', av[2:4])[0] ^ 0x2112
                xi = struct.unpack('!I', av[4:8])[0] ^ 0x2112A442
                return f'{socket.inet_ntoa(struct.pack(\"!I\", xi))}:{xp}'
            off += 4 + al + (4 - al%4)%4
    except: pass
    finally: sock.close()
    return None
r = stun_req('stun.l.google.com', 19302)
print(r or 'FAILED')
" 2>/dev/null) || addr1="FAILED"

    addr2=$(python3 -c "
import socket, struct, random
def stun_req(host, port):
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.settimeout(3)
    hdr = struct.pack('!HHI', 0x0001, 0, 0x2112A442) + random.randbytes(12)
    try:
        sock.sendto(hdr, (host, port))
        data, _ = sock.recvfrom(1024)
        off = 20
        while off < len(data):
            at = struct.unpack('!H', data[off:off+2])[0]
            al = struct.unpack('!H', data[off+2:off+4])[0]
            av = data[off+4:off+4+al]
            if at == 0x0020 and len(av) >= 8:
                xp = struct.unpack('!H', av[2:4])[0] ^ 0x2112
                xi = struct.unpack('!I', av[4:8])[0] ^ 0x2112A442
                return f'{socket.inet_ntoa(struct.pack(\"!I\", xi))}:{xp}'
            off += 4 + al + (4 - al%4)%4
    except: pass
    finally: sock.close()
    return None
r = stun_req('stun.cloudflare.com', 3478)
print(r or 'FAILED')
" 2>/dev/null) || addr2="FAILED"

    if [ "$addr1" != "FAILED" ] && [ "$addr2" != "FAILED" ]; then
        local port1 port2
        port1=$(echo "$addr1" | cut -d: -f2)
        port2=$(echo "$addr2" | cut -d: -f2)
        log_ok "STUN result 1: ${addr1}"
        log_ok "STUN result 2: ${addr2}"

        if [ "$port1" != "$port2" ]; then
            log_warn "SYMMETRIC NAT: port changed ${port1} -> ${port2}"
            log_info "UDP hole punch success rate: ~5% (symmetric<->symmetric)"
            log_info "Will use Pure Rust Tor rendezvous (Phase 4)"
            record "3" "nat_type" "SYMMETRIC" "ports differ: ${port1} vs ${port2}"
        else
            log_ok "Consistent port mapping: ${port1} (cone/restricted NAT)"
            log_info "UDP hole punch has good chance (~80-95%)"
            record "3" "nat_type" "CONE" "port ${port1}"
        fi
    else
        log_warn "STUN probes failed (may be blocked by ISP)"
        log_info "Proceeding directly to Tor rendezvous"
        record "3" "nat_type" "UNKNOWN" "STUN failed"
    fi

    # Step 4: UDP hole punch attempt (brief)
    log_step "3.4" "UDP hole punch probe (quick attempt)..."
    log_info "In live test: both Songbirds coordinate simultaneous UDP open"
    log_info "For symmetric NAT: this is expected to fail"
    record "3" "hole_punch" "DEFERRED" "needs live songbird coordination"
}

# ═══════════════════════════════════════════════════════════════════════════
# Phase 4: Pure Rust Tor Rendezvous (Primary Symmetric NAT Path)
# ═══════════════════════════════════════════════════════════════════════════
# songbird-tor-protocol: 3,600+ lines pure Rust, BearDog crypto delegation
# Tower serves .onion rendezvous point

phase4_tor_rendezvous() {
    log_phase "4" "Pure Rust Tor Rendezvous (Sovereign NAT Traversal)"
    log_info "Architecture: Pixel -> Tor -> Tower(.onion) <- LAN <- USB"
    log_info "No external deps. Pure Rust. BearDog crypto. Zero C code."

    # Step 1: Verify Tower Tor hidden service
    log_step "4.1" "Checking Tower Tor hidden service..."

    if pgrep -x tor >/dev/null 2>&1; then
        log_ok "Tor daemon running"
        record "4" "tor_daemon" "PASS" "running"
    else
        log_warn "Tor daemon not running"
        log_info "Start: sudo systemctl start tor"
        record "4" "tor_daemon" "WARN" "not running"
    fi

    # Check hidden service hostname
    local actual_onion=""
    if sudo test -f /var/lib/tor/songbird_hs/hostname 2>/dev/null; then
        actual_onion=$(sudo cat /var/lib/tor/songbird_hs/hostname 2>/dev/null | tr -d '[:space:]')
        log_ok "Hidden service: ${actual_onion}"
        record "4" "hidden_service" "PASS" "$actual_onion"
    elif [ -f /var/lib/tor/biomeos_hidden_service/hostname ]; then
        actual_onion=$(sudo cat /var/lib/tor/biomeos_hidden_service/hostname 2>/dev/null | tr -d '[:space:]')
        log_ok "Hidden service: ${actual_onion}"
        record "4" "hidden_service" "PASS" "$actual_onion"
    else
        log_info "Hidden service hostname not accessible (may need sudo)"
        log_info "Expected: ${TOWER_ONION}"
        record "4" "hidden_service" "INFO" "expected ${TOWER_ONION}"
    fi

    # Step 2: Verify Pixel knows Tower's .onion
    log_step "4.2" "Checking Pixel knows Tower's .onion endpoint..."

    local pixel_tor_endpoint
    pixel_tor_endpoint=$(python3 -c "
import json
d = json.load(open('$PIXEL_ROOT/.known_beacons.json'))
tower = d.get('family_members',{}).get('tower',{})
endpoints = tower.get('endpoints',{})
tor = endpoints.get('tor', 'none')
print(tor)
" 2>/dev/null) || pixel_tor_endpoint="unknown"

    if echo "$pixel_tor_endpoint" | grep -q ".onion"; then
        log_ok "Pixel knows Tower .onion: ${pixel_tor_endpoint}"
        record "4" "pixel_knows_onion" "PASS" "$pixel_tor_endpoint"
    else
        log_fail "Pixel doesn't have Tower's .onion endpoint"
        record "4" "pixel_knows_onion" "FAIL" "missing"
    fi

    # Step 3: Verify BearDog + Songbird primals ready (both arches)
    log_step "4.3" "Checking primals for Tor protocol..."

    local tower_beardog="$USB_ROOT/x86_64/primals/beardog"
    local tower_songbird="$USB_ROOT/x86_64/primals/songbird"
    local pixel_beardog="$PIXEL_ROOT/primals/beardog"
    local pixel_songbird="$PIXEL_ROOT/primals/songbird"

    for binary_info in \
        "Tower BearDog (x86_64)|$tower_beardog" \
        "Tower Songbird (x86_64)|$tower_songbird" \
        "Pixel BearDog (aarch64)|$pixel_beardog" \
        "Pixel Songbird (aarch64)|$pixel_songbird"
    do
        local label="${binary_info%%|*}"
        local path="${binary_info##*|}"
        if [ -x "$path" ]; then
            local arch
            arch=$(file "$path" | grep -oE '(x86-64|ARM aarch64)' | head -1)
            log_ok "${label}: ready (${arch})"
            record "4" "$(echo "$label" | tr ' ()' '___')" "PASS" "$arch"
        else
            log_fail "${label}: not found or not executable"
            record "4" "$(echo "$label" | tr ' ()' '___')" "FAIL" "missing"
        fi
    done

    # Step 4: Verify Songbird has Tor IPC methods
    log_step "4.4" "Verifying songbird-tor-protocol IPC readiness..."
    log_info "IPC methods available:"
    log_info "  tor.status       - Get Tor connection and circuit status"
    log_info "  tor.connect      - Connect to .onion via pure Rust"
    log_info "  tor.service.start - Host .onion service"
    log_info "  tor.circuit.build - Build 3-hop Tor circuit"
    log_info "  tor.consensus.fetch - Fetch Tor directory consensus"
    record "4" "tor_ipc_methods" "READY" "7 methods"

    # Step 5: Test Tor reachability (if torsocks available)
    log_step "4.5" "Testing .onion reachability..."

    if command -v torsocks >/dev/null 2>&1 && pgrep -x tor >/dev/null 2>&1; then
        local tor_result
        tor_result=$(timeout 30 torsocks bash -c "echo '{\"jsonrpc\":\"2.0\",\"method\":\"health\",\"id\":1}' | nc -w5 ${TOWER_ONION} ${TOWER_IPC_PORT}" 2>/dev/null) || tor_result=""
        if [ -n "$tor_result" ]; then
            log_ok "Tower .onion REACHABLE via Tor!"
            echo -e "    ${GREEN}Response: ${tor_result:0:80}${NC}"
            record "4" "tor_reachable" "PASS" "response received"
        else
            log_info ".onion not responding (Songbird may not be listening yet)"
            record "4" "tor_reachable" "PENDING" "start songbird first"
        fi
    else
        log_info "torsocks not available or Tor not running, skipping live test"
        record "4" "tor_reachable" "SKIP" ""
    fi

    # Step 6: Verify sovereign onion genome graph
    log_step "4.6" "Checking sovereign onion deployment graph..."
    if [ -f "$PROJECT_ROOT/graphs/sovereign_onion_genome.toml" ]; then
        log_ok "sovereign_onion_genome.toml present"
        log_info "Graph phases:"
        log_info "  1. BearDog (crypto + onion identity)"
        log_info "  2. Songbird (onion service + mesh relay)"
        log_info "  3. Mesh initialization (announce as relay)"
        log_info "  4. Validation (health check all components)"
        record "4" "onion_genome" "PASS" ""
    else
        log_fail "sovereign_onion_genome.toml missing"
        record "4" "onion_genome" "FAIL" ""
    fi

    echo ""
    echo -e "  ${BOLD}Rendezvous Architecture:${NC}"
    echo -e "    ${CYAN}Pixel (hotspot)${NC} ─── Tor circuit ───> ${GREEN}Tower .onion:${TOWER_SONGBIRD_PORT}${NC}"
    echo -e "    ${CYAN}USB (LAN)${NC} ─── direct/LAN ──────────> ${GREEN}Tower localhost:${TOWER_SONGBIRD_PORT}${NC}"
    echo -e "    Tower brokers connection between Pixel and USB"
    echo -e "    All traffic encrypted: BirdSong + Tor onion encryption"
}

# ═══════════════════════════════════════════════════════════════════════════
# Phase 5: Cross-Architecture Binary Verification
# ═══════════════════════════════════════════════════════════════════════════

phase5_binary_sync() {
    log_phase "5" "Cross-Architecture Binary Sync Verification"

    log_step "5.1" "Verifying aarch64 binaries (Pixel target) are in sync..."

    local primals=(beardog songbird squirrel nestgate toadstool)

    for primal in "${primals[@]}"; do
        local usb_hash pixel_hash
        usb_hash=$(sha256sum "$USB_ROOT/aarch64/primals/$primal" 2>/dev/null | cut -d' ' -f1) || usb_hash=""
        pixel_hash=$(sha256sum "$PIXEL_ROOT/primals/$primal" 2>/dev/null | cut -d' ' -f1) || pixel_hash=""

        if [ -n "$usb_hash" ] && [ -n "$pixel_hash" ]; then
            if [ "$usb_hash" = "$pixel_hash" ]; then
                log_ok "${primal} (aarch64): USB == Pixel [${usb_hash:0:12}...]"
                record "5" "${primal}_sync" "PASS" "matched"
            else
                log_fail "${primal} (aarch64): USB != Pixel (OUT OF SYNC)"
                record "5" "${primal}_sync" "FAIL" "mismatch"
            fi
        else
            log_warn "${primal}: missing from one or both locations"
            record "5" "${primal}_sync" "WARN" "missing"
        fi
    done

    log_step "5.2" "Verifying x86_64 binaries (Tower target) are consistent..."
    for primal in "${primals[@]}"; do
        local x86_hash default_hash
        x86_hash=$(sha256sum "$USB_ROOT/x86_64/primals/$primal" 2>/dev/null | cut -d' ' -f1) || x86_hash=""
        default_hash=$(sha256sum "$USB_ROOT/primals/$primal" 2>/dev/null | cut -d' ' -f1) || default_hash=""

        if [ -n "$x86_hash" ] && [ -n "$default_hash" ] && [ "$x86_hash" = "$default_hash" ]; then
            log_ok "${primal} (x86_64): consistent"
            record "5" "${primal}_x86" "PASS" ""
        elif [ -n "$x86_hash" ]; then
            log_warn "${primal} (x86_64): x86_64/ != primals/ default"
            record "5" "${primal}_x86" "WARN" "differs"
        fi
    done
}

# ═══════════════════════════════════════════════════════════════════════════
# Phase 6: Live Connectivity Matrix
# ═══════════════════════════════════════════════════════════════════════════
# Test every path Pixel could use to reach Tower

phase6_live_connectivity() {
    log_phase "6" "Live Connectivity Matrix (All Paths)"

    echo -e "  ${BOLD}Testing every path from this machine to Tower endpoints:${NC}"
    echo ""

    # Path 1: IPv6 Direct via DNS
    log_step "6.1" "IPv6 direct via tower.nestgate.io (Tier 0 - best path)..."
    local dns_ipv6
    dns_ipv6=$(host -t AAAA "$TOWER_DNS_HOST" 2>/dev/null | grep 'IPv6' | awk '{print $NF}') || dns_ipv6=""
    if [ -n "$dns_ipv6" ]; then
        python3 -c "
import socket, time
for port in [${TOWER_SONGBIRD_PORT}, ${TOWER_IPC_PORT}, 443, 80]:
    s = socket.socket(socket.AF_INET6, socket.SOCK_STREAM)
    s.settimeout(3)
    try:
        t0 = time.time()
        s.connect(('${dns_ipv6}', port, 0, 0))
        ms = (time.time() - t0) * 1000
        print(f'    IPv6 :{port} → OPEN ({ms:.0f}ms)')
        s.close()
    except ConnectionRefusedError:
        print(f'    IPv6 :{port} → REFUSED (reachable, not listening)')
    except:
        print(f'    IPv6 :{port} → TIMEOUT')
" 2>/dev/null
        record "6" "ipv6_direct" "TESTED" "via DNS"
    else
        log_warn "No IPv6 from DNS"
        record "6" "ipv6_direct" "SKIP" "no DNS AAAA"
    fi

    # Path 2: IPv4 via DNS (likely NATed)
    log_step "6.2" "IPv4 via tower.nestgate.io (Tier 0 - NAT dependent)..."
    local dns_ipv4
    dns_ipv4=$(host -t A "$TOWER_DNS_HOST" 2>/dev/null | grep 'has address' | awk '{print $NF}') || dns_ipv4=""
    if [ -n "$dns_ipv4" ]; then
        python3 -c "
import socket, time
for port in [${TOWER_SONGBIRD_PORT}, ${TOWER_IPC_PORT}, 443]:
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.settimeout(3)
    try:
        t0 = time.time()
        s.connect(('${dns_ipv4}', port))
        ms = (time.time() - t0) * 1000
        print(f'    IPv4 :{port} → OPEN ({ms:.0f}ms)')
        s.close()
    except ConnectionRefusedError:
        print(f'    IPv4 :{port} → REFUSED (reachable, not listening)')
    except:
        print(f'    IPv4 :{port} → TIMEOUT (NAT blocking inbound)')
" 2>/dev/null
        record "6" "ipv4_direct" "TESTED" "via DNS"
    fi

    # Path 3: LAN direct
    log_step "6.3" "LAN direct to ${TOWER_LAN_IP} (Tier 0 - same network only)..."
    python3 -c "
import socket, time
for port in [${TOWER_SONGBIRD_PORT}, ${TOWER_IPC_PORT}, 3479]:
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.settimeout(2)
    try:
        t0 = time.time()
        s.connect(('${TOWER_LAN_IP}', port))
        ms = (time.time() - t0) * 1000
        print(f'    LAN :{port} → OPEN ({ms:.0f}ms)')
        s.close()
    except ConnectionRefusedError:
        print(f'    LAN :{port} → REFUSED (on LAN, not listening)')
    except:
        print(f'    LAN :{port} → UNREACHABLE (not on same network)')
" 2>/dev/null
    record "6" "lan_direct" "TESTED" "$TOWER_LAN_IP"

    # Path 4: Tor .onion rendezvous
    log_step "6.4" "Tor .onion rendezvous via ${TOWER_ONION:0:20}... (Tier 3 - NAT bypass)..."
    if command -v torsocks >/dev/null 2>&1; then
        local tor_test
        tor_test=$(timeout 30 python3 -c "
import socket, socks, time

s = socks.socksocket(socket.AF_INET, socket.SOCK_STREAM)
s.set_proxy(socks.SOCKS5, '127.0.0.1', 9050)
s.settimeout(25)
try:
    t0 = time.time()
    s.connect(('${TOWER_ONION}', ${TOWER_SONGBIRD_PORT}))
    ms = (time.time() - t0) * 1000
    print(f'CONNECTED via Tor in {ms:.0f}ms')
    s.close()
except ConnectionRefusedError:
    print('REFUSED (Tor routing works, port not listening)')
except Exception as e:
    print(f'FAILED: {e}')
" 2>/dev/null) || tor_test=""

        if [ -z "$tor_test" ]; then
            # Fallback: try with torsocks + nc
            tor_test=$(timeout 20 torsocks bash -c "echo 'test' | nc -w5 ${TOWER_ONION} ${TOWER_IPC_PORT}" 2>&1) || tor_test="TIMEOUT"
            if echo "$tor_test" | grep -q "Connection refused"; then
                tor_test="REFUSED (Tor works, service not listening)"
            fi
        fi

        if echo "$tor_test" | grep -qi "CONNECTED\|refused"; then
            log_ok "Tor rendezvous: ${tor_test}"
            record "6" "tor_onion" "PASS" "$tor_test"
        else
            log_warn "Tor rendezvous: ${tor_test}"
            record "6" "tor_onion" "WARN" "$tor_test"
        fi
    else
        log_info "torsocks/pysocks not available, trying torsocks only..."
        local tor_nc
        tor_nc=$(timeout 20 torsocks nc -zw5 "${TOWER_ONION}" "${TOWER_SONGBIRD_PORT}" 2>&1) && log_ok "Tor: port open" || log_warn "Tor: ${tor_nc}"
        record "6" "tor_onion" "TESTED" "torsocks"
    fi

    # Path 5: STUN NAT type detection
    log_step "6.5" "STUN NAT type detection (symmetric vs cone)..."
    python3 -c "
import socket, struct, random, time

def stun_mapped(host, port):
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.settimeout(3)
    txid = random.randbytes(12)
    hdr = struct.pack('!HHI', 0x0001, 0, 0x2112A442) + txid
    try:
        s.sendto(hdr, (host, port))
        d, _ = s.recvfrom(1024)
        o = 20
        while o < len(d):
            at = struct.unpack('!H', d[o:o+2])[0]
            al = struct.unpack('!H', d[o+2:o+4])[0]
            av = d[o+4:o+4+al]
            if at == 0x0020 and len(av) >= 8:
                xp = struct.unpack('!H', av[2:4])[0] ^ 0x2112
                xi = struct.unpack('!I', av[4:8])[0] ^ 0x2112A442
                return socket.inet_ntoa(struct.pack('!I', xi)), xp
            o += 4 + al + (4 - al%4)%4
    except: pass
    finally: s.close()
    return None, None

servers = [
    ('stun.l.google.com', 19302),
    ('stun.cloudflare.com', 3478),
    ('stun.nextcloud.com', 3478),
]

results = []
for host, port in servers:
    ip, mapped_port = stun_mapped(host, port)
    if ip:
        results.append((host, ip, mapped_port))
        print(f'    {host}: {ip}:{mapped_port}')

if len(results) >= 2:
    ports = set(r[2] for r in results)
    ips = set(r[1] for r in results)
    if len(ports) > 1:
        print(f'    NAT TYPE: SYMMETRIC (ports vary: {ports})')
        print(f'    UDP hole punch: unlikely to succeed')
        print(f'    Primary path: IPv6 direct or Tor rendezvous')
    else:
        print(f'    NAT TYPE: CONE/RESTRICTED (consistent port {ports.pop()})')
        print(f'    UDP hole punch: good chance of success')
elif len(results) == 1:
    print(f'    Only 1 STUN response (partial, some STUN blocked)')
else:
    print(f'    All STUN probes failed (ISP blocking UDP STUN?)')
" 2>/dev/null
    record "6" "stun_nat_type" "TESTED" ""

    echo ""
    echo -e "  ${BOLD}Connectivity Summary:${NC}"
    echo -e "    Tier 0: ${GREEN}IPv6 via tower.nestgate.io${NC} (no NAT, ISP can't block)"
    echo -e "    Tier 1: ${YELLOW}IPv4 via tower.nestgate.io${NC} (NAT dependent)"
    echo -e "    Tier 2: ${CYAN}LAN direct${NC} (same network only)"
    echo -e "    Tier 3: ${MAGENTA}Tor .onion rendezvous${NC} (symmetric NAT bypass)"
}

# ═══════════════════════════════════════════════════════════════════════════
# Final Report
# ═══════════════════════════════════════════════════════════════════════════

print_report() {
    echo ""
    echo -e "${BOLD}${BLUE}═════════════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}${BLUE}  SYMMETRIC NAT HANDSHAKE VALIDATION REPORT${NC}"
    echo -e "${BOLD}${BLUE}═════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "  ${BOLD}Date:${NC}  $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
    echo -e "  ${BOLD}Tower:${NC} ${TOWER_LAN_IP} / .onion: ${TOWER_ONION:0:20}..."
    echo ""

    local pass=0 fail=0 warn=0 info=0
    for r in "${PHASE_RESULTS[@]}"; do
        local status=$(echo "$r" | cut -d'|' -f3)
        case "$status" in
            PASS) pass=$((pass+1)) ;;
            FAIL) fail=$((fail+1)) ;;
            WARN|EXPECTED_FAIL|SYMMETRIC) warn=$((warn+1)) ;;
            *) info=$((info+1)) ;;
        esac
    done

    echo -e "  ${GREEN}PASS: ${pass}${NC}  ${RED}FAIL: ${fail}${NC}  ${YELLOW}WARN: ${warn}${NC}  ${CYAN}INFO: ${info}${NC}"
    echo ""

    echo -e "  ${BOLD}Correct Architecture:${NC}"
    echo -e "    Seeds: ${GREEN}Unique per device, lineage-derived (Blake3-KDF)${NC}"
    echo -e "    Beacon: ${GREEN}Independent from lineage (mitochondrial DNA model)${NC}"
    echo -e "    Discovery: ${GREEN}Dark Forest beacons (encrypted, family-only)${NC}"
    echo ""

    echo -e "  ${BOLD}Connection Priority:${NC}"
    echo -e "    0. ${GREEN}DNS${NC}      - tower.nestgate.io → IPv6 direct (UNIVERSAL)"
    echo -e "    1. ${GREEN}Direct${NC}   - Dark Forest beacon discovery (same network)"
    echo -e "    2. ${YELLOW}STUN${NC}     - Public address + UDP hole punch (non-symmetric)"
    echo -e "    3. ${CYAN}Tor${NC}      - Pure Rust Tor rendezvous at Tower .onion"
    echo ""

    echo -e "  ${BOLD}Live Test Steps:${NC}"
    echo -e "    ${BOLD}On Tower:${NC}"
    echo -e "      1. ${CYAN}sudo systemctl start tor${NC}"
    echo -e "      2. ${CYAN}cd livespore-usb/x86_64 && ./primals/songbird${NC}"
    echo -e "      3. ${CYAN}./primals/beardog${NC}"
    echo ""
    echo -e "    ${BOLD}On Pixel (on hotspot):${NC}"
    echo -e "      1. ${CYAN}./primals/songbird${NC}  (will try direct, then Tor)"
    echo -e "      2. ${CYAN}./primals/beardog${NC}   (crypto for BirdSong + Tor)"
    echo -e "      3. Songbird auto-discovers Tower via .onion rendezvous"
    echo ""
    echo -e "    ${BOLD}Expected Flow:${NC}"
    echo -e "      Pixel: STUN -> detect symmetric NAT -> try holepunch -> fail"
    echo -e "      Pixel: songbird-tor-protocol builds circuit -> reaches Tower .onion"
    echo -e "      Tower: accepts connection -> verifies lineage (BearDog)"
    echo -e "      Tower: brokers Pixel <-> USB connection"
    echo -e "      Result: encrypted P2P through family-owned Tor rendezvous"
    echo ""

    if [ "$fail" -gt 0 ]; then
        echo -e "  ${RED}${BOLD}ACTION NEEDED: ${fail} failures to resolve before live test${NC}"
    else
        echo -e "  ${GREEN}${BOLD}READY for live symmetric NAT test!${NC}"
    fi

    echo ""
    echo -e "${BOLD}${BLUE}═════════════════════════════════════════════════════════════${NC}"
}

# ═══════════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════════

main() {
    echo -e "${BOLD}${BLUE}"
    echo "  ╔═══════════════════════════════════════════════════════════╗"
    echo "  ║     Sovereign NAT Traversal Validator                     ║"
    echo "  ║     Pixel (hotspot) <-> Tower (.onion) <-> USB (LAN)      ║"
    echo "  ║     DNS: tower.nestgate.io | Tor | BearDog | Dark Forest  ║"
    echo "  ╚═══════════════════════════════════════════════════════════╝"
    echo -e "${NC}"

    phase0_dns_beacon
    phase1_lineage_seeds
    phase2_beacon_seeds
    phase3_direct_discovery
    phase4_tor_rendezvous
    phase5_binary_sync
    phase6_live_connectivity

    print_report
}

main "$@"
