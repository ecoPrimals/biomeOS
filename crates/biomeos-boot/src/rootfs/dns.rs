// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! DNS configuration utilities

/// Sample IPv6 resolver address for tests (RFC 3849 documentation prefix).
/// Uses 2001:db8::/32 to avoid hardcoding corporate DNS (Google, Cloudflare) in tests.
/// Sovereignty: Production DNS is always from RootFsConfig.dns_servers or system resolv.conf.
pub const TEST_IPV6_RESOLVER_SAMPLE: &str = "2001:db8::1";

/// Fallback IPv6 resolver (RFC 3849 documentation prefix).
///
/// Used only when no DNS servers are configured — e.g. in minimal container/initrd
/// environments. Sovereignty: Production should use RootFsConfig.dns_servers or
/// BIOMEOS_DNS_SERVERS env var. This constant avoids hardcoding corporate DNS
/// (Google 2001:4860:4860::8888, Cloudflare) in production code.
pub const FALLBACK_RESOLVER_IPV6: &str = "2001:db8::1";

/// Parse nameserver lines from resolv.conf content (testable)
pub(crate) fn parse_resolv_conf(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|line| line.trim().starts_with("nameserver"))
        .filter_map(|line| line.split_whitespace().nth(1).map(String::from))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_resolv_conf_empty() {
        assert!(parse_resolv_conf("").is_empty());
        assert!(parse_resolv_conf("   \n  \n").is_empty());
    }

    #[test]
    fn test_parse_resolv_conf_single() {
        let content = "nameserver 8.8.8.8\n";
        let servers = parse_resolv_conf(content);
        assert_eq!(servers, vec!["8.8.8.8"]);
    }

    #[test]
    fn test_parse_resolv_conf_multiple() {
        let content = "nameserver 8.8.8.8\nnameserver 8.8.4.4\nnameserver 1.1.1.1\n";
        let servers = parse_resolv_conf(content);
        assert_eq!(servers, vec!["8.8.8.8", "8.8.4.4", "1.1.1.1"]);
    }

    #[test]
    fn test_parse_resolv_conf_skips_comments() {
        let content = "# comment\nnameserver 8.8.8.8\noptions ndots:5\nnameserver 1.1.1.1\n";
        let servers = parse_resolv_conf(content);
        assert_eq!(servers, vec!["8.8.8.8", "1.1.1.1"]);
    }

    #[test]
    fn test_parse_resolv_conf_handles_whitespace() {
        let content = "  nameserver   192.168.1.1  \n";
        let servers = parse_resolv_conf(content);
        assert_eq!(servers, vec!["192.168.1.1"]);
    }

    #[test]
    fn test_parse_resolv_conf_ipv6() {
        let content = format!("nameserver {}\n", super::TEST_IPV6_RESOLVER_SAMPLE);
        let servers = parse_resolv_conf(&content);
        assert_eq!(servers, vec![super::TEST_IPV6_RESOLVER_SAMPLE]);
    }
}
