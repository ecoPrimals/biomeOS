#!/usr/bin/env bash
#
# beacon_dns_updater.sh
#
# Anchors the Tower HPC as a permanently findable beacon via nestgate.io DNS.
#
# Architecture:
#   tower.nestgate.io   AAAA  → HPC's current global IPv6 (direct, no NAT)
#   tower.nestgate.io   A     → HPC's current public IPv4 (informational)
#   beacon.nestgate.io  TXT   → Dark Forest encrypted beacon (all endpoints)
#
# The Pixel (or any family device) resolves tower.nestgate.io → gets IPv6 → connects.
# DNS works everywhere. ISP can't block it. Domain never changes.
#
# Usage:
#   # First time setup:
#   ./scripts/beacon_dns_updater.sh setup
#
#   # Update records (run periodically or on IP change):
#   ./scripts/beacon_dns_updater.sh update
#
#   # Check current records:
#   ./scripts/beacon_dns_updater.sh status
#
#   # Run as daemon (checks every 5 minutes):
#   ./scripts/beacon_dns_updater.sh daemon
#
# Prerequisites:
#   - Porkbun API key and secret in ~/.config/biomeos/porkbun.env
#   - nestgate.io domain registered at Porkbun with API Access enabled
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# ═══════════════════════════════════════════════════════════════════════════
# Configuration
# ═══════════════════════════════════════════════════════════════════════════

DOMAIN="nestgate.io"
SUBDOMAIN="tower"                      # tower.nestgate.io
BEACON_SUBDOMAIN="beacon"             # beacon.nestgate.io TXT
SONGBIRD_PORT=3492
NODE_ID="${NODE_ID:-gate}"

# Family ID derivation from seed (canonical: hex of first 8 bytes)
derive_family_id() {
    local seed_file=""
    for candidate in "$PROJECT_ROOT/.family.seed" "$PROJECT_ROOT/livespore-usb/.family.seed"; do
        if [ -f "$candidate" ]; then seed_file="$candidate"; break; fi
    done
    [ -n "$seed_file" ] && xxd -p -l 8 "$seed_file" | tr -d '\n'
}
FAMILY_ID="${FAMILY_ID:-$(derive_family_id)}"
if [ -z "$FAMILY_ID" ] || [ ${#FAMILY_ID} -lt 16 ]; then
    echo "Warning: Could not derive 16-char FAMILY_ID from seed" >&2
fi

# Porkbun API (MUST use api.porkbun.com, NOT porkbun.com)
PORKBUN_API="https://api.porkbun.com/api/json/v3"
PORKBUN_ENV="${HOME}/.config/biomeos/porkbun.env"

# State file (tracks last known IPs to avoid unnecessary updates)
STATE_FILE="${HOME}/.config/biomeos/beacon_dns_state.json"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m'

# ═══════════════════════════════════════════════════════════════════════════
# Load API credentials
# ═══════════════════════════════════════════════════════════════════════════

load_credentials() {
    if [ ! -f "$PORKBUN_ENV" ]; then
        echo -e "${RED}Porkbun credentials not found at ${PORKBUN_ENV}${NC}"
        echo ""
        echo "Create the file with:"
        echo "  mkdir -p ~/.config/biomeos"
        echo "  cat > ~/.config/biomeos/porkbun.env << 'EOF'"
        echo "  PORKBUN_API_KEY=pk1_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
        echo "  PORKBUN_SECRET_KEY=sk1_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
        echo "  EOF"
        echo "  chmod 600 ~/.config/biomeos/porkbun.env"
        echo ""
        echo "Get your API keys from: https://porkbun.com/account/api"
        exit 1
    fi

    source "$PORKBUN_ENV"

    if [ -z "${PORKBUN_API_KEY:-}" ] || [ -z "${PORKBUN_SECRET_KEY:-}" ]; then
        echo -e "${RED}PORKBUN_API_KEY and PORKBUN_SECRET_KEY must be set in ${PORKBUN_ENV}${NC}"
        exit 1
    fi
}

# ═══════════════════════════════════════════════════════════════════════════
# IP Discovery
# ═══════════════════════════════════════════════════════════════════════════

get_ipv6() {
    # Get the stable global IPv6 (prefer DHCPv6/SLAAC stable address)
    # Filter for the /128 addresses (DHCPv6 assigned, most stable)
    local ipv6
    ipv6=$(ip -6 addr show scope global | grep 'inet6.*\/128' | head -1 | awk '{print $2}' | cut -d/ -f1)

    if [ -z "$ipv6" ]; then
        # Fall back to any global IPv6 (SLAAC, non-temporary)
        ipv6=$(ip -6 addr show scope global mngtmpaddr | grep 'inet6' | head -1 | awk '{print $2}' | cut -d/ -f1)
    fi

    if [ -z "$ipv6" ]; then
        # Last resort: any global IPv6
        ipv6=$(ip -6 addr show scope global | grep 'inet6' | head -1 | awk '{print $2}' | cut -d/ -f1)
    fi

    echo "$ipv6"
}

get_ipv4() {
    # Get public IPv4 via STUN (UDP, works on AT&T)
    python3 -c "
import socket, struct, random
def stun(host, port):
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.settimeout(3)
    hdr = struct.pack('!HHI', 0x0001, 0, 0x2112A442) + random.randbytes(12)
    try:
        s.sendto(hdr, (host, port))
        d, _ = s.recvfrom(1024)
        o = 20
        while o < len(d):
            at = struct.unpack('!H', d[o:o+2])[0]
            al = struct.unpack('!H', d[o+2:o+4])[0]
            av = d[o+4:o+4+al]
            if at == 0x0020 and len(av) >= 8:
                xi = struct.unpack('!I', av[4:8])[0] ^ 0x2112A442
                return socket.inet_ntoa(struct.pack('!I', xi))
            o += 4 + al + (4 - al%4)%4
    except: pass
    finally: s.close()
    return None
for h,p in [('stun.l.google.com',19302),('stun.cloudflare.com',3478)]:
    r = stun(h, p)
    if r:
        print(r)
        break
" 2>/dev/null
}

# ═══════════════════════════════════════════════════════════════════════════
# Porkbun API helpers
# ═══════════════════════════════════════════════════════════════════════════

porkbun_api() {
    local endpoint="$1"
    local data="$2"

    # Inject credentials into the JSON payload
    local auth_data
    auth_data=$(echo "$data" | python3 -c "
import sys, json
d = json.load(sys.stdin)
d['apikey'] = '${PORKBUN_API_KEY}'
d['secretapikey'] = '${PORKBUN_SECRET_KEY}'
print(json.dumps(d))
")

    curl -s -X POST \
        "${PORKBUN_API}${endpoint}" \
        -H "Content-Type: application/json" \
        -d "$auth_data" 2>/dev/null
}

# ═══════════════════════════════════════════════════════════════════════════
# DNS Record Management
# ═══════════════════════════════════════════════════════════════════════════

create_or_update_record() {
    local type="$1"      # AAAA, A, TXT
    local subdomain="$2" # tower, _beacon
    local content="$3"
    local ttl="${4:-300}" # 5 min default (fast updates)

    echo -ne "  ${type} ${subdomain}.${DOMAIN} → "

    # Try to edit existing record first
    local result
    result=$(porkbun_api "/dns/editByNameType/${DOMAIN}/${type}/${subdomain}" \
        "{\"content\": \"${content}\", \"ttl\": \"${ttl}\"}")

    if echo "$result" | python3 -c "import sys,json; r=json.load(sys.stdin); exit(0 if r.get('status')=='SUCCESS' else 1)" 2>/dev/null; then
        echo -e "${GREEN}updated${NC} (${content:0:50})"
        return 0
    fi

    # If edit fails, create new record
    result=$(porkbun_api "/dns/create/${DOMAIN}" \
        "{\"type\": \"${type}\", \"name\": \"${subdomain}\", \"content\": \"${content}\", \"ttl\": \"${ttl}\"}")

    if echo "$result" | python3 -c "import sys,json; r=json.load(sys.stdin); exit(0 if r.get('status')=='SUCCESS' else 1)" 2>/dev/null; then
        echo -e "${GREEN}created${NC} (${content:0:50})"
        return 0
    fi

    echo -e "${RED}FAILED${NC}"
    echo "    Response: ${result:0:200}"
    return 1
}

# ═══════════════════════════════════════════════════════════════════════════
# Beacon Generation (Dark Forest encrypted)
# ═══════════════════════════════════════════════════════════════════════════

generate_encrypted_beacon() {
    local ipv6="$1"
    local ipv4="$2"
    local beardog_sock="${BEARDOG_SOCKET:-/run/user/1000/biomeos/beardog.sock}"

    python3 -c "
import json, subprocess, base64, time, sys
from pathlib import Path

# Read family seed to derive shared key
seed_path = Path('${PROJECT_ROOT}') / '.family.seed'
if not seed_path.exists():
    print('ERROR: no family seed', file=sys.stderr)
    sys.exit(1)

seed_b64 = base64.b64encode(seed_path.read_bytes()).decode()
beardog_sock = '${beardog_sock}'

def beardog_rpc(method, params):
    req = json.dumps({'jsonrpc': '2.0', 'method': method, 'params': params, 'id': 1})
    with open('/tmp/_beacon_rpc.json', 'w') as f:
        f.write(req)
    r = subprocess.run(
        ['bash', '-c', f'cat /tmp/_beacon_rpc.json | nc -U {beardog_sock} -w 5 -q 3'],
        capture_output=True, text=True, timeout=10
    )
    return json.loads(r.stdout.strip())

# Derive shared key from family seed
key_resp = beardog_rpc('crypto.hmac_sha256', {
    'key': seed_b64,
    'data': base64.b64encode(b'beacon_shared_key').decode()
})
shared_key = key_resp['result']['mac']

# Build beacon payload
beacon = json.dumps({
    'v': 2,
    'family': '${FAMILY_ID}',
    'node': '${NODE_ID:-gate}',
    'ts': int(time.time()),
    'endpoints': {
        'ipv6': '${ipv6}',
        'ipv6_port': ${SONGBIRD_PORT},
        'ipv4': '${ipv4}',
        'dns': 'tower.nestgate.io',
    },
    'caps': ['compute', 'ai-server', 'gpu', 'storage', 'sovereign-beacon'],
}, separators=(',', ':')).encode()

# Encrypt with BearDog's ChaCha20-Poly1305 (real AEAD crypto)
enc_resp = beardog_rpc('crypto.chacha20_poly1305_encrypt', {
    'key': shared_key,
    'plaintext': base64.b64encode(beacon).decode(),
    'associated_data': base64.b64encode(b'nestgate-beacon-v2').decode()
})
r = enc_resp['result']

# Pack as: nonce || ciphertext || tag (compact binary)
import base64 as b64
nonce = b64.b64decode(r['nonce'])
ct = b64.b64decode(r['ciphertext'])
tag = b64.b64decode(r['tag'])
blob = base64.b64encode(nonce + ct + tag).decode()

print(f'v=biomeos2 b={blob}')
" 2>/dev/null
}

# ═══════════════════════════════════════════════════════════════════════════
# Commands
# ═══════════════════════════════════════════════════════════════════════════

cmd_setup() {
    echo -e "${BOLD}${CYAN}Beacon DNS Setup: ${DOMAIN}${NC}"
    echo -e "${BOLD}═══════════════════════════════════════════${NC}"
    echo ""

    load_credentials

    # Test API connectivity
    echo -e "Testing Porkbun API..."
    local ping_result
    ping_result=$(porkbun_api "/ping" "{}")

    if echo "$ping_result" | grep -q "SUCCESS"; then
        echo -e "  ${GREEN}API connected${NC}"
        local your_ip
        your_ip=$(echo "$ping_result" | python3 -c "import sys,json; print(json.load(sys.stdin).get('yourIp','unknown'))" 2>/dev/null)
        echo -e "  Your IP (Porkbun sees): ${your_ip}"
    else
        echo -e "  ${RED}API connection failed${NC}"
        echo "  Response: ${ping_result:0:200}"
        echo "  Check your API keys in ${PORKBUN_ENV}"
        exit 1
    fi

    # Discover current IPs
    echo ""
    echo -e "Discovering addresses..."
    local ipv6 ipv4
    ipv6=$(get_ipv6)
    ipv4=$(get_ipv4)
    echo -e "  IPv6: ${GREEN}${ipv6:-none}${NC}"
    echo -e "  IPv4: ${YELLOW}${ipv4:-none}${NC}"

    # Create DNS records
    echo ""
    echo -e "Creating DNS records..."

    if [ -n "$ipv6" ]; then
        create_or_update_record "AAAA" "$SUBDOMAIN" "$ipv6" "300"
    fi

    if [ -n "$ipv4" ]; then
        create_or_update_record "A" "$SUBDOMAIN" "$ipv4" "300"
    fi

    # Encrypted beacon TXT
    local beacon_txt
    beacon_txt=$(generate_encrypted_beacon "${ipv6:-}" "${ipv4:-}")
    if [ -n "$beacon_txt" ]; then
        create_or_update_record "TXT" "$BEACON_SUBDOMAIN" "$beacon_txt" "300"
    fi

    # Save state
    mkdir -p "$(dirname "$STATE_FILE")"
    python3 -c "
import json, time
state = {
    'ipv6': '${ipv6:-}',
    'ipv4': '${ipv4:-}',
    'last_update': time.time(),
    'domain': '${DOMAIN}',
    'subdomain': '${SUBDOMAIN}'
}
with open('${STATE_FILE}', 'w') as f:
    json.dump(state, f, indent=2)
"

    echo ""
    echo -e "${BOLD}${GREEN}Setup complete!${NC}"
    echo ""
    echo -e "  ${BOLD}Pixel connects to:${NC}"
    echo -e "    ${GREEN}tower.nestgate.io${NC} → [${ipv6}]:${SONGBIRD_PORT}"
    echo ""
    echo -e "  ${BOLD}DNS records:${NC}"
    echo -e "    tower.nestgate.io    AAAA  ${ipv6}"
    echo -e "    tower.nestgate.io    A     ${ipv4}"
    echo -e "    beacon.nestgate.io   TXT   (encrypted beacon)"
    echo ""
    echo -e "  ${BOLD}Next:${NC}"
    echo -e "    Run daemon: ${CYAN}./scripts/beacon_dns_updater.sh daemon${NC}"
    echo -e "    Or cron:    ${CYAN}*/5 * * * * /path/to/beacon_dns_updater.sh update${NC}"
}

cmd_update() {
    load_credentials

    local ipv6 ipv4
    ipv6=$(get_ipv6)
    ipv4=$(get_ipv4)

    # Check if IPs changed
    local prev_ipv6="" prev_ipv4=""
    if [ -f "$STATE_FILE" ]; then
        prev_ipv6=$(python3 -c "import json; print(json.load(open('$STATE_FILE')).get('ipv6',''))" 2>/dev/null) || true
        prev_ipv4=$(python3 -c "import json; print(json.load(open('$STATE_FILE')).get('ipv4',''))" 2>/dev/null) || true
    fi

    local changed=false

    if [ "$ipv6" != "$prev_ipv6" ] && [ -n "$ipv6" ]; then
        echo -e "$(date -u +%H:%M:%S) IPv6 changed: ${prev_ipv6:-none} → ${ipv6}"
        create_or_update_record "AAAA" "$SUBDOMAIN" "$ipv6" "300"
        changed=true
    fi

    if [ "$ipv4" != "$prev_ipv4" ] && [ -n "$ipv4" ]; then
        echo -e "$(date -u +%H:%M:%S) IPv4 changed: ${prev_ipv4:-none} → ${ipv4}"
        create_or_update_record "A" "$SUBDOMAIN" "$ipv4" "300"
        changed=true
    fi

    if [ "$changed" = true ]; then
        # Update beacon TXT
        local beacon_txt
        beacon_txt=$(generate_encrypted_beacon "${ipv6:-}" "${ipv4:-}")
        if [ -n "$beacon_txt" ]; then
            create_or_update_record "TXT" "$BEACON_SUBDOMAIN" "$beacon_txt" "300"
        fi

        # Save state
        python3 -c "
import json, time
state = {
    'ipv6': '${ipv6:-}',
    'ipv4': '${ipv4:-}',
    'last_update': time.time(),
    'domain': '${DOMAIN}',
    'subdomain': '${SUBDOMAIN}'
}
with open('${STATE_FILE}', 'w') as f:
    json.dump(state, f, indent=2)
"
        echo -e "$(date -u +%H:%M:%S) ${GREEN}DNS updated${NC}"
    fi
    # Silent if nothing changed (good for cron)
}

cmd_status() {
    echo -e "${BOLD}Beacon DNS Status${NC}"
    echo ""

    # Current IPs
    local ipv6 ipv4
    ipv6=$(get_ipv6)
    ipv4=$(get_ipv4)
    echo -e "  Current IPv6: ${GREEN}${ipv6:-none}${NC}"
    echo -e "  Current IPv4: ${YELLOW}${ipv4:-none}${NC}"

    # DNS resolution
    echo ""
    echo -e "  DNS resolution:"
    for type_flag in "-AAAA" "-A" "-TXT"; do
        local qtype="${type_flag#-}"
        local target
        if [ "$qtype" = "TXT" ]; then
            target="${BEACON_SUBDOMAIN}.${DOMAIN}"
        else
            target="${SUBDOMAIN}.${DOMAIN}"
        fi
        local result
        result=$(dig +short ${type_flag} "${target}" 2>/dev/null | head -3) || result=""
        if [ -n "$result" ]; then
            echo -e "    ${target} ${qtype}: ${GREEN}${result}${NC}"
        else
            echo -e "    ${target} ${qtype}: ${YELLOW}(not set)${NC}"
        fi
    done

    # State file
    if [ -f "$STATE_FILE" ]; then
        echo ""
        local last_update
        last_update=$(python3 -c "
import json, time
s = json.load(open('$STATE_FILE'))
ago = int(time.time() - s.get('last_update', 0))
if ago < 60: print(f'{ago}s ago')
elif ago < 3600: print(f'{ago//60}m ago')
else: print(f'{ago//3600}h ago')
" 2>/dev/null) || last_update="unknown"
        echo -e "  Last update: ${last_update}"
    fi
}

cmd_daemon() {
    echo -e "${BOLD}${CYAN}Beacon DNS Daemon${NC}"
    echo -e "Updating ${SUBDOMAIN}.${DOMAIN} every 5 minutes"
    echo -e "Press Ctrl+C to stop"
    echo ""

    load_credentials

    # Initial update
    cmd_update

    while true; do
        sleep 300  # 5 minutes
        cmd_update
    done
}

# ═══════════════════════════════════════════════════════════════════════════
# Main
# ═══════════════════════════════════════════════════════════════════════════

case "${1:-status}" in
    setup)  cmd_setup ;;
    update) cmd_update ;;
    status) cmd_status ;;
    daemon) cmd_daemon ;;
    *)
        echo "Usage: $0 {setup|update|status|daemon}"
        echo ""
        echo "  setup   - First-time DNS record creation"
        echo "  update  - Update records if IPs changed"
        echo "  status  - Show current DNS state"
        echo "  daemon  - Run continuously (update every 5 min)"
        ;;
esac
