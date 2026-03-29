// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

use super::PrimalCapability;
use biomeos_types::constants::ports;
use biomeos_types::{Health, JsonRpcRequest, JsonRpcResponse, PrimalType};
use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpStream, UdpSocket};
use tokio::time::timeout;

const MDNS_ADDR: &str = "224.0.0.251:5353";
const SERVICE_PTR: &str = "_biomeos._tcp.local";
const DNS_TYPE_PTR: u16 = 12;
const DNS_TYPE_SRV: u16 = 33;
const DNS_TYPE_TXT: u16 = 16;
const DNS_CLASS_IN: u16 = 1;

#[derive(Debug, Default, Clone)]
struct InstanceRecord {
    instance_fqdn: String,
    srv_target: Option<String>,
    srv_port: Option<u16>,
    txt: HashMap<String, String>,
}

pub(super) async fn discover_dns_sd_services() -> HashMap<String, serde_json::Value> {
    let mut out = HashMap::new();
    let mut seen_endpoints: HashSet<(String, u16)> = HashSet::new();

    let mut candidates = Vec::new();
    match browse_mdns_instances().await {
        Ok(v) => candidates.extend(v),
        Err(e) => {
            tracing::warn!(error = %e, "mDNS browse failed; continuing with fallback only");
        }
    }
    candidates.extend(fallback_lan_candidates());

    for c in candidates {
        let key = (c.host.clone(), c.port);
        if !seen_endpoints.insert(key) {
            continue;
        }
        let connect_timeout = if c.fallback_scan {
            Duration::from_millis(800)
        } else {
            Duration::from_secs(3)
        };
        match probe_liveness_jsonrpc(&c.host, c.port, connect_timeout).await {
            Ok((probe_ok, raw)) => {
                if !probe_ok {
                    tracing::debug!(
                        host = %c.host,
                        port = c.port,
                        "health.liveness probe did not confirm a live primal"
                    );
                    continue;
                }
                let entry = build_entry(&c, &raw);
                out.insert(entry.id, entry.value);
            }
            Err(e) => {
                tracing::debug!(
                    host = %c.host,
                    port = c.port,
                    error = %e,
                    "liveness probe failed"
                );
            }
        }
    }

    tracing::info!("DNS-SD discovery completed with {} service(s)", out.len());
    out
}

struct Candidate {
    host: String,
    port: u16,
    instance_fqdn: String,
    txt: HashMap<String, String>,
    /// Subnet / localhost probes use shorter TCP timeouts than mDNS-derived targets.
    fallback_scan: bool,
}

struct BuiltEntry {
    id: String,
    value: serde_json::Value,
}

fn build_entry(c: &Candidate, liveness: &serde_json::Value) -> BuiltEntry {
    let discovered = chrono::Utc::now();
    let name = c
        .txt
        .get("name")
        .cloned()
        .or_else(|| instance_display_name(&c.instance_fqdn))
        .unwrap_or_else(|| c.instance_fqdn.clone());
    let family_id = c.txt.get("family_id").cloned().unwrap_or_default();
    let id = if family_id.is_empty() {
        format!("dns-{}-{}", c.host, c.port)
    } else {
        format!("dns-{family_id}-{}", c.port)
    };
    let primal_type_name = c
        .txt
        .get("primal_type")
        .cloned()
        .unwrap_or_else(|| "discovered".to_string());
    let primal_type = PrimalType::new("network", primal_type_name, "0.0.0");
    let caps = parse_capabilities_txt(c.txt.get("capabilities"));
    let health = Health::Healthy;
    let endpoint = format!("tcp://{}:{}", c.host, c.port);
    BuiltEntry {
        id,
        value: serde_json::json!({
            "name": name,
            "type": primal_type,
            "endpoint": endpoint,
            "health": health,
            "capabilities": caps,
            "discovered_at": discovered,
            "dns_instance": c.instance_fqdn,
            "dns_txt": c.txt,
            "liveness": liveness,
        }),
    }
}

fn instance_display_name(fqdn: &str) -> Option<String> {
    let suffix = format!(".{SERVICE_PTR}");
    fqdn.strip_suffix(&suffix)
        .map(|s| s.trim_end_matches('.').to_string())
}

fn parse_capabilities_txt(raw: Option<&String>) -> Vec<PrimalCapability> {
    let Some(s) = raw else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for part in s.split([',', ';']) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let mut it = part.split(':');
        let a = it.next().unwrap_or("").trim();
        let b = it.next().unwrap_or("").trim();
        let v = it.next().unwrap_or("1.0").trim();
        if a.is_empty() {
            continue;
        }
        if b.is_empty() {
            out.push(PrimalCapability::new("general", a, v));
        } else {
            out.push(PrimalCapability::new(a, b, v));
        }
    }
    out
}

fn liveness_ok(resp: &serde_json::Value) -> bool {
    if resp.get("error").is_some() {
        return false;
    }
    if let Some(r) = resp.get("result") {
        if let Some(s) = r.get("status").and_then(|v| v.as_str()) {
            return matches!(s, "alive" | "healthy" | "ok");
        }
        return true;
    }
    false
}

fn tcp_connect_addr(host: &str, port: u16) -> String {
    if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    }
}

async fn probe_liveness_jsonrpc(
    host: &str,
    port: u16,
    connect_timeout: Duration,
) -> anyhow::Result<(bool, serde_json::Value)> {
    let addr = tcp_connect_addr(host, port);
    let connect_fut = TcpStream::connect(&addr);
    let mut stream = match timeout(connect_timeout, connect_fut).await {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => return Err(e.into()),
        Err(_) => anyhow::bail!("TCP connect timed out"),
    };

    let req = JsonRpcRequest::new("health.liveness", serde_json::json!({}));
    let request_str = serde_json::to_string(&req)?;
    stream.write_all(request_str.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    let read_timeout = connect_timeout.max(Duration::from_secs(1));
    let (read_half, _) = stream.into_split();
    let mut reader = BufReader::new(read_half);
    let mut line = String::new();
    timeout(read_timeout, reader.read_line(&mut line))
        .await
        .map_err(|_| anyhow::anyhow!("read timed out"))??;

    let resp: JsonRpcResponse = serde_json::from_str(line.trim()).map_err(|e| {
        anyhow::anyhow!(
            "invalid JSON-RPC response: {e}; line={}",
            &line[..line.len().min(200)]
        )
    })?;
    let as_value = serde_json::to_value(&resp)?;
    let ok = liveness_ok(&as_value);
    Ok((ok, as_value))
}

async fn browse_mdns_instances() -> anyhow::Result<Vec<Candidate>> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let mdns: SocketAddr = MDNS_ADDR.parse()?;
    let id: u16 = rand::random::<u16>();
    let query = build_ptr_query(SERVICE_PTR, id);
    if let Err(e) = socket.send_to(&query, mdns).await {
        tracing::debug!(error = %e, "mDNS send failed");
        return Ok(Vec::new());
    }

    let mut buf = vec![0u8; 9000];
    let mut instances: HashMap<String, InstanceRecord> = HashMap::new();
    let deadline = Instant::now() + Duration::from_millis(2000);
    while Instant::now() < deadline {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            break;
        }
        match timeout(
            left.min(Duration::from_millis(200)),
            socket.recv_from(&mut buf),
        )
        .await
        {
            Ok(Ok((n, _))) => {
                if let Ok(Some(parsed)) = parse_dns_packet(&buf[..n]) {
                    merge_records(&mut instances, &buf[..n], &parsed);
                }
            }
            Ok(Err(e)) => {
                tracing::debug!(error = %e, "mDNS recv error");
                break;
            }
            Err(_) => {}
        }
    }

    for rec in instances.values_mut() {
        if rec.srv_port.is_none() || rec.srv_target.is_none() {
            if let Ok(Some((target, port))) = query_srv(&socket, &rec.instance_fqdn).await {
                rec.srv_target.get_or_insert(target);
                rec.srv_port.get_or_insert(port);
            } else {
                tracing::debug!(
                    instance = %rec.instance_fqdn,
                    "supplemental SRV query did not return data"
                );
            }
        }
    }

    let mut out = Vec::new();
    for (_key, rec) in instances {
        let Some(target) = rec.srv_target else {
            tracing::debug!(instance = %rec.instance_fqdn, "SRV target missing for instance");
            continue;
        };
        let Some(port) = rec.srv_port else {
            tracing::debug!(instance = %rec.instance_fqdn, "SRV port missing for instance");
            continue;
        };
        let host = normalize_target_host(&target);
        out.push(Candidate {
            host,
            port,
            instance_fqdn: rec.instance_fqdn,
            txt: rec.txt,
            fallback_scan: false,
        });
    }
    Ok(out)
}

fn normalize_target_host(target: &str) -> String {
    target.trim_end_matches('.').to_string()
}

async fn query_srv(socket: &UdpSocket, instance: &str) -> anyhow::Result<Option<(String, u16)>> {
    let id: u16 = rand::random::<u16>();
    let q = build_query(instance, id, DNS_TYPE_SRV);
    let mdns: SocketAddr = MDNS_ADDR.parse()?;
    socket.send_to(&q, mdns).await?;

    let mut buf = vec![0u8; 9000];
    let deadline = Instant::now() + Duration::from_millis(800);
    while Instant::now() < deadline {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            break;
        }
        match timeout(
            left.min(Duration::from_millis(200)),
            socket.recv_from(&mut buf),
        )
        .await
        {
            Ok(Ok((n, _))) => {
                if let Ok(Some(parsed)) = parse_dns_packet(&buf[..n]) {
                    let slice = &buf[..n];
                    for rr in parsed
                        .answers
                        .iter()
                        .chain(parsed.authority.iter())
                        .chain(parsed.additional.iter())
                    {
                        if rr.rtype == DNS_TYPE_SRV && rr.name.eq_ignore_ascii_case(instance) {
                            if let Ok((target, port)) =
                                parse_srv_rdata(slice, rr.rdata_off, rr.rdata_len)
                            {
                                return Ok(Some((target, port)));
                            }
                        }
                    }
                }
            }
            Ok(Err(_)) => break,
            Err(_) => {}
        }
    }
    Ok(None)
}

fn fallback_lan_candidates() -> Vec<Candidate> {
    let mut v = Vec::new();
    let base_port = ports::TCP_PORT_SCAN_START;
    for host in &["127.0.0.1", "::1"] {
        v.push(Candidate {
            host: (*host).to_string(),
            port: base_port,
            instance_fqdn: format!("fallback.{SERVICE_PTR}"),
            txt: HashMap::new(),
            fallback_scan: true,
        });
    }
    if let Some(ip) = local_ipv4_via_udp() {
        let octets = ip.octets();
        let prefix = [octets[0], octets[1], octets[2]];
        let last = octets[3];
        let lo = last.saturating_sub(2);
        let hi = last.saturating_add(2).min(254);
        for b in lo..=hi {
            let addr = Ipv4Addr::new(prefix[0], prefix[1], prefix[2], b);
            v.push(Candidate {
                host: addr.to_string(),
                port: base_port,
                instance_fqdn: format!("fallback.{SERVICE_PTR}"),
                txt: HashMap::new(),
                fallback_scan: true,
            });
        }
    }
    v
}

fn local_ipv4_via_udp() -> Option<Ipv4Addr> {
    let s = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    s.connect("8.8.8.8:80").ok()?;
    match s.local_addr().ok()?.ip() {
        IpAddr::V4(v4) => Some(v4),
        IpAddr::V6(_) => None,
    }
}

fn build_ptr_query(service: &str, id: u16) -> Vec<u8> {
    build_query(service, id, DNS_TYPE_PTR)
}

fn build_query(qname: &str, id: u16, qtype: u16) -> Vec<u8> {
    let mut qname_bytes = Vec::new();
    for label in qname.split('.') {
        if label.is_empty() {
            continue;
        }
        let bytes = label.as_bytes();
        if bytes.len() > 63 {
            continue;
        }
        qname_bytes.push(bytes.len() as u8);
        qname_bytes.extend_from_slice(bytes);
    }
    qname_bytes.push(0);

    let mut pkt = Vec::with_capacity(12 + qname_bytes.len() + 4);
    pkt.extend_from_slice(&id.to_be_bytes());
    pkt.extend_from_slice(&[0x00, 0x00]);
    pkt.extend_from_slice(&[0x00, 0x01]);
    pkt.extend_from_slice(&[0x00, 0x00]);
    pkt.extend_from_slice(&[0x00, 0x00]);
    pkt.extend_from_slice(&[0x00, 0x00]);
    pkt.extend_from_slice(&qname_bytes);
    pkt.extend_from_slice(&qtype.to_be_bytes());
    pkt.extend_from_slice(&DNS_CLASS_IN.to_be_bytes());
    pkt
}

struct DnsPacket {
    answers: Vec<RrLoc>,
    authority: Vec<RrLoc>,
    additional: Vec<RrLoc>,
}

/// Resource record with offsets into the original DNS datagram (`rdata` compression).
struct RrLoc {
    name: String,
    rtype: u16,
    rdata_off: usize,
    rdata_len: usize,
}

fn parse_dns_packet(buf: &[u8]) -> Result<Option<DnsPacket>, ()> {
    if buf.len() < 12 {
        return Err(());
    }
    let flags = u16::from_be_bytes([buf[2], buf[3]]);
    if flags & 0x8000 == 0 {
        return Ok(None);
    }
    let qdcount = u16::from_be_bytes([buf[4], buf[5]]) as usize;
    let ancount = u16::from_be_bytes([buf[6], buf[7]]) as usize;
    let nscount = u16::from_be_bytes([buf[8], buf[9]]) as usize;
    let arcount = u16::from_be_bytes([buf[10], buf[11]]) as usize;
    let mut pos = 12usize;
    for _ in 0..qdcount {
        let (_, p) = read_domain_name(buf, pos)?;
        pos = p + 4;
    }
    let mut answers = Vec::new();
    for _ in 0..ancount {
        let (rec, p) = parse_rr(buf, pos)?;
        answers.push(rec);
        pos = p;
    }
    let mut authority = Vec::new();
    for _ in 0..nscount {
        let (rec, p) = parse_rr(buf, pos)?;
        authority.push(rec);
        pos = p;
    }
    let mut additional = Vec::new();
    for _ in 0..arcount {
        let (rec, p) = parse_rr(buf, pos)?;
        additional.push(rec);
        pos = p;
    }
    Ok(Some(DnsPacket {
        answers,
        authority,
        additional,
    }))
}

fn parse_rr(buf: &[u8], pos: usize) -> Result<(RrLoc, usize), ()> {
    let (name, mut pos) = read_domain_name(buf, pos)?;
    if pos + 10 > buf.len() {
        return Err(());
    }
    let rtype = u16::from_be_bytes([buf[pos], buf[pos + 1]]);
    pos += 8;
    let rdlen = u16::from_be_bytes([buf[pos], buf[pos + 1]]) as usize;
    pos += 2;
    if pos + rdlen > buf.len() {
        return Err(());
    }
    let rdata_off = pos;
    pos += rdlen;
    Ok((
        RrLoc {
            name,
            rtype,
            rdata_off,
            rdata_len: rdlen,
        },
        pos,
    ))
}

fn instance_key(name: &str) -> String {
    name.trim_end_matches('.').to_ascii_lowercase()
}

fn merge_records(instances: &mut HashMap<String, InstanceRecord>, buf: &[u8], pkt: &DnsPacket) {
    let service_suffix = SERVICE_PTR.to_ascii_lowercase();
    for rr in pkt
        .answers
        .iter()
        .chain(pkt.authority.iter())
        .chain(pkt.additional.iter())
    {
        let owner_lc = rr.name.to_ascii_lowercase();
        if rr.rtype == DNS_TYPE_PTR && owner_lc == service_suffix {
            if let Ok((target, _)) = read_domain_name(buf, rr.rdata_off) {
                let key = instance_key(&target);
                let entry = instances.entry(key).or_insert_with(|| InstanceRecord {
                    instance_fqdn: target.clone(),
                    ..Default::default()
                });
                entry.instance_fqdn = target;
            }
        }
    }

    for rr in pkt
        .answers
        .iter()
        .chain(pkt.authority.iter())
        .chain(pkt.additional.iter())
    {
        let name_lc = rr.name.to_ascii_lowercase();
        if rr.rtype == DNS_TYPE_SRV
            && name_lc.ends_with("._tcp.local")
            && name_lc.contains("_biomeos")
        {
            let Ok((target, port)) = parse_srv_rdata(buf, rr.rdata_off, rr.rdata_len) else {
                continue;
            };
            let key = instance_key(&rr.name);
            if let Some(rec) = instances.get_mut(&key) {
                rec.srv_target = Some(target);
                rec.srv_port = Some(port);
            } else {
                instances.insert(
                    key,
                    InstanceRecord {
                        instance_fqdn: rr.name.clone(),
                        srv_target: Some(target),
                        srv_port: Some(port),
                        txt: HashMap::new(),
                    },
                );
            }
        }
    }

    for rr in pkt
        .answers
        .iter()
        .chain(pkt.authority.iter())
        .chain(pkt.additional.iter())
    {
        let name_lc = rr.name.to_ascii_lowercase();
        if rr.rtype == DNS_TYPE_TXT
            && name_lc.ends_with("._tcp.local")
            && name_lc.contains("_biomeos")
        {
            let parsed = parse_txt_rdata(&buf[rr.rdata_off..rr.rdata_off + rr.rdata_len]);
            let key = instance_key(&rr.name);
            if let Some(rec) = instances.get_mut(&key) {
                for (k, v) in parsed {
                    rec.txt.insert(k, v);
                }
            } else {
                instances.insert(
                    key,
                    InstanceRecord {
                        instance_fqdn: rr.name.clone(),
                        txt: parsed,
                        ..Default::default()
                    },
                );
            }
        }
    }
}

fn parse_srv_rdata(buf: &[u8], rdata_off: usize, rdlen: usize) -> Result<(String, u16), ()> {
    if rdlen < 6 || rdata_off + rdlen > buf.len() {
        return Err(());
    }
    let port = u16::from_be_bytes([buf[rdata_off + 4], buf[rdata_off + 5]]);
    let (target, _) = read_domain_name(buf, rdata_off + 6)?;
    Ok((normalize_target_host(&target), port))
}

fn parse_txt_rdata(rdata: &[u8]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut pos = 0usize;
    while pos < rdata.len() {
        let len = rdata[pos] as usize;
        pos += 1;
        if pos + len > rdata.len() {
            break;
        }
        let chunk = &rdata[pos..pos + len];
        pos += len;
        let Ok(s) = std::str::from_utf8(chunk) else {
            continue;
        };
        if let Some((k, v)) = s.split_once('=') {
            map.insert(k.to_string(), v.to_string());
        } else {
            map.insert(s.to_string(), String::new());
        }
    }
    map
}

fn read_domain_name(buf: &[u8], mut pos: usize) -> Result<(String, usize), ()> {
    let mut labels: Vec<String> = Vec::new();
    let mut jumped = false;
    let mut end_pos = 0usize;
    let mut steps = 0usize;

    loop {
        if steps > 64 || pos >= buf.len() {
            return Err(());
        }
        steps += 1;
        let len = buf[pos];
        if len == 0 {
            if !jumped {
                end_pos = pos + 1;
            }
            break;
        }
        if (len & 0xC0) == 0xC0 {
            if pos + 1 >= buf.len() {
                return Err(());
            }
            if !jumped {
                end_pos = pos + 2;
            }
            jumped = true;
            pos = (((len as usize) & 0x3F) << 8) | (buf[pos + 1] as usize);
            continue;
        }
        let label_len = len as usize;
        pos += 1;
        if pos + label_len > buf.len() {
            return Err(());
        }
        let label = std::str::from_utf8(&buf[pos..pos + label_len]).map_err(|_| ())?;
        labels.push(label.to_string());
        pos += label_len;
    }

    Ok((labels.join("."), end_pos))
}
