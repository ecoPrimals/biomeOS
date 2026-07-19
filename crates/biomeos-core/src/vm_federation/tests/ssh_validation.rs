// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::time::{Duration, Instant};

use super::{ValidationConfig, validate_ssh_probe_output, wait_for_vm_ssh_ready};

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
