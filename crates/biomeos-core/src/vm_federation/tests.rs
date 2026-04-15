// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test")]

use super::*;

#[test]
fn test_validation_config_default() {
    let config = ValidationConfig::default();
    assert_eq!(config.cloud_init_timeout.as_secs(), 600);
    assert_eq!(config.ssh_timeout.as_secs(), 300);
    assert_eq!(config.ssh_retry_interval.as_secs(), 30);
    assert_eq!(config.ssh_max_retries, 20);
}

#[test]
fn test_validation_config_custom() {
    let config = ValidationConfig {
        cloud_init_timeout: Duration::from_secs(120),
        ssh_timeout: Duration::from_secs(60),
        ssh_retry_interval: Duration::from_secs(10),
        ssh_max_retries: 5,
    };
    assert_eq!(config.cloud_init_timeout.as_secs(), 120);
    assert_eq!(config.ssh_max_retries, 5);
}

#[test]
fn test_manager_creation() {
    let manager = VmFederationManager::new();
    // Manager creation requires benchscale directory to exist
    // This is a valid requirement, so we just verify the Result type works
    match manager {
        Ok(_) => {
            // benchscale exists - great!
        }
        Err(e) => {
            // benchscale doesn't exist - expected in CI/test environments
            assert!(
                e.to_string().contains("benchscale not found"),
                "Error should be about missing benchscale, got: {e}"
            );
        }
    }
}

#[test]
fn test_with_validation_config_requires_benchscale() {
    let config = ValidationConfig::default();
    let result = VmFederationManager::with_validation_config(config);
    match result {
        Ok(_) => {}
        Err(e) => {
            assert!(
                e.to_string().contains("benchscale not found") || e.to_string().contains("parent"),
                "Expected benchscale or path error, got: {e}"
            );
        }
    }
}

#[test]
fn test_parse_ip_from_domifaddr_output() {
    let output = " Name       MAC address          Protocol     Address\n\nvnet0      xx:xx:xx:xx:xx:xx    ipv4         192.168.122.34/24\n";
    assert_eq!(
        super::parse_ip_from_domifaddr_output(output),
        Some("192.168.122.34".to_string())
    );
}

#[test]
fn test_parse_ip_from_domifaddr_output_no_match() {
    assert_eq!(super::parse_ip_from_domifaddr_output(""), None);
    assert_eq!(
        super::parse_ip_from_domifaddr_output("ipv6  fe80::1/64"),
        None
    );
}

#[test]
fn test_parse_vm_names_from_list() {
    let list = " Id    Name                           State\n----------------------------------------------------\n 1     my-fed-node1                   running\n 2     my-fed-node2                   running\n";
    let names = super::parse_vm_names_from_list(list, "my-fed");
    assert_eq!(names, vec!["my-fed-node1", "my-fed-node2"]);
}

#[test]
fn test_parse_vm_names_from_list_empty() {
    let names = super::parse_vm_names_from_list("", "nonexistent");
    assert!(names.is_empty());
}

#[test]
fn test_parse_ip_from_domifaddr_multiple_lines() {
    let output = " Name       MAC address          Protocol     Address\n\nvnet0      xx:xx    ipv4         192.168.122.100/24\nvnet1      yy:yy    ipv4         192.168.122.101/24\n";
    let ip = super::parse_ip_from_domifaddr_output(output);
    assert_eq!(ip, Some("192.168.122.100".to_string()));
}

#[test]
fn test_parse_ip_from_domifaddr_ipv6_only() {
    assert_eq!(
        super::parse_ip_from_domifaddr_output("ipv6  fe80::1/64"),
        None
    );
}

#[test]
fn test_parse_vm_names_from_list_partial_match() {
    let list =
        " 1     fed-node1    running\n 2     fed-node2    running\n 3     other-node   running\n";
    let names = super::parse_vm_names_from_list(list, "fed");
    assert_eq!(names, vec!["fed-node1", "fed-node2"]);
}

#[test]
fn test_parse_vm_names_from_list_single_vm() {
    let list = " 1     my-fed-node1    running\n";
    let names = super::parse_vm_names_from_list(list, "my-fed");
    assert_eq!(names, vec!["my-fed-node1"]);
}

#[test]
fn test_parse_ip_from_domifaddr_192_168_prefix_only() {
    let output = " Name       MAC address          Protocol     Address\n\nvnet0      xx:xx    ipv4         192.168.0.50/24\n";
    assert_eq!(
        super::parse_ip_from_domifaddr_output(output),
        Some("192.168.0.50".to_string())
    );
}

#[test]
fn test_parse_ip_from_domifaddr_multiple_ipv4_takes_first() {
    let output = " vnet0  xx  ipv4  192.168.122.10/24\n vnet1  yy  ipv4  192.168.122.20/24\n";
    let ip = super::parse_ip_from_domifaddr_output(output);
    assert_eq!(ip, Some("192.168.122.10".to_string()));
}

#[test]
fn test_parse_vm_names_from_list_no_match() {
    let list = " 1     other-vm-1    running\n 2     other-vm-2    running\n";
    let names = super::parse_vm_names_from_list(list, "my-fed");
    assert!(names.is_empty());
}

#[test]
fn test_parse_vm_names_from_list_single_column() {
    let list = " 1     fed-node1\n";
    let names = super::parse_vm_names_from_list(list, "fed");
    assert_eq!(names, vec!["fed-node1"]);
}

#[test]
fn test_parse_ip_from_domifaddr_whitespace_variations() {
    let output = "  ipv4    192.168.100.1/24  ";
    let ip = super::parse_ip_from_domifaddr_output(output);
    assert_eq!(ip, Some("192.168.100.1".to_string()));
}

#[test]
fn test_validation_config_debug() {
    let config = ValidationConfig::default();
    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("ValidationConfig"));
}

#[test]
fn test_parse_ip_from_domifaddr_192_168_in_middle() {
    let output = " Name       MAC address          Protocol     Address\n\nvnet0      xx:xx    ipv4         10.0.0.1/24\nvnet1      yy:yy    ipv4         192.168.122.50/24\n";
    let ip = super::parse_ip_from_domifaddr_output(output);
    assert_eq!(ip, Some("192.168.122.50".to_string()));
}

#[test]
fn test_parse_vm_names_from_list_extra_columns() {
    let list = " Id    Name                State       CPU    Memory\n----------------------------------------------------\n 1     fed-node1           running     1      1024\n";
    let names = super::parse_vm_names_from_list(list, "fed");
    assert_eq!(names, vec!["fed-node1"]);
}

#[test]
fn test_validation_config_clone() {
    let config = ValidationConfig::default();
    let cloned = config.clone();
    assert_eq!(cloned.ssh_max_retries, config.ssh_max_retries);
    assert_eq!(cloned.cloud_init_timeout, config.cloud_init_timeout);
}

#[test]
fn test_parse_ip_from_domifaddr_empty_lines() {
    let output = "\n\n  ipv4    192.168.0.2/24  \n\n";
    let ip = super::parse_ip_from_domifaddr_output(output);
    assert_eq!(ip, Some("192.168.0.2".to_string()));
}

#[test]
fn test_parse_ip_from_domifaddr_contains_192_168_in_line() {
    let output = " ipv4  192.168.0.1/24";
    let ip = super::parse_ip_from_domifaddr_output(output);
    assert_eq!(ip, Some("192.168.0.1".to_string()));
}

#[test]
fn test_parse_ip_from_domifaddr_192_168_without_ipv4_keyword() {
    // Branch: line matches via `192.168` substring without `ipv4` label
    let output = " Name   MAC   Address\n  vnet0  xx  192.168.122.200/24\n";
    assert_eq!(
        super::parse_ip_from_domifaddr_output(output),
        Some("192.168.122.200".to_string())
    );
}

#[test]
fn test_parse_vm_names_from_list_malformed_line() {
    let list = " 1     fed-node1    running\n single_word\n";
    let names = super::parse_vm_names_from_list(list, "fed");
    assert_eq!(names, vec!["fed-node1"]);
}

#[test]
fn test_validation_config_builder_pattern() {
    let config = ValidationConfig {
        cloud_init_timeout: Duration::from_secs(900),
        ssh_timeout: Duration::from_secs(600),
        ssh_retry_interval: Duration::from_secs(15),
        ssh_max_retries: 40,
    };
    assert_eq!(config.cloud_init_timeout.as_secs(), 900);
    assert_eq!(config.ssh_max_retries, 40);
}

#[test]
fn test_parse_ip_ipv4_label_but_non_rfc1918_returns_none() {
    assert_eq!(
        super::parse_ip_from_domifaddr_output("ipv4         10.0.0.1/24"),
        None
    );
}

#[test]
fn test_parse_ip_line_ipv4_without_192_168_until_later_line() {
    let output = "vnet0  xx  ipv4  10.0.0.1/24\nvnet1  yy  ipv4  192.168.50.2/24\n";
    assert_eq!(
        super::parse_ip_from_domifaddr_output(output),
        Some("192.168.50.2".to_string())
    );
}

#[test]
fn test_parse_vm_names_single_token_line_not_pushed() {
    let list = "my-fed\n";
    let names = super::parse_vm_names_from_list(list, "my-fed");
    assert!(names.is_empty());
}

#[test]
fn test_parse_vm_names_header_line_extracts_second_column() {
    // Only lines containing the federation substring participate; `real-vm-1` line has no `my-fed`.
    let list = "my-fed header line\n 1     real-vm-1    running\n";
    let names = super::parse_vm_names_from_list(list, "my-fed");
    assert_eq!(names, vec!["header"]);
}

#[test]
fn test_parse_ip_last_token_not_ip() {
    assert_eq!(
        super::parse_ip_from_domifaddr_output("ipv4   garbage"),
        None
    );
}

#[test]
fn test_parse_ip_non_numeric_octets_still_matches_prefix_heuristic() {
    // Parser does not validate dotted-decimal; it only checks the `192.168` prefix.
    assert_eq!(
        super::parse_ip_from_domifaddr_output("foo 192.168.abc.1/24"),
        Some("192.168.abc.1".to_string())
    );
}

#[test]
fn test_parse_vm_names_tabs_and_multiple_spaces() {
    let list = "1\tmy-fed-node1\trunning\n";
    let names = super::parse_vm_names_from_list(list, "my-fed");
    assert_eq!(names, vec!["my-fed-node1"]);
}

#[test]
fn test_validation_config_extreme_retries_zero() {
    let config = ValidationConfig {
        cloud_init_timeout: Duration::from_secs(1),
        ssh_timeout: Duration::from_secs(1),
        ssh_retry_interval: Duration::from_secs(1),
        ssh_max_retries: 0,
    };
    assert_eq!(config.ssh_max_retries, 0);
}

#[test]
fn test_parse_vm_names_duplicate_lines() {
    let list = " 1     dup-fed-a    running\n 2     dup-fed-b    running\n";
    let names = super::parse_vm_names_from_list(list, "dup-fed");
    assert_eq!(names, vec!["dup-fed-a", "dup-fed-b"]);
}

#[test]
fn test_parse_ip_from_domifaddr_only_non_matching_lines() {
    let output = "header\n  ipv6  fe80::1/64\n  other  text\n";
    assert_eq!(super::parse_ip_from_domifaddr_output(output), None);
}

#[test]
fn test_parse_vm_names_numeric_id_with_federation_in_name() {
    let list = " 10    my-fed-10    running\n";
    let names = super::parse_vm_names_from_list(list, "my-fed");
    assert_eq!(names, vec!["my-fed-10"]);
}

#[test]
fn test_parse_ip_strips_cidr_from_token() {
    let output = "  ipv4    192.168.255.254/16  ";
    assert_eq!(
        super::parse_ip_from_domifaddr_output(output),
        Some("192.168.255.254".to_string())
    );
}

#[test]
fn test_parse_vm_names_line_contains_fed_but_less_than_two_columns() {
    let list = "my-fed\n";
    let names = parse_vm_names_from_list(list, "my-fed");
    assert!(names.is_empty());
}

#[test]
fn test_parse_ip_from_domifaddr_ipv4_keyword_non_matching_ip_token() {
    assert_eq!(
        parse_ip_from_domifaddr_output("ipv4         garbage/24"),
        None
    );
}

#[test]
fn test_parse_vm_names_preserves_order() {
    let list = " 2     fed-b    running\n 1     fed-a    running\n";
    let names = parse_vm_names_from_list(list, "fed");
    assert_eq!(names, vec!["fed-b", "fed-a"]);
}

#[test]
fn test_parse_vm_names_match_in_first_column_extracts_second_column() {
    // Any line containing the federation substring participates; the VM name is always column 2.
    let list = "my-fed-prefix    actual-vm-name    running\n";
    let names = parse_vm_names_from_list(list, "my-fed");
    assert_eq!(names, vec!["actual-vm-name"]);
}

#[test]
fn test_parse_ip_line_ipv4_only_no_192_match() {
    assert_eq!(
        parse_ip_from_domifaddr_output("proto  ipv4  10.11.12.13/24"),
        None
    );
}

#[test]
fn test_parse_vm_names_long_federation_substring() {
    let list = " 1     prefix-my-fed-suffix    running\n";
    let names = parse_vm_names_from_list(list, "my-fed");
    assert_eq!(names, vec!["prefix-my-fed-suffix"]);
}

#[test]
fn test_parse_ip_domifaddr_ipv4_keyword_non_192_line_then_valid() {
    let t = "vnet0  ipv4  10.0.0.1/24\nvnet1  ipv4  192.168.0.2/24\n";
    assert_eq!(
        parse_ip_from_domifaddr_output(t),
        Some("192.168.0.2".to_string())
    );
}

#[test]
fn test_validation_config_extreme_durations() {
    let c = ValidationConfig {
        cloud_init_timeout: Duration::from_secs(u64::MAX / 4),
        ssh_timeout: Duration::from_secs(1),
        ssh_retry_interval: Duration::from_millis(1),
        ssh_max_retries: u32::MAX,
    };
    assert!(c.cloud_init_timeout > Duration::from_secs(1_000_000));
}

#[test]
fn test_parse_vm_names_windows_style_line_endings() {
    let list = " 1     my-fed-w1    running\r\n 2     my-fed-w2    running\r\n";
    let names = parse_vm_names_from_list(list, "my-fed");
    assert_eq!(names, vec!["my-fed-w1", "my-fed-w2"]);
}

#[test]
fn test_parse_ip_slash_only_after_dot() {
    assert_eq!(
        parse_ip_from_domifaddr_output("ipv4  192.168.0.1/"),
        Some("192.168.0.1".to_string())
    );
}

#[test]
fn test_parse_vm_names_uuid_suffix_in_name() {
    let list = " 1     fed-node-550e8400-e29b-41d4-a716-446655440000    running\n";
    let names = parse_vm_names_from_list(list, "fed-node");
    assert_eq!(names, vec!["fed-node-550e8400-e29b-41d4-a716-446655440000"]);
}

#[test]
fn test_parse_ip_tabs_instead_of_spaces() {
    let t = "vnet0\tipv4\t192.168.99.1/24\n";
    assert_eq!(
        parse_ip_from_domifaddr_output(t),
        Some("192.168.99.1".to_string())
    );
}

#[test]
fn test_parse_vm_names_three_columns_id_name_state() {
    let list = "42    vm-fed-core    shut off\n";
    let names = parse_vm_names_from_list(list, "fed");
    assert_eq!(names, vec!["vm-fed-core"]);
}

#[test]
fn test_parse_ip_rejects_line_with_192_168_substring_in_wrong_token() {
    // Last token must parse as starting with 192.168 after split on '/'
    assert_eq!(
        parse_ip_from_domifaddr_output("note: 192.168 is reserved  garbage"),
        None
    );
}

#[test]
fn test_benchscale_create_argv() {
    let argv = benchscale_create_argv("my-fed", "/tmp/topology.yaml");
    assert_eq!(argv[0], "run");
    assert_eq!(argv[3], "create");
    assert_eq!(argv[4], "my-fed");
    assert_eq!(argv[6], "/tmp/topology.yaml");
    assert_eq!(argv.len(), 9);
}

#[test]
fn test_benchscale_subcommand_argv() {
    for cmd in ["start", "stop", "destroy", "test", "status"] {
        let argv = benchscale_subcommand_argv(cmd, "fed1");
        assert_eq!(argv[3], cmd);
        assert_eq!(argv[4], "fed1");
        assert_eq!(argv.len(), 5);
    }
}

#[test]
fn test_topology_path_for_cli_valid() {
    let path = Path::new("/tmp/topology.yaml");
    assert_eq!(topology_path_for_cli(path).unwrap(), "/tmp/topology.yaml");
}

#[test]
fn test_validate_ssh_probe_output_success() {
    use std::os::unix::process::ExitStatusExt;
    let output = std::process::Output {
        status: std::process::ExitStatus::from_raw(0),
        stdout: b"hostname\n".to_vec(),
        stderr: vec![],
    };
    assert!(validate_ssh_probe_output("10.0.0.1", &output).is_ok());
}

#[test]
fn test_validate_ssh_probe_output_failure() {
    use std::os::unix::process::ExitStatusExt;
    // exit code 1 in wait status encoding (1 << 8 = 256)
    let output = std::process::Output {
        status: std::process::ExitStatus::from_raw(256),
        stdout: vec![],
        stderr: b"Connection refused\n".to_vec(),
    };
    let result = validate_ssh_probe_output("10.0.0.1", &output);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("10.0.0.1"));
}

#[test]
fn test_collect_ips_for_vm_names_with_mock() {
    let ips = collect_ips_for_vm_names(vec!["vm1".to_string(), "vm2".to_string()], |name| {
        let ip = if name == "vm1" {
            "vnet0  xx  ipv4  192.168.0.10/24\n"
        } else {
            "vnet0  xx  ipv4  192.168.0.20/24\n"
        };
        Ok(std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: ip.as_bytes().to_vec(),
            stderr: vec![],
        })
    });
    assert_eq!(ips, vec!["192.168.0.10", "192.168.0.20"]);
}

#[test]
fn test_collect_ips_for_vm_names_io_error_skips() {
    let ips = collect_ips_for_vm_names(vec!["vm1".to_string()], |_| {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "virsh not found",
        ))
    });
    assert!(ips.is_empty());
}

#[test]
fn test_collect_ips_for_vm_names_no_ip_skips() {
    let ips = collect_ips_for_vm_names(vec!["vm1".to_string()], |_| {
        Ok(std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: b"no ip here\n".to_vec(),
            stderr: vec![],
        })
    });
    assert!(ips.is_empty());
}

#[tokio::test]
async fn test_wait_for_vm_ssh_ready_immediate_success() {
    use std::os::unix::process::ExitStatusExt;
    let config = ValidationConfig {
        cloud_init_timeout: Duration::from_secs(10),
        ssh_timeout: Duration::from_secs(5),
        ssh_retry_interval: Duration::from_millis(10),
        ssh_max_retries: 3,
    };
    let start = Instant::now();
    let result = wait_for_vm_ssh_ready("10.0.0.1", &config, start, || {
        Ok(std::process::Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: b"SSH ready\n".to_vec(),
            stderr: vec![],
        })
    })
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_wait_for_vm_ssh_ready_max_retries_exceeded() {
    let config = ValidationConfig {
        cloud_init_timeout: Duration::from_secs(60),
        ssh_timeout: Duration::from_secs(5),
        ssh_retry_interval: Duration::from_millis(1),
        ssh_max_retries: 2,
    };
    let start = Instant::now();
    let result = wait_for_vm_ssh_ready("10.0.0.1", &config, start, || {
        Err(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "refused",
        ))
    })
    .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("10.0.0.1"));
}

#[tokio::test]
async fn test_wait_for_vm_ssh_ready_cloud_init_timeout_zero_bails_immediately() {
    let config = ValidationConfig {
        cloud_init_timeout: Duration::ZERO,
        ssh_timeout: Duration::from_secs(5),
        ssh_retry_interval: Duration::from_millis(1),
        ssh_max_retries: 10,
    };
    let start = Instant::now();
    let result = wait_for_vm_ssh_ready("192.0.2.1", &config, start, || {
        unreachable!("SSH probe should not run when cloud_init_timeout is zero")
    })
    .await;
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("192.0.2.1"), "msg={msg}");
    assert!(msg.contains("Timeout") || msg.contains("timeout"));
}

#[tokio::test]
async fn test_wait_for_vm_ssh_ready_succeeds_on_second_ssh_attempt() {
    use std::os::unix::process::ExitStatusExt;
    let config = ValidationConfig {
        cloud_init_timeout: Duration::from_secs(30),
        ssh_timeout: Duration::from_secs(5),
        ssh_retry_interval: Duration::from_millis(1),
        ssh_max_retries: 5,
    };
    let start = Instant::now();
    let attempt = std::cell::Cell::new(0u32);
    let result = wait_for_vm_ssh_ready("10.0.0.2", &config, start, || {
        let n = attempt.get() + 1;
        attempt.set(n);
        if n == 1 {
            Ok(std::process::Output {
                status: std::process::ExitStatus::from_raw(256),
                stdout: vec![],
                stderr: vec![],
            })
        } else {
            Ok(std::process::Output {
                status: std::process::ExitStatus::from_raw(0),
                stdout: b"ok".to_vec(),
                stderr: vec![],
            })
        }
    })
    .await;
    assert!(result.is_ok());
    assert_eq!(attempt.get(), 2);
}

#[cfg(unix)]
#[test]
fn test_topology_path_for_cli_rejects_non_utf8_path() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let path = PathBuf::from(OsString::from_vec(vec![0xFF, 0xFE, 0xFD]));
    let err = topology_path_for_cli(&path).unwrap_err();
    assert!(err.to_string().contains("UTF-8") || err.to_string().contains("utf-8"));
}

#[test]
fn test_collect_ips_for_vm_names_first_fails_second_ok() {
    let mut calls = 0u32;
    let ips = collect_ips_for_vm_names(vec!["bad".to_string(), "good".to_string()], |_| {
        calls += 1;
        if calls == 1 {
            Err(std::io::Error::other("domifaddr failed"))
        } else {
            Ok(std::process::Output {
                status: std::process::ExitStatus::default(),
                stdout: b"vnet0  ipv4  192.168.50.10/24\n".to_vec(),
                stderr: vec![],
            })
        }
    });
    assert_eq!(ips, vec!["192.168.50.10"]);
    assert_eq!(calls, 2);
}

#[tokio::test]
#[ignore = "Requires benchscale VM harness and libvirt"]
async fn test_full_lifecycle() -> Result<()> {
    // Only run if benchscale is available AND libvirt testing is enabled
    if std::env::var("BENCHSCALE_TEST_LIBVIRT").is_err() {
        // Skip test if libvirt testing not enabled
        return Ok(());
    }

    let Ok(manager) = VmFederationManager::new() else {
        return Ok(());
    };

    let name = "test-federation";

    // This would actually create VMs if libvirt is available
    manager.create(name).await?;
    manager.start(name)?;
    manager.test(name)?;
    manager.stop(name)?;
    manager.destroy(name)?;
    Ok(())
}
