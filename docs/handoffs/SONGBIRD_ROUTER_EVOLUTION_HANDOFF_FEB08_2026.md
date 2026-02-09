# Songbird Router Evolution Handoff

**Date**: February 8, 2026  
**From**: biomeOS Integration Team  
**To**: Songbird Development Team  
**Priority**: CRITICAL — This is the #1 blocker for cross-network connectivity  
**Status**: Spec complete, ready for implementation

---

## Executive Summary

Today we completed full cross-network NAT traversal testing between the Tower (x86_64, home router) and the Pixel 8a (aarch64 Android, mobile hotspot). **Every local and crypto test passed.** The single remaining blocker: **the home router does not forward port 3492 to the Tower**, and Songbird has no way to configure the router programmatically.

This document specifies the `songbird-igd` evolution — a new crate that gives Songbird the ability to discover and configure the router via UPnP/IGD and NAT-PMP. This turns the router from a manual dependency into a tool Songbird configures automatically.

### What Was Proven Today

| Test | Result | Notes |
|------|--------|-------|
| BearDog alive (x86_64 Tower) | PASS | JSON-RPC via Unix socket |
| BearDog alive (aarch64 Pixel) | PASS | JSON-RPC via TCP :9900 |
| Songbird alive (Tower) | PASS | TCP :3492, dual-stack IPv6+IPv4 |
| Songbird alive (Pixel) | PASS | TCP :9901 orchestrator fallback |
| DNS resolution nestgate.io | PASS | Resolves to 162.226.225.148 |
| Family beacon key derivation | PASS | **Identical HMAC on both architectures** |
| Beacon encryption (ChaCha20-Poly1305) | PASS | Pixel BearDog encrypts correctly |
| Blake3 lineage proof generation | PASS | Pixel BearDog hashes correctly |
| **Cross-device beacon decrypt** | **PASS** | Tower encrypts -> DNS TXT -> Pixel decrypts -> **identical plaintext** |
| TCP to Tower:3492 from Pixel | **FAIL** | Router does not forward port |
| IPv6 direct from Pixel | FAIL | Carrier blocks IPv6 to residential |
| STUN from Pixel Songbird | SKIP | Android IPC gap (no TCP fallback for universal-ipc) |

### The Critical Finding

```
Pixel -> hotspot (172.20.10.0/28) -> carrier NAT -> internet -> home router (192.168.1.254) -> Tower (192.168.1.144)

DNS:     nestgate.io -> 162.226.225.148  ✅ resolves
Port 80: OPEN (router admin interface, NOT forwarded to Tower)
Port 443: OPEN (router admin interface, NOT forwarded to Tower)
Port 3492: CLOSED (no forwarding rule exists)
UPnP: NOT available (router has no IGD service — only found printer and Chromecast)
NAT-PMP: NOT available (port 5351 on gateway not responding)
```

The router at `192.168.1.254` is an AT&T gateway (Pace/Arris, CGI-based admin at `/cgi-bin/index.ha`). It does not expose an IGD service. This means:

1. **UPnP might be disabled** (common on ISP gateways) — Songbird should be able to probe and report this
2. **NAT-PMP might need to be tried differently** — some gateways only respond after specific initialization
3. **Manual fallback** — Songbird should clearly report "router does not support automatic port forwarding, manual configuration required" and provide the exact rule needed

---

## Architecture: Where IGD Fits in Songbird

```
songbird/crates/
├── songbird-stun/          ← EXISTS: STUN client + server (RFC 5389)
├── songbird-lineage-relay/  ← EXISTS: Relay service + UDP hole punch
├── songbird-sovereign-onion/← EXISTS: Onion overlay (.onion addresses)
├── songbird-igd/            ← NEW: Router port forwarding (UPnP IGD + NAT-PMP)
├── songbird-onion-relay/    ← EXISTS: Hole punch coordinator
├── songbird-orchestrator/   ← EXISTS: Network binding, sovereign socket
└── songbird-universal-ipc/  ← EXISTS: JSON-RPC service routing
    └── handlers/
        ├── stun_handler.rs    ← EXISTS: stun.* methods
        ├── punch_handler.rs   ← EXISTS: punch.* methods  
        ├── mesh_handler.rs    ← EXISTS: mesh.* methods
        ├── onion_handler.rs   ← EXISTS: onion.* methods
        └── igd_handler.rs     ← NEW: igd.* methods
```

### Handler Pattern (Follow Existing)

The new `igd_handler.rs` follows the exact same pattern as `stun_handler.rs`:

```rust
pub struct IgdHandler {
    /// Currently active gateway
    gateway: Arc<RwLock<Option<GatewayInstance>>>,
    /// Active port mappings
    mappings: Arc<RwLock<Vec<PortMapping>>>,
}
```

Wire into `service.rs` the same way STUN is wired:

```rust
// In service.rs handle() match:
"igd.discover" => self.igd_handler.handle_discover(params).await,
"igd.map_port" => self.igd_handler.handle_map_port(params).await,
"igd.unmap_port" => self.igd_handler.handle_unmap_port(params).await,
"igd.status" => self.igd_handler.handle_status(params).await,
"igd.external_ip" => self.igd_handler.handle_external_ip(params).await,
"igd.auto_configure" => self.igd_handler.handle_auto_configure(params).await,
```

---

## Specification: `songbird-igd` Crate

### Dependencies (Pure Rust Only)

```toml
[dependencies]
tokio = { version = "1", features = ["net", "time", "io-util"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"

# NO external crates needed — implement SSDP + SOAP directly
# Songbird already has an HTTP client (songbird-http-client)
# The protocols are simple enough to implement from scratch
```

### Protocol 1: UPnP IGD (RFC 6970)

UPnP IGD is a 3-step protocol:

#### Step 1: SSDP Discovery (UDP Multicast)

Send an M-SEARCH to `239.255.255.250:1900`:

```
M-SEARCH * HTTP/1.1
HOST: 239.255.255.250:1900
MAN: "ssdp:discover"
MX: 3
ST: urn:schemas-upnp-org:device:InternetGatewayDevice:1

```

Parse responses looking for:
- `ST: urn:schemas-upnp-org:device:InternetGatewayDevice:1`
- `LOCATION: http://192.168.1.254:5431/...` (the device description URL)

Also try `ST: urn:schemas-upnp-org:service:WANIPConnection:1` as some routers only respond to service-level queries.

#### Step 2: Device Description (HTTP GET)

Fetch the `LOCATION` URL to get the device XML. Parse it for:
- `<serviceType>urn:schemas-upnp-org:service:WANIPConnection:1</serviceType>`
- `<controlURL>/ctl/IPConn</controlURL>` (the SOAP endpoint)

Use Songbird's existing `songbird-http-client` for this — no new HTTP code needed.

#### Step 3: SOAP Control (HTTP POST)

POST SOAP actions to the control URL:

**AddPortMapping**:
```xml
<?xml version="1.0"?>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" 
            s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
  <s:Body>
    <u:AddPortMapping xmlns:u="urn:schemas-upnp-org:service:WANIPConnection:1">
      <NewRemoteHost></NewRemoteHost>
      <NewExternalPort>3492</NewExternalPort>
      <NewProtocol>TCP</NewProtocol>
      <NewInternalPort>3492</NewInternalPort>
      <NewInternalClient>192.168.1.144</NewInternalClient>
      <NewEnabled>1</NewEnabled>
      <NewPortMappingDescription>Songbird sovereign beacon</NewPortMappingDescription>
      <NewLeaseDuration>86400</NewLeaseDuration>
    </u:AddPortMapping>
  </s:Body>
</s:Envelope>
```

**GetExternalIPAddress**:
```xml
<?xml version="1.0"?>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"
            s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
  <s:Body>
    <u:GetExternalIPAddress xmlns:u="urn:schemas-upnp-org:service:WANIPConnection:1">
    </u:GetExternalIPAddress>
  </s:Body>
</s:Envelope>
```

**DeletePortMapping**:
```xml
<?xml version="1.0"?>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"
            s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
  <s:Body>
    <u:DeletePortMapping xmlns:u="urn:schemas-upnp-org:service:WANIPConnection:1">
      <NewRemoteHost></NewRemoteHost>
      <NewExternalPort>3492</NewExternalPort>
      <NewProtocol>TCP</NewProtocol>
    </u:DeletePortMapping>
  </s:Body>
</s:Envelope>
```

### Protocol 2: NAT-PMP (RFC 6886) — Fallback

NAT-PMP is a simpler binary UDP protocol. Send to gateway:5351.

**Public IP Request** (2 bytes):
```
0x00 0x00  (version=0, opcode=0)
```

**Port Mapping Request** (12 bytes):
```
0x00        version
0x02        opcode (TCP mapping; 0x01 for UDP)
0x00 0x00   reserved
0x0D 0xA4   internal port (3492 = 0x0DA4)
0x0D 0xA4   external port (3492)
0x00 0x01 0x51 0x80  lifetime (86400 seconds = 0x00015180)
```

**Response** (16 bytes):
```
0x00        version
0x82        result opcode (0x80 + request opcode)
0x00 0x00   result code (0=success)
...         seconds since epoch
0x0D 0xA4   internal port
0x0D 0xA4   mapped external port  
0x00 0x01 0x51 0x80  lifetime granted
```

### Protocol 3: PCP (RFC 6887) — Modern Alternative

Port Control Protocol (successor to NAT-PMP). Binary UDP to gateway:5351. More complex but handles IPv6, nested NATs, and offers THIRD_PARTY and FILTER options. Consider as future evolution if NAT-PMP is insufficient.

---

## JSON-RPC Methods

### `igd.discover`

Discover router IGD capabilities. Tries UPnP SSDP first, then NAT-PMP, then PCP.

```json
// Request
{"jsonrpc": "2.0", "method": "igd.discover", "params": {}, "id": 1}

// Response (UPnP found)
{"jsonrpc": "2.0", "result": {
  "protocol": "upnp_igd",
  "gateway_ip": "192.168.1.254",
  "control_url": "http://192.168.1.254:5431/ctl/IPConn",
  "external_ip": "162.226.225.148",
  "device_friendly_name": "BGW320-505",
  "manufacturer": "Arris",
  "capabilities": ["AddPortMapping", "DeletePortMapping", "GetExternalIPAddress"]
}, "id": 1}

// Response (NAT-PMP found)
{"jsonrpc": "2.0", "result": {
  "protocol": "nat_pmp",
  "gateway_ip": "192.168.1.254",
  "external_ip": "162.226.225.148",
  "epoch": 1707399600,
  "capabilities": ["MapPort", "GetExternalIP"]
}, "id": 1}

// Response (nothing found)
{"jsonrpc": "2.0", "result": {
  "protocol": "none",
  "gateway_ip": "192.168.1.254",
  "upnp_tried": true,
  "upnp_devices_found": ["printer (192.168.1.227)", "chromecast (192.168.1.100)"],
  "upnp_igd_found": false,
  "nat_pmp_tried": true,
  "nat_pmp_responded": false,
  "recommendation": "Enable UPnP on your router, or manually forward TCP port 3492 to 192.168.1.144",
  "manual_config": {
    "router_admin": "https://192.168.1.254",
    "section": "Firewall > NAT/Gaming",
    "rule": "TCP port 3492 -> 192.168.1.144:3492"
  }
}, "id": 1}
```

### `igd.map_port`

Request a port mapping from the router.

```json
// Request
{"jsonrpc": "2.0", "method": "igd.map_port", "params": {
  "external_port": 3492,
  "internal_port": 3492,
  "protocol": "TCP",
  "description": "Songbird sovereign beacon",
  "ttl": 86400
}, "id": 1}

// Response (success)
{"jsonrpc": "2.0", "result": {
  "mapped": true,
  "protocol_used": "upnp_igd",
  "external": "162.226.225.148:3492",
  "internal": "192.168.1.144:3492",
  "ttl": 86400,
  "description": "Songbird sovereign beacon"
}, "id": 1}

// Response (failure)
{"jsonrpc": "2.0", "error": {
  "code": -32001,
  "message": "Port mapping failed: 718 ConflictInMappingEntry",
  "data": {
    "conflicting_host": "192.168.1.100",
    "suggestion": "Try a different external port or remove existing mapping"
  }
}, "id": 1}
```

### `igd.unmap_port`

Remove a port mapping on shutdown.

```json
{"jsonrpc": "2.0", "method": "igd.unmap_port", "params": {
  "external_port": 3492,
  "protocol": "TCP"
}, "id": 1}

// Response
{"jsonrpc": "2.0", "result": {"unmapped": true}, "id": 1}
```

### `igd.status`

Report all current mappings and gateway state.

```json
{"jsonrpc": "2.0", "method": "igd.status", "params": {}, "id": 1}

// Response
{"jsonrpc": "2.0", "result": {
  "gateway_ip": "192.168.1.254",
  "external_ip": "162.226.225.148",
  "protocol": "upnp_igd",
  "mappings": [
    {
      "external_port": 3492,
      "internal_port": 3492,
      "internal_ip": "192.168.1.144",
      "protocol": "TCP",
      "description": "Songbird sovereign beacon",
      "ttl_remaining": 85200,
      "active": true
    }
  ],
  "auto_renew": true,
  "next_renewal_seconds": 43200
}, "id": 1}
```

### `igd.external_ip`

Quick method to get the external IP from the router (faster than STUN, no external traffic).

```json
{"jsonrpc": "2.0", "method": "igd.external_ip", "params": {}, "id": 1}

// Response
{"jsonrpc": "2.0", "result": {
  "external_ip": "162.226.225.148",
  "source": "upnp_igd"
}, "id": 1}
```

### `igd.auto_configure`

All-in-one: discover gateway, map the Songbird port, verify reachability.

```json
{"jsonrpc": "2.0", "method": "igd.auto_configure", "params": {
  "port": 3492,
  "protocol": "TCP",
  "verify": true
}, "id": 1}

// Response (full success)
{"jsonrpc": "2.0", "result": {
  "configured": true,
  "gateway": "192.168.1.254",
  "protocol_used": "upnp_igd",
  "external_endpoint": "162.226.225.148:3492",
  "verified_reachable": true,
  "auto_renew_enabled": true
}, "id": 1}

// Response (partial — mapped but can't verify)
{"jsonrpc": "2.0", "result": {
  "configured": true,
  "gateway": "192.168.1.254",
  "protocol_used": "nat_pmp",
  "external_endpoint": "162.226.225.148:3492",
  "verified_reachable": false,
  "verification_note": "Could not self-verify (hairpin NAT not supported). External peer should be able to reach this endpoint."
}, "id": 1}

// Response (gateway doesn't support auto-config)
{"jsonrpc": "2.0", "result": {
  "configured": false,
  "reason": "no_igd_support",
  "gateway": "192.168.1.254",
  "recommendation": "Enable UPnP on router, or manually forward TCP 3492 to 192.168.1.144",
  "fallback_tiers": [
    "IPv6 direct (if available): tower.nestgate.io AAAA -> [2600:1700:b0b0:5b90::27]:3492",
    "Sovereign onion: .onion address via onion.start",
    "STUN hole-punch: punch.request",
    "Family relay: mesh via other connected family device"
  ]
}, "id": 1}
```

---

## Implementation Structure

### `songbird-igd/src/lib.rs`

```rust
//! Sovereign Router Configuration — UPnP IGD + NAT-PMP
//!
//! Turns the router from a dependency into a tool Songbird configures.
//! Pure Rust, zero C dependencies, zero external crates.

pub mod ssdp;      // SSDP multicast discovery
pub mod soap;      // SOAP XML control messages  
pub mod nat_pmp;   // NAT-PMP binary protocol
pub mod gateway;   // Gateway abstraction over IGD/NAT-PMP
pub mod mapping;   // Port mapping management
pub mod renewal;   // TTL renewal task
pub mod error;     // Error types
```

### `songbird-igd/src/ssdp.rs`

```rust
//! SSDP Discovery — Find UPnP gateways on the LAN
//!
//! Sends M-SEARCH multicast to 239.255.255.250:1900,
//! parses responses for InternetGatewayDevice services.

use tokio::net::UdpSocket;
use std::time::Duration;

pub struct SsdpDiscovery {
    timeout: Duration,
}

pub struct SsdpResponse {
    pub location: String,       // Device description URL
    pub server: String,         // Server string
    pub service_type: String,   // ST header
    pub usn: String,            // Unique service name
}

impl SsdpDiscovery {
    pub fn new() -> Self {
        Self { timeout: Duration::from_secs(3) }
    }

    /// Send M-SEARCH and collect responses
    pub async fn discover_gateways(&self) -> Result<Vec<SsdpResponse>, SsdpError> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        
        // Enable multicast
        socket.set_broadcast(true)?;
        
        let search_msg = format!(
            "M-SEARCH * HTTP/1.1\r\n\
             HOST: 239.255.255.250:1900\r\n\
             MAN: \"ssdp:discover\"\r\n\
             MX: 3\r\n\
             ST: urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\n\
             \r\n"
        );
        
        socket.send_to(search_msg.as_bytes(), "239.255.255.250:1900").await?;
        
        let mut responses = Vec::new();
        let mut buf = [0u8; 2048];
        
        loop {
            tokio::select! {
                result = socket.recv_from(&mut buf) => {
                    if let Ok((len, _addr)) = result {
                        if let Some(resp) = Self::parse_response(&buf[..len]) {
                            responses.push(resp);
                        }
                    }
                }
                _ = tokio::time::sleep(self.timeout) => break,
            }
        }
        
        Ok(responses)
    }
    
    fn parse_response(data: &[u8]) -> Option<SsdpResponse> {
        let text = std::str::from_utf8(data).ok()?;
        // Parse HTTP-like headers: LOCATION, ST, USN, SERVER
        // ...
        todo!("Parse SSDP response headers")
    }
}
```

### `songbird-igd/src/soap.rs`

```rust
//! SOAP Control — Send actions to UPnP gateway
//!
//! Uses Songbird's HTTP client (songbird-http-client) for the actual HTTP requests.
//! Builds and parses SOAP XML envelopes.

pub struct SoapClient {
    control_url: String,
    service_type: String,
}

impl SoapClient {
    /// Send AddPortMapping action
    pub async fn add_port_mapping(&self, mapping: &PortMappingRequest) -> Result<(), SoapError> {
        let body = format!(
            r#"<?xml version="1.0"?>
            <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"
                        s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
              <s:Body>
                <u:AddPortMapping xmlns:u="{}">
                  <NewRemoteHost></NewRemoteHost>
                  <NewExternalPort>{}</NewExternalPort>
                  <NewProtocol>{}</NewProtocol>
                  <NewInternalPort>{}</NewInternalPort>
                  <NewInternalClient>{}</NewInternalClient>
                  <NewEnabled>1</NewEnabled>
                  <NewPortMappingDescription>{}</NewPortMappingDescription>
                  <NewLeaseDuration>{}</NewLeaseDuration>
                </u:AddPortMapping>
              </s:Body>
            </s:Envelope>"#,
            self.service_type,
            mapping.external_port,
            mapping.protocol,
            mapping.internal_port,
            mapping.internal_client,
            mapping.description,
            mapping.lease_duration
        );
        
        // POST via songbird-http-client with SOAPAction header
        // ...
        todo!()
    }
    
    /// Send GetExternalIPAddress action
    pub async fn get_external_ip(&self) -> Result<String, SoapError> {
        todo!()
    }
    
    /// Send DeletePortMapping action
    pub async fn delete_port_mapping(&self, external_port: u16, protocol: &str) -> Result<(), SoapError> {
        todo!()
    }
}
```

### `songbird-igd/src/nat_pmp.rs`

```rust
//! NAT-PMP — Simple binary protocol for Apple/compatible routers
//!
//! Send UDP packets to gateway:5351. Much simpler than UPnP.

use tokio::net::UdpSocket;
use std::net::SocketAddr;

pub struct NatPmpClient {
    gateway: SocketAddr,
}

impl NatPmpClient {
    pub fn new(gateway_ip: std::net::IpAddr) -> Self {
        Self {
            gateway: SocketAddr::new(gateway_ip, 5351),
        }
    }
    
    /// Get external IP address
    pub async fn get_external_ip(&self) -> Result<std::net::Ipv4Addr, NatPmpError> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        
        // Request: version=0, opcode=0 (2 bytes)
        socket.send_to(&[0x00, 0x00], self.gateway).await?;
        
        let mut buf = [0u8; 12];
        let (len, _) = tokio::time::timeout(
            std::time::Duration::from_secs(3),
            socket.recv_from(&mut buf)
        ).await??;
        
        if len >= 12 && buf[1] == 128 && buf[2] == 0 && buf[3] == 0 {
            // Success: IP at bytes 8-11
            Ok(std::net::Ipv4Addr::new(buf[8], buf[9], buf[10], buf[11]))
        } else {
            Err(NatPmpError::BadResponse)
        }
    }
    
    /// Request port mapping
    pub async fn map_port(
        &self,
        internal_port: u16,
        external_port: u16,
        protocol: Protocol,
        lifetime: u32,
    ) -> Result<MappingResponse, NatPmpError> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        
        let opcode = match protocol {
            Protocol::Udp => 0x01,
            Protocol::Tcp => 0x02,
        };
        
        let mut request = [0u8; 12];
        request[0] = 0x00;  // version
        request[1] = opcode;
        // bytes 2-3: reserved (0x00)
        request[4..6].copy_from_slice(&internal_port.to_be_bytes());
        request[6..8].copy_from_slice(&external_port.to_be_bytes());
        request[8..12].copy_from_slice(&lifetime.to_be_bytes());
        
        socket.send_to(&request, self.gateway).await?;
        
        let mut buf = [0u8; 16];
        let (len, _) = tokio::time::timeout(
            std::time::Duration::from_secs(3),
            socket.recv_from(&mut buf)
        ).await??;
        
        if len >= 16 && buf[2] == 0 && buf[3] == 0 {
            Ok(MappingResponse {
                internal_port: u16::from_be_bytes([buf[8], buf[9]]),
                external_port: u16::from_be_bytes([buf[10], buf[11]]),
                lifetime: u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]),
            })
        } else {
            Err(NatPmpError::MappingFailed(u16::from_be_bytes([buf[2], buf[3]])))
        }
    }
}
```

### `songbird-igd/src/gateway.rs`

```rust
//! Unified Gateway — Abstraction over UPnP IGD and NAT-PMP
//!
//! Tries UPnP IGD first (most common), falls back to NAT-PMP.

pub enum GatewayProtocol {
    UpnpIgd { control_url: String, service_type: String },
    NatPmp,
    None,
}

pub struct Gateway {
    pub ip: std::net::IpAddr,
    pub protocol: GatewayProtocol,
    pub external_ip: Option<std::net::IpAddr>,
    pub device_name: Option<String>,
}

impl Gateway {
    /// Discover the best available gateway protocol
    pub async fn discover() -> Result<Self, GatewayError> {
        let gateway_ip = Self::get_default_gateway()?;
        
        // Try UPnP IGD first
        if let Ok(upnp) = Self::try_upnp_igd(gateway_ip).await {
            return Ok(upnp);
        }
        
        // Fall back to NAT-PMP
        if let Ok(natpmp) = Self::try_nat_pmp(gateway_ip).await {
            return Ok(natpmp);
        }
        
        // Nothing available
        Ok(Self {
            ip: gateway_ip,
            protocol: GatewayProtocol::None,
            external_ip: None,
            device_name: None,
        })
    }
    
    /// Get default gateway IP from routing table
    fn get_default_gateway() -> Result<std::net::IpAddr, GatewayError> {
        // Read /proc/net/route on Linux, parse for default route
        // This avoids any external command dependency
        todo!()
    }
}
```

---

## Startup Integration

### Automatic (Preferred)

When `SONGBIRD_IGD_ENABLED=true` (or `SONGBIRD_AUTO_PORT_FORWARD=true`):

1. On startup, after binding to `:3492`, Songbird calls `igd.auto_configure`
2. If IGD/NAT-PMP succeeds: port is forwarded, beacon gets updated with verified external endpoint
3. If it fails: log a clear message with manual instructions, continue with other tiers
4. Spawn a renewal task that refreshes the mapping at half the TTL interval
5. On graceful shutdown: call `igd.unmap_port` to clean up

### Manual (via start_nucleus.sh)

```bash
# After Songbird starts, request IGD auto-configuration
IGD_RESULT=$(echo '{"jsonrpc":"2.0","method":"igd.auto_configure","params":{"port":3492,"protocol":"TCP","verify":true},"id":1}' | \
    nc -U /run/user/1000/biomeos/songbird.sock -w 10 -q 3)

if echo "$IGD_RESULT" | grep -q '"configured":true'; then
    echo "✅ Router port forwarding configured automatically"
else
    echo "⚠️  Router auto-config not available"
    echo "   Manual: Forward TCP 3492 to $(hostname -I | awk '{print $1}'):3492"
fi
```

---

## Renewal Task

Port mappings have TTLs (typically 86400 seconds / 24 hours for UPnP, varies for NAT-PMP). Songbird must renew them:

```rust
/// Spawn a background task that renews port mappings
fn spawn_renewal_task(gateway: Arc<Gateway>, mapping: PortMapping) {
    let renewal_interval = mapping.ttl / 2; // Renew at half TTL
    
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(renewal_interval as u64)).await;
            
            match gateway.renew_mapping(&mapping).await {
                Ok(new_ttl) => {
                    tracing::info!("🔄 Port mapping renewed (TTL: {}s)", new_ttl);
                }
                Err(e) => {
                    tracing::warn!("⚠️  Port mapping renewal failed: {}. Will retry.", e);
                    // Retry with exponential backoff
                }
            }
        }
    });
}
```

---

## Error Handling and User Guidance

When IGD is not available, Songbird should be maximally helpful:

```json
{
  "configured": false,
  "reason": "no_igd_support",
  "diagnostics": {
    "gateway_ip": "192.168.1.254",
    "gateway_reachable": true,
    "gateway_http": true,
    "gateway_admin_url": "https://192.168.1.254",
    "upnp_ssdp_sent": true,
    "upnp_responses": [
      {"device": "DPSIPPRINTER", "ip": "192.168.1.227", "is_igd": false},
      {"device": "Chromecast", "ip": "192.168.1.100", "is_igd": false}
    ],
    "upnp_igd_found": false,
    "nat_pmp_sent": true,
    "nat_pmp_response": false
  },
  "manual_instructions": {
    "steps": [
      "1. Open https://192.168.1.254 in a browser",
      "2. Log in to your router admin panel",
      "3. Navigate to Firewall > NAT/Gaming (or Port Forwarding)",
      "4. Add rule: TCP port 3492 -> 192.168.1.144:3492",
      "5. Save and apply",
      "6. Optionally: Enable UPnP in the router settings for auto-config next time"
    ],
    "router_type_hint": "AT&T gateway (Pace/Arris)"
  },
  "alternative_tiers": [
    "IPv6 direct via tower.nestgate.io AAAA record (works if peer has IPv6)",
    "Sovereign onion via onion.start (works everywhere, no port forward needed)",
    "STUN hole-punch via punch.request (works for non-symmetric NAT)",
    "Family relay via mesh (works if another family device has connectivity)"
  ]
}
```

---

## Testing Today's Scenario

After implementing `songbird-igd`, re-run this exact scenario:

```bash
# 1. Start Songbird on Tower with IGD enabled
SONGBIRD_IGD_ENABLED=true songbird server --port 3492 ...

# 2. Songbird automatically discovers gateway, attempts port mapping
# Expected: "Router auto-config not available — no IGD support"
# Because the AT&T gateway at 192.168.1.254 doesn't have UPnP enabled

# 3. Songbird provides clear manual instructions
# 4. User enables UPnP on router OR manually forwards port
# 5. Re-run igd.auto_configure — now it maps the port
# 6. From Pixel: nc -z -w 5 162.226.225.148 3492 → OPEN

# 7. Full NAT traversal test passes:
adb shell "sh /data/local/tmp/biomeos/test_nat_traversal.sh"
# → All 11 tests PASS
```

---

## Cross-Architecture Notes

### Linux (Tower, x86_64)
- SSDP multicast works normally
- `/proc/net/route` for default gateway discovery
- No special considerations

### Linux (USB, x86_64 / aarch64)
- Same as Tower
- May be on a different subnet if USB is a separate network device

### Android (Pixel 8a, aarch64)
- SELinux may restrict UDP multicast on `239.255.255.250`
- `/proc/net/route` readable but format may differ
- Gateway discovery: `ip route` output or `getprop dhcp.wlan0.gateway`
- On Android, the Pixel is usually the CLIENT needing to reach the Tower, not a server needing port forwarding. IGD on Android is lower priority.
- **Primary use**: The Pixel uses `igd.discover` to DETECT its NAT environment, not to configure it

### When to Use IGD vs Other Tiers

| Scenario | Best Tier | Why |
|----------|-----------|-----|
| Tower behind home router | IGD | Configure port forward automatically |
| Pixel on mobile hotspot | Onion/STUN | Can't configure carrier NAT |
| USB on LAN | LAN Direct | Same subnet, no NAT |
| Both behind symmetric NAT | Relay | Hole-punch won't work |
| ISP blocks all inbound | Onion | Overlay bypasses ISP restrictions |

---

## Files to Create/Modify

### New Files (in `phase1/songbird/`)
- `crates/songbird-igd/Cargo.toml`
- `crates/songbird-igd/src/lib.rs`
- `crates/songbird-igd/src/ssdp.rs`
- `crates/songbird-igd/src/soap.rs`
- `crates/songbird-igd/src/nat_pmp.rs`
- `crates/songbird-igd/src/gateway.rs`
- `crates/songbird-igd/src/mapping.rs`
- `crates/songbird-igd/src/renewal.rs`
- `crates/songbird-igd/src/error.rs`

### Modified Files
- `crates/songbird-universal-ipc/src/handlers/mod.rs` — add `pub mod igd_handler;`
- `crates/songbird-universal-ipc/src/handlers/igd_handler.rs` — new handler (follows `stun_handler.rs` pattern)
- `crates/songbird-universal-ipc/src/service.rs` — wire `igd.*` methods into `handle()` match
- `crates/songbird-universal-ipc/Cargo.toml` — add `songbird-igd` dependency
- `Cargo.toml` (workspace) — add `songbird-igd` to workspace members

---

## Also Fix: Android IPC Gap

During this session, we confirmed that Songbird's `songbird-universal-ipc` service layer (which provides STUN, Onion, Mesh, Birdsong methods) does NOT have a TCP fallback when Unix sockets fail on Android due to SELinux.

The orchestrator's TCP fallback at `:44493` (or `:9901`) only handles orchestrator-level methods, not the full universal-ipc method set.

**Fix**: When Songbird detects Unix socket failure (Android), bind the universal-ipc service to a TCP port (e.g., `:9902`) alongside the orchestrator's TCP port. This would make ALL methods (including `stun.*`, `onion.*`, `mesh.*`, `igd.*`) available on Android.

This is filed as a separate but related issue.

---

## Current Network State (Reference)

```
Tower (gate):
  LAN IP:     192.168.1.144
  Public IP:  162.226.225.148 (via STUN)
  Gateway:    192.168.1.254 (AT&T, no UPnP)
  Songbird:   *:3492 (TCP, dual-stack IPv6+IPv4)
  STUN:       0.0.0.0:3478 (UDP)
  BearDog:    127.0.0.1:9900 (Unix socket)
  Onion:      p6m5exqn44xpjtvpal6juhdzh3s7zvlpysrjcknzrxada6mny54ltiyd.onion:3492
  DNS:        tower.nestgate.io A + AAAA, beacon.nestgate.io TXT (encrypted)
  Family ID:  8ff3b864a4bc589a (from .family.seed)

Pixel (pixel8a):
  Hotspot IP: 172.20.10.2/28
  Carrier:    IWLAN (WiFi tethered to cellular)
  BearDog:    127.0.0.1:9900 (TCP, Android)
  Songbird:   127.0.0.1:9901 (orchestrator TCP fallback)
  Family ID:  8ff3b864a4bc589a (same seed)

Validated Crypto Chain:
  .family.seed (32 bytes) -> HMAC-SHA256(seed, "beacon_shared_key") -> shared_key
  Tower: shared_key = "+ljDOjXiYm9V5h2LX4v1luyONzjErIg9r0YI7b2zTRQ="
  Pixel: shared_key = "+ljDOjXiYm9V5h2LX4v1luyONzjErIg9r0YI7b2zTRQ="
  MATCH: ✅ Identical on x86_64 and aarch64
  
  ChaCha20-Poly1305 decrypt of beacon.nestgate.io TXT:
  Tower encrypted -> DNS -> Pixel decrypted -> IDENTICAL JSON payload ✅
```

---

## Priority

**This is the #1 evolution for Songbird.** Every other tier (IPv6, onion, STUN, relay) works or is close. The router is the last piece of manual configuration. Making Songbird speak IGD/NAT-PMP turns deployment into a zero-touch experience.

Implementation order:
1. `ssdp.rs` — discover gateway (simplest, most impactful)
2. `soap.rs` — AddPortMapping/DeletePortMapping
3. `igd_handler.rs` — wire into IPC
4. `nat_pmp.rs` — fallback for non-UPnP routers
5. `renewal.rs` — TTL management
6. `gateway.rs` — unified abstraction
7. Auto-configure on startup

Estimated effort: 2-3 focused sessions.
