// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::collect_ips_for_vm_names;

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
