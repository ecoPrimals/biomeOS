# Universal Beacon Specification

**Purpose**: Ensure family devices can ALWAYS find Tower HPC, regardless of ISP blocking  
**Status**: Design + Implementation  
**Created**: February 7, 2026  
**Problem**: AT&T blocks Tor relay IPs. ISPs can block any specific protocol.

---

## Findings: What AT&T Blocks vs Can't Block

| Protocol | Status | Notes |
|----------|--------|-------|
| Tor directory authorities | **BLOCKED** | TCP to known Tor IPs |
| TCP to STUN ports | **BLOCKED** | Port-based blocking |
| HTTPS (port 443) | **OPEN** | Can't block without breaking internet |
| UDP STUN | **OPEN** | Address discovery works |
| IPv6 outbound | **OPEN** | Global connectivity confirmed |
| IPv6 listen | **OPEN** | Tower can accept IPv6 connections |

## Key Insight

> **ISPs cannot block HTTPS or IPv6 without breaking the internet.**  
> These are the two universally available transport channels.

---

## Architecture: Universal Beacon

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        UNIVERSAL BEACON                                      │
│                                                                              │
│  "Any device, any network, always finds home"                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  TIER 0: IPv6 Direct (No NAT, No Traversal, No Blocking)                    │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │  Tower: [2600:1700:b0b0:5b90::27]:3492 (global, routable)            │ │
│  │  Pixel:  connects directly via IPv6 — zero NAT                        │ │
│  │                                                                        │ │
│  │  Problem: IPv6 address changes (privacy extensions, ISP)              │ │
│  │  Solution: Beacon Drop (Tier 1)                                       │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                              ↓ (address discovery)                           │
│  TIER 1: HTTPS Beacon Drop (Address Discovery)                               │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │  Tower encrypts current endpoints with mito beacon seed               │ │
│  │  Pushes encrypted blob to HTTPS URL (Cloudflare Worker, etc.)         │ │
│  │  Pixel fetches via HTTPS (ISP sees: normal web traffic)               │ │
│  │  Pixel decrypts → gets Tower's current IPv6/IPv4/onion               │ │
│  │                                                                        │ │
│  │  External observer sees: encrypted noise at a CDN URL                 │ │
│  │  Family sees: { ipv6: "2600:...", ipv4: "162.226...", port: 3492 }   │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                              ↓ (fallback)                                    │
│  TIER 2: DNS TXT Beacon (Ultra-Reliable)                                     │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │  Tower publishes encrypted beacon as DNS TXT record                   │ │
│  │  Domain: beacon.yourfamily.example                                    │ │
│  │  TXT: "v=biomeos1 b=<base64 encrypted beacon>"                       │ │
│  │  DNS is NEVER blocked (required for internet to function)             │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                              ↓ (fallback)                                    │
│  TIER 3: Tor via Bridges (If IPv6 Fails)                                     │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │  Use Tor bridges (unlisted relays) instead of directory authorities   │ │
│  │  obfs4 pluggable transport makes traffic look like noise             │ │
│  │  Songbird builds circuit through bridge → reaches .onion              │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                              ↓ (fallback)                                    │
│  TIER 4: WebSocket over HTTPS (Last Resort)                                  │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │  Tunnel Songbird protocol inside WebSocket over HTTPS (port 443)      │ │
│  │  Looks identical to normal web traffic (browser WebSocket)            │ │
│  │  ISP literally cannot distinguish from web browsing                   │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Tier 0: IPv6 Direct (PRIMARY — Immediate)

### Why This Works

- Tower has global IPv6: `2600:1700:b0b0:5b90::27`
- IPv6 has NO NAT — addresses are globally routable
- ISP provides /64 prefix — effectively unlimited addresses
- AT&T Fiber provides native IPv6 dual-stack
- Most US carriers (T-Mobile, AT&T, Verizon) provide IPv6 to mobile devices

### Implementation

```
Tower:  Songbird listens on [::]:3492 (dual-stack)
Pixel:  Songbird connects to [2600:1700:b0b0:5b90::27]:3492
Result: Direct P2P, no NAT, no relay, no Tor needed
```

### What Songbird Needs

1. Bind to `[::]:3492` (IPv6 dual-stack, accepts both v4 and v6)
2. Publish IPv6 address in beacon
3. Accept connections on IPv6

### Address Stability

AT&T Fiber assigns relatively stable IPv6 addresses, but they can change. 
Solutions:
- SLAAC stable address (`::27`) tends to persist across reboots
- DHCPv6 address may change on lease renewal
- Use privacy extensions address for outbound, stable for inbound
- Beacon Drop (Tier 1) handles address changes

---

## Tier 1: HTTPS Beacon Drop (Address Discovery)

### Concept

Tower periodically publishes its current endpoints (encrypted with family mito 
seed) to a publicly accessible HTTPS URL. Pixel fetches this URL to discover 
Tower's current address.

### Zero-Infrastructure Options

| Option | Cost | Sovereignty | Reliability |
|--------|------|-------------|-------------|
| Cloudflare Worker (KV) | Free (100k req/day) | Medium | High |
| GitHub Gist (raw URL) | Free | Medium | High |
| Self-hosted HTTPS | Free | Maximum | Medium |
| DNS TXT record | ~$10/yr domain | High | Very High |

### Beacon Drop Format

```json
{
  "v": 1,
  "t": 1770488848,
  "b": "<base64 ChaCha20-Poly1305 encrypted blob>"
}
```

Encrypted blob contains:
```json
{
  "ipv6": "2600:1700:b0b0:5b90::27",
  "ipv6_port": 3492,
  "ipv4": "162.226.225.148",
  "ipv4_port": 3492,
  "onion": "eaaz3...onion",
  "onion_port": 3492,
  "capabilities": ["compute", "ai-server", "relay"],
  "timestamp": 1770488848,
  "signature": "<Ed25519 signature>"
}
```

### Privacy

- External observer: sees HTTPS request to Cloudflare/GitHub (normal traffic)
- CDN/host: sees 256-byte encrypted blob (noise, no metadata)
- Family: decrypts with mito seed → gets all Tower endpoints

---

## Tier 2: DNS TXT Beacon

### Concept

Publish encrypted beacon as DNS TXT record. DNS resolution is never blocked.

```
_beacon.family.example.com TXT "v=biomeos1 b=<base64 encrypted beacon>"
```

### Why DNS is Universal

DNS is required for the internet to function. No ISP can block DNS 
without making their service unusable. Even if the ISP DNS is filtered,
DNS-over-HTTPS (DoH) to Cloudflare (1.1.1.1) or Google (8.8.8.8) works
over port 443 (which we confirmed is open).

---

## Connection Priority (Updated)

```
Pixel on Hotspot → Find Tower:

1. Check known IPv6 from beacon        → Direct connect (no NAT!)
2. If IPv6 stale → HTTPS beacon drop   → Get fresh IPv6 → connect
3. If IPv6 fails → DNS TXT beacon      → Get fresh IPv6 → connect  
4. If all IPv6 fails → Tor bridge      → .onion rendezvous
5. If Tor blocked → WebSocket/443      → Tunnel through HTTPS

In practice: Step 1-2 should work 99% of the time.
```

---

## IPv6 Advantages for biomeOS

1. **No NAT** — globally routable, direct P2P
2. **No STUN needed** — address is the real address
3. **No hole punching** — no NAT to punch through
4. **No relay needed** — direct connection
5. **ISP can't block** — required infrastructure
6. **Privacy addresses** — outbound uses random addresses
7. **Stable inbound** — SLAAC stable address for listening
8. **Dual-stack** — can accept both v4 and v6 connections

---

## Implementation Priority

| Component | Effort | Impact |
|-----------|--------|--------|
| Songbird IPv6 dual-stack listen | Low | **Massive** — solves NAT problem |
| Beacon with IPv6 endpoint | Low | Address discovery |
| HTTPS beacon drop | Medium | Dynamic address updates |
| DNS TXT beacon | Medium | Ultra-reliable fallback |
| Tor bridges integration | High | Blocked ISP fallback |
| WebSocket tunnel | High | Universal last resort |

**Recommendation**: IPv6 dual-stack + HTTPS beacon drop covers 99% of cases
with minimal effort and zero external dependencies.
