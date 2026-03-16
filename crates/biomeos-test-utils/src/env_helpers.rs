// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Environment variable helpers for testing.
//!
//! Rust 2024 edition makes `std::env::set_var` and `std::env::remove_var` unsafe
//! because they are not thread-safe. These helpers centralize the unsafe calls
//! with documented safety invariants, keeping `#![forbid(unsafe_code)]` on all
//! other crates.
//!
//! Production code should never mutate process-global env vars. Configuration
//! should flow through typed structs and `Command::env()` for child processes.

use std::ffi::OsStr;

/// Set an environment variable for testing.
///
/// # Safety contract
///
/// Callers must ensure no other thread is concurrently reading this env var
/// via `std::env::var`. In practice, tests that manipulate env vars should
/// use `#[serial_test::serial]` or equivalent serialization.
#[allow(unsafe_code)]
pub fn set_test_env<K: AsRef<OsStr>, V: AsRef<OsStr>>(key: K, value: V) {
    // SAFETY: Test-only function. Callers accept the thread-safety contract.
    unsafe { std::env::set_var(key, value) }
}

/// Remove an environment variable for testing.
///
/// See [`set_test_env`] for safety contract.
#[allow(unsafe_code)]
pub fn remove_test_env<K: AsRef<OsStr>>(key: K) {
    // SAFETY: Test-only function. Callers accept the thread-safety contract.
    unsafe { std::env::remove_var(key) }
}

/// RAII guard that restores an env var to its original value on drop.
///
/// Use this to avoid test pollution when tests run concurrently.
pub struct TestEnvGuard {
    key: String,
    original: Option<String>,
}

impl TestEnvGuard {
    /// Capture the current value and optionally set a new one.
    pub fn new(key: &str, new_value: Option<&str>) -> Self {
        let original = std::env::var(key).ok();
        match new_value {
            Some(v) => set_test_env(key, v),
            None => remove_test_env(key),
        }
        Self {
            key: key.to_string(),
            original,
        }
    }

    /// Set the env var and capture the original for restoration.
    pub fn set(key: &str, value: &str) -> Self {
        Self::new(key, Some(value))
    }

    /// Remove the env var and capture the original for restoration.
    pub fn remove(key: &str) -> Self {
        Self::new(key, None)
    }
}

impl Drop for TestEnvGuard {
    fn drop(&mut self) {
        match &self.original {
            Some(v) => set_test_env(&self.key, v),
            None => remove_test_env(&self.key),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_remove() {
        let key = "BIOMEOS_TEST_ENV_HELPER_UNIQUE_KEY";
        set_test_env(key, "hello");
        assert_eq!(std::env::var(key).unwrap(), "hello");
        remove_test_env(key);
        assert!(std::env::var(key).is_err());
    }

    #[test]
    fn test_guard_restores() {
        let key = "BIOMEOS_TEST_GUARD_UNIQUE_KEY";
        remove_test_env(key);

        {
            let _guard = TestEnvGuard::set(key, "temporary");
            assert_eq!(std::env::var(key).unwrap(), "temporary");
        }
        assert!(std::env::var(key).is_err());
    }
}
