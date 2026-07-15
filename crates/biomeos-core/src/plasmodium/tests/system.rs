// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::system;

#[test]
fn test_system_ram() {
    // Just verify it doesn't panic
    let ram = system::get_system_ram_gb();
    assert!(ram > 0);
}

#[test]
fn test_num_cpus() {
    assert!(system::num_cpus() > 0);
}
