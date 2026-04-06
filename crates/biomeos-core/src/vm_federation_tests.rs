// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unit tests for [`crate::vm_federation`] helpers extracted for testability.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use std::time::{Duration, Instant};

use crate::vm_federation::{
    ValidationConfig, benchscale_create_argv, benchscale_subcommand_argv, collect_ips_for_vm_names,
    topology_path_for_cli, validate_ssh_probe_output, wait_for_vm_ssh_ready,
};

#[cfg(unix)]
fn unix_output_ok(stdout: &[u8]) -> std::process::Output {
    use std::os::unix::process::ExitStatusExt;
    std::process::Output {
        status: std::process::ExitStatus::from_raw(0),
        stdout: stdout.to_vec(),
        stderr: vec![],
    }
}

#[test]
fn collect_ips_skips_io_errors_and_empty_parses() {
    let ips = collect_ips_for_vm_names(vec!["vm-a".to_string(), "vm-b".to_string()], |name| {
        if name == "vm-a" {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "virsh missing",
            ))
        } else {
            Ok(unix_output_ok(b"no 192.168 address here\n"))
        }
    });
    assert!(ips.is_empty());
}

#[test]
fn collect_ips_merges_multiple_vms() {
    let ips = collect_ips_for_vm_names(vec!["vm-a".to_string(), "vm-b".to_string()], |name| {
        let out = if name == "vm-a" {
            b"ipv4         192.168.122.10/24\n"
        } else {
            b"ipv4         192.168.122.11/24\n"
        };
        Ok(unix_output_ok(out))
    });
    assert_eq!(
        ips,
        vec!["192.168.122.10".to_string(), "192.168.122.11".to_string()]
    );
}

#[test]
fn validate_ssh_probe_ok_and_err() {
    let ok = std::process::Command::new("true").output().expect("true");
    validate_ssh_probe_output("10.0.0.1", &ok).expect("success status");

    let bad = std::process::Command::new("false").output().expect("false");
    let err = validate_ssh_probe_output("10.0.0.2", &bad).unwrap_err();
    assert!(err.to_string().contains("SSH validation failed"), "{err}");
}

#[cfg(unix)]
#[tokio::test]
async fn wait_for_vm_ssh_ready_succeeds_when_ssh_probe_succeeds() {
    let cfg = ValidationConfig {
        cloud_init_timeout: Duration::from_secs(60),
        ssh_timeout: Duration::from_secs(60),
        ssh_retry_interval: Duration::from_millis(1),
        ssh_max_retries: 5,
    };
    wait_for_vm_ssh_ready("192.0.2.1", &cfg, Instant::now(), || {
        std::process::Command::new("true").output()
    })
    .await
    .expect("immediate success");
}

#[cfg(unix)]
#[tokio::test]
async fn wait_for_vm_ssh_ready_fails_after_max_retries() {
    let cfg = ValidationConfig {
        cloud_init_timeout: Duration::from_secs(600),
        ssh_timeout: Duration::from_secs(60),
        ssh_retry_interval: Duration::from_nanos(1),
        ssh_max_retries: 4,
    };
    let err = wait_for_vm_ssh_ready("192.0.2.2", &cfg, Instant::now(), || {
        std::process::Command::new("false").output()
    })
    .await
    .unwrap_err();
    assert!(
        err.to_string().contains("Failed to SSH"),
        "unexpected: {err}"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn wait_for_vm_ssh_ready_times_out_before_max_retries() {
    let cfg = ValidationConfig {
        cloud_init_timeout: Duration::from_millis(80),
        ssh_timeout: Duration::from_secs(60),
        ssh_retry_interval: Duration::from_nanos(1),
        ssh_max_retries: 100_000,
    };
    let err = wait_for_vm_ssh_ready("192.0.2.3", &cfg, Instant::now(), || {
        std::process::Command::new("false").output()
    })
    .await
    .unwrap_err();
    assert!(
        err.to_string().contains("Timeout waiting for VM"),
        "unexpected: {err}"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn wait_for_vm_ssh_ready_io_error_then_success() {
    let cfg = ValidationConfig {
        cloud_init_timeout: Duration::from_secs(30),
        ssh_timeout: Duration::from_secs(30),
        ssh_retry_interval: Duration::from_nanos(1),
        ssh_max_retries: 10,
    };
    let mut n = 0u32;
    wait_for_vm_ssh_ready("192.0.2.4", &cfg, Instant::now(), || {
        n += 1;
        if n < 3 {
            Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "ssh unavailable",
            ))
        } else {
            std::process::Command::new("true").output()
        }
    })
    .await
    .expect("recovers after transient io errors");
}

#[test]
fn collect_ips_filters_non_192_168_domifaddr() {
    let ips = collect_ips_for_vm_names(vec!["vm-x".to_string()], |_| {
        Ok(unix_output_ok(b"ipv4  10.0.0.1/24\n"))
    });
    assert!(ips.is_empty());
}

#[test]
fn benchscale_create_argv_matches_expected_order() {
    let argv = benchscale_create_argv("my-fed", "/path/topo.yaml");
    assert_eq!(
        argv.as_slice(),
        &[
            "run",
            "--release",
            "--",
            "create",
            "my-fed",
            "--topology",
            "/path/topo.yaml",
            "--backend",
            "libvirt",
        ][..]
    );
}

#[test]
fn benchscale_subcommand_argv_matches_expected_order() {
    let argv = benchscale_subcommand_argv("stop", "fed");
    assert_eq!(argv.as_slice(), ["run", "--release", "--", "stop", "fed"]);
}

#[test]
fn topology_path_for_cli_accepts_utf8() {
    let p = std::path::Path::new("/tmp/topology.yaml");
    assert_eq!(
        topology_path_for_cli(p).expect("utf8"),
        "/tmp/topology.yaml"
    );
}

#[cfg(unix)]
#[test]
fn topology_path_for_cli_rejects_non_utf8() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let bytes = b"/tmp/\xFFbad".to_vec();
    let os = OsString::from_vec(bytes);
    let path = std::path::Path::new(&os);
    let err = topology_path_for_cli(path).expect_err("non-utf8 path");
    assert!(
        err.to_string().contains("invalid UTF-8") || err.to_string().contains("UTF-8"),
        "{err}"
    );
}

#[test]
fn collect_ips_skips_successful_output_without_192_168_ip() {
    let ips = collect_ips_for_vm_names(vec!["vm-z".to_string()], |_| {
        Ok(unix_output_ok(
            b" Name   MAC   Protocol   Address\n  vnet0  xx    ipv4       10.0.0.5/24\n",
        ))
    });
    assert!(ips.is_empty());
}

#[test]
fn collect_ips_includes_ip_when_status_successful() {
    let ips = collect_ips_for_vm_names(vec!["vm-ok".to_string()], |_| {
        Ok(unix_output_ok(b"ipv4         192.168.50.2/24\n"))
    });
    assert_eq!(ips, vec!["192.168.50.2".to_string()]);
}
