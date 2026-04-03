// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test")]

use super::*;
use std::collections::HashMap;

fn encode_domain(name: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for label in name.split('.') {
        if label.is_empty() {
            continue;
        }
        let b = label.as_bytes();
        assert!(b.len() <= 63, "label too long");
        out.push(b.len() as u8);
        out.extend_from_slice(b);
    }
    out.push(0);
    out
}

fn dns_header(id: u16, is_response: bool, qd: u16, an: u16, ns: u16, ar: u16) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(&id.to_be_bytes());
    let flags = if is_response { 0x8180u16 } else { 0u16 };
    v.extend_from_slice(&flags.to_be_bytes());
    v.extend_from_slice(&qd.to_be_bytes());
    v.extend_from_slice(&an.to_be_bytes());
    v.extend_from_slice(&ns.to_be_bytes());
    v.extend_from_slice(&ar.to_be_bytes());
    v
}

#[test]
fn parse_dns_packet_too_short_is_err() {
    assert!(parse_dns_packet(&[0u8; 11]).is_err());
}

#[test]
fn parse_dns_packet_query_returns_none() {
    let mut p = dns_header(1, false, 0, 0, 0, 0);
    assert!(parse_dns_packet(&p).unwrap().is_none());
    p[2] = 0x81;
    p[3] = 0x80;
    let pkt = parse_dns_packet(&p).unwrap().unwrap();
    assert!(pkt.answers.is_empty());
}

#[test]
fn parse_dns_packet_skips_question_section() {
    let mut p = dns_header(42, true, 1, 0, 0, 0);
    p.extend(encode_domain("example.com"));
    p.extend_from_slice(&DNS_TYPE_PTR.to_be_bytes());
    p.extend_from_slice(&DNS_CLASS_IN.to_be_bytes());
    let pkt = parse_dns_packet(&p).unwrap().unwrap();
    assert!(pkt.answers.is_empty());
}

#[test]
fn parse_dns_packet_reads_answer_ptr_record() {
    let mut p = dns_header(7, true, 0, 1, 0, 0);
    let owner = encode_domain("_biomeos._tcp.local");
    p.extend(owner);
    p.extend_from_slice(&DNS_TYPE_PTR.to_be_bytes());
    p.extend_from_slice(&DNS_CLASS_IN.to_be_bytes());
    p.extend_from_slice(&0u32.to_be_bytes());
    let target = encode_domain("inst._biomeos._tcp.local");
    p.extend_from_slice(&(target.len() as u16).to_be_bytes());
    p.extend(target);
    let pkt = parse_dns_packet(&p).unwrap().unwrap();
    assert_eq!(pkt.answers.len(), 1);
    assert_eq!(pkt.answers[0].rtype, DNS_TYPE_PTR);
}

#[test]
fn parse_rr_truncated_fixed_fields_is_err() {
    let mut p = dns_header(1, true, 0, 1, 0, 0);
    p.extend(encode_domain("a.local"));
    p.push(0x00);
    assert!(parse_dns_packet(&p).is_err());
}

#[test]
fn read_domain_name_simple_and_root() {
    let buf = [0x01, b'a', 0x00];
    let (s, end) = read_domain_name(&buf, 0).unwrap();
    assert_eq!(s, "a");
    assert_eq!(end, 3);

    let buf2 = [0x00];
    let (s2, end2) = read_domain_name(&buf2, 0).unwrap();
    assert!(s2.is_empty());
    assert_eq!(end2, 1);
}

#[test]
fn read_domain_name_compression_pointer() {
    let mut buf = vec![0u8; 8];
    buf[4] = 0x01;
    buf[5] = b'z';
    buf[6] = 0x00;
    buf[0] = 0xc0;
    buf[1] = 0x04;
    let (s, end) = read_domain_name(&buf, 0).unwrap();
    assert_eq!(s, "z");
    assert_eq!(end, 2);
}

#[test]
fn read_domain_name_invalid_utf8_label_is_err() {
    let buf = [0x01, 0xff, 0x00];
    assert!(read_domain_name(&buf, 0).is_err());
}

#[test]
fn read_domain_name_label_overflows_buffer_is_err() {
    let buf = [0x10, 0x00];
    assert!(read_domain_name(&buf, 0).is_err());
}

#[test]
fn read_domain_name_compression_out_of_bounds_is_err() {
    let buf = [0xc0, 0xff];
    assert!(read_domain_name(&buf, 0).is_err());
}

#[test]
fn read_domain_name_too_many_pointer_steps_is_err() {
    let mut buf = vec![0u8; 8];
    buf[2] = 0xc0;
    buf[3] = 0x02;
    assert!(read_domain_name(&buf, 2).is_err());
}

#[test]
fn parse_txt_rdata_pairs_and_bare_strings() {
    let raw = [0x03, b'a', b'=', b'b', 0x03, b'x', b'y', b'z'];
    let m = parse_txt_rdata(&raw);
    assert_eq!(m.get("a").map(String::as_str), Some("b"));
    assert_eq!(m.get("xyz").map(String::as_str), Some(""));
}

#[test]
fn parse_txt_rdata_invalid_utf8_chunk_skipped() {
    let raw = [0x01, 0xff];
    let m = parse_txt_rdata(&raw);
    assert!(m.is_empty());
}

#[test]
fn parse_txt_rdata_truncated_length_stops_cleanly() {
    let raw = [0x05, b'a', b'b'];
    let m = parse_txt_rdata(&raw);
    assert!(m.is_empty());
}

#[test]
fn parse_srv_rdata_too_short_is_err() {
    let buf = [0u8; 8];
    assert!(parse_srv_rdata(&buf, 0, 4).is_err());
}

#[test]
fn parse_srv_rdata_port_and_target() {
    let mut buf = vec![0u8; 32];
    let off = 10;
    buf[off + 4] = 0x12;
    buf[off + 5] = 0x34;
    let tgt = encode_domain("host.local");
    buf[off + 6..off + 6 + tgt.len()].copy_from_slice(&tgt);
    let rdlen = 6 + tgt.len();
    let (host, port) = parse_srv_rdata(&buf, off, rdlen).unwrap();
    assert_eq!(port, 0x1234);
    assert_eq!(host, "host.local");
}

#[test]
fn merge_records_ptr_populates_instance() {
    let mut pkt = dns_header(1, true, 0, 1, 0, 0);
    pkt.extend(encode_domain("_biomeos._tcp.local"));
    pkt.extend_from_slice(&DNS_TYPE_PTR.to_be_bytes());
    pkt.extend_from_slice(&DNS_CLASS_IN.to_be_bytes());
    pkt.extend_from_slice(&30u32.to_be_bytes());
    let ptr_tgt = encode_domain("mybox._biomeos._tcp.local");
    pkt.extend_from_slice(&(ptr_tgt.len() as u16).to_be_bytes());
    pkt.extend(&ptr_tgt);

    let parsed = parse_dns_packet(&pkt).unwrap().unwrap();
    let mut instances = HashMap::new();
    merge_records(&mut instances, &pkt, &parsed);
    let key = instance_key("mybox._biomeos._tcp.local");
    assert!(instances.contains_key(&key));
}

#[test]
fn merge_records_srv_inserts_when_no_ptr() {
    let mut buf = dns_header(2, true, 0, 1, 0, 0);
    let owner = encode_domain("orphan._biomeos._tcp.local");
    buf.extend(owner);
    buf.extend_from_slice(&DNS_TYPE_SRV.to_be_bytes());
    buf.extend_from_slice(&DNS_CLASS_IN.to_be_bytes());
    buf.extend_from_slice(&60u32.to_be_bytes());
    let mut rdata = vec![0u8; 6];
    rdata[4] = 0x01;
    rdata[5] = 0xbb;
    let tgt = encode_domain("srv.local.");
    rdata.extend(tgt);
    buf.extend_from_slice(&(rdata.len() as u16).to_be_bytes());
    buf.extend(&rdata);

    let parsed = parse_dns_packet(&buf).unwrap().unwrap();
    let mut instances = HashMap::new();
    merge_records(&mut instances, &buf, &parsed);
    let key = instance_key("orphan._biomeos._tcp.local");
    let rec = instances.get(&key).unwrap();
    assert_eq!(rec.srv_port, Some(443));
    assert_eq!(rec.srv_target.as_deref(), Some("srv.local"));
}

#[test]
fn merge_records_txt_merges_into_existing() {
    let mut buf = dns_header(3, true, 0, 1, 0, 0);
    let owner = encode_domain("t._biomeos._tcp.local");
    buf.extend(&owner);
    buf.extend_from_slice(&DNS_TYPE_TXT.to_be_bytes());
    buf.extend_from_slice(&DNS_CLASS_IN.to_be_bytes());
    buf.extend_from_slice(&10u32.to_be_bytes());
    let txt = [0x03, b'k', b'=', b'v', 0x04, b'a', b'=', b'b', b'c'];
    buf.extend_from_slice(&(txt.len() as u16).to_be_bytes());
    buf.extend(txt);

    let parsed = parse_dns_packet(&buf).unwrap().unwrap();
    let mut instances = HashMap::new();
    let key = instance_key("t._biomeos._tcp.local");
    instances.insert(
        key.clone(),
        InstanceRecord {
            instance_fqdn: "t._biomeos._tcp.local".to_string(),
            ..Default::default()
        },
    );
    merge_records(&mut instances, &buf, &parsed);
    let rec = instances.get(&key).unwrap();
    assert_eq!(rec.txt.get("k"), Some(&"v".to_string()));
    assert_eq!(rec.txt.get("a"), Some(&"bc".to_string()));
}

#[test]
fn build_query_includes_qtype_and_skips_long_label() {
    let q = build_query(
        "a.this-label-is-way-too-long-to-fit-in-a-single-dns-label-and-should-be-skipped._tcp.local",
        99,
        DNS_TYPE_PTR,
    );
    assert!(q.len() > 12);
    assert!(q.windows(2).any(|w| w == [0x00, 0x0c]));
}

#[test]
fn build_ptr_query_matches_build_query_service() {
    let id: u16 = 0xabcd;
    let a = build_ptr_query(SERVICE_PTR, id);
    let b = build_query(SERVICE_PTR, id, DNS_TYPE_PTR);
    assert_eq!(a, b);
}

#[test]
fn parse_capabilities_txt_empty_and_general_domain() {
    assert!(parse_capabilities_txt(None).is_empty());
    let s = "onlycap;; general:1.0; domain:op:2".to_string();
    let v = parse_capabilities_txt(Some(&s));
    assert!(v.iter().any(|c| c.category == "general"));
    assert!(v.iter().any(|c| c.category == "domain" && c.name == "op"));
}

#[test]
fn liveness_ok_branches() {
    assert!(!liveness_ok(&serde_json::json!({"error":{}})));
    assert!(liveness_ok(
        &serde_json::json!({"result":{"status":"alive"}})
    ));
    assert!(liveness_ok(
        &serde_json::json!({"result":{"status":"healthy"}})
    ));
    assert!(liveness_ok(&serde_json::json!({"result":{"status":"ok"}})));
    assert!(liveness_ok(&serde_json::json!({"result":{}})));
    assert!(!liveness_ok(&serde_json::json!({})));
}

#[test]
fn tcp_connect_addr_brackets_ipv6_literal() {
    assert_eq!(tcp_connect_addr("2001:db8::1", 443), "[2001:db8::1]:443");
    assert_eq!(tcp_connect_addr("127.0.0.1", 80), "127.0.0.1:80");
}

#[test]
fn instance_display_name_strips_suffix() {
    let fqdn = format!("MyBox.{SERVICE_PTR}");
    assert_eq!(instance_display_name(&fqdn).as_deref(), Some("MyBox"));
}

#[test]
fn normalize_target_host_trims_trailing_dot() {
    assert_eq!(normalize_target_host("host.local."), "host.local");
}

#[test]
fn parse_srv_rdata_oob_rdata_is_err() {
    let buf = [0u8; 10];
    assert!(parse_srv_rdata(&buf, 8, 10).is_err());
}
