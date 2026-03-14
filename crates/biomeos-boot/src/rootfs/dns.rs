// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! DNS configuration utilities

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
        let content = "nameserver 2001:4860:4860::8888\n";
        let servers = parse_resolv_conf(content);
        assert_eq!(servers, vec!["2001:4860:4860::8888"]);
    }
}
