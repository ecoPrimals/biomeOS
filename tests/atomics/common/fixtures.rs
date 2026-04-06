// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

// Test fixtures for NUCLEUS atomic testing
//
// Deep Debt: Fast AND Safe - uses nix crate for safe POSIX syscalls

use std::path::PathBuf;
use std::sync::LazyLock;

/// Test configuration
pub struct TestConfig {
    pub family_id: String,
    pub node_id: String,
    pub runtime_dir: PathBuf,
}

impl TestConfig {
    pub fn default() -> Self {
        // Fast AND Safe: nix::unistd::getuid() is a safe wrapper around the syscall
        let uid = nix::unistd::getuid();
        Self {
            family_id: "test0".to_string(),
            node_id: "test-node".to_string(),
            runtime_dir: PathBuf::from(format!("/run/user/{}/biomeos", uid)),
        }
    }
}

/// Global test configuration
pub static TEST_CONFIG: LazyLock<TestConfig> = LazyLock::new(TestConfig::default);

/// Test timeouts
pub mod timeouts {
    use std::time::Duration;
    
    pub const SOCKET_CREATION: Duration = Duration::from_secs(5);
    pub const HEALTH_CHECK: Duration = Duration::from_millis(500);
    pub const PRIMAL_STARTUP: Duration = Duration::from_secs(10);
    pub const RECOVERY: Duration = Duration::from_secs(30);
}

/// Test socket paths
pub fn socket_path(primal: &str) -> PathBuf {
    TEST_CONFIG.runtime_dir.join(format!("{}.sock", primal))
}

/// Test data generators
pub mod generators {
    use rand::Rng;
    
    pub fn random_jwt_secret() -> String {
        let mut rng = rand::rng();
        let secret: [u8; 48] = rng.random();
        base64::encode(secret)
    }
    
    pub fn random_family_id() -> String {
        format!("test{}", rand::rng().random::<u16>())
    }
    
    pub fn random_node_id() -> String {
        format!("node{}", rand::rng().random::<u16>())
    }
}
