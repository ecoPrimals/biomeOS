// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::parse_ip_from_domifaddr_output;

#[test]
fn test_parse_ip_from_domifaddr_output() {
    let output = " Name       MAC address          Protocol     Address\n\nvnet0      xx:xx:xx:xx:xx:xx    ipv4         192.168.122.34/24\n";
    assert_eq!(
        parse_ip_from_domifaddr_output(output),
        Some("192.168.122.34".to_string())
    );
}

#[test]
fn test_parse_ip_from_domifaddr_output_no_match() {
    assert_eq!(parse_ip_from_domifaddr_output(""), None);
    assert_eq!(parse_ip_from_domifaddr_output("ipv6  fe80::1/64"), None);
}

#[test]
fn test_parse_ip_from_domifaddr_multiple_lines() {
    let output = " Name       MAC address          Protocol     Address\n\nvnet0      xx:xx    ipv4         192.168.122.100/24\nvnet1      yy:yy    ipv4         192.168.122.101/24\n";
    let ip = parse_ip_from_domifaddr_output(output);
    assert_eq!(ip, Some("192.168.122.100".to_string()));
}

#[test]
fn test_parse_ip_from_domifaddr_ipv6_only() {
    assert_eq!(parse_ip_from_domifaddr_output("ipv6  fe80::1/64"), None);
}

#[test]
fn test_parse_ip_from_domifaddr_192_168_prefix_only() {
    let output = " Name       MAC address          Protocol     Address\n\nvnet0      xx:xx    ipv4         192.168.0.50/24\n";
    assert_eq!(
        parse_ip_from_domifaddr_output(output),
        Some("192.168.0.50".to_string())
    );
}

#[test]
fn test_parse_ip_from_domifaddr_multiple_ipv4_takes_first() {
    let output = " vnet0  xx  ipv4  192.168.122.10/24\n vnet1  yy  ipv4  192.168.122.20/24\n";
    let ip = parse_ip_from_domifaddr_output(output);
    assert_eq!(ip, Some("192.168.122.10".to_string()));
}

#[test]
fn test_parse_ip_from_domifaddr_whitespace_variations() {
    let output = "  ipv4    192.168.100.1/24  ";
    let ip = parse_ip_from_domifaddr_output(output);
    assert_eq!(ip, Some("192.168.100.1".to_string()));
}

#[test]
fn test_parse_ip_from_domifaddr_192_168_in_middle() {
    let output = " Name       MAC address          Protocol     Address\n\nvnet0      xx:xx    ipv4         10.0.0.1/24\nvnet1      yy:yy    ipv4         192.168.122.50/24\n";
    let ip = parse_ip_from_domifaddr_output(output);
    assert_eq!(ip, Some("192.168.122.50".to_string()));
}

#[test]
fn test_parse_ip_from_domifaddr_empty_lines() {
    let output = "\n\n  ipv4    192.168.0.2/24  \n\n";
    let ip = parse_ip_from_domifaddr_output(output);
    assert_eq!(ip, Some("192.168.0.2".to_string()));
}

#[test]
fn test_parse_ip_from_domifaddr_contains_192_168_in_line() {
    let output = " ipv4  192.168.0.1/24";
    let ip = parse_ip_from_domifaddr_output(output);
    assert_eq!(ip, Some("192.168.0.1".to_string()));
}

#[test]
fn test_parse_ip_from_domifaddr_192_168_without_ipv4_keyword() {
    // Branch: line matches via `192.168` substring without `ipv4` label
    let output = " Name   MAC   Address\n  vnet0  xx  192.168.122.200/24\n";
    assert_eq!(
        parse_ip_from_domifaddr_output(output),
        Some("192.168.122.200".to_string())
    );
}

#[test]
fn test_parse_ip_ipv4_label_but_non_rfc1918_returns_none() {
    assert_eq!(
        parse_ip_from_domifaddr_output("ipv4         10.0.0.1/24"),
        None
    );
}

#[test]
fn test_parse_ip_line_ipv4_without_192_168_until_later_line() {
    let output = "vnet0  xx  ipv4  10.0.0.1/24\nvnet1  yy  ipv4  192.168.50.2/24\n";
    assert_eq!(
        parse_ip_from_domifaddr_output(output),
        Some("192.168.50.2".to_string())
    );
}

#[test]
fn test_parse_ip_last_token_not_ip() {
    assert_eq!(parse_ip_from_domifaddr_output("ipv4   garbage"), None);
}

#[test]
fn test_parse_ip_non_numeric_octets_still_matches_prefix_heuristic() {
    // Parser does not validate dotted-decimal; it only checks the `192.168` prefix.
    assert_eq!(
        parse_ip_from_domifaddr_output("foo 192.168.abc.1/24"),
        Some("192.168.abc.1".to_string())
    );
}

#[test]
fn test_parse_ip_from_domifaddr_only_non_matching_lines() {
    let output = "header\n  ipv6  fe80::1/64\n  other  text\n";
    assert_eq!(parse_ip_from_domifaddr_output(output), None);
}

#[test]
fn test_parse_ip_strips_cidr_from_token() {
    let output = "  ipv4    192.168.255.254/16  ";
    assert_eq!(
        parse_ip_from_domifaddr_output(output),
        Some("192.168.255.254".to_string())
    );
}

#[test]
fn test_parse_ip_from_domifaddr_ipv4_keyword_non_matching_ip_token() {
    assert_eq!(
        super::parse_ip_from_domifaddr_output("ipv4         garbage/24"),
        None
    );
}

#[test]
fn test_parse_ip_line_ipv4_only_no_192_match() {
    assert_eq!(
        super::parse_ip_from_domifaddr_output("proto  ipv4  10.11.12.13/24"),
        None
    );
}

#[test]
fn test_parse_ip_domifaddr_ipv4_keyword_non_192_line_then_valid() {
    let t = "vnet0  ipv4  10.0.0.1/24\nvnet1  ipv4  192.168.0.2/24\n";
    assert_eq!(
        super::parse_ip_from_domifaddr_output(t),
        Some("192.168.0.2".to_string())
    );
}

#[test]
fn test_parse_ip_slash_only_after_dot() {
    assert_eq!(
        super::parse_ip_from_domifaddr_output("ipv4  192.168.0.1/"),
        Some("192.168.0.1".to_string())
    );
}

#[test]
fn test_parse_ip_tabs_instead_of_spaces() {
    let t = "vnet0\tipv4\t192.168.99.1/24\n";
    assert_eq!(
        super::parse_ip_from_domifaddr_output(t),
        Some("192.168.99.1".to_string())
    );
}

#[test]
fn test_parse_ip_rejects_line_with_192_168_substring_in_wrong_token() {
    // Last token must parse as starting with 192.168 after split on '/'
    assert_eq!(
        super::parse_ip_from_domifaddr_output("note: 192.168 is reserved  garbage"),
        None
    );
}
